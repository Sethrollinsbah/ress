use serde::{Deserialize, Serialize};

// pub struct to store score statistics (min, max, median, std deviation)
#[derive(Debug, Deserialize, Serialize)]
pub struct ScoreStats {
    pub min: f64,
    pub max: f64,
    pub median: f64,
    pub std_dev: f64,
}

// pub struct for categories with statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct CategoriesStats {
    pub performance: Option<ScoreStats>,
    pub accessibility: Option<ScoreStats>,
    pub best_practices: Option<ScoreStats>,
    pub seo: Option<ScoreStats>,
    pub pwa: Option<ScoreStats>,
}
