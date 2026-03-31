use assert_cmd::Command;
use predicates::prelude::*;
use std::io::Write;
use tempfile::NamedTempFile;

fn dtcat() -> Command {
    Command::cargo_bin("dtcat").unwrap()
}

fn csv_file(content: &str) -> NamedTempFile {
    let mut f = NamedTempFile::with_suffix(".csv").unwrap();
    write!(f, "{}", content).unwrap();
    f.flush().unwrap();
    f
}

#[test]
fn shows_csv_data() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtcat().arg(f.path()).assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn schema_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--schema").assert().success()
        .stdout(predicate::str::contains("Column"))
        .stdout(predicate::str::contains("Type"));
}

#[test]
fn csv_output_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--csv").assert().success()
        .stdout(predicate::str::contains("name,value"));
}

#[test]
fn head_flag() {
    let f = csv_file("x\n1\n2\n3\n4\n5\n");
    dtcat().arg(f.path()).arg("--head").arg("2").assert().success();
}

#[test]
fn nonexistent_file_exits_1() {
    dtcat().arg("/tmp/does_not_exist_12345.csv").assert().failure();
}

#[test]
fn format_override() {
    let mut f = NamedTempFile::with_suffix(".txt").unwrap();
    write!(f, "a,b\n1,2\n").unwrap();
    f.flush().unwrap();
    dtcat().arg(f.path()).arg("--format").arg("csv").assert().success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn describe_flag() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtcat().arg(f.path()).arg("--describe").assert().success()
        .stdout(predicate::str::contains("count"))
        .stdout(predicate::str::contains("mean"));
}
