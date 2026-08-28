use std::{fs, io};
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use colour::{green, yellow};

/// Render a byte count as a short human-readable string with binary units.
///
/// Values below 1 KiB are rendered without decimals
/// (e.g. `"100 B"`); larger values use one fractional digit
/// (e.g. `"12.3 KB"`, `"2.9 MB"`).
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

/// Recursively remove `path` if it exists. No-op when the path is absent.
pub fn clear_dir(path: &Path) -> io::Result<()> {
    if path.exists() {
        fs::remove_dir_all(path)?;
    }
    Ok(())
}

/// Copy every file under `source_dir` into `target_dir`, preserving the
/// relative layout. Creates `target_dir` (and any missing subdirectories)
/// as needed. Files are visited in sorted order.
///
/// Returns the number of files copied and the total bytes transferred.
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

/// Walk `current_dir` recursively and append every regular file's path
/// (relative to `source_root`) to `sources`. Files with paths that cannot
/// be encoded as UTF-8 are silently skipped.
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

/// Reconcile `target_dir` against `sources_manifest` using `target_manifest`
/// as the previous state. When `target_manifest` is `Some`, only files whose
/// hash differs from the previous manifest (or that are absent from it) are
/// copied. When it is `None`, the target is wiped and re-populated from
/// scratch via [`move_files`].
///
/// Returns `(copied, skipped, total_bytes_copied)`.
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

/// Walk `target_dir` recursively and remove every file whose relative
/// path is not a key in `sources_manifest`. `manifest.json` itself is
/// always preserved. Returns the number of files removed.
pub fn prune_stale_files(target_dir: &Path, sources_manifest: &BTreeMap<String, String>) -> Result<usize, Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        return Ok(0);
    }

    let mut to_remove: Vec<PathBuf> = Vec::new();
    let mut stack: Vec<PathBuf> = vec![target_dir.to_path_buf()];

    while let Some(dir) = stack.pop() {
        for entry in fs::read_dir(&dir)? {
            let entry = entry?;
            let path = entry.path();
            let file_name = entry.file_name();

            if dir == target_dir && file_name == "manifest.json" {
                continue;
            }

            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                if let Ok(relative) = path.strip_prefix(target_dir) {
                    if let Some(rel_str) = relative.to_str() {
                        if !sources_manifest.contains_key(rel_str) {
                            to_remove.push(path);
                        }
                    }
                }
            }
        }
    }

    let count = to_remove.len();
    for path in &to_remove {
        let relative = path.strip_prefix(target_dir).unwrap_or(path);
        yellow!("[MANAGER] "); println!("Удалён устаревший: {}", relative.display());
        fs::remove_file(path)?;
    }
    Ok(count)
}