use anyhow::{anyhow, Result};
use std::io::Read;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    Csv,
    Tsv,
    Parquet,
    Arrow,
    Json,
    Ndjson,
    Excel,
}

impl Format {
    /// Returns true if two formats belong to the same "family":
    /// - Csv and Tsv are the same family
    /// - Json and Ndjson are the same family
    /// - Everything else is only the same family as itself
    pub fn same_family(self, other: Format) -> bool {
        use Format::*;
        matches!(
            (self, other),
            (Csv, Tsv) | (Tsv, Csv) | (Json, Ndjson) | (Ndjson, Json)
        ) || self == other
    }
}

/// Parse a format from a string name. Case-insensitive.
pub fn parse_format_str(s: &str) -> Result<Format> {
    match s.to_ascii_lowercase().as_str() {
        "csv" => Ok(Format::Csv),
        "tsv" | "tab" => Ok(Format::Tsv),
        "parquet" | "pq" => Ok(Format::Parquet),
        "arrow" | "feather" | "ipc" => Ok(Format::Arrow),
        "json" => Ok(Format::Json),
        "ndjson" | "jsonl" => Ok(Format::Ndjson),
        "excel" | "xlsx" | "xls" | "xlsb" | "ods" => Ok(Format::Excel),
        other => Err(anyhow!("unknown format: {:?}", other)),
    }
}

/// Detect format from a file's extension.
pub fn detect_by_extension(path: &Path) -> Result<Format> {
    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .ok_or_else(|| anyhow!("file has no extension: {}", path.display()))?;
    parse_format_str(ext).map_err(|_| anyhow!("unknown file extension: {:?}", ext))
}

/// Read the first 8 bytes of a file and attempt to identify its format from
/// magic bytes. Returns `None` for formats (CSV/TSV) that have no distinctive
/// magic sequence.
pub fn detect_by_magic(path: &Path) -> Result<Option<Format>> {
    let mut file = std::fs::File::open(path)
        .map_err(|e| anyhow!("cannot open {:?}: {}", path, e))?;
    let mut buf = [0u8; 8];
    let n = file.read(&mut buf)?;
    let bytes = &buf[..n];

    // Parquet: magic "PAR1" at start (4 bytes)
    if bytes.len() >= 4 && &bytes[..4] == b"PAR1" {
        return Ok(Some(Format::Parquet));
    }

    // Arrow IPC: magic "ARROW1" at start (6 bytes)
    if bytes.len() >= 6 && &bytes[..6] == b"ARROW1" {
        return Ok(Some(Format::Arrow));
    }

    // Excel ZIP-based (xlsx, xlsb): PK signature
    if bytes.len() >= 2 && bytes[0] == 0x50 && bytes[1] == 0x4B {
        return Ok(Some(Format::Excel));
    }

    // Excel OLE2 (xls): D0 CF 11 E0
    if bytes.len() >= 4
        && bytes[0] == 0xD0
        && bytes[1] == 0xCF
        && bytes[2] == 0x11
        && bytes[3] == 0xE0
    {
        return Ok(Some(Format::Excel));
    }

    // JSON / NDJSON: check first non-whitespace byte
    for &b in bytes {
        if b.is_ascii_whitespace() {
            continue;
        }
        if b == b'[' {
            return Ok(Some(Format::Json));
        }
        if b == b'{' {
            return Ok(Some(Format::Ndjson));
        }
        break;
    }

    // CSV/TSV and anything else has no distinctive magic
    Ok(None)
}

/// Detect format with priority: explicit override > magic bytes > extension.
pub fn detect_format(path: &Path, override_fmt: Option<&str>) -> Result<Format> {
    if let Some(s) = override_fmt {
        return parse_format_str(s);
    }

    if let Some(fmt) = detect_by_magic(path)? {
        return Ok(fmt);
    }

    detect_by_extension(path)
}

/// Detect the delimiter used in a CSV-like file by sampling up to 8 KB / 10
/// lines and counting occurrences of `,`, `\t`, and `;`. Returns the delimiter
/// with the highest minimum count across lines. Defaults to `,`.
pub fn detect_csv_delimiter(path: &Path) -> Result<u8> {
    const MAX_BYTES: usize = 8 * 1024;
    const MAX_LINES: usize = 10;

    let mut file = std::fs::File::open(path)
        .map_err(|e| anyhow!("cannot open {:?}: {}", path, e))?;

    let mut buf = vec![0u8; MAX_BYTES];
    let n = file.read(&mut buf)?;
    buf.truncate(n);

    let candidates: &[u8] = b",\t;";
    // min count per delimiter across lines; start at usize::MAX so we can take min
    let mut min_counts = [usize::MAX; 3];
    let mut line_count = 0usize;

    for line in buf.split(|&b| b == b'\n').take(MAX_LINES) {
        if line.is_empty() {
            continue;
        }
        line_count += 1;
        for (i, &delim) in candidates.iter().enumerate() {
            let count = line.iter().filter(|&&b| b == delim).count();
            if count < min_counts[i] {
                min_counts[i] = count;
            }
        }
    }

    if line_count == 0 {
        return Ok(b',');
    }

    // Replace any usize::MAX (delimiter never appeared) with 0
    for m in min_counts.iter_mut() {
        if *m == usize::MAX {
            *m = 0;
        }
    }

    let best = min_counts
        .iter()
        .enumerate()
        .max_by_key(|&(_, &c)| c)
        .map(|(i, _)| candidates[i])
        .unwrap_or(b',');

    // If no delimiter had any occurrences, fall back to comma
    if min_counts.iter().all(|&c| c == 0) {
        Ok(b',')
    } else {
        Ok(best)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    // ── parse_format_str ──────────────────────────────────────────────────────

    #[test]
    fn parse_csv() {
        assert_eq!(parse_format_str("csv").unwrap(), Format::Csv);
    }

    #[test]
    fn parse_tsv() {
        assert_eq!(parse_format_str("tsv").unwrap(), Format::Tsv);
        assert_eq!(parse_format_str("tab").unwrap(), Format::Tsv);
    }

    #[test]
    fn parse_parquet() {
        assert_eq!(parse_format_str("parquet").unwrap(), Format::Parquet);
        assert_eq!(parse_format_str("pq").unwrap(), Format::Parquet);
    }

    #[test]
    fn parse_arrow() {
        assert_eq!(parse_format_str("arrow").unwrap(), Format::Arrow);
        assert_eq!(parse_format_str("feather").unwrap(), Format::Arrow);
        assert_eq!(parse_format_str("ipc").unwrap(), Format::Arrow);
    }

    #[test]
    fn parse_json() {
        assert_eq!(parse_format_str("json").unwrap(), Format::Json);
    }

    #[test]
    fn parse_ndjson() {
        assert_eq!(parse_format_str("ndjson").unwrap(), Format::Ndjson);
        assert_eq!(parse_format_str("jsonl").unwrap(), Format::Ndjson);
    }

    #[test]
    fn parse_excel() {
        assert_eq!(parse_format_str("excel").unwrap(), Format::Excel);
        assert_eq!(parse_format_str("xlsx").unwrap(), Format::Excel);
        assert_eq!(parse_format_str("xls").unwrap(), Format::Excel);
        assert_eq!(parse_format_str("xlsb").unwrap(), Format::Excel);
        assert_eq!(parse_format_str("ods").unwrap(), Format::Excel);
    }

    #[test]
    fn parse_unknown_errors() {
        assert!(parse_format_str("unknown").is_err());
        assert!(parse_format_str("").is_err());
    }

    #[test]
    fn parse_case_insensitive() {
        assert_eq!(parse_format_str("CSV").unwrap(), Format::Csv);
        assert_eq!(parse_format_str("Parquet").unwrap(), Format::Parquet);
        assert_eq!(parse_format_str("NDJSON").unwrap(), Format::Ndjson);
    }

    // ── detect_by_extension ───────────────────────────────────────────────────

    fn ext_path(ext: &str) -> std::path::PathBuf {
        std::path::PathBuf::from(format!("file.{ext}"))
    }

    #[test]
    fn ext_csv() {
        assert_eq!(detect_by_extension(&ext_path("csv")).unwrap(), Format::Csv);
    }

    #[test]
    fn ext_tsv() {
        assert_eq!(detect_by_extension(&ext_path("tsv")).unwrap(), Format::Tsv);
        assert_eq!(detect_by_extension(&ext_path("tab")).unwrap(), Format::Tsv);
    }

    #[test]
    fn ext_parquet() {
        assert_eq!(
            detect_by_extension(&ext_path("parquet")).unwrap(),
            Format::Parquet
        );
        assert_eq!(
            detect_by_extension(&ext_path("pq")).unwrap(),
            Format::Parquet
        );
    }

    #[test]
    fn ext_arrow() {
        assert_eq!(
            detect_by_extension(&ext_path("arrow")).unwrap(),
            Format::Arrow
        );
        assert_eq!(
            detect_by_extension(&ext_path("feather")).unwrap(),
            Format::Arrow
        );
        assert_eq!(
            detect_by_extension(&ext_path("ipc")).unwrap(),
            Format::Arrow
        );
    }

    #[test]
    fn ext_json() {
        assert_eq!(
            detect_by_extension(&ext_path("json")).unwrap(),
            Format::Json
        );
    }

    #[test]
    fn ext_ndjson() {
        assert_eq!(
            detect_by_extension(&ext_path("ndjson")).unwrap(),
            Format::Ndjson
        );
        assert_eq!(
            detect_by_extension(&ext_path("jsonl")).unwrap(),
            Format::Ndjson
        );
    }

    #[test]
    fn ext_excel() {
        assert_eq!(
            detect_by_extension(&ext_path("xlsx")).unwrap(),
            Format::Excel
        );
        assert_eq!(
            detect_by_extension(&ext_path("xls")).unwrap(),
            Format::Excel
        );
        assert_eq!(
            detect_by_extension(&ext_path("xlsb")).unwrap(),
            Format::Excel
        );
        assert_eq!(
            detect_by_extension(&ext_path("ods")).unwrap(),
            Format::Excel
        );
    }

    #[test]
    fn ext_unknown_errors() {
        assert!(detect_by_extension(&ext_path("txt")).is_err());
        assert!(detect_by_extension(&ext_path("bin")).is_err());
    }

    #[test]
    fn ext_no_extension_errors() {
        assert!(detect_by_extension(Path::new("myfile")).is_err());
    }

    // ── detect_by_magic ───────────────────────────────────────────────────────

    fn temp_with(bytes: &[u8]) -> NamedTempFile {
        let mut f = NamedTempFile::new().unwrap();
        f.write_all(bytes).unwrap();
        f.flush().unwrap();
        f
    }

    #[test]
    fn magic_parquet() {
        let f = temp_with(b"PAR1\x00\x01\x02\x03");
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Parquet)
        );
    }

    #[test]
    fn magic_arrow() {
        let f = temp_with(b"ARROW1\x00\x00");
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Arrow)
        );
    }

    #[test]
    fn magic_xlsx() {
        // ZIP magic: PK (0x50 0x4B)
        let f = temp_with(&[0x50, 0x4B, 0x03, 0x04, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Excel)
        );
    }

    #[test]
    fn magic_xls_ole() {
        // OLE2: D0 CF 11 E0
        let f = temp_with(&[0xD0, 0xCF, 0x11, 0xE0, 0x00, 0x00, 0x00, 0x00]);
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Excel)
        );
    }

    #[test]
    fn magic_json_array() {
        let f = temp_with(b"[{\"a\":1}]");
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Json)
        );
    }

    #[test]
    fn magic_ndjson() {
        let f = temp_with(b"{\"a\":1}\n{\"b\":2}\n");
        assert_eq!(
            detect_by_magic(f.path()).unwrap(),
            Some(Format::Ndjson)
        );
    }

    #[test]
    fn magic_csv_returns_none() {
        let f = temp_with(b"a,b,c\n1,2,3\n");
        assert_eq!(detect_by_magic(f.path()).unwrap(), None);
    }

    // ── detect_format ─────────────────────────────────────────────────────────

    #[test]
    fn detect_override_wins_over_extension() {
        // File content looks like CSV, but we override to parquet
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "a,b\n1,2\n").unwrap();
        let result = detect_format(f.path(), Some("parquet")).unwrap();
        assert_eq!(result, Format::Parquet);
    }

    #[test]
    fn detect_magic_beats_extension() {
        // Write Parquet magic bytes but name the file .csv so extension says Csv
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        f.write_all(b"PAR1\x00\x01\x02\x03").unwrap();
        let result = detect_format(f.path(), None).unwrap();
        assert_eq!(result, Format::Parquet);
    }

    #[test]
    fn detect_falls_back_to_extension() {
        // Plain CSV content → magic returns None → extension used
        let mut f = NamedTempFile::with_suffix(".tsv").unwrap();
        write!(f, "a\tb\n1\t2\n").unwrap();
        let result = detect_format(f.path(), None).unwrap();
        assert_eq!(result, Format::Tsv);
    }

    // ── same_family ───────────────────────────────────────────────────────────

    #[test]
    fn same_family_csv_tsv() {
        assert!(Format::Csv.same_family(Format::Tsv));
        assert!(Format::Tsv.same_family(Format::Csv));
    }

    #[test]
    fn same_family_json_ndjson() {
        assert!(Format::Json.same_family(Format::Ndjson));
        assert!(Format::Ndjson.same_family(Format::Json));
    }

    #[test]
    fn same_family_csv_parquet_different() {
        assert!(!Format::Csv.same_family(Format::Parquet));
        assert!(!Format::Parquet.same_family(Format::Csv));
    }

    #[test]
    fn same_family_same_format() {
        assert!(Format::Csv.same_family(Format::Csv));
        assert!(Format::Parquet.same_family(Format::Parquet));
    }

    // ── detect_csv_delimiter ─────────────────────────────────────────────────

    #[test]
    fn delimiter_comma() {
        let f = temp_with(b"a,b,c\n1,2,3\n4,5,6\n");
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b',');
    }

    #[test]
    fn delimiter_tab() {
        let f = temp_with(b"a\tb\tc\n1\t2\t3\n4\t5\t6\n");
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b'\t');
    }

    #[test]
    fn delimiter_semicolon() {
        let f = temp_with(b"a;b;c\n1;2;3\n4;5;6\n");
        assert_eq!(detect_csv_delimiter(f.path()).unwrap(), b';');
    }
}
