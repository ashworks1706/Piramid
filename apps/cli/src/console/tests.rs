//! Unit tests for the parts of the console that are not drawing.

use std::collections::HashMap;

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::app::parse_command;
use super::logs::{LogBuffer, LogWriter};
use super::runner::{parse_ps, sanitize_line};
use super::settings::{repo_root, Settings};
use super::types::{Command, Group, LogLine, ServiceState, Status, Stream};
use super::units::catalog;

#[test]
fn commands_parse_into_actions() {
    assert_eq!(parse_command("q"), Command::Quit);
    assert_eq!(parse_command("start serve"), Command::Start("serve".into()));
    assert_eq!(
        parse_command("stop  ollama"),
        Command::Stop("ollama".into())
    );
    assert_eq!(parse_command("restart web"), Command::Restart("web".into()));
    assert_eq!(parse_command("help"), Command::Help);
    assert_eq!(parse_command("clear"), Command::Clear);
}

#[test]
fn an_unrecognised_word_is_handed_to_just() {
    // The command line is the whole justfile, not a list kept in sync with it by hand.
    assert_eq!(
        parse_command("check-gpu"),
        Command::Just(vec!["check-gpu".into()])
    );
    assert_eq!(
        parse_command("bench --save-baseline main"),
        Command::Just(vec![
            "bench".into(),
            "--save-baseline".into(),
            "main".into()
        ])
    );
    // `start` with nothing to start is a recipe name, not a malformed start.
    assert_eq!(parse_command("start"), Command::Just(vec!["start".into()]));
    assert_eq!(parse_command("   "), Command::Unknown(String::new()));
}

#[test]
fn the_catalog_is_unique_and_every_unit_is_runnable() {
    let units = catalog();
    let ids: std::collections::HashSet<&str> = units.iter().map(|u| u.id.as_str()).collect();
    assert_eq!(ids.len(), units.len(), "two units share an id");
    // A unit is either a compose service or a just recipe; one with neither cannot be started.
    assert!(units
        .iter()
        .all(|u| u.service().is_some() || !u.args.is_empty()));
    assert!(units.iter().any(|u| u.id == "serve"));
    // A task is named for what it is, not for the command line that runs it.
    let config = units
        .iter()
        .find(|u| u.id == "config")
        .expect("the catalog offers the resolved configuration");
    assert_eq!(config.args, ["piramid", "show", "config"]);
    assert!(
        !units.iter().any(|u| u.id.starts_with("piramid ")),
        "a unit is showing its command line as its name"
    );
    assert!(units.iter().any(|u| u.id == "check"));
    assert!(units
        .iter()
        .any(|u| u.id == "prod-up" && u.group == Group::Deploy));
    assert!(units
        .iter()
        .any(|u| u.id == "ollama" && u.group == Group::Containers));
}

#[test]
fn every_catalog_recipe_exists_in_the_justfile() {
    let Some(root) = repo_root(std::path::Path::new(env!("CARGO_MANIFEST_DIR"))) else {
        return;
    };
    let justfile = std::fs::read_to_string(root.join("justfile")).unwrap_or_default();
    let recipes: std::collections::HashSet<String> = justfile
        .lines()
        .filter(|line| !line.starts_with(char::is_whitespace) && line.contains(':'))
        .filter_map(|line| line.split([':', ' ']).next())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect();

    for unit in catalog() {
        let Some(recipe) = unit.args.first() else {
            continue;
        };
        assert!(
            recipes.contains(recipe),
            "catalog runs `just {recipe}`, which the justfile does not define"
        );
    }
}

/// A console over a scratch directory, with no repo behind it.
fn console() -> super::app::App {
    let root = std::env::temp_dir().join(format!("piramid-console-{}", std::process::id()));
    let (tx, _rx) = tokio::sync::mpsc::unbounded_channel();
    super::app::App::new(Settings::default(), root, &tx).expect("the log directory is creatable")
}

fn press(key: char) -> super::types::Event {
    super::types::Event::Key(KeyEvent::new(KeyCode::Char(key), KeyModifiers::NONE))
}

#[test]
fn navigation_moves_through_the_catalog_and_stops_at_both_ends() {
    let mut app = console();
    assert_eq!(app.current().unit.id, "serve");

    app.handle(press('k'));
    assert_eq!(app.current().unit.id, "serve", "k at the top must not wrap");

    app.handle(press('j'));
    assert_eq!(app.current().unit.id, "web");

    app.handle(press('G'));
    let last = app.units.last().map(|state| state.unit.id.clone());
    assert_eq!(Some(app.current().unit.id.clone()), last);

    app.handle(press('j'));
    assert_eq!(
        Some(app.current().unit.id.clone()),
        last,
        "j at the end must not wrap"
    );

    app.handle(press('g'));
    app.handle(press('g'));
    assert_eq!(app.current().unit.id, "serve");
}

#[test]
fn help_swallows_the_next_key_rather_than_acting_on_it() {
    let mut app = console();
    app.handle(press('?'));
    assert!(app.help);

    // The key that closes help must not also start whatever is selected.
    app.handle(press('\r'));
    assert!(!app.help);
    assert_eq!(app.current().status, Status::Stopped);
}

#[test]
fn quitting_is_q_or_ctrl_c() {
    let mut app = console();
    app.handle(press('q'));
    assert!(app.should_quit);

    let mut app = console();
    app.handle(super::types::Event::Key(KeyEvent::new(
        KeyCode::Char('c'),
        KeyModifiers::CONTROL,
    )));
    assert!(app.should_quit);
}

#[test]
fn stopping_something_that_is_not_running_says_so_instead_of_signalling() {
    let mut app = console();
    app.handle(press('x'));
    assert_eq!(app.notice.as_deref(), Some("serve is not running"));
}

#[test]
fn compose_states_map_onto_statuses() {
    let state = |state: &str, health: &str, exit_code| ServiceState {
        state: state.into(),
        health: health.into(),
        exit_code,
    };
    assert_eq!(state("running", "healthy", 0).status(), Status::Running);
    assert_eq!(state("running", "", 0).status(), Status::Running);
    assert_eq!(state("running", "starting", 0).status(), Status::Starting);
    assert_eq!(
        state("running", "unhealthy", 0).status(),
        Status::Failed("unhealthy".into())
    );
    assert_eq!(state("exited", "", 0).status(), Status::Stopped);
    assert_eq!(state("exited", "", 137).status(), Status::Exited(137));
}

#[test]
fn ps_output_parses_as_an_array_or_as_lines() {
    let array = r#"[{"Service":"piramid","State":"running","Health":"healthy","ExitCode":0}]"#;
    assert_eq!(
        parse_ps(array).unwrap_or_default()["piramid"].health,
        "healthy"
    );
    let lines = "{\"Service\":\"piramid\",\"State\":\"exited\",\"ExitCode\":1}\n{\"Service\":\"ollama\",\"State\":\"running\"}\n";
    let parsed: HashMap<_, _> = parse_ps(lines).unwrap_or_default();
    assert_eq!(parsed["piramid"].exit_code, 1);
    assert_eq!(parsed["ollama"].status(), Status::Running);
    assert!(parse_ps("").is_ok_and(|m| m.is_empty()));
    // Reporting "nothing running" when the query failed is how you start a second copy.
    assert!(parse_ps("not json").is_err());
}

#[test]
fn a_log_line_cannot_move_the_cursor_out_of_its_pane() {
    assert_eq!(
        sanitize_line("\x1b[32m   Compiling\x1b[0m piramid"),
        "   Compiling piramid"
    );
    assert_eq!(sanitize_line("plain"), "plain");
    // compose rewrites its progress lines in place; a carriage return reaching the terminal
    // returns the cursor to column 0 and overwrites whatever is drawn to the left.
    assert_eq!(
        sanitize_line("Container deploy-piramid-1  Recreated\r"),
        "Container deploy-piramid-1  Recreated"
    );
    assert_eq!(sanitize_line("a\rb\x08c\x07"), "abc");
    assert_eq!(sanitize_line("keeps\ttabs"), "keeps\ttabs");
}

#[test]
fn the_log_buffer_drops_the_oldest_line_and_searches_wrapping() {
    let mut buffer = LogBuffer::new(3);
    for text in ["alpha", "Beta", "gamma", "delta"] {
        buffer.push(LogLine::now(Stream::Out, text));
    }
    let texts: Vec<&str> = buffer.lines().map(|line| line.text.as_str()).collect();
    assert_eq!(texts, ["Beta", "gamma", "delta"]);
    assert_eq!(buffer.find("beta", 2, false), Some(0));
    assert_eq!(buffer.find("delta", 0, true), Some(2));
    assert_eq!(buffer.find("zeta", 0, false), None);
    assert_eq!(buffer.find("", 0, false), None);
}

#[test]
fn full_output_is_kept_on_disk_after_the_pane_scrolls_past_it() {
    // CARGO_TARGET_TMPDIR is only defined for integration tests, and a path relative to the
    // crate would write into the source tree.
    let dir = std::env::temp_dir().join(format!("piramid-console-logs-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    let mut writer = LogWriter::new(&dir).expect("the log directory is creatable");

    writer
        .append("serve", &LogLine::now(Stream::Out, "listening"))
        .expect("a line is writable");
    // An ad-hoc task's id is a whole command line, which is not a filename.
    writer
        .append(
            "bench --save-baseline main",
            &LogLine::now(Stream::Out, "done"),
        )
        .expect("a line is writable");

    let saved = std::fs::read_to_string(dir.join("serve.log")).unwrap_or_default();
    assert!(saved.ends_with(" out listening\n"), "got {saved:?}");
    assert!(dir.join("bench---save-baseline-main.log").is_file());
    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn settings_default_to_the_ports_the_justfile_uses() {
    let settings = Settings::default();
    assert_eq!(settings.base_url, "http://localhost:6333");
    assert_eq!(settings.web_url, "http://localhost:3000");
    assert_eq!(
        settings.log_dir_under(std::path::Path::new("/repo")),
        std::path::Path::new("/repo/target/console-logs")
    );
}

#[test]
fn the_repo_root_is_found_from_a_nested_directory() {
    let here = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = repo_root(here).unwrap_or_default();
    assert!(root.join("justfile").is_file());
    // An installed binary run from outside a checkout finds nothing, and prints help instead.
    assert!(repo_root(std::path::Path::new("/")).is_none());
}
