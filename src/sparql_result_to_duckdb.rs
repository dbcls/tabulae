use std::{collections::HashMap, io::Write, path::Path};

use duckdb::{Connection, params};
use indicatif::{ProgressBar, ProgressStyle};

use crate::duckdb_util::{escape_sql_identifier, escape_sql_literal};

#[derive(serde::Deserialize, Debug)]
pub struct SparqlResults {
    #[allow(dead_code)]
    pub head: Head,
    pub results: Results,
}

#[derive(serde::Deserialize, Debug)]
pub struct Head {
    #[allow(dead_code)]
    pub vars: Vec<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Results {
    #[serde(default)]
    pub bindings: Vec<Binding>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Binding {
    #[serde(flatten)]
    pub values: std::collections::HashMap<String, Value>,
}

#[derive(serde::Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
#[allow(dead_code)]
pub struct Value {
    #[serde(rename = "type")]
    pub value_type: String,
    #[serde(rename = "xml:lang")]
    pub xml_lang: Option<String>,
    pub datatype: Option<String>,
    pub value: String,
}

#[derive(Debug)]
enum InferredResult {
    Inconsistent,
    Consistent { value_type: String },
}

pub fn find_consistent_types<S: AsRef<Path>>(
    src_paths: &[S],
) -> anyhow::Result<HashMap<String, String>> {
    let mut types: HashMap<String, InferredResult> = HashMap::new();

    for src_path in src_paths {
        let file = std::fs::File::open(src_path)?;
        let reader = std::io::BufReader::new(file);
        let sparql_results: SparqlResults = serde_json::from_reader(reader)?;

        for binding in &sparql_results.results.bindings {
            for (k, v) in binding.values.iter() {
                if let Some(t) = &v.datatype {
                    match types.get(k) {
                        None => {
                            types.insert(
                                k.clone(),
                                InferredResult::Consistent {
                                    value_type: t.clone(),
                                },
                            );
                        }
                        Some(InferredResult::Consistent { value_type }) => {
                            if value_type != t {
                                types.insert(k.clone(), InferredResult::Inconsistent);
                            }
                        }
                        Some(InferredResult::Inconsistent) => {}
                    };
                }
            }
        }
    }

    let mut consistent_types: HashMap<String, String> = HashMap::new();
    for (k, v) in types {
        if let InferredResult::Consistent { value_type } = v {
            consistent_types.insert(k, value_type);
        }
    }

    Ok(consistent_types)
}

fn xsd_type_to_duckdb_type(xml_type: &str) -> &str {
    match xml_type {
        "http://www.w3.org/2001/XMLSchema#integer" => "INT64",
        // NOTE xsd:decimal is difficult to convert, because the precision is not known
        "http://www.w3.org/2001/XMLSchema#double" => "DOUBLE",
        "http://www.w3.org/2001/XMLSchema#boolean" => "BOOLEAN",
        _ => "VARCHAR",
    }
}

fn read_json_columns_struct(vars: &[String], consistent_types: &HashMap<String, String>) -> String {
    let columns = vars
        .iter()
        .map(|var| {
            let t = consistent_types
                .get(var)
                .map(|t| xsd_type_to_duckdb_type(t))
                .unwrap_or("VARCHAR");

            format!("{}: {}", escape_sql_literal(var), escape_sql_literal(t))
        })
        .collect::<Vec<String>>()
        .join(", ");
    format!("{{{}}}", columns)
}

pub fn sparql_results_to_duckdb<S: AsRef<Path>>(
    conn: &Connection,
    base_name: &str,
    src_paths: &[S],
) -> anyhow::Result<()> {
    let mut temp_jsons = tempfile::NamedTempFile::new()?;

    let pb = ProgressBar::new(src_paths.len() as u64);
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {bar:20} {pos:>3}/{len:3} ({eta}) {msg}",
    )?);

    let consistent_types = find_consistent_types(src_paths)?;
    log::debug!("Consistent types: {:?}", consistent_types);

    let mut vars = vec![];
    for src_path in src_paths {
        pb.set_message("Parsing SPARQL Result JSON");
        let file = std::fs::File::open(src_path)?;
        let reader = std::io::BufReader::new(file);
        let sparql_results: SparqlResults = serde_json::from_reader(reader)?;
        vars = sparql_results.head.vars.clone();

        pb.set_message("Writing JSONs");
        for binding in &sparql_results.results.bindings {
            let row: serde_json::Map<String, serde_json::Value> = binding
                .values
                .iter()
                .map(|(k, v)| (k.clone(), serde_json::Value::String(v.value.clone())))
                .collect();
            let row = serde_json::Value::Object(row);
            let row = serde_json::to_string(&row)?;
            temp_jsons.write_all(row.as_bytes())?;
        }
        pb.inc(1);
    }
    temp_jsons.flush()?;

    pb.finish_with_message("Written intermediate JSONs");
    pb.set_style(ProgressStyle::with_template(
        "{spinner:.green} [{elapsed_precise}] {msg}",
    )?);

    let columns = read_json_columns_struct(&vars, &consistent_types);

    let jsons_path = temp_jsons.path().to_str().unwrap_or_default();
    pb.set_message("Loading JSONs to DuckDB");
    let query = format!(
        "CREATE OR REPLACE TABLE {} AS FROM read_json({}, columns = {})",
        escape_sql_identifier(base_name),
        escape_sql_literal(jsons_path),
        columns
    );
    log::debug!("Executing query:\n{}", query);
    conn.execute(&query, params![])?;
    pb.finish_with_message("Done");

    Ok(())
}
