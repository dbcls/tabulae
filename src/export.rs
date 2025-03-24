use std::path::Path;

use crate::duckdb_util::escape_sql_identifier;

pub fn export_relation_to_files<P: AsRef<Path>>(
    conn: &duckdb::Connection,
    database_name: &str,
    relation: &str,
    dest_dir: P,
) -> anyhow::Result<()> {
    let export_types = &[
        ("csv", "FORMAT CSV"),
        ("tsv", "FORMAT CSV, DELIMITER '\t'"),
        ("parquet", "FORMAT parquet"),
    ];
    for &(ext, format) in export_types {
        let dest_path = dest_dir.as_ref().join(format!("{}.{}", relation, ext));

        if std::path::Path::new(&dest_path).exists() {
            std::fs::remove_file(&dest_path)?;
        }

        let sql = format!(
            "COPY (FROM {}.{}) TO {} ({})",
            escape_sql_identifier(database_name),
            escape_sql_identifier(relation),
            escape_sql_identifier(&dest_path.to_string_lossy()),
            format
        );
        conn.execute_batch(&sql)?;
    }

    Ok(())
}
