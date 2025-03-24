pub fn escape_sql_identifier(table_name: &str) -> String {
    format!("\"{}\"", table_name.replace("\"", "\"\""))
}

pub fn escape_sql_literal(value: &str) -> String {
    format!("'{}'", value.replace("'", "''"))
}
