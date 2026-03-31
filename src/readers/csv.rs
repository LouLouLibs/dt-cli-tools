use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::reader::ReadOptions;

pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let separator = opts.separator.unwrap_or_else(|| {
        crate::format::detect_csv_delimiter(path).unwrap_or(b',')
    });

    let reader = CsvReadOptions::default()
        .with_has_header(true)
        .with_skip_rows(opts.skip_rows.unwrap_or(0))
        .with_parse_options(
            CsvParseOptions::default().with_separator(separator),
        )
        .try_into_reader_with_file_path(Some(path.into()))?;

    let df = reader.finish()?;
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

    #[test]
    fn read_basic_csv() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "name,value\nAlice,100\nBob,200\n").unwrap();
        f.flush().unwrap();

        let df = read(f.path(), &default_opts()).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn read_tsv() {
        let mut f = NamedTempFile::with_suffix(".tsv").unwrap();
        write!(f, "a\tb\n1\t2\n3\t4\n").unwrap();
        f.flush().unwrap();

        let opts = ReadOptions { separator: Some(b'\t'), ..Default::default() };
        let df = read(f.path(), &opts).unwrap();
        assert_eq!(df.height(), 2);
        assert_eq!(df.width(), 2);
    }

    #[test]
    fn read_with_skip() {
        let mut f = NamedTempFile::with_suffix(".csv").unwrap();
        write!(f, "metadata line\nname,value\nAlice,100\n").unwrap();
        f.flush().unwrap();

        let opts = ReadOptions { skip_rows: Some(1), ..Default::default() };
        let df = read(f.path(), &opts).unwrap();
        assert_eq!(df.height(), 1);
        let names: Vec<String> = df.get_column_names().iter().map(|s| s.to_string()).collect();
        assert_eq!(names, vec!["name", "value"]);
    }
}
