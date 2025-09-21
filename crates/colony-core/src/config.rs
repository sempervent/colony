use serde::{Deserialize, Serialize};
use crate::ResourceTunables;

#[derive(Serialize, Deserialize)]
pub struct GameConfig {
    pub tunables: ResourceTunables,
    pub seed: u64,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self {
            tunables: ResourceTunables::default(),
            seed: 42,
        }
    }
}

pub fn load_config(path: &str) -> anyhow::Result<GameConfig> {
    if std::path::Path::new(path).exists() {
        let contents = std::fs::read_to_string(path)?;
        let config: GameConfig = toml::from_str(&contents)?;
        Ok(config)
    } else {
        // Create default config if it doesn't exist
        let config = GameConfig::default();
        save_config(&config, path)?;
        Ok(config)
    }
}

pub fn save_config(config: &GameConfig, path: &str) -> anyhow::Result<()> {
    let contents = toml::to_string_pretty(config)?;
    std::fs::write(path, contents)?;
    Ok(())
}
