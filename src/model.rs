use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct ComprehensiveReport {
    pub category_stats: CategoriesStats,
    pub best_performance_page: Option<String>,
    pub worst_performance_page: Option<String>,
    pub common_failing_audits: Vec<String>,
    pub lighthouse_reports: Vec<Root>,
}

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

// pub struct for each category score
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Category {
    pub score: Option<f64>,
}

// pub struct for Lighthouse categories
#[derive(Debug, Deserialize, Serialize)]
pub struct Categories {
    pub performance: Option<Category>,
    pub accessibility: Option<Category>,
    pub best_practices: Option<Category>,
    pub seo: Option<Category>,
    pub pwa: Option<Category>,
}

// Root pub structure for the Lighthouse report
#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Root {
    pub requestedUrl: String, // Store URL for best/worst pages
    pub categories: Categories,
    pub audits: HashMap<String, Audit>, // Store audit results dynamically
}

// pub struct for individual audit results
#[derive(Debug, Deserialize, Serialize)]
pub struct Audit {
    pub score: Option<f64>,
}

// pub struct for final report with statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct AverageReport {
    pub category_stats: CategoriesStats,
    pub best_performance_page: Option<String>,
    pub worst_performance_page: Option<String>,
    pub common_failing_audits: Vec<String>,
}
