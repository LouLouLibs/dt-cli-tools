use anyhow::Result;
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
        _ => vec![],
    };

    Ok(FileInfo {
        file_size,
        format,
        sheets,
    })
}
