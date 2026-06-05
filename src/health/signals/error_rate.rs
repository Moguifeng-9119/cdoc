use super::super::model::*;

/// Detects elevated tool error rates.
pub struct ErrorRateSignal;

impl HealthSignal for ErrorRateSignal {
    fn name(&self) -> &str {
        "工具错误率"
    }
    fn category(&self) -> &str {
        "error_rate"
    }
    fn weight(&self) -> f64 {
        0.10
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let total = summary.tool_call_count;
        let errors = summary.tool_error_count;

        if total == 0 {
            return SignalResult {
                name: self.name().into(),
                category: self.category().into(),
                status: HealthStatus::Healthy,
                score: 1.0,
                weight: self.weight(),
                detail: "无工具调用，跳过检测".into(),
            };
        }

        let rate = errors as f64 / total as f64;

        let (status, score, detail) = if rate < 0.05 {
            (
                HealthStatus::Healthy,
                1.0,
                format!("{}/{} ({:.1}%) 错误率正常", errors, total, rate * 100.0),
            )
        } else if rate < 0.15 {
            (
                HealthStatus::Warning,
                0.6,
                format!("{}/{} ({:.1}%) 错误率偏高", errors, total, rate * 100.0),
            )
        } else {
            (
                HealthStatus::Critical,
                0.3,
                format!("{}/{} ({:.1}%) 错误率异常高", errors, total, rate * 100.0),
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
