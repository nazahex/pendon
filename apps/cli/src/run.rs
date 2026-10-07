use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pico_args::Arguments;
use std::fs;
use std::path::Path;
use std::process::ExitCode;
use walkdir::WalkDir;

use crate::cache::{should_skip_file, CacheManifest, CacheTask};
use crate::config::PendonConfig;
use crate::plugins::{build_vicado_hints_override, load_custom_registry};
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
    let cfg: PendonConfig = match toml::from_str(&cfg_text) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Error: invalid pendon.toml: {}", e);
            return ExitCode::from(2);
        }
    };

    // §13: an unknown key inside a `[[task]]` table would otherwise be dropped
    // silently. Report it and keep going, so a typo never changes the output
    // without saying so.
    for (index, task) in cfg.tasks.iter().enumerate() {
        let name = task
            .name
            .clone()
            .unwrap_or_else(|| format!("task-{}", index));
        for key in task.unknown.keys() {
            eprintln!("Warning: [{name}] unknown key '{key}' in pendon.toml; it is ignored (§13)");
        }
    }

    let custom_registry = match load_custom_registry(cfg.plugin_custom.as_ref()) {
        Ok(map) => map,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };
    let vicado_hints_override = build_vicado_hints_override(cfg.plugin_vicado.as_ref());

    // Parse cache flags using pico_args for consistency with the rest of the CLI.
    let mut pargs = Arguments::from_env();
    // Consume the binary name and the "run" subcommand safely.
    let _ = pargs.subcommand();

    let no_cache = pargs.contains("--no-cache");
    let clean_cache = pargs.contains("--clean-cache");
    // Ignore existing cache but save the new one at the end.
    let force_rebuild = pargs.contains(["-F", "--force"]);

    if clean_cache {
        if let Err(e) = CacheManifest::clean() {
            eprintln!("Warning: {}", e);
        }
    }

    // Determine initial cache state based on the provided flags.
    let mut manifest = if clean_cache {
        CacheManifest::default()
    } else if no_cache || force_rebuild {
        // Start with an empty manifest to force a full rebuild.
        CacheManifest::default()
    } else {
        CacheManifest::load()
    };

    let theme = pendon_tui::Theme::default();
    if pendon_tui::is_interactive_stderr() {
        pendon_tui::render_status_line("Scanning source files...", theme);
    }

    let mut exit = ExitCode::SUCCESS;

    for (task_idx, task) in cfg.tasks.iter().enumerate() {
        let task_name = task
            .name
            .clone()
            .unwrap_or_else(|| format!("task-{}", task_idx));

        let mut total_skipped = 0usize;
        let mut total_processed = 0usize;
        let mut total_skipped_write = 0usize;

        // Hash task config
        let task_config_str = format!(
            "{}:{}:{}:{}:{}",
            task.plugin.as_deref().unwrap_or(""),
            task.markdown_allow_html.unwrap_or(false),
            task.markdown_strip_comments.unwrap_or(false),
            task.wiki_link_prefix.as_deref().unwrap_or(""),
            task.format
        );
        let task_config_hash = crate::cache::hash_content(&task_config_str);

        // Update task in cache
        manifest.update_task(
            task_name.clone(),
            CacheTask {
                config_hash: task_config_hash.clone(),
                input_pattern: task.input.clone(),
                output_pattern: task.output.clone(),
                plugin: task.plugin.clone().unwrap_or_default(),
                format: task.format.clone(),
            },
        );

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
            .into_iter()
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

                // Check cache
                let (should_skip, _changed_deps) =
                    should_skip_file(&manifest, &path_str, &out_path, &task_config_hash);

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
                    &path_str,
                    &out_path,
                    &map,
                    &custom_registry,
                    vicado_hints_override.as_ref(),
                    task_markdown_opts,
                    task_wiki_opts.clone(),
                    deps,
                );

                if !result.success {
                    exit = ExitCode::from(2);
                    continue;
                }

                total_bytes += result.bytes_written;
                if result.skipped_write {
                    total_skipped_write += 1;
                }

                // Update cache
                if let Some((path, entry)) = result.cache_entry {
                    manifest.update_file(path, entry);
                }
            }
        }
        if pendon_tui::is_interactive_stderr() {
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

    // Save cache to disk unless --no-cache was explicitly requested.
    // --force and --clean-cache are allowed to save the new cache state.
    if !no_cache {
        if let Err(e) = manifest.save() {
            eprintln!("Warning: failed to save cache: {}", e);
        }
    }

    if pendon_tui::is_interactive_stderr() {
        pendon_tui::render_severity_line(
            pendon_tui::SeverityLine::Done,
            "All tasks completed",
            theme,
        );
    }
    exit
}
