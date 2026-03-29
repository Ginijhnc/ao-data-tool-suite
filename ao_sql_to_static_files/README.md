# ao_sql_to_static_files

### What It Does

Queries PostgreSQL for game data and uploads them to CDN storage (Cloudflare R2). Game data exports are formatted with timestamps and indexed positions, ready to be served directly to game clients or web frontends.

### Key Design Decisions

**CDN-First Architecture**: Game data exports update twice daily via cron and upload directly to CDN storage. This eliminates the need for a continuously-running API server, reducing resource consumption and operational costs. Hosting JSON files on a CDN instead of serving them through a live API server is inherently more resilient to DDoS attacks, which is a frequent issue in the Argentum Online community.

**Manifest-Based Change Detection**: Uses SHA-256 hashing to detect content changes before uploading to CDN. Each export file is hashed (excluding timestamp) and compared against stored hashes in PostgreSQL. Only files with changed content are uploaded, reducing CDN costs and write operations. The manifest file itself is only updated when at least one data file changes.

**Cache Control Strategy**: Data files use 1-year immutable cache (`max-age=31536000, immutable`), manifest file uses 1-minute cache (`max-age=60`). Clients should check/fetch the short-cached manifest first, then fetch data files only if hashes differ. The immutable cache works because file hashes serve as content identifiers—changed data gets a new hash.

**Optional Filesystem Output**: By default, JSON files are NOT written to disk - they're uploaded directly to CDN. Use `--write-to-disk` flag for local testing/debugging only.

**GM Filtering**: Characters flagged as game masters are automatically excluded from public data exports. The flag is set at import time by the parser crate, which marks a character as GM if their name appears in the Server.ini file.

**Flexible Output**: Game data is exported as JSON with metadata (timestamp, indexed positions) that can be consumed by any frontend without additional processing.

### How It Works

1. **Database Query**: Fetches game data ordered by various metrics (character level, PvP kills, etc.)
2. **GM Filtering**: Excludes characters where `is_gm = TRUE`
3. **Position Assignment**: Assigns 1-indexed positions to each entry
4. **JSON Generation**: Wraps data with timestamp and writes pretty-printed JSON
5. **Directory Creation**: Automatically creates nested output directories as needed

### Requirements

- PostgreSQL with populated `characters` table
- Rust >= 1.91

### Configuration

Set via environment variables. Copy `.env.example` to `.env` and edit as needed.

**CDN Upload Mode (Default):**
- When `WRITE_TO_DISK=false` (default), game data is uploaded directly to Cloudflare R2
- Requires R2 credentials: `R2_ACCOUNT_ID`, `R2_ACCESS_KEY_ID`, `R2_SECRET_ACCESS_KEY`, `R2_BUCKET_NAME`, `R2_ENDPOINT`

**Disk Write Mode (Testing/Debugging):**
- When `WRITE_TO_DISK=true`, JSON files are written to local filesystem instead
- R2 credentials are not required in this mode
- Useful for local testing and debugging without CDN access

### Usage

All commands must be run from the `ao-tools` workspace root directory.

**Run (locally):**

```bash
# Default: query DB only (no filesystem output)
cargo run --release --package ao_sql_to_static_files

# With filesystem output for testing
cargo run --release --package ao_sql_to_static_files -- --write-to-disk

# With custom options
cargo run --release --package ao_sql_to_static_files -- --write-to-disk --ranking-limit 100 --static-json-output-dir ./exports
```

**Run (Docker):**

```bash
# Default: query DB only (no filesystem output)
docker compose up --build json-gen

# With filesystem output for testing
docker compose run --rm json-gen ao_sql_to_static_files --write-to-disk

# With custom options
docker compose run --rm json-gen ao_sql_to_static_files --write-to-disk --ranking-limit 100
```

The Docker setup automatically mounts the `exported-json` directory and connects to the host database when `--write-to-disk` is enabled.

### Testing

**Testing (in Docker container):**

Tests run inside a Docker container. The test container spawns a temporary PostgreSQL instance via testcontainers, runs the actual binary against it, and cleans up automatically.

```bash
docker compose up --build test                                                 # Run all workspace tests
docker compose run --rm test cargo test --package ao_sql_to_static_files               # Run using cached image
docker compose run --rm --build test cargo test --package ao_sql_to_static_files       # Rebuild and run (use after modifying test files)
docker compose run --rm test cargo test --package ao_sql_to_static_files <name>        # Run specific test by name
```

**Testing (locally):**

Requires Docker to be running (testcontainers spawns a temporary PostgreSQL instance).

```bash
cargo test --package ao_sql_to_static_files                  # Run all tests
cargo test --package ao_sql_to_static_files <name>           # Run specific test by name
cargo test --package ao_sql_to_static_files -- --nocapture   # Run with output visible
```