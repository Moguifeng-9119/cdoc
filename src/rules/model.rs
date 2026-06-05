use serde::Serialize;

/// A rule category (directory under ~/.claude/rules/ecc/).
#[derive(Debug, Serialize)]
pub struct RuleCategory {
    pub name: String,
    pub files: Vec<RuleFile>,
}

/// A single rule file.
#[derive(Debug, Serialize)]
pub struct RuleFile {
    pub name: String,
    pub size: u64,
    pub has_frontmatter: bool,
    pub extends: Vec<String>,
    pub cross_refs: Vec<String>,
    pub skill_refs: Vec<String>,
}
