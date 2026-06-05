use super::super::model::*;

/// Detects when users repeat the same or similar instructions,
/// suggesting Claude Code didn't retain them.
pub struct RepeatedSignal;

impl HealthSignal for RepeatedSignal {
    fn name(&self) -> &str {
        "重复指令检测"
    }
    fn category(&self) -> &str {
        "repeated"
    }
    fn weight(&self) -> f64 {
        0.05
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        if summary.user_messages.len() < 2 {
            return SignalResult {
                name: self.name().into(),
                category: self.category().into(),
                status: HealthStatus::Healthy,
                score: 1.0,
                weight: self.weight(),
                detail: "消息过少，跳过检测".into(),
            };
        }

        let mut repeats = 0;
        let threshold = 0.85; // similarity threshold

        for i in 0..summary.user_messages.len() {
            for j in (i + 1)..summary.user_messages.len() {
                let a = &summary.user_messages[i];
                let b = &summary.user_messages[j];
                if a.len() < 5 || b.len() < 5 {
                    continue;
                }
                let max_len = a.len().max(b.len());
                let dist = lev_distance(a, b);
                let similarity = 1.0 - (dist as f64 / max_len as f64);
                if similarity > threshold {
                    repeats += 1;
                    if repeats > 5 {
                        break;
                    }
                }
            }
        }

        let (status, score, detail) = if repeats == 0 {
            (
                HealthStatus::Healthy,
                1.0,
                "未检测到重复指令".into(),
            )
        } else if repeats <= 2 {
            (
                HealthStatus::Healthy,
                0.8,
                format!("{} 条疑似重复指令", repeats),
            )
        } else if repeats <= 5 {
            (
                HealthStatus::Warning,
                0.5,
                format!("{} 条重复/相似指令 — 用户可能被迫反复提醒", repeats),
            )
        } else {
            (
                HealthStatus::Critical,
                0.3,
                format!("{} 条重复指令 — 上下文严重丢失", repeats),
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
