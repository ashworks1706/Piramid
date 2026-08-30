//! Thread-pool settings.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ParallelismMode {
    /// One thread.
    SingleThreaded,
    /// One thread per core.
    #[default]
    Auto,
    /// A fixed thread count.
    Fixed(usize),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ParallelismConfig {
    pub mode: ParallelismMode,

    /// Fan batch searches across the pool.
    pub parallel_search: bool,
}

impl Default for ParallelismConfig {
    fn default() -> Self {
        ParallelismConfig {
            mode: ParallelismMode::Auto,
            parallel_search: true,
        }
    }
}

impl ParallelismConfig {
    pub fn single_threaded() -> Self {
        ParallelismConfig {
            mode: ParallelismMode::SingleThreaded,
            parallel_search: false,
        }
    }

    /// Resolved thread count. Zero means let rayon decide.
    pub fn num_threads(&self) -> usize {
        match self.mode {
            ParallelismMode::SingleThreaded => 1,
            ParallelismMode::Auto => num_cpus::get(),
            ParallelismMode::Fixed(n) => n,
        }
    }

    pub fn with_num_threads(mut self, n: usize) -> Self {
        self.mode = ParallelismMode::Fixed(n);
        self.parallel_search = n > 1;
        self
    }
}
