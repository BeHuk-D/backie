use std::{fs, io};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use colour::{green, yellow};

pub fn format_bytes(n: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    let mut size = n as f64;
    let mut idx = 0;
    while size >= 1024.0 && idx < UNITS.len() - 1 {
        size /= 1024.0;
        idx += 1;
    }
    if idx == 0 {
        format!("{} {}", n, UNITS[0])
    } else {
        format!("{:.1} {}", size, UNITS[idx])
    }
}

pub fn clear_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

pub fn move_files(source_dir: &Path, target_dir: &Path) -> Result<(usize, u64), Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        fs::create_dir_all(target_dir)?;
    }

    let mut files = Vec::new();
    go_to_dir(source_dir.to_path_buf(), source_dir, &mut files)?;
    files.sort();

    let mut copied = 0;
    let mut total_bytes: u64 = 0;
    for relative_path in &files {
        let target_path = target_dir.join(relative_path);

        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent)?;
        }

        let source_path = source_dir.join(relative_path);
        let size = fs::metadata(&source_path)
            .map_err(|e| format!("не удалось прочитать размер {}: {}", source_path.display(), e))?
            .len();
        fs::copy(&source_path, &target_path)?;
        copied += 1;
        total_bytes += size;
        green!("[FILES] "); println!("Копирование: {} ({})", relative_path, format_bytes(size));
    }

    Ok((copied, total_bytes))
}

pub fn go_to_dir(current_dir: PathBuf, source_root: &Path, sources: &mut Vec<String>) -> Result<(), Box<dyn std::error::Error>> {
    for entry in fs::read_dir(current_dir)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            go_to_dir(path, source_root, sources)?;
        } else {
            if let Ok(relative) = path.strip_prefix(source_root) {
                if let Some(relative_str) = relative.to_str() {
                    sources.push(relative_str.to_string());
                }
            }
        }
    }

    Ok(())
}

pub fn copy_files(target_manifest:  Option<BTreeMap<String, String>>, sources_manifest: &BTreeMap<String, String>, source_dir: &Path, target_dir: &Path) -> Result<(usize, usize, u64), Box<dyn std::error::Error>> {
    match target_manifest {
        Some(old_manifest) => {
            let mut copied = 0;
            let mut skipped = 0;
            let mut total_bytes: u64 = 0;
            for (relative_path, file_hash) in sources_manifest {
                let need_copy = match old_manifest.get(relative_path) {
                    Some(old_hash) => old_hash != file_hash,
                    None => true,
                };

                if need_copy {
                    let target_path = target_dir.join(relative_path);

                    if let Some(parent) = target_path.parent() {
                        fs::create_dir_all(parent)?;
                    }

                    let source_path = source_dir.join(relative_path);
                    let size = fs::metadata(&source_path)
                        .map_err(|e| format!("не удалось прочитать размер {}: {}", source_path.display(), e))?
                        .len();
                    fs::copy(&source_path, &target_path)?;
                    copied += 1;
                    total_bytes += size;
                    green!("[FILES] "); println!("Скопирован: {} ({})", relative_path, format_bytes(size));
                }
                else {
                    skipped += 1;
                    yellow!("[MANAGER] "); println!("Копирование не требуется");
                }
            }
            Ok((copied, skipped, total_bytes))
        }
        None => {
            yellow!("[MANAGER] "); println!("Копируем все файлы...");
            clear_dir(target_dir)?;
            let (copied, total_bytes) = move_files(source_dir, target_dir)?;
            Ok((copied, 0, total_bytes))
        }
    }
}