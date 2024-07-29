use std::time::Duration;

use serde::{Deserialize, Serialize};

const INITIAL_RETRY_NUMBER: u32 = 1;

#[derive(Debug)]
pub struct Retry {
    number: u32,
    delay: Duration,
}

impl Retry {
    pub fn new(number: u32, delay: Duration) -> Self {
        Retry { number, delay }
    }

    pub fn number(&self) -> u32 {
        self.number
    }

    pub fn delay(&self) -> Duration {
        self.delay
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum RetryStrategy {
    #[serde(rename = "exponential")]
    Exponential {
        initial_delay: Duration,
        maximum_delay: Duration,
        multiplier: u32,
    },
}

impl RetryStrategy {
    pub fn next_retry(&self, previous_retry: Option<Retry>) -> Retry {
        match self {
            RetryStrategy::Exponential {
                initial_delay,
                maximum_delay,
                multiplier,
            } => match previous_retry {
                None => Retry::new(INITIAL_RETRY_NUMBER, *initial_delay),
                Some(previous) => {
                    let number = if let Some(n) = previous.number.checked_add(1) {
                        n
                    } else {
                        log::warn!("Exponential retry number overflow, resetting");
                        0
                    };
                    let duration = match previous.delay.checked_mul(*multiplier) {
                        Some(d) if d <= *maximum_delay => d,
                        _ => *maximum_delay,
                    };
                    Retry::new(number, duration)
                }
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use std::ops::Mul;
    use std::time::Duration;

    use crate::retry::{Retry, RetryStrategy, INITIAL_RETRY_NUMBER};

    #[test]
    fn exponential_next_retry_no_previous() {
        let initial_delay = Duration::from_millis(2);
        let retry_strategy = RetryStrategy::Exponential {
            initial_delay,
            maximum_delay: Duration::from_millis(20),
            multiplier: 5,
        };

        let result = retry_strategy.next_retry(None);

        assert_eq!(result.number, INITIAL_RETRY_NUMBER);
        assert_eq!(result.delay, initial_delay);
    }

    #[test]
    fn exponential_next_retry_previous_middle_delay() {
        let initial_delay = Duration::from_millis(2);
        let multiplier = 5;
        let retry_strategy = RetryStrategy::Exponential {
            initial_delay,
            maximum_delay: Duration::from_millis(20),
            multiplier,
        };
        let number = INITIAL_RETRY_NUMBER;
        let delay = initial_delay;
        let retry = Retry { number, delay };

        let result = retry_strategy.next_retry(Some(retry));

        assert_eq!(result.number, number + 1);
        assert_eq!(result.delay, delay.mul(multiplier));
    }

    #[test]
    fn exponential_next_retry_previous_maximum_delay() {
        let initial_delay = Duration::from_millis(2);
        let maximum_delay = Duration::from_millis(20);
        let multiplier = 5;
        let retry_strategy = RetryStrategy::Exponential {
            initial_delay,
            maximum_delay,
            multiplier,
        };
        let number = INITIAL_RETRY_NUMBER + 1;
        let retry = Retry {
            number,
            delay: initial_delay.mul(multiplier),
        };

        let result = retry_strategy.next_retry(Some(retry));

        assert_eq!(result.number, number + 1);
        assert_eq!(result.delay, maximum_delay);
    }
}
