use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::RealFileSystem;
use crate::output;
use colored::Colorize;

pub fn show_stats(paths: &ClaudePaths, json: bool) -> EccResult<()> {
    // Count rules
    let (rule_cats, rule_files) = count_rules(paths);
    // Count skills
    let skill_count = count_skills(paths);
    // Count agents
    let agent_count = count_agents(paths);
    // Count hooks
    let (hook_sources, hook_count) = count_hooks(paths);
    // Count sessions
    let (session_count, session_size) = count_sessions(paths);

    if json {
        let stats = serde_json::json!({
            "rules": { "categories": rule_cats, "files": rule_files },
            "skills": skill_count,
            "agents": agent_count,
            "hooks": { "sources": hook_sources, "total": hook_count },
            "sessions": { "files": session_count, "total_size_kb": session_size / 1024 }
        });
        output::json_output(&stats);
        return Ok(());
    }

    println!();
    println!("{}", "  CDoc — CC Doctor".bold());
    println!("{}", "  ─────────────────────────────".dimmed());
    println!();

    // Rules row
    println!(
        "  {:<20} {} categories, {} files",
        "Rules".bold(),
        rule_cats.to_string().cyan(),
        rule_files.to_string().dimmed()
    );

    // Skills row
    println!(
        "  {:<20} {} skills",
        "Skills".bold(),
        skill_count.to_string().cyan()
    );

    // Agents row
    println!(
        "  {:<20} {} agents",
        "Agents".bold(),
        agent_count.to_string().cyan()
    );

    // Hooks row
    println!(
        "  {:<20} {} hooks ({} sources)",
        "Hooks".bold(),
        hook_count.to_string().cyan(),
        hook_sources.to_string().dimmed()
    );

    // Sessions row
    println!(
        "  {:<20} {} sessions ({})",
        "Sessions".bold(),
        session_count.to_string().cyan(),
        format_size(session_size).dimmed()
    );

    println!();

    // Claude dir
    println!(
        "  {:<20} {}",
        "Claude dir".bold(),
        paths.home.display().to_string().dimmed()
    );

    // History
    let history_info = if RealFileSystem::exists(&paths.history) {
        match RealFileSystem::file_size(&paths.history) {
            Ok(s) => {
                let lines = std::fs::read_to_string(&paths.history)
                    .map(|c| c.lines().count())
                    .unwrap_or(0);
                format!("{} entries ({})", lines, format_size(s))
            }
            Err(_) => "exists".into(),
        }
    } else {
        "not found".into()
    };
    println!("  {:<20} {}", "History".bold(), history_info.dimmed());

    println!();
    Ok(())
}

fn count_rules(paths: &ClaudePaths) -> (usize, usize) {
    let mut cats = 0;
    let mut files = 0;
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.rules) {
        for entry in &entries {
            if RealFileSystem::is_dir(entry) {
                cats += 1;
                if let Ok(f) = RealFileSystem::read_dir_entries(entry) {
                    files += f
                        .iter()
                        .filter(|p| p.extension().is_some_and(|e| e == "md"))
                        .count();
                }
            }
        }
    }
    (cats, files)
}

fn count_skills(paths: &ClaudePaths) -> usize {
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.skills) {
        entries
            .iter()
            .filter(|e| RealFileSystem::is_dir(e) && RealFileSystem::exists(&e.join("SKILL.md")))
            .count()
    } else {
        0
    }
}

fn count_agents(paths: &ClaudePaths) -> usize {
    if let Ok(entries) = RealFileSystem::read_dir_entries(&paths.agents) {
        entries
            .iter()
            .filter(|e| e.extension().is_some_and(|ext| ext == "md"))
            .count()
    } else {
        0
    }
}

fn count_hooks(paths: &ClaudePaths) -> (usize, usize) {
    let mut sources = 0;
    let mut total = 0;

    if RealFileSystem::exists(&paths.settings) {
        if let Ok(content) = RealFileSystem::read_to_string(&paths.settings) {
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(hooks) = val.get("hooks").and_then(|h| h.as_object()) {
                    sources += 1;
                    for (_, matchers) in hooks {
                        if let Some(arr) = matchers.as_array() {
                            for m in arr {
                                if let Some(cmds) = m.get("hooks").and_then(|h| h.as_array()) {
                                    total += cmds.len();
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Some(ref hooks_path) = paths.hooks_json {
        if RealFileSystem::exists(hooks_path) {
            if let Ok(content) = RealFileSystem::read_to_string(hooks_path) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(obj) = val.as_object() {
                        sources += 1;
                        for (_, matchers) in obj {
                            if let Some(arr) = matchers.as_array() {
                                for m in arr {
                                    if let Some(cmds) = m.get("hooks").and_then(|h| h.as_array()) {
                                        total += cmds.len();
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    (sources, total)
}

fn count_sessions(paths: &ClaudePaths) -> (usize, u64) {
    let mut count = 0;
    let mut total_size = 0u64;

    if let Ok(project_dirs) = RealFileSystem::read_dir_entries(&paths.projects) {
        for dir in &project_dirs {
            if !RealFileSystem::is_dir(dir) {
                continue;
            }
            // Skip subagents directory
            if dir.file_name().is_some_and(|n| n == "subagents") {
                continue;
            }
            if let Ok(files) = RealFileSystem::read_dir_entries(dir) {
                for file in &files {
                    if file.extension().is_some_and(|e| e == "jsonl") {
                        count += 1;
                        total_size += RealFileSystem::file_size(file).unwrap_or(0);
                    }
                }
            }
        }
    }

    (count, total_size)
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 * 1024 {
        format!("{:.1} MB", bytes as f64 / (1024.0 * 1024.0))
    } else if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
