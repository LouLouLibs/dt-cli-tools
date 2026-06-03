use anyhow::{Result, bail};
use polars::prelude::*;
use std::path::Path;

use crate::format::Format;
use crate::writers;

/// Write a DataFrame to a file or stdout, dispatching to the appropriate writer.
pub fn write_file(df: &mut DataFrame, path: Option<&Path>, format: Format) -> Result<()> {
    match format {
        Format::Csv | Format::Tsv => writers::csv::write(df, path, format),
        Format::Parquet => writers::parquet::write(df, path),
        Format::Arrow => writers::arrow::write(df, path),
        Format::Json | Format::Ndjson => writers::json::write(df, path, format),
        Format::Excel => bail!("writing Excel format is not supported; use csv or parquet"),
    }
}
