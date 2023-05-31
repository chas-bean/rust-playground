use std::{collections::HashMap, time::Duration};
use time::{macros::datetime, OffsetDateTime};

type UserId = String;
type Minute = i64;

struct FixedWindowRateLimiter {
    capacity: u64,
    requests: HashMap<UserId, HashMap<Minute, u64>>,
}

impl FixedWindowRateLimiter {
    pub fn new(capacity: u64) -> Self {
        Self {
            capacity,
            requests: HashMap::new(),
        }
    }

    pub fn allow(&mut self, key: &str, timestamp: OffsetDateTime) -> Result<String, String> {
        let unix_minutes = timestamp.unix_timestamp() / 60;

        match self.requests.get_mut(key) {
            Some(history) => match history.get_mut(&unix_minutes) {
                Some(count) => {
                    if *count >= self.capacity {
                        return Err(format!(
                            "User {key} has reached rate limit {} with {count} requests",
                            self.capacity
                        ));
                    } else {
                        *count += 1;
                        Ok(format!("Hello {key}"))
                    }
                }
                None => {
                    history.insert(unix_minutes, 1);
                    Ok(format!("Hello {key}"))
                }
            },
            None => {
                let mut history = HashMap::new();

                history.insert(unix_minutes, 1);
                self.requests.insert(key.to_string(), history);

                Ok(format!("Hello {key}"))
            }
        }
    }
}

#[test]
fn allows_per_key() {
    let mut rate_limiter = FixedWindowRateLimiter::new(1);

    let now = datetime!(2023-01-01 0:00:00 UTC);

    let a = rate_limiter.allow("billy", now).unwrap();
    let c = rate_limiter.allow("tom", now).unwrap();

    assert_eq!(a, "Hello billy");
    assert_eq!(c, "Hello tom");
}

#[test]
fn allows_per_duration() {
    let mut rate_limiter = FixedWindowRateLimiter::new(1);

    let now = datetime!(2023-01-01 0:00:00 UTC);
    let future = datetime!(2023-01-01 0:01:00 UTC);

    rate_limiter.allow("billy", now).unwrap();
    rate_limiter.allow("billy", future).unwrap();
}

#[test]
fn enforces_per_duration() {
    let mut rate_limiter = FixedWindowRateLimiter::new(1);

    let now = datetime!(2023-01-01 0:00:00 UTC);
    let future = datetime!(2023-01-01 0:00:59 UTC);

    rate_limiter.allow("billy", now).unwrap();

    let prevented = rate_limiter.allow("billy", future);

    assert!(prevented.is_err());
}

#[test]
fn enforces_at_scale() {
    let mut rate_limiter = FixedWindowRateLimiter::new(1_000);

    let now = datetime!(2023-01-01 0:00:00 UTC);

    for _ in 0..1_000 {
        rate_limiter
            .allow("billy", now)
            .expect("Up to 1,000 requests per minute per user");

        rate_limiter
            .allow("tom", now)
            .expect("Up to 1,000 requests per minute per user");
    }

    let prevented = rate_limiter.allow("billy", now);

    assert!(prevented.is_err());
}
