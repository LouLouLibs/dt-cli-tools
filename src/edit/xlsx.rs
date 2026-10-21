use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::edit::cell::{CellAssignment, CellValue};

/// Check that `path` has an extension dtset can edit in place (.xlsx / .xlsm).
pub fn check_editable(path: &Path) -> Result<(), String> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();
    match ext.as_str() {
        "xlsx" | "xlsm" => Ok(()),
        "xls" => Err(format!(
            "legacy .xls format is not supported — convert '{}' to .xlsx first (dtset edits .xlsx/.xlsm only)",
            path.display()
        )),
        "" => Err(format!(
            "'{}' has no file extension — dtset edits .xlsx/.xlsm only",
            path.display()
        )),
        other => Err(format!("unsupported format '.{other}' — dtset edits .xlsx/.xlsm only")),
    }
}

/// Write cell assignments to an Excel workbook, preserving existing content and formatting.
///
/// Opens the workbook at `input_path`, resolves the target sheet, applies all assignments,
/// and saves the result to `output_path` atomically (temp file in the same directory,
/// then rename), so a failed save never clobbers an existing file.
///
/// Returns `(count_of_cells_updated, resolved_sheet_name)`.
pub fn write_cells(
    input_path: &Path,
    output_path: &Path,
    sheet_selector: Option<&str>,
    assignments: &[CellAssignment],
) -> Result<(usize, String)> {
    check_editable(input_path).map_err(anyhow::Error::msg)?;
    check_editable(output_path).map_err(anyhow::Error::msg)?;

    let mut book = umya_spreadsheet::reader::xlsx::read(input_path)
        .with_context(|| format!("failed to open workbook '{}'", input_path.display()))?;

    let sheet_count = book.get_sheet_count();
    let sheet_index = resolve_sheet_index(&book, sheet_selector, sheet_count)?;

    let sheet = book
        .get_sheet_mut(&sheet_index)
        .with_context(|| format!("failed to access sheet at index {}", sheet_index))?;

    let sheet_name = sheet.get_name().to_string();

    for assignment in assignments {
        apply_assignment(sheet, assignment);
    }

    save_atomic(&book, output_path)?;

    Ok((assignments.len(), sheet_name))
}

/// Save `book` to a temp file next to `output_path`, then rename it into place.
fn save_atomic(book: &umya_spreadsheet::Spreadsheet, output_path: &Path) -> Result<()> {
    let dir = match output_path.parent() {
        Some(p) if !p.as_os_str().is_empty() => p,
        _ => Path::new("."),
    };
    let tmp = tempfile::Builder::new()
        .prefix(".dtset-")
        .suffix(".xlsx")
        .tempfile_in(dir)
        .with_context(|| format!("failed to create temp file in '{}'", dir.display()))?;

    umya_spreadsheet::writer::xlsx::write(book, tmp.path())
        .with_context(|| format!("failed to write workbook to '{}'", output_path.display()))?;

    tmp.persist(output_path)
        .with_context(|| format!("failed to replace '{}'", output_path.display()))?;
    Ok(())
}

/// Resolve a sheet selector string to a 0-based sheet index.
///
/// - `None` → first sheet (index 0)
/// - Try matching by name first
/// - Then try parsing as a 0-based numeric index
/// - On failure, list available sheet names in the error
fn resolve_sheet_index(
    book: &umya_spreadsheet::Spreadsheet,
    selector: Option<&str>,
    sheet_count: usize,
) -> Result<usize> {
    let Some(selector) = selector else {
        return Ok(0);
    };

    // Try name match
    let sheets = book.get_sheet_collection_no_check();
    for (i, ws) in sheets.iter().enumerate() {
        if ws.get_name() == selector {
            return Ok(i);
        }
    }

    // Try 0-based index
    if let Ok(idx) = selector.parse::<usize>() {
        if idx < sheet_count {
            return Ok(idx);
        }
    }

    // Build error with available names
    let names: Vec<&str> = sheets.iter().map(|ws| ws.get_name()).collect();
    bail!(
        "sheet '{}' not found — available sheets: [{}]",
        selector,
        names.join(", ")
    );
}

/// Apply a single cell assignment to a worksheet.
fn apply_assignment(
    sheet: &mut umya_spreadsheet::Worksheet,
    assignment: &CellAssignment,
) {
    let cell = sheet.get_cell_mut(assignment.cell.label.as_str());

    match &assignment.value {
        CellValue::String(s) => {
            cell.set_value_string(s);
        }
        CellValue::Integer(i) => {
            cell.set_value_number(*i as f64);
        }
        CellValue::Float(f) => {
            cell.set_value_number(*f);
        }
        CellValue::Bool(b) => {
            cell.set_value_bool(*b);
        }
        CellValue::Date { year, month, day } => {
            let serial = date_to_serial(*year, *month, *day);
            cell.set_value_number(serial);
            cell.get_style_mut()
                .get_number_format_mut()
                .set_format_code("yyyy-mm-dd");
        }
        CellValue::Empty => {
            cell.set_value_string("");
        }
    }
}

/// Convert a (year, month, day) date to an Excel serial date number.
///
/// Excel serial dates count days since January 0, 1900 (i.e., Jan 1, 1900 = 1).
/// This function accounts for the Lotus 1-2-3 bug: Excel erroneously treats 1900
/// as a leap year, so dates after Feb 28, 1900 are incremented by 1 to match
/// Excel's numbering.
fn date_to_serial(year: i32, month: u32, day: u32) -> f64 {
    // Days in each month (non-leap)
    let days_in_month = [0, 31, 28, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];

    let mut total_days: i64 = 0;

    // Count full years from 1900 to year-1
    for y in 1900..year {
        total_days += if is_leap_year(y) { 366 } else { 365 };
    }

    // Count full months in the target year
    for m in 1..month {
        total_days += days_in_month[m as usize] as i64;
        if m == 2 && is_leap_year(year) {
            total_days += 1;
        }
    }

    // Add days
    total_days += day as i64;

    // Excel serial: Jan 1, 1900 = 1 (not 0)
    // Lotus 1-2-3 bug: Excel thinks Feb 29, 1900 exists.
    // Dates on or after Mar 1, 1900 (serial >= 61) need +1 to compensate.
    // Feb 29, 1900 itself would be serial 60 in Excel's world (the phantom day).
    if total_days >= 60 {
        total_days += 1;
    }

    total_days as f64
}

/// Check if a year is a real leap year.
fn is_leap_year(year: i32) -> bool {
    (year % 4 == 0 && year % 100 != 0) || (year % 400 == 0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::edit::cell::parse_assignment;
    use rust_xlsxwriter::{Formula, Workbook};
    use tempfile::TempDir;

    fn create_with_formula(path: &Path) {
        let mut wb = Workbook::new();
        let ws = wb.add_worksheet().set_name("Data").unwrap();
        ws.write_number(0, 0, 2.0).unwrap();
        ws.write_number(1, 0, 3.0).unwrap();
        ws.write_formula(2, 0, Formula::new("=A1+A2")).unwrap();
        wb.save(path).unwrap();
    }

    fn create_multi_sheet(path: &Path) {
        let mut wb = Workbook::new();
        for name in ["Revenue", "Expenses", "Summary"] {
            let ws = wb.add_worksheet().set_name(name).unwrap();
            ws.write_string(0, 0, "x").unwrap();
        }
        wb.save(path).unwrap();
    }

    #[test]
    fn check_editable_accepts_xlsx_xlsm() {
        assert!(check_editable(Path::new("foo.xlsx")).is_ok());
        assert!(check_editable(Path::new("foo.XLSM")).is_ok());
    }

    #[test]
    fn check_editable_rejects_others() {
        for p in ["foo.csv", "foo.parquet", "foo.xls", "foo.ods", "foo.xlsb", "foo"] {
            let err = check_editable(Path::new(p)).unwrap_err();
            assert!(err.contains(".xlsx/.xlsm"), "{p}: {err}");
        }
        assert!(check_editable(Path::new("foo.xls")).unwrap_err().contains("convert"));
    }

    #[test]
    fn write_preserves_formula() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("f.xlsx");
        create_with_formula(&path);

        let a = parse_assignment("A1=10").unwrap();
        write_cells(&path, &path, None, &[a]).unwrap();

        let book = umya_spreadsheet::reader::xlsx::read(&path).unwrap();
        let ws = book.get_sheet(&0).unwrap();
        assert_eq!(ws.get_cell("A3").unwrap().get_formula(), "A1+A2");
        assert_eq!(ws.get_value("A1"), "10");
    }

    #[test]
    fn failed_save_leaves_original() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("f.xlsx");
        create_with_formula(&path);
        let before = std::fs::read(&path).unwrap();

        let out = dir.path().join("missing-dir").join("out.xlsx");
        let a = parse_assignment("A1=10").unwrap();
        assert!(write_cells(&path, &out, None, &[a]).is_err());
        assert_eq!(std::fs::read(&path).unwrap(), before);
    }

    #[test]
    fn in_place_save_leaves_no_temp_files() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("f.xlsx");
        create_with_formula(&path);

        let a = parse_assignment("B1=hello").unwrap();
        write_cells(&path, &path, None, &[a]).unwrap();
        assert_eq!(std::fs::read_dir(dir.path()).unwrap().count(), 1);
    }

    #[cfg(unix)]
    #[test]
    fn in_place_save_keeps_permissions() {
        use std::os::unix::fs::PermissionsExt;
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("f.xlsx");
        create_with_formula(&path);
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();

        let a = parse_assignment("B1=hello").unwrap();
        write_cells(&path, &path, None, &[a]).unwrap();
        let mode = std::fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o644);
    }

    #[test]
    fn unknown_sheet_lists_available() {
        let dir = TempDir::new().unwrap();
        let path = dir.path().join("m.xlsx");
        create_multi_sheet(&path);

        let a = parse_assignment("A1=1").unwrap();
        let err = write_cells(&path, &path, Some("Nope"), &[a]).unwrap_err();
        assert!(err.to_string().contains("Revenue, Expenses, Summary"), "{err}");
    }

    #[test]
    fn test_date_to_serial_known_dates() {
        // Jan 1, 1900 = serial 1
        assert_eq!(date_to_serial(1900, 1, 1), 1.0);
        // Jan 1, 2024 = serial 45292
        assert_eq!(date_to_serial(2024, 1, 1), 45292.0);
    }

    #[test]
    fn test_date_to_serial_epoch_boundary() {
        // Feb 28, 1900 = serial 59 (last real date before the phantom leap day)
        assert_eq!(date_to_serial(1900, 2, 28), 59.0);
        // Mar 1, 1900 = serial 61 (after the phantom Feb 29)
        assert_eq!(date_to_serial(1900, 3, 1), 61.0);
    }

    #[test]
    fn test_date_to_serial_common_dates() {
        // Dec 31, 1899 is not representable (before epoch) — but Jan 2, 1900 = 2
        assert_eq!(date_to_serial(1900, 1, 2), 2.0);
        // Excel: 2000-01-01 = 36526
        assert_eq!(date_to_serial(2000, 1, 1), 36526.0);
    }

    #[test]
    fn test_is_leap_year() {
        assert!(!is_leap_year(1900)); // not a real leap year
        assert!(is_leap_year(2000));  // divisible by 400
        assert!(is_leap_year(2024));  // divisible by 4, not by 100
        assert!(!is_leap_year(1999)); // odd year
    }
}
