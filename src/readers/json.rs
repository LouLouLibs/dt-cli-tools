use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::format::Format;
use crate::reader::ReadOptions;

pub fn read(path: &Path, format: Format, opts: &ReadOptions) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;

    let mut df = match format {
        Format::Ndjson => {
            // NDJSON: one JSON object per line — use JsonLineReader
            JsonLineReader::new(file).finish()?
        }
        _ => {
            // JSON array format — JsonReader defaults to JsonFormat::Json
            JsonReader::new(file).finish()?
        }
    };

    if let Some(skip) = opts.skip_rows
        && skip > 0
        && skip < df.height()
    {
        df = df.slice(skip as i64, df.height() - skip);
    }

    Ok(df)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    fn default_opts() -> ReadOptions {
        ReadOptions::default()
    }

    // ── JSON array ────────────────────────────────────────────────────────────

    #[test]
    fn read_json_array_basic() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(
            f,
            r#"[{{"name":"Alice","value":100}},{{"name":"Bob","value":200}},{{"name":"Carol","value":300}}]"#
        )
        .unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Json, &default_opts()).unwrap();
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn read_json_array_with_skip() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(
            f,
            r#"[{{"id":1}},{{"id":2}},{{"id":3}},{{"id":4}},{{"id":5}}]"#
        )
        .unwrap();
        f.flush().unwrap();

        let opts = ReadOptions {
            skip_rows: Some(2),
            ..Default::default()
        };
        let df = read(f.path(), Format::Json, &opts).unwrap();
        // 5 rows total, skip 2 → 3 rows remain
        assert_eq!(df.height(), 3);
    }

    #[test]
    fn read_json_array_skip_zero_noop() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(f, r#"[{{"x":1}},{{"x":2}}]"#).unwrap();
        f.flush().unwrap();

        let opts = ReadOptions {
            skip_rows: Some(0),
            ..Default::default()
        };
        let df = read(f.path(), Format::Json, &opts).unwrap();
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn read_json_array_single_row() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(f, r#"[{{"a":42,"b":"hello"}}]"#).unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Json, &default_opts()).unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 2);
    }

    // ── NDJSON ────────────────────────────────────────────────────────────────

    #[test]
    fn read_ndjson_basic() {
        let mut f = NamedTempFile::with_suffix(".ndjson").unwrap();
        write!(
            f,
            "{}\n{}\n{}\n",
            r#"{"name":"Alice","value":100}"#,
            r#"{"name":"Bob","value":200}"#,
            r#"{"name":"Carol","value":300}"#
        )
        .unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Ndjson, &default_opts()).unwrap();
        assert_eq!(df.height(), 3);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn read_ndjson_with_skip() {
        let mut f = NamedTempFile::with_suffix(".ndjson").unwrap();
        for i in 1..=5 {
            writeln!(f, r#"{{"id":{}}}"#, i).unwrap();
        }
        f.flush().unwrap();

        let opts = ReadOptions {
            skip_rows: Some(2),
            ..Default::default()
        };
        let df = read(f.path(), Format::Ndjson, &opts).unwrap();
        // 5 rows total, skip 2 → 3 rows remain
        assert_eq!(df.height(), 3);
    }

    #[test]
    fn read_ndjson_no_trailing_newline() {
        let mut f = NamedTempFile::with_suffix(".jsonl").unwrap();
        write!(f, "{}\n{}", r#"{"x":1}"#, r#"{"x":2}"#).unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Ndjson, &default_opts()).unwrap();
        assert_eq!(df.height(), 2);
    }

    #[test]
    fn read_ndjson_single_row() {
        let mut f = NamedTempFile::with_suffix(".ndjson").unwrap();
        writeln!(f, r#"{{"a":1,"b":"z"}}"#).unwrap();
        f.flush().unwrap();

        let df = read(f.path(), Format::Ndjson, &default_opts()).unwrap();
        assert_eq!(df.height(), 1);
        assert_eq!(df.width(), 2);
    }

    // ── skip_rows boundary ────────────────────────────────────────────────────

    #[test]
    fn skip_rows_ge_height_noop() {
        let mut f = NamedTempFile::with_suffix(".json").unwrap();
        write!(f, r#"[{{"v":1}},{{"v":2}}]"#).unwrap();
        f.flush().unwrap();

        let opts = ReadOptions {
            skip_rows: Some(10),
            ..Default::default()
        };
        let df = read(f.path(), Format::Json, &opts).unwrap();
        // skip >= height: condition `skip < df.height()` is false → no-op
        assert_eq!(df.height(), 2);
    }
}
