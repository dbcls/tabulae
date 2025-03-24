use std::{collections::HashMap, path::Path};

use duckdb::{AccessMode, Config, Connection, params};

use crate::{Args, duckdb_util::escape_sql_literal};

#[derive(serde::Serialize, Debug)]
struct Column {
    column_name: String,
    column_type: String,
    min: Option<String>,
    max: Option<String>,
    approx_unique: i64,
    avg: Option<String>,
    std: Option<String>,
    q25: Option<String>,
    q50: Option<String>,
    q75: Option<String>,
    count: i64,
    null_percentage: f64,
    exact_unique: i64,
}

#[derive(serde::Serialize, Debug)]
struct Table {
    name: String,
    columns: Vec<Column>,
    num_rows: i64,
    query: Option<String>,
    collection: String,
    sizes: HashMap<String, usize>,
}

#[derive(serde::Serialize, Debug)]
struct Manifest {
    tables: Vec<Table>,
}

fn get_filesizes<P: AsRef<Path>>(
    dist_dir: &P,
    table_name: &str,
    collection: &str,
) -> HashMap<String, usize> {
    let mut sizes = HashMap::new();
    for ext in ["csv", "tsv", "parquet"] {
        let exported_path = dist_dir
            .as_ref()
            .join(collection)
            .join(table_name)
            .with_extension(ext);

        if exported_path.exists() {
            let Ok(metadata) = exported_path.metadata() else {
                continue;
            };
            sizes.insert(ext.to_string(), metadata.len() as usize);
        }
    }
    sizes
}

fn summarize_table<P: AsRef<Path>>(
    conn: &Connection,
    table_name: &str,
    collection: &str,
    dist_dir: &P,
) -> anyhow::Result<Table> {
    let mut stmt = conn.prepare(&format!(
        r#"
        SELECT
            sm.*, counts.exact_unique
        FROM
            (SUMMARIZE FROM {0}) sm
        JOIN
            (UNPIVOT (
                SELECT
                    COUNT(DISTINCT COLUMNS(*))
                FROM
                    {0}
                )
            ON *
            INTO
                NAME column_name
                VALUES exact_unique
            ) counts
        ON
            sm.column_name = counts.column_name;
        "#,
        escape_sql_literal(table_name),
    ))?;

    let columns = stmt.query_map([], |row| {
        let column: Column = Column {
            column_name: row.get(0)?,
            column_type: row.get(1)?,
            min: row.get(2)?,
            max: row.get(3)?,
            approx_unique: row.get(4)?,
            avg: row.get(5)?,
            std: row.get(6)?,
            q25: row.get(7)?,
            q50: row.get(8)?,
            q75: row.get(9)?,
            count: row.get(10)?,
            null_percentage: row.get(11)?,
            exact_unique: row.get(12)?,
        };
        Ok(column)
    })?;
    let num_rows = conn.query_row(
        &format!("SELECT COUNT(*) FROM {}", escape_sql_literal(table_name)),
        [],
        |row| row.get(0),
    )?;

    let comment = conn.query_row(
        "SELECT comment FROM duckdb_tables() WHERE table_name = ?",
        params![table_name],
        |row| row.get::<_, Option<String>>(0),
    )?;

    let sizes = get_filesizes(dist_dir, table_name, collection);

    Ok(Table {
        name: table_name.to_string(),
        columns: columns.collect::<Result<Vec<Column>, _>>()?,
        num_rows,
        query: comment,
        collection: collection.to_string(),
        sizes,
    })
}

fn summarize_tables(args: &Args, collection: &str) -> anyhow::Result<Vec<Table>> {
    let config = Config::default().access_mode(AccessMode::ReadOnly)?;
    let db_path = args.dist_dir.join(format!("{}.duckdb", collection));
    let conn = Connection::open_with_flags(db_path, config)?;

    let mut stmt = conn.prepare("SHOW TABLES")?;
    let rows = stmt.query_map([], |row| {
        let name: String = row.get(0)?;
        Ok(name)
    })?;
    let table_names = rows.collect::<Result<Vec<String>, _>>()?;
    let progress_bar = indicatif::ProgressBar::new(table_names.len() as u64);
    progress_bar.set_style(
        indicatif::ProgressStyle::default_bar()
            .template("{spinner:.green} [{elapsed_precise}] [{bar:20}] {pos}/{len} {msg}")?,
    );
    progress_bar.set_message(format!("Summarizing {} tables", collection));

    let tables = table_names
        .iter()
        .map(|name| {
            let result = summarize_table(&conn, name, collection, &args.dist_dir);
            progress_bar.inc(1);
            result
        })
        .collect::<Result<Vec<Table>, _>>()?;

    progress_bar.finish_with_message(format!("Summarized {} tables", collection));

    Ok(tables)
}

pub fn manifest(args: &Args) -> anyhow::Result<()> {
    let dest_dir = Path::new(&args.dist_dir);

    let layer1_tables = summarize_tables(args, "layer1")?;
    let layer2_tables = summarize_tables(args, "layer2")?;

    let tables = layer2_tables
        .into_iter()
        .chain(layer1_tables)
        .collect::<Vec<Table>>();

    let manifest = Manifest { tables };

    let manifest_path = dest_dir.join("manifest.json");
    let manifest_file = std::fs::File::create(&manifest_path)?;
    serde_json::to_writer_pretty(manifest_file, &manifest)?;
    log::info!(target: "manifest", "Wrote manifest to {}", manifest_path.display());

    Ok(())
}
