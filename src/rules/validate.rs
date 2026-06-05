use crate::config::ClaudePaths;
use crate::error::{EccError, EccResult};
use crate::fs::frontmatter::parse_frontmatter;
use crate::fs::RealFileSystem;
use crate::output;

pub fn validate_rules(paths: &ClaudePaths) -> EccResult<()> {
    if !RealFileSystem::exists(&paths.rules) {
        return Err(EccError::ClaudeDirNotFound(
            paths.rules.display().to_string(),
        ));
    }

    let entries = RealFileSystem::read_dir_entries(&paths.rules)?;
    let mut issues = 0;
    let mut checks = 0;

    output::header("Rules Validation");

    for entry in &entries {
        if !RealFileSystem::is_dir(entry) {
            continue;
        }

        let cat_name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let files = match RealFileSystem::read_dir_entries(entry) {
            Ok(f) => f,
            Err(_) => continue,
        };

        for file_path in &files {
            if file_path.extension().map_or(true, |e| e != "md") {
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

            let parsed = match parse_frontmatter(&content) {
                Ok(p) => p,
                Err(e) => {
                    output::warn(&format!("{}/{}: YAML parse error: {}", cat_name, fname, e));
                    issues += 1;
                    continue;
                }
            };

            // Check extends references
            let extends_re = regex::Regex::new(r"extends\s+\[([^\]]+)\]\(([^)]+)\)").unwrap();
            for line in &parsed.body_lines {
                for caps in extends_re.captures_iter(line) {
                    checks += 1;
                    let target = caps.get(2).unwrap().as_str();
                    // Resolve relative to the rule file's directory
                    let resolved = file_path
                        .parent()
                        .unwrap()
                        .join(target);
                    if !RealFileSystem::exists(&resolved) {
                        output::status(
                            false,
                            &format!(
                                "{}/{}: broken extends link → {}",
                                cat_name, fname, target
                            ),
                        );
                        issues += 1;
                    }
                }
            }

            // Check skill refs
            let skill_re =
                regex::Regex::new(r"See skill:\s*`?([a-zA-Z0-9_-]+)`?").unwrap();
            for line in &parsed.body_lines {
                for caps in skill_re.captures_iter(line) {
                    checks += 1;
                    let skill_name = caps.get(1).unwrap().as_str();
                    let skill_dir = paths.skills.join(skill_name);
                    let skill_md = skill_dir.join("SKILL.md");
                    if !RealFileSystem::exists(&skill_md) {
                        output::status(
                            false,
                            &format!(
                                "{}/{}: missing skill → {}",
                                cat_name, fname, skill_name
                            ),
                        );
                        issues += 1;
                    }
                }
            }
        }
    }

    println!();
    if issues == 0 {
        output::status(true, &format!("All {} checks passed", checks));
    } else {
        output::warn(&format!(
            "{} issue(s) found in {} checks",
            issues, checks
        ));
    }

    Ok(())
}
