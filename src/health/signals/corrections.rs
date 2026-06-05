use regex::Regex;

use super::super::model::*;

/// Detects user correction phrases indicating Claude Code forgot context.
pub struct CorrectionsSignal;

impl CorrectionsSignal {
    fn patterns() -> Vec<Regex> {
        vec![
            // Chinese
            Regex::new(r"不对").unwrap(),
            Regex::new(r"你忘[记了]").unwrap(),
            Regex::new(r"我说的是").unwrap(),
            Regex::new(r"不是这个").unwrap(),
            Regex::new(r"你搞错了").unwrap(),
            Regex::new(r"刚才说了").unwrap(),
            Regex::new(r"前面[说提]").unwrap(),
            Regex::new(r"已经[说讲]过").unwrap(),
            Regex::new(r"记[得]上下文").unwrap(),
            Regex::new(r"你之前说的").unwrap(),
            Regex::new(r"我之前说[过了]").unwrap(),
            Regex::new(r"重新说").unwrap(),
            Regex::new(r"怎么又").unwrap(),
            Regex::new(r"错了").unwrap(),
            Regex::new(r"不是这样").unwrap(),
            Regex::new(r"你又[来在]").unwrap(),
            Regex::new(r"别胡[说说扯]").unwrap(),
            // English
            Regex::new(r"no[,!]?\s*I said").unwrap(),
            Regex::new(r"you forgot").unwrap(),
            Regex::new(r"that's not what").unwrap(),
            Regex::new(r"you misunderstood").unwrap(),
            Regex::new(r"i already told").unwrap(),
            Regex::new(r"as I (said|mentioned)").unwrap(),
            Regex::new(r"that's not right").unwrap(),
            Regex::new(r"not correct").unwrap(),
            Regex::new(r"you'?re missing").unwrap(),
            Regex::new(r"you'?re ignoring").unwrap(),
            Regex::new(r"read my").unwrap(),
            Regex::new(r"i just said").unwrap(),
        ]
    }
}

impl HealthSignal for CorrectionsSignal {
    fn name(&self) -> &str {
        "用户纠正频率"
    }
    fn category(&self) -> &str {
        "corrections"
    }
    fn weight(&self) -> f64 {
        0.20
    }

    fn analyze(&self, summary: &SessionSummary) -> SignalResult {
        let mut corrections: Vec<&str> = Vec::new();

        for msg in &summary.user_messages {
            for pat in &Self::patterns() {
                if let Some(m) = pat.find(msg) {
                    corrections.push(m.as_str());
                    break; // one match per message
                }
            }
        }

        let count = corrections.len();
        let rate = if summary.assistant_count > 0 {
            count as f64 / summary.assistant_count as f64
        } else {
            0.0
        };

        let (status, score, detail) = if count == 0 {
            (HealthStatus::Healthy, 1.0, "用户未发出纠正指令".into())
        } else if rate < 0.05 {
            (
                HealthStatus::Healthy,
                0.8,
                format!(
                    "{} 次纠正 / {} 轮对话 = {:.1}%（正常）",
                    count,
                    summary.assistant_count,
                    rate * 100.0
                ),
            )
        } else if rate < 0.10 {
            (
                HealthStatus::Warning,
                0.5,
                format!(
                    "{} 次纠正（{:.1}%）— 用户频繁纠正，可能遗忘上下文",
                    count,
                    rate * 100.0
                ),
            )
        } else {
            (
                HealthStatus::Critical,
                0.2,
                format!("{} 次纠正（{:.1}%）— 上下文严重丢失", count, rate * 100.0),
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
