use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtdiff() -> Command {
    Command::cargo_bin("dtdiff").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

// ─── Positional mode ───

#[test]
fn no_diff_exits_0() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nAlice,100\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn positional_diff_exits_1() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nBob,200\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().code(1);
}

#[test]
fn positional_added_row() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nAlice,100\nBob,200\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().code(1)
        .stdout(predicate::str::contains("Added: 1"));
}

#[test]
fn positional_removed_row() {
    let a = csv_file("name,value\nAlice,100\nBob,200\n");
    let b = csv_file("name,value\nAlice,100\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().code(1)
        .stdout(predicate::str::contains("Removed: 1"));
}

// ─── Key-based mode ───

#[test]
fn keyed_diff_modified() {
    let a = csv_file("id,name\n1,Alice\n2,Bob\n");
    let b = csv_file("id,name\n1,Alice\n2,Robert\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Modified: 1"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn keyed_diff_added_and_removed() {
    let a = csv_file("id,name\n1,Alice\n2,Bob\n");
    let b = csv_file("id,name\n1,Alice\n3,Charlie\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Added: 1"))
        .stdout(predicate::str::contains("Removed: 1"));
}

#[test]
fn keyed_no_diff() {
    let a = csv_file("id,name\n1,Alice\n2,Bob\n");
    let b = csv_file("id,name\n2,Bob\n1,Alice\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id")
        .assert().success()
        .stdout(predicate::str::contains("No differences"));
}

// ─── Composite keys ───

#[test]
fn composite_key() {
    let a = csv_file("date,ticker,price\n2024-01-01,AAPL,150\n2024-01-01,GOOG,140\n");
    let b = csv_file("date,ticker,price\n2024-01-01,AAPL,150\n2024-01-01,GOOG,145\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("date,ticker")
        .assert().code(1)
        .stdout(predicate::str::contains("Modified: 1"))
        .stdout(predicate::str::contains("GOOG"));
}

// ─── Float tolerance ───

#[test]
fn tolerance_suppresses_small_diff() {
    let a = csv_file("id,price\n1,150.000\n");
    let b = csv_file("id,price\n1,150.005\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id").arg("--tolerance").arg("0.01")
        .assert().success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn tolerance_reports_large_diff() {
    let a = csv_file("id,price\n1,150.0\n");
    let b = csv_file("id,price\n1,155.0\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id").arg("--tolerance").arg("0.01")
        .assert().code(1)
        .stdout(predicate::str::contains("Modified: 1"));
}

// ─── Parquet ───

#[test]
fn parquet_keyed_diff() {
    dtdiff().arg("tests/fixtures/old.parquet").arg("tests/fixtures/new.parquet")
        .arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Added: 1"))
        .stdout(predicate::str::contains("Removed: 1"));
}

#[test]
fn parquet_no_diff() {
    dtdiff().arg("tests/fixtures/data.parquet").arg("tests/fixtures/data.parquet")
        .assert().success()
        .stdout(predicate::str::contains("No differences"));
}

// ─── Arrow/IPC ───

#[test]
fn arrow_keyed_diff() {
    dtdiff().arg("tests/fixtures/old.arrow").arg("tests/fixtures/new.arrow")
        .arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Added: 1"))
        .stdout(predicate::str::contains("Removed: 1"));
}

// ─── JSON ───

#[test]
fn json_keyed_diff() {
    dtdiff().arg("tests/fixtures/old.json").arg("tests/fixtures/new.json")
        .arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Modified: 1"));
}

// ─── NDJSON ───

#[test]
fn ndjson_keyed_diff() {
    dtdiff().arg("tests/fixtures/old.ndjson").arg("tests/fixtures/new.ndjson")
        .arg("--key").arg("id")
        .assert().code(1)
        .stdout(predicate::str::contains("Modified: 1"));
}

// ─── Output formats ───

#[test]
fn json_output() {
    let a = csv_file("id,val\n1,a\n");
    let b = csv_file("id,val\n1,b\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id").arg("--json")
        .assert().code(1)
        .stdout(predicate::str::contains("\"modified\""));
}

#[test]
fn csv_output() {
    let a = csv_file("id,val\n1,a\n");
    let b = csv_file("id,val\n1,b\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id").arg("--csv")
        .assert().code(1)
        .stdout(predicate::str::contains("_status"));
}

#[test]
fn no_color_flag() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nBob,200\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--no-color")
        .assert().code(1);
}

// ─── Excel ───

#[test]
fn excel_keyed_diff() {
    dtdiff().arg("demo/old.xlsx").arg("demo/new.xlsx").arg("--key").arg("ID")
        .assert().code(1)
        .stdout(predicate::str::contains("Added: 1"))
        .stdout(predicate::str::contains("Removed: 1"))
        .stdout(predicate::str::contains("Modified: 3"));
}

#[test]
fn excel_no_diff() {
    dtdiff().arg("demo/old.xlsx").arg("demo/old.xlsx")
        .assert().success()
        .stdout(predicate::str::contains("No differences"));
}
