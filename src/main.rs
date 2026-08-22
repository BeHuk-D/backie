use std::{fs, io};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::io::Read;
use std::path::{Path, PathBuf};
use clap::Parser;
use sha2::{Digest, Sha256};
use sha2::digest::Output;
use serde::{Deserialize, Serialize};

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

    match target_manifest {
        Some(old_manifest) => {
            for (file_path, file_hash) in &sources_manifest {
                let need_copy = match old_manifest.get(file_path) {
                    Some(old_hash) => old_hash != file_hash,
                    None => true,
                };

                if need_copy {
                    let relative_path = Path::new(file_path).strip_prefix(source_dir)?;
                    let target_path = target_dir.join(relative_path);

                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    fs::copy(file_path, &target_path)?;
                    println!("[FILES] Скопирован: {}", relative_path.display());
                }
            }
        }
        None => {
            println!("[MANAGER] Копируем все файлы...");
            clear_dir(target_dir)?;
            move_files(source_dir, target_dir)?;
        }
    }

    create_manifest(&sources_manifest, target_dir)?;
    println!("[MANAGER] Манифест сохранён в {}", target_dir.join("manifest.json").display());

    Ok(())
}

fn create_manifest(sources_manifest: &HashMap<String, String>, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = target_dir.join("manifest.json");
    save_hashmap_json(sources_manifest, &manifest_path)?;
    Ok(())
}

fn clear_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

fn move_files(source_dir: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let mut files = Vec::new();
    go_to_dir(source_dir.to_path_buf(), &mut files)?;

    for file_path in &files {
        let relative_path = Path::new(file_path).strip_prefix(source_dir)?;
        let target_path = target_dir.join(relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        fs::copy(file_path, &target_path)?;
        println!("[FILES] Копирование: {}", relative_path.display());
    }

    Ok(())
}

fn extract_manifest(target_dir: &Path) -> Result<Option<HashMap<String, String>>, Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        return Ok(None);
    }

    let manifest_path = target_dir.join("manifest.json");
    if manifest_path.exists() {
        return Ok(Some(load_hashmap_json(&manifest_path)?));
    }

    Ok(None)
}

fn go_to_dir(current_dir: PathBuf, sources: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            go_to_dir(path, sources)?;
        } else {
            if let Some(path_str) = path.to_str() {
                sources.push(path_str.to_string());
            }
        }
    }

    Ok(())
}

fn compute_file_hash(file_path: &str) -> Result<Output<Sha256>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(file_path)?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize())
}

fn save_hashmap_json(map: &HashMap<String, String>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(map)?;
    fs::write(path, json)?;
    Ok(())
}

fn load_hashmap_json(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let map: HashMap<String, String> = serde_json::from_str(&json)?;
    Ok(map)
}