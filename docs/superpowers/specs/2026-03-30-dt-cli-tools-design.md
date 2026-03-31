# dt-cli-tools Design Spec

**Date:** 2026-03-30
**Status:** Approved

## Summary

A new Rust CLI tool suite for inspecting, querying, and comparing tabular data files across formats. Three read-only tools ship in v0.1: `dtcat`, `dtfilter`, `dtdiff`. A write tool (`dtset`) is planned for v1.0.

The project reuses the format-agnostic modules from xl-cli-tools (formatter, filter, diff) and adds a multi-format reader layer with automatic format detection.

## Supported Formats

| Format | Extensions | Magic Bytes | Crate | Feature Flag |
|--------|-----------|-------------|-------|-------------|
| CSV | `.csv`, `.tsv`, `.tab` | Heuristic (text + delimiters) | polars | default |
| Parquet | `.parquet`, `.pq` | `PAR1` (4 bytes) | polars | default |
| Arrow/Feather | `.arrow`, `.feather`, `.ipc` | `ARROW1` (6 bytes) | polars | default |
| JSON/NDJSON | `.json`, `.ndjson`, `.jsonl` | `[` or `{` at start | polars | default |
| Excel | `.xlsx`, `.xls`, `.xlsb`, `.ods` | ZIP (`PK`) or OLE (`D0 CF`) | calamine | default |
| DTA (Stata) | `.dta` | Version byte (`0x71`-`0x77`, `0x117`-`0x119`) | readstat | `dta` |

Detection priority: `--format` flag > magic bytes > extension > error.

CSV delimiter detection: auto-detect comma vs tab vs semicolon by sampling the first few lines.

## Project Structure

```
dt-cli-tools/
  Cargo.toml
  src/
    lib.rs
    format.rs              # format detection (magic bytes + extension)
    reader.rs              # Format enum, ReadOptions, read_file dispatch
    formatter.rs           # ported from xl-cli-tools
    filter.rs              # ported from xl-cli-tools (letter-based column resolution removed)
    diff.rs                # ported from xl-cli-tools
    metadata.rs            # format-aware file inspection
    readers/
      csv.rs
      parquet.rs
      arrow.rs
      json.rs
      excel.rs
      dta.rs               # behind feature flag
  src/bin/
    dtcat.rs
    dtfilter.rs
    dtdiff.rs
  tests/
    integration/
  demo/
```

Library crate name: `dtcore`.

## Format Detection and Reading

```rust
pub enum Format {
    Csv, Tsv, Parquet, Arrow, Json, Ndjson, Excel, Dta,
}

pub struct ReadOptions {
    pub sheet: Option<String>,    // Excel only
    pub skip_rows: Option<usize>,
    pub separator: Option<u8>,    // CSV override
}

pub fn detect_format(path: &Path, override_fmt: Option<&str>) -> Result<Format>;
pub fn read_file(path: &Path, format: Format, opts: &ReadOptions) -> Result<DataFrame>;
```

Format dispatch uses a match on `Format` rather than a trait with dynamic dispatch. Each reader module exposes a single `read(path, opts) -> Result<DataFrame>` function.

## Binary Interfaces

### dtcat

```
dtcat <FILE> [OPTIONS]

Options:
  --format <FMT>        Override format detection
  --sheet <NAME|INDEX>  Select sheet (Excel only)
  --skip <N>            Skip first N rows
  --schema              Show column names and types only
  --describe            Show summary statistics
  --head <N>            Show first N rows (default: 50)
  --tail <N>            Show last N rows
  --csv                 Output as CSV instead of markdown table
  --info                Show file metadata (size, format, shape, sheets)
```

### dtfilter

```
dtfilter <FILE> [OPTIONS]

Options:
  --format <FMT>        Override format detection
  --sheet <NAME|INDEX>  Select sheet (Excel only)
  --skip <N>            Skip first N rows
  --filter <EXPR>...    Filter expressions (Amount>1000, Name~john)
  --sort <SPEC>...      Sort specifications (Amount:desc, Name:asc)
  --columns <COLS>      Select columns by name (comma-separated)
  --head <N>            First N rows (applied before filter)
  --tail <N>            Last N rows (applied before filter)
  --limit <N>           Max rows in output (applied after filter)
  --csv                 Output as CSV
```

Column selection by name only. No letter-based addressing.

### dtdiff

```
dtdiff <FILE_A> <FILE_B> [OPTIONS]

Options:
  --format <FMT>        Override format detection (both files must match format)
  --sheet <NAME|INDEX>  Select sheet (Excel only)
  --key <COL>...        Key columns for matched comparison
  --tolerance <N>       Float comparison tolerance (default: 1e-10)
  --json                Output as JSON
  --csv                 Output as CSV
```

Same-format only: errors if the detected formats of FILE_A and FILE_B differ. CSV and TSV are treated as the same format family (delimited text) and can be compared.

## Exit Codes

All tools: 0 = success, 1 = runtime error, 2 = invalid arguments.

Exception: `dtdiff` uses diff(1) convention: 0 = no differences, 1 = differences found, 2 = error.

## Code Reuse from xl-cli-tools

**Ported verbatim:**
- `formatter.rs` — pure Polars DataFrame formatting, no changes needed
- `filter.rs` — remove letter-based column resolution, keep everything else
- `diff.rs` — pure Polars DataFrame comparison, no changes needed

**Written fresh:**
- `format.rs` — magic byte reading, extension matching, format enum
- `reader.rs` — dispatch function and ReadOptions struct
- `readers/*.rs` — thin wrappers around Polars readers (~30-50 lines each)
- `readers/excel.rs` — adapted from xl-cli-tools reader.rs
- `readers/dta.rs` — readstat FFI binding, behind feature flag
- `metadata.rs` — format-aware file inspection
- `dtcat.rs`, `dtfilter.rs`, `dtdiff.rs` — adapted from xl-cli-tools binaries

Roughly 60% ported, 40% new. The new code is mostly thin plumbing. The complex logic (filtering pipeline, diff algorithm, table formatting) transfers from xl-cli-tools.

## Dependencies

| Crate | Purpose | Feature Flag |
|-------|---------|-------------|
| polars | DataFrame engine, CSV/Parquet/Arrow/JSON readers | default |
| calamine | Excel reading (.xlsx, .xls, .xlsb, .ods) | default |
| clap | CLI argument parsing (derive) | default |
| anyhow | Error handling | default |
| serde_json | JSON output for dtdiff | default |
| readstat-rs | DTA (Stata) reading | `dta` |

## Testing

- Unit tests port with their modules (formatter, filter, diff)
- New unit tests for format detection (magic bytes, extensions, ambiguous cases)
- Integration tests per binary, per format: a matrix of (tool x format) using fixture files in `demo/`
- DTA tests gated behind `#[cfg(feature = "dta")]`

## Milestones

- **v0.1:** `dtcat`, `dtfilter`, `dtdiff` — read-only tools, all default formats
- **v1.0:** Add `dtset` — write/edit support for formats where it makes sense
