use serde::{Deserialize, Serialize};

// pub struct for individual audit results
#[derive(Debug, Deserialize, Serialize)]
pub struct Audit {
    pub score: Option<f64>,
}
