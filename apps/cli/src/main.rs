use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;
use std::sync::OnceLock;
use std::thread;
use std::time::Duration;

use clap::{Args, CommandFactory, Parser, Subcommand, ValueEnum};
mod animation;
mod support;
use piramid::config::{self, AppConfig, LogLevel, LoggingConfig};
use piramid::state::AppState;
use piramid::{config::loader::RuntimeConfig, embeddings, server};
use piramid_observability::{ObservabilityConfig, ObservabilityGuard};
use tokio::runtime::Runtime;
use tracing_subscriber::EnvFilter;

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

fn main() {
    let cli = Cli::parse();
    match cli.command {
        Some(Commands::Init { path, format }) => {
            if let Err(e) = write_config_file(&path, format) {
                eprintln!("Failed to write config: {e}");
                std::process::exit(1);
            }
            println!("Wrote config to {}", path.display());
        }
        Some(Commands::SupportBundle {
            output,
            config,
            data_dir,
        }) => {
            if let Err(e) = support_bundle(output, config, data_dir) {
                eprintln!("Failed to write support bundle: {e}");
                std::process::exit(1);
            }
        }
        Some(Commands::Show { command }) => {
            if let Err(e) = handle_show_command(command) {
                eprintln!("Failed to show information: {e}");
                std::process::exit(1);
            }
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
            if let Err(e) = start_server_inline() {
                eprintln!("Failed to start piramid-server: {e}");
                std::process::exit(1);
            }
        }
        None => {
            let mut command = Cli::command();
            if let Err(e) = command.print_help() {
                eprintln!("Failed to print help: {e}");
                std::process::exit(1);
            }
            println!();
        }
    }
}

fn handle_show_command(command: ShowCommands) -> std::io::Result<()> {
    match command {
        ShowCommands::Config(args) => show_config(args),
        ShowCommands::Metrics(args) => show_metrics(args),
    }
}

/// Report a configuration failure and exit.
fn exit_on_config_error<T>(error: piramid::config::loader::ConfigError) -> T {
    eprintln!("piramid: {error}");
    std::process::exit(1);
}

fn show_config(args: ShowConfigArgs) -> std::io::Result<()> {
    if let Some(path) = args.config {
        std::env::set_var("CONFIG_FILE", path);
    }
    let cfg = config::loader::load_app_config().unwrap_or_else(exit_on_config_error);
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

    let runtime =
        piramid::config::loader::load_runtime_config().unwrap_or_else(exit_on_config_error);
    let state = std::sync::Arc::new(
        AppState::new(
            &runtime.data_dir,
            runtime.app.clone(),
            runtime.slow_query_ms,
            embeddings::EmbeddingsManager::disabled(),
            runtime.disk_min_free_bytes,
            runtime.disk_readonly_on_low_space,
        )
        .map_err(std::io::Error::other)?,
    );
    // Best-effort: a broken collection shouldn't stop the bundle from being written.
    let _ = preload_collections_for_metrics(&state);

    let path = support::write(&runtime, &state, Some(output))?;
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
    let RuntimeConfig {
        app: app_config,
        data_dir,
        slow_query_ms,
        disk_min_free_bytes,
        disk_readonly_on_low_space,
        ..
    } = piramid::config::loader::load_runtime_config().unwrap_or_else(exit_on_config_error);

    let state = std::sync::Arc::new(
        AppState::new(
            &data_dir,
            app_config,
            slow_query_ms,
            embeddings::EmbeddingsManager::disabled(),
            disk_min_free_bytes,
            disk_readonly_on_low_space,
        )
        .map_err(std::io::Error::other)?,
    );
    preload_collections_for_metrics(&state)?;
    let metrics = piramid::services::admin::metrics(&state).map_err(std::io::Error::other)?;
    print_serialized(&metrics, args.format)
}

fn preload_collections_for_metrics(state: &std::sync::Arc<AppState>) -> std::io::Result<()> {
    let entries = fs::read_dir(&state.data_dir)?;
    for entry in entries {
        let entry = entry?;
        let file_name = match entry.file_name().to_str() {
            Some(v) => v.to_string(),
            None => continue,
        };
        let collection_name = match collection_name_from_base_db_filename(&file_name) {
            Some(v) => v,
            None => continue,
        };
        if let Err(error) = state.get_existing_collection(&collection_name) {
            eprintln!(
                "Skipping collection '{}' while building metrics: {}",
                collection_name, error
            );
        }
    }
    Ok(())
}

fn collection_name_from_base_db_filename(file_name: &str) -> Option<String> {
    if !file_name.ends_with(".db") {
        return None;
    }
    if file_name.ends_with(".index.db")
        || file_name.ends_with(".vecindex.db")
        || file_name.ends_with(".metadata.db")
        || file_name.ends_with(".wal.db")
    {
        return None;
    }
    let name = file_name.strip_suffix(".db")?;
    if name.is_empty() {
        return None;
    }
    Some(name.to_string())
}

fn write_config_file(path: &Path, fmt: OutputFormat) -> std::io::Result<()> {
    let cfg = AppConfig::default();
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
        let RuntimeConfig {
            app: app_config,
            port,
            data_dir,
            slow_query_ms,
            embedding: embedding_config,
            disk_min_free_bytes,
            disk_readonly_on_low_space,
        } = piramid::config::loader::load_runtime_config().unwrap_or_else(exit_on_config_error);

        let _observability = init_tracing(app_config.logging)?;
        if app_config.logging.config {
            tracing::info!(
                target: "piramid::config",
                config = ?app_config,
                "using_configuration"
            );
        }

        let embeddings = match embedding_config.clone() {
            Some(config) => embeddings::EmbeddingsManager::from_config(&config).map_err(|e| {
                std::io::Error::other(format!(
                    "embedding provider configured but failed to initialize: {e}"
                ))
            })?,
            None => embeddings::EmbeddingsManager::disabled(),
        };
        let state = std::sync::Arc::new(
            AppState::new(
                &data_dir,
                app_config.clone(),
                slow_query_ms,
                embeddings,
                disk_min_free_bytes,
                disk_readonly_on_low_space,
            )
            .map_err(std::io::Error::other)?,
        );

        let app = server::create_router(state);
        let addr = format!("0.0.0.0:{}", port);
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

/// Install tracing and any configured telemetry exporters; `None` if disabled or already run.
fn init_tracing(cfg: LoggingConfig) -> std::io::Result<Option<ObservabilityGuard>> {
    static TRACING_INIT: OnceLock<()> = OnceLock::new();
    if TRACING_INIT.get().is_some() {
        return Ok(None);
    }
    if !cfg.enabled {
        TRACING_INIT.set(()).ok();
        return Ok(None);
    }

    let base_level = level_directive(cfg.level);
    let mut env_filter = if let Ok(val) = std::env::var("RUST_LOG") {
        EnvFilter::new(val)
    } else {
        EnvFilter::new(base_level)
    };

    if !cfg.config {
        env_filter = add_directive(env_filter, "piramid::config=off");
    }
    if !cfg.indexing {
        env_filter = add_directive(env_filter, "piramid::indexing=off");
    }
    if !cfg.search {
        env_filter = add_directive(env_filter, "piramid::search=off");
    }
    if !cfg.writes {
        env_filter = add_directive(env_filter, "piramid::writes=off");
    }
    if !cfg.inference {
        env_filter = add_directive(env_filter, "piramid::inference=off");
    }
    if !cfg.http {
        env_filter = add_directive(env_filter, "piramid::http=off");
    }

    // Exporters are opt-in per environment variable and no-ops when unset.
    let observability = ObservabilityConfig::from_env().map_err(std::io::Error::other)?;
    let guard = piramid_observability::init(&observability, env_filter, cfg.json);
    TRACING_INIT.set(()).ok();
    Ok(Some(guard))
}

/// Add a filter directive built from a literal in this file.
fn add_directive(mut filter: EnvFilter, directive: &str) -> EnvFilter {
    match tracing_subscriber::filter::Directive::from_str(directive) {
        Ok(parsed) => filter = filter.add_directive(parsed),
        Err(error) => eprintln!("piramid: ignoring malformed log directive '{directive}': {error}"),
    }
    filter
}

fn level_directive(level: LogLevel) -> &'static str {
    match level {
        LogLevel::Error => "error",
        LogLevel::Warn => "warn",
        LogLevel::Info => "info",
        LogLevel::Debug => "debug",
        LogLevel::Trace => "trace",
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
