# ao_data_to_sql

### What It Does

Scans a directory for game data files, parses them in parallel, and inserts the data into a PostgreSQL database. Handles encoding issues, strips sensitive fields, and works with any fork's custom fields via JSONB storage.

### Key Design Decisions

**JSONB Storage**: Character data is stored as JSONB in PostgreSQL instead of rigid table schemas. This handles the "fork problem" - hundreds of AO forks have modified the data structure by adding custom fields over the years. Non-technical users can run this tool without modifying type definitions.

**Encoding Detection**: Legacy VB6 servers use Windows-1252 encoding for Spanish characters (ñ, á, etc.). The parser tries UTF-8 first, then falls back gracefully.

**Privacy by Default**: Sensitive fields are filtered at parse time, never touching the database.

**Performance**: Multithreaded parsing and batch database operations handle tens of thousands of files efficiently.

### How It Works

1. **Database Migration**: Runs pending migrations automatically via SQLx
2. **File Discovery**: Recursively finds all `.chr` and `.dat` files (ignores `.chr.bk` backups)
3. **Parallel Parsing**: Uses Rayon for multithreaded parsing across CPU cores
4. **Encoding Detection**: Tries UTF-8, falls back to Windows-1252 for legacy VB6 servers
5. **Privacy Filtering**: Removes EMAIL, PASSWORD, PASSWORDHASH, PASSWORDSALT, LASTIP1-5, and the entire CONTACTO section
6. **Batch Insert**: Groups records and uses upsert (INSERT ... ON CONFLICT) for efficiency

### Requirements

- Docker
- PostgreSQL

### Configuration

Set via environment variables. Copy `.env.example` to `.env` and edit as needed.

### Usage

All commands must be run from the `ao-data-tool-suite` workspace root directory.

**1. Create the database (only once):**

```bash
createdb -U postgres ao_server_data
```

**2. Run (with Docker):**

```bash
docker compose up --build parser
```

**2b. Run (without Docker):**

Requires Rust >= 1.91

```bash
cargo run --release --package ao_data_to_sql
```

Or with custom options:

```bash
cargo run --release --package ao_data_to_sql -- --charfile-dir ./path/to/Charfile --batch-size 500
```

**3. Rollback migrations (optional):**

```bash
docker compose run --rm parser --rollback                      # Rollback all migrations
docker compose run --rm parser --rollback --rollback-target 1  # Rollback to version 1
```

### Testing

**4a. Testing (in Docker container):**

Tests run inside a Docker container. The test container spawns a temporary PostgreSQL instance via testcontainers, runs the actual binary against it, and cleans up automatically.

```bash
docker compose up --build test                                                 # Run all workspace tests
docker compose run --rm test cargo test --package ao_data_to_sql               # Run using cached image
docker compose run --rm --build test cargo test --package ao_data_to_sql       # Rebuild and run (use after modifying test files)
docker compose run --rm test cargo test --package ao_data_to_sql <name>        # Run specific test by name
```

**4b. Testing (locally):**

Requires Docker to be running (testcontainers spawns a temporary PostgreSQL instance).

```bash
cargo test --package ao_data_to_sql                  # Run all tests
cargo test --package ao_data_to_sql <name>           # Run specific test by name
cargo test --package ao_data_to_sql -- --nocapture   # Run with output visible
```