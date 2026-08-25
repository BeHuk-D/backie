use std::fs;
use std::io::Read;
use std::path::Path;
use sha2::{Digest, Sha256};
use sha2::digest::Output;

pub fn compute_file_hash(file_path: &Path) -> Result<Output<Sha256>, Box<dyn std::error::Error>> {
    let mut file = fs::File::open(file_path)
        .map_err(|e| format!("не удалось открыть {}: {}", file_path.display(), e))?;
    let mut hasher = Sha256::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("не удалось прочитать {}: {}", file_path.display(), e))?;
        if bytes_read == 0 {
            break;
        }
        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize())
}