use anyhow::Result;
use polars::prelude::*;
use std::io::Write as IoWrite;
use std::path::Path;

use crate::format::Format;

pub fn write(df: &mut DataFrame, path: Option<&Path>, format: Format) -> Result<()> {
    let json_format = match format {
        Format::Ndjson => JsonFormat::JsonLines,
        _ => JsonFormat::Json,
    };

    match path {
        Some(p) => {
            let file = std::fs::File::create(p)?;
            JsonWriter::new(file)
                .with_json_format(json_format)
                .finish(df)?;
        }
        None => {
            let mut buf = Vec::new();
            JsonWriter::new(&mut buf)
                .with_json_format(json_format)
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
    fn write_json_roundtrip() {
        let s = Series::new("x".into(), &[1i64, 2]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".json").unwrap();
        write(&mut df, Some(f.path()), Format::Json).unwrap();

        let result = crate::readers::json::read(f.path(), Format::Json, &crate::reader::ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
    }

    #[test]
    fn write_ndjson_roundtrip() {
        let s = Series::new("x".into(), &[1i64, 2]);
        let mut df = DataFrame::new(vec![s.into_column()]).unwrap();

        let f = NamedTempFile::with_suffix(".ndjson").unwrap();
        write(&mut df, Some(f.path()), Format::Ndjson).unwrap();

        let result = crate::readers::json::read(f.path(), Format::Ndjson, &crate::reader::ReadOptions::default()).unwrap();
        assert_eq!(result.height(), 2);
    }
}
