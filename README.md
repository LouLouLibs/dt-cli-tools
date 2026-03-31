# dt-cli-tools

CLI tools for viewing, filtering, and comparing tabular data files. Supports CSV, TSV, Parquet, Arrow/Feather, JSON, NDJSON, and Excel.

Three read-only tools: **dtcat**, **dtfilter**, **dtdiff**.

## Install

```bash
cargo install --path .
```

## Formats

| Format | Extensions | Detection |
|--------|-----------|-----------|
| CSV | `.csv` | delimiter heuristic |
| TSV | `.tsv`, `.tab` | delimiter heuristic |
| Parquet | `.parquet`, `.pq` | `PAR1` magic |
| Arrow/Feather | `.arrow`, `.feather`, `.ipc` | `ARROW1` magic |
| JSON | `.json` | `[` prefix |
| NDJSON | `.ndjson`, `.jsonl` | `{` prefix |
| Excel | `.xlsx`, `.xls`, `.xlsb`, `.ods` | ZIP/OLE magic |

Format detection: `--format` flag > magic bytes > file extension.

CSV delimiter auto-detected (comma, tab, semicolon).

---

## dtcat

View and inspect files. Outputs markdown tables by default.

```bash
dtcat data.parquet                  # schema + data (≤50 rows all, >50 head/tail 25)
dtcat data.csv --schema             # column names and types
dtcat data.csv --describe           # summary statistics
dtcat report.xlsx --info            # file metadata (size, format, sheets)
dtcat report.xlsx --sheet Revenue   # specific Excel sheet
dtcat data.csv --head 10            # first 10 rows
dtcat data.csv --tail 5             # last 5 rows
dtcat data.parquet --csv            # output as CSV for piping
dtcat data.txt --format csv         # override format detection
dtcat data.csv --skip 2             # skip metadata rows above header
```

Modes `--schema`, `--describe`, and data (default) are mutually exclusive.

## dtfilter

Filter, sort, and select.

```bash
dtfilter data.csv --filter State=CA                     # equality
dtfilter data.csv --filter Amount>1000                   # numeric comparison
dtfilter data.csv --filter State=CA --filter Amount>1000 # AND logic
dtfilter data.csv --filter Name~john                     # contains (case-insensitive)
dtfilter data.csv --filter Status!=Draft                 # not equals
dtfilter data.csv --columns State,City,Amount            # select columns
dtfilter data.csv --sort Amount:desc                     # sort descending
dtfilter data.csv --sort Name                            # sort ascending (default)
dtfilter data.csv --filter Active=true --limit 10        # cap output rows
dtfilter data.csv --head 100 --filter State=CA           # window before filter
dtfilter data.parquet --filter value>0 --csv             # CSV output
```

Filter operators: `=` `!=` `>` `<` `>=` `<=` `~` (contains) `!~` (not contains).

`--head`/`--tail` apply before filtering. `--limit` applies after. `--head` and `--tail` are mutually exclusive.

## dtdiff

Compare two files of the same format. Exit code 0 = identical, 1 = differences, 2 = error.

```bash
dtdiff old.csv new.csv                          # positional comparison
dtdiff old.csv new.csv --key ID                 # key-based (added/removed/modified)
dtdiff old.csv new.csv --key Date,Ticker        # composite key
dtdiff old.csv new.csv --key ID --tolerance 0.01  # float tolerance
dtdiff old.csv new.csv --key ID --json          # JSON output
dtdiff old.csv new.csv --key ID --csv           # CSV output
dtdiff old.csv new.csv --no-color               # plain text
dtdiff report.xlsx other.xlsx --sheet Revenue   # Excel sheets
```

Both files must be the same format (CSV/TSV are treated as compatible).

**Positional mode** (no `--key`): reports added/removed rows based on full-row equality.

**Key-based mode** (`--key`): matches by key columns, reports added/removed/modified with cell-level changes.

---

## Exit Codes

| Tool | 0 | 1 | 2 |
|------|---|---|---|
| dtcat | success | runtime error | invalid arguments |
| dtfilter | success | runtime error | invalid arguments |
| dtdiff | no differences | differences found | error |

## Architecture

Library crate `dtcore` with three thin binaries. ~60% ported from [xl-cli-tools](https://github.com/LouLouLibs/xl-cli-tools).

```
src/
  format.rs       # format detection (magic bytes + extension)
  reader.rs       # dispatch to format-specific readers
  readers/        # CSV, Parquet, Arrow, JSON, Excel
  formatter.rs    # DataFrame → markdown/CSV output
  filter.rs       # filter expressions, sort, pipeline
  diff.rs         # positional and key-based comparison
  metadata.rs     # FileInfo, SheetInfo, display helpers
```

Built on [Polars](https://pola.rs/) for DataFrames, [calamine](https://github.com/tafia/calamine) for Excel, [clap](https://clap.rs/) for CLI.
