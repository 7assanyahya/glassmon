//! Configuration loading and persistence from `~/.config/glassmon/config.toml`.

use std::fs;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_interval_ms")]
    pub interval_ms: u64,

    #[serde(default = "default_theme")]
    pub theme: String, // "dark" or "light"

    #[serde(default = "default_temp_unit")]
    pub temp_unit: String, // "celsius" or "fahrenheit"

    #[serde(default = "default_true")]
    pub mouse: bool,

    #[serde(default = "default_process_sort")]
    pub process_sort: String, // "cpu", "mem", "pid", "name"
}

fn default_interval_ms() -> u64 {
    1000
}

fn default_theme() -> String {
    "dark".to_string()
}

fn default_temp_unit() -> String {
    "celsius".to_string()
}

fn default_true() -> bool {
    true
}

fn default_process_sort() -> String {
    "cpu".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            interval_ms: default_interval_ms(),
            theme: default_theme(),
            temp_unit: default_temp_unit(),
            mouse: default_true(),
            process_sort: default_process_sort(),
        }
    }
}

impl Config {
    pub fn config_path() -> Option<PathBuf> {
        dirs::config_dir().map(|p| p.join("glassmon").join("config.toml"))
    }

    pub fn load() -> Self {
        if let Some(path) = Self::config_path()
            && path.exists()
            && let Ok(content) = fs::read_to_string(&path)
            && let Ok(cfg) = toml::from_str::<Config>(&content)
        {
            return cfg;
        }
        Self::default()
    }

    pub fn save(&self) -> Result<(), std::io::Error> {
        if let Some(path) = Self::config_path() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            let text = toml::to_string_pretty(self).unwrap_or_default();
            fs::write(path, text)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_defaults() {
        let cfg = Config::default();
        assert_eq!(cfg.interval_ms, 1000);
        assert_eq!(cfg.theme, "dark");
        assert_eq!(cfg.temp_unit, "celsius");
        assert!(cfg.mouse);
    }

    #[test]
    fn test_config_deserialize() {
        let toml_str = r#"
            interval_ms = 500
            theme = "light"
            temp_unit = "fahrenheit"
            mouse = false
        "#;
        let cfg: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(cfg.interval_ms, 500);
        assert_eq!(cfg.theme, "light");
        assert_eq!(cfg.temp_unit, "fahrenheit");
        assert!(!cfg.mouse);
    }
}
