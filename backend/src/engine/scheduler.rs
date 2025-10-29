use tokio::time::{interval, Duration};
use tracing::info;

pub struct Scheduler {
    interval_secs: u64,
}

impl Scheduler {
    pub fn new(interval_secs: u64) -> Self {
        Self { interval_secs }
    }

    pub async fn run<F>(&self, mut task: F) -> Result<(), anyhow::Error>
    where
        F: FnMut() -> Result<(), anyhow::Error>,
    {
        let mut ticker = interval(Duration::from_secs(self.interval_secs));

        loop {
            ticker.tick().await;
            info!("Scheduler tick");
            if let Err(e) = task() {
                tracing::error!("Scheduler task error: {}", e);
            }
        }
    }
}
