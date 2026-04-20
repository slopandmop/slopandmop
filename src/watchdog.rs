// Watchdog — runs inside the factory process. Every 60 s, scans the registry
// for rows whose `teardown_at < now()` and shells `gcloud compute instances
// delete` for each. On success, removes the row and atomic-writes the registry
// back to disk.
//
// Not a separate binary: `tokio::spawn(watchdog::run(registry))` from
// factory::run on startup.

use anyhow::Result;
use std::time::Duration;

pub async fn run() -> Result<()> {
    let mut interval = tokio::time::interval(Duration::from_secs(60));
    loop {
        interval.tick().await;
        // TODO: iterate registry, delete expired via gcloud::delete.
        tracing::debug!("watchdog tick (noop)");
    }
}
