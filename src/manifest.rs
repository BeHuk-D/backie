use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use colour::yellow;

pub fn create_manifest(sources_manifest: &BTreeMap<String, String>, target_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let manifest_path = target_dir.join("manifest.json");
    save_manifest_json(sources_manifest, &manifest_path)?;
    Ok(())
}

pub fn extract_manifest(target_dir: &Path) -> Result<Option<BTreeMap<String, String>>, Box<dyn std::error::Error>> {
    if !target_dir.exists() {
        return Ok(None);
    }

    let tmp_path = target_dir.join("manifest.json.tmp");
    if tmp_path.exists() {
        fs::remove_file(&tmp_path)?;
        yellow!("[MANAGER] "); println!("Удалён остаточный manifest.json.tmp от прерванного прогона");
    }

    let manifest_path = target_dir.join("manifest.json");
    if manifest_path.exists() {
        return Ok(Some(load_manifest_json(&manifest_path)?));
    }

    Ok(None)
}

pub fn save_manifest_json(map: &BTreeMap<String, String>, path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let json = serde_json::to_string_pretty(map)?;
    let tmp_path = path.with_extension("json.tmp");
    fs::write(&tmp_path, json)?;
    fs::rename(&tmp_path, path)?;
    Ok(())
}

pub fn load_manifest_json(path: &Path) -> Result<BTreeMap<String, String>, Box<dyn std::error::Error>> {
    let json = fs::read_to_string(path)?;
    let map: BTreeMap<String, String> = serde_json::from_str(&json)?;
    Ok(map)
}

