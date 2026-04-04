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

// ─── Basic viewing ───

#[test]
fn shows_csv_data() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtcat().arg(f.path()).assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Bob"));
}

#[test]
fn header_only_csv() {
    let f = csv_file("name,value\n");
    dtcat().arg(f.path()).assert().success()
        .stdout(predicate::str::contains("no data rows"));
}

#[test]
fn nonexistent_file_exits_1() {
    dtcat().arg("/tmp/does_not_exist_12345.csv").assert().failure();
}

// ─── Modes ───

#[test]
fn schema_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--schema").assert().success()
        .stdout(predicate::str::contains("Column"))
        .stdout(predicate::str::contains("Type"));
}

#[test]
fn describe_flag() {
    let f = csv_file("name,value\nAlice,100\nBob,200\n");
    dtcat().arg(f.path()).arg("--describe").assert().success()
        .stdout(predicate::str::contains("count"))
        .stdout(predicate::str::contains("mean"));
}

#[test]
fn info_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--info").assert().success()
        .stdout(predicate::str::contains("File:"));
}

// ─── Row windowing ───

#[test]
fn head_flag() {
    let f = csv_file("x\n1\n2\n3\n4\n5\n");
    dtcat().arg(f.path()).arg("--head").arg("2").assert().success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("3").not());
}

#[test]
fn tail_flag() {
    let f = csv_file("x\n1\n2\n3\n4\n5\n");
    dtcat().arg(f.path()).arg("--tail").arg("2").assert().success()
        .stdout(predicate::str::contains("4"))
        .stdout(predicate::str::contains("5"));
}

#[test]
fn head_and_tail_combined() {
    let f = csv_file("x\n1\n2\n3\n4\n5\n6\n7\n8\n9\n10\n");
    dtcat().arg(f.path()).arg("--head").arg("2").arg("--tail").arg("2")
        .assert().success()
        .stdout(predicate::str::contains("1"))
        .stdout(predicate::str::contains("2"))
        .stdout(predicate::str::contains("9"))
        .stdout(predicate::str::contains("10"));
}

// ─── Output format ───

#[test]
fn csv_output_flag() {
    let f = csv_file("name,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--csv").assert().success()
        .stdout(predicate::str::contains("name,value"));
}

// ─── Format detection ───

#[test]
fn format_override() {
    let mut f = NamedTempFile::with_suffix(".txt").unwrap();
    write!(f, "a,b\n1,2\n").unwrap();
    f.flush().unwrap();
    dtcat().arg(f.path()).arg("--format").arg("csv").assert().success()
        .stdout(predicate::str::contains("1"));
}

#[test]
fn tsv_detection() {
    let mut f = NamedTempFile::with_suffix(".tsv").unwrap();
    write!(f, "name\tvalue\nAlice\t100\n").unwrap();
    f.flush().unwrap();
    dtcat().arg(f.path()).assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("100"));
}

// ─── Skip rows ───

#[test]
fn skip_metadata_rows() {
    let f = csv_file("meta1\nmeta2\nname,value\nAlice,100\n");
    dtcat().arg(f.path()).arg("--skip").arg("2").assert().success()
        .stdout(predicate::str::contains("Alice"));
}

// ─── All flag ───

#[test]
fn all_flag_shows_every_row() {
    // 60 rows > threshold of 50, so without --all we'd get head+tail
    let mut content = String::from("x\n");
    for i in 1..=60 {
        content.push_str(&format!("{}\n", i));
    }
    let f = csv_file(&content);
    // With --all, row 30 should appear (it would be omitted in head25+tail25)
    dtcat().arg(f.path()).arg("--all").assert().success()
        .stdout(predicate::str::contains("| 30 "));
}

// ─── Sample ───

#[test]
fn sample_returns_n_rows() {
    let out = dtcat().arg("demo/sales.csv").arg("--sample").arg("5").arg("--csv")
        .assert().success();
    let stdout = String::from_utf8(out.get_output().stdout.clone()).unwrap();
    let lines: Vec<&str> = stdout.trim().lines().collect();
    assert_eq!(lines.len(), 6, "expected header + 5 rows, got {}", lines.len());
}

#[test]
fn sample_ge_total_returns_all() {
    let f = csv_file("x\n1\n2\n3\n");
    dtcat().arg(f.path()).arg("--sample").arg("100").arg("--csv")
        .assert().success();
}

#[test]
fn sample_conflicts_with_head() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--head").arg("1")
        .assert().code(2);
}

#[test]
fn sample_conflicts_with_tail() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--tail").arg("1")
        .assert().code(2);
}

#[test]
fn sample_conflicts_with_all() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--all")
        .assert().code(2);
}

#[test]
fn sample_conflicts_with_schema() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--schema")
        .assert().code(2);
}

#[test]
fn sample_conflicts_with_describe() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--describe")
        .assert().code(2);
}

#[test]
fn sample_conflicts_with_info() {
    let f = csv_file("x\n1\n");
    dtcat().arg(f.path()).arg("--sample").arg("1").arg("--info")
        .assert().code(2);
}

// ─── Parquet ───

#[test]
fn parquet_view() {
    dtcat().arg("tests/fixtures/data.parquet").assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"));
}

#[test]
fn parquet_schema() {
    dtcat().arg("tests/fixtures/data.parquet").arg("--schema").assert().success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("value"));
}

// ─── Arrow/IPC ───

#[test]
fn arrow_view() {
    dtcat().arg("tests/fixtures/data.arrow").assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"));
}

#[test]
fn arrow_schema() {
    dtcat().arg("tests/fixtures/data.arrow").arg("--schema").assert().success()
        .stdout(predicate::str::contains("name"))
        .stdout(predicate::str::contains("value"));
}

// ─── JSON ───

#[test]
fn json_view() {
    dtcat().arg("tests/fixtures/data.json").assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"));
}

// ─── NDJSON ───

#[test]
fn ndjson_view() {
    dtcat().arg("tests/fixtures/data.ndjson").assert().success()
        .stdout(predicate::str::contains("Alice"))
        .stdout(predicate::str::contains("Charlie"));
}

// ─── Excel ───

#[test]
fn excel_view() {
    dtcat().arg("demo/sales.xlsx").assert().success()
        .stdout(predicate::str::contains("Revenue"));
}

#[test]
fn excel_schema() {
    dtcat().arg("demo/sales.xlsx").arg("--schema").assert().success()
        .stdout(predicate::str::contains("Column"))
        .stdout(predicate::str::contains("Revenue"));
}

#[test]
fn excel_info() {
    dtcat().arg("demo/sales.xlsx").arg("--info").assert().success()
        .stdout(predicate::str::contains("Excel"))
        .stdout(predicate::str::contains("Sheet1"));
}
