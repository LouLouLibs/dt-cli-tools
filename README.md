<div align="center">

<h1>dt-cli-tools</h1>
<h3>View, filter, and diff tabular data files from the command line</h3>

[![Vibecoded](https://img.shields.io/badge/vibecoded-%E2%9C%A8-blueviolet)](https://claude.ai)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

<img src="demo/hero.gif" alt="dt-cli-tools demo" width="80%" />

</div>

***

[**dtcat**](#dtcat--view-data-files) · [**dtfilter**](#dtfilter--query-and-filter) · [**dtdiff**](#dtdiff--compare-two-files) · [**Install**](#installation) · [**Claude Code**](#claude-code-integration)

***

Three read-only binaries, no runtime dependencies. Supports CSV, TSV, Parquet, Arrow/Feather, JSON, NDJSON, and Excel.

```bash
# View a file
dtcat data.parquet

# Filter rows
dtfilter data.csv --filter "Amount>1000" --sort "Amount:desc"

# Diff two files
dtdiff old.csv new.csv --key ID
```

> **Supersedes [xl-cli-tools](https://github.com/LouLouLibs/xl-cli-tools)** — same filtering and diff capabilities, now for all tabular formats.

## Supported Formats

| Format | Extensions | Detection |
|--------|-----------|-----------|
| CSV | `.csv` | delimiter heuristic |
| TSV | `.tsv`, `.tab` | delimiter heuristic |
| Parquet | `.parquet`, `.pq` | `PAR1` magic |
| Arrow/Feather | `.arrow`, `.feather`, `.ipc` | `ARROW1` magic |
| JSON | `.json` | `[` prefix |
| NDJSON | `.ndjson`, `.jsonl` | `{` prefix |
| Excel | `.xlsx`, `.xls`, `.xlsb`, `.ods` | ZIP/OLE magic |

Format detection: `--format` flag > magic bytes > file extension. CSV delimiter auto-detected (comma, tab, semicolon).

## Installation

### Pre-built binaries (macOS)

Download from [Releases](https://github.com/LouLouLibs/dt-cli-tools/releases):

```bash
# Apple Silicon (macOS)
for tool in dtcat dtfilter dtdiff; do
  curl -L "https://github.com/LouLouLibs/dt-cli-tools/releases/latest/download/${tool}-aarch64-apple-darwin" \
    -o ~/.local/bin/$tool
done
chmod +x ~/.local/bin/dt{cat,filter,diff}

# Intel Mac (macOS)
for tool in dtcat dtfilter dtdiff; do
  curl -L "https://github.com/LouLouLibs/dt-cli-tools/releases/latest/download/${tool}-x86_64-apple-darwin" \
    -o ~/.local/bin/$tool
done
chmod +x ~/.local/bin/dt{cat,filter,diff}
```

### From source

```bash
cargo install --path .
```

Requires Rust 1.85+.

## dtcat — View Data Files

<img src="demo/dtcat.gif" alt="dtcat demo" width="80%" />

```bash
# Overview: schema + data (<=50 rows all, >50 head/tail 25)
dtcat data.parquet

# Column names and types only
dtcat data.csv --schema

# Summary statistics (count, mean, std, min, max, median)
dtcat data.csv --describe

# File metadata (size, format, sheets)
dtcat report.xlsx --info

# Pick a sheet in a multi-sheet workbook
dtcat report.xlsx --sheet Revenue

# First 10 rows / last 5 rows
dtcat data.csv --head 10
dtcat data.csv --tail 5

# CSV output for piping
dtcat data.parquet --csv

# Override format detection
dtcat data.txt --format csv

# Skip metadata rows above header
dtcat data.csv --skip 2
```

### Example output

```
# File: sales.parquet (245 KB)
# Format: Parquet

## Data (1240 rows x 4 cols)

| Column  | Type   |
|---------|--------|
| date    | Date   |
| region  | String |
| amount  | Float  |
| units   | Int    |

| date       | region | amount  | units |
|------------|--------|---------|-------|
| 2024-01-01 | East   | 1234.56 | 100   |
| 2024-01-02 | West   | 987.00  | 75    |
... (1190 rows omitted) ...
| 2024-12-30 | East   | 1100.00 | 92    |
| 2024-12-31 | West   | 1250.75 | 110   |
```

### Adaptive defaults

- **Single sheet/file, <=50 rows:** shows all data
- **Single sheet/file, >50 rows:** first 25 + last 25 rows
- **Multiple sheets:** lists schemas, pick one with `--sheet`

Modes `--schema`, `--describe`, `--info`, and data (default) are mutually exclusive.

## dtfilter — Query and Filter

<img src="demo/dtfilter.gif" alt="dtfilter demo" width="80%" />

```bash
# Filter rows by value
dtfilter data.csv --filter State=CA

# Numeric comparisons
dtfilter data.csv --filter Amount>1000

# Multiple filters (AND)
dtfilter data.csv --filter State=CA --filter Amount>1000

# Contains filter (case-insensitive)
dtfilter data.csv --filter Name~john

# Select columns
dtfilter data.csv --columns State,City,Amount

# Sort results
dtfilter data.csv --sort Amount:desc

# Limit output
dtfilter data.csv --sort Amount:desc --limit 10

# Window before filter
dtfilter data.csv --head 100 --filter State=CA

# CSV output for piping
dtfilter data.parquet --filter value>0 --csv
```

### Filter operators

| Operator | Meaning | Example |
|----------|---------|---------|
| `=` | Equals | `State=CA` |
| `!=` | Not equals | `Status!=Draft` |
| `>` | Greater than | `Amount>1000` |
| `<` | Less than | `Year<2024` |
| `>=` | Greater or equal | `Score>=90` |
| `<=` | Less or equal | `Price<=50` |
| `~` | Contains (case-insensitive) | `Name~john` |
| `!~` | Not contains | `Name!~test` |

`--head`/`--tail` apply before filtering. `--limit` applies after. Row count is printed to stderr.

## dtdiff — Compare Two Files

<img src="demo/dtdiff.gif" alt="dtdiff demo" width="80%" />

```bash
# Positional diff (whole-row comparison)
dtdiff old.csv new.csv

# Key-based diff (match rows by ID, compare cell by cell)
dtdiff old.csv new.csv --key ID

# Composite key
dtdiff old.csv new.csv --key Date,Ticker

# Float tolerance (differences <= 0.01 treated as equal)
dtdiff old.csv new.csv --key ID --tolerance 0.01

# Only compare specific columns
dtdiff old.csv new.csv --key ID --columns Name,Salary

# Excel sheets
dtdiff report.xlsx other.xlsx --sheet Revenue

# Output formats
dtdiff old.csv new.csv --key ID --json
dtdiff old.csv new.csv --key ID --csv
dtdiff old.csv new.csv --no-color
```

### Example output

```
--- old.csv
+++ new.csv

Added: 1 | Removed: 1 | Modified: 2

- ID: "3"  Name: "Charlie"  Department: "Engineering"  Salary: "88000"
+ ID: "5"  Name: "Eve"  Department: "Marketing"  Salary: "70000"
~ ID: "1"
    Salary: "95000" → "98000"
~ ID: "2"
    Department: "Marketing" → "Design"
    Salary: "72000" → "75000"
```

### Diff modes

**Positional (no `--key`):** Every column defines row identity. Reports added/removed rows only.

**Key-based (`--key`):** Match rows by key columns, compare remaining columns cell by cell. Reports added, removed, and modified rows with per-cell changes. Supports composite keys, duplicate key detection, and float tolerance.

### Exit codes (diff convention)

| Code | Meaning |
|------|---------|
| 0 | No differences |
| 1 | Differences found |
| 2 | Error |

## Claude Code integration

Claude Code skills are available in [claude-skills](https://github.com/LouLouLibs/claude-skills). Claude can view data files, analyze schemas, filter rows, and compare files in conversations.

## Exit codes

| Tool | 0 | 1 | 2 |
|------|---|---|---|
| dtcat | success | runtime error | invalid arguments |
| dtfilter | success | runtime error | invalid arguments |
| dtdiff | no differences | differences found | error |

## License

MIT
