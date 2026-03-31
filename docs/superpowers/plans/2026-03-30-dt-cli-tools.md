# dt-cli-tools Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a Rust CLI tool suite (`dtcat`, `dtfilter`, `dtdiff`) for inspecting, querying, and comparing tabular data files across formats (CSV, Parquet, Arrow, JSON, Excel).

**Architecture:** Multi-format reader layer with automatic format detection feeds DataFrames into format-agnostic modules (formatter, filter, diff) ported from xl-cli-tools. Three binaries share the `dtcore` library crate.

**Tech Stack:** Rust 2024 edition, Polars 0.46 (DataFrame engine + CSV/Parquet/Arrow/JSON readers), calamine (Excel), clap (CLI), anyhow (errors), serde_json (JSON output).

**Source reference:** xl-cli-tools at `/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/`

---

## File Structure

```
dt-cli-tools/
  Cargo.toml
  src/
    lib.rs                    # pub mod declarations
    format.rs                 # Format enum, magic-byte + extension detection
    reader.rs                 # ReadOptions, read_file dispatch
    metadata.rs               # FileInfo, format_file_size (generalized)
    formatter.rs              # ported from xl-cli-tools (pure DataFrame formatting)
    filter.rs                 # ported from xl-cli-tools (letter-based column resolution removed)
    diff.rs                   # ported from xl-cli-tools (pure DataFrame comparison)
    readers/
      mod.rs                  # sub-module declarations
      csv.rs                  # CSV/TSV reader via Polars CsvReader
      parquet.rs              # Parquet reader via Polars ParquetReader
      arrow.rs                # Arrow IPC reader via Polars IpcReader
      json.rs                 # JSON/NDJSON reader via Polars JsonReader/JsonLineReader
      excel.rs                # Excel reader via calamine (ported from xl-cli-tools reader.rs)
  src/bin/
    dtcat.rs                  # view/inspect any tabular file
    dtfilter.rs               # filter/query any tabular file
    dtdiff.rs                 # compare two tabular files
  tests/
    integration/
      dtcat.rs
      dtfilter.rs
      dtdiff.rs
  demo/                       # fixture files for tests
```

---

### Task 1: Project Scaffolding

**Files:**
- Create: `Cargo.toml`
- Create: `src/lib.rs`
- Create: `src/readers/mod.rs`

- [ ] **Step 1: Create Cargo.toml**

```toml
[package]
name = "dt-cli-tools"
version = "0.1.0"
edition = "2024"
description = "CLI tools for viewing, filtering, and comparing tabular data files"
license = "MIT"

[lib]
name = "dtcore"
path = "src/lib.rs"

[[bin]]
name = "dtcat"
path = "src/bin/dtcat.rs"

[[bin]]
name = "dtfilter"
path = "src/bin/dtfilter.rs"

[[bin]]
name = "dtdiff"
path = "src/bin/dtdiff.rs"

[dependencies]
polars = { version = "0.46", default-features = false, features = [
    "dtype-datetime",
    "csv",
    "parquet",
    "ipc",
    "json",
] }
calamine = "0.26"
clap = { version = "4", features = ["derive"] }
anyhow = "1"
serde_json = { version = "1", features = ["preserve_order"] }

[profile.release]
strip = true
lto = true
codegen-units = 1
panic = "abort"
opt-level = "z"

[dev-dependencies]
assert_cmd = "2"
predicates = "3"
tempfile = "3"
```

- [ ] **Step 2: Create src/lib.rs with module declarations**

```rust
pub mod diff;
pub mod filter;
pub mod format;
pub mod formatter;
pub mod metadata;
pub mod reader;
pub mod readers;
```

- [ ] **Step 3: Create src/readers/mod.rs**

```rust
pub mod arrow;
pub mod csv;
pub mod excel;
pub mod json;
pub mod parquet;
```

- [ ] **Step 4: Create placeholder files so the project compiles**

Create minimal empty-module stubs for every file declared in lib.rs and readers/mod.rs. Each stub is just an empty file or contains only `use anyhow::Result;` as needed. Also create empty `src/bin/dtcat.rs`, `src/bin/dtfilter.rs`, `src/bin/dtdiff.rs` with `fn main() {}`.

- [ ] **Step 5: Verify the project compiles**

Run: `cargo check 2>&1`
Expected: compiles with no errors (warnings OK at this stage)

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/
git commit -m "feat: scaffold dt-cli-tools project structure"
```

---

### Task 2: Format Detection (`format.rs`)

**Files:**
- Create: `src/format.rs`

- [ ] **Step 1: Write tests for format detection**

```rust
// src/format.rs

use anyhow::{Result, bail};
use std::path::Path;
use std::io::Read;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Csv,
    Tsv,
    Parquet,
    Arrow,
    Json,
    Ndjson,
    Excel,
}

impl Format {
    /// Returns true if this format and `other` belong to the same family
    /// (e.g. Csv and Tsv are both delimited text).
    pub fn same_family(&self, other: &Format) -> bool {
        matches!(
            (self, other),
            (Format::Csv, Format::Tsv)
                | (Format::Tsv, Format::Csv)
                | (Format::Json, Format::Ndjson)
                | (Format::Ndjson, Format::Json)
        ) || self == other
    }
}

// Placeholder public functions — will implement in Step 3
pub fn detect_format(path: &Path, override_fmt: Option<&str>) -> Result<Format> {
    todo!()
}

pub fn parse_format_str(s: &str) -> Result<Format> {
    todo!()
}

fn detect_by_magic(path: &Path) -> Result<Option<Format>> {
    todo!()
}

fn detect_by_extension(path: &Path) -> Result<Format> {
    todo!()
}

/// Auto-detect CSV delimiter by sampling the first few lines.
/// Returns b',' (comma), b'\t' (tab), or b';' (semicolon).
pub fn detect_csv_delimiter(path: &Path) -> Result<u8> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // -- parse_format_str --

    #[test]
    fn parse_csv() {
        assert_eq!(parse_format_str("csv").unwrap(), Format::Csv);
    }

    #[test]
    fn parse_tsv() {
        assert_eq!(parse_format_str("tsv").unwrap(), Format::Tsv);
    }

    #[test]
    fn parse_parquet() {
        assert_eq!(parse_format_str("parquet").unwrap(), Format::Parquet);
    }

    #[test]
    fn parse_arrow() {
        assert_eq!(parse_format_str("arrow").unwrap(), Format::Arrow);
    }

    #[test]
    fn parse_json() {
        assert_eq!(parse_format_str("json").unwrap(), Format::Json);
    }

    #[test]
    fn parse_ndjson() {
        assert_eq!(parse_format_str("ndjson").unwrap(), Format::Ndjson);
    }

    #[test]
    fn parse_excel() {
        assert_eq!(parse_format_str("excel").unwrap(), Format::Excel);
        assert_eq!(parse_format_str("xlsx").unwrap(), Format::Excel);
    }

    #[test]
    fn parse_unknown_is_err() {
        assert!(parse_format_str("banana").is_err());
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse_format_str("CSV").unwrap(), Format::Csv);
        assert_eq!(parse_format_str("Parquet").unwrap(), Format::Parquet);
    }

    // -- detect_by_extension --

    #[test]
    fn ext_csv() {
        assert_eq!(detect_by_extension(Path::new("data.csv")).unwrap(), Format::Csv);
    }

    #[test]
    fn ext_tsv() {
        assert_eq!(detect_by_extension(Path::new("data.tsv")).unwrap(), Format::Tsv);
        assert_eq!(detect_by_extension(Path::new("data.tab")).unwrap(), Format::Tsv);
    }

    #[test]
    fn ext_parquet() {
        assert_eq!(detect_by_extension(Path::new("data.parquet")).unwrap(), Format::Parquet);
        assert_eq!(detect_by_extension(Path::new("data.pq")).unwrap(), Format::Parquet);
    }

    #[test]
    fn ext_arrow() {
        assert_eq!(detect_by_extension(Path::new("data.arrow")).unwrap(), Format::Arrow);
        assert_eq!(detect_by_extension(Path::new("data.feather")).unwrap(), Format::Arrow);
        assert_eq!(detect_by_extension(Path::new("data.ipc")).unwrap(), Format::Arrow);
    }

    #[test]
    fn ext_json() {
        assert_eq!(detect_by_extension(Path::new("data.json")).unwrap(), Format::Json);
    }

    #[test]
    fn ext_ndjson() {
        assert_eq!(detect_by_extension(Path::new("data.ndjson")).unwrap(), Format::Ndjson);
        assert_eq!(detect_by_extension(Path::new("data.jsonl")).unwrap(), Format::Ndjson);
    }

    #[test]
    fn ext_excel() {
        assert_eq!(detect_by_extension(Path::new("data.xlsx")).unwrap(), Format::Excel);
        assert_eq!(detect_by_extension(Path::new("data.xls")).unwrap(), Format::Excel);
        assert_eq!(detect_by_extension(Path::new("data.xlsb")).unwrap(), Format::Excel);
        assert_eq!(detect_by_extension(Path::new("data.ods")).unwrap(), Format::Excel);
    }

    #[test]
    fn ext_unknown_is_err() {
        assert!(detect_by_extension(Path::new("data.txt")).is_err());
        assert!(detect_by_extension(Path::new("data")).is_err());
    }

    // -- detect_by_magic --

    #[test]
    fn magic_parquet() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(b"PAR1some_data").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Parquet));
    }

    #[test]
    fn magic_arrow() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(b"ARROW1some_data").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Arrow));
    }

    #[test]
    fn magic_xlsx_zip() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(&[0x50, 0x4B, 0x03, 0x04, 0x00]).unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Excel));
    }

    #[test]
    fn magic_xls_ole() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(&[0xD0, 0xCF, 0x11, 0xE0, 0xA1, 0xB1, 0x1A, 0xE1]).unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Excel));
    }

    #[test]
    fn magic_json_array() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(b"[{\"a\":1}]").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Json));
    }

    #[test]
    fn magic_json_object() {
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(b"{\"a\":1}\n{\"a\":2}").unwrap();
        f.flush().unwrap();
        // Leading { suggests NDJSON
        assert_eq!(detect_by_magic(f.path()).unwrap(), Some(Format::Ndjson));
    }

    #[test]
    fn magic_csv_fallback_none() {
        // Plain text with commas — magic returns None, falls back to extension
        let mut f = NamedTempFile::with_suffix(".bin").unwrap();
        f.write_all(b"a,b,c\n1,2,3\n").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_by_magic(f.path()).unwrap(), None);
    }

    // -- detect_format (integration) --

    #[test]
    fn override_wins() {
        // Even with .csv extension, override to parquet
        assert_eq!(
            detect_format(Path::new("data.csv"), Some("parquet")).unwrap(),
            Format::Parquet
        );
    }

    // -- same_family --

    #[test]
    fn csv_tsv_same_family() {
        assert!(Format::Csv.same_family(&Format::Tsv));
        assert!(Format::Tsv.same_family(&Format::Csv));
    }

    #[test]
    fn json_ndjson_same_family() {
        assert!(Format::Json.same_family(&Format::Ndjson));
    }

    #[test]
    fn csv_parquet_different_family() {
        assert!(!Format::Csv.same_family(&Format::Parquet));
    }

    // -- detect_csv_delimiter --

    #[test]
    fn delimiter_comma() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        f.write_all(b"a,b,c\n1,2,3\n4,5,6\n").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b',');
    }

    #[test]
    fn delimiter_tab() {
        let mut f = NamedTempFile::with_suffix(".tsv").unwrap();
        f.write_all(b"a\tb\tc\n1\t2\t3\n").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b'\t');
    }

    #[test]
    fn delimiter_semicolon() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        f.write_all(b"a;b;c\n1;2;3\n").unwrap();
        f.flush().unwrap();
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b';');
    }
}
```

- [ ] **Step 2: Run tests to verify they fail**

Run: `cargo test --lib format:: 2>&1 | tail -5`
Expected: all tests FAIL (todo! panics)

- [ ] **Step 3: Implement format detection**

Replace the `todo!()` bodies with real implementations:

```rust
pub fn parse_format_str(s: &str) -> Result<Format> {
    match s.to_lowercase().as_str() {
        "csv" => Ok(Format::Csv),
        "tsv" | "tab" => Ok(Format::Tsv),
        "parquet" | "pq" => Ok(Format::Parquet),
        "arrow" | "feather" | "ipc" => Ok(Format::Arrow),
        "json" => Ok(Format::Json),
        "ndjson" | "jsonl" => Ok(Format::Ndjson),
        "excel" | "xlsx" | "xls" | "xlsb" | "ods" => Ok(Format::Excel),
        _ => bail!("unknown format '{}'. Supported: csv, tsv, parquet, arrow, json, ndjson, excel", s),
    }
}

fn detect_by_extension(path: &Path) -> Result<Format> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|e| e.to_lowercase());

    match ext.as_deref() {
        Some("csv") => Ok(Format::Csv),
        Some("tsv") | Some("tab") => Ok(Format::Tsv),
        Some("parquet") | Some("pq") => Ok(Format::Parquet),
        Some("arrow") | Some("feather") | Some("ipc") => Ok(Format::Arrow),
        Some("json") => Ok(Format::Json),
        Some("ndjson") | Some("jsonl") => Ok(Format::Ndjson),
        Some("xlsx") | Some("xls") | Some("xlsb") | Some("ods") => Ok(Format::Excel),
        Some(other) => bail!("unrecognized extension '.{}'. Use --format to specify.", other),
        None => bail!("no file extension. Use --format to specify the format."),
    }
}

fn detect_by_magic(path: &Path) -> Result<Option<Format>> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = [0u8; 8];
    let n = file.read(&mut buf)?;
    if n < 2 {
        return Ok(None);
    }

    // Parquet: "PAR1"
    if n >= 4 && &buf[..4] == b"PAR1" {
        return Ok(Some(Format::Parquet));
    }
    // Arrow IPC: "ARROW1"
    if n >= 6 && &buf[..6] == b"ARROW1" {
        return Ok(Some(Format::Arrow));
    }
    // ZIP (xlsx, ods): PK\x03\x04
    if buf[0] == 0x50 && buf[1] == 0x4B {
        return Ok(Some(Format::Excel));
    }
    // OLE2 (xls): D0 CF 11 E0
    if n >= 4 && buf[0] == 0xD0 && buf[1] == 0xCF && buf[2] == 0x11 && buf[3] == 0xE0 {
        return Ok(Some(Format::Excel));
    }
    // JSON array: starts with [
    // Need to skip leading whitespace
    let first_non_ws = buf[..n].iter().find(|b| !b.is_ascii_whitespace());
    if let Some(b'[') = first_non_ws {
        return Ok(Some(Format::Json));
    }
    if let Some(b'{') = first_non_ws {
        return Ok(Some(Format::Ndjson));
    }

    // CSV/TSV: no distinctive magic bytes — return None to fall through to extension
    Ok(None)
}

pub fn detect_format(path: &Path, override_fmt: Option<&str>) -> Result<Format> {
    if let Some(fmt) = override_fmt {
        return parse_format_str(fmt);
    }
    if let Some(fmt) = detect_by_magic(path)? {
        return Ok(fmt);
    }
    detect_by_extension(path)
}

pub fn detect_csv_delimiter(path: &Path) -> Result<u8> {
    let mut file = std::fs::File::open(path)?;
    let mut buf = String::new();
    // Read up to 8KB for sampling
    file.take(8192).read_to_string(&mut buf)?;

    let lines: Vec<&str> = buf.lines().take(10).collect();
    if lines.is_empty() {
        return Ok(b',');
    }

    let delimiters = [b',', b'\t', b';'];
    let mut best = b',';
    let mut best_score = 0usize;

    for &d in &delimiters {
        let counts: Vec<usize> = lines
            .iter()
            .map(|line| line.as_bytes().iter().filter(|&&b| b == d).count())
            .collect();
        // Score: minimum count across lines (consistency matters)
        let min_count = *counts.iter().min().unwrap_or(&0);
        if min_count > best_score {
            best_score = min_count;
            best = d;
        }
    }

    Ok(best)
}
```

- [ ] **Step 4: Run tests to verify they pass**

Run: `cargo test --lib format:: 2>&1`
Expected: all tests PASS

- [ ] **Step 5: Commit**

```bash
git add src/format.rs
git commit -m "feat: add format detection with magic bytes and extension matching"
```

---

### Task 3: Metadata Module (`metadata.rs`)

**Files:**
- Create: `src/metadata.rs`

- [ ] **Step 1: Write metadata module with tests**

Port `format_file_size` from xl-cli-tools (`/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/metadata.rs`). Generalize `FileInfo` to include the detected format and work for non-Excel files.

```rust
// src/metadata.rs

use crate::format::Format;

/// Info about a single sheet (Excel) or the entire file (other formats).
#[derive(Debug, Clone)]
pub struct SheetInfo {
    pub name: String,
    pub rows: usize, // total rows including header
    pub cols: usize,
}

/// Info about the file.
#[derive(Debug)]
pub struct FileInfo {
    pub file_size: u64,
    pub format: Format,
    pub sheets: Vec<SheetInfo>,
}

/// Format file size for display: "245 KB", "1.2 MB", etc.
pub fn format_file_size(bytes: u64) -> String {
    if bytes < 1_024 {
        format!("{bytes} B")
    } else if bytes < 1_048_576 {
        format!("{:.0} KB", bytes as f64 / 1_024.0)
    } else if bytes < 1_073_741_824 {
        format!("{:.1} MB", bytes as f64 / 1_048_576.0)
    } else {
        format!("{:.1} GB", bytes as f64 / 1_073_741_824.0)
    }
}

/// Format name for a Format variant.
pub fn format_name(fmt: Format) -> &'static str {
    match fmt {
        Format::Csv => "CSV",
        Format::Tsv => "TSV",
        Format::Parquet => "Parquet",
        Format::Arrow => "Arrow IPC",
        Format::Json => "JSON",
        Format::Ndjson => "NDJSON",
        Format::Excel => "Excel",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_file_size() {
        assert_eq!(format_file_size(500), "500 B");
        assert_eq!(format_file_size(2_048), "2 KB");
        assert_eq!(format_file_size(1_500_000), "1.4 MB");
    }

    #[test]
    fn test_format_name() {
        assert_eq!(format_name(Format::Csv), "CSV");
        assert_eq!(format_name(Format::Parquet), "Parquet");
        assert_eq!(format_name(Format::Excel), "Excel");
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib metadata:: 2>&1`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/metadata.rs
git commit -m "feat: add metadata module with FileInfo and format_file_size"
```

---

### Task 4: Formatter Module (`formatter.rs`)

**Files:**
- Create: `src/formatter.rs`

- [ ] **Step 1: Port formatter.rs from xl-cli-tools**

Copy `/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/formatter.rs` and update imports:
- Change `use crate::metadata::{format_file_size, FileInfo, SheetInfo};` to `use crate::metadata::{format_file_size, FileInfo, SheetInfo, format_name};`
- Update `format_header` to include the format name: `# File: report.csv (245 KB) [CSV]`
- The rest of the module (format_schema, format_data_table, format_head_tail, format_csv, format_describe, all helper functions, and all tests) transfers verbatim.

Key change to `format_header`:
```rust
pub fn format_header(file_name: &str, info: &FileInfo) -> String {
    let size_str = format_file_size(info.file_size);
    let fmt_name = format_name(info.format);
    let sheet_count = info.sheets.len();
    if sheet_count > 1 {
        format!("# File: {file_name} ({size_str}) [{fmt_name}]\n# Sheets: {sheet_count}\n")
    } else {
        format!("# File: {file_name} ({size_str}) [{fmt_name}]\n")
    }
}
```

Update the `format_header` test to match the new output:
```rust
#[test]
fn test_format_header() {
    let info = FileInfo {
        file_size: 250_000,
        format: Format::Excel,
        sheets: vec![
            SheetInfo { name: "Sheet1".into(), rows: 100, cols: 5 },
            SheetInfo { name: "Sheet2".into(), rows: 50, cols: 3 },
        ],
    };
    let out = format_header("test.xlsx", &info);
    assert!(out.contains("# File: test.xlsx (244 KB) [Excel]"));
    assert!(out.contains("# Sheets: 2"));
}

#[test]
fn test_format_header_single_sheet() {
    let info = FileInfo {
        file_size: 1_000,
        format: Format::Csv,
        sheets: vec![SheetInfo { name: "data".into(), rows: 10, cols: 3 }],
    };
    let out = format_header("data.csv", &info);
    assert!(out.contains("[CSV]"));
    assert!(!out.contains("Sheets"));
}
```

All other tests (format_data_table, format_head_tail, format_schema, format_csv, format_describe, etc.) transfer verbatim from xl-cli-tools. They test pure DataFrame formatting and don't reference Excel-specific types.

- [ ] **Step 2: Run tests**

Run: `cargo test --lib formatter:: 2>&1`
Expected: all tests PASS

- [ ] **Step 3: Commit**

```bash
git add src/formatter.rs
git commit -m "feat: port formatter module from xl-cli-tools with format-name support"
```

---

### Task 5: Filter Module (`filter.rs`)

**Files:**
- Create: `src/filter.rs`

- [ ] **Step 1: Port filter.rs from xl-cli-tools, removing letter-based column resolution**

Copy `/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/filter.rs` and make these changes:

1. **Remove** `col_letter_to_index` function entirely.
2. **Simplify** `resolve_column` to only do name matching (exact, then case-insensitive). Remove the letter-based fallback step:

```rust
/// Resolve a column specifier to a DataFrame column name.
/// Accepts a header name (exact match first, then case-insensitive).
pub fn resolve_column(spec: &str, df_columns: &[String]) -> Result<String, String> {
    // 1. Exact header name match
    if df_columns.contains(&spec.to_string()) {
        return Ok(spec.to_string());
    }
    // 2. Case-insensitive header name match
    let spec_lower = spec.to_lowercase();
    for col in df_columns {
        if col.to_lowercase() == spec_lower {
            return Ok(col.clone());
        }
    }
    let available = df_columns.join(", ");
    Err(format!("column '{}' not found. Available columns: {}", spec, available))
}
```

3. **Remove** the letter-based tests: `resolve_by_letter`, `resolve_by_letter_lowercase`, `resolve_header_takes_priority_over_letter`, `resolve_letter_out_of_range_is_err`, `pipeline_cols_by_letter`.
4. Keep everything else: `parse_filter_expr`, `parse_sort_spec`, `build_filter_mask`, `apply_filters`, `filter_pipeline`, `FilterOptions`, `SortSpec`, `FilterExpr`, `FilterOp`, `apply_sort`, and all their tests.

- [ ] **Step 2: Run tests**

Run: `cargo test --lib filter:: 2>&1`
Expected: all tests PASS

- [ ] **Step 3: Commit**

```bash
git add src/filter.rs
git commit -m "feat: port filter module from xl-cli-tools without letter-based column resolution"
```

---

### Task 6: Diff Module (`diff.rs`)

**Files:**
- Create: `src/diff.rs`

- [ ] **Step 1: Port diff.rs verbatim from xl-cli-tools**

Copy `/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/diff.rs` and update the import path:
- Change `use crate::formatter;` to `use crate::formatter;` (same - no change needed)

The entire module (SheetSource, DiffRow, CellChange, ModifiedRow, DiffResult, DiffOptions, diff_positional, diff_keyed, diff_sheets, and all tests) transfers verbatim. No changes to logic.

- [ ] **Step 2: Run tests**

Run: `cargo test --lib diff:: 2>&1`
Expected: all tests PASS

- [ ] **Step 3: Commit**

```bash
git add src/diff.rs
git commit -m "feat: port diff module from xl-cli-tools"
```

---

### Task 7: CSV Reader (`readers/csv.rs`)

**Files:**
- Create: `src/readers/csv.rs`

- [ ] **Step 1: Write CSV reader with tests**

```rust
// src/readers/csv.rs

use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::reader::ReadOptions;

pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let separator = opts.separator.unwrap_or_else(|| {
        crate::format::detect_csv_delimiter(path).unwrap_or(b',')
    });

    let mut reader = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(opts.skip_rows.unwrap_or(0))
        .with_parse_options(
            CsvParseOptions::default().with_separator(separator),
        )
        .try_into_reader_with_file_path(Some(path.into()))?;

    let df = reader.finish()?;
    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn default_opts() -> ReadOptions {
        ReadOptions::default()
    }

    #[test]
    fn read_basic_csv() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "name,value\nAlice,100\nBob,200\n").unwrap();
        f.flush().unwrap();

        let df = read(f.path(), &default_opts()).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
        let names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
        assert_eq!(names, vec!["name", "value"]);
    }

    #[test]
    fn read_tsv() {
        let mut f = NamedTempFile::with_suffix(".tsv").unwrap();
        write!(f, "a\tb\n1\t2\n3\t4\n").unwrap();
        f.flush().unwrap();

        let opts = ReadOptions { separator: Some(b'\t'), ..Default::default() };
        let df = read(f.path(), &opts).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn read_with_skip() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "metadata line\nname,value\nAlice,100\n").unwrap();
        f.flush().unwrap();

        let opts = ReadOptions { skip_rows: Some(1), ..Default::default() };
        let df = read(f.path(), &opts).unwrap();
        assert_eq!(df.height(), 1);
        let names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
        assert_eq!(names, vec!["name", "value"]);
    }
}
```

Note: This requires `ReadOptions` from `reader.rs`. Define it first (in the next step, or define a minimal version now).

- [ ] **Step 2: Define ReadOptions in reader.rs**

```rust
// src/reader.rs

/// Options that control how a file is read.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    pub sheet: Option<String>,     // Excel only
    pub skip_rows: Option<usize>,
    pub separator: Option<u8>,     // CSV override
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib readers::csv:: 2>&1`
Expected: all tests PASS

- [ ] **Step 4: Commit**

```bash
git add src/readers/csv.rs src/reader.rs
git commit -m "feat: add CSV/TSV reader with delimiter auto-detection"
```

---

### Task 8: Parquet Reader (`readers/parquet.rs`)

**Files:**
- Create: `src/readers/parquet.rs`

- [ ] **Step 1: Write Parquet reader with tests**

```rust
// src/readers/parquet.rs

use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::reader::ReadOptions;

pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;
    let mut df = ParquetReader::new(file).finish()?;

    if let Some(skip) = opts.skip_rows {
        if skip > 0 && skip < df.height() {
            df = df.slice(skip as i64, df.height() - skip);
        }
    }

    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn default_opts() -> ReadOptions {
        ReadOptions::default()
    }

    #[test]
    fn read_parquet_roundtrip() {
        // Create a parquet file using Polars writer
        let s1 = Series::new("name".into(), &["Alice", "Bob"]);
        let s2 = Series::new("value".into(), &[100i64, 200]);
        let mut df = DataFrame::new(vec![s1.into_column(), s2.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".parquet").unwrap();
        let file = std::fs::File::create(f.path()).unwrap();
        ParquetWriter::new(file).finish(&mut df).unwrap();

        let result = read(f.path(), &default_opts()).unwrap();
        assert_eq!(result.height(), 2);
        assert_eq!(result.width(), 2);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib readers::parquet:: 2>&1`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/readers/parquet.rs
git commit -m "feat: add Parquet reader"
```

---

### Task 9: Arrow IPC Reader (`readers/arrow.rs`)

**Files:**
- Create: `src/readers/arrow.rs`

- [ ] **Step 1: Write Arrow IPC reader with tests**

```rust
// src/readers/arrow.rs

use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::reader::ReadOptions;

pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;
    let mut df = IpcReader::new(file).finish()?;

    if let Some(skip) = opts.skip_rows {
        if skip > 0 && skip < df.height() {
            df = df.slice(skip as i64, df.height() - skip);
        }
    }

    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    fn default_opts() -> ReadOptions {
        ReadOptions::default()
    }

    #[test]
    fn read_arrow_roundtrip() {
        let s1 = Series::new("x".into(), &[1i64, 2, 3]);
        let mut df = DataFrame::new(vec![s1.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".arrow").unwrap();
        let file = std::fs::File::create(f.path()).unwrap();
        IpcWriter::new(file).finish(&mut df).unwrap();

        let result = read(f.path(), &default_opts()).unwrap();
        assert_eq!(result.height(), 3);
        assert_eq!(result.width(), 1);
    }
}
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib readers::arrow:: 2>&1`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/readers/arrow.rs
git commit -m "feat: add Arrow IPC reader"
```

---

### Task 10: JSON/NDJSON Reader (`readers/json.rs`)

**Files:**
- Create: `src/readers/json.rs`

- [ ] **Step 1: Write JSON reader with tests**

```rust
// src/readers/json.rs

use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::format::Format;
use crate::reader::ReadOptions;

pub fn read(path: &Path, format: Format, opts: &ReadOptions) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;

    let mut df = match format {
        Format::Ndjson => {
            JsonLineReader::new(file).finish()?
        }
        _ => {
            // JSON array format
            JsonReader::new(file).finish()?
        }
    };

    if let Some(skip) = opts.skip_rows {
        if skip > 0 && skip < df.height() {
            df = df.slice(skip as i64, df.height() - skip);
        }
    }

    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn default_opts() -> ReadOptions {
        ReadOptions::default()
    }

    #[test]
    fn read_json_array() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(f, r#"[{{"name":"Alice","value":1}},{{"name":"Bob","value":2}}]"#).unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Json, &default_opts()).unwrap();
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn read_ndjson() {
        let mut f = NamedTempFile::with_suffix(".ndjson").unwrap();
        write!(f, "{}\n{}\n",
            r#"{{"name":"Alice","value":1}}"#,
            r#"{{"name":"Bob","value":2}}"#,
        ).unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Ndjson, &default_opts()).unwrap();
        assert_eq!(df.height(), 2);
    }
}
```

Note: Polars JSON reader API may vary. If `JsonReader` is not directly available, use `JsonFormat::Json` with the appropriate reader. The implementer should check the exact Polars 0.46 API and adapt. Alternative approach if `JsonReader` doesn't exist:

```rust
// Alternative using LazyFrame
let lf = LazyJsonLineReader::new(path).finish()?;
let df = lf.collect()?;
```

- [ ] **Step 2: Run tests**

Run: `cargo test --lib readers::json:: 2>&1`
Expected: PASS (adapt if Polars API differs)

- [ ] **Step 3: Commit**

```bash
git add src/readers/json.rs
git commit -m "feat: add JSON/NDJSON reader"
```

---

### Task 11: Excel Reader (`readers/excel.rs`)

**Files:**
- Create: `src/readers/excel.rs`

- [ ] **Step 1: Port Excel reader from xl-cli-tools**

Copy `/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/reader.rs` to `src/readers/excel.rs` and adapt:

1. Change the public API from `read_sheet(path, sheet_name)` / `read_sheet_with_skip(path, sheet_name, skip)` to a single function matching the reader pattern:

```rust
pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame>
```

This function:
- Resolves the sheet name from `opts.sheet` (defaults to the first sheet).
- Applies `opts.skip_rows`.
- Reuses `range_to_dataframe_skip`, `infer_column_type`, `build_series` verbatim from xl-cli-tools.

2. Also provide a helper for reading Excel metadata (sheet names, dimensions):

```rust
pub fn read_excel_info(path: &Path) -> Result<Vec<SheetInfo>>
```

This reuses the calamine-based metadata reading from xl-cli-tools `metadata.rs:read_file_info`, but returns just the sheet list.

3. Port all internal functions (`infer_column_type`, `build_series`, `range_to_dataframe_skip`) and unit tests verbatim.

- [ ] **Step 2: Run tests**

Run: `cargo test --lib readers::excel:: 2>&1`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/readers/excel.rs
git commit -m "feat: port Excel reader from xl-cli-tools"
```

---

### Task 12: Reader Dispatch (`reader.rs`)

**Files:**
- Modify: `src/reader.rs` (already has ReadOptions from Task 7)

- [ ] **Step 1: Add read_file dispatch function**

```rust
// Add to src/reader.rs

use anyhow::{Result, bail};
use polars::prelude::*;
use std::path::Path;

use crate::format::Format;
use crate::metadata::{FileInfo, SheetInfo};
use crate::readers;

/// Options that control how a file is read.
#[derive(Debug, Clone, Default)]
pub struct ReadOptions {
    pub sheet: Option<String>,     // Excel only
    pub skip_rows: Option<usize>,
    pub separator: Option<u8>,     // CSV override
}

/// Read a file into a DataFrame, dispatching to the appropriate reader.
pub fn read_file(path: &Path, format: Format, opts: &ReadOptions) -> Result<DataFrame> {
    match format {
        Format::Csv | Format::Tsv => readers::csv::read(path, opts),
        Format::Parquet => readers::parquet::read(path, opts),
        Format::Arrow => readers::arrow::read(path, opts),
        Format::Json | Format::Ndjson => readers::json::read(path, format, opts),
        Format::Excel => readers::excel::read(path, opts),
    }
}

/// Read file metadata: size, format, and sheet info (for Excel).
pub fn read_file_info(path: &Path, format: Format) -> Result<FileInfo> {
    let file_size = std::fs::metadata(path)?.len();

    let sheets = match format {
        Format::Excel => readers::excel::read_excel_info(path)?,
        _ => vec![], // Non-Excel formats have no sheet concept
    };

    Ok(FileInfo {
        file_size,
        format,
        sheets,
    })
}
```

- [ ] **Step 2: Write integration test for dispatch**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn dispatch_csv() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "a,b\n1,2\n").unwrap();
        f.flush().unwrap();

        let df = read_file(f.path(), Format::Csv, &ReadOptions::default()).unwrap();
        assert_eq!(df.height(), 1);
    }

    #[test]
    fn dispatch_parquet() {
        use polars::prelude::*;
        let s = Series::new("x".into(), &[1i64, 2]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".parquet").unwrap();
        let file = std::fs::File::create(f.path()).unwrap();
        ParquetWriter::new(file).finish(&mut df).unwrap();

        let result = read_file(f.path(), Format::Parquet, &ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test --lib reader:: 2>&1`
Expected: PASS

- [ ] **Step 4: Commit**

```bash
git add src/reader.rs
git commit -m "feat: add reader dispatch with read_file and read_file_info"
```

---

### Task 13: dtcat Binary (`src/bin/dtcat.rs`)

**Files:**
- Create: `src/bin/dtcat.rs`

- [ ] **Step 1: Implement dtcat**

Adapt from xl-cli-tools `xlcat.rs` (`/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/bin/xlcat.rs`). Key changes:

1. Replace `xlcat::` imports with `dtcore::`.
2. Add `--format` flag for format override.
3. Replace Excel-specific file validation with format detection.
4. Add `--info` flag (show file metadata).
5. For non-Excel files, skip sheet resolution (no sheets concept). For Excel files with multiple sheets, keep the same listing behavior.
6. Use `reader::read_file` and `reader::read_file_info` instead of `metadata::read_file_info` + `reader::read_sheet`.

```rust
// src/bin/dtcat.rs

use dtcore::format;
use dtcore::formatter;
use dtcore::metadata::{self, SheetInfo};
use dtcore::reader::{self, ReadOptions};

use anyhow::Result;
use clap::Parser;
use polars::prelude::*;
use std::path::PathBuf;
use std::process;

#[derive(Parser, Debug)]
#[command(name = "dtcat", about = "View tabular data files in the terminal")]
struct Cli {
    /// Path to data file
    file: PathBuf,

    /// Override format detection (csv, tsv, parquet, arrow, json, ndjson, excel)
    #[arg(long)]
    format: Option<String>,

    /// Select sheet by name or 0-based index (Excel only)
    #[arg(long)]
    sheet: Option<String>,

    /// Skip first N rows
    #[arg(long)]
    skip: Option<usize>,

    /// Show column names and types only
    #[arg(long)]
    schema: bool,

    /// Show summary statistics
    #[arg(long)]
    describe: bool,

    /// Show first N rows (default: 50)
    #[arg(long)]
    head: Option<usize>,

    /// Show last N rows
    #[arg(long)]
    tail: Option<usize>,

    /// Output as CSV instead of markdown table
    #[arg(long)]
    csv: bool,

    /// Show file metadata (size, format, shape, sheets)
    #[arg(long)]
    info: bool,
}

#[derive(Debug)]
struct ArgError(String);

impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for ArgError {}

fn run(cli: &Cli) -> Result<()> {
    // Validate flag combinations
    if cli.schema && cli.describe {
        return Err(ArgError("--schema and --describe are mutually exclusive".into()).into());
    }

    // Detect format
    let fmt = format::detect_format(&cli.file, cli.format.as_deref())?;

    // Read file info
    let file_info = reader::read_file_info(&cli.file, fmt)?;
    let file_name = cli.file
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| cli.file.display().to_string());

    // --info mode
    if cli.info {
        let mut out = formatter::format_header(&file_name, &file_info);
        out.push_str(&format!("Format: {}\n", metadata::format_name(fmt)));
        if !file_info.sheets.is_empty() {
            for sheet in &file_info.sheets {
                out.push_str(&format!("  {}: {} rows x {} cols\n", sheet.name, sheet.rows, sheet.cols));
            }
        }
        print!("{out}");
        return Ok(());
    }

    // Build read options
    let read_opts = ReadOptions {
        sheet: cli.sheet.clone(),
        skip_rows: cli.skip,
        separator: None,
    };

    // For Excel with multiple sheets and no --sheet flag: list sheets
    if fmt == format::Format::Excel && file_info.sheets.len() > 1 && cli.sheet.is_none() {
        let has_row_flags = cli.head.is_some() || cli.tail.is_some() || cli.csv;
        if has_row_flags {
            return Err(ArgError(
                "Multiple sheets found. Use --sheet <name> to select one.".into(),
            ).into());
        }

        // List all sheets with schemas
        let mut out = formatter::format_header(&file_name, &file_info);
        out.push('\n');
        for sheet in &file_info.sheets {
            let opts = ReadOptions { sheet: Some(sheet.name.clone()), ..read_opts.clone() };
            let df = reader::read_file(&cli.file, fmt, &opts)?;
            if sheet.rows == 0 && sheet.cols == 0 {
                out.push_str(&formatter::format_empty_sheet(sheet));
            } else {
                out.push_str(&formatter::format_schema(sheet, &df));
            }
            out.push('\n');
        }
        out.push_str("Use --sheet <name> to view a specific sheet.\n");
        print!("{out}");
        return Ok(());
    }

    // Read the data
    let df = reader::read_file(&cli.file, fmt, &read_opts)?;

    // Build a SheetInfo for display
    let sheet_info = if let Some(si) = file_info.sheets.first() {
        si.clone()
    } else {
        SheetInfo {
            name: file_name.clone(),
            rows: df.height() + 1, // +1 for header
            cols: df.width(),
        }
    };

    // Render output
    render_output(cli, &file_name, &file_info, &sheet_info, &df)
}

fn render_output(
    cli: &Cli,
    file_name: &str,
    file_info: &metadata::FileInfo,
    sheet_info: &SheetInfo,
    df: &DataFrame,
) -> Result<()> {
    if cli.csv {
        let selected = select_rows(cli, df);
        print!("{}", formatter::format_csv(&selected));
        return Ok(());
    }

    let mut out = formatter::format_header(file_name, file_info);
    out.push('\n');

    if df.height() == 0 {
        out.push_str(&formatter::format_schema(sheet_info, df));
        out.push_str("\n(no data rows)\n");
        print!("{out}");
        return Ok(());
    }

    if cli.schema {
        out.push_str(&formatter::format_schema(sheet_info, df));
    } else if cli.describe {
        out.push_str(&formatter::format_schema(sheet_info, df));
        out.push_str(&formatter::format_describe(df));
    } else {
        out.push_str(&formatter::format_schema(sheet_info, df));
        out.push('\n');
        out.push_str(&format_data_selection(cli, df));
    }

    print!("{out}");
    Ok(())
}

fn format_data_selection(cli: &Cli, df: &DataFrame) -> String {
    let total = df.height();

    if cli.head.is_some() || cli.tail.is_some() {
        let head_n = cli.head.unwrap_or(0);
        let tail_n = cli.tail.unwrap_or(0);
        if head_n + tail_n >= total || (head_n == 0 && tail_n == 0) {
            return formatter::format_data_table(df);
        }
        if cli.tail.is_none() {
            return formatter::format_data_table(&df.head(Some(head_n)));
        }
        if cli.head.is_none() {
            return formatter::format_data_table(&df.tail(Some(tail_n)));
        }
        return formatter::format_head_tail(df, head_n, tail_n);
    }

    // Default: <=50 rows show all, >50 show head 25 + tail 25
    if total <= 50 {
        formatter::format_data_table(df)
    } else {
        formatter::format_head_tail(df, 25, 25)
    }
}

fn select_rows(cli: &Cli, df: &DataFrame) -> DataFrame {
    let total = df.height();

    if cli.head.is_some() || cli.tail.is_some() {
        let head_n = cli.head.unwrap_or(0);
        let tail_n = cli.tail.unwrap_or(0);
        if head_n + tail_n >= total || (head_n == 0 && tail_n == 0) {
            return df.clone();
        }
        if cli.tail.is_none() {
            return df.head(Some(head_n));
        }
        if cli.head.is_none() {
            return df.tail(Some(tail_n));
        }
        let head_df = df.head(Some(head_n));
        let tail_df = df.tail(Some(tail_n));
        return head_df.vstack(&tail_df).unwrap_or_else(|_| df.clone());
    }

    if total <= 50 { df.clone() } else {
        let h = df.head(Some(25));
        let t = df.tail(Some(25));
        h.vstack(&t).unwrap_or_else(|_| df.clone())
    }
}

fn main() {
    let cli = Cli::parse();
    if let Err(err) = run(&cli) {
        if err.downcast_ref::<ArgError>().is_some() {
            eprintln!("dtcat: {err}");
            process::exit(2);
        }
        eprintln!("dtcat: {err}");
        process::exit(1);
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --bin dtcat 2>&1`
Expected: compiles successfully

- [ ] **Step 3: Manual smoke test**

Create a quick test CSV and run dtcat on it:
```bash
echo "name,value\nAlice,100\nBob,200" > /tmp/test.csv
cargo run --bin dtcat -- /tmp/test.csv
cargo run --bin dtcat -- /tmp/test.csv --schema
cargo run --bin dtcat -- /tmp/test.csv --csv
```

- [ ] **Step 4: Commit**

```bash
git add src/bin/dtcat.rs
git commit -m "feat: add dtcat binary for viewing tabular data files"
```

---

### Task 14: dtfilter Binary (`src/bin/dtfilter.rs`)

**Files:**
- Create: `src/bin/dtfilter.rs`

- [ ] **Step 1: Implement dtfilter**

Adapt from xl-cli-tools `xlfilter.rs` (`/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/bin/xlfilter.rs`). Key changes:

1. Replace `xlcat::` imports with `dtcore::`.
2. Add `--format` flag.
3. Replace Excel-specific file reading with format detection + `reader::read_file`.
4. Remove Excel-specific sheet resolution for non-Excel formats.
5. Change `--cols` description to "Select columns by name" (no letter-based).

```rust
// src/bin/dtfilter.rs

use std::path::PathBuf;
use std::process;

use anyhow::Result;
use clap::Parser;

use dtcore::filter::{parse_filter_expr, parse_sort_spec, filter_pipeline, FilterOptions};
use dtcore::format;
use dtcore::formatter;
use dtcore::reader::{self, ReadOptions};

#[derive(Parser)]
#[command(
    name = "dtfilter",
    about = "Filter and query tabular data files",
    version
)]
struct Args {
    /// Path to data file
    file: PathBuf,

    /// Override format detection
    #[arg(long)]
    format: Option<String>,

    /// Select sheet (Excel only)
    #[arg(long)]
    sheet: Option<String>,

    /// Skip first N rows
    #[arg(long)]
    skip: Option<usize>,

    /// Select columns by name (comma-separated)
    #[arg(long)]
    columns: Option<String>,

    /// Filter expressions (e.g., Amount>1000, Name~john)
    #[arg(long = "filter")]
    filters: Vec<String>,

    /// Sort specification (e.g., Amount:desc)
    #[arg(long)]
    sort: Option<String>,

    /// Max rows in output (applied after filter)
    #[arg(long)]
    limit: Option<usize>,

    /// First N rows (applied before filter)
    #[arg(long)]
    head: Option<usize>,

    /// Last N rows (applied before filter)
    #[arg(long)]
    tail: Option<usize>,

    /// Output as CSV
    #[arg(long)]
    csv: bool,
}

#[derive(Debug)]
struct ArgError(String);
impl std::fmt::Display for ArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl std::error::Error for ArgError {}

fn run(args: Args) -> Result<()> {
    if !args.file.exists() {
        return Err(ArgError(format!("file not found: {}", args.file.display())).into());
    }
    if args.head.is_some() && args.tail.is_some() {
        return Err(ArgError("--head and --tail are mutually exclusive".into()).into());
    }

    let fmt = format::detect_format(&args.file, args.format.as_deref())?;

    let read_opts = ReadOptions {
        sheet: args.sheet,
        skip_rows: args.skip,
        separator: None,
    };

    let df = reader::read_file(&args.file, fmt, &read_opts)?;

    if df.height() == 0 {
        eprintln!("0 rows");
        println!("(no data rows)");
        return Ok(());
    }

    // Parse filter expressions
    let filters: Vec<_> = args.filters
        .iter()
        .map(|s| parse_filter_expr(s))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|e| anyhow::anyhow!(ArgError(e)))?;

    let sort = args.sort
        .as_deref()
        .map(parse_sort_spec)
        .transpose()
        .map_err(|e| anyhow::anyhow!(ArgError(e)))?;

    let cols = args.columns.map(|s| {
        s.split(',').map(|c| c.trim().to_string()).collect::<Vec<_>>()
    });

    let opts = FilterOptions {
        filters,
        cols,
        sort,
        limit: args.limit,
        head: args.head,
        tail: args.tail,
    };

    let result = filter_pipeline(df, &opts)?;

    eprintln!("{} rows", result.height());

    if result.height() == 0 {
        println!("{}", formatter::format_data_table(&result));
    } else if args.csv {
        print!("{}", formatter::format_csv(&result));
    } else {
        println!("{}", formatter::format_data_table(&result));
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(args) {
        if err.downcast_ref::<ArgError>().is_some() {
            eprintln!("dtfilter: {err}");
            process::exit(2);
        }
        eprintln!("dtfilter: {err}");
        process::exit(1);
    }
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --bin dtfilter 2>&1`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/bin/dtfilter.rs
git commit -m "feat: add dtfilter binary for filtering tabular data files"
```

---

### Task 15: dtdiff Binary (`src/bin/dtdiff.rs`)

**Files:**
- Create: `src/bin/dtdiff.rs`

- [ ] **Step 1: Implement dtdiff**

Adapt from xl-cli-tools `xldiff.rs` (`/Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/bin/xldiff.rs`). Key changes:

1. Replace `xlcat::` imports with `dtcore::`.
2. Add `--format` flag.
3. **Same-format enforcement**: detect format of both files and error if they differ (Csv/Tsv are same family and allowed).
4. Replace Excel-specific reading with format detection + `reader::read_file`.
5. Remove letter-based column resolution in key/cols parsing (use name-only `resolve_column`).
6. Port all output formatters (format_text, format_markdown, format_json, format_csv) and tests verbatim.

Exit codes: 0 = no differences, 1 = differences found, 2 = error.

```rust
// src/bin/dtdiff.rs
// Adapted from xl-cli-tools xldiff.rs

use std::io::IsTerminal;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::Parser;
use serde_json::{Map, Value, json};

use dtcore::diff::{DiffOptions, DiffResult, SheetSource};
use dtcore::format;
use dtcore::formatter;
use dtcore::reader::{self, ReadOptions};

#[derive(Parser)]
#[command(
    name = "dtdiff",
    about = "Compare two tabular data files and show differences",
    version
)]
struct Args {
    /// First file
    file_a: PathBuf,

    /// Second file
    file_b: PathBuf,

    /// Override format detection (both files must be this format)
    #[arg(long)]
    format: Option<String>,

    /// Select sheet (Excel only)
    #[arg(long)]
    sheet: Option<String>,

    /// Key column(s) for matched comparison (comma-separated names)
    #[arg(long)]
    key: Option<String>,

    /// Numeric tolerance for float comparisons (default: 1e-10)
    #[arg(long, default_value = "1e-10")]
    tolerance: f64,

    /// Output as JSON
    #[arg(long)]
    json: bool,

    /// Output as CSV
    #[arg(long)]
    csv: bool,

    /// Disable colored output
    #[arg(long)]
    no_color: bool,
}

fn run(args: Args) -> Result<()> {
    if !args.file_a.exists() {
        bail!("file not found: {}", args.file_a.display());
    }
    if !args.file_b.exists() {
        bail!("file not found: {}", args.file_b.display());
    }

    // Detect formats
    let fmt_a = format::detect_format(&args.file_a, args.format.as_deref())?;
    let fmt_b = format::detect_format(&args.file_b, args.format.as_deref())?;

    // Same-format enforcement (Csv/Tsv are same family)
    if !fmt_a.same_family(&fmt_b) {
        bail!(
            "format mismatch: {} is {:?} but {} is {:?}. Both files must be the same format.",
            args.file_a.display(), fmt_a,
            args.file_b.display(), fmt_b,
        );
    }

    let read_opts = ReadOptions {
        sheet: args.sheet.clone(),
        skip_rows: None,
        separator: None,
    };

    let df_a = reader::read_file(&args.file_a, fmt_a, &read_opts)?;
    let df_b = reader::read_file(&args.file_b, fmt_b, &read_opts)?;

    // Resolve key columns
    let key_columns: Vec<String> = if let Some(ref key_str) = args.key {
        key_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![]
    };

    let file_name_a = args.file_a.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file_a.display().to_string());
    let file_name_b = args.file_b.file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file_b.display().to_string());

    let source_a = SheetSource {
        file_name: file_name_a,
        sheet_name: args.sheet.clone().unwrap_or_else(|| "data".into()),
    };
    let source_b = SheetSource {
        file_name: file_name_b,
        sheet_name: args.sheet.unwrap_or_else(|| "data".into()),
    };

    let opts = DiffOptions {
        key_columns,
        tolerance: Some(args.tolerance),
    };

    let result = dtcore::diff::diff_sheets(&df_a, &df_b, &opts, source_a, source_b)?;

    let use_color = !args.no_color && std::io::stdout().is_terminal();

    // Format output
    let output = if args.json {
        format_json(&result)
    } else if args.csv {
        format_csv_output(&result)
    } else {
        format_text(&result, use_color)
    };

    print!("{}", output);

    if result.has_differences() {
        process::exit(1);
    }

    Ok(())
}

// Port format_text, format_json, format_csv_output (renamed from format_csv to avoid
// collision with the flag) verbatim from xl-cli-tools xldiff.rs.
// Include format_row_inline, csv_quote, csv_row helpers.

// [Full implementations copied from xl-cli-tools xldiff.rs - see source at
//  /Users/loulou/Dropbox/projects_claude/xl-cli-tool/src/bin/xldiff.rs lines 141-455]
// The only rename: format_csv -> format_csv_output to avoid name collision.

fn main() {
    let args = Args::parse();
    if let Err(err) = run(args) {
        eprintln!("dtdiff: {err}");
        process::exit(2);
    }
}
```

The output formatter functions (`format_text`, `format_json`, `format_csv_output`, `format_row_inline`, `csv_quote`, `csv_row`) and their tests transfer verbatim from xldiff.rs lines 141-827. Copy them into dtdiff.rs.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --bin dtdiff 2>&1`
Expected: compiles

- [ ] **Step 3: Commit**

```bash
git add src/bin/dtdiff.rs
git commit -m "feat: add dtdiff binary for comparing tabular data files"
```

---

### Task 16: Demo Fixtures and Integration Tests

**Files:**
- Create: `demo/` fixture files
- Create: `tests/integration/dtcat.rs`
- Create: `tests/integration/dtfilter.rs`
- Create: `tests/integration/dtdiff.rs`

- [ ] **Step 1: Create demo fixture files**

Create small test files in `demo/`:

```bash
# demo/sample.csv
echo 'name,value,category
Alice,100,A
Bob,200,B
Charlie,300,A
Diana,400,B
Eve,500,A' > demo/sample.csv

# demo/sample.tsv
printf 'name\tvalue\ncategory\nAlice\t100\tA\nBob\t200\tB\n' > demo/sample.tsv
```

Also create Parquet and Arrow fixtures programmatically in a test helper, or via a small Rust script.

- [ ] **Step 2: Write dtcat integration tests**

```rust
// tests/integration/dtcat.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtcat() -> Command {
    Command::cargo_bin("dtcat").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn shows_csv_data() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtcat()
        .arg(f.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn schema_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat()
        .arg(f.path())
        .arg("--schema")
        .assert()
        .success()
        .stdout(predicate::str::contains("Column"))
        .stdout(predicate::str::contains("Type"));
}

#[test]
fn csv_output_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat()
        .arg(f.path())
        .arg("--csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("name,value"));
}

#[test]
fn head_flag() {
    let f = csv_file("x\n1\n2\n3\n4\n5\n");
    dtcat()
        .arg(f.path())
        .arg("--head")
        .arg("2")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"));
}

#[test]
fn nonexistent_file_exits_1() {
    dtcat()
        .arg("/tmp/does_not_exist.csv")
        .assert()
        .failure();
}

#[test]
fn format_override() {
    // A .txt file read as CSV
    let mut f = NamedTempFile::with_suffix(".txt").unwrap();
    write!(f, "a,b\n1,2\n").unwrap();
    f.flush().unwrap();

    dtcat()
        .arg(f.path())
        .arg("--format")
        .arg("csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("1"));
}
```

- [ ] **Step 3: Write dtfilter integration tests**

```rust
// tests/integration/dtfilter.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtfilter() -> Command {
    Command::cargo_bin("dtfilter").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn filter_eq() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name=Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

#[test]
fn filter_gt() {
    let f = csv_file("name,value\nAlice,100\nBob,200\nCharlie,300\n");
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value>150")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"));
}

#[test]
fn sort_desc() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtfilter()
        .arg(f.path())
        .arg("--sort")
        .arg("value:desc")
        .assert()
        .success();
}

#[test]
fn columns_select() {
    let f = csv_file("name,value,extra\nAlice,100,x\n");
    dtfilter()
        .arg(f.path())
        .arg("--columns")
        .arg("name,value")
        .assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("extra").not());
}

#[test]
fn csv_output() {
    let f = csv_file("name,value\nAlice,100\n");
    dtfilter()
        .arg(f.path())
        .arg("--csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("name,value"));
}
```

- [ ] **Step 4: Write dtdiff integration tests**

```rust
// tests/integration/dtdiff.rs

use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtdiff() -> Command {
    Command::cargo_bin("dtdiff").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn no_diff_exits_0() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nAlice,100\n");
    dtdiff()
        .arg(a.path())
        .arg(b.path())
        .assert()
        .success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn diff_exits_1() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nBob,200\n");
    dtdiff()
        .arg(a.path())
        .arg(b.path())
        .assert()
        .code(1);
}

#[test]
fn keyed_diff() {
    let a = csv_file("id,name\n1,Alice\n2,Bob\n");
    let b = csv_file("id,name\n1,Alice\n2,Robert\n");
    dtdiff()
        .arg(a.path())
        .arg(b.path())
        .arg("--key")
        .arg("id")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("Bob").or(predicate::str::contains("Robert")));
}

#[test]
fn json_output() {
    let a = csv_file("id,val\n1,a\n");
    let b = csv_file("id,val\n1,b\n");
    dtdiff()
        .arg(a.path())
        .arg(b.path())
        .arg("--key")
        .arg("id")
        .arg("--json")
        .assert()
        .code(1)
        .stdout(predicate::str::contains("\"modified\""));
}

#[test]
fn format_mismatch_exits_2() {
    let csv = csv_file("a,b\n1,2\n");
    // Create a file with .parquet extension but CSV content - format detection
    // will see it as parquet by extension, creating a mismatch
    let mut pq = NamedTempFile::with_suffix(".parquet").unwrap();
    write!(pq, "a,b\n1,2\n").unwrap();
    pq.flush().unwrap();
    // This should fail because formats differ (or parquet reader fails on CSV content)
    dtdiff()
        .arg(csv.path())
        .arg(pq.path())
        .assert()
        .failure();
}
```

- [ ] **Step 5: Run all integration tests**

Run: `cargo test --test '*' 2>&1`
Expected: all integration tests PASS

- [ ] **Step 6: Commit**

```bash
git add demo/ tests/
git commit -m "feat: add demo fixtures and integration tests for all binaries"
```

---

### Task 17: Final Verification

- [ ] **Step 1: Run full test suite**

Run: `cargo test 2>&1`
Expected: all unit tests and integration tests PASS

- [ ] **Step 2: Run clippy**

Run: `cargo clippy 2>&1`
Expected: no errors (warnings acceptable)

- [ ] **Step 3: Build release binaries**

Run: `cargo build --release 2>&1`
Expected: builds successfully, produces `dtcat`, `dtfilter`, `dtdiff` in `target/release/`

- [ ] **Step 4: Smoke test all binaries**

```bash
echo "name,value\nAlice,100\nBob,200" > /tmp/dt_test.csv
./target/release/dtcat /tmp/dt_test.csv
./target/release/dtcat /tmp/dt_test.csv --schema
./target/release/dtcat /tmp/dt_test.csv --describe
./target/release/dtfilter /tmp/dt_test.csv --filter "value>100"
echo "name,value\nAlice,100\nCharlie,300" > /tmp/dt_test2.csv
./target/release/dtdiff /tmp/dt_test.csv /tmp/dt_test2.csv
```

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup and verification for v0.1"
```
