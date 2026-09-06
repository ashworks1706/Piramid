//! `piramid top`: a live view of a running server.
//!
//! Everything here reads the HTTP API, so the dashboard watches the process actually serving
//! traffic and never opens the data directory itself — two processes with the same collection
//! open is exactly the contention this is meant to help diagnose.

mod client;
mod dashboard;
mod run;
mod ui;

pub use run::run;

#[cfg(test)]
mod tests;
