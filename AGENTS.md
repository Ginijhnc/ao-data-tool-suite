# AGENTS.md

## Coding Style & Naming Conventions

- **Functions**: `snake_case`
- **Variables**: `snake_case`
- **Constants**: `SCREAMING_SNAKE_CASE`
- **Structs/Types**: `PascalCase`
- **Enums**: `PascalCase` for enum name, `PascalCase` for variants (e.g., `ConfigError`, `ParseError`)
- **Type Aliases**: `PascalCase`
- **Modules**: `snake_case` (e.g., `error`, `config`, `api`)
- **Crate Names**: `snake_case` with hyphens in Cargo.toml (e.g., `my-app`)

### Code Organization

- Group imports in three sections separated by blank lines: stdlib, external crates, internal modules
- Use `pub` visibility only when necessary; keep implementation details private
- Organize modules by feature or domain (e.g., `auth`, `api`, `db`, `config`)

## Error Handling

- Use `anyhow` for application-level error handling
- Import `anyhow::Result` and use it as the return type: `use anyhow::Result;`
- Use `.context("description")` to add context to errors
- Use `anyhow::bail!("message")` for early returns with custom errors
- For domain logic, use `thiserror` for typed errors, then convert to `anyhow` at boundaries

```rust
use anyhow::{Context, Result};

fn load_config(path: &str) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read configuration file")?;
    parse_config(&content).context("Failed to parse configuration")
}
```

### Domain Errors with thiserror

```rust
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("configuration file not found at {0}")]
    NotFound(String),
    #[error("invalid configuration: {0}")]
    Invalid(String),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}
```

### Result Unwrapping

- Always prefer the `?` operator for error propagation
- Use `.expect("descriptive message")` only in tests
- Avoid `.unwrap()` in production code

### Type Aliases

- Always create a `Result<T>` type alias for custom error types: `pub type Result<T> = std::result::Result<T, YourError>;`
- Use type aliases for frequently-used complex types (e.g., `Arc<Mutex<HashMap<String, Value>>>`)
- Avoid over-aliasing simple types

```rust
// Always do this for Result
pub type Result<T> = std::result::Result<T, ConfigError>;

// Good for complex types
pub type ConnectionPool = Arc<Mutex<Vec<Connection>>>;

// Don't alias simple types
// Bad: pub type UserId = u64;
// Good: just use u64 directly
```

Complex types repeated in 2+ locations are refactoring candidates. Suggest a type alias to reduce duplication, but do not implement without asking first.

## Async Runtime

- Use `tokio` as the async runtime
- In `Cargo.toml`: `tokio = { version = "1", features = ["full"] }`
- Use `#[tokio::main]` for the main function
- Use `#[tokio::test]` for async tests

## Dependencies

- Avoid version ranges like `"0.4"` or `"^0.4"` - be explicit about the exact version
- Prefer pinning dependencies to the latest specific version (e.g., `chrono = "0.4.43"`)
- Check the latest version on [crates.io](https://crates.io/) before adding a dependency

### Workspace Dependencies

All dependencies must be defined in the root `Cargo.toml` under `[workspace.dependencies]`, then referenced from crate-level `Cargo.toml` files using `.workspace = true`:

```toml
# Root Cargo.toml
[workspace.dependencies]
serde = { version = "1.0.228", features = ["derive"] }

# Crate Cargo.toml
[dependencies]
serde.workspace = true
```

This ensures consistent versioning across all crates and provides a single source of truth for dependency versions.

## String Parameters

- Use `&str` for function parameters that only need to read the string
- Use `String` only when the function needs to own or modify the string

## Trait Bounds

- Use inline syntax for simple bounds: `fn foo<T: Trait>(item: T)`
- Use `where` clause for complex bounds

```rust
// Simple
fn process<T: Serialize>(item: T) -> Result<String> {
    serde_json::to_string(&item)
}

// Complex
fn complex<T, U>(item: T, other: U) -> Result<()>
where
    T: Serialize + DeserializeOwned + Clone,
    U: Display + Debug,
{
    // Implementation
}
```

## Logging and Tracing

- Use `tracing` for all logging (avoid `println!` in production code)
- Use appropriate log levels: `info!`, `warn!`, `error!`, `debug!`, `trace!`
- Use `#[instrument]` for automatic span creation
- Configure log level via `RUST_LOG` environment variable

## Dependency Injection

Use constructor injection with `Arc` for shared state:

```rust
use std::sync::Arc;

pub struct Service {
    db: Arc<Database>,
    cache: Arc<Cache>,
}

impl Service {
    pub fn new(db: Arc<Database>, cache: Arc<Cache>) -> Self {
        Self { db, cache }
    }
}
```

## Documentation & Comments

- Spanish is used for user-facing messages (logs, CLI descriptions)
- Code identifiers remain in English
- No emojis anywhere
- Code comments (`///` and `//!` for rustdoc, `//` for inline) must be in English

### Rustdoc Guidelines

Use `//!` for module/crate docs and `///` for item docs (functions, structs, enums).

### Module-Level Docs (`//!`)

- Every `.rs` file must have module-level documentation at the top
- 3-5 lines: first line describes what the module does, following lines add context

### Item-Level Docs (`///`)

- All public items (`pub fn`, `pub struct`, `pub enum`, etc.) must be documented
- Maximum 2-3 lines per item; keep it brief

## Commit Guidelines

- Use **Conventional Commits** with a **mandatory scope**: `feat(scope):`, `fix(scope):`, `refactor(scope):`, `test(scope):`, `docs(scope):`, `chore(scope):`
- Always include a commit body describing **what the change does and why it exists**, not how it is implemented
- Do not reference function/file names or hard-coded values; keep the body implementation-agnostic and future-proof

### Scopes

| Scope      | Crate                  | Description                                |
| ---------- | ---------------------- | ------------------------------------------ |
| `parser`   | ao_data_to_sql         | CHR/DAT file parsing and PostgreSQL import |
| `json-gen` | ao_sql_to_static_files | SQL queries to JSON file export            |
| `shared`   | ao_shared              | Shared utilities (DB connection, etc.)     |
| `docker`   | -                      | Dockerfile and container configuration     |
| `deps`     | -                      | Dependency updates                         |
| `ci`       | -                      | CI/CD pipelines                            |
| `tooling`  | -                      | Linting, formatting, workspace config      |

## Development Commands

### Building

```bash
cargo build --workspace                # Build all crates (debug)
cargo build -p ao_data_to_sql          # Build a specific crate
cargo build --workspace -v             # Build with verbose output
cargo check --workspace                # Type-check without producing binaries (faster)
```

### Code Formatting

```bash
cargo fmt --all                        # Format all crates
cargo fmt --all --check                # Check formatting without modifying files
cargo fmt -p ao_data_to_sql            # Format a specific crate
```

### Linting

```bash
cargo clippy --workspace --all-targets -- -D warnings  # Run clippy, fail on warnings
cargo clippy -p ao_data_to_sql --all-targets -- -D warnings  # Lint a specific crate, fail on warnings
```

### Testing

```bash
cargo test --workspace                 # Run all tests
cargo test -p ao_data_to_sql           # Run tests for a specific crate
cargo test -p ao_data_to_sql <name>    # Run a specific test by name
cargo test --workspace -- --nocapture  # Run tests with stdout visible
```

### Documentation

```bash
cargo doc --workspace --no-deps        # Build docs for all crates
cargo doc --workspace --no-deps --open # Build and open in browser
cargo doc -p ao_data_to_sql            # Build docs for a specific crate
RUSTDOCFLAGS="-D warnings" cargo doc --workspace --no-deps  # Fail on doc warnings
```