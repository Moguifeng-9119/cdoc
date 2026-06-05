use colored::*;

/// Render a simple key-value table.
#[allow(dead_code)]
pub fn render_kv(rows: &[(&str, &str)], indent: usize) {
    let prefix = " ".repeat(indent);
    let _max_key = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(0);
    for (key, val) in rows {
        println!("{}{}: {}", prefix, key.bold(), val);
    }
}

/// Print a section header.
pub fn header(text: &str) {
    println!();
    println!("{}", text.bold().underline());
}

/// Print a status line with colored indicator.
pub fn status(ok: bool, text: &str) {
    if ok {
        println!("  {} {}", "✓".green(), text);
    } else {
        println!("  {} {}", "✗".red(), text);
    }
}

/// Print a warning line.
pub fn warn(text: &str) {
    println!("  {} {}", "⚠".yellow(), text.yellow());
}

/// Print info line.
pub fn info(text: &str) {
    println!("  {} {}", "ℹ".dimmed(), text.dimmed());
}

/// Print a JSON output (when --json flag is used).
pub fn json_output<T: serde::Serialize>(data: &T) {
    match serde_json::to_string_pretty(data) {
        Ok(json) => println!("{}", json),
        Err(e) => eprintln!("Failed to serialize: {}", e),
    }
}

/// Render a horizontal rule.
pub fn hr() {
    println!("{}", "─".repeat(60).dimmed());
}
