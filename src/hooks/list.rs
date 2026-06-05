use std::collections::BTreeMap;

use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::RealFileSystem;
use crate::hooks::model::{HookConfig, HookMatcher, HookSource};
use crate::output;
use colored::Colorize;

pub fn list_hooks(paths: &ClaudePaths, long: bool, json: bool) -> EccResult<()> {
    let mut all_configs: Vec<HookConfig> = Vec::new();

    // Load from settings.json
    if RealFileSystem::exists(&paths.settings) {
        let content = RealFileSystem::read_to_string(&paths.settings)?;
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(hooks) = val.get("hooks") {
                let events = parse_hook_events(hooks);
                all_configs.push(HookConfig {
                    source: HookSource::Settings,
                    events,
                });
            }
        }
    }

    // Load from hooks.json
    if let Some(ref hooks_path) = paths.hooks_json {
        if RealFileSystem::exists(hooks_path) {
            let content = RealFileSystem::read_to_string(hooks_path)?;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                let events = parse_hook_events(&val);
                all_configs.push(HookConfig {
                    source: HookSource::HooksJson,
                    events,
                });
            }
        }
    }

    if json {
        // For JSON output, collect flattened representation
        let mut flat: Vec<serde_json::Value> = Vec::new();
        for config in &all_configs {
            for (event, matchers) in &config.events {
                for m in matchers {
                    for h in &m.hooks {
                        flat.push(serde_json::json!({
                            "source": config.source.to_string(),
                            "event": event,
                            "matcher": m.matcher,
                            "command": h.command,
                        }));
                    }
                }
            }
        }
        output::json_output(&flat);
        return Ok(());
    }

    output::header("Hooks");

    let total_hooks: usize = all_configs
        .iter()
        .flat_map(|c| {
            c.events
                .values()
                .flat_map(|v| v.iter().flat_map(|m| m.hooks.iter()))
        })
        .count();

    println!(
        "  {} sources, {} hooks total",
        all_configs.len().to_string().dimmed(),
        total_hooks.to_string().dimmed()
    );
    output::hr();

    for config in &all_configs {
        println!("  [{}]", config.source.to_string().bold());
        let mut events: Vec<_> = config.events.iter().collect();
        events.sort_by(|(a, _), (b, _)| a.cmp(b));

        for (event, matchers) in events {
            println!("    {:<20} {} matcher(s)", event.cyan(), matchers.len());
            if long {
                for m in matchers {
                    println!(
                        "      when: {:<20} {} hook(s)",
                        m.matcher.yellow(),
                        m.hooks.len()
                    );
                    for h in &m.hooks {
                        let preview = truncate_cmd(&h.command, 80);
                        println!("        $ {}", preview.dimmed());
                    }
                }
            }
        }
    }

    Ok(())
}

fn parse_hook_events(json: &serde_json::Value) -> BTreeMap<String, Vec<HookMatcher>> {
    let mut events = BTreeMap::new();
    if let Some(obj) = json.as_object() {
        for (key, val) in obj {
            let matchers: Vec<HookMatcher> = match serde_json::from_value(val.clone()) {
                Ok(m) => m,
                Err(_) => continue,
            };
            events.insert(key.clone(), matchers);
        }
    }
    events
}

fn truncate_cmd(cmd: &str, max: usize) -> String {
    if cmd.len() <= max {
        cmd.to_string()
    } else {
        format!("{}...", &cmd[..max - 3])
    }
}
