# Performance Profiling Guide

This guide explains how to analyze and optimize the parser's performance.

## Quick Performance Check

The parser includes detailed timing breakdowns automatically:

```bash
cargo run --release --package ao_data_to_sql
```

## Enable CPU/RAM Profiling

For detailed resource usage metrics, enable profiling:

```bash
# Via command line flag
cargo run --release --package ao_data_to_sql -- --enable-profiling

# Via environment variable (add to .env)
ENABLE_PROFILING=true
cargo run --release --package ao_data_to_sql
```

### Sample Output (first run - with profiling enabled)

```
2026-03-03T23:27:34.129962Z  INFO ao_data_to_sql: ========================================
2026-03-03T23:27:34.130242Z  INFO ao_data_to_sql: RESUMEN DE IMPORTACIÓN
2026-03-03T23:27:34.130402Z  INFO ao_data_to_sql: ========================================
2026-03-03T23:27:34.130596Z  INFO ao_data_to_sql: Archivos encontrados:      20057
2026-03-03T23:27:34.130747Z  INFO ao_data_to_sql: Archivos parseados:        20057
2026-03-03T23:27:34.130892Z  INFO ao_data_to_sql: Errores de parseo:         0
2026-03-03T23:27:34.131030Z  INFO ao_data_to_sql: Registros insertados:      20057
2026-03-03T23:27:34.131158Z  INFO ao_data_to_sql: Errores de inserción:      0
2026-03-03T23:27:34.131307Z  INFO ao_data_to_sql: Batch size:                1000
2026-03-03T23:27:34.131445Z  INFO ao_data_to_sql: RAM pico (script):         842.3 MB
2026-03-03T23:27:34.131601Z  INFO ao_data_to_sql: CPU pico (script):         947.7%
2026-03-03T23:27:34.131737Z  INFO ao_data_to_sql: ----------------------------------------
2026-03-03T23:27:34.131873Z  INFO ao_data_to_sql: DESGLOSE DE TIEMPOS:
2026-03-03T23:27:34.132007Z  INFO ao_data_to_sql:   Búsqueda archivos:       0.446s (5.7%)
2026-03-03T23:27:34.132142Z  INFO ao_data_to_sql:   Filtrado modificados:    0.000s (0.0%)
2026-03-03T23:27:34.132282Z  INFO ao_data_to_sql:   Parseo:                  1.819s (23.3%) - 11025 archivos/s
2026-03-03T23:27:34.132433Z  INFO ao_data_to_sql:   Inserción DB:            5.535s (71.0%) - 3624 registros/s
2026-03-03T23:27:34.132563Z  INFO ao_data_to_sql: ----------------------------------------
2026-03-03T23:27:34.132697Z  INFO ao_data_to_sql: TIEMPO TOTAL:              7.801 segundos
2026-03-03T23:27:34.132840Z  INFO ao_data_to_sql: THROUGHPUT GENERAL:        2571 archivos/s
2026-03-03T23:27:34.132974Z  INFO ao_data_to_sql: ========================================
```

## Interpreting Results

### Time Breakdown

- **Búsqueda archivos**: File system scan time
- **Filtrado modificados**: Incremental update filtering
- **Parseo**: INI parsing and data extraction
- **Inserción DB**: PostgreSQL batch inserts

### Performance Metrics

**Throughput Rates:**

- **archivos/s**: Files processed per second
- **registros/s**: Database records inserted per second
- **THROUGHPUT GENERAL**: Overall end-to-end throughput

**Resource Usage (when profiling enabled):**

- **RAM pico**: Peak memory used by the parser process only (not total system RAM)
- **CPU pico**: Peak CPU usage as a percentage
  - `100%` = Using 1 logical processor at 100%
  - `200%` = Using 2 logical processors at 100%
  - `600%` = Using 6 logical processors at 100%
  - `1200%` = Using 12 logical processors at 100%

**Understanding CPU %**: The value represents total CPU time across all available logical processors (physical cores × hyperthreading). "Logical processors" are what the OS schedules work on.

- **6-core CPU with hyperthreading** (12 logical processors): Maximum is 1200%
- **8-core CPU without hyperthreading** (8 logical processors): Maximum is 800%
- If you see `710%` on a Ryzen 5 5600, you're using ~6 out of 6 available logical processors during peak processing.
