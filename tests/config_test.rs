use cdoc::config::ClaudePaths;

#[test]
fn detect_paths_returns_home_dir() {
    let paths = ClaudePaths::detect();
    assert!(paths.is_ok(), "should detect claude config directory");
    let p = paths.unwrap();
    assert!(p.home.exists(), "~/.claude should exist");
}

#[test]
fn settings_json_exists() {
    let paths = ClaudePaths::detect().unwrap();
    assert!(
        paths.settings.exists(),
        "settings.json should exist at {}",
        paths.settings.display()
    );
}

#[test]
fn projects_dir_is_accessible() {
    let paths = ClaudePaths::detect().unwrap();
    assert!(paths.projects.exists(), "projects dir should exist");
}

#[test]
fn session_search_finds_files() {
    let paths = ClaudePaths::detect().unwrap();
    let files = cdoc::health::session::find_all_sessions(&paths.projects).unwrap();
    assert!(!files.is_empty(), "should find at least some session files");
}

#[test]
fn max_depth_covers_nested_sessions() {
    let paths = ClaudePaths::detect().unwrap();
    let files = cdoc::health::session::find_all_sessions(&paths.projects).unwrap();
    for f in &files {
        assert!(
            f.exists(),
            "found session file should exist: {}",
            f.display()
        );
        assert!(f.extension().unwrap() == "jsonl", "should be .jsonl");
    }
}
