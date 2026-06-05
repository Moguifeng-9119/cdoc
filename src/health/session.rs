use std::collections::HashSet;
use std::io::BufRead;
use std::path::Path;

use crate::error::EccResult;
use crate::fs::RealFileSystem;

use super::model::*;

pub fn parse_session(path: &Path) -> EccResult<SessionSummary> {
    let reader = match RealFileSystem::open_buffered(path) {
        Ok(r) => r,
        Err(e) => {
            return Err(crate::error::EccError::Other(format!(
                "Cannot read session file {}: {}. Check file permissions.",
                path.display(),
                e
            )));
        }
    };

    let mut message_count = 0;
    let mut assistant_count = 0;
    let mut user_count = 0;
    let mut tool_call_count = 0;
    let mut tool_error_count = 0;
    let mut compaction_count = 0;
    let mut total_input_tokens: u64 = 0;
    let mut total_output_tokens: u64 = 0;
    let mut total_cache_read: u64 = 0;
    let mut total_cache_creation: u64 = 0;
    let mut peak_input_tokens: u64 = 0;
    let mut all_assistant_text: Vec<String> = Vec::new();
    let mut user_messages: Vec<String> = Vec::new();
    let mut model_name: Option<String> = None;
    let mut cwd: Option<String> = None;
    let mut version: Option<String> = None;
    let mut malformed_lines: usize = 0;
    let mut file_errors: Vec<String> = Vec::new();
    let max_errors = 5; // cap error messages to avoid bloat

    let mut prev_input_tokens: u64 = 0;
    let mut timestamps: Vec<String> = Vec::new();
    let mut error_tool_ids: HashSet<String> = HashSet::new();

    let mut line_num: usize = 0;

    for line in reader.lines() {
        line_num += 1;
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                malformed_lines += 1;
                if file_errors.len() < max_errors {
                    file_errors.push(format!("Line {}: read error: {}", line_num, e));
                }
                continue;
            }
        };
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let event: JsonlEvent = match serde_json::from_str(trimmed) {
            Ok(e) => e,
            Err(e) => {
                malformed_lines += 1;
                if file_errors.len() < max_errors {
                    let preview: String = trimmed.chars().take(80).collect();
                    file_errors.push(format!(
                        "Line {}: invalid JSON in session log — consider deleting this session file: {}",
                        line_num, preview
                    ));
                    let _ = e; // suppress unused warning
                }
                continue;
            }
        };

        // Capture metadata once
        if cwd.is_none() {
            cwd = event.cwd.clone();
        }
        if version.is_none() {
            version = event.version.clone();
        }
        if let Some(ref ts) = event.timestamp {
            timestamps.push(ts.clone());
        }

        // Skip sidechain events (sub-agents)
        if event.is_sidechain.unwrap_or(false) {
            continue;
        }

        match event.event_type.as_str() {
            "assistant" => {
                assistant_count += 1;
                message_count += 1;

                if let Some(ref msg) = event.message {
                    if model_name.is_none() {
                        model_name = msg.model.clone();
                    }

                    let blocks = parse_content_blocks(&msg.content);
                    let texts = extract_assistant_text(&blocks);
                    let thoughts = extract_thinking_text(&blocks);
                    all_assistant_text.extend(texts);
                    all_assistant_text.extend(thoughts);

                    for block in &blocks {
                        if matches!(block, ContentBlock::ToolUse { .. }) {
                            tool_call_count += 1;
                        }
                    }

                    if let Some(ref u) = msg.usage {
                        total_input_tokens += u.input_tokens;
                        total_output_tokens += u.output_tokens;
                        total_cache_read += u.cache_read_input_tokens;
                        total_cache_creation += u.cache_creation_input_tokens;

                        if u.input_tokens > peak_input_tokens {
                            peak_input_tokens = u.input_tokens;
                        }

                        let floor = (peak_input_tokens / 4).max(50000);
                        if prev_input_tokens > floor
                            && u.input_tokens > 0
                            && (u.input_tokens as f64) < (prev_input_tokens as f64 * 0.4)
                        {
                            compaction_count += 1;
                        }
                        prev_input_tokens = u.input_tokens;
                    }
                }
            }

            "user" => {
                user_count += 1;
                message_count += 1;

                if let Some(ref msg) = event.message {
                    match &msg.content {
                        serde_json::Value::String(s) => {
                            user_messages.push(s.clone());
                        }
                        serde_json::Value::Array(arr) => {
                            for block in arr {
                                if let Ok(cb) =
                                    serde_json::from_value::<ContentBlock>(block.clone())
                                {
                                    match cb {
                                        ContentBlock::ToolResult {
                                            is_error: Some(true),
                                            tool_use_id,
                                            ..
                                        } => {
                                            tool_error_count += 1;
                                            if let Some(id) = tool_use_id {
                                                error_tool_ids.insert(id);
                                            }
                                        }
                                        ContentBlock::Text { text } => {
                                            user_messages.push(text);
                                        }
                                        _ => {}
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            _ => {}
        }
    }

    if tool_error_count == 0 {
        tool_error_count = error_tool_ids.len();
    }

    let duration = if timestamps.len() >= 2 {
        let first = &timestamps[0];
        let last = &timestamps[timestamps.len() - 1];
        let t1 = chrono::DateTime::parse_from_rfc3339(first).map(|d| d.with_timezone(&chrono::Utc));
        let t2 = chrono::DateTime::parse_from_rfc3339(last).map(|d| d.with_timezone(&chrono::Utc));
        match (t1, t2) {
            (Ok(t1), Ok(t2)) => {
                let dur = t2 - t1;
                Some(dur.num_minutes() as f64 + dur.num_seconds() as f64 / 60.0)
            }
            _ => None,
        }
    } else {
        None
    };

    Ok(SessionSummary {
        session_id: Path::new(path)
            .file_stem()
            .unwrap_or_default()
            .to_string_lossy()
            .to_string(),
        file_path: path.display().to_string(),
        message_count,
        assistant_count,
        user_count,
        tool_call_count,
        tool_error_count,
        compaction_count,
        total_input_tokens,
        total_output_tokens,
        total_cache_read,
        total_cache_creation,
        peak_input_tokens,
        duration_minutes: duration,
        all_assistant_text,
        user_messages,
        model_name,
        cwd,
        version,
        malformed_lines,
        file_errors,
    })
}

/// Find all JSONL files under ~/.claude/projects/
pub fn find_all_sessions(projects_dir: &Path) -> EccResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(projects_dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "jsonl") {
            if p.parent()
                .is_some_and(|parent| parent.file_name().is_some_and(|n| n == "subagents"))
            {
                continue;
            }
            files.push(p.to_path_buf());
        }
    }
    files.sort_by_key(|f| {
        f.metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    Ok(files)
}

/// Find sessions within a specific project directory
pub fn find_project_sessions(project_dir: &Path) -> EccResult<Vec<std::path::PathBuf>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(project_dir)
        .max_depth(10)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let p = entry.path();
        if p.extension().is_some_and(|e| e == "jsonl") {
            if p.parent()
                .is_some_and(|parent| parent.file_name().is_some_and(|n| n == "subagents"))
            {
                continue;
            }
            files.push(p.to_path_buf());
        }
    }
    files.sort_by_key(|f| {
        f.metadata()
            .ok()
            .and_then(|m| m.modified().ok())
            .unwrap_or(std::time::SystemTime::UNIX_EPOCH)
    });
    Ok(files)
}
