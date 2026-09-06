//! The event loop: terminal setup, the tasks that feed it, and the draw cycle.

use std::path::PathBuf;
use std::time::Duration;

use crossterm::event::{Event as TermEvent, EventStream, KeyEventKind};
use futures::StreamExt;
use tokio::sync::mpsc::{self, UnboundedSender};

use crate::console::app::App;
use crate::console::runner::Runner;
use crate::console::settings::Settings;
use crate::console::types::Event;
use crate::console::{health, ui};

/// How often docker compose ps is re-read.
const SERVICES_INTERVAL: Duration = Duration::from_secs(3);

/// Runs the console over the repo at root.
pub fn run(root: PathBuf) -> std::io::Result<()> {
    let settings = Settings::from_env().map_err(std::io::Error::other)?;
    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async move {
        let (tx, mut rx) = mpsc::unbounded_channel();
        let mut app = App::new(settings.clone(), root.clone(), &tx)?;

        tokio::spawn(health::poll(
            health::Targets::from_settings(&settings),
            tx.clone(),
        ));
        tokio::spawn(services(root, tx.clone()));
        tokio::spawn(ticker(tx.clone()));
        tokio::spawn(keys(tx));

        // The terminal is restored before a panic message is printed.
        let original_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |info| {
            ratatui::restore();
            original_hook(info);
        }));
        let mut terminal = ratatui::init();
        let outcome = drive(&mut terminal, &mut app, &mut rx).await;
        app.shutdown();
        ratatui::restore();
        outcome
    })
}

async fn drive(
    terminal: &mut ratatui::DefaultTerminal,
    app: &mut App,
    rx: &mut mpsc::UnboundedReceiver<Event>,
) -> std::io::Result<()> {
    terminal.draw(|frame| ui::draw(frame, app))?;
    while let Some(event) = rx.recv().await {
        app.handle(event);
        // Everything else already queued is drained into one redraw.
        while let Ok(more) = rx.try_recv() {
            app.handle(more);
        }
        if app.should_quit {
            break;
        }
        terminal.draw(|frame| ui::draw(frame, app))?;
    }
    Ok(())
}

async fn services(root: PathBuf, tx: UnboundedSender<Event>) {
    loop {
        let states = Runner::service_states(&root).await;
        if tx.send(Event::Services(states)).is_err() {
            return;
        }
        tokio::time::sleep(SERVICES_INTERVAL).await;
    }
}

/// Redraws on a timer, advancing the elapsed-time counters while nothing else happens.
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
            // A lost keyboard is reported and shuts the console down.
            Err(e) => Event::InputLost(e.to_string()),
        };
        if tx.send(mapped).is_err() {
            return;
        }
    }
}
