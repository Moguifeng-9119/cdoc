use std::path::PathBuf;

use crate::error::{EccError, EccResult};

/// Resolved paths to all Claude Code configuration locations.
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ClaudePaths {
    pub home: PathBuf,
    pub rules: PathBuf,
    pub skills: PathBuf,
    pub agents: PathBuf,
    pub hooks_json: Option<PathBuf>,
    pub settings: PathBuf,
    pub settings_local: PathBuf,
    pub projects: PathBuf,
    pub history: PathBuf,
    pub session_data: PathBuf,
    pub cdoc_global_config: PathBuf,
}

impl ClaudePaths {
    pub fn detect() -> EccResult<Self> {
        let home = dirs::home_dir()
            .ok_or_else(|| EccError::ClaudeDirNotFound("cannot find home directory".into()))?
            .join(".claude");

        if !home.exists() {
            return Err(EccError::ClaudeDirNotFound(home.display().to_string()));
        }

        let hooks_json = home.join("hooks").join("hooks.json");
        Ok(ClaudePaths {
            rules: home.join("rules").join("ecc"),
            skills: home.join("skills"),
            agents: home.join("agents"),
            hooks_json: if hooks_json.exists() {
                Some(hooks_json)
            } else {
                None
            },
            settings: home.join("settings.json"),
            settings_local: home.join("settings.local.json"),
            projects: home.join("projects"),
            history: home.join("history.jsonl"),
            session_data: home.join("session-data"),
            cdoc_global_config: home.join("cdoc.toml"),
            home,
        })
    }

    /// Find project-level .ecc.toml by walking up from cwd.
    #[allow(dead_code)]
    pub fn find_project_config(cwd: &std::path::Path) -> Option<PathBuf> {
        let mut current = Some(cwd);
        while let Some(dir) = current {
            let candidate = dir.join(".cdoc.toml");
            if candidate.exists() {
                return Some(candidate);
            }
            current = dir.parent();
        }
        None
    }
}
