use clap::{Parser, Subcommand, ValueHint};

#[derive(Parser)]
#[command(
    name = "cdoc",
    about = "CDoc — CC Doctor (Claude Code Doctor): diagnose and manage rules, skills, hooks, agents, and session health",
    version
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,

    /// Output as JSON (machine-readable)
    #[arg(long, global = true)]
    pub json: bool,

    /// Disable colored output
    #[arg(long, global = true)]
    pub no_color: bool,

    /// Verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Path to .cdoc.toml config file
    #[arg(long, global = true, value_hint = ValueHint::FilePath)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum Command {
    /// List installed configuration
    List {
        #[command(subcommand)]
        what: ListTarget,
    },

    /// Validate configuration integrity
    Validate {
        #[command(subcommand)]
        what: Option<ValidateTarget>,
    },

    /// Show one-page overview of all Claude Code configuration
    Stats,

    /// Run full diagnostic scan
    Doctor,

    /// Session health analysis and monitoring
    Health {
        #[command(subcommand)]
        action: HealthAction,
    },
}

#[derive(Subcommand)]
pub enum ListTarget {
    /// List rule categories and files
    Rules {
        /// Show extended details (extends, cross-refs, skills)
        #[arg(long)]
        long: bool,
    },
    /// List installed skills
    Skills {
        #[arg(long)]
        long: bool,
    },
    /// List configured hooks
    Hooks {
        #[arg(long)]
        long: bool,
    },
    /// List installed agents
    Agents {
        #[arg(long)]
        long: bool,
    },
}

#[derive(Subcommand)]
pub enum ValidateTarget {
    /// Validate cross-references in rules
    Rules {},
    /// Validate hook syntax and script existence
    Hooks {
        /// Check that referenced scripts exist on disk
        #[arg(long)]
        check_scripts: bool,
    },
}

#[derive(Subcommand)]
pub enum HealthAction {
    /// Analyze a specific session by ID
    Session {
        /// Session UUID
        session_id: String,
    },
    /// Analyze the most recent session in the current project
    Latest,
    /// Analyze all sessions for a specific project
    Project {
        #[arg(value_hint = ValueHint::DirPath)]
        dir: String,

        /// Limit number of sessions to analyze
        #[arg(long)]
        limit: Option<usize>,

        /// Only analyze sessions from the last N days
        #[arg(long)]
        last_days: Option<u32>,
    },
    /// Watch for health issues in real-time
    Watch {
        /// Polling interval in seconds
        #[arg(long, default_value = "10")]
        interval: u64,

        /// Project directory to watch
        #[arg(long, value_hint = ValueHint::DirPath)]
        project: Option<String>,
    },
    /// Generate a comprehensive health report
    Report {
        /// Output file path
        #[arg(long, short)]
        output: Option<String>,

        /// Output format
        #[arg(long, default_value = "text")]
        format: String,
    },
}

#[derive(Subcommand)]
pub enum CanaryAction {
    /// Add a new canary pattern
    Add {
        name: String,
        #[arg(long)]
        pattern: String,
        #[arg(long, default_value = "per_turn")]
        canary_type: String,
        #[arg(long, default_value = "high")]
        severity: String,
        #[arg(long, default_value = "3")]
        max_miss: u32,
    },
    /// List all canary patterns
    List,
    /// Remove a canary pattern by name
    Remove { name: String },
    /// Test canary patterns against a session
    Test { session_id: String },
}

#[derive(Subcommand)]
pub enum MemoryAction {
    /// Initialize .ecc.toml config file
    Init {
        /// Create global config (~/.claude/ecc.toml) instead of project-level
        #[arg(long)]
        global: bool,
    },
    /// Add a memory entry
    Add {
        /// The memory text to store
        text: String,
        /// Optional tag for categorization
        #[arg(long)]
        tag: Option<String>,
    },
    /// Remove a memory entry by index
    Remove { index: usize },
    /// List all stored memories
    List {
        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,
    },
    /// Output memory entries for context injection
    Inject,
}
