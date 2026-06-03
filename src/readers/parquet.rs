use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

use crate::reader::ReadOptions;

pub fn read(path: &Path, opts: &ReadOptions) -> Result<DataFrame> {
    let file = std::fs::File::open(path)?;
    let mut df = ParquetReader::new(file).finish()?;

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
    use tempfile::NamedTempFile;

    #[test]
    fn read_parquet_roundtrip() {
        let s1 = Series::new("name".into(), &["Alice", "Bob"]);
        let s2 = Series::new("value".into(), &[100i64, 200]);
        let mut df = DataFrame::new(vec![s1.into_column(), s2.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".parquet").unwrap();
        let file = std::fs::File::create(f.path()).unwrap();
        ParquetWriter::new(file).finish(&mut df).unwrap();

        let result = read(f.path(), &ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
        assert_eq!(result.width(), 2);
    }
}
