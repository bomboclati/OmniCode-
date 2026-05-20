use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "omni", about = "OmniCode - Autonomous AI Coding Agent", version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    #[command(about = "Launch the TUI (default)")]
    Omni,

    #[command(about = "Start the web server")]
    Serve {
        #[arg(short, long, default_value = "9420")]
        port: u16,
    },

    #[command(about = "Run a single agent task headlessly")]
    Task {
        description: String,
    },

    #[command(about = "Run swarm mode headlessly")]
    Swarm {
        task: String,
    },

    #[command(subcommand, about = "Sentinel operations")]
    Sentinel(SentinelCommands),

    #[command(subcommand, about = "Skill management")]
    Skill(SkillCommands),

    #[command(about = "Set up PR review webhook")]
    Review,

    #[command(about = "Run self-healing on test failures")]
    Heal,

    #[command(about = "Run prompt-to-binary pipeline")]
    Release {
        description: String,
    },

    #[command(about = "Migration mode")]
    Archeologist {
        #[arg(long)]
        migrate: String,
        target_lang: String,
    },

    #[command(about = "Launch onboarding wizard")]
    Onboard,

    #[command(about = "Run dependency guardian daemon")]
    Guardian,

    #[command(about = "Query decision historian")]
    Why {
        query: String,
    },

    #[command(about = "Semantic code search")]
    Find {
        query: String,
    },

    #[command(about = "Create collaboration session")]
    Share,

    #[command(about = "Join collaboration session")]
    Join {
        code: String,
    },

    #[command(about = "Open config in editor")]
    Config,

    #[command(about = "Regenerate all documentation")]
    Docs,

    #[command(about = "Run deployment pipeline")]
    Deploy,
}

#[derive(Subcommand, Debug)]
pub enum SentinelCommands {
    #[command(about = "Start CI/CD sentinel daemon")]
    Watch,

    #[command(about = "Start incident whisperer daemon")]
    ProdWatch,
}

#[derive(Subcommand, Debug)]
pub enum SkillCommands {
    #[command(about = "Install a skill from marketplace")]
    Install { name: String },

    #[command(about = "Publish a skill")]
    Publish { path: String },
}
