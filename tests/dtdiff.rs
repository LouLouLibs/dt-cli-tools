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

#[test]
fn no_diff_exits_0() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nAlice,100\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().success()
        .stdout(predicate::str::contains("No differences"));
}

#[test]
fn diff_exits_1() {
    let a = csv_file("name,value\nAlice,100\n");
    let b = csv_file("name,value\nBob,200\n");
    dtdiff().arg(a.path()).arg(b.path()).assert().code(1);
}

#[test]
fn keyed_diff() {
    let a = csv_file("id,name\n1,Alice\n2,Bob\n");
    let b = csv_file("id,name\n1,Alice\n2,Robert\n");
    dtdiff().arg(a.path()).arg(b.path()).arg("--key").arg("id")
        .assert().code(1);
}

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
