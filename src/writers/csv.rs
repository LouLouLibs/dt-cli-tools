use anyhow::Result;
use polars::prelude::*;
use std::io::Write;
use std::path::Path;

use crate::format::Format;

pub fn write(df: &mut DataFrame, path: Option<&Path>, format: Format) -> Result<()> {
    let separator = match format {
        Format::Tsv => b'\t',
        _ => b',',
    };

    match path {
        Some(p) => {
            let file = std::fs::File::create(p)?;
            CsvWriter::new(file)
                .with_separator(separator)
                .finish(df)?;
        }
        None => {
            let mut buf = Vec::new();
            CsvWriter::new(&mut buf)
                .with_separator(separator)
                .finish(df)?;
            std::io::stdout().write_all(&buf)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn write_csv_roundtrip() {
        let s1 = Series::new("name".into(), &["Alice", "Bob"]);
        let s2 = Series::new("value".into(), &[100i64, 200]);
        let mut df = DataFrame::new(vec![s1.into_column(), s2.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".csv").unwrap();
        write(&mut df, Some(f.path()), Format::Csv).unwrap();

        let result = crate::readers::csv::read(f.path(), &crate::reader::ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
        assert_eq!(result.get_column_names(), df.get_column_names());
    }

    #[test]
    fn write_tsv_uses_tab() {
        let s = Series::new("x".into(), &[1i64]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".tsv").unwrap();
        write(&mut df, Some(f.path()), Format::Tsv).unwrap();

        let content = std::fs::read_to_string(f.path()).unwrap();
        assert!(!content.contains(','));
    }
}
