pub mod loader;
pub mod schemas;

pub use loader::*;
pub use schemas::*;

use std::path::Path;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ModError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("TOML parse error: {0}")]
    Toml(#[from] toml::de::Error),
    #[error("Invalid mod structure: {0}")]
    InvalidStructure(String),
}

pub struct ModLoader {
    mods_path: std::path::PathBuf,
}

impl ModLoader {
    pub fn new(mods_path: impl AsRef<Path>) -> Self {
        Self {
            mods_path: mods_path.as_ref().to_path_buf(),
        }
    }

    pub fn load_mod(&self, mod_name: &str) -> Result<ModContent, ModError> {
        let mod_path = self.mods_path.join(mod_name);
        
        if !mod_path.exists() {
            return Err(ModError::InvalidStructure(format!("Mod '{}' not found", mod_name)));
        }

        let mut content = ModContent::default();

        // Load pipelines
        let pipelines_path = mod_path.join("pipelines.toml");
        if pipelines_path.exists() {
            let pipelines_data = std::fs::read_to_string(&pipelines_path)?;
            content.pipelines = toml::from_str(&pipelines_data)?;
        }

        // Load events
        let events_path = mod_path.join("events.toml");
        if events_path.exists() {
            let events_data = std::fs::read_to_string(&events_path)?;
            content.events = toml::from_str(&events_data)?;
        }

        // Load tech
        let tech_path = mod_path.join("tech.toml");
        if tech_path.exists() {
            let tech_data = std::fs::read_to_string(&tech_path)?;
            content.tech = toml::from_str(&tech_data)?;
        }

        Ok(content)
    }

    pub fn list_mods(&self) -> Result<Vec<String>, ModError> {
        let mut mods = Vec::new();
        
        if !self.mods_path.exists() {
            return Ok(mods);
        }

        for entry in std::fs::read_dir(&self.mods_path)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    mods.push(name.to_string());
                }
            }
        }

        Ok(mods)
    }
}
