use cc_doctor::config::ClaudePaths;
use std::path::PathBuf;

fn claude_home() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".claude"))
}

#[test]
fn detect_paths_finds_home() {
    if let Some(home) = claude_home() {
        if !home.exists() {
            eprintln!("skipping: ~/.claude not found on this machine");
            return;
        }
    }
    let paths = ClaudePaths::detect();
    assert!(paths.is_ok(), "should detect claude config dir");
    let p = paths.unwrap();
    assert!(p.home.exists(), "~/.claude dir should exist");
}

#[test]
fn settings_json_exists_if_claude_installed() {
    if let Some(home) = claude_home() {
        if !home.exists() {
            eprintln!("skipping: ~/.claude not found");
            return;
        }
    }
    let paths = ClaudePaths::detect().unwrap();
    assert!(
        paths.settings.exists(),
        "settings.json should exist at {}",
        paths.settings.display()
    );
}

#[test]
fn projects_dir_if_present() {
    let paths = match ClaudePaths::detect() {
        Ok(p) => p,
        Err(_) => {
            eprintln!("skipping: claude dir not found");
            return;
        }
    };
    if !paths.projects.exists() {
        eprintln!("skipping: projects dir not found");
        return;
    }
    let files = cc_doctor::health::session::find_all_sessions(&paths.projects).unwrap();
    // CI may have zero sessions — that's fine, shouldn't panic
    eprintln!("found {} session files", files.len());
}

#[test]
fn session_search_does_not_panic() {
    // Test with a nonexistent path — should return error, not panic
    let nonexistent = std::path::PathBuf::from("/nonexistent/path/for/testing");
    let result = cc_doctor::health::session::find_all_sessions(&nonexistent);
    // WalkDir gracefully handles nonexistent paths by returning empty
    assert!(result.is_ok());
}

#[test]
fn project_session_search_handles_nonexistent() {
    let nonexistent = std::path::PathBuf::from("/nonexistent/project");
    let result = cc_doctor::health::session::find_project_sessions(&nonexistent);
    assert!(result.is_ok());
    assert!(result.unwrap().is_empty());
}
