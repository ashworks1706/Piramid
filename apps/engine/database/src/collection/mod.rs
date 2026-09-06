//! The collection: the object that composes a record store, its caches, a checkpoint policy and
//! an index into one queryable thing.
//!
//! [storage](crate::storage) is bytes, and [index](crate::index) and [search](crate::search) are
//! how those bytes are found. This module owns them together and decides lifecycle: when a
//! collection opens, when it checkpoints, when it compacts, and what a write is allowed to do.
//!
//! [state] holds what a collection owns: a record store, its caches, an index and a manifest.
//! Every file beside it is one thing done to that state. CollectionHandle is the shared pointer
//! callers hold, and Collection is the state behind its lock.
//!
//! [search_target] and [near_duplicates] turn the configuration of a collection into a
//! SearchTarget, which keeps search below this module rather than circular with it.

mod checkpoint;
mod compact;
pub(crate) mod limits;
mod manager;
mod near_duplicates;
mod open;
mod search_target;
mod state;

pub use checkpoint::CheckpointManager;
pub use compact::{compact, CompactStats};
pub use manager::{CollectionHandle, CollectionManager};
pub use near_duplicates::find_duplicates;
pub use open::CollectionOpenOptions;
pub use state::Collection;
