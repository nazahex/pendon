//! D9: the `sandbox/ultimate` regression gate.
//!
//! `sandbox/ultimate` is the torture document: the same `src/ultimate.md` is
//! rendered by three different task configurations (all layers configured, no
//! layers configured, and a subset). Its frozen outputs live in
//! `sandbox/ultimate/out.frozen/<task>/ultimate.jsx`.
//!
//! This test copies the sandbox into a temp dir, renders all three tasks with the
//! real CLI and compares the result byte for byte, so a parser or renderer change
//! that moves the output cannot land unnoticed. Re-freezing is deliberate:
//! refresh `out/` with `pendon run -F`, read the three diffs, then copy them over
//! `out.frozen/`.

use assert_cmd::cargo::cargo_bin_cmd;
use std::fs;
use std::path::{Path, PathBuf};

/// The three tasks of `sandbox/ultimate/pendon.toml`, in output-dir order.
const TASKS: [&str; 3] = ["default", "full", "subset"];

/// The sandbox root, relative to the workspace root.
fn sandbox_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("apps dir")
        .parent()
        .expect("workspace root")
        .join("sandbox/ultimate")
}

/// Copies the sandbox project (config + source) into a temp dir and returns it.
fn write_project() -> PathBuf {
    let sandbox = sandbox_dir();
    let dir = std::env::temp_dir().join("pendon-ultimate-freeze");
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");

    fs::copy(sandbox.join("src/ultimate.md"), dir.join("src/ultimate.md")).expect("sandbox source");
    fs::copy(sandbox.join("pendon.toml"), dir.join("pendon.toml")).expect("sandbox config");
    dir
}

/// Renders every task and compares it with its frozen baseline.
#[test]
fn ultimate_matches_the_frozen_baseline() {
    let dir = write_project();
    let mut cmd = cargo_bin_cmd!("pendon");
    cmd.current_dir(&dir)
        .arg("run")
        .arg("-F")
        .assert()
        .success();

    for task in TASKS {
        let frozen_path = sandbox_dir().join(format!("out.frozen/{task}/ultimate.jsx"));
        let expected = fs::read_to_string(&frozen_path)
            .unwrap_or_else(|error| panic!("{}: {error}", frozen_path.display()));
        let actual = fs::read_to_string(dir.join(format!("out/{task}/ultimate.jsx")))
            .unwrap_or_else(|error| panic!("out/{task}/ultimate.jsx: {error}"));

        if actual == expected {
            continue;
        }

        let first_diff = actual
            .lines()
            .zip(expected.lines())
            .enumerate()
            .find(|(_, (got, want))| got != want)
            .map(|(index, (got, want))| {
                format!(
                    "\n  line {}:\n    got:  {}\n    want: {}",
                    index + 1,
                    got,
                    want
                )
            })
            .unwrap_or_else(|| "\n  (outputs differ in length)".to_string());
        panic!(
            "sandbox/ultimate/{task} drifted from out.frozen:{first_diff}\n\n\
             Re-freeze deliberately: run `pendon run -F` inside sandbox/ultimate, review all \
             three diffs, then copy out/<task>/ultimate.jsx over out.frozen/<task>/ultimate.jsx."
        );
    }
}
