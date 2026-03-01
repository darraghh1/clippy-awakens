use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tauri::Manager;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub x: f64,
    pub y: f64,
}

impl Default for Position {
    fn default() -> Self {
        // Bottom-right area (80% of a 1920x1080 screen)
        Self {
            x: 1536.0,
            y: 864.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub agent: String,
    pub muted: bool,
    pub position: Position,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            agent: "Clippy".to_string(),
            muted: false,
            position: Position::default(),
        }
    }
}

/// Get the config file path inside Tauri's app data directory
pub fn get_config_file(app_handle: &tauri::AppHandle) -> PathBuf {
    let path = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    fs::create_dir_all(&path).ok();
    path.join("config.json")
}

/// Load config from disk. Returns error if file missing or corrupt.
pub fn load_config(path: &Path) -> Result<AppConfig, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let config: AppConfig = serde_json::from_str(&content)?;
    Ok(config)
}

/// Save config to disk as pretty-printed JSON.
pub fn save_config(path: &Path, config: &AppConfig) -> Result<(), Box<dyn std::error::Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let content = serde_json::to_string_pretty(config)?;
    fs::write(path, content)?;
    Ok(())
}

/// Tauri command: get the current config (returns defaults if no file)
#[tauri::command]
pub fn get_config(app_handle: tauri::AppHandle) -> AppConfig {
    let path = get_config_file(&app_handle);
    load_config(&path).unwrap_or_default()
}

/// Tauri command: save the agent's drag position
#[tauri::command]
pub fn save_position_cmd(app_handle: tauri::AppHandle, x: f64, y: f64) {
    let path = get_config_file(&app_handle);
    let mut cfg = load_config(&path).unwrap_or_default();
    cfg.position = Position { x, y };
    if let Err(e) = save_config(&path, &cfg) {
        log::warn!("Failed to save position: {}", e);
    }
}

/// Tauri command: save the preferred agent name
#[tauri::command]
pub fn save_agent_preference(app_handle: tauri::AppHandle, agent: String) {
    let path = get_config_file(&app_handle);
    let mut cfg = load_config(&path).unwrap_or_default();
    cfg.agent = agent;
    if let Err(e) = save_config(&path, &cfg) {
        log::warn!("Failed to save agent preference: {}", e);
    }
}

/// Tauri command: save mute state
#[tauri::command]
pub fn save_mute_state(app_handle: tauri::AppHandle, muted: bool) {
    let path = get_config_file(&app_handle);
    let mut cfg = load_config(&path).unwrap_or_default();
    cfg.muted = muted;
    if let Err(e) = save_config(&path, &cfg) {
        log::warn!("Failed to save mute state: {}", e);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        assert_eq!(config.agent, "Clippy");
        assert!(!config.muted);
        assert_eq!(config.position.x, 1536.0);
        assert_eq!(config.position.y, 864.0);
    }

    #[test]
    fn test_default_position() {
        let pos = Position::default();
        assert!(pos.x > 0.0);
        assert!(pos.y > 0.0);
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let config = AppConfig {
            agent: "Merlin".to_string(),
            muted: true,
            position: Position { x: 100.0, y: 200.0 },
        };
        save_config(&path, &config).unwrap();

        let loaded = load_config(&path).unwrap();
        assert_eq!(loaded.agent, "Merlin");
        assert!(loaded.muted);
        assert_eq!(loaded.position.x, 100.0);
        assert_eq!(loaded.position.y, 200.0);
    }

    #[test]
    fn test_load_missing_file_returns_error() {
        let path = PathBuf::from("/nonexistent/config.json");
        let result = load_config(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_missing_file_fallback_to_defaults() {
        let path = PathBuf::from("/nonexistent/config.json");
        let config = load_config(&path).unwrap_or_default();
        assert_eq!(config.agent, "Clippy");
        assert!(!config.muted);
    }

    #[test]
    fn test_load_corrupt_file_returns_error() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");
        fs::write(&path, "not valid json!!!").unwrap();

        let result = load_config(&path);
        assert!(result.is_err());
    }

    #[test]
    fn test_save_creates_parent_dirs() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("nested").join("dir").join("config.json");

        let config = AppConfig::default();
        save_config(&path, &config).unwrap();

        assert!(path.exists());
    }

    #[test]
    fn test_config_roundtrip_all_agents() {
        let dir = tempdir().unwrap();
        let path = dir.path().join("config.json");

        let agents = [
            "Clippy", "Bonzi", "F1", "Genie", "Genius", "Links", "Merlin", "Peedy", "Rocky",
            "Rover",
        ];

        for agent in &agents {
            let config = AppConfig {
                agent: agent.to_string(),
                muted: false,
                position: Position::default(),
            };
            save_config(&path, &config).unwrap();
            let loaded = load_config(&path).unwrap();
            assert_eq!(loaded.agent, *agent);
        }
    }

    #[test]
    fn test_config_serialization_format() {
        let config = AppConfig {
            agent: "Clippy".to_string(),
            muted: false,
            position: Position { x: 100.0, y: 200.0 },
        };
        let json = serde_json::to_string_pretty(&config).unwrap();
        assert!(json.contains("\"agent\""));
        assert!(json.contains("\"muted\""));
        assert!(json.contains("\"position\""));
        assert!(json.contains("\"x\""));
        assert!(json.contains("\"y\""));
    }
}
