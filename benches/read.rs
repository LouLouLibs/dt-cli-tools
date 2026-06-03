use criterion::{Criterion, criterion_group, criterion_main};
use polars::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

use dtcore::format::Format;
use dtcore::reader::{ReadOptions, read_file};

fn generate_csv(n: usize) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    writeln!(f, "id,name,region,value").unwrap();
    let regions = ["East", "West", "North", "South"];
    for i in 0..n {
        writeln!(f, "{},name_{},{},{}", i, i, regions[i % 4], i * 100).unwrap();
    }
    f.flush().unwrap();
    f
}

fn generate_parquet(n: usize) -> NamedTempFile {
    let ids: Vec<i64> = (0..n as i64).collect();
    let names: Vec<String> = (0..n).map(|i| format!("name_{}", i)).collect();
    let regions: Vec<&str> = (0..n)
        .map(|i| ["East", "West", "North", "South"][i % 4])
        .collect();
    let values: Vec<i64> = (0..n).map(|i| i as i64 * 100).collect();

    let mut df = DataFrame::new(vec![
        Series::new("id".into(), &ids).into_column(),
        Series::new("name".into(), &names).into_column(),
        Series::new("region".into(), &regions).into_column(),
        Series::new("value".into(), &values).into_column(),
    ])
    .unwrap();

    let f = NamedTempFile::with_suffix(".parquet").unwrap();
    let file = std::fs::File::create(f.path()).unwrap();
    ParquetWriter::new(file).finish(&mut df).unwrap();
    f
}

fn generate_arrow(n: usize) -> NamedTempFile {
    let ids: Vec<i64> = (0..n as i64).collect();
    let values: Vec<i64> = (0..n).map(|i| i as i64 * 100).collect();

    let mut df = DataFrame::new(vec![
        Series::new("id".into(), &ids).into_column(),
        Series::new("value".into(), &values).into_column(),
    ])
    .unwrap();

    let f = NamedTempFile::with_suffix(".arrow").unwrap();
    let file = std::fs::File::create(f.path()).unwrap();
    IpcWriter::new(file).finish(&mut df).unwrap();
    f
}

fn generate_ndjson(n: usize) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".ndjson").unwrap();
    let regions = ["East", "West", "North", "South"];
    for i in 0..n {
        writeln!(
            f,
            r#"{{"id":{},"name":"name_{}","region":"{}","value":{}}}"#,
            i,
            i,
            regions[i % 4],
            i * 100
        )
        .unwrap();
    }
    f.flush().unwrap();
    f
}

fn bench_read(c: &mut Criterion) {
    let opts = ReadOptions::default();

    for &size in &[1_000, 10_000, 100_000] {
        let csv = generate_csv(size);
        let parquet = generate_parquet(size);
        let arrow = generate_arrow(size);
        let ndjson = generate_ndjson(size);

        let mut group = c.benchmark_group(format!("read_{size}"));

        group.bench_function("csv", |b| {
            b.iter(|| read_file(csv.path(), Format::Csv, &opts).unwrap())
        });

        group.bench_function("parquet", |b| {
            b.iter(|| read_file(parquet.path(), Format::Parquet, &opts).unwrap())
        });

        group.bench_function("arrow", |b| {
            b.iter(|| read_file(arrow.path(), Format::Arrow, &opts).unwrap())
        });

        group.bench_function("ndjson", |b| {
            b.iter(|| read_file(ndjson.path(), Format::Ndjson, &opts).unwrap())
        });

        group.finish();
    }
}

criterion_group!(benches, bench_read);
criterion_main!(benches);
