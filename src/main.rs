use std::path::PathBuf;

use clap::{Parser, Subcommand};

pub mod duckdb_util;
pub mod export;

mod sparql_client;
mod sparql_query_metadata;
mod sparql_query_modifier;
mod sparql_result_to_duckdb;
mod used_queries;

#[cfg(feature = "frontend")]
mod frontend;
mod layer1;
mod layer2;
mod manifest;

/// Issue SPARQL queries
#[derive(Parser, Debug)]
#[clap(
    name = env!("CARGO_PKG_NAME"),
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    about = env!("CARGO_PKG_DESCRIPTION"),
    arg_required_else_help = true,
)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[clap(subcommand)]
    subcommand: SubCommand,
    /// Query directory
    #[arg(long, default_value = "queries")]
    queries_dir: PathBuf,
    /// Output directory
    #[arg(short, long, default_value = "dist")]
    dist_dir: PathBuf,
}

impl Args {
    pub fn layer1_queries_dir(&self) -> PathBuf {
        self.queries_dir.join("layer1")
    }

    pub fn layer2_queries_dir(&self) -> PathBuf {
        self.queries_dir.join("layer2")
    }
}

#[derive(Debug, Subcommand)]
enum SubCommand {
    /// Build tables
    Build {
        /// Don't skip even if the output is newer than the query file
        #[arg(short, long)]
        force: bool,
    },
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> anyhow::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();
    let args = Args::parse();

    std::fs::create_dir_all(&args.dist_dir)?;

    match args.subcommand {
        SubCommand::Build { force } => {
            layer1::layer1(&args, force).await?;
            layer2::layer2(&args)?;
            manifest::manifest(&args)?;

            #[cfg(feature = "frontend")]
            frontend::frontend(&args)?;
        }
    }

    Ok(())
}
