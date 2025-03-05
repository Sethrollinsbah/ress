use crate::services;
use crate::utils::bun_log;
use futures::future::join_all;
use std::{
    error::Error,
    fs::File,
    io::{BufRead, BufReader, Write},
    path::Path,
};
use tokio::{fs, task};

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

    for url in urls {
        // ✅ Now `urls` is iterable
        let baseurl = domain_tld.to_string();
        let output_path = output_dir.clone(); // Clone since `output_path` is moved into async block

        handles.push(task::spawn(async move {
            println!("BASEURL {}", &baseurl);
            let _ = bun_log(&baseurl, &format!("Processing URL: {}", url));
            match services::run_lighthouse(&url, &baseurl, &output_path).await {
                Ok(_) => {
                    let _ = bun_log(
                        &baseurl,
                        &format!("Lighthouse ran successfully for {}", url),
                    );
                    let _ = bun_log(&baseurl, &format!("$SUCCESS_URL::{}", url));
                }
                Err(e) => {
                    let _ = bun_log(
                        &baseurl,
                        &format!("ERROR: Error running Lighthouse for {}: {:?}", url, e),
                    );
                    let _ = bun_log(&baseurl, &format!("$FAIL_URL::{}", url));
                }
            }
        }));
    }

    let _ = bun_log(
        &domain_tld,
        "✅ All Lighthouse tasks spawned, waiting for completion...",
    );

    join_all(handles).await; // Wait for all tasks to finish

    let _ = bun_log(
        &domain_tld,
        "✅ Lighthouse processing completed for all URLs",
    );
    Ok(())
}
