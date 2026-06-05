use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Hook event type.
#[derive(Debug, Clone, Serialize, PartialEq, Eq, PartialOrd, Ord)]
#[allow(dead_code)]
pub enum HookEvent {
    SessionStart,
    PreToolUse,
    PostToolUse,
    PostToolUseFailure,
    PreCompact,
    Stop,
    SessionEnd,
    Notification,
}

impl std::fmt::Display for HookEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SessionStart => write!(f, "SessionStart"),
            Self::PreToolUse => write!(f, "PreToolUse"),
            Self::PostToolUse => write!(f, "PostToolUse"),
            Self::PostToolUseFailure => write!(f, "PostToolUseFailure"),
            Self::PreCompact => write!(f, "PreCompact"),
            Self::Stop => write!(f, "Stop"),
            Self::SessionEnd => write!(f, "SessionEnd"),
            Self::Notification => write!(f, "Notification"),
        }
    }
}

impl std::str::FromStr for HookEvent {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "SessionStart" => Ok(Self::SessionStart),
            "PreToolUse" => Ok(Self::PreToolUse),
            "PostToolUse" => Ok(Self::PostToolUse),
            "PostToolUseFailure" => Ok(Self::PostToolUseFailure),
            "PreCompact" => Ok(Self::PreCompact),
            "Stop" => Ok(Self::Stop),
            "SessionEnd" => Ok(Self::SessionEnd),
            "Notification" => Ok(Self::Notification),
            other => Err(format!("unknown hook event: {}", other)),
        }
    }
}

/// A single hook command entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookCommand {
    #[serde(rename = "type")]
    pub hook_type: String,
    pub command: String,
}

/// A hook matcher group.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HookMatcher {
    pub matcher: String,
    pub hooks: Vec<HookCommand>,
}

/// Parsed hook configuration.
#[derive(Debug)]
pub struct HookConfig {
    pub source: HookSource,
    pub events: BTreeMap<String, Vec<HookMatcher>>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HookSource {
    Settings,
    HooksJson,
}

impl std::fmt::Display for HookSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Settings => write!(f, "settings.json"),
            Self::HooksJson => write!(f, "hooks.json"),
        }
    }
}
