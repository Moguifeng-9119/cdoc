use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::frontmatter::{parse_frontmatter, SkillFrontmatter};
use crate::fs::RealFileSystem;
use crate::output;
use crate::skills::model::Skill;
use colored::Colorize;

pub fn list_skills(paths: &ClaudePaths, long: bool, json: bool) -> EccResult<()> {
    if !RealFileSystem::exists(&paths.skills) {
        println!("Skills directory not found: {}", paths.skills.display());
        return Ok(());
    }

    let entries = RealFileSystem::read_dir_entries(&paths.skills)?;
    let mut skills: Vec<Skill> = Vec::new();

    for entry in &entries {
        if !RealFileSystem::is_dir(entry) {
            continue;
        }

        let skill_md = entry.join("SKILL.md");
        if !RealFileSystem::exists(&skill_md) {
            continue;
        }

        let name = entry
            .file_name()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string();

        let content = match RealFileSystem::read_to_string(&skill_md) {
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

        let description = parsed
            .frontmatter
            .as_ref()
            .and_then(|fm| {
                serde_yaml::to_string(fm)
                    .ok()
                    .and_then(|s| serde_yaml::from_str::<SkillFrontmatter>(&s).ok())
            })
            .map(|sf| sf.description)
            .unwrap_or_else(|| "(no description)".into());

        let has_agents_dir = RealFileSystem::is_dir(&entry.join("agents"));

        let file_count = walkdir::WalkDir::new(entry)
            .max_depth(4)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_type().is_file())
            .count();

        skills.push(Skill {
            name,
            description,
            has_agents_dir,
            file_count,
        });
    }

    skills.sort_by(|a, b| a.name.cmp(&b.name));

    if json {
        output::json_output(&skills);
        return Ok(());
    }

    output::header("Skills");
    println!(
        "  {} skills installed",
        skills.len().to_string().dimmed()
    );
    output::hr();

    for skill in &skills {
        let extra = if skill.has_agents_dir {
            " [has agents]".dimmed().to_string()
        } else {
            String::new()
        };

        println!("  {:<25} {}{}", skill.name.bold().cyan(), skill.description.dimmed(), extra);

        if long {
            println!("    {} files", skill.file_count.to_string().dimmed());
        }
    }

    Ok(())
}
