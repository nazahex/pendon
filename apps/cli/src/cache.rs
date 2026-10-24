//! The render cache: **one TOML record per `(task, input)` pair**.
//!
//! `.pendon/cache/<shard>/<key>.toml`, where `key = sha256(task ␟ input)` and
//! `<shard>` is the first two hex digits of `key` (256 directories, so a project
//! with tens of thousands of files still gets small directories). One record
//! answers exactly one question — "is this input up to date for this task?" — so:
//!
//! - a file may render for two tasks without either record overwriting the other;
//! - a run writes only the records of the files it rendered, instead of
//!   re-serialising every file in the project on every run;
//! - a lookup reads one small file, not the project's whole history;
//! - a crash can damage at most one record (writes go through a temp file +
//!   rename), and the damage costs one re-render, nothing more.
//!
//! Everything is plain TOML on purpose: `cat`, `grep -r` and `git diff` work on
//! it, and `--clean-cache` is `rm -rf .pendon/cache`. The layout decision and the
//! alternatives that were rejected are in
//! [`docs/decisions/0006-per-file-cache-records.md`](../../docs/decisions/0006-per-file-cache-records.md).

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

/// Record schema. A record written by another version is ignored (and replaced),
/// so a layout change costs one cold run, never a manual `--clean-cache`.
pub const CACHE_VERSION: u32 = 1;

const CACHE_ROOT: &str = "./.pendon/cache";

/// Pre-v1 layout: a single manifest describing the whole project. It is not
/// readable by this version and not worth migrating (its records are keyed by
/// input path only, which is the bug this layout fixes).
const LEGACY_MANIFEST: &str = "./.pendon/cache.toml";

/// A file the render depended on, with the content hash it had at the time.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DepInfo {
    pub path: String,
    pub hash: String,
}

/// Everything needed to decide whether one input is still up to date for one
/// task. Self-contained: no separate task table to look up, so a record can be
/// read, checked and deleted on its own.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CacheEntry {
    /// Schema of this record ([`CACHE_VERSION`]).
    pub version: u32,
    /// Task that produced the output; also part of the record's key.
    pub task: String,
    /// Input path the task matched; also part of the record's key.
    pub input: String,
    /// Output path the task wrote (`[slug]` already expanded).
    pub output: String,
    /// Hash of the task's effective configuration (`run.rs::task_config_hash`),
    /// `post_hook` included, so editing the task invalidates its records.
    pub config_hash: String,
    pub input_hash: String,
    pub output_hash: String,
    pub mtime: u64,
    pub deps: Vec<DepInfo>,
}

/// Handle on `.pendon/cache`. Cheap to create; every operation is a single file
/// read/write, so there is nothing to load up front and nothing to save at the
/// end of a run.
#[derive(Debug, Clone)]
pub struct CacheStore {
    root: PathBuf,
}

impl CacheStore {
    /// Open the project's cache, dropping a pre-v1 manifest if one survives.
    ///
    /// Discarding it is deliberate: it cannot be trusted (its records are keyed
    /// by input path alone) and leaving it behind would confuse both `git status`
    /// and anyone reading the directory.
    pub fn open() -> Self {
        let _ = fs::remove_file(LEGACY_MANIFEST);
        Self::at(CACHE_ROOT)
    }

    /// A store rooted anywhere: unit tests, or a caller that is not in the
    /// project root.
    pub fn at(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    /// Where `(task, input)`'s record lives. Both halves are in the name, so two
    /// tasks over one input keep two records (ADR-0006).
    pub fn record_path(&self, task: &str, input: &str) -> PathBuf {
        let key = record_key(task, input);
        self.root.join(&key[..2]).join(format!("{key}.toml"))
    }

    /// The record for this `(task, input)`, if a readable one exists.
    ///
    /// A record that cannot be parsed, or that was written by another schema, is
    /// deleted rather than trusted: the cost is one re-render, never a wrong
    /// "up to date".
    pub fn get(&self, task: &str, input: &str) -> Option<CacheEntry> {
        let path = self.record_path(task, input);
        let text = fs::read_to_string(&path).ok()?;
        match toml::from_str::<CacheEntry>(&text) {
            Ok(entry) if entry.version == CACHE_VERSION => Some(entry),
            _ => {
                let _ = fs::remove_file(&path);
                None
            }
        }
    }

    /// Record a successful render.
    ///
    /// Writes a sibling temp file and renames it over the record, so a reader
    /// (or a second `pendon`) never observes a half-written entry and a crash
    /// leaves either the old record or the new one.
    pub fn put(&self, entry: &CacheEntry) -> Result<(), String> {
        let path = self.record_path(&entry.task, &entry.input);
        let dir = path
            .parent()
            .ok_or_else(|| format!("cache path {} has no parent", path.display()))?;
        fs::create_dir_all(dir).map_err(|e| format!("cannot create {}: {}", dir.display(), e))?;

        let text = toml::to_string_pretty(entry)
            .map_err(|e| format!("cannot serialize cache record: {}", e))?;
        let tmp = path.with_extension("toml.tmp");
        fs::write(&tmp, text).map_err(|e| format!("cannot write {}: {}", tmp.display(), e))?;
        fs::rename(&tmp, &path).map_err(|e| format!("cannot replace {}: {}", path.display(), e))
    }

    /// True when `input` would still render `output` byte-identically for this
    /// task: same task config, same input bytes, every dependency unchanged, and
    /// the output still on disk.
    pub fn is_fresh(&self, task: &str, input: &str, output: &str, config_hash: &str) -> bool {
        let Some(entry) = self.get(task, input) else {
            return false;
        };

        if entry.config_hash != config_hash {
            return false;
        }

        // The record must describe *this* output: a changed `output` template is
        // a different file, even when the old one is still on disk.
        if entry.output != output {
            return false;
        }

        // A deleted or moved output is not up to date, whatever the record says.
        if !Path::new(output).exists() {
            return false;
        }

        match hash_file(Path::new(input)) {
            Ok(hash) if hash == entry.input_hash => {}
            _ => return false,
        }

        entry
            .deps
            .iter()
            .all(|dep| hash_file(Path::new(&dep.path)).is_ok_and(|hash| hash == dep.hash))
    }

    /// `--clean-cache`: forget every record, and a legacy manifest with them.
    pub fn clean() -> Result<(), String> {
        if Path::new(CACHE_ROOT).exists() {
            fs::remove_dir_all(CACHE_ROOT)
                .map_err(|e| format!("cannot remove {}: {}", CACHE_ROOT, e))?;
        }
        if Path::new(LEGACY_MANIFEST).exists() {
            fs::remove_file(LEGACY_MANIFEST)
                .map_err(|e| format!("cannot remove {}: {}", LEGACY_MANIFEST, e))?;
        }
        // Leave no empty `.pendon/` behind once nothing is cached.
        if let Some(dir) = Path::new(CACHE_ROOT).parent().filter(|dir| dir.exists()) {
            let _ = fs::remove_dir(dir);
        }
        Ok(())
    }
}

/// A record's file name: the task *and* the input, so one input may be cached
/// for any number of tasks without collisions.
fn record_key(task: &str, input: &str) -> String {
    hash_content(&format!("{}\u{1f}{}", task, input))
}

pub fn hash_content(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    result.iter().map(|byte| format!("{:02x}", byte)).collect()
}

pub fn hash_file(path: &Path) -> Result<String, String> {
    let content = fs::read_to_string(path)
        .map_err(|e| format!("cannot read file {}: {}", path.display(), e))?;
    Ok(hash_content(&content))
}

pub fn get_file_mtime(path: &Path) -> Result<u64, String> {
    let metadata = fs::metadata(path)
        .map_err(|e| format!("cannot get metadata for {}: {}", path.display(), e))?;

    Ok(metadata
        .modified()
        .map_err(|e| format!("cannot get mtime for {}: {}", path.display(), e))?
        .duration_since(SystemTime::UNIX_EPOCH)
        .map_err(|e| format!("cannot convert mtime for {}: {}", path.display(), e))?
        .as_secs())
}

/// Build the record for a render that succeeded: the state of the input, the
/// output and every dependency, stamped with the task configuration that
/// produced it.
///
/// `output` may be missing (a task whose output a hook moved, for instance); the
/// record then stores an empty `output_hash`, and [`CacheStore::is_fresh`] keeps
/// reporting the file as stale until the output exists again.
pub fn create_entry(
    task: &str,
    config_hash: &str,
    input: &str,
    output: &str,
    deps: Vec<String>,
) -> Result<CacheEntry, String> {
    let input_path = Path::new(input);
    let output_path = Path::new(output);

    let input_hash = hash_file(input_path)?;

    let output_hash = if output_path.exists() {
        hash_file(output_path).unwrap_or_default()
    } else {
        String::new()
    };

    let mtime = get_file_mtime(input_path).unwrap_or(0);

    let dep_infos = deps
        .into_iter()
        .filter_map(|path| {
            let hash = hash_file(Path::new(&path)).ok()?;
            Some(DepInfo { path, hash })
        })
        .collect();

    Ok(CacheEntry {
        version: CACHE_VERSION,
        task: task.to_string(),
        input: input.to_string(),
        output: output.to_string(),
        config_hash: config_hash.to_string(),
        input_hash,
        output_hash,
        mtime,
        deps: dep_infos,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    /// A project-shaped temp dir whose files exist for real: `is_fresh` hashes
    /// them, so faking them would test nothing.
    struct Project {
        dir: tempfile::TempDir,
        input: String,
        output: String,
        config: String,
    }

    fn project() -> Project {
        let dir = tempdir().expect("temp dir");
        fs::create_dir_all(dir.path().join("src")).expect("src dir");
        fs::create_dir_all(dir.path().join("out")).expect("out dir");

        let input = dir.path().join("src").join("page.md");
        let output = dir.path().join("out").join("page.html");
        let config = dir.path().join("pendon.toml");
        fs::write(&input, "Body.\n").expect("write input");
        fs::write(&output, "<p>Body.</p>\n").expect("write output");
        fs::write(&config, "[[task]]\n").expect("write config");

        Project {
            input: input.to_string_lossy().into_owned(),
            output: output.to_string_lossy().into_owned(),
            config: config.to_string_lossy().into_owned(),
            dir,
        }
    }

    impl Project {
        fn store(&self) -> CacheStore {
            CacheStore::at(self.dir.path().join(".pendon").join("cache"))
        }

        /// A record claiming the task rendered `input` to `output` with every
        /// dependency unchanged.
        fn entry(&self, task: &str) -> CacheEntry {
            create_entry(
                task,
                "cfg",
                &self.input,
                &self.output,
                vec![self.config.clone()],
            )
            .expect("build entry")
        }
    }

    #[test]
    fn a_record_round_trips() {
        let project = project();
        let store = project.store();
        let entry = project.entry("Site");

        store.put(&entry).expect("put");

        assert_eq!(store.get("Site", &project.input), Some(entry));
        assert_eq!(store.get("Other", &project.input), None);
    }

    /// The bug this layout fixes: one input, two tasks, two records.
    #[test]
    fn two_tasks_keep_separate_records_for_one_input() {
        let project = project();
        let store = project.store();

        let html = project.entry("Site (HTML)");
        store.put(&html).expect("put html");
        let mut solid = project.entry("Site (Solid)");
        solid.output_hash = "a-solid-hash".to_string();
        store.put(&solid).expect("put solid");

        assert_ne!(
            store.record_path("Site (HTML)", &project.input),
            store.record_path("Site (Solid)", &project.input),
            "both tasks must not share one record"
        );
        assert_eq!(store.get("Site (HTML)", &project.input), Some(html));
        assert_eq!(store.get("Site (Solid)", &project.input), Some(solid));
        assert!(store.is_fresh("Site (HTML)", &project.input, &project.output, "cfg"));
        assert!(store.is_fresh("Site (Solid)", &project.input, &project.output, "cfg"));
    }

    #[test]
    fn editing_the_task_invalidates_the_record() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");

        assert!(store.is_fresh("Site", &project.input, &project.output, "cfg"));
        assert!(
            !store.is_fresh("Site", &project.input, &project.output, "edited"),
            "a changed config hash must invalidate the record"
        );
    }

    #[test]
    fn changing_the_input_invalidates_the_record() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");

        fs::write(&project.input, "Changed.\n").expect("rewrite input");

        assert!(!store.is_fresh("Site", &project.input, &project.output, "cfg"));
    }

    #[test]
    fn changing_a_dependency_invalidates_the_record() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");

        fs::write(&project.config, "[[task]]\nname = \"changed\"\n").expect("rewrite config");
        assert!(!store.is_fresh("Site", &project.input, &project.output, "cfg"));

        // A dependency that disappeared is just as invalid as a changed one.
        fs::remove_file(&project.config).expect("remove config");
        assert!(!store.is_fresh("Site", &project.input, &project.output, "cfg"));
    }

    #[test]
    fn a_changed_output_path_invalidates_the_record() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");

        // An `output` template edit lands on a different file; it must not be
        // considered fresh just because the record's own output still exists.
        let other = project.dir.path().join("out").join("page.md");
        fs::write(&other, "moved\n").expect("write other output");

        assert!(!store.is_fresh("Site", &project.input, &other.to_string_lossy(), "cfg"));
    }

    #[test]
    fn a_missing_output_is_stale() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");
        assert!(store.is_fresh("Site", &project.input, &project.output, "cfg"));

        fs::remove_file(&project.output).expect("remove output");

        assert!(
            !store.is_fresh("Site", &project.input, &project.output, "cfg"),
            "a deleted output must be rebuilt, not skipped"
        );
    }

    #[test]
    fn a_record_from_another_schema_is_dropped() {
        let project = project();
        let store = project.store();
        store.put(&project.entry("Site")).expect("put");

        let path = store.record_path("Site", &project.input);
        let text = fs::read_to_string(&path)
            .expect("read record")
            .replace("version = 1", "version = 0");
        fs::write(&path, text).expect("downgrade record");

        assert_eq!(store.get("Site", &project.input), None);
        assert!(
            !path.exists(),
            "an unreadable record must be removed, not kept to rot"
        );
    }
}
