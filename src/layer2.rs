use std::path::Path;

use glob::glob;

use crate::{
    Args,
    duckdb_util::{escape_sql_identifier, escape_sql_literal},
    export,
};

pub fn layer2(args: &Args) -> anyhow::Result<()> {
    let dest_dir = Path::new(&args.dist_dir);

    let layer1_db_path = dest_dir.join("layer1.duckdb");
    let layer2_db_path = dest_dir.join("layer2.duckdb");

    let layer2_dist_dir = Path::new(&args.dist_dir).join("layer2");
    if layer2_dist_dir.exists() {
        std::fs::remove_dir_all(&layer2_dist_dir)?;
    }
    std::fs::create_dir_all(&layer2_dist_dir)?;

    {
        if layer2_db_path.exists() {
            std::fs::remove_file(&layer2_db_path)?;
        }
        let conn = duckdb::Connection::open(&layer2_db_path)?;
        conn.prepare(&format!(
            "ATTACH {} (READ_ONLY)",
            escape_sql_literal(&layer1_db_path.to_string_lossy())
        ))?
        .execute([])?;
        conn.prepare("USE layer1")?.execute([])?;

        let src_pattern = args.layer2_queries_dir().join("*.sql");
        let files = glob(&src_pattern.to_string_lossy())?;
        let names = files
            .filter_map(|x| {
                x.ok()
                    .and_then(|x| x.file_stem()?.to_str().map(|s| s.to_string()))
            })
            .collect::<Vec<String>>();

        for name in names {
            let file = args.layer2_queries_dir().join(format!("{}.sql", name));

            let base_query = std::fs::read_to_string(&file)?;
            let log_target = format!("layer2/{}", name);
            log::info!(target: &log_target, "Executing query from {}", file.display());

            let query = format!(
                "CREATE TABLE layer2.{} AS {}",
                escape_sql_identifier(&name),
                base_query
            );
            conn.execute(&query, [])?;

            let add_comment_stmt = format!(
                "COMMENT ON TABLE layer2.{} IS {}",
                escape_sql_identifier(&name),
                escape_sql_literal(&base_query)
            );
            conn.execute(&add_comment_stmt, [])?;

            export::export_relation_to_files(&conn, "layer2", &name, &layer2_dist_dir)?;
        }
        log::info!(target: "layer2", "Wrote {}", layer2_db_path.display());
    }

    Ok(())
}
