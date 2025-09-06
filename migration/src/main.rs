use clap::{Parser, Subcommand};
use sea_orm::TransactionTrait;
use sea_orm::{ConnectOptions, Database, Statement};
use sea_orm_migration::prelude::*;
use url::Url;

// Use the Migrator defined in lib.rs
use migration::TableMigrator;
use migration::LocalSeed;
use migration::DevSeed;
use migration::StgSeed;
use migration::ProdSeed;

#[derive(Parser)]
#[command(name = "migration")]
#[command(about = "Database migration tool")]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Database schema (defaults to public)
    #[arg(short = 's', long = "schema", default_value = "public")]
    schema: String,

    /// Database URL (overrides environment variable)
    #[arg(
        short = 'u',
        long = "url",
        default_value = "postgresql://postgres:1234qwer@localhost:35432/aura"
    )]
    db_url: String,
}

#[derive(Subcommand)]
enum Commands {
    /// Run migrations up
    Up {
        /// Number of steps to migrate (default: all)
        #[arg(short, long)]
        steps: Option<u32>,
    },
    /// Run migrations down
    Down {
        /// Number of steps to rollback (default: 1)
        #[arg(short, long, default_value = "1")]
        steps: u32,
    },
    /// Get migration status
    Status,
    /// Refresh (down all, then up all)
    Fresh,
    /// Reset (down all)
    Reset,
    /// Run seeds for specific environment
    Seed {
        #[arg(short, long, default_value = "local")]
        env: String,
    }
}

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    println!("🐘 Database: {}", mask_password(&cli.db_url));
    println!("📋 Schema: {}", cli.schema);

    // Create database if it doesn't exist and get connection
    let db = create_database_if_not_exists(&cli.db_url, &cli.schema).await?;

    // Create schema if it doesn't exist
    create_schema_if_not_exists(&cli.db_url, &cli.schema).await?;

    execute_commands(&cli.command, &db).await?;

    Ok(())
}

/// Parse database URL and extract database name
fn extract_database_name(database_url: &str) -> Result<String, String> {
    let url = Url::parse(database_url).map_err(|e| format!("Invalid URL: {}", e))?;
    let path = url.path();
    if path.len() <= 1 {
        return Err("No database name found in URL".to_string());
    }
    Ok(path[1..].to_string()) // Remove leading '/'
}

/// Generate connection string without database name (for connecting to postgres default database)
fn get_admin_url(database_url: &str) -> Result<String, String> {
    let mut url = Url::parse(database_url).map_err(|e| format!("Invalid URL: {}", e))?;
    url.set_path("/postgres"); // Connect to default postgres database
    Ok(url.to_string())
}

/// Create database if it doesn't exist and return database connection
async fn create_database_if_not_exists(
    database_url: &str,
    schema: &str,
) -> Result<sea_orm::DatabaseConnection, Box<dyn std::error::Error>> {
    // Create connection options with schema search path
    let mut connect_options = ConnectOptions::new(database_url);
    if schema != "public" {
        connect_options.set_schema_search_path(schema);
    }

    // Try to connect to database first
    match Database::connect(connect_options.clone()).await {
        Ok(connection) => {
            println!("✅ Connected to database");
            return Ok(connection);
        }
        Err(e) => {
            if !e.to_string().contains("does not exist") {
                return Err(e.into());
            }
            // Database doesn't exist, continue to create it
            println!("⚠️  Database does not exist. Creating database...");
        }
    }

    let database_name = extract_database_name(database_url)?;
    let admin_url = get_admin_url(database_url)?;

    println!("🔍 Checking if database '{}' exists...", database_name);

    // Connect to postgres default database
    let admin_db = Database::connect(&admin_url).await?;

    // Check if database exists using a different approach
    let query = format!(
        "SELECT datname FROM pg_database WHERE datname = '{}'",
        database_name
    );

    let result = admin_db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            query,
        ))
        .await?;

    if result.is_empty() {
        println!("🏗️  Creating database '{}'...", database_name);
        let create_query = format!("CREATE DATABASE \"{}\"", database_name);
        admin_db
            .execute(Statement::from_string(
                sea_orm::DatabaseBackend::Postgres,
                create_query,
            ))
            .await?;
        println!("✅ Database '{}' created successfully!", database_name);
    } else {
        println!("✅ Database '{}' already exists", database_name);
    }

    admin_db.close().await?;

    // Connect to the target database with schema search path
    println!("🔄 Connecting to database...");
    let db = Database::connect(connect_options).await?;
    println!("✅ Connected to database successfully!");

    Ok(db)
}

/// Create schema if it doesn't exist
async fn create_schema_if_not_exists(
    database_url: &str,
    schema_name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if schema_name == "public" {
        // public schema always exists, no need to create
        return Ok(());
    }

    println!("🔍 Checking if schema '{}' exists...", schema_name);

    // Connect to the target database
    let db = Database::connect(database_url).await?;

    // Check if schema exists
    let query = format!(
        "SELECT schema_name FROM information_schema.schemata WHERE schema_name = '{}'",
        schema_name
    );

    let result = db
        .query_all(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            query,
        ))
        .await?;

    if result.is_empty() {
        println!("🏗️  Creating schema '{}'...", schema_name);
        let create_query = format!("CREATE SCHEMA \"{}\"", schema_name);
        db.execute(Statement::from_string(
            sea_orm::DatabaseBackend::Postgres,
            create_query,
        ))
        .await?;
        println!("✅ Schema '{}' created successfully!", schema_name);
    } else {
        println!("✅ Schema '{}' already exists", schema_name);
    }

    db.close().await?;
    Ok(())
}

/// Function to mask password in URL for display
fn mask_password(url: &str) -> String {
    if let Ok(mut parsed_url) = Url::parse(url) {
        if parsed_url.password().is_some() {
            let _ = parsed_url.set_password(Some("***"));
        }
        parsed_url.to_string()
    } else {
        url.to_string()
    }
}

/// Execute migration commands based on user input
async fn execute_commands(
    command: &Commands,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    match command {
        Commands::Up { steps } => {
            if let Some(step_count) = steps {
                println!("⬆️  Running {} migration(s) up...", step_count);
                TableMigrator::up(db, Some(*step_count)).await?;
            } else {
                println!("⬆️  Running all pending migrations up...");
                TableMigrator::up(db, None).await?;
            }
        }
        Commands::Down { steps } => {
            println!("⬇️  Rolling back {} migration(s)...", steps);
            TableMigrator::down(db, Some(*steps)).await?;
        }
        Commands::Status => {
            println!("📊 Migration status...");
            TableMigrator::status(db).await?;
        }
        Commands::Fresh => {
            println!("🧹 Fresh database (drop all tables + up all)...");
            TableMigrator::fresh(db).await?;
        }
        Commands::Reset => {
            println!("🔄 Resetting database (down all)...");
            TableMigrator::reset(db).await?;
        }
        Commands::Seed { env} => {
            println!("🌱 Seed the initial data...");
            execute_seed_commands(env, db).await?;
        }
    }
    Ok(())
}

/// Execute seed commands for the specified environment
async fn execute_seed_commands(
    env: &str,
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    match env {
        "local" => {
            println!("🌱 Executing local seeds...");
            execute_seeds_without_tracking::<LocalSeed>(db).await?;
        }
        "dev" => {
            println!("🌱 Executing dev seeds...");
            execute_seeds_without_tracking::<DevSeed>(db).await?;
        }
        "stg" => {
            println!("🌱 Executing stg seeds...");
            execute_seeds_without_tracking::<StgSeed>(db).await?;
        }
        "prod" => {
            println!("🌱 Executing prod seeds...");
            execute_seeds_without_tracking::<ProdSeed>(db).await?;
        }
        _ => {
            return Err(format!("Unknown environment: {}", env).into());
        }
    }
    
    println!("✅ Seeds completed successfully!");
    Ok(())
}

/// Execute all seeds using transaction without migration tracking
async fn execute_seeds_without_tracking<S: MigratorTrait>(
    db: &sea_orm::DatabaseConnection,
) -> Result<(), Box<dyn std::error::Error>> {
    let migrations = S::migrations();

    // Begin transaction
    let txn = db.begin().await?;
    let schema_manager = SchemaManager::new(&txn);

    // Execute all seed migrations
    for migration in migrations {
        let migration_name = migration.name();
        if migration_name.contains("seed") {
            println!("🌱 Executing seed: {}", migration_name);
            migration.up(&schema_manager).await?;
            println!("✅ Completed seed: {}", migration_name);
        }
    }
    // Commit transaction
    txn.commit().await?;
    Ok(())
}