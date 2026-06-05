use super::super::model::*;

/// Detects declining cache hit ratios.
pub struct CacheRatioSignal;

impl HealthSignal for CacheRatioSignal {
    fn name(&self) -> &str {
        "缓存命中率"
    }
    fn category(&self) -> &str {
        "cache_ratio"
    }
    fn weight(&self) -> f64 {
        0.05
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let total_cache = summary.total_cache_read + summary.total_cache_creation;

        if total_cache == 0 {
            return SignalResult {
                name: self.name().into(),
                category: self.category().into(),
                status: HealthStatus::Healthy,
                score: 1.0,
                weight: self.weight(),
                detail: "无缓存数据（可能模型/提供商不支持）".into(),
            };
        }

        let hit_rate = summary.total_cache_read as f64 / total_cache as f64;

        let (status, score, detail) = if hit_rate > 0.7 {
            (
                HealthStatus::Healthy,
                1.0,
                format!(
                    "命中率 {:.0}%（{}/{}）正常",
                    hit_rate * 100.0,
                    summary.total_cache_read / 1000,
                    (summary.total_cache_read + summary.total_cache_creation) / 1000
                ),
            )
        } else if hit_rate > 0.4 {
            (
                HealthStatus::Healthy,
                0.8,
                format!("命中率 {:.0}% 偏低", hit_rate * 100.0),
            )
        } else if hit_rate > 0.1 {
            (
                HealthStatus::Warning,
                0.5,
                format!("命中率 {:.0}% — 缓存利用率低，成本偏高", hit_rate * 100.0),
            )
        } else {
            (
                HealthStatus::Warning,
                0.4,
                format!("命中率 {:.0}% — 几乎无缓存命中", hit_rate * 100.0),
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
