use std::time::Duration;

#[derive(Debug, Clone)]
pub struct BackoffPolicy {
    pub initial_delay: Duration,
    pub max_delay: Duration,
    pub multiplier: f64,
    pub max_attempts: u32,
}

impl Default for BackoffPolicy {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_millis(500),
            max_delay: Duration::from_secs(10),
            multiplier: 2.0,
            max_attempts: 5,
        }
    }
}

impl BackoffPolicy {
    pub fn delay_for_attempt(&self, attempt: u32, jitter_factor: f64) -> Duration {
        if attempt >= self.max_attempts {
            return self.max_delay;
        }
        let exp = self.initial_delay.as_secs_f64() * self.multiplier.powi(attempt as i32);
        let clamped = exp.min(self.max_delay.as_secs_f64());
        let jittered = clamped * (0.8 + 0.4 * jitter_factor.clamp(0.0, 1.0));
        Duration::from_secs_f64(jittered)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_backoff_increases_and_caps() {
        let policy = BackoffPolicy::default();
        let d0 = policy.delay_for_attempt(0, 0.5);
        let d1 = policy.delay_for_attempt(1, 0.5);
        let d10 = policy.delay_for_attempt(10, 0.5);

        assert!(d1 > d0);
        assert!(d10 <= Duration::from_secs_f64(12.0)); // bounded by max_delay + jitter
    }
}
