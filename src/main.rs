use std::fs::{File, OpenOptions};
use std::io::{Read, self, BufRead, Write, BufReader, Seek, SeekFrom};

fn main() -> io::Result<()> {
    let input_file = File::open("input.log")?;
    let mut reader = BufReader::new(input_file);

    let mut first_line = String::new();
    reader.read_line(&mut first_line)?;

    // Process headers from the first line
    let mut headers: Vec<String> = first_line.split(',')
        .map(|entry| entry.split('=').next().unwrap_or("").trim_matches('"').to_string())
        .collect();

    let mut empty_cols: Vec<usize> = headers.iter().enumerate()
        .filter_map(|(i, header)| if header.is_empty() { Some(i) } else { None })
        .collect();

    // Do not move the pointer to the start to avoid skipping the first line when writing
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

    let mut output_file = OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open("output.log")?;

    // Write the updated headers
    writeln!(output_file, "{}", headers.join(","))?;

    // Move the pointer to the start to copy the entire file, including the first line
    reader.seek(SeekFrom::Start(0))?;
    for line in reader.lines() {
        writeln!(output_file, "{}", line?)?;
    }

    Ok(())
}
