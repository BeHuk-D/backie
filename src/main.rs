mod manifest;
mod file;
mod hash;

use std::collections::HashMap;
use std::path::Path;
use clap::Parser;
use colour::yellow;
use crate::file::{copy_files, go_to_dir};
use crate::hash::compute_file_hash;
use crate::manifest::{create_manifest, extract_manifest};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    source: String,

    #[arg(short, long)]
    target: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let source_dir = Path::new(&args.source);
    let target_dir = Path::new(&args.target);

    let mut files: Vec<String> = Vec::new();
    go_to_dir(source_dir.to_path_buf(), &mut files)?;

    let mut sources_manifest: HashMap<String, String> = HashMap::new();
    for file in &files {
        let hash = compute_file_hash(file)?;
        sources_manifest.insert(file.to_string(), hex::encode(hash));
    }

    let target_manifest = extract_manifest(target_dir)?;

    copy_files(target_manifest, &sources_manifest, source_dir, target_dir)?;

    create_manifest(&sources_manifest, target_dir)?;
    yellow!("[MANAGER] "); println!("Манифест сохранён в {}", target_dir.join("manifest.json").display());

    Ok(())
}

