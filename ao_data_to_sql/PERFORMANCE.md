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

## Failed Performance Improvement Attempts

This section records optimization attempts that were tested and rejected because they caused performance regressions or failed to deliver meaningful improvements. The performance improvement attempts (items 1-9) were designed using Claude Opus 4.6.

### 1. Stream-based Pipeline (Parse + Insert Concurrently)

**What was done**: Replaced the sequential "parse all, then insert all" flow with a producer-consumer pipeline. Added a `tokio::sync::mpsc` channel between parsing and insertion. The parsing phase used `spawn_blocking` to run Rayon parsing in batches, sending each completed batch through the channel. A separate async task consumed batches from the channel and inserted them using the existing `buffer_unordered(8)` concurrent insertion. The goal was to overlap CPU-bound parsing with I/O-bound database writes so that while batch N was being inserted, batch N+1 could be parsed simultaneously.

**Result**: 58% regression on average (3.3s to 5.2s). The `spawn_blocking` overhead per batch and channel synchronization costs exceeded any overlap benefit.

### 2. PostgreSQL COPY Instead of INSERT

**What was done**: Replaced the multi-row `INSERT ... VALUES (...), (...), ... ON CONFLICT` approach with PostgreSQL's `COPY FROM STDIN`. Since COPY doesn't support `ON CONFLICT`, implemented a staging table pattern: (1) created a temp table `_char_staging` with `name TEXT` and `data JSONB` columns, (2) built a CSV string in memory by iterating all 20,057 characters and escaping quotes per CSV spec, (3) used sqlx's `copy_in_raw()` to stream the CSV into the staging table, (4) executed `INSERT INTO characters SELECT * FROM _char_staging ON CONFLICT (name) DO UPDATE SET data = EXCLUDED.data` to merge, (5) wrapped everything in an explicit transaction with `pool.begin()` and `tx.commit()`. Removed the batching and `buffer_unordered(8)` concurrency since all data was processed in one COPY operation.

**Result**: 104% regression on average (3.5s to 7.2s). Processing all records in a single transaction eliminated the 8-way concurrency that previously saturated the connection pool.

### 3. Cow<str> to Avoid UTF-8 String Allocation

**What was proposed**: Replace `s.to_owned()` with `Cow::Borrowed(s)` in `parse_ini_bytes()` when the input bytes are valid UTF-8. The theory was that most character files would be pure ASCII/UTF-8, allowing zero-copy parsing without heap allocation. The change would look like:

```rust
use std::borrow::Cow;

let content: Cow<str> = match core::str::from_utf8(bytes) {
    Ok(s) => Cow::Borrowed(s),  // No allocation for UTF-8
    Err(_) => Cow::Owned(WINDOWS_1252.decode(bytes).0.into_owned()),
};
```

**Result**: Not implemented after developer feedback. Sampled character files from multiple Argentum Online servers revealed that nearly all files contain non-ASCII bytes (accented Spanish characters like a and e in text). When `from_utf8()` fails, the Windows-1252 fallback requires `.into_owned()` anyway, so the optimization would only benefit the minority of pure ASCII files.

**Alternative consideration**: Standardizing all character files to a single encoding (either UTF-8 or Windows-1252) at the game server level would eliminate the dual-path decoding entirely. If all files were guaranteed UTF-8, the `Cow<str>` optimization would provide measurable benefit. If all files were guaranteed Windows-1252, the UTF-8 check could be removed. However, this requires changes outside the parser's scope and depends on game server configuration.

### 4. Increased Connection Pool and Concurrency

**What was done**: Added CLI flags `--pool-size` and `--concurrency` to tune database parallelism. Modified `ao_shared::create_pool()` call in `main.rs` to use the configurable pool size instead of hardcoded 10. Updated `insert_charfiles()` signature to accept a `concurrency` parameter, replacing the hardcoded `buffer_unordered(8)` with `buffer_unordered(concurrency)`. Exported `DEFAULT_POOL_SIZE` and `DEFAULT_CONCURRENCY` constants from `db::characters` module. Tested with `--pool-size 16 --concurrency 12` (60% increase over defaults of 10/8).

**Result**: 146% regression on average. The regression appears to be due to increased lock contention on the `name` unique index when 12 concurrent batches compete for index insertions.

### 5. FxHashMap Instead of std HashMap

**What was done**: Replaced `std::collections::HashMap` with `rustc_hash::FxHashMap` in the INI parser and `std::collections::HashSet` with `FxHashSet` in the charfile parser. Added `rustc-hash = "2.1.1"` to workspace dependencies. Modified `ini.rs` to import `FxHashMap` and updated the type aliases `IniSection` and `IniData` from `HashMap<String, String>` and `HashMap<String, IniSection>` to use `FxHashMap`. Changed `parse_ini_string()` to initialize with `FxHashMap::default()` instead of `HashMap::new()`. In `charfile.rs`, replaced `HashSet<String>` with `FxHashSet<String>` for the `skip_fields` and `skip_sections` fields in `CharfileParser`. The theory was that FxHashMap uses a faster non-cryptographic hash function (FxHash) compared to std HashMap's SipHash, which could provide 2-3x faster hash operations for string keys.

**Result**: No statistically significant improvement. The HashMaps used in INI parsing are small (10-20 sections per file, 5-15 keys per section), so hashing overhead is a tiny fraction of total parsing work compared to file I/O, string allocation, and line iteration.

### 6. Query String Caching by Batch Size

**What was done**: Added a static query cache using `OnceLock<Mutex<HashMap<usize, &'static str>>>` to store prebuilt INSERT query strings keyed by batch size. Created a `get_cached_query(batch_size)` function that checks the cache first, and if the batch size hasn't been seen before, builds the query string using the existing placeholder generation logic, leaks it via `Box::leak(query.into_boxed_str())` to obtain a `&'static str`, and stores it in the cache. Extracted the query building logic into a separate `build_insert_query(batch_size)` function. The `insert_characters_batch()` function was modified to call `get_cached_query(characters.len())` instead of building the query inline. The cache handles variable batch sizes naturally: full batches (e.g., 100 or 1000 depending on CLI arg) and tail batches (e.g., 57 remaining records) are each cached on first use. Memory overhead is bounded since query strings are ~10 bytes per placeholder and only a handful of distinct batch sizes are encountered per run.

**Result**: No statistically significant improvement. The `format!()` and `join()` overhead for building query strings is negligible compared to PostgreSQL server-side processing (query parsing, index updates, WAL writes, network I/O). Additionally, sqlx likely caches prepared statements internally, reducing the benefit of client-side string caching.

### 7. SIMD-Accelerated UTF-8 Validation

**What was done**: Added the `simdutf8` crate (version 0.1.5) to workspace dependencies and replaced the standard library's `core::str::from_utf8()` call in `parse_ini_bytes()` with `simdutf8::basic::from_utf8()`. The simdutf8 crate uses SIMD instructions (SSE2/AVX2 on x86, NEON on ARM) to validate UTF-8 byte sequences in parallel, processing multiple bytes per CPU cycle instead of one byte at a time. The function signature and error handling remained identical since simdutf8's API mirrors the standard library. Updated the module documentation to note the SIMD acceleration.

**Result**: No statistically significant improvement. Parsing times ranged from 0.653s to 0.678s (run-to-run variance ~4%), compared to baseline 0.666s. The majority of character files contain non-ASCII bytes (Spanish accented characters like á, é, ñ), causing the UTF-8 validation to fail immediately on the first non-ASCII byte. The SIMD fast path is only beneficial when validating large UTF-8 buffers; early rejection on the first invalid byte means the validation work is minimal regardless of implementation.

### 8. HashMap Pre-allocation

**What was done**: Modified `parse_ini_string()` in `ini.rs` to pre-allocate HashMap capacity based on typical INI file structure. Added two constants: `ESTIMATED_SECTIONS = 16` and `ESTIMATED_KEYS_PER_SECTION = 12`. Changed the outer HashMap initialization from `HashMap::new()` to `HashMap::with_capacity(ESTIMATED_SECTIONS)`. Changed the section entry creation from `data.entry(section_name).or_default()` to `data.entry(section_name).or_insert_with(|| HashMap::with_capacity(ESTIMATED_KEYS_PER_SECTION))`. These capacity hints allow the allocator to reserve memory upfront, avoiding the rehashing operations that occur when a HashMap grows beyond its current capacity (which typically happens at 75% load factor and requires reallocating and reinserting all entries).

**Result**: No statistically significant improvement. The HashMaps in INI parsing are small (typically 10-20 sections with 5-15 keys each), well below the threshold where rehashing overhead becomes measurable. At these sizes, even multiple rehash operations complete in nanoseconds, while the dominant costs are file I/O (~20μs per file on NVMe), string allocations for keys/values, and line iteration.

### 9. Optimized Uppercase Conversion

**What was done**: Created a new `to_uppercase_optimized()` function in `ini.rs` to replace direct `.to_uppercase()` calls on section names and keys. The function first iterates through the string's bytes using `.bytes().all(|b| !b.is_ascii_lowercase())` to check if any lowercase ASCII characters exist. If the string is already uppercase (or contains only non-ASCII characters), it returns `s.to_owned()` without uppercasing. If lowercase characters are found, it calls `s.to_ascii_uppercase()` instead of the Unicode-aware `to_uppercase()`, since INI keys are ASCII-only. Applied the function to section name extraction (`trimmed[1..trimmed.len() - 1]`) and key extraction (`trimmed[..eq_pos].trim()`). Added `#[inline]` attribute to encourage the compiler to inline the function at call sites.

**Result**: No statistically significant improvement. The byte iteration to detect lowercase characters has overhead comparable to the allocation it aims to avoid. When lowercase is detected, the allocation happens anyway. Additionally, `to_ascii_uppercase()` internally performs a similar byte-by-byte check. The net effect is replacing one allocation with a check-then-allocate pattern that costs roughly the same in aggregate across 20,057 files with ~15 sections and ~10 keys each.