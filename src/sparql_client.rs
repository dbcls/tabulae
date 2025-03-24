use futures_util::StreamExt;
use indicatif::{ProgressBar, ProgressStyle};
use std::{io::Write, path::Path};

fn num_bindings(result: &serde_json::Value) -> usize {
    result
        .get("results")
        .and_then(|r| r.get("bindings"))
        .and_then(|b| b.as_array())
        .map_or(0, |bindings| bindings.len())
}

pub async fn save_sparql_result_to_file<P: AsRef<Path>>(
    query: &str,
    endpoint: &str,
    file: P,
) -> anyhow::Result<usize> {
    let pb: ProgressBar = ProgressBar::new(0);
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {msg}",
    )?);
    pb.enable_steady_tick(std::time::Duration::from_millis(100));

    let client = reqwest::Client::new();
    pb.set_message(format!("Sending query to endpoint {}", endpoint));
    let response = client
        .post(endpoint)
        .header("Content-Type", "application/sparql-query")
        .header("Accept", "application/sparql-results+json")
        .body(query.to_owned())
        .send();
    pb.set_message("Waiting for response");
    let response = response.await?;
    pb.set_message(format!("Response received; {}", response.status()));

    if !response.status().is_success() {
        let error = response.text().await?;
        return Err(anyhow::anyhow!(error));
    }

    let total_size = response.content_length().unwrap_or_default();
    pb.set_length(total_size);
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {bar:20} {bytes:>12}/{total_bytes:12} ({eta}) {msg}",
    )?);

    {
        let destination = std::fs::File::create(&file)?;
        let mut writer = std::io::BufWriter::new(destination);
        let mut stream = response.bytes_stream();

        let mut first_time = true;
        while let Some(chunk) = stream.next().await {
            if first_time {
                first_time = false;
                // pb.reset_elapsed();
                pb.set_message("Receiving data");
            }
            let chunk = chunk?;
            pb.inc(chunk.len() as u64);
            writer.write_all(&chunk)?;
        }
        pb.finish_with_message(format!(
            "Done; saved to {}",
            file.as_ref().to_str().unwrap_or_default()
        ));
        pb.set_style(ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] {msg}",
        )?);
        // ensure drop of writer to flush and close file
    }

    let data = std::fs::read_to_string(&file)?;
    let result: serde_json::Value = serde_json::from_str(&data)?;
    let num_bindings = num_bindings(&result);

    pb.finish_with_message(format!(
        "Done; {} bindings saved to {}",
        num_bindings,
        file.as_ref().to_str().unwrap_or_default()
    ));

    Ok(num_bindings)
}
