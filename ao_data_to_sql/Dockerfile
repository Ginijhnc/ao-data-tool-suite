FROM rust:1.91.1-bookworm AS builder

WORKDIR /app

COPY Cargo.toml ./
COPY ao_shared ./ao_shared
COPY ao_data_to_sql ./ao_data_to_sql
COPY ao_sql_to_static_files ./ao_sql_to_static_files

RUN cargo build --release --package ao_data_to_sql

FROM debian:12.13-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/ao_data_to_sql /usr/local/bin/
COPY --from=builder /app/ao_shared/migrations ./ao_shared/migrations

RUN useradd -r -s /bin/false aouser && \
    chown -R aouser:aouser /app

USER aouser

CMD ["ao_data_to_sql"]
