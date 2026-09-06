//! Draws the console: status bar, unit list, log pane, command line, help.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph, Sparkline,
};
use ratatui::Frame;

use crate::console::app::{App, UnitState};
use crate::console::collections::Row;
use crate::console::types::{Focus, Group, Mode, Probe, Profile, Status, Stream, View};

const ACCENT: Color = Color::Cyan;
const DIM: Color = Color::DarkGray;
const SIDEBAR: u16 = 34;

/// Renders one frame.
pub fn draw(frame: &mut Frame, app: &mut App) {
    let [bar, body, bottom] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(3),
        Constraint::Length(1),
    ])
    .areas(frame.area());
    status_bar(frame, app, bar);
    match app.view {
        View::Units => {
            let [side, right] =
                Layout::horizontal([Constraint::Length(SIDEBAR), Constraint::Min(20)]).areas(body);
            sidebar(frame, app, side);
            logs(frame, app, right);
        }
        View::Collections => collections(frame, app, body),
        View::Config => config(frame, app, body),
    }
    bottom_line(frame, app, bottom);
    if app.help {
        help(frame, app, frame.area());
    }
}

/// The collection list beside the detail of the selected one.
fn collections(frame: &mut Frame, app: &App, area: Rect) {
    let [left, right] =
        Layout::horizontal([Constraint::Length(SIDEBAR), Constraint::Min(24)]).areas(area);
    let view = &app.collections;

    let width = usize::from(left.width.saturating_sub(4)).max(12);
    let items: Vec<ListItem> = view
        .rows
        .iter()
        .map(|row| {
            let count = thousands(row.vectors());
            let name_width = width.saturating_sub(count.len() + 3);
            let (glyph, color) = match (row.problem().is_some(), row.loaded()) {
                (true, _) => ("!", Color::Red),
                (false, true) => ("*", Color::Green),
                (false, false) => ("o", DIM),
            };
            ListItem::new(Line::from(vec![
                Span::styled(format!(" {glyph} "), Style::default().fg(color)),
                Span::raw(format!("{:<name_width$}", truncate(&row.name, name_width))),
                Span::styled(count, Style::default().fg(DIM)),
            ]))
        })
        .collect();
    let title = format!(" collections {} ", view.rows.len());
    if items.is_empty() {
        let note = if view.snapshot.is_some() {
            "  no collections in the data directory"
        } else {
            "  waiting for the server"
        };
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(note, Style::default().fg(DIM))))
                .block(pane(&title, true)),
            left,
        );
    } else {
        let list = List::new(items).block(pane(&title, true)).highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );
        let mut state = ListState::default().with_selected(Some(view.selected));
        frame.render_stateful_widget(list, left, &mut state);
    }

    let [detail, latency] =
        Layout::vertical([Constraint::Min(3), Constraint::Length(9)]).areas(right);
    match view.current() {
        Some(row) => {
            let title = match &row.metrics {
                Some(metrics) => format!(
                    " {} {} {} vectors ",
                    row.name,
                    metrics.index_type,
                    thousands(metrics.vector_count)
                ),
                None => format!(" {} not open ", row.name),
            };
            frame.render_widget(
                Paragraph::new(collection_detail(row)).block(pane(&title, true)),
                detail,
            );
        }
        None => frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  select a collection",
                Style::default().fg(DIM),
            )))
            .block(pane(" collection ", true)),
            detail,
        ),
    }

    let history: Vec<u64> = view
        .current()
        .and_then(|row| view.history.get(&row.name))
        .map(|h| h.iter().copied().collect())
        .unwrap_or_default();
    if history.is_empty() {
        frame.render_widget(
            Paragraph::new(Line::from(Span::styled(
                "  no search has been measured yet",
                Style::default().fg(DIM),
            )))
            .block(pane(" search latency ", false)),
            latency,
        );
    } else {
        let peak = history.iter().copied().max().unwrap_or(0);
        let width = usize::from(latency.width.saturating_sub(2)).max(1);
        let visible = &history[history.len().saturating_sub(width)..];
        frame.render_widget(
            Sparkline::default()
                .block(pane(
                    &format!(" search latency peak {} ", micros(peak)),
                    false,
                ))
                .data(visible)
                .style(Style::default().fg(ACCENT)),
            latency,
        );
    }
}

fn collection_detail(row: &Row) -> Vec<Line<'static>> {
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
            "  on disk, not open. The server loads a collection on first use.",
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
        &size.map_or_else(|| "none".to_owned(), bytes),
        18,
    ));
    lines
}

/// The configuration as the server resolved it.
fn config(frame: &mut Frame, app: &App, area: Rect) {
    let body = match &app.config {
        Some(Ok(text)) if text.is_empty() => Paragraph::new(Line::from(Span::styled(
            "  reading the configuration from the server",
            Style::default().fg(DIM),
        ))),
        Some(Ok(text)) => {
            let rows = usize::from(area.height.saturating_sub(2)).max(1);
            let lines: Vec<Line> = text
                .lines()
                .skip(app.config_scroll)
                .take(rows)
                .map(|line| {
                    let indent = line.len() - line.trim_start().len();
                    let style = if line.trim_end().ends_with(':') {
                        Style::default().fg(ACCENT)
                    } else {
                        Style::default()
                    };
                    Line::from(vec![
                        Span::raw(" ".repeat(2 + indent)),
                        Span::styled(line.trim_start().to_owned(), style),
                    ])
                })
                .collect();
            Paragraph::new(lines)
        }
        Some(Err(why)) => Paragraph::new(Line::from(Span::styled(
            format!("  {why}"),
            Style::default().fg(Color::Red),
        ))),
        None => Paragraph::new(Line::from(Span::styled(
            "  not loaded",
            Style::default().fg(DIM),
        ))),
    };
    frame.render_widget(body.block(pane(" config ", true)), area);
}

fn status_bar(frame: &mut Frame, app: &App, area: Rect) {
    let mode = match app.mode {
        Mode::Normal => " NORMAL ",
        Mode::Command => " COMMAND ",
        Mode::Search => " SEARCH ",
    };
    let mut spans = vec![
        Span::styled(
            " piramid ",
            Style::default().fg(Color::Black).bg(ACCENT).bold(),
        ),
        Span::styled(mode, Style::default().fg(Color::Black).bg(Color::White)),
    ];
    // One digit per view, so the tabs are also their own key hints.
    for (index, view) in app.profile.views().iter().enumerate() {
        let selected = *view == app.view;
        let style = if selected {
            Style::default().fg(Color::Black).bg(Color::White).bold()
        } else {
            Style::default().fg(DIM)
        };
        spans.push(Span::styled(
            format!(" {} {} ", index + 1, view.title()),
            style,
        ));
    }
    if !app.collections.version.is_empty() {
        spans.push(Span::styled(
            format!(" {} ", app.collections.version),
            Style::default().fg(DIM),
        ));
    }
    spans.push(probe_span("server", &app.health.live));
    spans.push(probe_span("ready", &app.health.ready));
    if app.profile == Profile::Developer {
        spans.push(probe_span("web", &app.health.web));
    }
    if let Probe::Degraded(why) = &app.health.ready {
        spans.push(Span::styled(
            format!("  {why}"),
            Style::default().fg(Color::Yellow),
        ));
    }
    if let Some(notice) = &app.notice {
        spans.push(Span::styled(
            format!("  {notice}"),
            Style::default().fg(Color::Magenta),
        ));
    }
    frame.render_widget(Line::from(spans), area);
}

fn probe_span(name: &str, probe: &Probe) -> Span<'static> {
    let (glyph, color) = match probe {
        Probe::Unknown => ("·", DIM),
        Probe::Up => ("●", Color::Green),
        Probe::Degraded(_) => ("◐", Color::Yellow),
        Probe::Down => ("○", Color::Red),
    };
    Span::styled(format!(" {glyph} {name}"), Style::default().fg(color))
}

fn status_style(status: &Status) -> Style {
    match status {
        Status::Stopped => Style::default().fg(DIM),
        Status::Starting => Style::default().fg(Color::Yellow),
        Status::Running => Style::default().fg(Color::Green),
        Status::Exited(0) => Style::default().fg(Color::Blue),
        Status::Exited(_) | Status::Failed(_) => Style::default().fg(Color::Red),
    }
}

fn sidebar(frame: &mut Frame, app: &App, area: Rect) {
    let mut items: Vec<ListItem> = Vec::new();
    let mut selected_row = 0;
    for group in Group::ALL {
        let members: Vec<(usize, &UnitState)> = app
            .units
            .iter()
            .enumerate()
            .filter(|(_, state)| state.unit.group == group)
            .collect();
        if members.is_empty() {
            continue;
        }
        items.push(ListItem::new(Line::from(Span::styled(
            format!(" {}", group.title()),
            Style::default().fg(DIM).add_modifier(Modifier::BOLD),
        ))));
        for (index, state) in members {
            if index == app.selected {
                selected_row = items.len();
            }
            let port = state
                .unit
                .url
                .as_deref()
                .and_then(|url| url.rsplit(':').next())
                .filter(|port| port.chars().all(|c| c.is_ascii_digit()))
                .map(|port| format!(":{port}"))
                .unwrap_or_default();
            let width = usize::from(SIDEBAR).saturating_sub(6 + port.len());
            items.push(ListItem::new(Line::from(vec![
                Span::styled(
                    format!("  {} ", state.status.glyph()),
                    status_style(&state.status),
                ),
                Span::raw(format!("{:<width$}", truncate(&state.unit.id, width))),
                Span::styled(port, Style::default().fg(DIM)),
            ])));
        }
    }
    let list = List::new(items)
        .block(pane(" units ", app.focus == Focus::Units))
        .highlight_style(
            Style::default()
                .bg(Color::Rgb(40, 44, 52))
                .add_modifier(Modifier::BOLD),
        );
    let mut state = ListState::default().with_selected(Some(selected_row));
    frame.render_stateful_widget(list, area, &mut state);
}

fn logs(frame: &mut Frame, app: &mut App, area: Rect) {
    let focused = app.focus == Focus::Logs;
    let rows = usize::from(area.height.saturating_sub(2)).max(1);
    app.log_rows = rows;
    let search = app.search.to_lowercase();
    let hit = app.search_hit;
    let state = app.current();
    let elapsed = state
        .started_at
        .filter(|_| state.status.is_active())
        .map(|at| format!(" · {}s", at.elapsed().as_secs()))
        .unwrap_or_default();
    let title = format!(
        " {} · {}{} · {} lines{} ",
        state.unit.id,
        state.status.label(),
        elapsed,
        state.logs.len(),
        if state.follow { " · follow" } else { "" }
    );
    let top = if state.follow {
        state.logs.len().saturating_sub(rows)
    } else {
        state.scroll.min(state.logs.len().saturating_sub(rows))
    };
    let lines: Vec<Line> = state
        .logs
        .lines()
        .enumerate()
        .skip(top)
        .take(rows)
        .map(|(index, line)| {
            let mut style = match line.stream {
                Stream::Out => Style::default(),
                Stream::Err => Style::default().fg(Color::Gray),
                Stream::Meta => Style::default().fg(ACCENT).italic(),
            };
            if !search.is_empty() && line.text.to_lowercase().contains(&search) {
                style = style.fg(Color::Yellow);
                if hit == Some(index) {
                    style = style.add_modifier(Modifier::BOLD | Modifier::REVERSED);
                }
            }
            Line::from(vec![
                Span::styled(
                    line.at.format("%H:%M:%S ").to_string(),
                    Style::default().fg(DIM),
                ),
                Span::styled(line.text.clone(), style),
            ])
        })
        .collect();
    let block = pane(&title, focused).title_bottom(
        Line::from(Span::styled(
            format!(" {} ", state.unit.hint),
            Style::default().fg(DIM),
        ))
        .right_aligned(),
    );
    let body = if state.logs.is_empty() {
        Paragraph::new(Line::from(Span::styled(
            "  nothing captured yet — press enter to start",
            Style::default().fg(DIM),
        )))
    } else {
        Paragraph::new(lines)
    };
    frame.render_widget(body.block(block), area);
}

fn bottom_line(frame: &mut Frame, app: &App, area: Rect) {
    // A confirmation takes the line over, whichever view raised it.
    if let Some(pending) = &app.collections.pending {
        frame.render_widget(
            Paragraph::new(Line::from(vec![
                Span::styled(
                    " confirm ",
                    Style::default().fg(Color::Black).bg(Color::Yellow),
                ),
                Span::styled(
                    format!(" {}", pending.question()),
                    Style::default().fg(Color::Yellow),
                ),
            ])),
            area,
        );
        return;
    }
    let line = match app.mode {
        Mode::Command => Line::from(vec![
            Span::styled(":", Style::default().fg(ACCENT)),
            Span::raw(app.input.clone()),
            Span::styled("█", Style::default().fg(ACCENT)),
        ]),
        Mode::Search => Line::from(vec![
            Span::styled("/", Style::default().fg(Color::Yellow)),
            Span::raw(app.input.clone()),
            Span::styled("█", Style::default().fg(Color::Yellow)),
        ]),
        Mode::Normal => {
            let mut spans: Vec<Span> = Vec::new();
            let hints: &[(&str, &str)] = match app.view {
                View::Units => &[
                    ("j/k", "move"),
                    ("⏎", "start/stop"),
                    ("r", "restart"),
                    ("l/h", "logs/units"),
                    ("/", "search"),
                    (":", "command"),
                    ("o", "open url"),
                ],
                View::Collections => &[
                    ("j/k", "move"),
                    ("r", "rebuild index"),
                    ("c", "compact"),
                    ("R", "refresh"),
                ],
                View::Config => &[("j/k", "scroll"), ("g", "top"), ("R", "reload")],
            };
            for (key, what) in hints.iter().copied().chain([("?", "help"), ("q", "quit")]) {
                spans.push(Span::styled(
                    format!(" {key}"),
                    Style::default().fg(ACCENT).bold(),
                ));
                spans.push(Span::styled(format!(" {what}"), Style::default().fg(DIM)));
            }
            spans.push(Span::styled(
                match app.profile {
                    Profile::Developer => {
                        format!("   {} · {}", app.base_url(), app.web_url())
                    }
                    Profile::Production => format!("   {}", app.base_url()),
                },
                Style::default().fg(DIM),
            ));
            Line::from(spans)
        }
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn help(frame: &mut Frame, app: &App, area: Rect) {
    let shared: &[(&str, &str)] = &[
        ("1 to 9", "switch view, numbered as in the bar above"),
        ("?", "this help"),
        ("q, ctrl-c", "quit"),
    ];
    let per_view: &[(&str, &str)] = match app.view {
        View::Units => &[
            ("j / k", "move selection, or scroll the log pane"),
            ("gg / G", "top, bottom. G on logs resumes following"),
            ("ctrl-d / ctrl-u", "half page in logs"),
            ("enter / s", "start or stop the selected unit"),
            ("x / r", "stop, restart the selected unit"),
            ("h / l, tab", "focus units, logs"),
            ("/ then n / N", "search the logs of the selected unit"),
            ("C", "clear the logs of the selected unit"),
            ("o", "open the URL of the unit in a browser"),
            (":start x  :stop x", "act on a unit by name"),
            (":<recipe> [args]", "run any just recipe"),
        ],
        View::Collections => &[
            ("j / k", "move between collections"),
            ("gg / G", "first, last collection"),
            (
                "r",
                "rebuild the index of the selected collection, after y or n",
            ),
            ("c", "compact the selected collection, after y or n"),
            ("R", "refresh now instead of waiting for the interval"),
        ],
        View::Config => &[
            ("j / k", "scroll"),
            ("g", "top"),
            ("R", "read the configuration again"),
        ],
    };

    let mut lines = vec![
        Line::from(Span::styled(
            format!("{} view", app.view.title()),
            Style::default().fg(ACCENT).bold(),
        )),
        Line::default(),
    ];
    for (key, what) in per_view.iter().chain(shared) {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<24}"), Style::default().fg(Color::Yellow)),
            Span::raw((*what).to_owned()),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        match app.profile {
            Profile::Developer => {
                "  Running inside a checkout, so the units view can drive the repo."
            }
            Profile::Production => {
                "  Running outside a checkout. The units view needs a justfile and is hidden."
            }
        },
        Style::default().fg(DIM),
    )));
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "any key closes this",
        Style::default().fg(DIM),
    )));

    let width = 88.min(area.width.saturating_sub(4));
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

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_owned();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
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

/// A count with thousands separators.
pub fn thousands(n: usize) -> String {
    thousands_u64(n as u64)
}

/// A u64 count with thousands separators.
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

/// A millisecond measurement, or a placeholder when nothing has been measured.
pub fn millis(ms: Option<f32>) -> String {
    ms.map_or_else(|| "none".to_owned(), |v| format!("{v:.2} ms"))
}

/// A microsecond measurement rendered in milliseconds.
pub fn micros(us: u64) -> String {
    format!("{:.2} ms", us as f64 / 1000.0)
}

/// A span of seconds at the coarsest unit that still says something.
pub fn duration(secs: u64) -> String {
    match secs {
        0..=59 => format!("{secs}s"),
        60..=3599 => format!("{}m", secs / 60),
        3600..=86_399 => format!("{}h", secs / 3600),
        _ => format!("{}d", secs / 86_400),
    }
}
