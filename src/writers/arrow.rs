use anyhow::Result;
use polars::prelude::*;
use std::path::Path;

pub fn write(df: &mut DataFrame, path: Option<&Path>) -> Result<()> {
    let path = path.ok_or_else(|| anyhow::anyhow!("--convert arrow requires -o PATH"))?;
    let file = std::fs::File::create(path)?;
    IpcWriter::new(file).finish(df)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn write_arrow_roundtrip() {
        let s = Series::new("x".into(), &[1i64, 2, 3]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".arrow").unwrap();
        write(&mut df, Some(f.path())).unwrap();

        let result =
            crate::readers::arrow::read(f.path(), &crate::reader::ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 3);
    }

    #[test]
    fn write_arrow_no_path_errors() {
        let s = Series::new("x".into(), &[1i64]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();
        assert!(write(&mut df, None).is_err());
    }
}
