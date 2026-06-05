use cdoc::health::model::*;
use cdoc::health::signals::*;

fn empty_summary() -> SessionSummary {
    SessionSummary {
        session_id: "test".into(),
        file_path: "test.jsonl".into(),
        message_count: 0,
        assistant_count: 0,
        user_count: 0,
        tool_call_count: 0,
        tool_error_count: 0,
        compaction_count: 0,
        total_input_tokens: 0,
        total_output_tokens: 0,
        total_cache_read: 0,
        total_cache_creation: 0,
        peak_input_tokens: 0,
        duration_minutes: None,
        all_assistant_text: vec![],
        user_messages: vec![],
        model_name: None,
        cwd: None,
        version: None,
        malformed_lines: 0,
        file_errors: vec![],
    }
}

fn summary_with_text(assistant: Vec<&str>, user: Vec<&str>) -> SessionSummary {
    let mut s = empty_summary();
    s.all_assistant_text = assistant.into_iter().map(|t| t.to_string()).collect();
    s.user_messages = user.into_iter().map(|t| t.to_string()).collect();
    s.assistant_count = s.all_assistant_text.len();
    s.user_count = s.user_messages.len();
    s
}

// ── Canary: empty session does not panic ──

#[test]
fn canary_empty_session_is_healthy() {
    let sig = CanarySignal::default();
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn canary_consistent_greeting_survives() {
    let sig = CanarySignal::default();
    let s = summary_with_text(
        vec!["曼波~ 你好", "曼波~ 好的", "曼波~ 没问题"],
        vec![],
    );
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
    assert!(result.score > 0.8);
}

// ── Compaction ──

#[test]
fn compaction_zero_is_healthy() {
    let sig = CompactionSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn compaction_many_is_critical() {
    let sig = CompactionSignal;
    let mut s = empty_summary();
    s.compaction_count = 5;
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Critical);
}

// ── Corrections ──

#[test]
fn corrections_zero_is_healthy() {
    let sig = CorrectionsSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn corrections_detects_chinese_patterns() {
    let sig = CorrectionsSignal;
    let s = summary_with_text(vec!["ok"], vec!["不对", "你忘了之前说的"]);
    let result = sig.analyze(&s);
    assert!(result.score < 1.0, "should detect corrections");
}

#[test]
fn corrections_detects_english_patterns() {
    let sig = CorrectionsSignal;
    let s = summary_with_text(vec!["ok"], vec!["no, I said something else"]);
    let result = sig.analyze(&s);
    assert!(result.score < 1.0, "should detect English corrections");
}

// ── Error rate ──

#[test]
fn error_rate_zero_tools_is_healthy() {
    let sig = ErrorRateSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn error_rate_high_is_critical() {
    let sig = ErrorRateSignal;
    let mut s = empty_summary();
    s.tool_call_count = 100;
    s.tool_error_count = 20; // 20%
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Critical);
}

// ── Context usage ──

#[test]
fn context_usage_empty_is_healthy() {
    let sig = ContextUsageSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn context_usage_detects_deepseek_v4_limit() {
    let sig = ContextUsageSignal;
    let mut s = empty_summary();
    s.model_name = Some("deepseek-v4-pro".into());
    s.peak_input_tokens = 800_000; // 80% of 1M
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Warning);
}

// ── Cache ratio ──

#[test]
fn cache_ratio_no_cache_is_healthy() {
    let sig = CacheRatioSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

// ── Repeated ──

#[test]
fn repeated_empty_is_healthy() {
    let sig = RepeatedSignal;
    let s = empty_summary();
    let result = sig.analyze(&s);
    assert_eq!(result.status, HealthStatus::Healthy);
}

#[test]
fn repeated_detects_duplicates() {
    let sig = RepeatedSignal;
    let s = summary_with_text(
        vec!["ok"],
        vec![
            "请帮我写一个函数",
            "请帮我写一个函数", // exact duplicate
        ],
    );
    let result = sig.analyze(&s);
    assert!(result.score < 1.0);
}

// ── All signals run without panic ──

#[test]
fn all_signals_handle_empty_session() {
    let s = empty_summary();
    let results = run_all_signals(&s);
    assert_eq!(results.len(), 7);
    for r in &results {
        assert!(r.score >= 0.0 && r.score <= 1.0);
    }
}

#[test]
fn all_signals_handle_null_model() {
    let mut s = empty_summary();
    s.model_name = None;
    s.peak_input_tokens = 50_000;
    s.all_assistant_text = vec!["hello world".into()];
    s.assistant_count = 1;
    let results = run_all_signals(&s);
    assert_eq!(results.len(), 7);
    for r in &results {
        assert!(r.score >= 0.0 && r.score <= 1.0);
    }
}

// ── Model context limit detection ──

#[test]
fn model_limits() {
    use cdoc::health::signals::context_usage::ContextUsageSignal;
    let sig = ContextUsageSignal;

    let cases = vec![
        ("deepseek-v4-pro", 1_000_000),
        ("claude-sonnet-4-6", 1_000_000),
        ("gpt-5.4", 1_000_000),
        ("gpt-5.1-codex", 400_000),
        ("qwen3-235b-a22b", 256_000),
        ("kimi-k2-thinking", 256_000),
        ("glm-4.7", 200_000),
        ("claude-haiku-4-5", 200_000),
        ("deepseek-v3", 128_000),
        ("qwen2.5-7b", 128_000),
        ("llama-3-70b", 128_000),
        ("unknown-model", 200_000),
    ];

    for (model, expected_limit) in cases {
        let mut s = empty_summary();
        s.model_name = Some(model.into());
        s.peak_input_tokens = (expected_limit as f64 * 0.5) as u64; // 50%
        let result = sig.analyze(&s);
        assert!(
            result.detail.contains("50%"),
            "Model {}: expected 50% usage but got: {}",
            model,
            result.detail
        );
    }
}
