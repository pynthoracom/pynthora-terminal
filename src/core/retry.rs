use anyhow::Result;
use std::time::Duration;
use tokio::time::sleep;
use tracing::{debug, warn};

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {
    pub max_attempts: u32,
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub backoff_multiplier: f64,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_delay: Duration::from_millis(100),
            max_delay: Duration::from_secs(30),
            backoff_multiplier: 2.0,
        }
    }
}

/// Retry a function with exponential backoff
pub async fn retry_with_backoff<F, Fut, T, E>(
    config: &RetryConfig,
    mut f: F,
) -> Result<T>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = std::result::Result<T, E>>,
    E: std::fmt::Display,
{
    let mut delay = config.initial_delay;
    let mut last_error = None;

    for attempt in 1..=config.max_attempts {
        match f().await {
            Ok(value) => {
                if attempt > 1 {
                    debug!("Operation succeeded after {} attempts", attempt);
                }
                return Ok(value);
            }
            Err(e) => {
                last_error = Some(e.to_string());
                if attempt < config.max_attempts {
                    warn!(
                        "Attempt {} failed, retrying in {:?}...",
                        attempt, delay
                    );
                    sleep(delay).await;
                    delay = Duration::from_millis(
                        (delay.as_millis() as f64 * config.backoff_multiplier) as u64,
                    );
                    delay = delay.min(config.max_delay);
                } else {
                    warn!("All {} attempts failed", config.max_attempts);
                }
            }
        }
    }

    anyhow::bail!(
        "Operation failed after {} attempts. Last error: {}",
        config.max_attempts,
        last_error.unwrap_or_else(|| "Unknown error".to_string())
    )
}

/// Check if an error is retryable
pub fn is_retryable_error(error: &str) -> bool {
    let retryable_patterns = [
        "timeout",
        "connection",
        "network",
        "temporary",
        "503",
        "502",
        "504",
        "429", // Rate limit
    ];

    let error_lower = error.to_lowercase();
    retryable_patterns.iter().any(|pattern| error_lower.contains(pattern))
}

