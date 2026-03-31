use std::io::IsTerminal;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::Parser;
use serde_json::{Map, Value, json};

use dtcore::diff::{DiffOptions, DiffResult, SheetSource};
use dtcore::format::{detect_format, Format};
use dtcore::reader::{ReadOptions, read_file};

#[derive(Parser)]
#[command(
    name = "dtdiff",
    about = "Compare two tabular data files and show differences",
    version
)]
struct Args {
    /// First file to compare
    file_a: String,

    /// Second file to compare
    file_b: String,

    /// Override format detection (applies to both files)
    #[arg(long, value_name = "FMT")]
    format: Option<String>,

    /// Select sheet by name or index (Excel only)
    #[arg(long, value_name = "NAME|INDEX")]
    sheet: Option<String>,

    /// Key column(s) for matched comparison (comma-separated)
    #[arg(long, value_name = "COL")]
    key: Option<String>,

    /// Float comparison tolerance (default: 1e-10)
    #[arg(long)]
    tolerance: Option<f64>,

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

// ---------------------------------------------------------------------------
// Output formatters (ported from xldiff.rs)
// ---------------------------------------------------------------------------

/// Format a row's values inline: `Name: "Alice"  Score: "90"`
fn format_row_inline(headers: &[String], values: &[String]) -> String {
    headers
        .iter()
        .zip(values.iter())
        .map(|(h, v)| format!("{}: \"{}\"", h, v))
        .collect::<Vec<_>>()
        .join("  ")
}

/// Format diff result as colored (or plain) text output.
fn format_text(result: &DiffResult, color: bool) -> String {
    if !result.has_differences() {
        return "No differences found.\n".to_string();
    }

    let (red, green, yellow, reset) = if color {
        ("\x1b[31m", "\x1b[32m", "\x1b[33m", "\x1b[0m")
    } else {
        ("", "", "", "")
    };

    let mut out = String::new();

    // Header
    out.push_str(&format!(
        "--- {} ({})\n+++ {} ({})\n\n",
        result.source_a.sheet_name,
        result.source_a.file_name,
        result.source_b.sheet_name,
        result.source_b.file_name,
    ));

    // Summary
    out.push_str(&format!(
        "Added: {} | Removed: {} | Modified: {}\n\n",
        result.added.len(),
        result.removed.len(),
        result.modified.len(),
    ));

    // Removed rows
    for row in &result.removed {
        out.push_str(&format!(
            "{}- {}{}",
            red,
            format_row_inline(&result.headers, &row.values),
            reset,
        ));
        out.push('\n');
    }

    // Added rows
    for row in &result.added {
        out.push_str(&format!(
            "{}+ {}{}",
            green,
            format_row_inline(&result.headers, &row.values),
            reset,
        ));
        out.push('\n');
    }

    // Modified rows
    for m in &result.modified {
        let key_display: Vec<String> = result
            .key_columns
            .iter()
            .zip(m.key.iter())
            .map(|(col, val)| format!("{}: \"{}\"", col, val))
            .collect();
        out.push_str(&format!(
            "{}~ {}{}",
            yellow,
            key_display.join("  "),
            reset,
        ));
        out.push('\n');
        for change in &m.changes {
            out.push_str(&format!(
                "    {}: \"{}\" \u{2192} \"{}\"\n",
                change.column, change.old_value, change.new_value,
            ));
        }
    }

    out
}

/// Format diff result as JSON.
fn format_json(result: &DiffResult) -> String {
    let added: Vec<Value> = result
        .added
        .iter()
        .map(|row| {
            let mut map = Map::new();
            for (h, v) in result.headers.iter().zip(row.values.iter()) {
                map.insert(h.clone(), Value::String(v.clone()));
            }
            Value::Object(map)
        })
        .collect();

    let removed: Vec<Value> = result
        .removed
        .iter()
        .map(|row| {
            let mut map = Map::new();
            for (h, v) in result.headers.iter().zip(row.values.iter()) {
                map.insert(h.clone(), Value::String(v.clone()));
            }
            Value::Object(map)
        })
        .collect();

    let modified: Vec<Value> = result
        .modified
        .iter()
        .map(|m| {
            let mut key_map = Map::new();
            for (col, val) in result.key_columns.iter().zip(m.key.iter()) {
                key_map.insert(col.clone(), Value::String(val.clone()));
            }
            let changes: Vec<Value> = m
                .changes
                .iter()
                .map(|c| {
                    json!({
                        "column": c.column,
                        "old": c.old_value,
                        "new": c.new_value,
                    })
                })
                .collect();
            json!({
                "key": Value::Object(key_map),
                "changes": changes,
            })
        })
        .collect();

    let output = json!({
        "added": added,
        "removed": removed,
        "modified": modified,
    });

    serde_json::to_string_pretty(&output).unwrap() + "\n"
}

/// Quote a value per RFC 4180: if it contains comma, quote, or newline, wrap
/// in double quotes and escape any internal quotes by doubling them.
fn csv_quote(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

/// Build a CSV row from a slice of values.
fn csv_row(values: &[String]) -> String {
    values.iter().map(|v| csv_quote(v)).collect::<Vec<_>>().join(",")
}

/// Format diff result as CSV.
///
/// Header: _status, col1, col2, ..., _old_col1, _old_col2, ...
/// Added rows: "added" + values + empty _old_ columns
/// Removed rows: "removed" + values + empty _old_ columns
/// Modified rows: "modified" + new values + old values in _old_ columns
fn format_csv_output(result: &DiffResult) -> String {
    let mut out = String::new();

    // Build header
    let mut header_parts: Vec<String> = vec!["_status".to_string()];
    for h in &result.headers {
        header_parts.push(h.clone());
    }
    for h in &result.headers {
        header_parts.push(format!("_old_{}", h));
    }
    out.push_str(&csv_row(&header_parts));
    out.push('\n');

    let empty_cols: Vec<String> = result.headers.iter().map(|_| String::new()).collect();

    // Removed rows
    for row in &result.removed {
        let mut parts: Vec<String> = vec!["removed".to_string()];
        parts.extend(row.values.iter().cloned());
        while parts.len() < 1 + result.headers.len() {
            parts.push(String::new());
        }
        parts.extend(empty_cols.iter().cloned());
        out.push_str(&csv_row(&parts));
        out.push('\n');
    }

    // Added rows
    for row in &result.added {
        let mut parts: Vec<String> = vec!["added".to_string()];
        parts.extend(row.values.iter().cloned());
        while parts.len() < 1 + result.headers.len() {
            parts.push(String::new());
        }
        parts.extend(empty_cols.iter().cloned());
        out.push_str(&csv_row(&parts));
        out.push('\n');
    }

    // Modified rows
    for m in &result.modified {
        let mut main_cols: Vec<String> = Vec::new();
        let mut old_cols: Vec<String> = Vec::new();

        for h in &result.headers {
            if let Some(key_idx) = result.key_columns.iter().position(|k| k == h) {
                main_cols.push(m.key.get(key_idx).cloned().unwrap_or_default());
                old_cols.push(String::new());
            } else if let Some(change) = m.changes.iter().find(|c| c.column == *h) {
                main_cols.push(change.new_value.clone());
                old_cols.push(change.old_value.clone());
            } else {
                // Unchanged non-key column — leave empty in both
                main_cols.push(String::new());
                old_cols.push(String::new());
            }
        }

        let mut parts: Vec<String> = vec!["modified".to_string()];
        parts.extend(main_cols);
        parts.extend(old_cols);
        out.push_str(&csv_row(&parts));
        out.push('\n');
    }

    out
}

// ---------------------------------------------------------------------------
// run / main
// ---------------------------------------------------------------------------

fn run(args: Args) -> Result<()> {
    let path_a = PathBuf::from(&args.file_a);
    let path_b = PathBuf::from(&args.file_b);

    // Validate files exist
    if !path_a.exists() {
        bail!("file not found: {}", path_a.display());
    }
    if !path_b.exists() {
        bail!("file not found: {}", path_b.display());
    }

    // Detect formats
    let fmt_a = detect_format(&path_a, args.format.as_deref())?;
    let fmt_b = detect_format(&path_b, args.format.as_deref())?;

    // Enforce same-format constraint
    if !fmt_a.same_family(fmt_b) {
        bail!(
            "files have incompatible formats: {:?} vs {:?}. Both files must use the same format family.",
            fmt_a,
            fmt_b
        );
    }

    // Build read options
    let opts_a = ReadOptions {
        sheet: args.sheet.clone(),
        skip_rows: None,
        separator: None,
    };
    let opts_b = ReadOptions {
        sheet: args.sheet.clone(),
        skip_rows: None,
        separator: None,
    };

    // Read DataFrames
    let df_a = read_file(&path_a, fmt_a, &opts_a)?;
    let df_b = read_file(&path_b, fmt_b, &opts_b)?;

    // Resolve key columns
    let key_columns: Vec<String> = if let Some(ref key_str) = args.key {
        key_str.split(',').map(|s| s.trim().to_string()).collect()
    } else {
        vec![]
    };

    // Build source labels
    let file_name_a = path_a
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file_a.clone());
    let file_name_b = path_b
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file_b.clone());

    // Use file name as "sheet name" for non-Excel formats; for Excel use the
    // sheet name from opts (or a placeholder if none was specified).
    let sheet_name_a = if fmt_a == Format::Excel {
        args.sheet.clone().unwrap_or_else(|| file_name_a.clone())
    } else {
        file_name_a.clone()
    };
    let sheet_name_b = if fmt_b == Format::Excel {
        args.sheet.clone().unwrap_or_else(|| file_name_b.clone())
    } else {
        file_name_b.clone()
    };

    let source_a = SheetSource {
        file_name: file_name_a,
        sheet_name: sheet_name_a,
    };
    let source_b = SheetSource {
        file_name: file_name_b,
        sheet_name: sheet_name_b,
    };

    let diff_opts = DiffOptions {
        key_columns,
        tolerance: args.tolerance,
    };

    // Run diff
    let result = dtcore::diff::diff_sheets(&df_a, &df_b, &diff_opts, source_a, source_b)?;

    // TTY detection for color
    let use_color = !args.no_color && std::io::stdout().is_terminal();

    // Format output: --json and --csv are mutually exclusive flags; default is text
    let output = if args.json {
        format_json(&result)
    } else if args.csv {
        format_csv_output(&result)
    } else {
        format_text(&result, use_color)
    };

    print!("{}", output);

    // Exit 1 if differences found (diff convention), 0 if identical
    if result.has_differences() {
        process::exit(1);
    }

    Ok(())
}

fn main() {
    let args = Args::parse();
    if let Err(err) = run(args) {
        eprintln!("dtdiff: {err}");
        process::exit(2);
    }
}
