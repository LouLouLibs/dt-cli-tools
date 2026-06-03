use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtfilter() -> Command {
    Command::cargo_bin("dtfilter").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

const DATA: &str = "name,value\nAlice,100\nBob,200\nCharlie,300\n";

// ─── Equality ───

#[test]
fn filter_eq() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name=Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

#[test]
fn filter_neq() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name!=Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

// ─── Numeric comparisons ───

#[test]
fn filter_gt() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value>150")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

#[test]
fn filter_lt() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value<200")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

#[test]
fn filter_gte() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value>=200")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

#[test]
fn filter_lte() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value<=200")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie").not());
}

// ─── String matching ───

#[test]
fn filter_contains() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name~ob")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Alice").not());
}

#[test]
fn filter_not_contains() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name!~ob")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Bob").not());
}

// ─── Multiple filters (AND) ───

#[test]
fn multiple_filters_and() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("value>=200")
        .arg("--filter")
        .arg("value<=300")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

// ─── Sort ───

#[test]
fn sort_desc() {
    let f = csv_file(DATA);
    let out = dtfilter()
        .arg(f.path())
        .arg("--sort")
        .arg("value:desc")
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let charlie_pos = stdout.find("Charlie").unwrap();
    let alice_pos = stdout.find("Alice").unwrap();
    assert!(
        charlie_pos < alice_pos,
        "Charlie (300) should appear before Alice (100) in desc sort"
    );
}

#[test]
fn sort_asc() {
    let f = csv_file(DATA);
    let out = dtfilter()
        .arg(f.path())
        .arg("--sort")
        .arg("value")
        .assert()
        .success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let alice_pos = stdout.find("Alice").unwrap();
    let charlie_pos = stdout.find("Charlie").unwrap();
    assert!(
        alice_pos < charlie_pos,
        "Alice (100) should appear before Charlie (300) in asc sort"
    );
}

// ─── Column selection ───

#[test]
fn columns_select() {
    let f = csv_file("name,value,extra\nAlice,100,x\n");
    dtfilter()
        .arg(f.path())
        .arg("--columns")
        .arg("name,value")
        .assert()
        .success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("extra").not());
}

// ─── Limit ───

#[test]
fn limit_output() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--sort")
        .arg("value:desc")
        .arg("--limit")
        .arg("1")
        .assert()
        .success()
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

// ─── Output format ───

#[test]
fn csv_output() {
    let f = csv_file("name,value\nAlice,100\n");
    dtfilter()
        .arg(f.path())
        .arg("--csv")
        .assert()
        .success()
        .stdout(predicate::str::contains("name,value"));
}

// ─── Windowing ───

#[test]
fn head_before_filter() {
    let f = csv_file("name,value\nAlice,100\nBob,200\nCharlie,300\n");
    dtfilter()
        .arg(f.path())
        .arg("--head")
        .arg("2")
        .arg("--filter")
        .arg("value>150")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie").not());
}

#[test]
fn head_tail_exclusive() {
    let f = csv_file("x\n1\n2\n");
    dtfilter()
        .arg(f.path())
        .arg("--head")
        .arg("1")
        .arg("--tail")
        .arg("1")
        .assert()
        .code(2);
}

// ─── Excel ───

#[test]
fn filter_excel() {
    dtfilter()
        .arg("demo/sales.xlsx")
        .arg("--filter")
        .arg("Region=East")
        .assert()
        .success()
        .stdout(predicate::str::contains("East"))
        .stdout(predicate::str::contains("West").not());
}

// ─── Parquet ───

#[test]
fn filter_parquet() {
    dtfilter()
        .arg("tests/fixtures/data.parquet")
        .arg("--filter")
        .arg("value>150")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

// ─── Arrow/IPC ───

#[test]
fn filter_arrow() {
    dtfilter()
        .arg("tests/fixtures/data.arrow")
        .arg("--filter")
        .arg("value>150")
        .assert()
        .success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"))
        .stdout(predicate::str::contains("Alice").not());
}

// ─── JSON ───

#[test]
fn filter_json() {
    dtfilter()
        .arg("tests/fixtures/data.json")
        .arg("--filter")
        .arg("name=Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

// ─── NDJSON ───

#[test]
fn filter_ndjson() {
    dtfilter()
        .arg("tests/fixtures/data.ndjson")
        .arg("--filter")
        .arg("name=Alice")
        .assert()
        .success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

// ─── Edge cases ───

#[test]
fn filter_no_matches() {
    let f = csv_file(DATA);
    dtfilter()
        .arg(f.path())
        .arg("--filter")
        .arg("name=Nobody")
        .assert()
        .success()
        .stderr(predicate::str::contains("0 rows"));
}
