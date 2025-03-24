use duckdb::params;

pub fn ensure_metadata_schema(conn: &duckdb::Connection) -> anyhow::Result<()> {
    conn.prepare("CREATE SCHEMA IF NOT EXISTS tabulae")?
        .execute([])?;
    conn.prepare(
        "CREATE TABLE IF NOT EXISTS tabulae.sparql_queries (name STRING PRIMARY KEY, query STRING NOT NULL, mtime TIMESTAMP NOT NULL)",
    )?
    .execute([])?;

    Ok(())
}

pub fn get_mtime(conn: &duckdb::Connection, name: &str) -> anyhow::Result<Option<i64>> {
    let mut stmt = conn.prepare("SELECT mtime FROM tabulae.sparql_queries WHERE name = ?")?;
    let mut rows = stmt.query(params![name])?;
    let t = if let Some(row) = rows.next()? {
        Some(row.get(0)?)
    } else {
        None
    };

    Ok(t)
}

pub fn touch_mtime(
    conn: &duckdb::Connection,
    name: &str,
    query: &str,
    mtime: i64,
) -> anyhow::Result<()> {
    conn.execute(
        "INSERT OR REPLACE INTO tabulae.sparql_queries (name, query, mtime) VALUES (?, ?, make_timestamp(?))",
        params![name, query, mtime],
    )?;

    Ok(())
}

pub fn all_query_names(conn: &duckdb::Connection) -> anyhow::Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT name FROM tabulae.sparql_queries")?;
    let rows = stmt.query_map([], |row| row.get(0))?;
    let names = rows.collect::<Result<Vec<String>, _>>()?;

    Ok(names)
}

pub fn delete(conn: &duckdb::Connection, name: &str) -> anyhow::Result<()> {
    conn.execute(
        "DELETE FROM tabulae.sparql_queries WHERE name = ?",
        params![name],
    )?;
    Ok(())
}
