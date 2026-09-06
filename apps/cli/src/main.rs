use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
mod animation;
mod console;
mod support;
mod top;
use piramid::config::{self, Config, StartupConfig};
use piramid::observability;
use piramid::state::AppState;
use piramid::{embeddings, server};
use tokio::runtime::Runtime;

#[derive(Parser)]
#[command(author, version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Start the server directly
    Serve {
        /// Optional config file (sets CONFIG_FILE)
        #[arg(long)]
        config: Option<PathBuf>,
        /// Override port (sets PORT)
        #[arg(long)]
        port: Option<u16>,
        /// Override data dir (sets DATA_DIR)
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },

    /// Generate a config file with defaults (YAML)
    Init {
        /// Path to write the config file
        #[arg(long, short, default_value = "piramid.yaml")]
        path: PathBuf,
        /// Output format (yaml or json)
        #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
        format: OutputFormat,
    },

    /// Watch a running server: collections, index state, latency, WAL and disk
    Top {
        /// Base URL of the server to watch
        #[arg(long, env = "PIRAMID_URL", default_value = "http://localhost:6333")]
        url: String,
        /// Seconds between refreshes
        #[arg(long, default_value_t = 2, value_parser = clap::value_parser!(u64).range(1..=3600))]
        interval: u64,
    },

    /// Show runtime/configuration information
    Show {
        #[command(subcommand)]
        command: ShowCommands,
    },

    /// Write a diagnostic bundle to attach to a bug report. Secrets are redacted; review it first
    SupportBundle {
        /// Where to write the bundle
        #[arg(long, short, default_value = "piramid-support-bundle.md")]
        output: PathBuf,
        /// Optional config file to load (overrides CONFIG_FILE)
        #[arg(long)]
        config: Option<PathBuf>,
        /// Data directory to inspect (overrides DATA_DIR)
        #[arg(long)]
        data_dir: Option<PathBuf>,
    },
}

#[derive(Subcommand)]
enum ShowCommands {
    /// Print the resolved configuration
    Config(ShowConfigArgs),
    /// Print collection and WAL metrics from local data dir
    Metrics(ShowMetricsArgs),
}

#[derive(Args)]
struct ShowConfigArgs {
    /// Optional config file to load (overrides CONFIG_FILE)
    #[arg(long)]
    config: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Yaml)]
    format: OutputFormat,
}

#[derive(Args)]
struct ShowMetricsArgs {
    /// Optional config file to load (overrides CONFIG_FILE)
    #[arg(long)]
    config: Option<PathBuf>,
    /// Optional data directory (overrides DATA_DIR)
    #[arg(long)]
    data_dir: Option<PathBuf>,
    #[arg(long, value_enum, default_value_t = OutputFormat::Json)]
    format: OutputFormat,
}

#[derive(Copy, Clone, ValueEnum)]
enum OutputFormat {
    Yaml,
    Json,
}

/// Run `action`, printing `context` and exiting 1 on failure.
fn run_or_exit(action: impl FnOnce() -> std::io::Result<()>, context: &str) {
    if let Err(e) = action() {
        eprintln!("{context}: {e}");
        std::process::exit(1);
    }
}

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Init { path, format }) => {
            run_or_exit(
                || write_config_file(&path, format),
                "Failed to write config",
            );
            println!("Wrote config to {}", path.display());
        }
        Some(Commands::SupportBundle {
            output,
            config,
            data_dir,
        }) => {
            run_or_exit(
                || support_bundle(output, config, data_dir),
                "Failed to write support bundle",
            );
        }
        Some(Commands::Top { url, interval }) => {
            run_or_exit(
                || top::run(&url, Duration::from_secs(interval)),
                "Failed to start the dashboard",
            );
        }
        Some(Commands::Show { command }) => {
            run_or_exit(
                || handle_show_command(command),
                "Failed to show information",
            );
        }
        Some(Commands::Serve {
            config,
            port,
            data_dir,
        }) => {
            animate();
            if let Some(path) = config {
                std::env::set_var("CONFIG_FILE", path);
            }
            if let Some(port) = port {
                std::env::set_var("PORT", port.to_string());
            }
            if let Some(dir) = data_dir {
                std::env::set_var("DATA_DIR", dir);
            }
            run_or_exit(start_server_inline, "Failed to start the server");
        }
        // No subcommand opens the developer console, which is what you want from inside a
        // checkout. It drives `just`, so outside one there is nothing for it to run and the
        // help is the useful answer instead.
        None => match std::env::current_dir()
            .ok()
            .and_then(|cwd| console::repo_root(&cwd))
        {
            Some(root) => run_or_exit(|| console::run(root), "Failed to start the console"),
            None => {
                let mut command = Cli::command();
                run_or_exit(|| command.print_help(), "Failed to print help");
                println!();
            }
        },
    }
}

fn handle_show_command(command: ShowCommands) -> std::io::Result<()> {
    match command {
        ShowCommands::Config(args) => show_config(args),
        ShowCommands::Metrics(args) => show_metrics(args),
    }
}

/// Report a configuration failure and exit.
fn exit_on_config_error<T>(error: piramid::error::ConfigError) -> T {
    eprintln!("piramid: {error}");
    std::process::exit(1);
}

fn show_config(args: ShowConfigArgs) -> std::io::Result<()> {
    if let Some(path) = args.config {
        std::env::set_var("CONFIG_FILE", path);
    }
    let cfg = config::loader::load().unwrap_or_else(exit_on_config_error);
    print_serialized(&cfg, args.format)
}

fn support_bundle(
    output: PathBuf,
    config: Option<PathBuf>,
    data_dir: Option<PathBuf>,
) -> std::io::Result<()> {
    if let Some(path) = config {
        std::env::set_var("CONFIG_FILE", path);
    }
    if let Some(dir) = data_dir {
        std::env::set_var("DATA_DIR", dir);
    }

    let config = piramid::config::loader::load().unwrap_or_else(exit_on_config_error);
    let state = std::sync::Arc::new(
        AppState::new(config.clone(), embeddings::EmbeddingsManager::disabled())
            .map_err(std::io::Error::other)?,
    );
    // Best-effort: a broken collection shouldn't stop the bundle from being written.
    let _ = preload_collections_for_metrics(&state);

    let path = support::write(&config, &state, Some(output))?;
    println!("wrote {}", path.display());
    println!("Review it before sharing — it contains your configuration and collection names.");
    Ok(())
}

fn show_metrics(args: ShowMetricsArgs) -> std::io::Result<()> {
    if let Some(path) = args.config {
        std::env::set_var("CONFIG_FILE", path);
    }
    if let Some(dir) = args.data_dir {
        std::env::set_var("DATA_DIR", dir);
    }
    let config = piramid::config::loader::load().unwrap_or_else(exit_on_config_error);
    let state = std::sync::Arc::new(
        AppState::new(config, embeddings::EmbeddingsManager::disabled())
            .map_err(std::io::Error::other)?,
    );
    preload_collections_for_metrics(&state)?;
    let metrics = piramid::services::admin::metrics(&state).map_err(std::io::Error::other)?;
    print_serialized(&metrics, args.format)
}

fn preload_collections_for_metrics(state: &std::sync::Arc<AppState>) -> std::io::Result<()> {
    for collection_name in state.collection_manager.discover_on_disk() {
        if let Err(error) = state.get_existing_collection(&collection_name) {
            eprintln!("Skipping collection '{collection_name}' while building metrics: {error}");
        }
    }
    Ok(())
}

fn write_config_file(path: &Path, fmt: OutputFormat) -> std::io::Result<()> {
    let cfg = Config::default();
    let contents = serialize_to_string(&cfg, fmt)?;
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            fs::create_dir_all(parent)?;
        }
    }
    fs::write(path, contents)
}

fn start_server_inline() -> std::io::Result<()> {
    let rt = Runtime::new().map_err(std::io::Error::other)?;
    rt.block_on(async {
        let config = piramid::config::loader::load().unwrap_or_else(exit_on_config_error);

        let _observability =
            observability::install(config.startup.logging, &config.startup.telemetry);
        init_thread_pool(&config.startup);
        if config.startup.logging.config {
            tracing::info!(
                target: "piramid::config",
                config = ?config,
                "using_configuration"
            );
        }

        let embeddings = match &config.startup.embedding {
            Some(embedding) => {
                embeddings::EmbeddingsManager::from_config(embedding).map_err(|e| {
                    std::io::Error::other(format!(
                        "embedding provider configured but failed to initialize: {e}"
                    ))
                })?
            }
            None => embeddings::EmbeddingsManager::disabled(),
        };
        let addr = config.startup.bind.clone();
        let data_dir = config.startup.data_dir.clone();
        let state =
            std::sync::Arc::new(AppState::new(config, embeddings).map_err(std::io::Error::other)?);

        let app = server::create_router(state);
        tracing::info!(
            target: "piramid::config",
            address = addr.as_str(),
            data_dir = data_dir.as_str(),
            "server_starting"
        );
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .map_err(|e| std::io::Error::other(format!("bind failed: {e}")))?;
        axum::serve(listener, app)
            .await
            .map_err(std::io::Error::other)
    })
}

/// Build the global rayon pool. Called once, before any collection opens.
fn init_thread_pool(startup: &StartupConfig) {
    let num_threads = startup.num_threads();
    if let Err(error) = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build_global()
    {
        tracing::warn!(target: "piramid::config", %error, "thread_pool_already_built");
    }
}

fn serialize_to_string<T: serde::Serialize>(
    value: &T,
    fmt: OutputFormat,
) -> std::io::Result<String> {
    match fmt {
        OutputFormat::Yaml => serde_yaml::to_string(value).map_err(std::io::Error::other),
        OutputFormat::Json => serde_json::to_string_pretty(value).map_err(std::io::Error::other),
    }
}

fn print_serialized<T: serde::Serialize>(value: &T, fmt: OutputFormat) -> std::io::Result<()> {
    let rendered = serialize_to_string(value, fmt)?;
    println!("{rendered}");
    Ok(())
}

fn animate() {
    // Cursor moves and screen clears are meaningless to a pipe, and the console captures stdout
    // line by line — the splash arrived there as 280 lines of banner ahead of the first log.
    if !std::io::IsTerminal::is_terminal(&std::io::stdout()) {
        return;
    }
    print!("\x1b[2J\x1b[H\x1b[?25l");
    let _ = std::io::stdout().flush();

    for (i, frame) in animation::CLI_FRAMES.iter().enumerate() {
        print!("\x1b[H{frame}");
        let _ = std::io::stdout().flush();
        thread::sleep(Duration::from_millis(45));
        if i > 12 {
            break;
        }
    }

    print!("\x1b[2J\x1b[H\n\x1b[?25h");
    let _ = std::io::stdout().flush();
}
