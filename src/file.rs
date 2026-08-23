use std::{fs, io};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

pub fn clear_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn move_files(source_dir: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn go_to_dir(current_dir: PathBuf, sources: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
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

pub fn copy_files(target_manifest:  Option<HashMap<String, String>>, sources_manifest: &HashMap<String, String>, source_dir: &Path, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    match target_manifest {
        Some(old_manifest) => {
            for (file_path, file_hash) in sources_manifest {
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
                else {
                    println!("[MANAGER] Копирование не требуется");
                }
            }
        }
        None => {
            println!("[MANAGER] Копируем все файлы...");
            clear_dir(target_dir)?;
            move_files(source_dir, target_dir)?;
        }
    }
    Ok(())
}