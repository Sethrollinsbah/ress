use crate::api;
use crate::model::AverageReport;
use crate::model::CategoriesStats;
use crate::model::ComprehensiveReport;
use crate::model::Root;
use crate::model::ScoreStats;
use futures::future::join_all;
use std::{
    collections::HashMap,
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
    process::{Command, Stdio},
};
use tokio::{fs, task};
// Function to run Lighthouse on a given URL

pub async fn run_lighthouse(
    url: &str,
    baseurl: &str,
    output_path: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let full_url = format!("https://{}{}", baseurl, url);
    let full_local_path = format!("{}/{}.json", &output_path, sanitize_filename(&url));

    // Execute the Lighthouse command with corrected arguments
    let command = Command::new("lighthouse")
        .arg(&full_url)
        .arg("--output=json") // Combine output format flag
        .arg("--no-enable-error-reporting")
        .arg("--chrome-flags=\"--headless --no-sandbox\"") // Combine Chrome flags
        .arg("--max-wait-for-load=120000")
        .arg("--output-path")
        .arg(full_local_path)
        .stdout(Stdio::piped()) // Capture stdout as well
        .stderr(Stdio::piped())
        .spawn()?;

    // Wait for the command to finish and capture output
    let output = command.wait_with_output()?;

    // Print stdout and stderr for debugging
    // println!("stdout: {}", String::from_utf8_lossy(&output.stdout));
    // println!("stderr: {}", String::from_utf8_lossy(&output.stderr));

    // Check if the command succeeded
    if !output.status.success() {
        return Err(format!(
            "Lighthouse failed for {}: {}",
            full_url,
            String::from_utf8_lossy(&output.stderr)
        )
        .into());
    }

    let _ = api::bun_log(&baseurl, &format!("Lighthouse report saved for URL"));
    Ok(())
}

pub async fn process_urls_from_file(
    file_path: &str,
    _output_folder: &str,
) -> Result<Vec<String>, Box<dyn Error + Send + Sync>> {
    // ✅ Change return type to Vec<String>
    // Ensure the file exists
    if !Path::new(file_path).exists() {
        let mut file = File::create(file_path)?;
        file.write_all(b"")?;
        println!("Created new file: {}", file_path);
    }

    // Read URLs from the file
    let file = File::open(file_path)?;
    let reader = BufReader::new(file);
    let urls: Vec<String> = reader.lines().filter_map(|line| line.ok()).collect();

    Ok(urls) // ✅ Return the vector of URLs
}

pub async fn process_urls(
    current_dir: &str,
    domain_tld: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Read URLs from file
    let input_file = format!("{}/http/{}.txt", current_dir, domain_tld);
    let output_dir = format!(
        "{}/lighthouse_reports/{}",
        current_dir,
        domain_tld.replace("/", "___")
    );

    let urls = process_urls_from_file(&input_file, &output_dir).await?; // ✅ Now `urls` is a Vec<String>
                                                                        // println!("URLs: {:?}", &urls);

    // Ensure the output directory exists
    fs::create_dir_all(&output_dir).await?;

    let mut handles = Vec::new();

    let _ = api::bun_log(&domain_tld, &format!("$FOUND_URL::{:?}", urls));
    for url in urls {
        // ✅ Now `urls` is iterable
        let baseurl = domain_tld.to_string();
        let output_path = output_dir.clone(); // Clone since `output_path` is moved into async block

        handles.push(task::spawn(async move {
            println!("BASEURL {}", &baseurl);
            let _ = api::bun_log(&baseurl, &format!("Processing URL: {}", url));
            match run_lighthouse(&url, &baseurl, &output_path).await {
                Ok(_) => {
                    let _ = api::bun_log(
                        &baseurl,
                        &format!("Lighthouse ran successfully for {}", url),
                    );
                    let _ = api::bun_log(&baseurl, &format!("$SUCCESS_URL::{}", url));
                }
                Err(e) => {
                    let _ = api::bun_log(
                        &baseurl,
                        &format!("ERROR: Error running Lighthouse for {}: {:?}", url, e),
                    );
                    let _ = api::bun_log(&baseurl, &format!("$FAIL_URL::{}", url));
                }
            }
        }));
    }

    let _ = api::bun_log(
        &domain_tld,
        "✅ All Lighthouse tasks spawned, waiting for completion...",
    );

    join_all(handles).await; // Wait for all tasks to finish

    let _ = api::bun_log(
        &domain_tld,
        "✅ Lighthouse processing completed for all URLs",
    );
    Ok(())
}

// Helper function to sanitize filenames
fn sanitize_filename(url: &str) -> String {
    url.replace(|c: char| !c.is_alphanumeric() && c != '.', &'_'.to_string())
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
