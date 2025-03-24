use std::path::Path;

use glob::glob;
use tempfile::NamedTempFile;

use crate::{
    Args,
    duckdb_util::{escape_sql_identifier, escape_sql_literal},
    export, sparql_client, sparql_query_metadata, sparql_query_modifier, sparql_result_to_duckdb,
    used_queries::{self, ensure_metadata_schema},
};

async fn request_single(
    log_target: &str,
    query: &str,
    endpoint: &str,
) -> anyhow::Result<Vec<NamedTempFile>> {
    let temp = tempfile::NamedTempFile::new()?;
    let num_bindings = sparql_client::save_sparql_result_to_file(query, endpoint, &temp).await?;
    log::info!(target: log_target, "{} binding(s) received", num_bindings);

    Ok(vec![temp])
}

async fn request_with_pagination(
    log_target: &str,
    query: &str,
    endpoint: &str,
    paginate: usize,
) -> anyhow::Result<Vec<NamedTempFile>> {
    log::info!(target: log_target, "Pagination enabled with limit {}", paginate);

    let mut paths = vec![];
    let mut offset = 0;
    loop {
        let query = sparql_query_modifier::rewrite_query_limit_offset(query, paginate, offset)?;
        let query = query.to_string();

        let temp_file: NamedTempFile = NamedTempFile::new()?;
        let num_bindings =
            sparql_client::save_sparql_result_to_file(&query, endpoint, &temp_file).await?;
        paths.push(temp_file);
        if num_bindings < paginate {
            log::info!(target: log_target, "{} binding(s) received in total", offset + num_bindings);
            break;
        }
        offset += num_bindings;
    }

    Ok(paths)
}
async fn process_query<P: AsRef<Path>>(
    conn: &duckdb::Connection,
    query_path: P,
    args: &Args,
    force: bool,
) -> anyhow::Result<()> {
    let name = query_path
        .as_ref()
        .file_stem()
        .ok_or_else(|| anyhow::anyhow!("Failed to get file stem"))?
        .to_str()
        .ok_or_else(|| anyhow::anyhow!("Failed to convert to string"))?;
    let log_target = format!("layer1/{}", name);

    log::info!(target: &log_target, "Start processing");

    // Check if the query needs to be updated
    let query_mtime_us = std::fs::metadata(&query_path)?
        .modified()?
        .duration_since(std::time::SystemTime::UNIX_EPOCH)?
        .as_micros() as i64;
    if !force {
        let stored_mtime_us = used_queries::get_mtime(conn, name)?;

        if let Some(stored_mtime) = stored_mtime_us {
            if stored_mtime >= query_mtime_us {
                log::info!(target: &log_target, "Skipping as the query is up-to-date");
                return Ok(());
            }
        }
    }

    let query = std::fs::read_to_string(&query_path)?;
    let qm = sparql_query_metadata::extract_query_metadata(&query)?;
    log::info!(target: &log_target, "Using endpoint {}", qm.endpoint);

    let paths = if let Some(paginate) = qm.paginate {
        request_with_pagination(&log_target, &query, &qm.endpoint, paginate).await?
    } else {
        request_single(&log_target, &query, &qm.endpoint).await?
    };

    sparql_result_to_duckdb::sparql_results_to_duckdb(conn, name, &paths)?;
    let layer1_dist_dir = Path::new(&args.dist_dir).join("layer1");
    std::fs::create_dir_all(&layer1_dist_dir)?;
    export::export_relation_to_files(conn, "layer1", name, &layer1_dist_dir)?;

    let add_comment_stmt = format!(
        "COMMENT ON TABLE layer1.{} IS {}",
        escape_sql_identifier(name),
        escape_sql_literal(&query)
    );
    conn.execute(&add_comment_stmt, [])?;

    used_queries::touch_mtime(conn, name, &query, query_mtime_us)?;

    Ok(())
}

fn drop_tables_for_not_existing_queries(
    conn: &duckdb::Connection,
    names: &[String],
) -> anyhow::Result<()> {
    let names_in_db = used_queries::all_query_names(conn)?;
    let layer1_dist_dir = Path::new("dist").join("layer1");

    for name in names_in_db {
        let log_target = format!("layer1/{}", name);
        if !names.contains(&name) {
            log::info!(target: &log_target, "Dropping table");
            used_queries::delete(conn, &name)?;

            conn.execute(
                &format!("DROP TABLE IF EXISTS {}", escape_sql_identifier(&name)),
                [],
            )?;
            for ext in &["csv", "tsv", "parquet"] {
                let path = layer1_dist_dir.join(format!("{}.{}", name, ext));
                if path.exists() {
                    std::fs::remove_file(&path)?;
                    log::info!(target: &log_target, "Removed {}", path.display());
                }
            }
        }
    }

    Ok(())
}

pub async fn layer1(args: &Args, force: bool) -> anyhow::Result<()> {
    let dest_dir = Path::new(&args.dist_dir);
    let dest_db_path = dest_dir.join("layer1.duckdb");

    log::info!(target: "layer1", "Output DB path: {}", dest_db_path.display());

    let conn = duckdb::Connection::open(dest_db_path)?;
    ensure_metadata_schema(&conn)?;

    let src_dir = args.layer1_queries_dir();
    let src_pattern = src_dir.join("*.rq");
    let files = glob(&src_pattern.to_string_lossy())?;
    let names = files
        .filter_map(|x| {
            x.ok()
                .and_then(|x| x.file_stem()?.to_str().map(|s| s.to_string()))
        })
        .collect::<Vec<String>>();
    if names.is_empty() {
        return Err(anyhow::anyhow!("No queries found in {}", src_dir.display()));
    }
    drop_tables_for_not_existing_queries(&conn, &names)?;

    for name in names {
        let query_path = src_dir.join(name).with_extension("rq");
        process_query(&conn, query_path, args, force).await?;
    }

    Ok(())
}
