//! Periodic probes of the server and the website.

use tokio::sync::mpsc::UnboundedSender;

use crate::console::settings::Settings;
use crate::console::types::{Event, Health, Probe};

/// What to probe and how often.
#[derive(Debug, Clone)]
pub struct Targets {
    live: String,
    ready: String,
    web: String,
    every: std::time::Duration,
}

impl Targets {
    /// Targets from settings.
    pub fn from_settings(settings: &Settings) -> Self {
        let base = settings.base_url.trim_end_matches('/');
        Self {
            live: format!("{base}/api/health"),
            ready: format!("{base}/api/readyz"),
            web: settings.web_url.clone(),
            every: settings.refresh.max(std::time::Duration::from_secs(1)),
        }
    }
}

/// Probes forever on the configured interval, sending each result to the UI.
///
/// The website probe is skipped where there is no checkout to serve one from.
pub async fn poll(targets: Targets, probe_web: bool, tx: UnboundedSender<Event>) {
    let Ok(http) = reqwest::Client::builder()
        .connect_timeout(std::time::Duration::from_secs(2))
        // Readiness opens every collection on disk and can be slow on a large data directory.
        .timeout(std::time::Duration::from_secs(8))
        .build()
    else {
        return;
    };
    loop {
        let (live, ready) = tokio::join!(probe(&http, &targets.live), probe(&http, &targets.ready));
        let web = if probe_web {
            probe(&http, &targets.web).await
        } else {
            Probe::Unknown
        };
        if tx
            .send(Event::Health(Box::new(Health { live, ready, web })))
            .is_err()
        {
            return;
        }
        tokio::time::sleep(targets.every).await;
    }
}

async fn probe(http: &reqwest::Client, url: &str) -> Probe {
    match http.get(url).send().await {
        Ok(response) if response.status().is_success() => Probe::Up,
        Ok(response) => {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            Probe::Degraded(format!(
                "{status}: {}",
                body.chars().take(120).collect::<String>()
            ))
        }
        Err(_) => Probe::Down,
    }
}
