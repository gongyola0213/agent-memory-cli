mod commands;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-memory-cli")]
#[command(about = "Local-first memory CLI scaffold", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Validate project setup
    Doctor,
    /// Manage canonical users
    User {
        #[command(subcommand)]
        command: UserCommands,
    },
    /// Manage channel identities linked to users
    Identity {
        #[command(subcommand)]
        command: IdentityCommands,
    },
    /// Manage memory scopes and members
    Scope {
        #[command(subcommand)]
        command: ScopeCommands,
    },
    /// Register and validate dynamic schemas
    Schema {
        #[command(subcommand)]
        command: SchemaCommands,
    },
    /// Ingest time-series events
    Ingest {
        #[command(subcommand)]
        command: IngestCommands,
    },
    /// Query materialized state/metrics
    Query {
        #[command(subcommand)]
        command: QueryCommands,
    },
    /// Direct state CRUD
    State {
        #[command(subcommand)]
        command: StateCommands,
    },
    /// Operational and maintenance commands
    Admin {
        #[command(subcommand)]
        command: AdminCommands,
    },
}

#[derive(Subcommand, Debug)]
enum UserCommands {
    Create,
    List,
    Show,
    Update,
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    Link,
    Resolve,
    Unlink,
}

#[derive(Subcommand, Debug)]
enum ScopeCommands {
    Create,
    AddMember,
    List,
    Members,
}

#[derive(Subcommand, Debug)]
enum SchemaCommands {
    Register,
    List,
    Validate,
}

#[derive(Subcommand, Debug)]
enum IngestCommands {
    Event,
    Batch,
}

#[derive(Subcommand, Debug)]
enum QueryCommands {
    Latest,
    Metric,
    Topk,
}

#[derive(Subcommand, Debug)]
enum StateCommands {
    Get,
    Set,
    Delete,
}

#[derive(Subcommand, Debug)]
enum AdminCommands {
    Migrate,
    Reindex,
    Compact,
    Archive,
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Doctor => commands::doctor(),
        Commands::User { command } => commands::todo("user", &format!("{:?}", command)),
        Commands::Identity { command } => commands::todo("identity", &format!("{:?}", command)),
        Commands::Scope { command } => commands::todo("scope", &format!("{:?}", command)),
        Commands::Schema { command } => commands::todo("schema", &format!("{:?}", command)),
        Commands::Ingest { command } => commands::todo("ingest", &format!("{:?}", command)),
        Commands::Query { command } => commands::todo("query", &format!("{:?}", command)),
        Commands::State { command } => commands::todo("state", &format!("{:?}", command)),
        Commands::Admin { command } => commands::todo("admin", &format!("{:?}", command)),
    }
}
