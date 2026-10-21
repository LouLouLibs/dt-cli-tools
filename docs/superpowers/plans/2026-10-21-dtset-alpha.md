# dtset (alpha) Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port `xlset` from xl-cli-tools into dt-cli-tools as `dtset`, an .xlsx/.xlsm cell writer behind an off-by-default `alpha` feature. Then retire xl-cli-tools and its Claude skills.

**Architecture:** A new library module `dtcore::edit` (gated `#[cfg(feature = "alpha")]`) holds the A1 cell parser (`edit::cell`) and the umya-spreadsheet writer (`edit::xlsx`). `src/bin/dtset.rs` is a thin clap front end with `required-features = ["alpha"]`, so default builds and release binaries stay read-only. Most of the code is copied from `~/Dropbox/projects_claude/xl-cli-tool` (`src/cell.rs`, `src/writer.rs`, `src/bin/xlset.rs`, `tests/test_xlset.rs`, `tests/common/mod.rs`). There are two behavioural changes: saves are atomic, and the format is restricted to xlsx/xlsm.

**Tech Stack:** Rust 2024, clap 4, anyhow, umya-spreadsheet 2 (optional), rust_xlsxwriter 0.82 (dev), assert_cmd/predicates/tempfile.

**Spec:** https://github.com/LouLouLibs/dt-cli-tools/issues/2

## Global Constraints

- `cargo build`, `cargo test` and `cargo clippy` with **no features** must behave exactly as they do on `main`, and must not compile umya-spreadsheet.
- Feature name: `alpha`. Dependencies: `umya-spreadsheet = { version = "2", optional = true }` and `tempfile = { version = "3", optional = true }` (it also stays in dev-deps); `alpha = ["dep:umya-spreadsheet", "dep:tempfile"]`.
- Supported edit formats: `.xlsx`, `.xlsm` only (case-insensitive extension). Every other format errors.
- Exit codes match dtcat/dtfilter: 0 success, 1 runtime error, 2 invalid arguments. Error prefix `dtset: `.
- Success message on stderr: `dtset: updated {n} cells in {sheet} ({file_name})`.
- `--sheet` uses dt conventions: `Option<String>`, by name or 0-based index; the default is the first sheet.
- Version bump: `0.2.0` → `0.3.0`.

## Review Focus

1. **Failed save on an in-place edit:** if the write fails, the original file must survive byte-for-byte. Covered by the atomic-save test in Task 2.
2. **Non-xlsx input (`.csv`, `.parquet`, `.xls`, `.ods`, `.xlsb`):** a user will try `dtset data.csv A1=1`. Expect exit 2 with an error naming `.xlsx/.xlsm`, and the file left untouched. Covered in Task 3.
3. **`--output` with a non-xlsx extension (`-o out.csv`):** without a check, umya would write xlsx bytes into a `.csv` file. Must exit 2. Covered in Task 3.
4. **Formulas and untouched cells survive an edit:** this is the point of using umya. Covered in Task 2.
5. **Values containing `=` or `,` (`A1=a=b`, `--from` with a quoted `"x, y"`):** the full value must be stored. Covered in Task 3.

---

### Task 1: `alpha` feature and `edit::cell` parser

**Files:**
- Modify: `Cargo.toml` (add `[features]`, the optional dep, the `rust_xlsxwriter = "0.82"` dev-dep, and the `[[bin]] dtset` with `required-features = ["alpha"]`; bump version to `0.3.0`)
- Modify: `src/lib.rs` (add `#[cfg(feature = "alpha")] pub mod edit;`)
- Create: `src/edit/mod.rs` (`pub mod cell; pub mod xlsx;`)
- Create: `src/edit/cell.rs` (copied verbatim from `xl-cli-tool/src/cell.rs`, including its `#[cfg(test)]` module)
- Create: `src/bin/dtset.rs` (placeholder `fn main() {}` so the bin target resolves; Task 3 replaces it)
- Create: `src/edit/xlsx.rs` (empty for now; Task 2 fills it)

**Interfaces:**
- Produces: `dtcore::edit::cell::{CellRef, CellValue, CellAssignment, parse_cell_ref(&str) -> Result<CellRef, String>, infer_value(&str) -> CellValue, parse_assignment(&str) -> Result<CellAssignment, String>}`. Field names (`col`, `row` 0-based, `label`; `cell`, `value`) are unchanged from xl-cli-tool.

- [ ] **Step 1:** Make the Cargo.toml and lib.rs changes and copy `cell.rs` (its unit tests come with it).
- [ ] **Step 2:** Run `cargo test --features alpha edit::cell`. Expected: all copied cell tests PASS.
- [ ] **Step 3:** Run `cargo tree -e normal | grep -c umya`. Expected: `0` (no features means no umya). Run `cargo test` and `cargo clippy --all-targets`. Expected: same results as `main`.
- [ ] **Step 4:** Commit `feat(alpha): add edit::cell A1 parser behind alpha feature (#2)`.

### Task 2: `edit::xlsx` writer with atomic save

**Files:**
- Modify: `src/edit/xlsx.rs`
- Create: `tests/common/xlsx.rs` (copy `create_simple` and `create_multi_sheet` from `xl-cli-tool/tests/common/mod.rs`, and add `create_with_formula(path)`: sheet "Data", A1=2, A2=3, A3 formula `=A1+A2`)
- Test: unit tests in `src/edit/xlsx.rs`

**Interfaces:**
- Consumes: `edit::cell::{CellAssignment, CellValue}`.
- Produces: `pub fn write_cells(input: &Path, output: &Path, sheet: Option<&str>, assignments: &[CellAssignment]) -> anyhow::Result<(usize, String)>` returns (cells updated, resolved sheet name). It also produces `pub fn check_editable(path: &Path) -> Result<(), String>`, which returns `Ok` for `.xlsx`/`.xlsm` (case-insensitive) and otherwise `Err("unsupported format '.{ext}' — dtset edits .xlsx/.xlsm only")`, with a dedicated message for `.xls` suggesting conversion. Task 3 maps `Err` to exit 2.

- [ ] **Step 1: Write failing unit tests.**
  - `check_editable_accepts_xlsx_xlsm`: `foo.xlsx`, `foo.XLSM` → Ok.
  - `check_editable_rejects_others`: `foo.csv`, `foo.parquet`, `foo.xls`, `foo.ods`, `foo.xlsb`, `foo` (no extension) → Err; the `.xls` error contains `"convert"`.
  - `write_preserves_formula`: `create_with_formula`, then `write_cells` sets A1=10. Re-open with `umya_spreadsheet::reader::xlsx::read` and assert that A3's formula is `"A1+A2"` and A1's value is `"10"`.
  - `failed_save_leaves_original`: copy a fixture, then call `write_cells` with `output` = a path inside a non-existent directory. Assert it returns Err and that the input's bytes equal the bytes read before the call.
  - `in_place_save_leaves_no_temp_files`: an in-place write succeeds, and the directory then holds exactly one file.
  - `unknown_sheet_lists_available`: `create_multi_sheet`, `sheet = Some("Nope")` → Err containing `Revenue, Expenses, Summary`.
- [ ] **Step 2:** Run `cargo test --features alpha edit::xlsx`. Expected: FAIL (`write_cells` not defined).
- [ ] **Step 3: Implement.** Port `write_cells`, `resolve_sheet_index`, `apply_assignment`, `date_to_serial` and `is_leap_year` (with their unit tests) from `xl-cli-tool/src/writer.rs`. Changes:
  - `sheet: Option<&str>` (`None` → index 0).
  - Call `check_editable` on both `input` and `output`.
  - Save by writing to a sibling temp file (`tempfile::NamedTempFile::new_in(output.parent())` with suffix `.xlsx`, so `tempfile` moves from dev-deps to an optional dep under `alpha`), then `persist(output)`. Map errors with `with_context`.
- [ ] **Step 4:** Run `cargo test --features alpha edit::xlsx`. Expected: PASS.
- [ ] **Step 5:** Commit `feat(alpha): add atomic xlsx cell writer (#2)`.

### Task 3: `dtset` binary and integration tests

**Files:**
- Modify: `src/bin/dtset.rs` (port of `xl-cli-tool/src/bin/xlset.rs`)
- Create: `tests/dtset.rs` (first line `#![cfg(feature = "alpha")]`, then `mod common;`. Port all 11 tests from `xl-cli-tool/tests/test_xlset.rs`, using `dtset`/`dtcat` and replacing `"xlset:"` with `"dtset:"`.)
- Create: `tests/common/mod.rs` (`pub mod xlsx;`). If no existing test already has a `tests/common/` dir, this is new.

**Interfaces:**
- Consumes: `edit::cell::{parse_assignment, parse_cell_ref, CellAssignment}`, `edit::xlsx::{write_cells, check_editable}`.
- CLI: `dtset <FILE> [ASSIGNMENTS]... [--sheet S] [-o|--output PATH] [--from FILE|-]`, plus `#[command(version)]`.

- [ ] **Step 1: Write failing tests.** Port the 11 xlset tests, then add:
  - `rejects_csv_input`: a `.csv` temp file with `A1=1` → exit 2, stderr contains `.xlsx/.xlsm`, and the file content is unchanged.
  - `rejects_non_xlsx_output`: `create_simple` + `-o out.csv A1=1` → exit 2, and `out.csv` does not exist.
  - `value_with_equals_sign`: `A2=a=b`, then `dtcat` stdout contains `a=b`.
  - `from_csv_quoted_comma`: stdin `A2,"x, y"\n` via `--from -`, then `dtcat` stdout contains `x, y`.
  - `xls_error_is_exit_2`: change the ported `test_error_xls_not_supported` to expect exit 2 (it was 1 in xlset; format errors are now argument errors).
- [ ] **Step 2:** Run `cargo test --features alpha --test dtset`. Expected: FAIL.
- [ ] **Step 3:** Port `xlset.rs`. Before calling `write_cells`, run `check_editable` on the input and on `--output` and map `Err` to `ArgError`. Run "file not found" before the format check (exit 1, as in xlset).
- [ ] **Step 4:** Run `cargo test --features alpha`. Expected: everything PASSES. Run `cargo test`. Expected: PASS, with `tests/dtset.rs` compiled to nothing. Run `cargo clippy --all-targets --features alpha -- -D warnings`. Expected: clean.
- [ ] **Step 5:** Commit `feat(alpha): add dtset binary (#2)`.

### Task 4: Docs, then PR

**Files:**
- Modify: `README.md`. Keep "Three read-only binaries" and add a **dtset (alpha)** section after dtdiff covering: the `cargo install --path . --features alpha` install line, the xlset quick-reference examples renamed to dtset, type tags `:str :num :bool :date`, the `--from` CSV format, and an "xlsx/xlsm only; not in release binaries" note. Add `dtset` to the exit-code table. Change the "Supersedes xl-cli-tools" blurb so it also mentions `xlset` → `dtset`.

- [ ] **Step 1:** Write the README section.
- [ ] **Step 2:** Run `cargo build --release --features alpha && ./target/release/dtset --version`. Expected: `dtset 0.3.0`.
- [ ] **Step 3:** Commit `docs: document dtset (alpha) (#2)`. Push `feat/dtset-alpha` and open a PR whose body says `Closes #2`.

### Task 5: claude-skills repo (separate PR)

Work in a worktree off `main` (`git -C ~/Dropbox/projects_claude/claude-skills worktree add ../claude-skills-dtset -b dtset-replace-xl main`). The main checkout has uncommitted work on another branch; don't touch it.

- [ ] **Step 1:** `git rm -r skills/xlcat skills/xlfilter skills/xldiff skills/xlset`.
- [ ] **Step 2:** Create `skills/dtset/SKILL.md`. Base it on `skills/xlset/SKILL.md` with `xlset`→`dtset`. The description should trigger on editing, updating, or writing cells in .xlsx/.xlsm. Add an install note (`--features alpha`) and the "xlsx/xlsm only" restriction.
- [ ] **Step 3:** Update `README.md`: delete the deprecated xl-cli-tools table and paragraph, and add a `dtset` row to the dt-cli-tools table.
- [ ] **Step 4:** Commit, push, and open a PR that references LouLouLibs/dt-cli-tools#2.

### Task 6: Local machine cleanup (after both PRs merge; confirm with user before each destructive step)

- [ ] **Step 1:** Install the dt tools. `cargo install --path ~/Dropbox/projects_claude/dt-cli-tools --features alpha` puts all four binaries in `~/.local/share/cargo/bin`. Then remove the stale release copies `~/.local/bin/dt{cat,filter,diff}`. Verify with `which -a dtcat dtset` (one hit each) and `dtcat --version` (`dtcat 0.3.0`).
- [ ] **Step 2:** `cargo uninstall xl-cli-tools`. The crates.toml entry is `xlcat 0.2.1 (path+…)`, so use the name cargo reports. Verify that `command -v xlcat xlfilter xldiff xlset` prints nothing.
- [ ] **Step 3:** In the chezmoi Brewfile, delete `cargo "xlcat"` (`~/.local/share/chezmoi/dot_config/homebrew/Brewfile:526`) and do the per-file preview, backup and apply. dt-cli-tools isn't on crates.io, so nothing replaces it there.
- [ ] **Step 4:** Run `~/Dropbox/projects_claude/claude-skills/install.sh` from the merged main (or copy `skills/dtset`). Delete `~/.claude/skills/{xlcat,xlfilter,xldiff,xlset}`. In `~/.claude/settings.json` `skillOverrides`, remove `xlfilter`, `xldiff` and `xlset`, and remove `dtfilter` and `dtdiff` so those two skills come back on.
- [ ] **Step 5:** In `~/Dropbox/munis_home/CLAUDE.md:430-440`, delete the `/xlcat…` row and replace the `/xlset` row with `/dtset` (alpha build).
- [ ] **Step 6:** Verify with `grep -rnE '\bxl(cat|filter|diff|set)\b' ~/.claude/settings.json ~/.claude/skills ~/Dropbox/munis_home/CLAUDE.md`. Expected: no output.
