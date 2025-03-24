pub struct QueryMetadata {
    pub endpoint: String,
    pub paginate: Option<usize>,
}

pub fn extract_query_metadata(query: &str) -> anyhow::Result<QueryMetadata> {
    let re = regex::Regex::new(r"^#\s*([^:]+)\s*:\s*(.+)").unwrap();

    let mut endpoint = None;
    let mut paginate = None;
    for line in query.lines() {
        if let Some(caps) = re.captures(line) {
            let value = caps.get(2).unwrap().as_str().to_string();

            match caps[1].to_lowercase().as_str() {
                "endpoint" => {
                    endpoint = Some(value);
                }
                "paginate" => {
                    paginate = Some(value.parse()?);
                }
                _ => {}
            }
        }
    }

    let endpoint =
        endpoint.ok_or_else(|| anyhow::anyhow!("Failed to extract endpoint from query"))?;

    Ok(QueryMetadata { endpoint, paginate })
}
