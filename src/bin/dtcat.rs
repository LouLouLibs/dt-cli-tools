use std::path::PathBuf;
use std::process;

use anyhow::{bail, Result};
use clap::Parser;

use dtcore::format::{detect_format, Format};
use dtcore::formatter::{
    format_csv, format_data_table, format_describe, format_empty_sheet, format_head_tail,
    format_header, format_schema, format_sheet_listing,
};
use dtcore::metadata::SheetInfo;
use dtcore::reader::{read_file, read_file_info, ReadOptions};

/// Default row threshold: show all rows if <= this many, otherwise head+tail
const DEFAULT_THRESHOLD: usize = 50;
/// Default head/tail row count when splitting
const DEFAULT_HEAD_TAIL: usize = 25;

#[derive(Parser)]
#[command(
    name = "dtcat",
    about = "View tabular data files in the terminal",
    version
)]
struct Args {
    /// File to view
    file: String,

    /// Override format detection (csv, tsv, parquet, arrow, json, ndjson, excel)
    #[arg(long, value_name = "FMT")]
    format: Option<String>,

    /// Select sheet by name or 0-based index (Excel only)
    #[arg(long, value_name = "NAME|INDEX")]
    sheet: Option<String>,

    /// Skip first N rows
    #[arg(long, value_name = "N")]
    skip: Option<usize>,

    /// Show column names and types only
    #[arg(long)]
    schema: bool,

    /// Show summary statistics
    #[arg(long)]
    describe: bool,

    /// Show first N rows
    #[arg(long, value_name = "N")]
    head: Option<usize>,

    /// Show last N rows
    #[arg(long, value_name = "N")]
    tail: Option<usize>,

    /// Output as CSV instead of markdown table
    #[arg(long)]
    csv: bool,

    /// Show all rows (override adaptive row limit)
    #[arg(long)]
    all: bool,

    /// Randomly sample N rows
    #[arg(long, value_name = "N")]
    sample: Option<usize>,

    /// Show file metadata only
    #[arg(long)]
    info: bool,
}

fn validate_args(args: &Args) -> Result<()> {
    if args.schema && args.describe {
        bail!("--schema and --describe are mutually exclusive");
    }
    if args.sample.is_some() {
        if args.schema {
            bail!("--sample and --schema are mutually exclusive");
        }
        if args.describe {
            bail!("--sample and --describe are mutually exclusive");
        }
        if args.info {
            bail!("--sample and --info are mutually exclusive");
        }
        if args.head.is_some() {
            bail!("--sample and --head are mutually exclusive");
        }
        if args.tail.is_some() {
            bail!("--sample and --tail are mutually exclusive");
        }
        if args.all {
            bail!("--sample and --all are mutually exclusive");
        }
    }
    Ok(())
}

/// Build a synthetic SheetInfo for non-Excel formats from a loaded DataFrame.
fn sheet_info_from_df(file_name: &str, df: &polars::prelude::DataFrame) -> SheetInfo {
    SheetInfo {
        name: file_name.to_string(),
        // rows includes the header row conceptually; formatter subtracts 1
        rows: df.height() + 1,
        cols: df.width(),
    }
}

fn run(args: Args) -> Result<()> {
    validate_args(&args)?;

    let path = PathBuf::from(&args.file);
    if !path.exists() {
        bail!("file not found: {}", path.display());
    }

    let fmt = detect_format(&path, args.format.as_deref())?;

    let file_name = path
        .file_name()
        .map(|s| s.to_string_lossy().to_string())
        .unwrap_or_else(|| args.file.clone());

    // --info: show metadata and exit
    if args.info {
        let info = read_file_info(&path, fmt)?;
        print!("{}", format_header(&file_name, &info));

        // For Excel, also list sheet names and dimensions
        if fmt == Format::Excel && !info.sheets.is_empty() {
            println!();
            for sheet in &info.sheets {
                let data_rows = if sheet.rows == 0 { 0 } else { sheet.rows - 1 };
                println!("  {} ({} rows x {} cols)", sheet.name, data_rows, sheet.cols);
            }
        }
        return Ok(());
    }

    // Excel with multiple sheets and no --sheet: show sheet listing
    if fmt == Format::Excel && args.sheet.is_none() {
        let info = read_file_info(&path, fmt)?;
        if info.sheets.len() > 1 {
            // Load a small sample of each sheet to display schemas
            let mut schemas: Vec<(SheetInfo, polars::prelude::DataFrame)> = Vec::new();
            for sheet in &info.sheets {
                let opts = ReadOptions {
                    sheet: Some(sheet.name.clone()),
                    skip_rows: args.skip,
                    separator: None,
                };
                match read_file(&path, fmt, &opts) {
                    Ok(df) => schemas.push((sheet.clone(), df)),
                    Err(_) => {
                        // Empty or unreadable sheet
                        schemas.push((
                            SheetInfo {
                                name: sheet.name.clone(),
                                rows: 0,
                                cols: 0,
                            },
                            polars::prelude::DataFrame::default(),
                        ));
                    }
                }
            }
            let schema_refs: Vec<(&SheetInfo, polars::prelude::DataFrame)> = schemas
                .iter()
                .map(|(s, df)| (s, df.clone()))
                .collect();
            print!(
                "{}",
                format_sheet_listing(&file_name, &info, &schema_refs)
            );
            return Ok(());
        }
    }

    // Build read options
    let opts = ReadOptions {
        sheet: args.sheet.clone(),
        skip_rows: args.skip,
        separator: None,
    };

    let df = read_file(&path, fmt, &opts)?;

    // Determine sheet info for display
    let sheet = if fmt == Format::Excel {
        // Try to get the sheet name we actually read
        let info = read_file_info(&path, fmt)?;
        if let Some(sheet_arg) = &args.sheet {
            // Find the matching sheet in info
            let matched = info.sheets.iter().find(|s| {
                &s.name == sheet_arg
                    || sheet_arg
                        .parse::<usize>()
                        .map(|idx| {
                            info.sheets
                                .iter()
                                .position(|x| x.name == s.name)
                                .map(|i| i == idx)
                                .unwrap_or(false)
                        })
                        .unwrap_or(false)
            });
            if let Some(s) = matched {
                s.clone()
            } else {
                // Fallback: build from df
                SheetInfo {
                    name: sheet_arg.clone(),
                    rows: df.height() + 1,
                    cols: df.width(),
                }
            }
        } else if let Some(first) = info.sheets.first() {
            first.clone()
        } else {
            sheet_info_from_df(&file_name, &df)
        }
    } else {
        sheet_info_from_df(&file_name, &df)
    };

    // Apply sampling if requested (before any display mode)
    let df = if let Some(n) = args.sample {
        if n >= df.height() {
            df
        } else {
            df.sample_n_literal(n, false, false, None)?
        }
    } else {
        df
    };

    // Handle empty DataFrame
    if df.is_empty() {
        print!("{}", format_empty_sheet(&sheet));
        return Ok(());
    }

    // --schema
    if args.schema {
        print!("{}", format_schema(&sheet, &df));
        return Ok(());
    }

    // --describe
    if args.describe {
        print!("{}", format_describe(&df));
        return Ok(());
    }

    // --csv output mode
    if args.csv {
        print!("{}", format_csv(&df));
        return Ok(());
    }

    // Determine what to display
    let output = match (args.head, args.tail) {
        (Some(h), Some(t)) => {
            // Both specified: show head + tail with omission line
            format_head_tail(&df, h, t)
        }
        (Some(h), None) => {
            // Only --head: slice the DataFrame and show all
            let sliced = df.head(Some(h));
            format_data_table(&sliced)
        }
        (None, Some(t)) => {
            // Only --tail: slice and show all
            let sliced = df.tail(Some(t));
            format_data_table(&sliced)
        }
        (None, None) => {
            // Default: show all if <= threshold or --all, otherwise head+tail
            if args.all || df.height() <= DEFAULT_THRESHOLD {
                format_data_table(&df)
            } else {
                format_head_tail(&df, DEFAULT_HEAD_TAIL, DEFAULT_HEAD_TAIL)
            }
        }
    };

    print!("{}", output);
    Ok(())
}

fn main() {
    let args = Args::parse();
    match run(args) {
        Ok(()) => {}
        Err(err) => {
            // Check if this is an arg validation error (exit 2) vs runtime error (exit 1)
            let msg = err.to_string();
            if msg.contains("mutually exclusive")
                || msg.contains("invalid")
                || msg.contains("unknown format")
            {
                eprintln!("dtcat: {err}");
                process::exit(2);
            } else {
                eprintln!("dtcat: {err}");
                process::exit(1);
            }
        }
    }
}
