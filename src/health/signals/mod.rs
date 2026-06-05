mod cache_ratio;
mod canary;
mod compaction;
pub mod context_usage;
mod corrections;
mod error_rate;
mod repeated;

pub use cache_ratio::CacheRatioSignal;
pub use canary::CanarySignal;
pub use compaction::CompactionSignal;
pub use context_usage::ContextUsageSignal;
pub use corrections::CorrectionsSignal;
pub use error_rate::ErrorRateSignal;
pub use repeated::RepeatedSignal;

use super::model::*;

pub fn all_signals() -> Vec<Box<dyn HealthSignal>> {
    vec![
        Box::new(CanarySignal::default()),
        Box::new(CompactionSignal),
        Box::new(CorrectionsSignal),
        Box::new(ErrorRateSignal),
        Box::new(ContextUsageSignal),
        Box::new(CacheRatioSignal),
        Box::new(RepeatedSignal),
    ]
}

pub fn run_all_signals(summary: &SessionSummary) -> Vec<SignalResult> {
    all_signals().iter().map(|s| s.analyze(summary)).collect()
}

pub fn compute_overall(results: &[SignalResult]) -> f64 {
    if results.is_empty() {
        return 0.0;
    }
    let total_weight: f64 = results.iter().map(|r| r.weight).sum();
    if total_weight == 0.0 {
        return 0.0;
    }
    results.iter().map(|r| r.score * r.weight).sum::<f64>() / total_weight
}
