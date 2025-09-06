# Aura RS Migration CLI

A powerful database migration tool for the Aura RS project with automatic database and schema creation capabilities and environment-specific data seeding.

## Features

- 🗄️ **Auto Database Creation**: Automatically creates PostgreSQL databases if they don't exist
- 🏗️ **Schema Management**: Creates and manages custom database schemas
- 🔄 **Migration Control**: Run, rollback, and track migration status
- 🌱 **Environment-based Seeds**: Separate migration and seed commands for better control
- 🔧 **Smart Connection**: Uses SeaORM ConnectOptions for optimal schema handling
- 📊 **Status Tracking**: Monitor migration status and applied changes

## Quick Start

```bash
# Run all pending migrations (table structure only)
cargo run --bin migration -- up

# Run migrations in a custom schema
cargo run --bin migration -- -s my_schema up

# Seed local environment data (after migrations)
cargo run --bin migration -- seed --env local

# Check migration status
cargo run --bin migration -- status
```

## Usage

```bash
migration [OPTIONS] <COMMAND>

Options:
  -s, --schema <SCHEMA>      Database schema [default: public]
  -u, --url <DATABASE_URL>   Database URL [default: postgresql://postgres:1234qwer@localhost:35432/aura]
  -h, --help                 Print help
  -V, --version             Print version

Commands:
  up        Run migrations up (table structure only)
  down      Rollback migrations  
  status    Get migration status
  fresh     Refresh (down all, then up all)
  reset     Reset (down all)
  seed      Run seeds for specific environment (data only)
  help      Print this message or the help of the given subcommand(s)
```

## Command Details

### Migration Commands (Structure Only)

These commands only handle database schema and table structure using `TableMigrator`:

```bash
# Apply all pending migrations
cargo run --bin migration -- up

# Apply specific number of migrations
cargo run --bin migration -- up --steps 3

# Rollback last migration
cargo run --bin migration -- down

# Rollback specific number of migrations
cargo run --bin migration -- down --steps 2

# Check migration status
cargo run --bin migration -- status

# Fresh database (drop all tables + recreate)
cargo run --bin migration -- fresh

# Reset database (rollback all migrations)
cargo run --bin migration -- reset
```

### Seed Commands (Data Only)

These commands handle environment-specific data seeding with complete migration sets:

```bash
# Seed local environment data (includes base migrations + local seeds)
cargo run --bin migration -- seed --env local

# Seed development environment data (includes common seeds)
cargo run --bin migration -- seed --env dev

# Seed staging environment data (includes common seeds)
cargo run --bin migration -- seed --env stg

# Seed production environment data (includes common seeds)
cargo run --bin migration -- seed --env prod
```

## Recommended Workflow

### 1. Development Setup
```bash
# First, create database structure
cargo run --bin migration -- up

# Then, seed local development data
cargo run --bin migration -- seed --env local
```

### 2. Different Environments
```bash
# For development environment
cargo run --bin migration -- up
cargo run --bin migration -- seed --env dev

# For staging environment
cargo run --bin migration -- -s staging up
cargo run --bin migration -- -s staging seed --env stg

# For production environment
cargo run --bin migration -- -s production up
cargo run --bin migration -- -s production seed --env prod
```

### 3. Schema Management
```bash
# Create and use a custom schema
cargo run --bin migration -- -s development up
cargo run --bin migration -- -s development seed --env local

# Different database connection
cargo run --bin migration -- -u postgresql://user:pass@host:5432/mydb up
```

## Migration Structure

### Current Tables

**`turbo_togel_draw_shedule`** - Game schedule configuration
- `id` (bigint, primary key) - Unique game identifier
- `cron` (varchar) - Cron expression for scheduling
- `location` (varchar) - Timezone location (e.g., Asia/Jakarta)
- `first_draw` (varchar) - First draw time (HH:MM:SS format)
- `interval` (smallint) - Seconds between draws
- `close` (smallint) - Betting close time before draw
- `min`/`max` (smallint) - Number range for the game
- `count` (smallint) - Number of digits in result
- `repeatable` (boolean) - Whether numbers can repeat
- `status` (smallint) - Game status (0: inactive, 1: active)

**`turbo_togel_draw_result`** - Game draw results
- `game_id` (bigint) - References schedule game
- `period` (bigint) - Draw period number
- `numbers` (varchar) - Drawn numbers
- `remark` (varchar) - Optional remark for manual draws
- `created_at` / `drawing_at` / `updated_at` - Timestamps
- `updated_by` (varchar) - User who updated
- `is_broadcasted` (boolean) - Broadcast status

### Migration Architecture

The migration system is organized into clear separation of concerns:

**TableMigrator** - Base structure only
- Contains DDL migrations for table creation
- Used by `up`, `down`, `status`, `fresh`, `reset` commands
- Pure schema management without data

**Environment Seeds** - Complete migration sets including data
- **LocalSeed**: TableMigrator + common seeds (for local development)
- **DevSeed**: Common seeds only (for development environment)
- **StgSeed**: Common seeds only (for staging environment)  
- **ProdSeed**: Common seeds only (for production environment)

### Seed Strategy

Each environment has different data seeding strategies:

- **Local**: Full base migrations + test data for local development
- **Dev**: Common seed data for development environment
- **Stg**: Common seed data for staging environment
- **Prod**: Common seed data for production environment

## File Structure

```text
migration/
├── src/
│   ├── ddl/                       # Database schema migrations
│   │   ├── mod.rs
│   │   ├── m20250905_113743_create_turbo_togel_draw_shedule_table.rs
│   │   └── m20250906_063502_create_turbo_togel_draw_result_table.rs
│   ├── seed/                      # Data seeding migrations
│   │   ├── mod.rs                 # Export functions for each environment
│   │   ├── common/                # Shared seed data
│   │   │   ├── mod.rs
│   │   │   └── m20250905_164736_seed_draw_shedule.rs
│   │   ├── dev/                   # Development-specific seeds
│   │   │   └── mod.rs
│   │   ├── local/                 # Local development seeds  
│   │   │   └── mod.rs
│   │   ├── stg/                   # Staging-specific seeds
│   │   │   └── mod.rs
│   │   └── prod/                  # Production-specific seeds
│   │       └── mod.rs
│   ├── lib.rs                     # Migration registration and environment setup
│   └── main.rs                    # CLI implementation
├── Cargo.toml
└── README.md
```

## Adding New Migrations

### 1. Generate DDL Migration File
```bash
# Create new table migration
sea-orm-cli migrate generate -d migration/src/ddl create_new_table
```

### 2. Register DDL Migration
Add to `src/lib.rs` in `TableMigrator`:
```rust
#[async_trait::async_trait]
impl MigratorTrait for TableMigrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(ddl::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration),
            Box::new(ddl::m20250906_063502_create_turbo_togel_draw_result_table::Migration),
            Box::new(ddl::m20250906_100000_create_new_table::Migration), // Add here
        ]
    }
}
```

### 3. Register in DDL mod.rs
Add to `src/ddl/mod.rs`:
```rust
pub mod m20250905_113743_create_turbo_togel_draw_shedule_table;
pub mod m20250906_063502_create_turbo_togel_draw_result_table;
pub mod m20250906_100000_create_new_table; // Add here
```

## Adding New Seeds

### 1. Create Seed File
```bash
# Example: Create common seed
touch migration/src/seed/common/m20250907_000000_seed_new_data.rs
```

### 2. Implement Seed Migration
```rust
use common::entity::your_table;
use sea_orm_migration::prelude::*;
use sea_orm_migration::sea_orm::{entity::*, query::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        let db = manager.get_connection();
        let transaction = db.begin().await?;

        your_table::ActiveModel {
            id: Set(1),
            name: Set("example".to_string()),
            // Your seed data here
        }
        .insert(&transaction)
        .await?;

        transaction.commit().await?;
        Ok(())
    }

    async fn down(&self, _manager: &SchemaManager) -> Result<(), DbErr> {
        Ok(())
    }
}
```

### 3. Register Seed
Add to `src/seed/common/mod.rs`:
```rust
pub mod m20250905_164736_seed_draw_shedule;
pub mod m20250907_000000_seed_new_data; // Add here
```

Update `src/seed/mod.rs`:
```rust
fn common_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    vec![
        Box::new(common::m20250905_164736_seed_draw_shedule::Migration),
        Box::new(common::m20250907_000000_seed_new_data::Migration), // Add here
    ]
}
```

### 4. Environment-specific Seeds

For environment-specific data, create files in the respective directories:

```bash
# Local-specific seed
touch migration/src/seed/local/m20250907_100000_local_test_data.rs

# Dev-specific seed  
touch migration/src/seed/dev/m20250907_200000_dev_config.rs
```

Then update the corresponding functions in `src/seed/mod.rs`:

```rust
pub fn local_seed_migrations() -> Vec<Box<dyn MigrationTrait>> {
    let mut migrations = common_seed_migrations();
    
    // Add local-specific seed migrations
    migrations.push(Box::new(local::m20250907_100000_local_test_data::Migration));
    
    migrations
}
```

## Technical Details

### Automatic Features

**Database Auto-Creation**: Detects missing databases and creates them automatically
**Schema Auto-Creation**: Creates custom schemas when specified
**Smart Connection**: Uses SeaORM ConnectOptions with schema search path
**Transaction Safety**: All seeds use transactions for data integrity

### Migration Execution Order

1. **DDL Migrations** (via `TableMigrator`): 
   - Execute in timestamp order
   - Create table structures only

2. **Seed Migrations** (via environment-specific migrators):
   - Include base DDL migrations first  
   - Then add environment-specific seed data
   - Maintain proper dependencies

### Schema Search Path
When using custom schemas:
```sql
SET search_path TO "custom_schema", public
```

### Default Configuration
- **Database URL**: `postgresql://postgres:1234qwer@localhost:35432/aura`
- **Schema**: `public`
- **Default Environment**: `local`

## Sea ORM CLI Integration

Generate new migrations:
```bash
# For DDL changes
sea-orm-cli migrate generate -d migration/src/ddl create_new_table

# For seed data, create manually in appropriate seed directory
```

Generate entities (after running migrations):
```bash
sea-orm-cli generate entity -u postgresql://postgres:1234qwer@localhost:35432/aura -o ./common/src/entity
```

## Examples

### Complete Setup Flow
```bash
# 1. Setup database structure
cargo run --bin migration -- up

# 2. Seed development data  
cargo run --bin migration -- seed --env local

# 3. Check status
cargo run --bin migration -- status

# 4. For production (different environment)
cargo run --bin migration -- -u postgresql://prod_user:pass@prod_host:5432/aura_prod up
cargo run --bin migration -- -u postgresql://prod_user:pass@prod_host:5432/aura_prod seed --env prod
```

### Development Workflow
```bash
# Daily development - structure changes only
cargo run --bin migration -- up                    # Apply new DDL migrations

# Reset and rebuild with fresh data
cargo run --bin migration -- fresh                 # Drop all + recreate structure
cargo run --bin migration -- seed --env local      # Re-seed with test data

# Environment-specific deployment
cargo run --bin migration -- -s staging up         # Deploy structure to staging
cargo run --bin migration -- -s staging seed --env stg  # Seed staging data
```

### Debugging and Status
```bash
# Check current migration status
cargo run --bin migration -- status

# Check specific schema status
cargo run --bin migration -- -s development status

# Rollback recent changes
cargo run --bin migration -- down --steps 1
```

## Current Seed Data

### Default Game Configuration
The common seed includes a default turbo togel game configuration:

- **Game ID**: 1
- **Schedule**: Every 5 minutes (0,5,10,15,20,25,30,35,40,45,50,55 * * * ?)
- **Timezone**: Asia/Jakarta
- **First Draw**: 00:00:00
- **Interval**: 300 seconds (5 minutes)
- **Close Time**: 30 seconds before draw
- **Number Range**: 0-9
- **Digit Count**: 4 digits
- **Repeatable**: Yes
- **Status**: Active (1)

This seed data is included in all environments through the common seed migration system.