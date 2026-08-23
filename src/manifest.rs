use std::collections::HashMap;
use std::fs;
use std::path::Path;

pub fn create_manifest(sources_manifest: &HashMap<String, String>, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = target_dir.join("manifest.json");
    save_hashmap_json(sources_manifest, &manifest_path)?;
    Ok(())
}

pub fn extract_manifest(target_dir: &Path) -> Result<Option<HashMap<String, String>>, Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        return Ok(None);
    }

    let manifest_path = target_dir.join("manifest.json");
    if manifest_path.exists() {
        return Ok(Some(load_hashmap_json(&manifest_path)?));
    }

    Ok(None)
}

pub fn save_hashmap_json(map: &HashMap<String, String>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(map)?;
    fs::write(path, json)?;
    Ok(())
}

pub fn load_hashmap_json(path: &Path) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let map: HashMap<String, String> = serde_json::from_str(&json)?;
    Ok(map)
}

