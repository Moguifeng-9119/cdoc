use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::frontmatter::parse_frontmatter;
use crate::fs::RealFileSystem;
use crate::output;
use crate::rules::model::{RuleCategory, RuleFile};
use colored::Colorize;

/// Regex patterns for extracting cross-references from rule body lines.
fn find_extends(lines: &[String]) -> Vec<String> {
    let re = regex::Regex::new(r"extends\s+\[([^\]]+)\]").unwrap();
    lines
        .iter()
        .filter_map(|line| {
            re.captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}

fn find_cross_refs(lines: &[String]) -> Vec<String> {
    let re = regex::Regex::new(r"\[([^\]]+)\]\(([^)]+)\)").unwrap();
    lines
        .iter()
        .flat_map(|line| {
            re.captures_iter(line)
                .filter_map(|caps| {
                    let target = caps.get(2)?.as_str();
                    if target.ends_with(".md") {
                        Some(target.to_string())
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>()
        })
        .collect()
}

fn find_skill_refs(lines: &[String]) -> Vec<String> {
    let re = regex::Regex::new(r"See skill:\s*`?([a-zA-Z0-9_-]+)`?").unwrap();
    lines
        .iter()
        .filter_map(|line| {
            re.captures(line)
                .and_then(|caps| caps.get(1))
                .map(|m| m.as_str().to_string())
        })
        .collect()
}

pub fn list_rules(paths: &ClaudePaths, long: bool, json: bool) -> EccResult<()> {
    if !RealFileSystem::exists(&paths.rules) {
        println!("Rules directory not found: {}", paths.rules.display());
        return Ok(());
    }

    let entries = RealFileSystem::read_dir_entries(&paths.rules)?;
    let mut categories: Vec<RuleCategory> = Vec::new();

    for entry in &entries {
        if !RealFileSystem::is_dir(entry) {
            continue;
        }

        let name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let files = match RealFileSystem::read_dir_entries(entry) {
            Ok(f) => f,
            Err(_) => continue,
        };

        let mut rule_files: Vec<RuleFile> = Vec::new();
        for file_path in &files {
            if file_path.extension().is_none_or(|e| e != "md") {
                continue;
            }

            let fname = file_path
                .file_stem()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string();

            let content = match RealFileSystem::read_to_string(file_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            let parsed = parse_frontmatter(&content).unwrap_or_else(|_| {
                crate::fs::frontmatter::MarkdownWithFrontmatter {
                    frontmatter: None,
                    body: content.clone(),
                    body_lines: content.lines().map(String::from).collect(),
                }
            });

            let extends = find_extends(&parsed.body_lines);
            let cross_refs = find_cross_refs(&parsed.body_lines);
            let skill_refs = find_skill_refs(&parsed.body_lines);

            let size = RealFileSystem::file_size(file_path).unwrap_or(0);

            rule_files.push(RuleFile {
                name: fname,
                size,
                has_frontmatter: parsed.frontmatter.is_some(),
                extends,
                cross_refs,
                skill_refs,
            });
        }

        rule_files.sort_by(|a, b| a.name.cmp(&b.name));
        categories.push(RuleCategory {
            name,
            files: rule_files,
        });
    }

    categories.sort_by(|a, b| a.name.cmp(&b.name));

    if json {
        output::json_output(&categories);
        return Ok(());
    }

    output::header("Rules");

    let total_files: usize = categories.iter().map(|c| c.files.len()).sum();
    let total_size: u64 = categories
        .iter()
        .flat_map(|c| c.files.iter().map(|f| f.size))
        .sum();
    println!(
        "  {} categories, {} files, {}",
        categories.len(),
        total_files,
        format_size(total_size).dimmed()
    );
    output::hr();

    for cat in &categories {
        println!(
            "  {:<15} {} files",
            cat.name.bold().cyan(),
            cat.files.len().to_string().dimmed()
        );

        if long {
            for file in &cat.files {
                let mut meta = Vec::new();
                if !file.extends.is_empty() {
                    meta.push(format!("extends: {}", file.extends.join(", ")));
                }
                if !file.skill_refs.is_empty() {
                    meta.push(format!("skills: {}", file.skill_refs.join(", ")));
                }
                let meta_str = if meta.is_empty() {
                    String::new()
                } else {
                    format!("  {}", meta.join(" | ").dimmed())
                };
                println!(
                    "    {:<30} {}{}",
                    file.name,
                    format_size(file.size).dimmed(),
                    meta_str
                );
            }
        }
    }

    Ok(())
}

fn format_size(bytes: u64) -> String {
    if bytes >= 1024 {
        format!("{:.1} KB", bytes as f64 / 1024.0)
    } else {
        format!("{} B", bytes)
    }
}
