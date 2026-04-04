use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

pub fn write(df: &mut DataFrame, path: Option<&Path>) -> Result<()> {
    let path = path.ok_or_else(|| anyhow::anyhow!("--convert parquet requires -o PATH"))?;
    let file = std::fs::File::create(path)?;
    ParquetWriter::new(file).finish(df)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn write_parquet_roundtrip() {
        let s1 = Series::new("name".into(), &["Alice", "Bob"]);
        let s2 = Series::new("value".into(), &[100i64, 200]);
        let mut df = DataFrame::new(vec![s1.into_column(), s2.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".parquet").unwrap();
        write(&mut df, Some(f.path())).unwrap();

        let result = crate::readers::parquet::read(f.path(), &crate::reader::ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn write_parquet_no_path_errors() {
        let s = Series::new("x".into(), &[1i64]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();
        assert!(write(&mut df, None).is_err());
    }
}
