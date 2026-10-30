//! S2-10.3: the clean-break audit (D-16,
//! [ADR-0018](../../docs/decisions/0018-clean-replacement.md)).
//!
//! Nothing in the workspace keeps an old config alive: there is no `serde`
//! alias, no `#[deprecated]`, no dual-parse branch and no `MIGRATION.md`. This
//! file is the executable half of that claim — a **stale config** must stop the
//! build with exactly one `Error` that names the key it cannot honour, so a
//! user's old file fails loudly instead of rendering something that looks
//! configured.
//!
//! Each row below is one retired key (§14) in its own project: the run must fail
//! (exit 2), print **one** `Error:` line, and name the replacement in it. Rows
//! that still parsed would have rendered a document, which is the failure mode
//! D-16 exists to forbid.

use assert_cmd::cargo::cargo_bin_cmd;
use predicates::str::contains;
use std::fs;
use std::path::PathBuf;

/// One retired config key and the message it must produce (§13, §14).
struct Case {
    /// File/directory stem, also the label of the row.
    name: &'static str,
    /// The task header: `plugin`, `format` and anything the row needs.
    header: &'static str,
    /// The retired key, as the config writes it.
    stale: &'static str,
    /// What the error must say.
    expect: &'static str,
}

/// Every row is a key §14 retires, with the plugin that used to read it.
const CASES: &[Case] = &[
    Case {
        name: "import",
        header: "plugin = \"anchor\"\nformat = \"solid\"",
        stale: "[[task.anchor.custom]]\nimport = \"import A from 'x'\"\n\
                template = \"<A {...attrs}>{children}</A>\"",
        expect: "`import` was removed; use `imports = [\"…\"]`",
    },
    Case {
        name: "custom-node",
        header: "plugin = \"anchor\"\nformat = \"solid\"",
        stale: "[task.anchor.custom_node]\nname = \"A\"",
        expect: "task.anchor.custom_node was removed",
    },
    Case {
        name: "backtick-key",
        header: "plugin = \"anchor\"\nformat = \"solid\"",
        stale: "[[task.anchor.custom]]\nbacktick_key = \"slug\"\n\
                template = \"<A {...attrs}>{children}</A>\"",
        expect: "unknown key `backtick_key`",
    },
    Case {
        name: "auto-number",
        header: "plugin = \"heading\"\nformat = \"solid\"",
        stale: "[task.heading]\nauto_number = true",
        expect: "task.heading.auto_number / number_style were removed",
    },
    Case {
        name: "number-style",
        header: "plugin = \"heading\"\nformat = \"solid\"",
        stale: "[task.heading]\nnumber_style = \"decimal\"",
        expect: "task.heading.auto_number / number_style were removed",
    },
    Case {
        name: "recommended",
        header: "plugin = \"extract\"\nformat = \"solid\"",
        stale: "[[task.heading.extract]]\nformat = \"json\"\noutput = \"./out/a.json\"\n\
                recommended = true",
        expect: "task.heading.extract[0].recommended was removed",
    },
    Case {
        name: "figure-layer",
        header: "plugin = \"image\"\nformat = \"solid\"",
        stale: "[task.image.custom.figure]\nname = \"Fig\"\ntemplate = \"<F />\"",
        expect: "unsupported layer for `image`",
    },
];

/// One stale project per row, and the document every row renders from.
fn project(case: &Case) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("pendon-clean-break-{}", case.name));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(dir.join("src")).expect("src dir");
    fs::write(
        dir.join("src/a.mdp"),
        "# [slug](\"T\") Heading\n\nAn @[x](/u) para.\n",
    )
    .expect("source");
    fs::write(
        dir.join("pendon.toml"),
        format!(
            "[[task]]\nname = \"a\"\ninput = \"./src/a.mdp\"\noutput = \"./out/a.jsx\"\n\
             {header}\n\n{stale}\n",
            header = case.header,
            stale = case.stale,
        ),
    )
    .expect("config");
    dir
}

/// D-16: a stale config is one `Error` and no output, for every row.
#[test]
fn a_stale_config_fails_the_build_with_one_error() {
    for case in CASES {
        let dir = project(case);
        let assert = cargo_bin_cmd!("pendon")
            .current_dir(&dir)
            .arg("run")
            .assert()
            .failure()
            .code(2)
            .stderr(contains(case.expect));
        let stderr = String::from_utf8_lossy(&assert.get_output().stderr).to_string();
        assert_eq!(
            stderr
                .lines()
                .filter(|line| line.starts_with("Error:"))
                .count(),
            1,
            "{}: one Error, got:\n{stderr}",
            case.name
        );
        assert!(
            !dir.join("out/a.jsx").exists(),
            "{}: a stale config must not render anything",
            case.name
        );
    }
}
