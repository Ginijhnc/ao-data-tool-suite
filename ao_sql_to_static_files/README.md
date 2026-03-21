# ao_sql_to_static_files

### What It Does

Queries PostgreSQL for game data and uploads them to CDN storage (Cloudflare R2). Rankings are exported with timestamps and assigned positions, ready to be served directly to game clients or web frontends.

### Key Design Decisions

**CDN-First Architecture**: Rankings update twice daily via cron and upload directly to CDN storage. This eliminates the need for a continuously-running API server, reducing resource consumption and operational costs. Hosting JSON files on a CDN instead of serving them through a live API server is inherently more resilient to DDoS attacks, which is a frequent issue in the Argentum Online community.

**Optional Filesystem Output**: By default, JSON files are NOT written to disk - they're uploaded directly to CDN. Use `--write-to-disk` flag for local testing/debugging only.

**GM Filtering**: Characters flagged as game masters are automatically excluded from public rankings.

**Flexible Output**: Rankings are exported as JSON with metadata (timestamp, rank positions) that can be consumed by any frontend without additional processing.

### How It Works

1. **Database Query**: Fetches top characters ranked by various metrics (level, PvP kills, etc.)
2. **GM Filtering**: Excludes characters where `is_gm = TRUE`
3. **Rank Assignment**: Assigns 1-indexed positions to each entry
4. **JSON Generation**: Wraps data with timestamp and writes pretty-printed JSON
5. **Directory Creation**: Automatically creates nested output directories as needed

### Requirements

- PostgreSQL with populated `characters` table
- Rust >= 1.91

### Configuration

Set via environment variables. Copy `.env.example` to `.env` and edit as needed.

**Important:** By default, `WRITE_TO_DISK=false` - rankings are only queried from the database. Set `WRITE_TO_DISK=true` for local testing/debugging to write JSON files to disk.

### Usage

All commands must be run from the `ao-tools` workspace root directory.

**Run (locally):**

```bash
# Default: query DB only (no filesystem output)
cargo run --release --package ao_sql_to_static_files

# With filesystem output for testing
cargo run --release --package ao_sql_to_static_files -- --write-to-disk

# With custom options
cargo run --release --package ao_sql_to_static_files -- --write-to-disk --ranking-limit 100 --static-json-output-dir ./rankings
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
