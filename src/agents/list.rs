use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::frontmatter::{parse_frontmatter, AgentFrontmatter};
use crate::fs::RealFileSystem;
use crate::output;
use crate::agents::model::Agent;
use colored::Colorize;

pub fn list_agents(paths: &ClaudePaths, long: bool, json: bool) -> EccResult<()> {
    if !RealFileSystem::exists(&paths.agents) {
        println!("Agents directory not found: {}", paths.agents.display());
        return Ok(());
    }

    let entries = RealFileSystem::read_dir_entries(&paths.agents)?;
    let mut agents: Vec<Agent> = Vec::new();

    for entry in &entries {
        if !RealFileSystem::exists(entry) || entry.extension().map_or(true, |e| e != "md") {
            continue;
        }

        let content = match RealFileSystem::read_to_string(entry) {
            Ok(c) => c,
            Err(_) => continue,
        };

        let parsed = parse_frontmatter(&content).unwrap_or_else(|_| {
            crate::fs::frontmatter::MarkdownWithFrontmatter {
                frontmatter: None,
                body: content,
                body_lines: Vec::new(),
            }
        });

        let (name, description, model, tools) = match parsed.frontmatter {
            Some(ref fm) => {
                let af: Option<AgentFrontmatter> = serde_yaml::to_string(fm)
                    .ok()
                    .and_then(|s| serde_yaml::from_str::<AgentFrontmatter>(&s).ok());
                match af {
                    Some(a) => (a.name, a.description, a.model, a.tools),
                    None => {
                        let fallback = entry
                            .file_stem()
                            .unwrap_or_default()
                            .to_string_lossy()
                            .to_string();
                        (fallback, String::from("(no description)"), None, Vec::new())
                    }
                }
            }
            None => {
                let fallback = entry
                    .file_stem()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string();
                (fallback, String::from("(no description)"), None, Vec::new())
            }
        };

        agents.push(Agent {
            name,
            description,
            model,
            tools,
        });
    }

    agents.sort_by(|a, b| a.name.cmp(&b.name));

    if json {
        output::json_output(&agents);
        return Ok(());
    }

    output::header("Agents");
    println!(
        "  {} agents installed",
        agents.len().to_string().dimmed()
    );
    output::hr();

    for agent in &agents {
        let model_str = agent
            .model
            .as_ref()
            .map(|m| format!("[{}]", m))
            .unwrap_or_default();
        let tools_str = if agent.tools.is_empty() {
            String::new()
        } else {
            format!("  tools: {}", agent.tools.join(", ").dimmed())
        };

        println!(
            "  {:<30} {}{}{}",
            agent.name.bold().cyan(),
            model_str.yellow(),
            agent.description.dimmed(),
            tools_str
        );

        if long {
            for tool in &agent.tools {
                println!("    - {}", tool);
            }
        }
    }

    Ok(())
}
