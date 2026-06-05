use serde::Deserialize;
use serde_yaml::Value as YamlValue;

use crate::error::EccResult;

/// Parsed markdown file with optional YAML frontmatter.
#[derive(Debug)]
#[allow(dead_code)]
pub struct MarkdownWithFrontmatter {
    pub frontmatter: Option<YamlValue>,
    pub body: String,
    pub body_lines: Vec<String>,
}

/// Parse YAML frontmatter delimited by `---`.
pub fn parse_frontmatter(content: &str) -> EccResult<MarkdownWithFrontmatter> {
    let lines: Vec<&str> = content.lines().collect();

    if lines.is_empty() {
        return Ok(MarkdownWithFrontmatter {
            frontmatter: None,
            body: String::new(),
            body_lines: Vec::new(),
        });
    }

    // Check for opening ---
    if lines[0].trim() == "---" {
        // Find closing ---
        let mut end = None;
        for (i, line) in lines.iter().enumerate().skip(1) {
            if line.trim() == "---" {
                end = Some(i);
                break;
            }
        }

        if let Some(end_idx) = end {
            let fm_text: String = lines[1..end_idx]
                .iter()
                .map(|s| *s)
                .collect::<Vec<&str>>()
                .join("\n");

            let frontmatter: YamlValue = serde_yaml::from_str(&fm_text)?;

            let body_lines: Vec<String> =
                lines[end_idx + 1..].iter().map(|s| s.to_string()).collect();
            let body = body_lines.join("\n");

            return Ok(MarkdownWithFrontmatter {
                frontmatter: Some(frontmatter),
                body,
                body_lines,
            });
        }
    }

    // No frontmatter: entire content is body
    let body_lines: Vec<String> = lines.iter().map(|s| s.to_string()).collect();
    let body = body_lines.join("\n");
    Ok(MarkdownWithFrontmatter {
        frontmatter: None,
        body,
        body_lines,
    })
}

/// Agent frontmatter schema (for type-safe access).
#[derive(Debug, Deserialize)]
pub struct AgentFrontmatter {
    pub name: String,
    pub description: String,
    #[serde(default)]
    pub tools: Vec<String>,
    #[serde(default)]
    pub model: Option<String>,
}

/// Skill frontmatter schema.
#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct SkillFrontmatter {
    pub name: String,
    pub description: String,
}
