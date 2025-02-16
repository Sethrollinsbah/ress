use std::{
    collections::HashMap,
    fs::{File, OpenOptions},
    io::{BufRead, BufReader, Write},
    path::Path,
    time::Instant,
    process::{Command, Stdio},
};

use crate::model::AverageReport;
use crate::model::CategoriesStats;
use crate::model::ComprehensiveReport;
use crate::model::Root;
use crate::model::ScoreStats;
// Function to run Lighthouse on a given URL

pub async fn run_lighthouse(
    url: &str,
    baseurl: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Construct the full URL
    let full_url = format!("https://{}{}", baseurl, url);
    println!("Testing URL: {}", full_url);
    println!("Output path: {}", output_path);

    // Execute the Lighthouse command
    let command = Command::new("lighthouse")
        .arg(&full_url) // URL to test
        .arg("--output") // Specify output format
        .arg("json") // Output format is JSON
        .arg("--chrome-flags=--headless") // Chrome flags
        .arg("--chrome-flags=--no-sandbox") // Chrome flags
        .arg("--output-path") // Specify where to save the output
        .arg(output_path) // Path to save the JSON output
        .stderr(Stdio::piped()) // Capture stderr
        .spawn()?;

    // Wait for the command to finish and capture output
    let output = command.wait_with_output()?;

    // Print stdout and stderr for debugging
    println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Check if the command succeeded
    if !output.status.success() {
        return Err(format!(
            "Lighthouse failed for {}: {}",
            full_url,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    println!("Lighthouse report saved for URL: {}", full_url);
    Ok(())
}


pub async fn read_urls_from_file(
    file_path: &str,
) -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    if !Path::new(file_path).exists() {
        let mut file = File::create(file_path)?;
        file.write_all(b"")?;
        println!("Created new file: {}", file_path);
    }

    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let urls: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();
    
    println!("Read {} URLs", urls.len());
    Ok(urls)
}

fn compute_score_stats(scores: &mut Vec<f64>) -> ScoreStats {
    scores.sort_by(|a, b| a.partial_cmp(b).unwrap());

    let count = scores.len();
    let sum: f64 = scores.iter().sum();
    let mean = sum / count as f64;

    let variance = scores.iter().map(|&x| (x - mean).powi(2)).sum::<f64>() / count as f64;
    let std_dev = variance.sqrt();

    let median = if count % 2 == 0 {
        (scores[count / 2 - 1] + scores[count / 2]) / 2.0
    } else {
        scores[count / 2]
    };

    ScoreStats {
        min: *scores.first().unwrap(),
        max: *scores.last().unwrap(),
        median,
        std_dev,
    }
}

pub fn compute_averages(reports: &[Root]) -> AverageReport {
    let mut scores: HashMap<&str, Vec<f64>> = HashMap::new();
    let mut audit_fails: HashMap<String, u32> = HashMap::new();
    let mut best_page = (None, 0.0);
    let mut worst_page = (None, 1.0);

    for report in reports {
        let url = &report.requestedUrl;
        let categories = &report.categories;

        let fields = [
            (
                "performance",
                categories.performance.as_ref().and_then(|c| c.score),
            ),
            (
                "accessibility",
                categories.accessibility.as_ref().and_then(|c| c.score),
            ),
            (
                "best_practices",
                categories.best_practices.as_ref().and_then(|c| c.score),
            ),
            ("seo", categories.seo.as_ref().and_then(|c| c.score)),
            ("pwa", categories.pwa.as_ref().and_then(|c| c.score)),
        ];

        for (key, value) in fields.iter() {
            if let Some(score) = value {
                scores.entry(key).or_insert(Vec::new()).push(*score);

                if *key == "performance" {
                    if *score > best_page.1 {
                        best_page = (Some(url.clone()), *score);
                    }
                    if *score < worst_page.1 {
                        worst_page = (Some(url.clone()), *score);
                    }
                }
            }
        }

        // Track common failing audits (score < 0.5)
        for (audit_name, audit) in &report.audits {
            if let Some(score) = audit.score {
                if score < 0.5 {
                    *audit_fails.entry(audit_name.clone()).or_insert(0) += 1;
                }
            }
        }
    }

    let compute_stat = |key: &str| {
        scores
            .get(key)
            .map(|values| compute_score_stats(&mut values.clone()))
    };

    // Sort common failing audits by frequency
    let mut sorted_audits: Vec<_> = audit_fails.into_iter().collect();
    sorted_audits.sort_by(|a, b| b.1.cmp(&a.1)); // Sort descending

    AverageReport {
        category_stats: CategoriesStats {
            performance: compute_stat("performance"),
            accessibility: compute_stat("accessibility"),
            best_practices: compute_stat("best_practices"),
            seo: compute_stat("seo"),
            pwa: compute_stat("pwa"),
        },
        best_performance_page: best_page.0.clone(),
        worst_performance_page: worst_page.0.clone(),
        common_failing_audits: sorted_audits
            .into_iter()
            .take(5)
            .map(|(name, _)| name)
            .collect(),
    }
}

pub async fn save_report(
    output_path: &str,
    report: &ComprehensiveReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(())
}
