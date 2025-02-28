// src/lighthouse/mod.rs

pub mod lighthouse;
pub mod compute;

pub use lighthouse::{run_lighthouse, run_lighthouse_process};
pub use compute::{compute_score_stats, compute_averages};
