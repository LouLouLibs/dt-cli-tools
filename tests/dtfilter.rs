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

#[test]
fn filter_eq() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtfilter().arg(f.path()).arg("--filter").arg("name=Alice").assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob").not());
}

#[test]
fn filter_gt() {
    let f = csv_file("name,value\nAlice,100\nBob,200\nCharlie,300\n");
    dtfilter().arg(f.path()).arg("--filter").arg("value>150").assert().success()
        .stdout(predicate::str::contains("Bob"))
        .stdout(predicate::str::contains("Charlie"));
}

#[test]
fn sort_desc() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtfilter().arg(f.path()).arg("--sort").arg("value:desc").assert().success();
}

#[test]
fn columns_select() {
    let f = csv_file("name,value,extra\nAlice,100,x\n");
    dtfilter().arg(f.path()).arg("--columns").arg("name,value").assert().success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("extra").not());
}

#[test]
fn csv_output() {
    let f = csv_file("name,value\nAlice,100\n");
    dtfilter().arg(f.path()).arg("--csv").assert().success()
        .stdout(predicate::str::contains("name,value"));
}

#[test]
fn head_tail_exclusive() {
    let f = csv_file("x\n1\n2\n");
    dtfilter().arg(f.path()).arg("--head").arg("1").arg("--tail").arg("1")
        .assert().code(2);
}
