//! Draws the dashboard: status bar, collection list, server totals, detail, latency, help.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Sparkline,
};
use ratatui::Frame;

use super::dashboard::{Dashboard, Row};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;

/// Renders one frame.
pub fn draw(frame: &mut Frame, dash: &Dashboard) {
    let [bar, body, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    let [left, right] =
        Layout::horizontal([Constraint::Length(36), Constraint::Min(24)]).areas(body);
    let [list, totals] = Layout::vertical([Constraint::Min(3), Constraint::Length(10)]).areas(left);
    let [detail, latency] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(9)]).areas(right);

    status_bar(frame, dash, bar);
    collections(frame, dash, list);
    server(frame, dash, totals);
    detail_pane(frame, dash, detail);
    latency_pane(frame, dash, latency);
    bottom_line(frame, dash, bottom);
    if dash.help {
        help(frame, frame.area());
    }
}

fn status_bar(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let reachable = dash.error.is_none() && dash.snapshot.is_some();
    let ready = dash.snapshot.as_ref().is_some_and(|s| s.ready.ok);
    let mut spans = vec![
        Span::styled(
            " piramid ",
            Style::default().fg(Color::Black).bg(ACCENT).bold(),
        ),
        Span::styled(format!(" {} ", dash.version), Style::default().fg(DIM)),
        dot("live", reachable, false),
        dot("ready", ready, reachable),
        Span::styled(
            format!("  {}", dash.client.base()),
            Style::default().fg(DIM),
        ),
    ];
    if let Some(at) = dash.last_refresh {
        spans.push(Span::styled(
            format!("  updated {}s ago", at.elapsed().as_secs()),
            Style::default().fg(DIM),
        ));
    }
    if let Some(error) = &dash.error {
        spans.push(Span::styled(
            format!("  {error}"),
            Style::default().fg(Color::Red),
        ));
    }
    if let Some(notice) = &dash.notice {
        spans.push(Span::styled(
            format!("  {notice}"),
            Style::default().fg(Color::Magenta),
        ));
    }
    frame.render_widget(Line::from(spans), area);
}

/// A status dot. `degraded` marks a "no" that is only a warning because the thing it depends on
/// is itself down — an unreachable server tells you nothing about readiness.
fn dot(name: &str, up: bool, degraded: bool) -> Span<'static> {
    let (glyph, color) = match (up, degraded) {
        (true, _) => ("●", Color::Green),
        (false, true) => ("·", DIM),
        (false, false) => ("○", Color::Red),
    };
    Span::styled(format!(" {glyph} {name}"), Style::default().fg(color))
}

fn collections(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let width = usize::from(area.width.saturating_sub(4)).max(12);
    let items: Vec<ListItem> = dash
        .rows
        .iter()
        .map(|row| {
            let count = thousands(row.vectors());
            let name_width = width.saturating_sub(count.len() + 3);
            let (glyph, color) = match (row.problem().is_some(), row.loaded()) {
                (true, _) => ("!", Color::Red),
                (false, true) => ("●", Color::Green),
                (false, false) => ("○", DIM),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {glyph} "), Style::default().fg(color)),
                Span::raw(format!("{:<name_width$}", truncate(&row.name, name_width))),
                Span::styled(count, Style::default().fg(DIM)),
            ]))
        })
        .collect();
    let title = format!(" collections · {} ", dash.rows.len());
    if items.is_empty() {
        let note = if dash.snapshot.is_some() {
            "  no collections in the data directory"
        } else {
            "  waiting for the server…"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(note, Style::default().fg(DIM))))
                .block(pane(&title, true)),
            area,
        );
        return;
    }
    let list = List::new(items).block(pane(&title, true)).highlight_style(
        Style::default()
            .bg(Color::Rgb(40, 44, 52))
            .add_modifier(Modifier::BOLD),
    );
    let mut state = ListState::default().with_selected(Some(dash.selected));
    frame.render_stateful_widget(list, area, &mut state);
}

fn server(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let Some(snapshot) = &dash.snapshot else {
        frame.render_widget(Paragraph::new("").block(pane(" server ", false)), area);
        return;
    };
    let disk = match (
        snapshot.ready.disk_available_bytes,
        snapshot.ready.disk_total_bytes,
    ) {
        (Some(free), Some(total)) => format!("{} / {}", bytes(free), bytes(total)),
        _ => "unknown".into(),
    };
    // The tail of a data directory is what distinguishes one from another, so it survives and
    // the leading path is what gets dropped.
    let data_dir = truncate_left(&snapshot.ready.data_dir, 17);
    let embedding = &snapshot.metrics.embedding;
    let rows = [
        // From the sidebar, not `metrics.total_collections`: metrics counts what the server has
        // open, and this line sitting under a list of four saying "1" reads as a bug.
        ("collections", thousands(dash.rows.len())),
        ("loaded", thousands(snapshot.ready.loaded_collections)),
        ("vectors", thousands(snapshot.metrics.total_vectors)),
        ("disk free", disk),
        ("data dir", data_dir),
        ("embed calls", thousands_u64(embedding.requests)),
        ("embed tokens", thousands_u64(embedding.total_tokens)),
        ("embed latency", millis(embedding.avg_latency_ms)),
    ];
    frame.render_widget(
        Paragraph::new(
            rows.iter()
                .map(|(k, v)| field(k, v, 15))
                .collect::<Vec<_>>(),
        )
        .block(pane(" server ", false)),
        area,
    );
}

fn detail_pane(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let Some(row) = dash.current() else {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  select a collection",
                Style::default().fg(DIM),
            )))
            .block(pane(" collection ", true)),
            area,
        );
        return;
    };
    let index = row
        .metrics
        .as_ref()
        .map_or("—", |m| m.index_type.as_str())
        .to_owned();
    let title = format!(
        " {} · {index} · {} vectors ",
        row.name,
        thousands(row.vectors())
    );
    frame.render_widget(
        Paragraph::new(detail_lines(row)).block(pane(&title, true)),
        area,
    );
}

fn detail_lines(row: &Row) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    if let Some(problem) = row.problem() {
        lines.push(Line::from(Span::styled(
            format!("  {problem}"),
            Style::default().fg(Color::Red).bold(),
        )));
        lines.push(Line::default());
    }
    let Some(metrics) = &row.metrics else {
        lines.push(Line::from(Span::styled(
            "  on disk but not open — the server loads a collection on first use",
            Style::default().fg(DIM),
        )));
        return lines;
    };
    lines.push(heading("index"));
    lines.push(field("type", &metrics.index_type, 18));
    if let Some(ef) = metrics.hnsw_ef_search {
        lines.push(field("ef_search", &ef.to_string(), 18));
    }
    if let Some(nprobe) = metrics.ivf_nprobe {
        lines.push(field("nprobe", &nprobe.to_string(), 18));
    }
    lines.push(field(
        "filter overfetch",
        &metrics
            .filter_overfetch
            .map_or_else(|| "—".to_owned(), |v| v.to_string()),
        18,
    ));
    lines.push(field(
        "memory",
        &bytes(metrics.memory_usage_bytes as u64),
        18,
    ));

    lines.push(Line::default());
    lines.push(heading("latency"));
    lines.push(field("search", &millis(metrics.search_latency_ms), 18));
    lines.push(field("insert", &millis(metrics.insert_latency_ms), 18));
    lines.push(field("lock read", &millis(metrics.lock_read_ms), 18));
    lines.push(field("lock write", &millis(metrics.lock_write_ms), 18));

    lines.push(Line::default());
    lines.push(heading("durability"));
    let (age, size) = row
        .wal
        .as_ref()
        .map_or((None, None), |w| (w.checkpoint_age_secs, w.wal_size_bytes));
    lines.push(field(
        "last checkpoint",
        &age.map_or_else(|| "never".to_owned(), |s| format!("{} ago", duration(s))),
        18,
    ));
    lines.push(field(
        "wal size",
        &size.map_or_else(|| "—".to_owned(), bytes),
        18,
    ));
    if let Some(version) = row.health.as_ref().and_then(|h| h.schema_version) {
        lines.push(field("schema", &format!("v{version}"), 18));
    }
    lines
}

fn latency_pane(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let history: Vec<u64> = dash
        .current()
        .and_then(|row| dash.history.get(&row.name))
        .map(|h| h.iter().copied().collect())
        .unwrap_or_default();
    let peak = history.iter().copied().max().unwrap_or(0);
    let title = if history.is_empty() {
        " search latency ".to_owned()
    } else {
        format!(" search latency · peak {} ", micros(peak))
    };
    let block = pane(&title, false);
    if history.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  nothing measured yet — the server reports latency once a search has run",
                Style::default().fg(DIM),
            )))
            .block(block),
            area,
        );
        return;
    }
    // Newest on the right, so the sparkline scrolls the way a chart does.
    let width = usize::from(area.width.saturating_sub(2)).max(1);
    let visible = &history[history.len().saturating_sub(width)..];
    frame.render_widget(
        Sparkline::default()
            .block(block)
            .data(visible)
            .style(Style::default().fg(ACCENT)),
        area,
    );
}

fn bottom_line(frame: &mut Frame, dash: &Dashboard, area: Rect) {
    let line = match &dash.pending {
        Some(pending) => Line::from(vec![
            Span::styled(
                " confirm ",
                Style::default().fg(Color::Black).bg(Color::Yellow),
            ),
            Span::styled(
                format!(" {}", pending.question()),
                Style::default().fg(Color::Yellow),
            ),
        ]),
        None => {
            let mut spans: Vec<Span> = Vec::new();
            for (key, what) in [
                ("j/k", "move"),
                ("r", "rebuild index"),
                ("c", "compact"),
                ("R", "refresh now"),
                ("?", "help"),
                ("q", "quit"),
            ] {
                spans.push(Span::styled(
                    format!(" {key}"),
                    Style::default().fg(ACCENT).bold(),
                ));
                spans.push(Span::styled(format!(" {what}"), Style::default().fg(DIM)));
            }
            Line::from(spans)
        }
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn help(frame: &mut Frame, area: Rect) {
    let keys = [
        ("j / k, ↑ / ↓", "move between collections"),
        ("gg / G", "first / last collection"),
        ("r", "rebuild the selected collection's index, after y/n"),
        ("c", "compact the selected collection, after y/n"),
        ("R", "refresh now instead of waiting for the interval"),
        ("q, ctrl-c", "quit"),
    ];
    let mut lines = vec![
        Line::from(Span::styled("keys", Style::default().fg(ACCENT).bold())),
        Line::default(),
    ];
    for (key, what) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<16}"), Style::default().fg(Color::Yellow)),
            Span::raw(what),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  the dashboard only reads, rebuilds and compacts. It never deletes: a collection is",
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(Span::styled(
        "  removed with `piramid` against the API, where the name has to be typed out.",
        Style::default().fg(DIM),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "any key closes this",
        Style::default().fg(DIM),
    )));

    let width = 78.min(area.width.saturating_sub(4));
    let height = u16::try_from(lines.len() + 2)
        .unwrap_or(u16::MAX)
        .min(area.height.saturating_sub(2));
    let popup = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, popup);
    frame.render_widget(
        Paragraph::new(lines)
            .block(pane(" help ", true))
            .alignment(Alignment::Left),
        popup,
    );
}

fn pane(title: &str, focused: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(if focused { ACCENT } else { DIM }))
        .title(Span::styled(
            title.to_owned(),
            Style::default().fg(Color::White).bold(),
        ))
}

fn heading(text: &str) -> Line<'static> {
    Line::from(Span::styled(
        format!("  {text}"),
        Style::default().fg(DIM).add_modifier(Modifier::BOLD),
    ))
}

fn field(key: &str, value: &str, width: usize) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("  {key:<width$}"), Style::default().fg(DIM)),
        Span::raw(value.to_owned()),
    ])
}

/// A count with thousands separators, so six figures can be read at a glance.
pub fn thousands(n: usize) -> String {
    thousands_u64(n as u64)
}

/// A `u64` count with thousands separators.
pub fn thousands_u64(n: u64) -> String {
    let digits = n.to_string();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3);
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(c);
    }
    out
}

/// A byte count at the largest unit that keeps it under four digits.
pub fn bytes(n: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KB", "MB", "GB", "TB"];
    let mut value = n as f64;
    let mut unit = 0;
    while value >= 1024.0 && unit + 1 < UNITS.len() {
        value /= 1024.0;
        unit += 1;
    }
    if unit == 0 {
        format!("{n} B")
    } else {
        format!("{value:.1} {}", UNITS[unit])
    }
}

/// A millisecond measurement, or an em dash when nothing has been measured.
pub fn millis(ms: Option<f32>) -> String {
    ms.map_or_else(|| "—".to_owned(), |v| format!("{v:.2} ms"))
}

/// A microsecond measurement rendered in milliseconds.
pub fn micros(us: u64) -> String {
    format!("{:.2} ms", us as f64 / 1000.0)
}

/// A span of seconds as the coarsest unit that still says something useful.
pub fn duration(secs: u64) -> String {
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}

/// Keeps the last `width` characters, marking what was dropped from the front.
pub fn truncate_left(s: &str, width: usize) -> String {
    let count = s.chars().count();
    if count <= width {
        return s.to_owned();
    }
    let mut out = String::from("…");
    out.extend(s.chars().skip(count - width.saturating_sub(1)));
    out
}

fn truncate(s: &str, width: usize) -> String {
    if s.chars().count() <= width {
        return s.to_owned();
    }
    let mut out: String = s.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}
