use criterion::{Criterion, criterion_group, criterion_main};
use polars::prelude::*;

use dtcore::filter::{
    FilterExpr, FilterOp, FilterOptions, SortSpec, apply_filters, filter_pipeline,
};

fn make_df(n: usize) -> DataFrame {
    let ids: Vec<i64> = (0..n as i64).collect();
    let regions: Vec<&str> = (0..n)
        .map(|i| ["East", "West", "North", "South"][i % 4])
        .collect();
    let values: Vec<i64> = (0..n).map(|i| i as i64 * 100).collect();
    let names: Vec<String> = (0..n).map(|i| format!("name_{}", i)).collect();

    DataFrame::new(vec![
        Series::new("id".into(), &ids).into_column(),
        Series::new("region".into(), &regions).into_column(),
        Series::new("value".into(), &values).into_column(),
        Series::new("name".into(), &names).into_column(),
    ])
    .unwrap()
}

fn bench_filter(c: &mut Criterion) {
    for &size in &[1_000, 10_000, 100_000] {
        let df = make_df(size);

        let mut group = c.benchmark_group(format!("filter_{size}"));

        // Equality filter
        let eq_expr = vec![FilterExpr {
            column: "region".into(),
            op: FilterOp::Eq,
            value: "East".into(),
        }];
        group.bench_function("eq", |b| b.iter(|| apply_filters(&df, &eq_expr).unwrap()));

        // Numeric comparison
        let gt_expr = vec![FilterExpr {
            column: "value".into(),
            op: FilterOp::Gt,
            value: (size as i64 * 50).to_string(),
        }];
        group.bench_function("gt", |b| b.iter(|| apply_filters(&df, &gt_expr).unwrap()));

        // Contains (string scan)
        let contains_expr = vec![FilterExpr {
            column: "name".into(),
            op: FilterOp::Contains,
            value: "42".into(),
        }];
        group.bench_function("contains", |b| {
            b.iter(|| apply_filters(&df, &contains_expr).unwrap())
        });

        // Full pipeline: filter + sort + limit
        let pipeline_opts = FilterOptions {
            filters: vec![FilterExpr {
                column: "region".into(),
                op: FilterOp::Eq,
                value: "East".into(),
            }],
            sort: Some(SortSpec {
                column: "value".into(),
                descending: true,
            }),
            limit: Some(10),
            cols: None,
            head: None,
            tail: None,
        };
        group.bench_function("pipeline", |b| {
            b.iter(|| filter_pipeline(df.clone(), &pipeline_opts).unwrap())
        });

        group.finish();
    }
}

criterion_group!(benches, bench_filter);
criterion_main!(benches);
