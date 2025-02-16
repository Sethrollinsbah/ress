use crate::lighthouse;
use crate::lighthouse::compute_averages;
use crate::lighthouse::save_report;
use crate::mail;
use crate::model;
use axum::{extract::Query, response::IntoResponse, Json};
use serde::Deserialize;
use serde_json::json;
use std::fs;
use tokio;

#[derive(Deserialize)]
pub struct ParamsRunLighthouse {
    domain: String,
    email: String,
    name: String,
}

pub async fn run_lighthouse_handler(Query(params): Query<ParamsRunLighthouse>) -> &'static str {
    tokio::task::spawn(async move {
        run_lighthouse_process(params.domain, params.email, params.name).await;
    });
    // match run_lighthouse_process(params.domain, params.email, params.name).await {
    //     Ok(_) => Json(json!({"status": "success"})),
    //     Err(e) => Json(json!({
    //         "status": "error",
    //         "message": e.to_string()
    //     })),
    // }
    "OK"
}

async fn run_lighthouse_process(
    domain: String,
    email: String,
    name: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
            Scanning: <span id=\"domain\">{}</span>
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
        <button href=\"https://planetbun.com/en/contact\" onclick=\"window.location.href='/consultation'\">
            Get Your Free Consultation
        </button>
        
        <p style=\"margin-top: 1rem; font-size: 0.9rem; color: #718096;\">
            Discover more opportunities to improve your website with our expert consultation
        </p>
    </div>
    <script>
        // Replace domain in the span
        const urlParams = new URLSearchParams(window.location.search);
        const domain = urlParams.get('domain') || 'example.com';
        document.getElementById('domain').textContent = domain;
    </script>
</body>
</html>", &domain);

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
                        <li style=\"margin-bottom: 10px;\">Analysis of {} unique URLs on your website</li>
                        <li style=\"margin-bottom: 10px;\">Complete Lighthouse performance scores</li>
                        <li style=\"margin-bottom: 10px;\">Security vulnerability checks</li>
                        <li style=\"margin-bottom: 10px;\">SEO optimization opportunities</li>
                    </ul>
                </div>
                <div style=\"margin: 30px 0;\">
                    <a href=\"https://planetbun.com/en/eval/{}\" style=\"background-color: #4299e1; color: #ffffff; padding: 12px 24px; border-radius: 6px; text-decoration: none; font-weight: bold; display: inline-block;\">
                        View Your Full Report
                    </a>
                </div>
                <p style=\"color: #718096; font-size: 14px; line-height: 1.6; margin: 20px 0 0;\">
                    Want to discuss your results with our experts?<br>
                    <a href=\"https://planetbun.com/en/contact\" style=\"color: #4299e1; text-decoration: underline;\">
                        Schedule a free consultation
                    </a>
                </p>
            </td>
        </tr>
        <tr>
            <td style=\"padding: 30px; text-align: center; background-color: #f8fafc; border-top: 1px solid #e2e8f0;\">
                <p style=\"color: #718096; font-size: 14px; margin: 0;\">
                    © 2025 PlanetBun Dev Shop<br>
                    <a href=\"[unsubscribe-url]\" style=\"color: #718096; text-decoration: underline;\">Unsubscribe</a>
                </p>
            </td>
        </tr>
    </table>
</body>
</html>", &domain, &domain, &domain);

    let _starting_message =
        mail::send_mail(&domain, &email, &name, &starting_subject, &starting_message).await?;
    let args: Vec<String> = std::env::args().collect();
    let domain_tld = domain;
    let https_domain_tld = format!("https://{}.txt", &domain_tld);

    std::process::Command::new("bun")
        .args([
            "/root/lightavg/unlit/index.ts",
            // "cp",
            &format!("siteUrl=https://{}", &domain_tld),
            "maxLinks=100",
        ])
        .status()?;

    println!("file_name: {}", &domain_tld);
    let output_folder = format!("/root/lightavg/lighthouse_reports/{}.txt/", &domain_tld);
    fs::create_dir_all(&output_folder)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    ;
    let urls = lighthouse::read_urls_from_file(&format!("./https:/{}.txt", &domain_tld)).await?;
    

    println!("urls: {:?}", &urls);
    let mut handles = Vec::new();
    for url in urls {
        println!("here: {}", &url);
        let baseurl = domain_tld.to_string();
        println!("base_url: {}", &baseurl);
        let output_path = format!(
            "/root/lightavg/lighthouse_reports/https:/planetbun.com.txt/",
            // &baseurl,
            // url.to_string().replacen("/", "", 1).replace("/", "___")
        );
        println!("output_path: {:?}", &output_path);
        fs::create_dir_all(&output_path)
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

        handles.push(tokio::spawn(async move {
            println!("hre");
            // lighthouse::run_lighthouse(&url, &baseurl, &output_path).await

            match lighthouse::run_lighthouse(&url, &baseurl, &output_path).await {
                Ok(_) => println!("Lighthouse ran successfully"),
                Err(e) => eprintln!("Error running Lighthouse: ", ),
            }
        }));
    }

    for handle in handles {
        let _ = handle.await?;
    }

    let directory = format!(
        "/root/lightavg/lighthouse_reports/https:/{}.txt/",
        domain_tld
    );
    let mut reports = Vec::new();

    for entry in fs::read_dir(directory)? {
        let entry = entry?;
        if entry.path().is_file() {
            reports.push(serde_json::from_reader(std::io::BufReader::new(
                std::fs::File::open(entry.path())?,
            ))?);
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

    let output_path = format!(
        "/root/lightavg/comprehensive_lighthouse_{}_report.json",
        domain_tld
    );
    save_report(&output_path, &comprehensive_report).await?;

    let status = std::process::Command::new("aws")
        .args([
            "s3",
            "cp",
            &output_path,
            &format!("s3://planet-bun/reports/{}.json", domain_tld),
            "--endpoint-url",
            "https://0e9b5fad61935c0d6483962f4a522a89.r2.cloudflarestorage.com",
            "--checksum-algorithm",
            "CRC32",
        ])
        .status()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

    let _completion_message = mail::send_mail(
        &domain_tld,
        &email,
        &name,
        &completion_subject,
        &completion_mail,
    )
    .await?;
    Ok(())
}
