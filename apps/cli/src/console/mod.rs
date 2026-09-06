//! The developer console: run every unit in the repo, stream its output, and watch the server,
//! from one modal terminal UI.
//!
//! Contributor tooling that happens to ship in the binary. It drives `just` recipes and
//! `docker compose`, so it only means anything inside a checkout — an installed `piramid` run
//! from somewhere else finds no justfile and prints help instead.
//!
//! Nothing here reimplements a recipe. Starting the server runs `just serve`, exactly what you
//! would type, so the console cannot drift from the justfile.

mod app;
mod health;
mod logs;
mod run;
mod runner;
mod settings;
mod types;
mod ui;
mod units;

pub use run::run;
pub use settings::repo_root;

#[cfg(test)]
mod tests;
