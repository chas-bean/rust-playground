use std::io::Write;
use std::thread;

use clap::Command;
use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    FromSample, Sample, SampleRate, SizedSample,
};
use crossbeam::channel::bounded;
use eframe::egui;
use rust_playground::{waveform, wavetable};

enum MainThreadMessage {
    Sample(f64),
}

enum AudioThreadMessage {
    RequestSample,
}

#[derive(Default)]
struct App {
    frequency: f64,
}

impl App {
    fn new(cc: &eframe::CreationContext<'_>) -> Self {
        // Customize egui here with cc.egui_ctx.set_fonts and cc.egui_ctx.set_visuals.
        // Restore app state using cc.storage (requires the "persistence" feature).
        // Use the cc.gl (a glow::Context) to create graphics shaders and buffers that you can use
        // for e.g. egui::PaintCallback.
        Self::default()
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Hello World!");
            ui.add(egui::Slider::new(&mut self.frequency, 0.0..=120.0).text("frequency"));
        });
    }
}

fn main() -> Result<(), String> {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No default audio device active on host");

    let config = device
        .default_output_config()
        .expect("No default audio output device config");

    let handle = thread::spawn(move || {
        match config.sample_format() {
            cpal::SampleFormat::I8 => run::<i8>(&device, &config.into()),
            cpal::SampleFormat::I16 => run::<i16>(&device, &config.into()),
            // cpal::SampleFormat::I24 => run::<I24>(&device, &config.into()),
            cpal::SampleFormat::I32 => run::<i32>(&device, &config.into()),
            // cpal::SampleFormat::I48 => run::<I48>(&device, &config.into()),
            cpal::SampleFormat::I64 => run::<i64>(&device, &config.into()),
            cpal::SampleFormat::U8 => run::<u8>(&device, &config.into()),
            cpal::SampleFormat::U16 => run::<u16>(&device, &config.into()),
            // cpal::SampleFormat::U24 => run::<U24>(&device, &config.into()),
            cpal::SampleFormat::U32 => run::<u32>(&device, &config.into()),
            // cpal::SampleFormat::U48 => run::<U48>(&device, &config.into()),
            cpal::SampleFormat::U64 => run::<u64>(&device, &config.into()),
            cpal::SampleFormat::F32 => run::<f32>(&device, &config.into()),
            cpal::SampleFormat::F64 => run::<f64>(&device, &config.into()),
            sample_format => panic!("Unsupported sample format '{sample_format}'"),
        }
    });

    // let native_options = eframe::NativeOptions::default();

    // eframe::run_native(
    //     "My egui App",
    //     native_options,
    //     Box::new(|cc| Box::new(App::new(cc))),
    // )
    // .unwrap();

    // handle.join().unwrap();

    loop {
        let line = readline()?;
        let line = line.trim();

        if line.is_empty() {
            continue;
        }

        match respond(line) {
            Ok(quit) => {
                if quit {
                    break Ok(());
                }
            }
            Err(err) => {
                write!(std::io::stdout(), "{err}").map_err(|e| e.to_string())?;
                std::io::stdout().flush().map_err(|e| e.to_string())?;
            }
        }
    }
}

pub fn run<T>(device: &cpal::Device, config: &cpal::StreamConfig) -> !
where
    T: SizedSample + FromSample<f32>,
{
    let channels = config.channels as usize;
    let SampleRate(sample_rate) = config.sample_rate;

    let (main_producer, audio_consumer) = bounded(1_000); // @note: Could also use a zero-size?
    let (audio_producer, main_consumer) = bounded(1_000);

    let mut saw_table = wavetable::Wavetable::new(64);

    saw_table.fill(waveform::sawtooth);

    let mut wavetable_iter_220 = saw_table.iter(220.0_f64, sample_rate as f64);

    let mut next_sample = move || {
        audio_producer
            .send(AudioThreadMessage::RequestSample)
            .unwrap();

        match audio_consumer.recv().unwrap() {
            MainThreadMessage::Sample(sample) => sample.to_sample(),
        }
    };

    let stream = device
        .build_output_stream(
            config,
            move |data: &mut [T], _: &cpal::OutputCallbackInfo| {
                write_data(data, channels, &mut next_sample) // @note: Separate thread
            },
            move |err| println!("Stream error: {}", err),
            None,
        )
        .expect("Stream built based on proper config");

    stream.play().expect("Stream is played");

    loop {
        match main_consumer.recv().unwrap() {
            AudioThreadMessage::RequestSample => {
                let sample = wavetable_iter_220.next().unwrap() * 0.1;

                main_producer
                    .send(MainThreadMessage::Sample(sample))
                    .unwrap();
            }
        }
    }
}

fn write_data<T>(output: &mut [T], channels: usize, next_sample: &mut dyn FnMut() -> f32)
where
    T: Sample + FromSample<f32>,
{
    for frame in output.chunks_mut(channels) {
        let value: T = T::from_sample(next_sample());
        for sample in frame.iter_mut() {
            *sample = value;
        }
    }
}

fn respond(line: &str) -> Result<bool, String> {
    let args = line.split('\n');
    let matches = cli()
        .try_get_matches_from(args)
        .map_err(|e| e.to_string())?;

    match matches.subcommand() {
        Some(("ping", _matches)) => {
            write!(std::io::stdout(), "Pong").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
        }
        Some(("quit", _matches)) => {
            write!(std::io::stdout(), "Exiting ...").map_err(|e| e.to_string())?;
            std::io::stdout().flush().map_err(|e| e.to_string())?;
            return Ok(true);
        }
        Some((name, _matches)) => unimplemented!("{name}"),
        None => unreachable!("subcommand required"),
    }

    Ok(false)
}

fn readline() -> Result<String, String> {
    write!(std::io::stdout(), "$ ").map_err(|e| e.to_string())?;
    std::io::stdout().flush().map_err(|e| e.to_string())?;
    let mut buffer = String::new();
    std::io::stdin()
        .read_line(&mut buffer)
        .map_err(|e| e.to_string())?;
    Ok(buffer)
}

fn cli() -> Command {
    // strip out usage
    const PARSER_TEMPLATE: &str = "\
        {all-args}
    ";
    // strip out name/version
    const APPLET_TEMPLATE: &str = "\
        {about-with-newline}\n\
        {usage-heading}\n    {usage}\n\
        \n\
        {all-args}{after-help}\
    ";

    Command::new("repl")
        .multicall(true)
        .arg_required_else_help(true)
        .subcommand_required(true)
        .subcommand_value_name("APPLET")
        .subcommand_help_heading("APPLETS")
        .help_template(PARSER_TEMPLATE)
        .subcommand(
            Command::new("ping")
                .about("Get a response")
                .help_template(APPLET_TEMPLATE),
        )
        .subcommand(
            Command::new("quit")
                .alias("exit")
                .about("Quit the REPL")
                .help_template(APPLET_TEMPLATE),
        )
}
