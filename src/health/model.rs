use serde::{Deserialize, Serialize};

// ── JSONL wire format (one line = one event) ──

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct JsonlEvent {
    #[serde(rename = "type")]
    pub event_type: String,
    pub uuid: String,
    #[serde(rename = "sessionId")]
    pub session_id: String,
    pub timestamp: Option<String>,
    #[serde(rename = "parentUuid")]
    pub parent_uuid: Option<String>,
    pub message: Option<Message>,
    pub attachment: Option<Attachment>,
    #[serde(rename = "isSidechain")]
    pub is_sidechain: Option<bool>,
    pub cwd: Option<String>,
    pub version: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Message {
    pub role: String,
    pub content: serde_json::Value, // string (user) or array of blocks (assistant)
    pub model: Option<String>,
    pub stop_reason: Option<String>,
    pub usage: Option<UsageRaw>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UsageRaw {
    pub input_tokens: u64,
    #[serde(default)]
    pub output_tokens: u64,
    #[serde(default)]
    pub cache_read_input_tokens: u64,
    #[serde(default)]
    pub cache_creation_input_tokens: u64,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Attachment {
    #[serde(rename = "type")]
    pub att_type: String,
    pub content: Option<String>,
    #[serde(rename = "hookName")]
    pub hook_name: Option<String>,
}

// ── Content block (extracted from message.content array) ──

#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "type")]
#[allow(dead_code)]
pub enum ContentBlock {
    #[serde(rename = "text")]
    Text { text: String },
    #[serde(rename = "tool_use")]
    ToolUse {
        name: String,
        id: Option<String>,
        input: serde_json::Value,
    },
    #[serde(rename = "thinking")]
    Thinking {
        thinking: String,
        #[serde(default)]
        signature: Option<String>,
    },
    #[serde(rename = "tool_result")]
    ToolResult {
        content: serde_json::Value,
        #[serde(rename = "is_error")]
        is_error: Option<bool>,
        #[serde(rename = "tool_use_id")]
        tool_use_id: Option<String>,
    },
}

#[allow(dead_code)]
impl ContentBlock {
    pub fn text_value(&self) -> Option<&str> {
        match self {
            ContentBlock::Text { text } => Some(text.as_str()),
            ContentBlock::Thinking { thinking, .. } => Some(thinking.as_str()),
            _ => None,
        }
    }

    pub fn is_tool_error(&self) -> bool {
        matches!(self, ContentBlock::ToolResult { is_error: Some(true), .. })
    }
}

// ── Parsed content helpers ──

pub fn parse_content_blocks(content: &serde_json::Value) -> Vec<ContentBlock> {
    match content {
        serde_json::Value::Array(arr) => arr
            .iter()
            .filter_map(|v| serde_json::from_value::<ContentBlock>(v.clone()).ok())
            .collect(),
        _ => Vec::new(),
    }
}

pub fn extract_assistant_text(blocks: &[ContentBlock]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Text { text } => Some(text.clone()),
            _ => None,
        })
        .collect()
}

#[allow(dead_code)]
pub fn extract_thinking_text(blocks: &[ContentBlock]) -> Vec<String> {
    blocks
        .iter()
        .filter_map(|b| match b {
            ContentBlock::Thinking { thinking, .. } => Some(thinking.clone()),
            _ => None,
        })
        .collect()
}

// ── Session summary (aggregated) ──

#[derive(Debug, Clone, Serialize)]
pub struct SessionSummary {
    pub session_id: String,
    pub file_path: String,
    pub message_count: usize,
    pub assistant_count: usize,
    pub user_count: usize,
    pub tool_call_count: usize,
    pub tool_error_count: usize,
    pub compaction_count: usize,
    pub total_input_tokens: u64,
    pub total_output_tokens: u64,
    pub total_cache_read: u64,
    pub total_cache_creation: u64,
    pub peak_input_tokens: u64,
    pub duration_minutes: Option<f64>,
    pub all_assistant_text: Vec<String>,
    pub user_messages: Vec<String>,
    pub model_name: Option<String>,
    pub cwd: Option<String>,
    pub version: Option<String>,
    pub malformed_lines: usize,
    pub file_errors: Vec<String>,
}

// ── Health report output ──

#[derive(Debug, Clone, Serialize)]
pub struct HealthReport {
    pub session: SessionSummary,
    pub signals: Vec<SignalResult>,
    pub overall_score: f64,
    pub overall_status: HealthStatus,
}

#[derive(Debug, Clone, Serialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl HealthStatus {
    pub fn from_score(s: f64) -> Self {
        if s >= 0.7 { HealthStatus::Healthy }
        else if s >= 0.4 { HealthStatus::Warning }
        else { HealthStatus::Critical }
    }
}

// ── Signal result ──

#[derive(Debug, Clone, Serialize)]
pub struct SignalResult {
    pub name: String,
    pub category: String,
    pub status: HealthStatus,
    pub score: f64,
    pub weight: f64,
    pub detail: String,
}

// ── HealthSignal trait ──

pub trait HealthSignal {
    fn name(&self) -> &str;
    fn category(&self) -> &str;
    fn weight(&self) -> f64;
    fn analyze(&self, summary: &SessionSummary) -> SignalResult;
}

// ── Utility ──

pub fn lev_distance(a: &str, b: &str) -> usize {
    let a_chars: Vec<char> = a.chars().collect();
    let b_chars: Vec<char> = b.chars().collect();
    let n = a_chars.len();
    let m = b_chars.len();
    if n == 0 { return m; }
    if m == 0 { return n; }
    let mut prev: Vec<usize> = (0..=m).collect();
    let mut curr = vec![0; m + 1];
    for i in 1..=n {
        curr[0] = i;
        for j in 1..=m {
            let cost = if a_chars[i - 1] == b_chars[j - 1] { 0 } else { 1 };
            curr[j] = (prev[j] + 1).min(curr[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut curr);
    }
    prev[m]
}
