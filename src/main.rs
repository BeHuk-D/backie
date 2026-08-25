mod manifest;
mod file;
mod hash;

use std::collections::BTreeMap;
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

    if Path::new(&args.source) == Path::new(&args.target) {
        return Err(format!(
            "source и target указывают на один и тот же путь ({}) — операция отменёна, чтобы не уничтожить исходные данные.",
            args.source
        ).into());
    }

    let source_dir = Path::new(&args.source).canonicalize()?;
    let target_dir = Path::new(&args.target);

    if let Ok(canonical_target) = target_dir.canonicalize() {
        if source_dir == canonical_target {
            return Err(format!(
                "после канонизации source и target совпадают ({}) — операция отменёна, чтобы не уничтожить исходные данные.",
                source_dir.display()
            ).into());
        }
    }

    let mut files: Vec<String> = Vec::new();
    go_to_dir(source_dir.clone(), &source_dir, &mut files)?;

    let mut sources_manifest: BTreeMap<String, String> = BTreeMap::new();
    for file in &files {
        let source_file_path = source_dir.join(file);
        let hash = compute_file_hash(&source_file_path)?;
        sources_manifest.insert(file.to_string(), hex::encode(hash));
    }

    let target_manifest = extract_manifest(target_dir)?;

    copy_files(target_manifest, &sources_manifest, &source_dir, target_dir)?;

    create_manifest(&sources_manifest, target_dir)?;
    yellow!("[MANAGER] "); println!("Манифест сохранён в {}", target_dir.join("manifest.json").display());

    Ok(())
}

