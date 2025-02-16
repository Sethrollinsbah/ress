use axum::{routing::post, Router};
use tower_http::cors::CorsLayer;
use crate::lighthouse::run_lighthouse;
use std::{
    collections::HashMap,
    error::Error,
    fs::{self, File},
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
};
use tokio;
use tokio::task;
use crate::lighthouse::save_report;
use crate::lighthouse::compute_averages;
// Function to run Lighthouse on a given URL

async fn notmain() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = std::env::args().collect();
    let file_name = &args[1];
    let file_name_1 = &("https:/".to_owned() + &args[1] + ".txt");
    let key = &file_name;
    let output = std::process::Command::new("bun")
        .args([
            "unlit/index.ts",
            "cp",
            &("siteUrl=https://".to_owned() + &file_name),
            &("maxLinks=100"),
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(output) => {
            dbg!("Command failed: {}", output);
        }
        Err(err) => {
            eprintln!("Command failed to execute");
        }
    }
    let output_folder = &("./lighthouse_reports/".to_owned() + &file_name_1 + "/");

    // Create output folder if not exists
    fs::create_dir_all(output_folder)?;

    // Read URLs from a text file
    let urls = lighthouse::read_urls_from_file(&file_name_1)?;
    println!("Found {:?} URLs.", urls.len());

    // Vector to hold the join handles of the spawned tasks
    let mut handles = Vec::new();

    for url in urls {
        let baseurl = file_name.clone();
        let output_path = format!("{}/{}_report.json", output_folder, url.replace("/", "_"));

        // Spawn a new asynchronous task for each URL
        let handle = task::spawn(async move {
            if let Err(e) = lighthouse::run_lighthouse(&url, &baseurl, &output_path).await {
                eprintln!("Error running lighthouse for {}: {}", url, e);
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await?;
    }

    println!("All Lighthouse reports have been generated.{}", file_name);

    let directory = format!("./lighthouse_reports/https:/{}.txt/", file_name);
    let mut reports: Vec<model::Root> = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        println!("{:?}", entry);
        if entry.path().is_file() {
            //         // Load the generated report and add to the reports list
            let file = File::open(&entry.path())?;
            let reader = BufReader::new(file);
            let report: model::Root = serde_json::from_reader(reader)?;
            reports.push(report);
        }
    }

    let average_report = compute_averages(&reports);

    let comprehensive_report = model::ComprehensiveReport {
        category_stats: average_report.category_stats,
        best_performance_page: average_report.best_performance_page,
        worst_performance_page: average_report.worst_performance_page,
        common_failing_audits: average_report.common_failing_audits,
        lighthouse_reports: reports,
    };

    let output_path = &("./comprehensive_lighthouse_".to_owned() + &file_name + "_report.json");
    save_report(output_path, &comprehensive_report)?;

    println!("Comprehensive Lighthouse report saved to: {}", output_path);

    let output = std::process::Command::new("aws")
        .args([
            "s3",
            "cp",
            &("./comprehensive_lighthouse_".to_owned() + &file_name + "_report.json"),
            &("s3://planet-bun/reports/".to_owned() + file_name + ".json"),
            "--endpoint-url",
            "https://0e9b5fad61935c0d6483962f4a522a89.r2.cloudflarestorage.com",
            "--checksum-algorithm",
            "CRC32",
        ])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            println!("{}", String::from_utf8_lossy(&output.stdout));
        }
        Ok(output) => {
            dbg!("Command failed: {}", output);
        }
        Err(err) => {
            eprintln!("Command failed to execute");
        }
    }
    Ok(())
}

pub fn create_router() -> Router {
    Router::new()
        .route("/run_lighthouse", post(notmain))
        .layer(CorsLayer::permissive())
}

