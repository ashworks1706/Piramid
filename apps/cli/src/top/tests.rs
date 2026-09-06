//! Unit tests for the parts of the dashboard that are not drawing.

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::client::{
    Client, CollectionHealth, CollectionMetrics, Metrics, Readyz, Snapshot, WalStats,
};
use super::dashboard::{Dashboard, Event, Pending};
use super::ui::{bytes, duration, millis, thousands, truncate_left};

fn dashboard() -> Dashboard {
    let client = Client::new("http://localhost:6333", std::time::Duration::from_secs(1))
        .expect("a client over a static URL is always constructible");
    Dashboard::new(client, std::time::Duration::from_secs(2))
}

fn press(key: char) -> Event {
    Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE))
}

fn snapshot(names: &[&str], open: &[&str]) -> Snapshot {
    Snapshot {
        metrics: Metrics {
            total_collections: open.len(),
            collections: open
                .iter()
                .map(|name| CollectionMetrics {
                    name: (*name).to_owned(),
                    vector_count: 10,
                    index_type: "hnsw".into(),
                    search_latency_ms: Some(1.5),
                    ..CollectionMetrics::default()
                })
                .collect(),
            ..Metrics::default()
        },
        ready: Readyz {
            ok: true,
            collections: names
                .iter()
                .map(|name| CollectionHealth {
                    name: (*name).to_owned(),
                    loaded: open.contains(name),
                    integrity_ok: true,
                    ..CollectionHealth::default()
                })
                .collect(),
            ..Readyz::default()
        },
    }
}

#[test]
fn rows_come_from_readiness_so_an_unopened_collection_is_still_listed() {
    let mut dash = dashboard();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["docs", "notes"],
        &["docs"],
    )))));

    let names: Vec<&str> = dash.rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["docs", "notes"]);
    assert!(dash.rows[0].loaded());
    // A collection listed from the data directory but never opened has no counters.
    assert!(!dash.rows[1].loaded());
    assert_eq!(dash.rows[1].vectors(), 0);
}

#[test]
fn a_wal_stat_keyed_by_something_else_is_dropped_rather_than_listed() {
    let mut dash = dashboard();
    let mut snap = snapshot(&["docs"], &["docs"]);
    snap.metrics.wal_stats = vec![
        WalStats {
            collection: "docs".into(),
            wal_size_bytes: Some(8596),
            ..WalStats::default()
        },
        // A durability stat keyed by something other than a collection name is dropped.
        WalStats {
            collection: "/var/lib/piramid/docs.db".into(),
            wal_size_bytes: Some(8596),
            ..WalStats::default()
        },
    ];
    dash.handle(Event::Snapshot(Box::new(Ok(snap))));

    let names: Vec<&str> = dash.rows.iter().map(|r| r.name.as_str()).collect();
    assert_eq!(names, ["docs"]);
    assert_eq!(
        dash.rows[0].wal.as_ref().and_then(|w| w.wal_size_bytes),
        Some(8596)
    );
}

#[test]
fn the_cursor_stays_on_its_collection_when_the_list_changes() {
    let mut dash = dashboard();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["a", "b", "c"],
        &["a", "b", "c"],
    )))));
    dash.handle(press('j'));
    dash.handle(press('j'));
    assert_eq!(dash.current().map(|r| r.name.as_str()), Some("c"));

    // The first collection is dropped, so the old index would name a different collection.
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["b", "c"],
        &["b", "c"],
    )))));
    assert_eq!(dash.current().map(|r| r.name.as_str()), Some("c"));
}

#[test]
fn a_failed_refresh_keeps_the_last_good_numbers_on_screen() {
    let mut dash = dashboard();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["docs"],
        &["docs"],
    )))));
    dash.handle(Event::Snapshot(Box::new(Err(
        super::client::ClientError::Unreachable("/api/metrics".into(), "refused".into()),
    ))));

    assert!(dash.error.is_some());
    assert_eq!(dash.rows.len(), 1);
    assert!(dash.snapshot.is_some());
}

#[test]
fn mutating_keys_ask_before_they_act() {
    let mut dash = dashboard();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["docs"],
        &["docs"],
    )))));

    assert!(dash.handle(press('r')).is_none());
    assert_eq!(dash.pending, Some(Pending::Rebuild("docs".into())));

    // Any key other than the confirm key cancels.
    assert!(dash.handle(press('n')).is_none());
    assert_eq!(dash.pending, None);

    dash.handle(press('c'));
    assert_eq!(dash.pending, Some(Pending::Compact("docs".into())));
    assert_eq!(
        dash.handle(press('y')),
        Some(Pending::Compact("docs".into()))
    );
    assert_eq!(dash.pending, None);
}

#[test]
fn an_action_on_an_empty_list_is_refused_rather_than_sent() {
    let mut dash = dashboard();
    assert!(dash.handle(press('r')).is_none());
    assert_eq!(dash.pending, None);
    assert!(dash.notice.is_some());
}

#[test]
fn latency_history_accumulates_per_collection() {
    let mut dash = dashboard();
    for _ in 0..3 {
        dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
            &["docs"],
            &["docs"],
        )))));
    }
    assert_eq!(dash.history.get("docs").map(VecLen::len), Some(3));
}

trait VecLen {
    fn len(&self) -> usize;
}

impl VecLen for std::collections::VecDeque<u64> {
    fn len(&self) -> usize {
        std::collections::VecDeque::len(self)
    }
}

#[test]
fn quit_keys_end_the_loop() {
    let mut dash = dashboard();
    dash.handle(press('q'));
    assert!(dash.should_quit);

    let mut dash = dashboard();
    dash.handle(Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
    assert!(dash.should_quit);
}

#[test]
fn help_swallows_the_next_key_rather_than_acting_on_it() {
    let mut dash = dashboard();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["docs"],
        &["docs"],
    )))));
    dash.handle(press('?'));
    assert!(dash.help);

    // The key that closes help must not also start a rebuild.
    assert!(dash.handle(press('r')).is_none());
    assert!(!dash.help);
    assert_eq!(dash.pending, None);
}

#[test]
fn numbers_render_the_way_an_operator_reads_them() {
    assert_eq!(thousands(0), "0");
    assert_eq!(thousands(999), "999");
    assert_eq!(thousands(12_430), "12,430");
    assert_eq!(thousands(1_000_000), "1,000,000");

    assert_eq!(bytes(512), "512 B");
    assert_eq!(bytes(1024), "1.0 KB");
    assert_eq!(bytes(43_000_000), "41.0 MB");

    assert_eq!(millis(None), "—");
    assert_eq!(millis(Some(1.234)), "1.23 ms");

    assert_eq!(duration(45), "45s");
    assert_eq!(duration(90), "1m");
    assert_eq!(duration(7200), "2h");
    assert_eq!(duration(180_000), "2d");

    // A data directory is truncated from the left, keeping its tail.
    assert_eq!(truncate_left("/var/lib/piramid", 19), "/var/lib/piramid");
    assert_eq!(
        truncate_left("/home/ash/projects/piramid/data", 19),
        "…jects/piramid/data"
    );
}

#[test]
fn a_full_frame_renders_every_pane() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let mut dash = dashboard();
    dash.version = "v0.2.0".into();
    dash.handle(Event::Snapshot(Box::new(Ok(snapshot(
        &["docs", "notes"],
        &["docs"],
    )))));

    let mut terminal = Terminal::new(TestBackend::new(100, 30))
        .expect("the test backend cannot fail to initialize");
    terminal
        .draw(|frame| super::ui::draw(frame, &dash))
        .expect("drawing into the test backend cannot fail");

    let rendered: String = terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();

    for expected in [
        "piramid",
        "v0.2.0",
        "collections",
        "docs",
        "notes",
        "server",
        "hnsw",
        "search",
        "rebuild index",
    ] {
        assert!(
            rendered.contains(expected),
            "the frame does not mention {expected:?}"
        );
    }
}

#[test]
fn the_frame_renders_before_the_first_snapshot_lands() {
    use ratatui::backend::TestBackend;
    use ratatui::Terminal;

    let dash = dashboard();
    let mut terminal = Terminal::new(TestBackend::new(100, 30))
        .expect("the test backend cannot fail to initialize");

    // An empty dashboard draws on a terminal narrower than its content.
    terminal
        .draw(|frame| super::ui::draw(frame, &dash))
        .expect("drawing an empty dashboard cannot fail");
    let mut narrow = Terminal::new(TestBackend::new(40, 12))
        .expect("the test backend cannot fail to initialize");
    narrow
        .draw(|frame| super::ui::draw(frame, &dash))
        .expect("drawing into a small terminal cannot fail");
}
