use crate::models::update_cloudflare_kv;
use crate::models::ComprehensiveReport;
use crate::utils::bun_log;
use anyhow::{Context, Result};
use std::fs::File;
use tokio::fs;

pub async fn save_report(
    output_path: &str,
    report: &ComprehensiveReport,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = File::create(output_path)?;
    serde_json::to_writer_pretty(file, report)?;
    Ok(())
}

pub fn sanitize_filename(url: &str) -> String {
    url.replace(|c: char| !c.is_alphanumeric() && c != '.', &'_'.to_string())
}

pub async fn delete_reports(report_id: &str) -> Result<()> {
    let _ = bun_log(&report_id, "Running cleanup on servers");
    let email_list = vec!["sethryanrollins@gmail.com".to_string()];
    match update_cloudflare_kv(&report_id, email_list).await {
        Ok(response) => {
            println!("Status: {:?}", response);
        }
        Err(e) => eprintln!("Error making request: {}", e),
    }
    let current_dir = std::env::current_dir().context("Failed to get current directory")?;

    // Define paths using PathBuf for better cross-platform compatibility
    let paths = [
        (
            "directory",
            current_dir.join("lighthouse_reports").join(report_id),
        ),
        (
            "http report",
            current_dir.join("http").join(format!("{}.txt", report_id)),
        ),
        (
            "json report",
            current_dir.join(format!(
                "comprehensive_lighthouse_{}_report.json",
                report_id
            )),
        ),
    ];

    // Process each path
    for (path_type, path) in paths.iter() {
        if path.exists() {
            match path_type {
                &"directory" => {
                    fs::remove_dir_all(path).await.with_context(|| {
                        format!("Failed to delete directory: {}", path.display())
                    })?;
                }
                _ => {
                    fs::remove_file(path)
                        .await
                        .with_context(|| format!("Failed to delete file: {}", path.display()))?;
                }
            }
        }
    }
    let _ = bun_log(&report_id, "Cleanup process complete");
    let _ = bun_log(&report_id, "$REDIRECT::goto");
    fs::remove_file(&format!("/tmp/reports/{}.txt", &report_id))
        .await
        .with_context(|| "Failed to delete file")?;

    Ok(())
}
