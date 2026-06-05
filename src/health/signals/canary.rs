use regex::Regex;

use super::super::model::*;

/// Self-calibrating canary: extracts behavioral fingerprints from the first
/// few assistant responses in a session, then tracks whether they survive.
///
/// No configuration required. Works for any Claude Code user regardless of
/// language, model, or instruction set.
///
/// Optionally augmented by explicit [[canary]] entries in .cdoc.toml.
pub struct CanarySignal {
    custom_patterns: Vec<CanaryPattern>,
}

#[derive(Clone)]
struct CanaryPattern {
    name: String,
    regex: Regex,
    max_miss: usize,
}

impl Default for CanarySignal {
    fn default() -> Self {
        Self {
            custom_patterns: load_custom_canaries(),
        }
    }
}

fn load_custom_canaries() -> Vec<CanaryPattern> {
    let config_path = dirs::home_dir()
        .map(|h| h.join(".claude").join("cdoc.toml"))
        .unwrap_or_default();

    let contents = match std::fs::read_to_string(&config_path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let config: serde_json::Value = match toml::from_str(&contents) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };

    let canaries = match config.get("canary").and_then(|c| c.as_array()) {
        Some(c) => c,
        None => return Vec::new(),
    };

    canaries
        .iter()
        .filter_map(|entry| {
            let name = entry.get("name")?.as_str()?.to_string();
            let pattern = entry.get("pattern")?.as_str()?;
            let max_miss = entry
                .get("max_miss")
                .and_then(|v| v.as_i64())
                .unwrap_or(5) as usize;
            let regex = Regex::new(pattern).ok()?;
            Some(CanaryPattern {
                name,
                regex,
                max_miss,
            })
        })
        .collect()
}

/// Auto-detect behavioral patterns from the first N assistant responses.
struct BehavioralBaseline {
    greeting_regex: Option<Regex>,
    uses_markdown: bool,
    chinese_ratio: f64,
}

impl BehavioralBaseline {
    fn from_session(all_assistant_text: &[String]) -> Self {
        let sample: Vec<&str> = all_assistant_text.iter()
            .take(10)
            .map(|s| s.as_str())
            .collect();

        if sample.is_empty() {
            return Self {
                greeting_regex: None,
                uses_markdown: false,
                chinese_ratio: 0.0,
            };
        }

        // 1. Detect greeting/prefix pattern from first 3 responses
        let greeting = detect_greeting(&sample);

        // 2. Detect markdown usage in early responses
        let uses_md = sample.iter().any(|s| s.contains("```") || s.contains("## ") || s.contains("**"));

        // 3. Detect language mix
        let n = sample.len().max(1);
        let cn_ratio = sample.iter()
            .map(|s| chinese_char_ratio(s))
            .sum::<f64>() / n as f64;

        Self {
            greeting_regex: greeting,
            uses_markdown: uses_md,
            chinese_ratio: cn_ratio,
        }
    }
}

/// Try to detect a consistent greeting/opening pattern from the first responses.
/// If the first N responses all start with the same first few characters,
/// those characters become a canary pattern.
fn detect_greeting(samples: &[&str]) -> Option<Regex> {
    if samples.len() < 3 {
        return None;
    }

    // Look at first 3 non-empty responses, take first 20 chars safely
    let openings: Vec<String> = samples.iter()
        .filter(|s| !s.is_empty())
        .take(3)
        .map(|s| {
            let trimmed = s.trim();
            trimmed.chars().take(20).collect::<String>()
        })
        .collect();

    if openings.len() < 2 {
        return None;
    }

    // Try to find a common prefix among at least 2 of the first 3
    for prefix_len in (3..=20).rev() {
        let prefixes: Vec<&str> = openings.iter()
            .filter_map(|o| {
                if o.chars().count() >= prefix_len {
                    Some(safe_prefix(o, prefix_len))
                } else {
                    None
                }
            })
            .collect();

        if prefixes.len() < 2 {
            continue;
        }

        // Count matching prefixes
        let first = prefixes[0];
        let matches = prefixes.iter().filter(|p| **p == first).count();

        if matches >= 2 && first.chars().all(|c| c.is_alphanumeric() || c == '~' || c == '～' || c == '-' || c == ':' || c == '/' || c == '(' || c == ')' || c == ' ') {
            let escaped = regex::escape(first);
            return Regex::new(&escaped).ok();
        }
    }

    None
}

fn safe_prefix(s: &str, n: usize) -> &str {
    s.char_indices()
        .nth(n)
        .map(|(i, _)| &s[..i])
        .unwrap_or(s)
}

/// Ratio of Chinese characters in a string (0.0 ~ 1.0).
fn chinese_char_ratio(text: &str) -> f64 {
    let total = text.chars().count();
    if total == 0 {
        return 0.0;
    }
    let cn = text.chars()
        .filter(|c| ('\u{4e00}'..='\u{9fff}').contains(c)
                 || ('\u{3400}'..='\u{4dbf}').contains(c))
        .count();
    cn as f64 / total as f64
}

impl HealthSignal for CanarySignal {
    fn name(&self) -> &str {
        "行为基线一致性"
    }
    fn category(&self) -> &str {
        "canary"
    }
    fn weight(&self) -> f64 {
        0.25
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let baseline = BehavioralBaseline::from_session(&summary.all_assistant_text);

        let mut issues: Vec<String> = Vec::new();
        let mut score = 1.0;

        // 1. Check greeting persistence
        if let Some(ref greeting_re) = &baseline.greeting_regex {
            let mut miss_count = 0;
            for text in summary.all_assistant_text.iter().rev() {
                if greeting_re.is_match(text) {
                    break;
                }
                miss_count += 1;
            }
            if miss_count > 5 {
                issues.push(format!("开头模式在最近 {} 轮中消失", miss_count));
                score -= 0.3;
            }
        }

        // 2. Check custom canary patterns
        for canary in &self.custom_patterns {
            let mut miss_count = 0;
            for text in summary.all_assistant_text.iter().rev() {
                if canary.regex.is_match(text) {
                    break;
                }
                miss_count += 1;
            }
            if miss_count > canary.max_miss {
                issues.push(format!("「{}」连续 {} 轮缺失", canary.name, miss_count));
                score -= 0.15;
            }
        }

        // 3. Check language consistency
        if summary.all_assistant_text.len() > 20 {
            let recent: Vec<&str> = summary.all_assistant_text.iter()
                .rev().take(10).map(|s| s.as_str()).collect();
            let recent_n = recent.len().max(1);
            let recent_cn_ratio = recent.iter()
                .map(|s| chinese_char_ratio(s))
                .sum::<f64>() / recent_n as f64;

            if baseline.chinese_ratio > 0.1 && recent_cn_ratio < 0.02 {
                issues.push("中文输出突然消失，可能切换到纯英文模式".into());
                score -= 0.15;
            } else if baseline.chinese_ratio < 0.02 && recent_cn_ratio > 0.1 {
                issues.push("英文输出突然切换为中文".into());
                score -= 0.15;
            }
        }

        // 4. Check markdown usage consistency
        if baseline.uses_markdown {
            let recent_has_md = summary.all_assistant_text.iter()
                .rev().take(10)
                .any(|s| s.contains("```") || s.contains("## ") || s.contains("**"));
            if !recent_has_md && summary.all_assistant_text.len() > 20 {
                issues.push("Markdown 格式突然消失".into());
                score -= 0.1;
            }
        }

        score = f64::max(score, 0.1);

        if issues.is_empty() {
            SignalResult {
                name: self.name().into(),
                category: self.category().into(),
                status: HealthStatus::Healthy,
                score: 1.0,
                weight: self.weight(),
                detail: "行为基线正常，未检测到偏离".into(),
            }
        } else {
            let truncated: Vec<String> = issues.iter().take(3).cloned().collect();
            let detail = truncated.join("; ");
            SignalResult {
                name: self.name().into(),
                category: self.category().into(),
                status: HealthStatus::from_score(score),
                score,
                weight: self.weight(),
                detail,
            }
        }
    }
}
