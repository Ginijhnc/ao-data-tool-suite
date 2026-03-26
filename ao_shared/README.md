# ao_shared

### What It Does

Provides shared utilities for the AO Data Tool Suite workspace, including database connection management and test infrastructure.

### Features

**Database Connection**: Environment-based PostgreSQL connection pool management with configurable timeouts.

**Test Infrastructure**: Shared helpers for E2E tests that spawn PostgreSQL containers via testcontainers and run migrations (available via the `testing` feature flag).

**Schema Management**: Centralized database migrations used across all workspace crates for consistent schema versions.

### Configuration

Database connection is configured via environment variables. See `.env.example` in the workspace root for the full configuration template.
