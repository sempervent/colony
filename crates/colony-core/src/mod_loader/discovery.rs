use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use anyhow::Result;
use colony_modsdk::ModManifest;

pub fn discover_mods_in_directory(mods_dir: &Path) -> Result<Vec<ModManifest>> {
    let mut manifests = Vec::new();
    
    for entry in WalkDir::new(mods_dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_name() == "mod.toml")
    {
        let manifest_path = entry.path();
        if let Ok(manifest) = load_mod_manifest(manifest_path) {
            manifests.push(manifest);
        }
    }
    
    Ok(manifests)
}

fn load_mod_manifest(path: &Path) -> Result<ModManifest> {
    let content = std::fs::read_to_string(path)?;
    let manifest: ModManifest = toml::from_str(&content)?;
    Ok(manifest)
}
