use std::io::BufRead;
use std::path::Path;

use serde_json::Value;

use crate::error::EccResult;
use crate::fs::RealFileSystem;

/// Iterate over JSONL lines. Each line is parsed as serde_json::Value.
/// Malformed lines are skipped with a warning to stderr.
#[allow(dead_code)]
pub fn read_jsonl_lines(path: &Path) -> EccResult<Vec<Value>> {
    let reader = RealFileSystem::open_buffered(path)?;
    let mut results = Vec::new();

    for line in reader.lines() {
        let line = line?;
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        match serde_json::from_str::<Value>(trimmed) {
            Ok(v) => results.push(v),
            Err(e) => {
                eprintln!(
                    "Warning: skipping malformed JSONL line in {}: {}",
                    path.display(),
                    e
                );
            }
        }
    }

    Ok(results)
}

/// Parse a list of JSONL file paths and return all events sorted by timestamp.
#[allow(dead_code)]
pub fn read_multiple_sessions(paths: &[&Path]) -> EccResult<Vec<Value>> {
    let mut all = Vec::new();
    for path in paths {
        let mut events = read_jsonl_lines(path)?;
        all.append(&mut events);
    }
    Ok(all)
}
