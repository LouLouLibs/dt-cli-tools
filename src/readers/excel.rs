use anyhow::{Context, Result};
use calamine::{Data, Reader, open_workbook_auto};
use polars::prelude::*;
use std::path::Path;

use crate::metadata::SheetInfo;
use crate::reader::ReadOptions;

#[derive(Debug, Clone, Copy, PartialEq)]
enum InferredType {
    Int,
    Float,
    String,
    Bool,
    DateTime,
    Empty,
}

/// Read an Excel file into a DataFrame using the provided options.
///
/// Sheet resolution order:
/// 1. `opts.sheet` treated as a sheet name (exact match)
/// 2. `opts.sheet` parsed as a 0-based index
/// 3. First sheet (default)
pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let mut workbook = open_workbook_auto(path)
        .with_context(|| format!("Cannot open workbook: {}", path.display()))?;

    let sheet_names = workbook.sheet_names().to_vec();
    if sheet_names.is_empty() {
        return Ok(DataFrame::default());
    }

    let sheet_name: String = match &opts.sheet {
        Some(s) => {
            // Try exact name match first
            if sheet_names.contains(s) {
                s.clone()
            } else {
                // Try to parse as 0-based index
                match s.parse::<usize>() {
                    Ok(idx) if idx < sheet_names.len() => sheet_names[idx].clone(),
                    _ => {
                        return Err(anyhow::anyhow!(
                            "Sheet '{}' not found in workbook (available: {})",
                            s,
                            sheet_names.join(", ")
                        ));
                    }
                }
            }
        }
        None => sheet_names[0].clone(),
    };

    let range = workbook
        .worksheet_range(&sheet_name)
        .with_context(|| format!("Cannot read sheet: {sheet_name}"))?;

    let skip = opts.skip_rows.unwrap_or(0);
    range_to_dataframe_skip(&range, skip)
}

/// Return sheet names and dimensions for an Excel file.
pub fn read_excel_info(path: &Path) -> Result<Vec<SheetInfo>> {
    let mut workbook = open_workbook_auto(path)
        .with_context(|| format!("Cannot open workbook: {}", path.display()))?;

    let sheet_names = workbook.sheet_names().to_vec();
    let mut infos = Vec::with_capacity(sheet_names.len());

    for name in sheet_names {
        let range = workbook
            .worksheet_range(&name)
            .with_context(|| format!("Cannot read sheet: {name}"))?;
        let (rows, cols) = range.get_size();
        infos.push(SheetInfo { name, rows, cols });
    }

    Ok(infos)
}

pub fn range_to_dataframe(range: &calamine::Range<Data>) -> Result<DataFrame> {
    range_to_dataframe_skip(range, 0)
}

/// Convert a calamine Range to a DataFrame, skipping `skip` rows before the header.
pub fn range_to_dataframe_skip(range: &calamine::Range<Data>, skip: usize) -> Result<DataFrame> {
    let rows: Vec<&[Data]> = range.rows().skip(skip).collect();
    let cols = if rows.is_empty() {
        0
    } else {
        rows.iter().map(|r| r.len()).max().unwrap_or(0)
    };

    if rows.is_empty() || cols == 0 {
        return Ok(DataFrame::default());
    }

    // First row (after skip) = headers
    let headers: Vec<String> = rows[0]
        .iter()
        .enumerate()
        .map(|(i, cell)| match cell {
            Data::String(s) => s.clone(),
            _ => format!("column_{i}"),
        })
        .collect();

    if rows.len() == 1 {
        // Header only, no data
        let series: Vec<Column> = headers
            .iter()
            .map(|name| {
                Series::new_empty(PlSmallStr::from(name.as_str()), &DataType::Null).into_column()
            })
            .collect();
        return DataFrame::new(series).map_err(Into::into);
    }

    let data_rows = &rows[1..];
    let mut columns: Vec<Column> = Vec::with_capacity(cols);

    for col_idx in 0..cols {
        let cells: Vec<&Data> = data_rows
            .iter()
            .map(|row| {
                if col_idx < row.len() {
                    &row[col_idx]
                } else {
                    &Data::Empty
                }
            })
            .collect();

        let col_type = infer_column_type(&cells);
        let series = build_series(&headers[col_idx], &cells, col_type)?;
        columns.push(series.into_column());
    }

    DataFrame::new(columns).map_err(Into::into)
}

fn infer_column_type(cells: &[&Data]) -> InferredType {
    let mut has_int = false;
    let mut has_float = false;
    let mut has_string = false;
    let mut has_bool = false;
    let mut has_datetime = false;
    let mut all_empty = true;

    for cell in cells {
        match cell {
            Data::Empty => {}
            Data::String(_) | Data::DateTimeIso(_) | Data::DurationIso(_) => {
                has_string = true;
                all_empty = false;
            }
            Data::Float(_) => {
                has_float = true;
                all_empty = false;
            }
            Data::Int(_) => {
                has_int = true;
                all_empty = false;
            }
            Data::Bool(_) => {
                has_bool = true;
                all_empty = false;
            }
            Data::DateTime(_) => {
                has_datetime = true;
                all_empty = false;
            }
            Data::Error(_) => {
                has_string = true;
                all_empty = false;
            }
        }
    }

    if all_empty {
        return InferredType::Empty;
    }
    // String trumps everything
    if has_string {
        return InferredType::String;
    }
    // DateTime only if all non-empty cells are datetime
    if has_datetime && !has_int && !has_float && !has_bool {
        return InferredType::DateTime;
    }
    // Bool only if all non-empty cells are bool
    if has_bool && !has_int && !has_float && !has_datetime {
        return InferredType::Bool;
    }
    // Float if any float or mix of int/float
    if has_float {
        return InferredType::Float;
    }
    if has_int {
        return InferredType::Int;
    }
    // Fallback: mixed datetime/bool/etc -> string
    InferredType::String
}

fn build_series(name: &str, cells: &[&Data], col_type: InferredType) -> Result<Series> {
    let plname = PlSmallStr::from(name);
    match col_type {
        InferredType::Int => {
            let values: Vec<Option<i64>> = cells
                .iter()
                .map(|cell| match cell {
                    Data::Int(v) => Some(*v),
                    Data::Empty => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(plname, &values))
        }
        InferredType::Float => {
            let values: Vec<Option<f64>> = cells
                .iter()
                .map(|cell| match cell {
                    Data::Float(v) => Some(*v),
                    Data::Int(v) => Some(*v as f64),
                    Data::Empty => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(plname, &values))
        }
        InferredType::Bool => {
            let values: Vec<Option<bool>> = cells
                .iter()
                .map(|cell| match cell {
                    Data::Bool(v) => Some(*v),
                    Data::Empty => None,
                    _ => None,
                })
                .collect();
            Ok(Series::new(plname, &values))
        }
        InferredType::DateTime => {
            // calamine ExcelDateTime wraps a serial date float (days since 1899-12-30)
            // Convert to milliseconds since Unix epoch for polars
            let values: Vec<Option<i64>> = cells
                .iter()
                .map(|cell| match cell {
                    Data::DateTime(v) => {
                        let serial = v.as_f64();
                        // Excel epoch: 1899-12-30 = -25569 days from Unix epoch
                        let days_from_unix = serial - 25569.0;
                        let ms = (days_from_unix * 86_400_000.0) as i64;
                        Some(ms)
                    }
                    Data::Empty => None,
                    _ => None,
                })
                .collect();
            let series = Series::new(plname, &values);
            Ok(series.cast(&DataType::Datetime(TimeUnit::Milliseconds, None))?)
        }
        InferredType::String | InferredType::Empty => {
            let values: Vec<Option<String>> = cells
                .iter()
                .map(|cell| match cell {
                    Data::String(s) => Some(s.clone()),
                    Data::Float(v) => Some(v.to_string()),
                    Data::Int(v) => Some(v.to_string()),
                    Data::Bool(v) => Some(v.to_string()),
                    Data::DateTime(v) => Some(v.as_f64().to_string()),
                    Data::Error(e) => Some(format!("{e:?}")),
                    Data::DateTimeIso(s) | Data::DurationIso(s) => Some(s.clone()),
                    Data::Empty => None,
                })
                .collect();
            Ok(Series::new(plname, &values))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Tests that use rust_xlsxwriter to create fixtures are skipped here because
    // rust_xlsxwriter is not a dev-dependency in dt-cli-tools. The type-inference
    // and range-conversion logic is tested below using calamine types directly.
    // To enable the full xlsx integration tests, add `rust_xlsxwriter` to
    // [dev-dependencies] in Cargo.toml and port the create_simple / create_empty_data
    // / create_with_metadata_rows helpers from xl-cli-tool/src/reader.rs.

    #[test]
    fn test_infer_int_column() {
        let cells = vec![&Data::Int(1), &Data::Int(2), &Data::Empty, &Data::Int(4)];
        assert_eq!(infer_column_type(&cells), InferredType::Int);
    }

    #[test]
    fn test_infer_float_when_mixed_int_float() {
        let cells = vec![&Data::Int(1), &Data::Float(2.5), &Data::Int(3)];
        assert_eq!(infer_column_type(&cells), InferredType::Float);
    }

    #[test]
    fn test_infer_string_trumps_all() {
        let s = Data::String("hello".to_string());
        let cells: Vec<&Data> = vec![&Data::Int(1), &s, &Data::Float(3.0)];
        assert_eq!(infer_column_type(&cells), InferredType::String);
    }

    #[test]
    fn test_infer_empty_column() {
        let cells: Vec<&Data> = vec![&Data::Empty, &Data::Empty];
        assert_eq!(infer_column_type(&cells), InferredType::Empty);
    }

    #[test]
    fn test_infer_bool_column() {
        let cells = vec![&Data::Bool(true), &Data::Bool(false), &Data::Empty];
        assert_eq!(infer_column_type(&cells), InferredType::Bool);
    }

    #[test]
    fn test_empty_range() {
        let range: calamine::Range<Data> = Default::default();
        let df = range_to_dataframe(&range).unwrap();
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 0);
    }

    #[test]
    fn test_build_series_int() {
        let cells = vec![&Data::Int(10), &Data::Int(20), &Data::Empty, &Data::Int(40)];
        let series = build_series("nums", &cells, InferredType::Int).unwrap();
        assert_eq!(series.dtype(), &DataType::Int64);
        assert_eq!(series.len(), 4);
        assert_eq!(series.null_count(), 1);
    }

    #[test]
    fn test_build_series_float() {
        let cells = vec![&Data::Float(1.5), &Data::Int(2), &Data::Empty];
        let series = build_series("vals", &cells, InferredType::Float).unwrap();
        assert_eq!(series.dtype(), &DataType::Float64);
        assert_eq!(series.len(), 3);
        assert_eq!(series.null_count(), 1);
    }

    #[test]
    fn test_build_series_bool() {
        let cells = vec![&Data::Bool(true), &Data::Bool(false), &Data::Empty];
        let series = build_series("flags", &cells, InferredType::Bool).unwrap();
        assert_eq!(series.dtype(), &DataType::Boolean);
        assert_eq!(series.len(), 3);
        assert_eq!(series.null_count(), 1);
    }

    #[test]
    fn test_build_series_string() {
        let s1 = Data::String("foo".to_string());
        let s2 = Data::String("bar".to_string());
        let cells: Vec<&Data> = vec![&s1, &s2, &Data::Empty];
        let series = build_series("words", &cells, InferredType::String).unwrap();
        assert_eq!(series.dtype(), &DataType::String);
        assert_eq!(series.len(), 3);
        assert_eq!(series.null_count(), 1);
    }

    #[test]
    fn test_range_to_dataframe_skip_empty_range() {
        use calamine::Range;
        let range: Range<Data> = Default::default();
        let df = range_to_dataframe_skip(&range, 0).unwrap();
        assert_eq!(df.height(), 0);
        assert_eq!(df.width(), 0);
    }

    #[test]
    fn test_sheet_resolution_default_opts() {
        // Confirm ReadOptions default has sheet=None and skip_rows=None
        let opts = ReadOptions::default();
        assert!(opts.sheet.is_none());
        assert!(opts.skip_rows.is_none());
    }
}
