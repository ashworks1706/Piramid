#![deny(missing_docs)]

//! The machine: the math, the device that runs it, and the encodings it runs over.
//!
//! [`compute`] owns what a distance means and which strategy computes it. [`gpu`] owns talking to
//! a device and nothing about what the math means; vendor SDK types stay inside `gpu::backends`.

pub mod compute;
pub mod gpu;
