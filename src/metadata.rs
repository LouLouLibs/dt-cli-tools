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
