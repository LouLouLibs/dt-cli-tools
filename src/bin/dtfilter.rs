use std::io::Write;
use std::path::PathBuf;
use std::process;

use anyhow::{Result, bail};
use clap::Parser;

use dtcore::filter::{FilterOptions, parse_filter_expr, parse_sort_spec, filter_pipeline};
use dtcore::format::detect_format;
use dtcore::formatter::{format_data_table, format_csv};
use dtcore::reader::{ReadOptions, read_file};

// ---------------------------------------------------------------------------
// Argument parsing
// ---------------------------------------------------------------------------

#[derive(Parser)]
#[command(
    name = "dtfilter",
    about = "Filter, sort, and select columns from tabular data files",
    version
)]
struct Args {
    /// Input file
    file: String,

    /// Override format detection
    #[arg(long, value_name = "FMT")]
    format: Option<String>,

    /// Select sheet by name or index (Excel only)
    #[arg(long, value_name = "NAME|INDEX")]
    sheet: Option<String>,

    /// Skip first N rows after the header
    #[arg(long, value_name = "N")]
    skip: Option<usize>,

    /// Filter expression(s), e.g. "State=CA", "Amount>1000" (repeatable, ANDed)
    #[arg(long = "filter", value_name = "EXPR", action = clap::ArgAction::Append)]
    filters: Vec<String>,

    /// Sort spec, e.g. "Amount:desc" or "Name"
    #[arg(long, value_name = "SPEC")]
    sort: Option<String>,

    /// Select columns by name (comma-separated)
    #[arg(long, value_name = "COLS")]
    columns: Option<String>,

    /// First N rows (before filter)
    #[arg(long, value_name = "N")]
    head: Option<usize>,

    /// Last N rows (before filter)
    #[arg(long, value_name = "N")]
    tail: Option<usize>,

    /// Max output rows (after filter)
    #[arg(long, value_name = "N")]
    limit: Option<usize>,

    /// Output as CSV
    #[arg(long)]
    csv: bool,
}

// ---------------------------------------------------------------------------
// Validation helpers
// ---------------------------------------------------------------------------

/// Validate args and return an error message for invalid combinations.
/// Returns exit-code 2 on any argument error.
fn validate_args(args: &Args) -> Result<(), ArgError> {
    if args.head.is_some() && args.tail.is_some() {
        return Err(ArgError("--head and --tail are mutually exclusive".to_string()));
    }
    Ok(())
}

struct ArgError(String);

// ---------------------------------------------------------------------------
// Core logic
// ---------------------------------------------------------------------------

fn run(args: Args) -> Result<()> {
    let path = PathBuf::from(&args.file);

    if !path.exists() {
        bail!("file not found: {}", path.display());
    }

    // Detect format
    let fmt = detect_format(&path, args.format.as_deref())?;

    // Build read options
    let read_opts = ReadOptions {
        sheet: args.sheet.clone(),
        skip_rows: args.skip,
        separator: None,
    };

    // Read the DataFrame
    let df = read_file(&path, fmt, &read_opts)?;

    // Parse filter expressions
    let filters = args
        .filters
        .iter()
        .map(|s| parse_filter_expr(s).map_err(|e| anyhow::anyhow!("{}", e)))
        .collect::<Result<Vec<_>>>()?;

    // Parse sort spec
    let sort = args
        .sort
        .as_deref()
        .map(|s| parse_sort_spec(s).map_err(|e| anyhow::anyhow!("{}", e)))
        .transpose()?;

    // Parse column selection
    let cols: Option<Vec<String>> = args.columns.as_deref().map(|s| {
        s.split(',')
            .map(|c| c.trim().to_string())
            .filter(|c| !c.is_empty())
            .collect()
    });

    // Build filter options
    let filter_opts = FilterOptions {
        filters,
        cols,
        sort,
        limit: args.limit,
        head: args.head,
        tail: args.tail,
    };

    // Run the pipeline
    let result = filter_pipeline(df, &filter_opts)?;

    // Report row count to stderr
    let row_count = result.height();
    eprintln!("{} row{}", row_count, if row_count == 1 { "" } else { "s" });

    // Output
    let output = if args.csv {
        format_csv(&result)
    } else {
        format_data_table(&result)
    };

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    out.write_all(output.as_bytes())?;

    Ok(())
}

fn main() {
    let args = Args::parse();

    if let Err(e) = validate_args(&args) {
        eprintln!("dtfilter: {}", e.0);
        process::exit(2);
    }

    if let Err(err) = run(args) {
        eprintln!("dtfilter: {err}");
        process::exit(1);
    }
}
