use std::fs;
use std::path::PathBuf;

/// Known bundled agents (shipped with the app)
const BUNDLED_AGENTS: &[&str] = &[
    "Bonzi", "Clippy", "F1", "Genie", "Genius",
    "Links", "Merlin", "Peedy", "Rocky", "Rover",
];

#[derive(serde::Serialize)]
pub struct AgentInfo {
    pub name: String,
    pub source: String,
}

/// Get the user agents directory (platform config dir / clippy-awakens / agents)
pub fn get_user_agents_dir() -> Option<PathBuf> {
    dirs::config_dir().map(|d| d.join("clippy-awakens").join("agents"))
}

/// List all available agents from bundled + user directories
#[tauri::command]
pub fn list_available_agents() -> Vec<AgentInfo> {
    let mut agents: Vec<AgentInfo> = BUNDLED_AGENTS
        .iter()
        .map(|name| AgentInfo {
            name: name.to_string(),
            source: "bundled".to_string(),
        })
        .collect();

    // Scan user agents directory
    if let Some(user_dir) = get_user_agents_dir() {
        if user_dir.exists() {
            if let Ok(entries) = fs::read_dir(&user_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.is_dir() {
                        // Verify it has agent.js and map.png
                        if path.join("agent.js").exists() && path.join("map.png").exists() {
                            let name = entry.file_name().to_string_lossy().to_string();
                            // Don't duplicate bundled agents
                            if !BUNDLED_AGENTS.contains(&name.as_str()) {
                                agents.push(AgentInfo {
                                    name,
                                    source: "user".to_string(),
                                });
                            }
                        }
                    }
                }
            }
        } else {
            // Create user agents directory for future use
            if let Err(e) = fs::create_dir_all(&user_dir) {
                log::warn!("Failed to create user agents directory: {}", e);
            } else {
                log::info!("Created user agents directory: {:?}", user_dir);
            }
        }
    }

    agents
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundled_agents_count() {
        assert_eq!(BUNDLED_AGENTS.len(), 10);
    }

    #[test]
    fn test_bundled_agents_contains_clippy() {
        assert!(BUNDLED_AGENTS.contains(&"Clippy"));
    }

    #[test]
    fn test_list_available_agents_includes_bundled() {
        let agents = list_available_agents();
        assert!(agents.len() >= 10);
        let names: Vec<&str> = agents.iter().map(|a| a.name.as_str()).collect();
        assert!(names.contains(&"Clippy"));
        assert!(names.contains(&"Merlin"));
        assert!(names.contains(&"Bonzi"));
    }

    #[test]
    fn test_all_bundled_agents_marked_as_bundled() {
        let agents = list_available_agents();
        for agent in &agents {
            if BUNDLED_AGENTS.contains(&agent.name.as_str()) {
                assert_eq!(agent.source, "bundled");
            }
        }
    }

    #[test]
    fn test_user_agents_dir_returns_path() {
        let dir = get_user_agents_dir();
        assert!(dir.is_some());
        let path = dir.unwrap();
        assert!(path.to_string_lossy().contains("clippy-awakens"));
    }
}
