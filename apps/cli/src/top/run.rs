//! The event loop: terminal setup, the tasks that feed it, and the draw cycle.

use std::time::Duration;

use crossterm::event::{Event as TermEvent, EventStream, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc::{self, UnboundedSender};

use super::client::Client;
use super::dashboard::{verb, Dashboard, Event, Pending};
use super::ui;

/// How long a poll may take before it is abandoned and reported.
///
/// Readiness opens every collection on disk, so this is generous; what it protects against is a
/// hung server, not a slow one.
const REQUEST_TIMEOUT: Duration = Duration::from_secs(15);

/// Runs the dashboard against `base_url` until the operator quits.
pub fn run(base_url: &str, interval: Duration) -> std::io::Result<()> {
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let client = Client::new(base_url, REQUEST_TIMEOUT).map_err(std::io::Error::other)?;
        // One request before the terminal is taken over, so an unreachable server is an ordinary
        // error message rather than an empty dashboard the operator has to quit out of to read.
        let version = client.version().await.map_err(std::io::Error::other)?;
        let mut dashboard = Dashboard::new(client, interval);
        dashboard.version = match version.git_commit {
            Some(commit) => format!("v{} ({commit})", version.version),
            None => format!("v{}", version.version),
        };

        let (tx, mut rx) = mpsc::unbounded_channel();
        tokio::spawn(ticker(tx.clone()));
        tokio::spawn(keys(tx.clone()));

        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            ratatui::restore();
            original_hook(info);
        }));
        let mut terminal = ratatui::init();
        let outcome = drive(&mut terminal, &mut dashboard, &mut rx, &tx).await;
        ratatui::restore();
        outcome
    })
}

async fn drive(
    terminal: &mut ratatui::DefaultTerminal,
    dashboard: &mut Dashboard,
    rx: &mut mpsc::UnboundedReceiver<Event>,
    tx: &UnboundedSender<Event>,
) -> std::io::Result<()> {
    refresh(dashboard, tx);
    terminal.draw(|frame| ui::draw(frame, dashboard))?;
    while let Some(event) = rx.recv().await {
        dispatch(dashboard, tx, event);
        // Drain whatever else is queued so a burst costs one redraw.
        while let Ok(more) = rx.try_recv() {
            dispatch(dashboard, tx, more);
        }
        if dashboard.should_quit {
            break;
        }
        if dashboard.refresh_due() {
            refresh(dashboard, tx);
        }
        terminal.draw(|frame| ui::draw(frame, dashboard))?;
    }
    Ok(())
}

fn dispatch(dashboard: &mut Dashboard, tx: &UnboundedSender<Event>, event: Event) {
    if let Some(pending) = dashboard.handle(event) {
        act(dashboard, tx, pending);
    }
}

fn refresh(dashboard: &mut Dashboard, tx: &UnboundedSender<Event>) {
    dashboard.refreshing = true;
    let client = dashboard.client.clone();
    let tx = tx.clone();
    tokio::spawn(async move {
        let snapshot = client.snapshot().await;
        let _ = tx.send(Event::Snapshot(Box::new(snapshot)));
    });
}

/// Runs a confirmed action, then reports what it did.
///
/// A rebuild is accepted and then runs on the server, so the outcome is read back from the
/// status endpoint rather than inferred from the acceptance.
fn act(dashboard: &Dashboard, tx: &UnboundedSender<Event>, pending: Pending) {
    let client = dashboard.client.clone();
    let tx = tx.clone();
    let name = verb(&pending);
    tokio::spawn(async move {
        let result = match &pending {
            Pending::Rebuild(collection) => match client.rebuild(collection).await {
                Ok(()) => match client.rebuild_status(collection).await {
                    Ok(status) => match status.error {
                        Some(error) => Err(format!("{name} failed: {error}")),
                        None => Ok(match status.elapsed_ms {
                            Some(ms) => format!("{name} {} in {ms:.0} ms", status.status),
                            None => format!("{name} {}", status.status),
                        }),
                    },
                    Err(e) => Err(format!("{name} started, status unknown: {e}")),
                },
                Err(e) => Err(format!("{name} failed: {e}")),
            },
            Pending::Compact(collection) => match client.compact(collection).await {
                Ok(()) => Ok(format!("{name} done")),
                Err(e) => Err(format!("{name} failed: {e}")),
            },
        };
        let _ = tx.send(Event::Action(result));
    });
}

/// Redraws on a timer, so "updated Ns ago" and the refresh interval both advance while nothing
/// else is happening.
async fn ticker(tx: UnboundedSender<Event>) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));
    loop {
        interval.tick().await;
        if tx.send(Event::Tick).is_err() {
            return;
        }
    }
}

async fn keys(tx: UnboundedSender<Event>) {
    let mut stream = EventStream::new();
    while let Some(event) = stream.next().await {
        let mapped = match event {
            Ok(TermEvent::Key(key)) if key.kind == KeyEventKind::Press => Event::Key(key),
            Ok(TermEvent::Resize(_, _)) => Event::Resize,
            Ok(_) => continue,
            // Losing the keyboard leaves a dashboard that redraws but cannot be quit, so it says
            // why and shuts down rather than sitting there.
            Err(e) => Event::InputLost(e.to_string()),
        };
        if tx.send(mapped).is_err() {
            return;
        }
    }
}
