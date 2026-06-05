use crate::config::ClaudePaths;
use crate::error::EccResult;
use crate::fs::RealFileSystem;
use crate::output;

pub fn validate_hooks(paths: &ClaudePaths, check_scripts: bool) -> EccResult<()> {
    let mut issues = 0;
    let mut total = 0;

    output::header("Hooks Validation");

    // Validate settings.json hooks
    if RealFileSystem::exists(&paths.settings) {
        let content = RealFileSystem::read_to_string(&paths.settings)?;
        if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
            if let Some(hooks) = val.get("hooks") {
                if let Some(obj) = hooks.as_object() {
                    for (event, matchers) in obj {
                        if let Some(arr) = matchers.as_array() {
                            for matcher in arr {
                                if let Some(cmds) = matcher.get("hooks").and_then(|h| h.as_array())
                                {
                                    for cmd in cmds {
                                        total += 1;
                                        let command = cmd
                                            .get("command")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("");
                                        validate_command(
                                            paths,
                                            "settings.json",
                                            event,
                                            command,
                                            check_scripts,
                                            &mut issues,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    // Validate hooks.json hooks
    if let Some(ref hooks_path) = paths.hooks_json {
        if RealFileSystem::exists(hooks_path) {
            let content = RealFileSystem::read_to_string(hooks_path)?;
            if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                if let Some(obj) = val.as_object() {
                    for (event, matchers) in obj {
                        if let Some(arr) = matchers.as_array() {
                            for matcher in arr {
                                if let Some(cmds) = matcher.get("hooks").and_then(|h| h.as_array())
                                {
                                    for cmd in cmds {
                                        total += 1;
                                        let command = cmd
                                            .get("command")
                                            .and_then(|c| c.as_str())
                                            .unwrap_or("");
                                        validate_command(
                                            paths,
                                            "hooks.json",
                                            event,
                                            command,
                                            check_scripts,
                                            &mut issues,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    println!();
    if issues == 0 {
        output::status(true, &format!("All {} hooks validated", total));
    } else {
        output::warn(&format!(
            "{} issue(s) found in {} hooks",
            issues, total
        ));
    }

    Ok(())
}

fn validate_command(
    _paths: &ClaudePaths,
    source: &str,
    event: &str,
    command: &str,
    check_scripts: bool,
    issues: &mut usize,
) {
    // Basic: check for empty command
    if command.trim().is_empty() {
        output::status(false, &format!("[{}] {}: empty command", source, event));
        *issues += 1;
        return;
    }

    // Basic: check for balanced quotes
    let double_quotes = command.matches('"').count();
    let single_quotes = command.matches('\'').count();
    if double_quotes % 2 != 0 || single_quotes % 2 != 0 {
        output::warn(&format!(
            "[{}] {}: possibly unbalanced quotes",
            source, event
        ));
        *issues += 1;
        return;
    }

    if check_scripts {
        // Extract the first script/binary path from the command
        let first_word = command
            .trim_matches(|c: char| c == '"' || c == '\'')
            .split_whitespace()
            .next()
            .unwrap_or("");

        // Skip inline node/python commands, they're valid
        if first_word == "node" || first_word == "python" || first_word == "python3" {
            return;
        }

        // Check if the script file exists (expand $HOME)
        let expanded = command.replace("$HOME", "");
        let script_path = expanded
            .trim_matches(|c: char| c == '"' || c == '\'')
            .split_whitespace()
            .next()
            .unwrap_or("");

        let path = std::path::Path::new(script_path);

        // Only check if it looks like a file path (contains / or \)
        if script_path.contains('/') || script_path.contains('\\') {
            let exists = if path.is_absolute() {
                path.exists()
            } else {
                // Try to resolve relative to home
                let home = dirs::home_dir().unwrap_or_default();
                home.join(path).exists()
            };

            if !exists {
                output::warn(&format!(
                    "[{}] {}: script not found: {}",
                    source, event, script_path
                ));
                *issues += 1;
            }
        }
    }
}
