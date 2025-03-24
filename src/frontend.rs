use crate::Args;
use std::fs;
use std::io::Cursor;
use tar::Archive;

const FRONTEND_TAR: &'static [u8] = include_bytes!("../frontend.tar");

pub fn frontend(args: &Args) -> anyhow::Result<()> {
    log::info!(target: "frontend", "Populating frontend");

    let index_path = &args.dist_dir.join("index.html");
    if index_path.exists() {
        fs::remove_file(index_path)?;
    }

    let assets_path = &args.dist_dir.join("assets");
    if assets_path.exists() {
        fs::remove_dir_all(assets_path)?;
    }

    let tar = Cursor::new(FRONTEND_TAR);
    let mut archive = Archive::new(tar);

    archive.unpack(&args.dist_dir)?;

    log::info!(target: "frontend", "Done");

    Ok(())
}
