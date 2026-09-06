//! The top subcommand: a live view of a running server.
//!
//! Everything here reads the HTTP API. The dashboard never opens the data directory itself.

mod client;
mod dashboard;
mod run;
mod ui;

pub use run::run;

#[cfg(test)]
mod tests;
