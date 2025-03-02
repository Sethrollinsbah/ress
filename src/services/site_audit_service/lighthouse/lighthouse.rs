// use crate::site_audit::;
use crate::utils::{process_urls, sanitize_filename, save_report};

use anyhow;
use anyhow::{Context, Result};
use axum::extract::Query;
use chrono::Utc;
use futures::StreamExt;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::OpenOptions;
use std::io::{self};
use tokio::io::AsyncReadExt;
use tokio_stream::wrappers::ReadDirStream;
//

use crate::api;
use crate::mail;
use crate::models;
use crate::models::AverageReport;
use crate::models::CategoriesStats;
use crate::models::ComprehensiveReport;
use crate::models::Root;
use crate::models::ScoreStats;
use crate::services::{compute_averages, compute_score_stats};
use crate::utils::bun_log;
use crate::utils::delete_reports;
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

    let _ = bun_log(&baseurl, &format!("Lighthouse report saved for URL"));
    Ok(())
}

pub async fn run_lighthouse_process(
    domain: String,
    email: String,
    name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let file = std::fs::File::create(format!("/tmp/reports/{}.txt", &domain));
    let _ = bun_log(
        &domain,
        &format!("Initializing website crawl on {}", &domain),
    );
    let starting_subject = format!("Website Scan in Progress: {}", &domain);
    let starting_message = format!("<!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>PlanetBun Dev Shop - Website Scanner</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, sans-serif;
            margin: 0;
            min-height: 100vh;
            display: flex;
            flex-direction: column;
            align-items: center;
            justify-content: center;
            background: linear-gradient(135deg, #f5f7fa 0%, #e4e9f2 100%);
            color: #2d3748;
            padding: 20px;
        }}
        .container {{
            background: white;
            padding: 2rem;
            border-radius: 12px;
            box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
            max-width: 600px;
            width: 90%;
            text-align: center;
        }}
        .logo {{
            margin-bottom: 2rem;
            font-size: 1.5rem;
            font-weight: bold;
            color: #4a5568;
        }}
        .scanner {{
            margin: 2rem 0;
        }}
        .progress {{
            width: 100%;
            height: 4px;
            background: #e2e8f0;
            border-radius: 2px;
            overflow: hidden;
            position: relative;
        }}
        .progress-bar {{
            position: absolute;
            height: 100%;
            background: #4299e1;
            animation: scan 2s ease-in-out infinite;
            width: 30%;
        }}
        .domain {{
            font-size: 1.25rem;
            color: #2b6cb0;
            margin: 1rem 0;
        }}
        .status {{
            color: #4a5568;
            margin: 1rem 0;
        }}
        .cta-button {{
            background: #4299e1;
            color: white;
            border: none;
            padding: 12px 24px;
            border-radius: 6px;
            font-size: 1rem;
            cursor: pointer;
            transition: background 0.3s ease;
            margin-top: 2rem;
        }}
        .cta-button:hover {{
            background: #2b6cb0;
        }}
        @keyframes scan {{
            0% {{
                left: -30%;
            }}
            100% {{
                left: 100%;
            }}
        }}
        .findings {{
            margin: 2rem 0;
            text-align: left;
        }}
        .finding-item {{
            display: flex;
            align-items: center;
            margin: 0.5rem 0;
            opacity: 0;
            animation: fadeIn 0.5s ease forwards;
        }}
        .finding-item::before {{
            content: \"•\";
            color: #4299e1;
            margin-right: 0.5rem;
        }}
        @keyframes fadeIn {{
            to {{
                opacity: 1;
            }}
        }}
    </style>
</head>
<body>
    <div class=\"container\">
        <div class=\"logo\">
            🌍 PlanetBun Dev Shop
        </div>
        
        <h1>Website Scanner</h1>
        
        <div class=\"domain\">
            Scanning: <span id=\"{}\">{}</span>
        </div>
        
        <div class=\"scanner\">
            <div class=\"progress\">
                <div class=\"progress-bar\"></div>
            </div>
            <p class=\"status\">Analyzing website performance and security...</p>
        </div>
        <div class=\"findings\">
            <div class=\"finding-item\">Checking page load speed optimization...</div>
            <div class=\"finding-item\" style=\"animation-delay: 1s\">Analyzing mobile responsiveness...</div>
            <div class=\"finding-item\" style=\"animation-delay: 2s\">Evaluating SEO performance...</div>
            <div class=\"finding-item\" style=\"animation-delay: 3s\">Scanning security vulnerabilities...</div>
        </div>
        <button href=\"https://planetbun.com/en/quiz?contact\" onclick=\"window.location.href='/en/quiz?contact'\">
            Get Your Free Consultation
        </button>
        
        <p style=\"margin-top: 1rem; font-size: 0.9rem; color: #718096;\">
            Discover more opportunities to improve your website with our expert consultation
        </p>
    </div>
    <script>
        // Replace domain in the span
        const urlParams = new URLSearchParams(window.location.search);
        const domain = '{}';
    </script>
</body>
</html>", &domain,  &domain,  &domain);

    let completion_subject = format!("Website Scan Results: {}", &domain);
    let completion_mail = format!("
        <!DOCTYPE html>
<html lang=\"en\">
<head>
    <meta charset=\"UTF-8\">
    <meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">
    <title>Your Website Scan Results - PlanetBun Dev Shop</title>
</head>
<body style=\"margin: 0; padding: 0; font-family: Arial, sans-serif; background-color: #f5f7fa; color: #2d3748;\">
    <table role=\"presentation\" style=\"width: 100%; max-width: 600px; margin: 0 auto; background-color: #ffffff; padding: 0; border-spacing: 0; border-collapse: collapse;\">
        <tr>
            <td style=\"padding: 40px 30px; text-align: center; background-color: #ffffff;\">
                <h1 style=\"margin: 0; font-size: 24px; color: #4a5568;\">
                    🌍 PlanetBun Dev Shop
                </h1>
            </td>
        </tr>
        <tr>
            <td style=\"padding: 0 30px 30px; text-align: center; background-color: #ffffff;\">
                <h2 style=\"color: #2d3748; font-size: 20px; margin: 0 0 20px;\">
                    Your Website Scan is Complete!
                </h2>
                <p style=\"color: #4a5568; font-size: 16px; line-height: 1.6; margin: 0 0 20px;\">
                    We've completed a comprehensive analysis of <span style=\"color: #2b6cb0; font-weight: bold;\">{}</span>
                </p>
                <div style=\"background-color: #f8fafc; border-radius: 8px; padding: 20px; margin: 20px 0; text-align: left;\">
                    <p style=\"color: #4a5568; font-size: 16px; line-height: 1.6; margin: 0 0 10px;\">
                        Our scan included:
                    </p>
                    <ul style=\"color: #4a5568; font-size: 16px; line-height: 1.6; margin: 0; padding-left: 20px;\">
                        <li style=\"margin-bottom: 10px;\">Analysis of a set of unique URLs on your website</li>
                        <li style=\"margin-bottom: 10px;\">Complete Lighthouse performance scores</li>
                        <li style=\"margin-bottom: 10px;\">Security vulnerability checks</li>
                        <li style=\"margin-bottom: 10px;\">SEO optimization opportunities</li>
                    </ul>
                </div>
                <div style=\"margin: 30px 0;\">
                    <a href=\"https://planetbun.com/en/tools/scanova/{}\" style=\"background-color: #4299e1; color: #ffffff; padding: 12px 24px; border-radius: 6px; text-decoration: none; font-weight: bold; display: inline-block;\">
                        View Your Full Report
                    </a>
                </div>
                <p style=\"color: #718096; font-size: 14px; line-height: 1.6; margin: 20px 0 0;\">
                    Want to discuss your results with our experts?<br>
                    <a href=\"https://planetbun.com/en/home?contact=true\" style=\"color: #4299e1; text-decoration: underline;\">
                        Schedule a free consultation
                    </a>
                </p>
            </td>
        </tr>
        <tr>
            <td style=\"padding: 30px; text-align: center; background-color: #f8fafc; border-top: 1px solid #e2e8f0;\">
                <p style=\"color: #718096; font-size: 14px; margin: 0;\">
                    © 2025 PlanetBun Dev Shop<br>
                    <a href=\"https://planetbun.com/api/emailCampaign/unsubscribe?email={}\" style=\"color: #718096; text-decoration: underline;\">Unsubscribe</a>
                </p>
            </td>
        </tr>
    </table>
</body>
</html>", &domain, &domain, &email);

    let _starting_message =
        mail::send_mail(&domain, &email, &name, &starting_subject, &starting_message).await?;
    let domain_tld = domain.clone();

    let current_dir = std::env::current_dir()?
        .to_str()
        .ok_or("Failed to convert current directory to string")?
        .to_string();

    let _ = bun_log(&domain, "Starting to crawl the domain.");
    std::process::Command::new("bun")
        .args([
            &format!("{}/unlit/index.ts", &current_dir),
            &format!("siteUrl=https://{}", &domain_tld),
            "maxLinks=100",
        ])
        .status()?;

    // println!("file_name: {}", &current_dir);

    if let Err(e) = process_urls(&current_dir, &domain_tld).await {
        // eprintln!("❌ process_urls failed: {}", e);
        let _ = bun_log(&domain, "❌ Error: Failed to process urls.");
        return Err(e.into()); // Ensure the error propagates if necessary
    } else {
        let _ = bun_log(&domain, "✅ Successfully processed all urls found");
    }

    let directory = format!("{}/lighthouse_reports/{}/", &current_dir, domain_tld);
    let mut reports = Vec::new();

    // Use ReadDirStream correctly
    let dir_stream = tokio::fs::read_dir(directory).await?; // .await here is correct for tokio::fs::read_dir

    // Convert it to ReadDirStream
    let mut stream = ReadDirStream::new(dir_stream);

    while let Some(entry) = stream.next().await {
        let entry = entry?;

        let _ = bun_log(&domain, &format!("🔍 Processing entry: {:?}", entry.path()));

        if entry.path().is_file() {
            let _ = bun_log(&domain, &format!("📄 Found file: {:?}", entry.path()));

            // Try to open the file asynchronously
            match tokio::fs::File::open(entry.path()).await {
                Ok(mut file) => {
                    let _ = bun_log(
                        &domain,
                        &format!("📝 File opened successfully: {:?}", entry.path()),
                    );

                    let mut buffer = Vec::new();
                    match file.read_to_end(&mut buffer).await {
                        Ok(_) => {
                            let _ = bun_log(
                                &domain,
                                &format!(
                                    "📚 File read successfully into buffer. Size: {}",
                                    buffer.len()
                                ),
                            );

                            // Try to parse the JSON from the buffer
                            match serde_json::from_slice::<Root>(&buffer) {
                                Ok(report) => {
                                    let _ = bun_log(
                                        &domain,
                                        &format!(
                                            "✅ Successfully parsed JSON for: {:?}",
                                            entry.path()
                                        ),
                                    );
                                    reports.push(report);
                                }
                                Err(_e) => {
                                    let _ = bun_log(
                                        &domain,
                                        &format!(
                                            "❌ Error parsing JSON from file {:?}",
                                            entry.path()
                                        ),
                                    );
                                    // eprintln!(
                                    //     entry.path(),
                                    //     e
                                    // );
                                }
                            }
                        }
                        Err(_e) => {
                            let _ = bun_log(
                                &domain,
                                &format!("❌ Error reading file  {:?}", entry.path()),
                            );
                            // eprintln!("❌ Error reading file {:?}: {}", entry.path(), e);
                        }
                    }
                }
                Err(_e) => {
                    let _ = bun_log(
                        &domain,
                        &format!("❌ EError opening file  {:?}", entry.path()),
                    );
                    // eprintln!("❌ {:?}: {}", entry.path(), e);
                }
            }
        } else {
            let _ = bun_log(
                &domain,
                &format!("❌ Skipping non-file entry: {:?}", entry.path()),
            );
        }
    }

    let average_report = compute_averages(&reports);
    let _ = bun_log(&domain, "✅ Averages computed for reports");

    let comprehensive_report = models::ComprehensiveReport {
        category_stats: average_report.category_stats,
        best_performance_page: average_report.best_performance_page,
        worst_performance_page: average_report.worst_performance_page,
        common_failing_audits: average_report.common_failing_audits,
        lighthouse_reports: reports,
    };

    let _ = bun_log(&domain, "✅ Comprehensive report generated");
    let output_path = format!(
        "{}/comprehensive_lighthouse_{}_report.json",
        &current_dir, domain_tld
    );

    // Save the comprehensive report
    match save_report(&output_path, &comprehensive_report).await {
        Ok(_) => {
            let _ = bun_log(&domain, "✅ Comprehensive report generated");
            println!("✅ Report saved successfully at: {}", &output_path);
        }
        Err(_) => bun_log(
            &domain,
            &format!("❌ Error saving report at {}", &output_path),
        )?,
    };

    // Upload the report to S3
    let status = std::process::Command::new("aws")
        .args([
            "s3",
            "cp",
            &output_path,
            &format!("s3://planet-bun/reports/{}.json", &domain_tld),
            "--endpoint-url",
            "https://0e9b5fad61935c0d6483962f4a522a89.r2.cloudflarestorage.com",
            "--checksum-algorithm",
            "CRC32",
        ])
        .status()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    if status.success() {
        println!("");
        let _ = bun_log(
            &domain,
            "✅ Report successfully uploaded to PlanetBun bucket",
        );
    } else {
        let _ = bun_log(&domain, "❌ Failed to upload report to PlanetBun bucket");
    }

    // Send completion email
    match mail::send_mail(
        &domain_tld,
        &email,
        &name,
        &completion_subject,
        &completion_mail,
    )
    .await
    {
        Ok(_) => {
            let _ = bun_log(
                &domain,
                &format!("✅ Completion email sent successfully to: {}", email),
            );
        }
        Err(e) => {
            let _ = bun_log(&domain, "❌ Error sending completion email");
        }
    }

    let _ = delete_reports(&domain_tld).await;

    Ok(())
}
