use super::super::model::*;

/// Detects when context window usage approaches model-specific limits.
/// Auto-detects the model limit from the session's model name.
pub struct ContextUsageSignal;

/// Matches model name (from JSONL) to known context window limit in tokens.
fn model_context_limit(model: &Option<String>) -> u64 {
    let name = match model.as_deref() {
        Some(n) => n.to_lowercase(),
        None => return 200_000,
    };

    // ── 1M+ tier ──
    if name.contains("claude-opus-4-7")
        || name.contains("claude-opus-4-6")
        || name.contains("claude-sonnet-4-6")
        || name.contains("claude-sonnet-4-5")
        || (name.contains("claude-sonnet") && (name.contains("4.6") || name.contains("4.5") || name.contains("4-6") || name.contains("4-5")))
        || name.contains("deepseek-v4")
        || name.contains("minimax-m3")
        || name.contains("minimax-m2.5")
        || name.contains("minimax-m1")
        || name.contains("qwen2.5-turbo")
        || name.contains("qwen-turbo")
        || (name.contains("qwen") && name.contains("1m"))
        || name.contains("mimo-v2-pro")
        || name.contains("mimo-v2.0-pro")
        || (name.contains("mimo") && name.contains("pro"))
        || name.contains("gpt-5.4")
        || name.contains("gpt-5.5")
        || name.contains("gemini-2.5")
        || name.contains("gemini-3")
    {
        return 1_000_000;
    }

    // ── 400K tier ──
    if name.contains("gpt-5.1")
        || name.contains("gpt-5.2")
        || name.contains("gpt-5.3")
        || (name.contains("gpt-5") && name.contains("codex"))
    {
        return 400_000;
    }

    // ── 256K tier ──
    if (name.contains("qwen3") && name.contains("235b"))
        || name.contains("qwen3-vl")
        || (name.contains("qwen") && name.contains("256k"))
        || name.contains("kimi-k2-instruct")
        || name.contains("kimi-k2-thinking")
        || name.contains("kimi-k2.1")
        || (name.contains("kimi") && name.contains("k2"))
        || name.contains("mimo-v2-flash")
        || (name.contains("mimo") && name.contains("flash"))
        || name.contains("gpt-5 ")
        || name.contains("gpt-5-")
        || name.contains("gpt-5.")
    {
        return 256_000;
    }

    // ── 200K tier ──
    if name.contains("claude")
        || name.contains("glm-4.7")
        || name.contains("glm-4-7")
        || name.contains("minimax-m2.7")
        || name.contains("minimax-m2-7")
        || (name.contains("minimax") && name.contains("m2"))
        || name.contains("gpt-4o")
        || name.contains("gpt-4.1")
        || name.contains("gpt-4-")
    {
        return 200_000;
    }

    // ── 128K tier ──
    if name.contains("deepseek")
        || name.contains("qwen")
        || name.contains("glm")
        || name.contains("kimi")
        || name.contains("minimax")
        || name.contains("mimo")
        || name.contains("gpt-3.5")
        || name.contains("gpt-4")
        || name.contains("llama")
        || name.contains("mistral")
        || name.contains("gemma")
        || name.contains("yi-")
        || name.contains("command-r")
    {
        return 128_000;
    }

    // ── default: assume 200K (Claude standard) ──
    200_000
}

impl HealthSignal for ContextUsageSignal {
    fn name(&self) -> &str {
        "上下文使用趋势"
    }
    fn category(&self) -> &str {
        "context_usage"
    }
    fn weight(&self) -> f64 {
        0.10
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let limit = model_context_limit(&summary.model_name);
        let peak = summary.peak_input_tokens;
        let pct = if limit > 0 {
            peak as f64 / limit as f64
        } else {
            0.0
        };

        let limit_str = if limit >= 1_000_000 {
            format!("{}M", limit / 1_000_000)
        } else {
            format!("{}K", limit / 1000)
        };

        let peak_str = if peak >= 1_000_000 {
            format!("{:.1}M", peak as f64 / 1_000_000.0)
        } else {
            format!("{}K", peak / 1000)
        };

        let (status, score, detail) = if peak == 0 {
            (HealthStatus::Healthy, 1.0, "无 token 数据".into())
        } else if pct < 0.5 {
            (
                HealthStatus::Healthy,
                1.0,
                format!("峰值 {} / {} ({}%) 充足", peak_str, limit_str, (pct * 100.0) as u32),
            )
        } else if pct < 0.75 {
            (
                HealthStatus::Healthy,
                0.8,
                format!("峰值 {} / {} ({}%) 偏高", peak_str, limit_str, (pct * 100.0) as u32),
            )
        } else if pct < 0.90 {
            (
                HealthStatus::Warning,
                0.5,
                format!("峰值 {} / {} ({}%) 接近上限，可能触发压缩", peak_str, limit_str, (pct * 100.0) as u32),
            )
        } else if pct <= 1.0 {
            (
                HealthStatus::Critical,
                0.2,
                format!("峰值 {} / {} ({}%) 已达上限！", peak_str, limit_str, (pct * 100.0) as u32),
            )
        } else {
            (
                HealthStatus::Critical,
                0.15,
                format!("峰值 {} / {} ({}%) 超出上限！", peak_str, limit_str, (pct * 100.0) as u32),
            )
        };

        SignalResult {
            name: self.name().into(),
            category: self.category().into(),
            status,
            score,
            weight: self.weight(),
            detail,
        }
    }
}
