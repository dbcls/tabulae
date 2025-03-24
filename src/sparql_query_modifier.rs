use spargebra::{Query, algebra::GraphPattern};

pub fn rewrite_query_limit_offset(
    query_str: &str,
    new_limit: usize,
    new_offeset: usize,
) -> anyhow::Result<Query> {
    let mut query = Query::parse(query_str, None)?;

    match &mut query {
        Query::Select { pattern, .. } => match pattern {
            GraphPattern::Slice { length, start, .. } => {
                *length = Some(new_limit);
                *start = new_offeset;
            }
            GraphPattern::Project { .. } | GraphPattern::Distinct { .. } => {
                *pattern = GraphPattern::Slice {
                    length: Some(new_limit),
                    start: new_offeset,
                    inner: Box::new(pattern.clone()),
                };
            }
            _ => {
                return Err(anyhow::anyhow!("Unsupported graph pattern type"));
            }
        },
        _ => {
            return Err(anyhow::anyhow!("Unsupported query type"));
        }
    }

    Ok(query)
}
