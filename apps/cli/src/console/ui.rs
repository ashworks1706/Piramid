//! Draws the console: status bar, unit list, log pane, command line, help.

use ratatui::layout::{Alignment, Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType, Borders, Clear, List, ListItem, ListState, Paragraph};
use ratatui::Frame;

use crate::console::app::{App, UnitState};
use crate::console::types::{Focus, Group, Mode, Probe, Status, Stream};

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
    let [side, right] =
        Layout::horizontal([Constraint::Length(SIDEBAR), Constraint::Min(20)]).areas(body);

    status_bar(frame, app, bar);
    sidebar(frame, app, side);
    logs(frame, app, right);
    bottom_line(frame, app, bottom);
    if app.help {
        help(frame, frame.area());
    }
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
        probe_span("server", &app.health.live),
        probe_span("ready", &app.health.ready),
        probe_span("web", &app.health.web),
    ];
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
            for (key, what) in [
                ("j/k", "move"),
                ("⏎", "start/stop"),
                ("r", "restart"),
                ("l/h", "logs/units"),
                ("/", "search"),
                (":", "command"),
                ("o", "open url"),
                ("?", "help"),
                ("q", "quit"),
            ] {
                spans.push(Span::styled(
                    format!(" {key}"),
                    Style::default().fg(ACCENT).bold(),
                ));
                spans.push(Span::styled(format!(" {what}"), Style::default().fg(DIM)));
            }
            spans.push(Span::styled(
                format!("   {} · {}", app.base_url(), app.web_url()),
                Style::default().fg(DIM),
            ));
            Line::from(spans)
        }
    };
    frame.render_widget(Paragraph::new(line), area);
}

fn help(frame: &mut Frame, area: Rect) {
    let keys = [
        ("j / k, ↑ / ↓", "move selection, or scroll the focused pane"),
        ("gg / G", "top / bottom (G on logs resumes following)"),
        ("ctrl-d / ctrl-u", "half page in logs"),
        ("enter / s", "start or stop the selected unit"),
        ("x / r", "stop / restart the selected unit"),
        ("h / l, tab", "focus units / logs"),
        ("/ then n / N", "search the selected unit's logs"),
        ("C", "clear the selected unit's logs"),
        ("o", "open the unit's URL in a browser"),
        (
            ":start x  :stop x  :restart x",
            "by unit id, e.g. :start serve",
        ),
        (":<recipe> [args]", "run any just recipe, e.g. :check-gpu"),
        (":q, q", "quit: host processes stop, containers stay up"),
    ];
    let mut lines = vec![
        Line::from(Span::styled("keys", Style::default().fg(ACCENT).bold())),
        Line::default(),
    ];
    for (key, what) in keys {
        lines.push(Line::from(vec![
            Span::styled(format!("  {key:<30}"), Style::default().fg(Color::Yellow)),
            Span::raw(what),
        ]));
    }
    lines.push(Line::default());
    lines.push(Line::from(Span::styled(
        "  every unit runs the same just recipe you would type; full output is kept under",
        Style::default().fg(DIM),
    )));
    lines.push(Line::from(Span::styled(
        "  target/console-logs even after the pane scrolls past it.",
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
