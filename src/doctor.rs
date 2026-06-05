use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::RealFileSystem;
use crate::output;
use colored::Colorize;

pub fn run_doctor(paths: &ClaudePaths) -> EccResult<()> {
    println!();
    println!("{}", "  🔍 CDoc Full Diagnostic".bold());
    println!("{}", "  ────────────────────────────────".dimmed());
    println!();

    // 1. Check directories exist
    output::header("Directory Structure");
    check_dir("Rules", &paths.rules);
    check_dir("Skills", &paths.skills);
    check_dir("Agents", &paths.agents);
    check_dir("Settings", &paths.settings);
    check_dir("Projects", &paths.projects);
    if let Some(ref hj) = paths.hooks_json {
        check_dir("Hooks JSON", hj);
    } else {
        output::info("Hooks JSON not found (using settings.json only)");
    }

    // 2. Count everything
    output::header("Configuration Health");
    let rule_count = count_dirs(&paths.rules);
    let skill_count = count_skills(&paths.skills);
    let agent_count = count_files(&paths.agents, "md");
    output::info(&format!(
        "Rules: {} categories | Skills: {} | Agents: {}",
        rule_count, skill_count, agent_count
    ));

    // 3. Check for potential issues
    output::header("Potential Issues");

    // Empty directories in rules
    check_empty_rule_dirs(paths);

    // Skills without SKILL.md
    check_broken_skills(paths);

    // Agents without proper frontmatter
    check_broken_agents(paths);

    // Settings file parseability
    check_settings_valid(paths);

    // Session file count
    check_sessions(paths);

    println!();
    output::info("CDoc check complete. Use 'cdoc validate' for detailed validation.");
    println!();

    Ok(())
}

fn check_dir(name: &str, path: &std::path::Path) {
    if RealFileSystem::exists(path) {
        output::status(true, &format!("{}: {}", name, path.display()));
    } else {
        output::status(false, &format!("{}: not found at {}", name, path.display()));
    }
}

fn count_dirs(path: &std::path::Path) -> usize {
    RealFileSystem::read_dir_entries(path)
        .map(|entries| {
            entries
                .iter()
                .filter(|e| RealFileSystem::is_dir(e))
                .count()
        })
        .unwrap_or(0)
}

fn count_skills(path: &std::path::Path) -> usize {
    RealFileSystem::read_dir_entries(path)
        .map(|entries| {
            entries
                .iter()
                .filter(|e| {
                    RealFileSystem::is_dir(e) && RealFileSystem::exists(&e.join("SKILL.md"))
                })
                .count()
        })
        .unwrap_or(0)
}

fn count_files(path: &std::path::Path, ext: &str) -> usize {
    RealFileSystem::read_dir_entries(path)
        .map(|entries| {
            entries
                .iter()
                .filter(|e| e.extension().map_or(false, |e| e == ext))
                .count()
        })
        .unwrap_or(0)
}

fn check_empty_rule_dirs(paths: &ClaudePaths) {
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.rules) {
        for entry in &entries {
            if RealFileSystem::is_dir(entry) {
                let mds = RealFileSystem::read_dir_entries(entry)
                    .map(|files| {
                        files
                            .iter()
                            .filter(|f| f.extension().map_or(false, |e| e == "md"))
                            .count()
                    })
                    .unwrap_or(0);
                if mds == 0 {
                    let name = entry
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    output::warn(&format!("Empty rule category: {}", name));
                }
            }
        }
    }
}

fn check_broken_skills(paths: &ClaudePaths) {
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.skills) {
        for entry in &entries {
            if RealFileSystem::is_dir(entry) {
                let skill_md = entry.join("SKILL.md");
                if !RealFileSystem::exists(&skill_md) {
                    let name = entry
                        .file_name()
                        .unwrap_or_default()
                        .to_string_lossy();
                    output::warn(&format!("Skill directory without SKILL.md: {}", name));
                }
            }
        }
    }
}

fn check_broken_agents(paths: &ClaudePaths) {
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.agents) {
        for entry in &entries {
            if entry.extension().map_or(true, |e| e != "md") {
                continue;
            }
            if let Ok(content) = RealFileSystem::read_to_string(entry) {
                if !content.starts_with("---") {
                    let name = entry
                        .file_stem()
                        .unwrap_or_default()
                        .to_string_lossy();
                    output::warn(&format!("Agent without frontmatter: {}", name));
                }
            }
        }
    }
}

fn check_settings_valid(paths: &ClaudePaths) {
    if RealFileSystem::exists(&paths.settings) {
        match RealFileSystem::read_to_string(&paths.settings) {
            Ok(content) => match serde_json::from_str::<serde_json::Value>(&content) {
                Ok(_) => output::status(true, "settings.json is valid JSON"),
                Err(e) => output::warn(&format!("settings.json parse error: {}", e)),
            },
            Err(e) => output::warn(&format!("Cannot read settings.json: {}", e)),
        }
    }
}

fn check_sessions(paths: &ClaudePaths) {
    if let Ok(project_dirs) = RealFileSystem::read_dir_entries(&paths.projects) {
        let mut total = 0;
        let mut corrupt = 0;
        for dir in &project_dirs {
            if !RealFileSystem::is_dir(dir) {
                continue;
            }
            if dir.file_name().map_or(false, |n| n == "subagents") {
                continue;
            }
            if let Ok(files) = RealFileSystem::read_dir_entries(dir) {
                for file in &files {
                    if file.extension().map_or(false, |e| e == "jsonl") {
                        total += 1;
                        // Quick check: first line is valid JSON?
                        if let Ok(content) = RealFileSystem::read_to_string(file) {
                            if let Some(first_line) = content.lines().next() {
                                if serde_json::from_str::<serde_json::Value>(first_line).is_err() {
                                    corrupt += 1;
                                }
                            }
                        }
                    }
                }
            }
        }
        output::info(&format!(
            "Session files: {} total, {} corrupt first lines",
            total, corrupt
        ));
    }
}
