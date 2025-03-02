use crate::models::lighthouse::audit::Audit;
use crate::models::lighthouse::category::{Categories, CategoriesStats};
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

#[derive(Debug, Serialize, Deserialize)]
pub struct AverageReport {
    pub category_stats: CategoriesStats,
    pub best_performance_page: Option<String>,
    pub worst_performance_page: Option<String>,
    pub common_failing_audits: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize)]
#[allow(non_snake_case)]
pub struct Root {
    pub requestedUrl: String,
    pub categories: Categories,
    pub audits: HashMap<String, Audit>,
}
