use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pico_args::Arguments;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use toml::Value;
use walkdir::WalkDir;

use crate::cache::{hash_content, hash_file, CacheStore};
use crate::config::{PendonConfig, PostHook};
use crate::hook::run_post_hooks;
use crate::plugins::load_custom_registry;
use crate::process::process_single_file;
use crate::utils::{captures_to_map, input_pattern_to_regex, substitute_output};

pub fn run_from_config() -> ExitCode {
    let cfg_text = match fs::read_to_string("pendon.toml") {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: cannot read pendon.toml: {}", e);
            return ExitCode::from(2);
        }
    };
    // Parse as `toml::Value` first so `task.include` can be resolved before
    // deserialising into `PendonConfig`. Included files' task content is
    // deep-merged into the including task, then `include` is dropped so it
    // never surfaces as an unknown key (§13).
    let mut cfg_value: Value = match toml::from_str(&cfg_text) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("Error: invalid pendon.toml: {}", e);
            return ExitCode::from(2);
        }
    };

    if let Err(msg) = resolve_includes(&mut cfg_value) {
        eprintln!("Error: {msg}");
        return ExitCode::from(2);
    }

    // Round-trip through TOML text to deserialise the merged value into
    // `PendonConfig` (toml 1.x has no `from_value`).
    let cfg_text = match toml::to_string(&cfg_value) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Error: cannot serialise merged config: {e}");
            return ExitCode::from(2);
        }
    };
    let cfg: PendonConfig = match toml::from_str(&cfg_text) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: invalid pendon.toml: {}", e);
            return ExitCode::from(2);
        }
    };

    // §13: an unknown key inside a `[[task]]` table is a **load error**, not a
    // warning — a typo must never change the output while the build succeeds.
    // Every offending key is reported before aborting, so one run fixes them all.
    let mut unknown_keys = false;
    for (index, task) in cfg.tasks.iter().enumerate() {
        let name = task
            .name
            .clone()
            .unwrap_or_else(|| format!("task-{}", index));
        for key in task.unknown.keys() {
            eprintln!("Error: [{name}] unknown key '{key}' in pendon.toml (§13); remove it or fix the typo");
            unknown_keys = true;
        }
    }
    if unknown_keys {
        return ExitCode::from(2);
    }

    let custom_registry = match load_custom_registry(cfg.plugin_custom.as_ref()) {
        Ok(map) => map,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };

    // Parse cache flags using pico_args for consistency with the rest of the CLI.
    let mut pargs = Arguments::from_env();
    // Consume the binary name and the "run" subcommand safely.
    let _ = pargs.subcommand();

    let no_cache = pargs.contains("--no-cache");
    let clean_cache = pargs.contains("--clean-cache");
    // Ignore existing cache but record the fresh state for the next run.
    let force_rebuild = pargs.contains(["-F", "--force"]);
    // `-q` / `--quiet` silences the `run` report (`docs/spec/CLI-REPORT.md` §5).
    // Diagnostics, warnings and errors are unaffected.
    let quiet = pargs.contains(["-q", "--quiet"]);

    if clean_cache {
        if let Err(e) = CacheStore::clean() {
            eprintln!("Warning: {}", e);
        }
    }

    // The cache is one record per rendered file (ADR-0006), so there is nothing
    // to load or save: reads and writes are gated per run and per task instead.
    // `--no-cache` reads nothing *and* records nothing, so the next run starts
    // cold; `-F/--force` only declines to read, and refreshes every record.
    let store = CacheStore::open();
    let read_cache = !no_cache && !force_rebuild;
    let write_cache = !no_cache;

    let theme = pendon_tui::Theme::default();
    if !quiet && pendon_tui::is_interactive_stderr() {
        pendon_tui::render_status_line("Scanning source files...", theme);
    }

    let mut exit = ExitCode::SUCCESS;

    for (task_idx, task) in cfg.tasks.iter().enumerate() {
        let task_name = task
            .name
            .clone()
            .unwrap_or_else(|| format!("task-{}", task_idx));

        // Per-task `no_cache` (`docs/rfc/no-cache-and-post-hook.md`): the file is
        // always rendered, and nothing is read or recorded for this task. Other
        // tasks keep their records.
        let cache_disabled = task.no_cache.unwrap_or(false);
        let cache_read = read_cache && !cache_disabled;
        let cache_write = write_cache && !cache_disabled;

        // A failed render or post-hook marks the whole task failed: its remaining
        // hooks are skipped and its cache record is not refreshed.
        let mut task_failed = false;

        // Resolved once per task, expanded per file (the placeholders are
        // file-scoped).
        let task_hooks: Option<Vec<&str>> = task.post_hook.as_ref().map(PostHook::commands);

        let mut total_skipped = 0usize;
        let mut total_processed = 0usize;
        let mut total_skipped_write = 0usize;

        // Hash task config. `post_hook` participates: hooks only run for files
        // this run actually rendered, so adding or editing a hook must
        // invalidate the task once — otherwise a warm cache would never format.
        // Commands are joined with a unit separator so a `:` inside a command
        // can never collide with the field separator.
        let hook_signature = task_hooks
            .as_ref()
            .map(|commands| commands.join("\u{1f}"))
            .unwrap_or_default();
        let task_config_str = format!(
            "{}:{}:{}:{}:{}:{}",
            task.plugin.as_deref().unwrap_or(""),
            task.markdown_allow_html.unwrap_or(false),
            task.markdown_strip_comments.unwrap_or(false),
            task.wiki_link_prefix.as_deref().unwrap_or(""),
            task.format,
            hook_signature
        );
        let task_config_hash = hash_content(&task_config_str);

        let re = match input_pattern_to_regex(&task.input) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: invalid input pattern '{}': {}", task.input, e);
                exit = ExitCode::from(2);
                continue;
            }
        };
        let task_markdown_opts = MarkdownOptions {
            allow_html: task.markdown_allow_html.unwrap_or(false),
            strip_comments: task.markdown_strip_comments.unwrap_or(false),
        };
        let task_wiki_opts = WikiOptions {
            link_prefix: task.wiki_link_prefix.clone(),
            latex: if crate::plugins::has_plugin(task.plugin.as_deref(), "latex") {
                Some(pendon_plugin_latex::LatexOptions {
                    target: crate::plugins::latex_target_for_format(&task.format),
                })
            } else {
                None
            },
        };
        let mut matched = 0usize;
        let mut total_bytes: usize = 0;
        let mut unique_ids: std::collections::HashSet<String> = std::collections::HashSet::new();

        for entry in WalkDir::new(Path::new("."))
            // The cache is not a document tree: pruning `.pendon` keeps its
            // records out of every task's glob *and* out of the walk itself.
            .into_iter()
            .filter_entry(|e| e.path() != Path::new("./.pendon"))
            .filter_map(|e| e.ok())
        {
            if !entry.file_type().is_file() {
                continue;
            }
            let path_str = entry.path().to_string_lossy();
            if let Some(caps) = re.captures(&path_str) {
                matched += 1;
                let map = match captures_to_map(&task.input, &caps) {
                    Ok(m) => m,
                    Err(e) => {
                        eprintln!("Error: capture mapping failed: {}", e);
                        exit = ExitCode::from(2);
                        continue;
                    }
                };
                if let Some(id) = map.get("id") {
                    unique_ids.insert(id.clone());
                }
                let out_path = match substitute_output(&task.output, &map) {
                    Ok(p) => p,
                    Err(e) => {
                        eprintln!("Error: output substitution failed: {}", e);
                        exit = ExitCode::from(2);
                        continue;
                    }
                };

                // Check cache. A `no_cache` task never consults it, so it always
                // renders from scratch (RFC `no-cache-and-post-hook`).
                let should_skip = cache_read
                    && store.is_fresh(&task_name, &path_str, &out_path, &task_config_hash);

                if should_skip {
                    total_skipped += 1;
                    if let Ok(metadata) = fs::metadata(&out_path) {
                        total_bytes += metadata.len() as usize;
                    }
                    continue;
                }

                total_processed += 1;

                // Collect dependencies
                let mut deps = Vec::new();
                deps.push("pendon.toml".to_string());

                // Add external reference file if exists
                if let Some(cite_config) = &task.cite {
                    if cite_config.reference_source.as_deref() == Some("external") {
                        if let Some(ref_file) = &cite_config.reference_file {
                            if let Ok(resolved) = substitute_output(ref_file, &map) {
                                if Path::new(&resolved).exists() {
                                    deps.push(resolved);
                                }
                            }
                        }
                    }
                }

                // Add custom plugin specs
                if let Some(pstr) = task.plugin.as_deref() {
                    for name in pstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
                        if let Some(path) = name.strip_prefix("toml:") {
                            if Path::new(path).exists() {
                                deps.push(path.to_string());
                            }
                        }
                    }
                }

                let result = process_single_file(
                    task,
                    &task_name,
                    &task_config_hash,
                    &path_str,
                    &out_path,
                    &map,
                    &custom_registry,
                    task_markdown_opts,
                    task_wiki_opts.clone(),
                    deps,
                );

                if !result.success {
                    exit = ExitCode::from(2);
                    task_failed = true;
                    continue;
                }

                total_bytes += result.bytes_written;
                if result.skipped_write {
                    total_skipped_write += 1;
                }

                // Post-hooks (RFC `no-cache-and-post-hook`): per rendered file,
                // never for a task that already failed. A file served from the
                // cache did not run the pipeline, so its hooks are not re-run.
                let mut cache_entry = result.cache_entry;
                if let Some(commands) = task_hooks.as_deref() {
                    if !task_failed {
                        match run_post_hooks(&task_name, commands, &map) {
                            Ok(()) => {
                                // A hook may rewrite the output; keep the cache
                                // record's `output_hash` honest.
                                if let Some(entry) = cache_entry.as_mut() {
                                    if let Ok(hash) = hash_file(Path::new(&out_path)) {
                                        entry.output_hash = hash;
                                    }
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: {}", e);
                                task_failed = true;
                                exit = ExitCode::from(2);
                                // Do not record a success the task did not reach,
                                // so the next run renders and hooks it again.
                                cache_entry = None;
                            }
                        }
                    }
                }

                // Record the render (never for a `no_cache` task). Written after
                // the hooks, so what a formatter rewrote is what gets remembered.
                if cache_write {
                    if let Some(entry) = cache_entry {
                        if let Err(e) = store.put(&entry) {
                            eprintln!("Warning: {}", e);
                        }
                    }
                }
            }
        }

        if !quiet && pendon_tui::is_interactive_stderr() {
            pendon_tui::render_severity_line(
                pendon_tui::SeverityLine::Info,
                &format!("Found: {} files", matched),
                theme,
            );
            let size_text = if total_bytes >= 1_048_576 {
                format!("{:.1} MB", (total_bytes as f64) / 1_048_576.0)
            } else if total_bytes >= 1024 {
                format!("{:.1} kB", (total_bytes as f64) / 1024.0)
            } else {
                format!("{} B", total_bytes)
            };
            let fmt_text = task.format.as_str();
            let total_text = matched.to_string();

            let cache_text = if total_skipped > 0 || total_skipped_write > 0 {
                format!(
                    "{} cached, {} rebuilt ({} write-skipped)",
                    total_skipped, total_processed, total_skipped_write
                )
            } else {
                format!("{} processed", total_processed)
            };

            if let Some(name) = task.name.as_deref() {
                let items = [
                    ("name", name),
                    ("format", fmt_text),
                    ("size", size_text.as_str()),
                    ("total", total_text.as_str()),
                    ("cache", cache_text.as_str()),
                ];
                pendon_tui::render_kv_list("› Wrote:", &items, theme);
            } else {
                let items = [
                    ("input", task.input.as_str()),
                    ("output", task.output.as_str()),
                    ("format", fmt_text),
                    ("size", size_text.as_str()),
                    ("total", total_text.as_str()),
                    ("cache", cache_text.as_str()),
                ];
                pendon_tui::render_kv_list("› Wrote:", &items, theme);
            }
        }
    }

    if !quiet && pendon_tui::is_interactive_stderr() {
        pendon_tui::render_severity_line(
            pendon_tui::SeverityLine::Done,
            "All tasks completed",
            theme,
        );
    }
    exit
}

/// Resolve all `task.include` directives in `cfg_value`, merging included
/// files' `task` content into the including task and removing `include` keys.
///
/// Paths in `include` are relative to `pendon.toml`'s directory (the CWD,
/// since `pendon.toml` is read from there).
///
/// Returns an error on missing files, parse errors, or circular includes.
fn resolve_includes(cfg: &mut Value) -> Result<(), String> {
    let mut chain: Vec<String> = Vec::new();
    resolve_includes_inner(cfg, &mut chain, 0)
}

fn resolve_includes_inner(
    cfg: &mut Value,
    chain: &mut Vec<String>,
    depth: usize,
) -> Result<(), String> {
    const MAX_DEPTH: usize = 16;

    if depth > MAX_DEPTH {
        return Err("task.include: maximum include depth exceeded (circular include?)".to_string());
    }

    let tasks = match cfg.get_mut("task") {
        Some(Value::Array(tasks)) => tasks,
        // A table-form `task` (e.g. from `[[task.anchor.custom]]` in an
        // included file) has no `[[task]]` entries to scan for `include`,
        // so there is nothing to resolve here.
        Some(_) => return Ok(()),
        None => return Ok(()),
    };

    let mut i = 0;
    while i < tasks.len() {
        let task = match tasks.get_mut(i) {
            Some(task) => task,
            _ => {
                i += 1;
                continue;
            }
        };

        let include_paths = match task.get("include") {
            Some(Value::Array(paths)) => {
                let mut result = Vec::new();
                for p in paths {
                    match p {
                        Value::String(s) => result.push(s.clone()),
                        _ => return Err("`include` must be an array of strings".to_string()),
                    }
                }
                result
            }
            Some(_) => return Err("`include` must be an array of strings".to_string()),
            None => {
                i += 1;
                continue;
            }
        };

        // Remove `include` so it never surfaces as an unknown key (§13).
        if let Value::Table(ref mut task_map) = task {
            task_map.remove("include");
        }

        for path in &include_paths {
            if chain.contains(path) {
                return Err(format!(
                    "task.include: circular include detected: {}",
                    chain.join(" -> ") + " -> " + path
                ));
            }
            chain.push(path.clone());

            let included = match load_included_file(path) {
                Ok(v) => v,
                Err(msg) => {
                    chain.pop();
                    return Err(msg);
                }
            };

            // Resolve includes within the included file first (nested includes).
            let mut included_mut = included;
            resolve_includes_inner(&mut included_mut, chain, depth + 1)?;

            chain.pop();

            // Merge the included file's `task` content into this task.
            if let Some(included_task) = included_mut.get("task") {
                merge_task_content(task, included_task);
            }
        }

        i += 1;
    }

    Ok(())
}

/// Read and parse an included TOML file. Paths are relative to the CWD
/// (where `pendon.toml` lives).
fn load_included_file(path: &str) -> Result<Value, String> {
    let text = match fs::read_to_string(path) {
        Ok(s) => s,
        Err(e) => return Err(format!("task.include: cannot read '{path}': {e}")),
    };
    toml::from_str(&text).map_err(|e| format!("task.include: invalid TOML in '{path}': {e}"))
}

/// Merge an included file's `task` content into a target task table.
///
/// - If the included `task` is a table (e.g. from `[[task.anchor.custom]]`),
///   its fields are deep-merged into the target.
/// - If the included `task` is an array (e.g. from `[[task]]`), each element
///   is deep-merged into the target in order.
fn merge_task_content(target: &mut Value, included: &Value) {
    match (target, included) {
        (Value::Table(target_map), Value::Table(source_map)) => {
            for (key, value) in source_map {
                if let Some(existing) = target_map.get_mut(key.as_str()) {
                    merge_value(existing, value);
                } else {
                    target_map.insert(key.clone(), value.clone());
                }
            }
        }
        (Value::Table(target_map), Value::Array(source_arr)) => {
            for element in source_arr {
                if let Value::Table(element_map) = element {
                    for (key, value) in element_map {
                        if let Some(existing) = target_map.get_mut(key.as_str()) {
                            merge_value(existing, value);
                        } else {
                            target_map.insert(key.clone(), value.clone());
                        }
                    }
                }
            }
        }
        _ => {}
    }
}

/// Deep-merge `source` into `target`, modifying `target` in place.
///
/// - Tables merge recursively.
/// - Arrays concatenate.
/// - Scalars: `source` wins.
fn merge_value(target: &mut Value, source: &Value) {
    if target.as_table().is_some() && source.as_table().is_some() {
        let t = target.as_table_mut().expect("target is a table");
        let s = source.as_table().expect("source is a table");
        for (k, v) in s {
            if let Some(e) = t.get_mut(k.as_str()) {
                merge_value(e, v);
            } else {
                t.insert(k.clone(), v.clone());
            }
        }
    } else if target.as_array().is_some() && source.as_array().is_some() {
        let t = target.as_array_mut().expect("target is an array");
        let s = source.as_array().expect("source is an array");
        t.extend(s.iter().cloned());
    } else {
        *target = source.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn parse_value(src: &str) -> Value {
        toml::from_str(src).expect("valid toml")
    }

    /// Deep-merge: scalars, source wins.
    #[test]
    fn merge_value_scalars_source_wins() {
        let mut target = parse_value("a = 1\nb = 2");
        let source = parse_value("b = 3\nc = 4");
        merge_value(&mut target, &source);
        assert_eq!(target.get("a").and_then(|v| v.as_integer()), Some(1));
        assert_eq!(target.get("b").and_then(|v| v.as_integer()), Some(3));
        assert_eq!(target.get("c").and_then(|v| v.as_integer()), Some(4));
    }

    /// Deep-merge: tables merge recursively.
    #[test]
    fn merge_value_tables_recursive() {
        let mut target = parse_value("a = 1\n[b]\nx = 1");
        let source = parse_value("[b]\ny = 2\n[c]\nz = 3");
        merge_value(&mut target, &source);
        assert_eq!(target.get("a").and_then(|v| v.as_integer()), Some(1));
        let b = target.get("b").and_then(|v| v.as_table()).expect("b table");
        assert_eq!(b.get("x").and_then(|v| v.as_integer()), Some(1));
        assert_eq!(b.get("y").and_then(|v| v.as_integer()), Some(2));
        let c = target.get("c").and_then(|v| v.as_table()).expect("c table");
        assert_eq!(c.get("z").and_then(|v| v.as_integer()), Some(3));
    }

    /// Deep-merge: arrays concatenate.
    #[test]
    fn merge_value_arrays_concatenate() {
        let mut target = parse_value("items = [{a = 1}]");
        let source = parse_value("items = [{b = 2}]");
        merge_value(&mut target, &source);
        let items = target
            .get("items")
            .and_then(|v| v.as_array())
            .expect("items array");
        assert_eq!(items.len(), 2);
    }

    /// `merge_task_content`: table form (e.g. `[[task.anchor.custom]]`).
    #[test]
    fn merge_task_content_table_form() {
        let mut target = parse_value("name = \"Demo\"");
        let included = parse_value("task = { anchor = { custom = [{ type = [\"A\"] }] } }");
        merge_task_content(&mut target, included.get("task").unwrap());
        assert_eq!(target.get("name").and_then(|v| v.as_str()), Some("Demo"));
        let anchor = target
            .get("anchor")
            .and_then(|v| v.as_table())
            .expect("anchor table");
        let custom = anchor
            .get("custom")
            .and_then(|v| v.as_array())
            .expect("custom array");
        assert_eq!(custom.len(), 1);
        let entry = custom.first().unwrap().as_table().unwrap();
        assert_eq!(
            entry.get("type").and_then(|v| v.as_array()).unwrap().len(),
            1
        );
    }

    /// `merge_task_content`: array form (e.g. `[[task]]`).
    #[test]
    fn merge_task_content_array_form() {
        let mut target = parse_value("name = \"Demo\"");
        let included = parse_value("task = [{ markdown_strip_comments = true }]");
        merge_task_content(&mut target, included.get("task").unwrap());
        assert_eq!(target.get("name").and_then(|v| v.as_str()), Some("Demo"));
        assert_eq!(
            target
                .get("markdown_strip_comments")
                .and_then(|v| v.as_bool()),
            Some(true)
        );
    }

    /// `merge_value`: mismatched types — source overrides target.
    #[test]
    fn merge_value_type_mismatch_source_wins() {
        let mut target = parse_value("key = { a = 1 }");
        let source = parse_value("key = \"override\"");
        merge_value(&mut target, &source);
        assert_eq!(target.get("key").and_then(|v| v.as_str()), Some("override"));
    }
}
