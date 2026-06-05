mod agents;
mod cli;
mod config;
mod doctor;
mod error;
mod fs;
mod health;
mod hooks;
mod memory;
mod output;
mod rules;
mod skills;
mod stats;

use clap::Parser;
use cli::{Cli, Command, HealthAction};
use colored::Colorize;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();

    if cli.no_color {
        colored::control::set_override(false);
    }

    let paths = match config::ClaudePaths::detect() {
        Ok(p) => p,
        Err(e) => {
            eprintln!("{} {}", "error:".red().bold(), e);
            std::process::exit(1);
        }
    };

    let result = match &cli.command {
        Command::List { what } => match what {
            cli::ListTarget::Rules { long, category: _ } => {
                rules::list::list_rules(&paths, *long, cli.json)
            }
            cli::ListTarget::Skills { long } => {
                skills::list::list_skills(&paths, *long, cli.json)
            }
            cli::ListTarget::Hooks { long } => {
                hooks::list::list_hooks(&paths, *long, cli.json)
            }
            cli::ListTarget::Agents {
                long,
                tool: _,
                model: _,
            } => agents::list::list_agents(&paths, *long, cli.json),
        },

        Command::Validate { what } => {
            let target = what.as_ref().unwrap_or(&cli::ValidateTarget::Hooks {
                check_scripts: false,
            });
            match target {
                cli::ValidateTarget::Rules { strict: _ } => rules::validate::validate_rules(&paths),
                cli::ValidateTarget::Hooks { check_scripts } => {
                    hooks::validate::validate_hooks(&paths, *check_scripts)
                }
            }
        }

        Command::Stats => stats::show_stats(&paths, cli.json),

        Command::Doctor => doctor::run_doctor(&paths),

        Command::Health { action } => run_health(&paths, action, cli.json),

        Command::Canary { action: _ } => {
            println!("{}", "Canary management coming in Phase 2".dimmed());
            Ok(())
        }

        Command::Memory { action: _ } => {
            println!("{}", "Memory management coming in Phase 3".dimmed());
            Ok(())
        }

        Command::Completion { shell: _ } => {
            println!("{}", "Shell completions coming soon".dimmed());
            Ok(())
        }
    };

    if let Err(e) = result {
        eprintln!("{} {}", "error:".red().bold(), e);
        std::process::exit(1);
    }
}

fn run_health(
    paths: &config::ClaudePaths,
    action: &HealthAction,
    json: bool,
) -> crate::error::EccResult<()> {
    use health::model::HealthStatus;
    use health::session;
    use output;

    match action {
        HealthAction::Session { session_id } => {
            let file = find_session_by_id(&paths.projects, session_id)?;
            print_health_report(&file, json)?;
        }
        HealthAction::Latest => {
            let files = session::find_all_sessions(&paths.projects)?;
            if files.is_empty() {
                eprintln!("No sessions found");
                return Ok(());
            }
            let latest = files.last().unwrap();
            print_health_report(latest, json)?;
        }
        HealthAction::Project { dir, limit, last_days } => {
            let project_dir = PathBuf::from(dir);
            let mut files = session::find_project_sessions(&project_dir)?;

            if let Some(days) = last_days {
                let cutoff = std::time::SystemTime::now()
                    - std::time::Duration::from_secs((days * 86400) as u64);
                files.retain(|f| {
                    f.metadata()
                        .map(|m| m.modified().unwrap_or(std::time::UNIX_EPOCH) >= cutoff)
                        .unwrap_or(false)
                });
            }

            if let Some(limit) = limit {
                files.truncate(*limit);
            }

            if files.is_empty() {
                println!("{}", "No sessions found in project".dimmed());
                return Ok(());
            }

            println!();
            println!(
                "{}  {} sessions in {}",
                "📊".bold(),
                files.len().to_string().bold(),
                project_dir.display()
            );
            output::hr();

            let mut all_reports = Vec::new();
            for file in &files {
                match analyze_session(file) {
                    Ok(report) => {
                        if json {
                            all_reports.push(report);
                        } else {
                            print_compact_report(&report);
                        }
                    }
                    Err(e) => {
                        eprintln!("  {} {}: {}", "✗".red(), file.display(), e);
                    }
                }
            }

            if json && !all_reports.is_empty() {
                output::json_output(&all_reports);
            }

            // Summary
            if !all_reports.is_empty() {
                let avg_score = all_reports.iter().map(|r| r.overall_score).sum::<f64>()
                    / all_reports.len() as f64;
                output::hr();
                println!(
                    "  Average health: {:.2} — {}",
                    avg_score,
                    if avg_score >= 0.7 {
                        "Healthy".green()
                    } else if avg_score >= 0.4 {
                        "Warning".yellow()
                    } else {
                        "Critical".red()
                    }
                );
            }
        }
        HealthAction::Watch { interval, project: _ } => {
            println!();
            println!("{}  Watching for health issues...", "👀".bold());
            println!(
                "   Polling every {}s. Press Ctrl+C to stop.",
                interval
            );
            println!();

            loop {
                let files = session::find_all_sessions(&paths.projects)?;
                if let Some(latest) = files.last() {
                    match analyze_session(latest) {
                        Ok(report) => {
                            let now = chrono::Local::now().format("%H:%M:%S").to_string();
                            let icon = match report.overall_status {
                                HealthStatus::Healthy => "🟢",
                                HealthStatus::Warning => "🟡",
                                HealthStatus::Critical => "🔴",
                            };
                            println!(
                                "{} [{}] {} — score {:.2}",
                                icon,
                                now,
                                report.session.session_id,
                                report.overall_score
                            );
                            for sig in &report.signals {
                                if sig.status != HealthStatus::Healthy {
                                    println!("     {} {}: {}",
                                        match sig.status {
                                            HealthStatus::Critical => "🔴",
                                            HealthStatus::Warning => "🟡",
                                            _ => "  ",
                                        },
                                        sig.name,
                                        sig.detail
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            eprintln!("  {} {}", "✗".red(), e);
                        }
                    }
                }
                std::thread::sleep(std::time::Duration::from_secs(*interval));
            }
        }
        HealthAction::Report { output: out, format } => {
            let files = session::find_all_sessions(&paths.projects)?;
            if files.is_empty() {
                eprintln!("No sessions found");
                return Ok(());
            }
            let latest = files.last().unwrap();
            let report = analyze_session(latest)?;

            let output_str = match format.as_str() {
                "json" => serde_json::to_string_pretty(&report)?,
                _ => format_text_report(&report),
            };

            if let Some(path) = out {
                std::fs::write(path, &output_str)?;
                println!("Report written to {}", path);
            } else {
                println!("{}", output_str);
            }
        }
    }

    Ok(())
}

fn analyze_session(file: &std::path::Path) -> crate::error::EccResult<health::model::HealthReport> {
    use health::model::*;
    use health::session;
    use health::signals;

    let summary = session::parse_session(file)?;
    let signals = signals::run_all_signals(&summary);
    let overall_score = signals::compute_overall(&signals);
    let overall_status = HealthStatus::from_score(overall_score);

    Ok(HealthReport {
        session: summary,
        signals,
        overall_score,
        overall_status,
    })
}

fn print_health_report(file: &std::path::Path, json: bool) -> crate::error::EccResult<()> {
    let report = analyze_session(file)?;
    if json {
        output::json_output(&report);
    } else {
        println!("{}", format_text_report(&report));
    }
    Ok(())
}

fn print_compact_report(report: &health::model::HealthReport) {
    use health::model::HealthStatus;
    let s = &report.session;
    let icon = match report.overall_status {
        HealthStatus::Healthy => "🟢",
        HealthStatus::Warning => "🟡",
        HealthStatus::Critical => "🔴",
    };

    let dur = match s.duration_minutes {
        Some(m) if m >= 60.0 => format!("{}h {}m", (m / 60.0) as u32, (m % 60.0) as u32),
        Some(m) => format!("{}m", m as u32),
        None => "?m".into(),
    };

    println!(
        "{} {:.2} | {} | {} msg {} tool | {} compact | peak {}K | {}",
        icon,
        report.overall_score,
        s.session_id.chars().take(8).collect::<String>(),
        s.message_count,
        s.tool_call_count,
        s.compaction_count,
        s.peak_input_tokens / 1000,
        dur,
    );

    for sig in &report.signals {
        if sig.status != HealthStatus::Healthy {
            println!(
                "   {} {}: {}",
                match sig.status {
                    HealthStatus::Critical => "🔴",
                    HealthStatus::Warning => "🟡",
                    _ => "  ",
                },
                sig.name,
                sig.detail
            );
        }
    }
}

fn format_text_report(report: &health::model::HealthReport) -> String {
    use health::model::HealthStatus;
    use std::fmt::Write;

    let s = &report.session;
    let mut buf = String::new();

    let _ = writeln!(buf);
    let _ = writeln!(
        buf,
        "{}  Session Health: {}",
        "🔍".bold(),
        s.session_id
    );
    let _ = writeln!(buf, "   {}", "─".repeat(50).dimmed());

    let dur = match s.duration_minutes {
        Some(m) if m >= 60.0 => format!("{}h {}m", (m / 60.0) as u32, (m % 60.0) as u32),
        Some(m) => format!("{}m", m as u32),
        None => "unknown".into(),
    };

    let _ = writeln!(
        buf,
        "   Duration: {}  |  Messages: {}  |  Tools: {}  |  Compactions: {}",
        dur, s.message_count, s.tool_call_count, s.compaction_count
    );
    let _ = writeln!(
        buf,
        "   Tokens: {}K in / {}K out  |  Peak: {}K  |  Cache: {}K read / {}K write",
        s.total_input_tokens / 1000,
        s.total_output_tokens / 1000,
        s.peak_input_tokens / 1000,
        s.total_cache_read / 1000,
        s.total_cache_creation / 1000,
    );
    if let Some(ref model) = s.model_name {
        let _ = writeln!(buf, "   Model: {}", model);
    }
    if let Some(ref cwd) = s.cwd {
        let _ = writeln!(buf, "   CWD: {}", cwd);
    }

    let _ = writeln!(buf);
    let _ = writeln!(buf, "{}", "  Signals".bold());

    for sig in &report.signals {
        let icon = match sig.status {
            HealthStatus::Healthy => "🟢",
            HealthStatus::Warning => "🟡",
            HealthStatus::Critical => "🔴",
        };
        let _ = writeln!(
            buf,
            "  {} {} [{:.2}] {}",
            icon, sig.name, sig.score, sig.detail
        );
    }

    let _ = writeln!(buf);
    let _ = writeln!(
        buf,
        "  {} Overall: {} ({:.2})",
        match report.overall_status {
            HealthStatus::Healthy => "🟢",
            HealthStatus::Warning => "🟡",
            HealthStatus::Critical => "🔴",
        },
        match report.overall_status {
            HealthStatus::Healthy => "HEALTHY",
            HealthStatus::Warning => "WARNING",
            HealthStatus::Critical => "CRITICAL",
        },
        report.overall_score
    );
    let _ = writeln!(buf);

    buf
}

fn find_session_by_id(
    projects_dir: &std::path::Path,
    session_id: &str,
) -> crate::error::EccResult<std::path::PathBuf> {
    for entry in walkdir::WalkDir::new(projects_dir)
        .max_depth(2)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().map_or(false, |e| e == "jsonl") {
            if p.file_stem()
                .map_or(false, |s| s.to_string_lossy().starts_with(session_id))
            {
                return Ok(p.to_path_buf());
            }
        }
    }
    Err(crate::error::EccError::SessionNotFound(session_id.into()))
}
