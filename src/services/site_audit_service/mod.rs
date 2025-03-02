// src/lighthouse/mod.rs

pub mod compute;
pub mod lighthouse;

pub use compute::{compute_averages, compute_score_stats};
pub use lighthouse::{run_lighthouse, run_lighthouse_process};
