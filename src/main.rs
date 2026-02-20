mod commands;
mod db;
mod domain;
mod repository;
mod service;

use clap::{Args, Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "agent-memory-cli")]
#[command(about = "Local-first memory CLI scaffold", long_about = None)]
struct Cli {
    /// SQLite database path
    #[arg(long, global = true, default_value = "data/agent-memory.db")]
    db: String,

    /// Emit JSON output for supported commands
    #[arg(long, global = true, default_value_t = false)]
    json: bool,

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
    Merge(UserMergeArgs),
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

#[derive(Args, Debug)]
struct UserMergeArgs {
    #[arg(long = "from")]
    from_uid: String,
    #[arg(long = "to")]
    to_uid: String,
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
    Register(SchemaFileArgs),
    List,
    Validate(SchemaFileArgs),
}

#[derive(Args, Debug)]
struct SchemaFileArgs {
    #[arg(long = "file")]
    file: String,
}

#[derive(Subcommand, Debug)]
enum IngestCommands {
    Event(IngestEventArgs),
    Batch,
}

#[derive(Args, Debug)]
struct IngestEventArgs {
    #[arg(long)]
    uid: String,
    #[arg(long = "scope")]
    scope_id: String,
    #[arg(long = "type")]
    event_type: String,
    #[arg(long = "file")]
    file: String,
    #[arg(long = "idempotency-key")]
    idempotency_key: Option<String>,
}

#[derive(Subcommand, Debug)]
enum QueryCommands {
    Latest(QueryLatestArgs),
    Metric(QueryMetricArgs),
    Topk(QueryTopkArgs),
}

#[derive(Args, Debug)]
struct QueryLatestArgs {
    #[arg(long)]
    uid: String,
    #[arg(long = "scope")]
    scope_id: String,
}

#[derive(Args, Debug)]
struct QueryMetricArgs {
    #[arg(long)]
    uid: String,
    #[arg(long = "scope")]
    scope_id: String,
    #[arg(long = "key")]
    key: Option<String>,
    #[arg(long = "prefix")]
    prefix: Option<String>,
}

#[derive(Args, Debug)]
struct QueryTopkArgs {
    #[arg(long)]
    uid: String,
    #[arg(long = "scope")]
    scope_id: String,
    #[arg(long)]
    topic: String,
    #[arg(long, default_value_t = 3)]
    limit: usize,
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

    let result = match cli.command {
        Commands::Doctor => commands::doctor(&cli.db, cli.json),
        Commands::User { command } => match command {
            UserCommands::Create(args) => commands::user_create(&cli.db, &args.name),
            UserCommands::List => commands::user_list(&cli.db),
            UserCommands::Show(args) => commands::user_show(&cli.db, &args.uid),
            UserCommands::Update(args) => commands::user_update(&cli.db, &args.uid, &args.name),
            UserCommands::Merge(args) => {
                commands::user_merge(&cli.db, &args.from_uid, &args.to_uid)
            }
        },
        Commands::Identity { command } => match command {
            IdentityCommands::Link(args) => {
                commands::identity_link(&cli.db, &args.uid, &args.channel, &args.channel_user_id)
            }
            IdentityCommands::Resolve(args) => {
                commands::identity_resolve(&cli.db, &args.channel, &args.channel_user_id)
            }
            IdentityCommands::Unlink(args) => {
                commands::identity_unlink(&cli.db, &args.channel, &args.channel_user_id)
            }
        },
        Commands::Scope { command } => match command {
            ScopeCommands::Create(args) => {
                commands::scope_create(&cli.db, &args.scope_id, &args.scope_type)
            }
            ScopeCommands::AddMember(args) => {
                commands::scope_add_member(&cli.db, &args.scope_id, &args.uid, &args.role)
            }
            ScopeCommands::List => commands::scope_list(&cli.db),
            ScopeCommands::Members(args) => commands::scope_members(&cli.db, &args.scope_id),
        },
        Commands::Schema { command } => match command {
            SchemaCommands::Register(args) => commands::schema_register(&cli.db, &args.file),
            SchemaCommands::List => commands::schema_list(&cli.db),
            SchemaCommands::Validate(args) => commands::schema_validate(&args.file),
        },
        Commands::Ingest { command } => match command {
            IngestCommands::Event(args) => commands::ingest_event(
                &cli.db,
                &args.uid,
                &args.scope_id,
                &args.event_type,
                &args.file,
                args.idempotency_key.as_deref(),
            ),
            IngestCommands::Batch => {
                commands::todo("ingest", "batch");
                Ok(())
            }
        },
        Commands::Query { command } => match command {
            QueryCommands::Latest(args) => {
                commands::query_latest(&cli.db, &args.uid, &args.scope_id, cli.json)
            }
            QueryCommands::Metric(args) => commands::query_metric(
                &cli.db,
                &args.uid,
                &args.scope_id,
                args.key.as_deref(),
                args.prefix.as_deref(),
                cli.json,
            ),
            QueryCommands::Topk(args) => commands::query_topk(
                &cli.db,
                &args.uid,
                &args.scope_id,
                &args.topic,
                args.limit,
                cli.json,
            ),
        },
        Commands::State { command } => {
            commands::todo("state", &format!("{:?}", command));
            Ok(())
        }
        Commands::Admin { command } => match command {
            AdminCommands::Migrate => commands::admin_migrate(&cli.db),
            AdminCommands::Reindex => {
                commands::todo("admin", "reindex");
                Ok(())
            }
            AdminCommands::Compact => {
                commands::todo("admin", "compact");
                Ok(())
            }
            AdminCommands::Archive => {
                commands::todo("admin", "archive");
                Ok(())
            }
        },
    };

    if let Err(e) = result {
        eprintln!("{e}");
        std::process::exit(1);
    }
}
