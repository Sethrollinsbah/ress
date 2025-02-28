use std::fs::OpenOptions;
use std::io::{self, Write};
use chrono::Utc;
use std;

pub fn bun_log(domain: &str, text: &str) -> io::Result<()> {
    // Obtain the current UTC timestamp
    let base_dir = std::env::current_dir()?.join("tmp/reports");
    let filename = base_dir.join(format!("{:?}.txt", base_dir));
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.fZ");

    // Format the log entry with the timestamp
    let log_entry = format!("{}::{}    \n-----", timestamp, text);

    // Open the file in append mode, creating it if it doesn't exist
    let mut file = OpenOptions::new()
        .append(true)
        .create(true)
        .open(filename)?;

    // Write the log entry to the file
    file.write_all(log_entry.as_bytes())?;

    Ok(())
}
