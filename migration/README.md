# Aura RS Migration CLI

A powerful database migration tool for the Aura RS project with automatic database and schema creation capabilities.

## Features

- 🗄️ **Auto Database Creation**: Automatically creates PostgreSQL databases if they don't exist
- 🏗️ **Schema Management**: Creates and manages custom database schemas
- 🔄 **Migration Control**: Run, rollback, and track migration status
- 🌱 **Environment Seeds**: Support for environment-specific data seeding
- 🔧 **Smart Connection**: Uses SeaORM ConnectOptions for optimal schema handling

## Quick Start

```bash
# Run all pending migrations
cargo run --bin migration -- up

# Run migrations in a custom schema
cargo run --bin migration -- -s my_schema up

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
  up        Run migrations up
  down      Rollback migrations  
  status    Get migration status
  fresh     Refresh (down all, then up all)
  reset     Reset (down all)
  seed      Set the default data for given environment
  help      Print this message or the help of the given subcommand(s)
```

## Examples

### Basic Migration Commands

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
```

### Schema Management

```bash
# Create and use a custom schema
cargo run --bin migration -- -s development up

# Work with staging schema
cargo run --bin migration -- -s staging status

# Production schema operations
cargo run --bin migration -- -s production fresh
```

### Database Operations

```bash
# Fresh database (drop all tables + recreate)
cargo run --bin migration -- fresh

# Reset database (rollback all migrations)
cargo run --bin migration -- reset

# Different database connection
cargo run --bin migration -- -u postgresql://user:pass@host:5432/mydb up
```

### Data Seeding

```bash
# Seed local environment data
cargo run --bin migration -- seed --env local

# Seed development environment data
cargo run --bin migration -- seed --env dev

# Seed staging environment data
cargo run --bin migration -- seed --env stg

# Seed production environment data
cargo run --bin migration -- seed --env prod
```

## Automatic Features

### Database Auto-Creation
The CLI automatically detects if the target database doesn't exist and creates it:

```bash
# This will create 'myapp_db' if it doesn't exist
cargo run --bin migration -- -u postgresql://postgres:1234qwer@localhost:35432/aura up
```

### Schema Auto-Creation
Custom schemas are automatically created when specified:

```bash
# This will create 'analytics' schema if it doesn't exist
cargo run --bin migration -- -s analytics up
```

## Default Configuration

- **Database URL**: `postgresql://postgres:1234qwer@localhost:35432/aura`
- **Schema**: `public`
- **Environment**: `local` (for seeding)

## Migration Files

Migration files are located in `src/common/` directory and follow the naming convention:
`m{timestamp}_{description}.rs`

### Current Migrations

- `m20250905_113743_create_turbo_togel_draw_shedule_table.rs` - Creates the main draw schedule table

### Adding New Migrations

1. Use sea-orm-cli to generate migration files:
   ```bash
   sea-orm-cli migrate generate create_new_table
   ```

2. Move the generated file to `src/common/`

3. Add the migration to `src/lib.rs`:
   ```rust
   impl MigratorTrait for Migrator {
       fn migrations() -> Vec<Box<dyn MigrationTrait>> {
           vec![
               Box::new(common::m20250905_113743_create_turbo_togel_draw_shedule_table::Migration),
               Box::new(common::m20250906_000000_create_new_table::Migration), // Add here
           ]
       }
   }
   ```

## Technical Details

### Connection Management

- Uses SeaORM's `ConnectOptions` with `set_schema_search_path()` for optimal schema handling
- Automatically falls back to creating databases when connection fails
- Proper connection pooling and cleanup

### Schema Search Path

When using custom schemas, the CLI sets the PostgreSQL search path to:

```sql
SET search_path TO "custom_schema", public
```

This ensures that:

- All migrations run in the specified schema
- The `seaql_migrations` table is created in the correct schema
- Fallback to `public` schema for system functions

## Environment Structure

```text
migration/
├── src/
│   ├── common/          # Shared migrations
│   │   ├── mod.rs
│   │   └── m*.rs        # Migration files
│   ├── dev/             # Development seeds
│   ├── stg/             # Staging seeds
│   ├── prod/            # Production seeds
│   ├── lib.rs           # Migration registration
│   └── main.rs          # CLI implementation
└── README.md
```
