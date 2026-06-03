use criterion::{Criterion, criterion_group, criterion_main};
use polars::prelude::*;

use dtcore::diff::{DiffOptions, SheetSource, diff_keyed, diff_positional};

fn source(name: &str) -> SheetSource {
    SheetSource {
        file_name: name.into(),
        sheet_name: "Sheet1".into(),
    }
}

fn make_df(n: usize, offset: i64) -> DataFrame {
    let ids: Vec<i64> = (offset..offset + n as i64).collect();
    let names: Vec<String> = ids.iter().map(|i| format!("name_{}", i)).collect();
    let values: Vec<i64> = ids.iter().map(|i| i * 100).collect();

    DataFrame::new(vec![
        Series::new("id".into(), &ids).into_column(),
        Series::new("name".into(), &names).into_column(),
        Series::new("value".into(), &values).into_column(),
    ])
    .unwrap()
}

fn bench_diff(c: &mut Criterion) {
    let opts_positional = DiffOptions::default();
    let opts_keyed = DiffOptions {
        key_columns: vec!["id".into()],
        tolerance: None,
    };

    for &size in &[1_000, 10_000, 100_000] {
        let df_a = make_df(size, 0);
        // 10% of rows differ (shifted by 10% of size)
        let shift = (size / 10) as i64;
        let df_b = make_df(size, shift);

        let mut group = c.benchmark_group(format!("diff_{size}"));

        group.bench_function("positional", |b| {
            b.iter(|| {
                diff_positional(&df_a, &df_b, &opts_positional, source("a"), source("b")).unwrap()
            })
        });

        group.bench_function("keyed", |b| {
            b.iter(|| diff_keyed(&df_a, &df_b, &opts_keyed, source("a"), source("b")).unwrap())
        });

        group.finish();
    }
}

criterion_group!(benches, bench_diff);
criterion_main!(benches);
