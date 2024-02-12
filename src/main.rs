use std::collections::HashMap;
use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Seek, SeekFrom, Write};
use reqwest::blocking::Client;
use scraper::{Html, Selector};
use std::time::Duration;
use std::io::BufWriter;

// Modified function with caching support and request timeout
fn fetch_title_from_url(url: &str, cache: &mut HashMap<String, String>) -> io::Result<String> {
    if let Some(cached_title) = cache.get(url) {
        println!("Cached ; Title = {} ; URL = {}", cached_title, url);
        return Ok(cached_title.clone());
    }

    // Client configuration with timeout
    let client = Client::builder()
        .timeout(Duration::from_secs(1)) // Set the timeout to 1 seconds
        .build()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    // Making the request with the set timeout
    let html = client.get(url)
        .send()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
        .text()
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

    let document = Html::parse_document(&html);
    let title_selector = Selector::parse("title").unwrap();
    let title = document.select(&title_selector)
        .next()
        .and_then(|n| n.text().next())
        .unwrap_or_default()
        .to_string();

    cache.insert(url.to_string(), title.clone());

    println!("Fetched ; Title = {} ; URL = {}", title, url);
    Ok(title)
}

fn main() -> io::Result<()> {
    let input_file = File::open("input.log")?;
    let mut reader = BufReader::new(input_file);
    let mut cache = HashMap::new(); // Cache for titles

    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    let mut headers: Vec<String> = first_line.split(',')
        .map(|entry| entry.split('=').next().unwrap_or_default().trim_matches('"').to_string())
        .collect();

    headers.push("title".to_string()); // Adding a new column

    let mut empty_cols: Vec<usize> = headers.iter().enumerate()
        .filter_map(|(i, header)| if header.is_empty() { Some(i) } else { None })
        .collect();

    for line in reader.by_ref().lines().skip(1) {
        let line = line?;
        let values: Vec<&str> = line.split(',')
            .map(|entry| entry.split('=').next().unwrap_or("").trim_matches('"'))
            .collect();

        let mut updated = false;
        for &col in &empty_cols {
            if let Some(value) = values.get(col) {
                if !value.is_empty() {
                    headers[col] = value.to_string();
                    updated = true;
                }
            }
        }

        if updated {
            empty_cols.retain(|&i| headers[i].is_empty());
        }

        if empty_cols.is_empty() {
            break;
        }
    }

    let referralurl_index = headers.iter().position(|header| header == "referralurl");

    // Creating an output_file object using BufWriter for buffered writing
    let output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output.log")?;
    let mut writer = BufWriter::new(output_file);

// Writing headers in one line
    writeln!(writer, "{}", headers.join(","))?;

// Moving the read pointer back to the start of the file, skipping headers
    reader.seek(SeekFrom::Start(0))?;
    reader.read_line(&mut first_line)?; // Skipping the first line with headers

// Checking for index presence outside the loop
    if let Some(index) = referralurl_index {
        for line in reader.lines() {
            let mut line = line?;
            let values: Vec<&str> = line.split(',').collect();
            if let Some(referral) = values.get(index) {
                let parts: Vec<&str> = referral.splitn(2, '=').collect();
                if parts.len() == 2 {
                    let url = parts[1].trim_matches('"');
                    if let Ok(title) = fetch_title_from_url(url, &mut cache) {
                        line.push_str(&format!(",\"{}\"", title));
                    }
                }
            }

            // Writing the updated line to the file
            writeln!(writer, "{}", line)?;
        }
    }

// Mandatory buffer flush after writing is finished
    writer.flush()?;

    Ok(())
}