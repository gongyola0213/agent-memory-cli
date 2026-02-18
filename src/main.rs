mod commands;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-memory-cli")]
#[command(about = "Local-first memory CLI scaffold", long_about = None)]
struct Cli {
    /// SQLite database path
    #[arg(long, global = true, default_value = "data/agent-memory.db")]
    db: String,

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
    Create(UserCreateArgs),
    List,
    Show(UserShowArgs),
    Update(UserUpdateArgs),
}

#[derive(Args, Debug)]
struct UserCreateArgs {
    #[arg(long)]
    name: String,
}

#[derive(Args, Debug)]
struct UserShowArgs {
    #[arg(long)]
    uid: String,
}

#[derive(Args, Debug)]
struct UserUpdateArgs {
    #[arg(long)]
    uid: String,
    #[arg(long)]
    name: String,
}

#[derive(Subcommand, Debug)]
enum IdentityCommands {
    Link(IdentityLinkArgs),
    Resolve(IdentityResolveArgs),
    Unlink(IdentityResolveArgs),
}

#[derive(Args, Debug)]
struct IdentityLinkArgs {
    #[arg(long)]
    uid: String,
    #[arg(long)]
    channel: String,
    #[arg(long = "channel-user-id")]
    channel_user_id: String,
}

#[derive(Args, Debug)]
struct IdentityResolveArgs {
    #[arg(long)]
    channel: String,
    #[arg(long = "channel-user-id")]
    channel_user_id: String,
}

#[derive(Subcommand, Debug)]
enum ScopeCommands {
    Create(ScopeCreateArgs),
    AddMember(ScopeAddMemberArgs),
    List,
    Members(ScopeMembersArgs),
}

#[derive(Args, Debug)]
struct ScopeCreateArgs {
    #[arg(long = "id")]
    scope_id: String,
    #[arg(long = "type")]
    scope_type: String,
}

#[derive(Args, Debug)]
struct ScopeAddMemberArgs {
    #[arg(long = "id")]
    scope_id: String,
    #[arg(long)]
    uid: String,
    #[arg(long, default_value = "member")]
    role: String,
}

#[derive(Args, Debug)]
struct ScopeMembersArgs {
    #[arg(long = "id")]
    scope_id: String,
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
        Commands::User { command } => match command {
            UserCommands::Create(args) => {
                commands::todo("user", &format!("create name={}", args.name))
            }
            UserCommands::List => commands::todo("user", "list"),
            UserCommands::Show(args) => commands::todo("user", &format!("show uid={}", args.uid)),
            UserCommands::Update(args) => commands::todo(
                "user",
                &format!("update uid={} name={}", args.uid, args.name),
            ),
        },
        Commands::Identity { command } => match command {
            IdentityCommands::Link(args) => commands::todo(
                "identity",
                &format!(
                    "link uid={} channel={} channel_user_id={}",
                    args.uid, args.channel, args.channel_user_id
                ),
            ),
            IdentityCommands::Resolve(args) => commands::todo(
                "identity",
                &format!(
                    "resolve channel={} channel_user_id={}",
                    args.channel, args.channel_user_id
                ),
            ),
            IdentityCommands::Unlink(args) => commands::todo(
                "identity",
                &format!(
                    "unlink channel={} channel_user_id={}",
                    args.channel, args.channel_user_id
                ),
            ),
        },
        Commands::Scope { command } => match command {
            ScopeCommands::Create(args) => commands::todo(
                "scope",
                &format!("create id={} type={}", args.scope_id, args.scope_type),
            ),
            ScopeCommands::AddMember(args) => commands::todo(
                "scope",
                &format!(
                    "add-member id={} uid={} role={}",
                    args.scope_id, args.uid, args.role
                ),
            ),
            ScopeCommands::List => commands::todo("scope", "list"),
            ScopeCommands::Members(args) => {
                commands::todo("scope", &format!("members id={}", args.scope_id))
            }
        },
        Commands::Schema { command } => commands::todo("schema", &format!("{:?}", command)),
        Commands::Ingest { command } => commands::todo("ingest", &format!("{:?}", command)),
        Commands::Query { command } => commands::todo("query", &format!("{:?}", command)),
        Commands::State { command } => commands::todo("state", &format!("{:?}", command)),
        Commands::Admin { command } => match command {
            AdminCommands::Migrate => {
                if let Err(e) = commands::admin_migrate(&cli.db) {
                    eprintln!("{e}");
                    std::process::exit(1);
                }
            }
            AdminCommands::Reindex => commands::todo("admin", "reindex"),
            AdminCommands::Compact => commands::todo("admin", "compact"),
            AdminCommands::Archive => commands::todo("admin", "archive"),
        },
    }
}
