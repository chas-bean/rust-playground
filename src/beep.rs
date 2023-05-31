use cpal::{
    traits::{DeviceTrait, HostTrait, StreamTrait},
    Sample, SampleRate,
};
use std::{
    f32::{self, consts::PI},
    thread,
    time::Duration,
};

///
/// @todo: Should this be static const?
///
fn sine_table(size: usize) -> Vec<f32> {
    let mut sine_table: Vec<f32> = Vec::with_capacity(size);

    // sine(0/2pi), sine(1/2pi), sine(2/2pi), sine(3/2pi)
    for i in 0..size {
        let phase = 2.0 * PI * (i as f32 / size as f32);
        let value = phase.sin();
        sine_table.push(value);
    }

    sine_table
}

fn main() {
    let host = cpal::default_host();

    let device = host
        .default_output_device()
        .expect("No default audio device active on host");

    let config = device
        .default_output_config()
        .expect("No default audio output device config");

    let channels = config.channels() as usize;
    let SampleRate(sample_rate) = config.sample_rate();

    // let size = 64;

    // let mut wavetable: Vec<f32> = Vec::with_capacity(size);

    // for i in 0..size {
    //     let sample = (2.0f32 * PI * i as f32 / size as f32).sin();
    //     wavetable[i] = sample;
    // }

    // // let sine_table: Vec<f32> = sine_table(sample_rate as usize);

    // // let mut frequency = 440f32;
    // // let phase_increment = sample_rate as f32 / frequency;

    // // loop {
    // //     let current_sample = sine_table[phase_accumulator.floor() as usize];

    // //     dbg!(current_sample.clone());

    // //     phase_accumulator = (phase_accumulator + phase_increment) % sample_rate;
    // // }

    // let mut phase_accumulator = 0f32;
    // let mut sample_clock = 0f32;

    // let stream = device
    //     .build_output_stream(
    //         &config.into(),
    //         move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
    //             for frame in data.chunks_mut(channels) {
    //                 for sample in frame.iter_mut() {
    //                     // phase_accumulator = (phase_accumulator + phase_increment);

    //                     // let current_sample = sine_table[phase_accumulator as usize];

    //                     // *sample = current_sample.to_sample();

    //                     sample_clock = (sample_clock + 1.0) % sample_rate as f32;
    //                     // frequency = (frequency + 0.001f32) % 200f32;

    //                     let value = (sample_clock * frequency * 2.0 * std::f32::consts::PI
    //                         / sample_rate as f32)
    //                         .sin();

    //                     *sample = value.to_sample();

    //                     // dbg!(
    //                     //     *sample,
    //                     //     sample_clock,
    //                     //     current_sample,
    //                     //     phase_accumulator as usize
    //                     // );

    //                     // thread::sleep(Duration::from_millis(1000));
    //                 }
    //             }
    //         },
    //         move |err| println!("{}", err),
    //         None,
    //     )
    //     .expect("Stream built based on proper config");

    // stream.play().expect("Stream is played");

    // loop {}
}
