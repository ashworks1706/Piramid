//! How vectors are found: the structures that narrow the candidate set, and the planning over them.
//!
//! [`index`] owns traversal and the sidecar format. [`search`] owns what a query asks for —
//! overfetch, filtering, scoring and ranking — and never learns what a collection is.

pub mod index;
pub mod search;
