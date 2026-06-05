use super::super::model::*;

/// Detects excessive context compaction (>3 compactions per session is concerning).
pub struct CompactionSignal;

impl HealthSignal for CompactionSignal {
    fn name(&self) -> &str {
        "上下文压缩频率"
    }
    fn category(&self) -> &str {
        "compaction"
    }
    fn weight(&self) -> f64 {
        0.20
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let count = summary.compaction_count;
        let (status, score, detail) = if count == 0 {
            (HealthStatus::Healthy, 1.0, "未检测到上下文压缩".into())
        } else if count <= 2 {
            (
                HealthStatus::Healthy,
                0.8,
                format!("检测到 {} 次压缩，属正常范围", count),
            )
        } else if count <= 4 {
            (
                HealthStatus::Warning,
                0.5,
                format!("检测到 {} 次压缩，上下文频繁被挤出", count),
            )
        } else {
            (
                HealthStatus::Critical,
                0.2,
                format!(
                    "检测到 {} 次压缩！关键指令可能已被挤出上下文窗口",
                    count
                ),
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
