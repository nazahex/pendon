use pendon_core::{Event, NodeKind, Options};
use pendon_plugin_custom::PluginSpec;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_solid::{render_solid_with_hints, SolidRenderHints};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

use crate::cache::{create_cache_entry, CacheFile};
use crate::config::ConfigTask;
use crate::plugins::{
    build_anchor_options, build_blockquote_options, build_cite_options,
    build_context_inline_pipeline, build_directive_options, build_heading_options,
    build_img_options, build_list_options, build_marker_options, build_section_options,
    build_table_options, has_plugin, latex_target_for_format, load_custom_spec, merge_solid_hints,
    plugin_names, process_stateless_plugin, track_used_spec, DocumentContext,
};
use crate::utils::{extract_frontmatter_for_cite, maybe_pretty, merge_refs_for_context};

pub struct ProcessResult {
    pub success: bool,
    pub bytes_written: usize,
    pub cache_entry: Option<(String, CacheFile)>,
    pub skipped_write: bool,
}

pub fn process_single_file(
    task: &ConfigTask,
    task_name: &str,
    path_str: &str,
    out_path: &str,
    map: &HashMap<String, String>,
    custom_registry: &HashMap<String, PluginSpec>,
    vicado_hints_override: Option<&SolidRenderHints>,
    task_markdown_opts: MarkdownOptions,
    task_wiki_opts: WikiOptions,
    deps: Vec<String>,
) -> ProcessResult {
    let input_text = match fs::read_to_string(path_str) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error: cannot read input file '{}': {}", path_str, e);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let opts = Options {
        strict: task.strict.unwrap_or(false),
        max_doc_bytes: task.max_doc_bytes,
        max_line_len: task.max_line_len,
        max_blank_run: task.max_blank_run,
    };
    let mut events = pendon_core::parse(&input_text, &opts);

    // Run micromatter FIRST if it's in the plugin list
    if let Some(pstr) = task.plugin.as_deref() {
        if has_plugin(Some(pstr), "micromatter") {
            events = pendon_plugin_micromatter::process(&events);
        }
    }

    // Extract frontmatter and build CitationContext
    let cite_options =
        match build_cite_options(task.cite.as_ref(), &task.input, path_str, Some(map)) {
            Ok(o) => o,
            Err(msg) => {
                eprintln!("Error: {}", msg);
                return ProcessResult {
                    success: false,
                    bytes_written: 0,
                    cache_entry: None,
                    skipped_write: false,
                };
            }
        };
    let frontmatter_data = extract_frontmatter_for_cite(&events);
    let front_refs = frontmatter_data
        .as_ref()
        .and_then(|d| d.get("references"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let merged_refs =
        merge_refs_for_context(&front_refs, cite_options.external_references.as_ref());
    let mut document_context = DocumentContext::new(pendon_plugin_cite::CitationContext::new(
        merged_refs,
        cite_options.clone(),
    ));

    let enabled_plugins: HashSet<&str> = plugin_names(task.plugin.as_deref()).into_iter().collect();

    let latex_options = if enabled_plugins.contains("latex") {
        Some(pendon_plugin_latex::LatexOptions {
            target: latex_target_for_format(&task.format),
        })
    } else {
        None
    };

    let img_options = match build_img_options(task.img.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let anchor_options = match build_anchor_options(task.anchor.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let marker_options = match build_marker_options(task.marker.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let directive_options = match build_directive_options(task.directive.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let list_options = match build_list_options(task.list.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let blockquote_options = match build_blockquote_options(task.blockquote.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let section_options = match build_section_options(task.section.as_ref()) {
        Ok(o) => o,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    let inline_pipeline = build_context_inline_pipeline(
        img_options.clone(),
        task_wiki_opts.clone(),
        anchor_options.clone(),
        latex_options,
        &enabled_plugins,
    );

    // Run remaining plugins
    let mut used_custom_specs: Vec<PluginSpec> = Vec::new();
    let mut builtin_hints: Vec<SolidRenderHints> = Vec::new();
    let mut used_quiz = false;
    let mut used_vicado = false;
    let mut anchor_ran = false;
    let mut cite_ran = false;
    let mut markdown_ran = false;
    let mut quiz_pending = false;
    let mut custom_cache: HashMap<String, PluginSpec> = HashMap::new();
    // §13: report each unknown plugin name once per task.
    let mut reported_plugins: HashSet<String> = HashSet::new();

    if let Some(pstr) = task.plugin.as_deref() {
        for name in plugin_names(Some(pstr)) {
            if name == "micromatter" {
                continue;
            }

            if let Some(path) = name.strip_prefix("toml:") {
                let spec = match load_custom_spec(path, &mut custom_cache) {
                    Ok(spec) => spec,
                    Err(msg) => {
                        eprintln!("Error: {}", msg);
                        return ProcessResult {
                            success: false,
                            bytes_written: 0,
                            cache_entry: None,
                            skipped_write: false,
                        };
                    }
                };
                track_used_spec(&mut used_custom_specs, spec.clone());
                events = pendon_plugin_custom::process_with_context(
                    &events,
                    &spec,
                    &inline_pipeline,
                    &mut document_context,
                );
                continue;
            }

            if let Some(processed) =
                process_stateless_plugin(name, &events, &task_wiki_opts, latex_options.as_ref())
            {
                events = processed;
                continue;
            }

            events = match name {
                "quiz" => {
                    used_quiz = true;
                    if let Some(spec) = custom_registry.get("quiz") {
                        track_used_spec(&mut used_custom_specs, spec.clone());
                    }
                    if markdown_ran {
                        pendon_plugin_quiz::process(&events)
                    } else {
                        quiz_pending = true;
                        events
                    }
                }
                "img" => {
                    let img_opts = &img_options;
                    let result = pendon_plugin_img::process_with_context(
                        &events,
                        &img_opts,
                        &inline_pipeline,
                        &mut document_context,
                    );
                    if let Some(hints) = pendon_plugin_img::solid_hints(img_opts) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "table" => {
                    let table_opts = match build_table_options(task.table.as_ref()) {
                        Ok(o) => o,
                        Err(msg) => {
                            eprintln!("Error: {}", msg);
                            return ProcessResult {
                                success: false,
                                bytes_written: 0,
                                cache_entry: None,
                                skipped_write: false,
                            };
                        }
                    };
                    let result = pendon_plugin_table::process_with_context(
                        &events,
                        &table_opts,
                        &inline_pipeline,
                        &mut document_context,
                    );
                    if let Some(hints) = pendon_plugin_table::solid_hints(&table_opts) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "heading" => {
                    let mut opts = match build_heading_options(task.heading.as_ref()) {
                        Ok(o) => o,
                        Err(msg) => {
                            eprintln!("Error: {}", msg);
                            return ProcessResult {
                                success: false,
                                bytes_written: 0,
                                cache_entry: None,
                                skipped_write: false,
                            };
                        }
                    };
                    // §9.5: when the section outline owns the ids, the heading
                    // yields its id (it transfers to the section).
                    opts.section_owns_id = enabled_plugins.contains("section");
                    let result = pendon_plugin_heading::process(&events, &opts);
                    if let Some(hints) = pendon_plugin_heading::solid_hints(&opts) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "anchor" => {
                    anchor_ran = true;
                    pendon_plugin_anchor::process(&events, &anchor_options)
                }
                "cite" => {
                    cite_ran = true;
                    document_context.process_citations(&events)
                }
                "marker" => {
                    // §10.1: markers are plain text patterns, so this plugin runs
                    // before markdown like the other construct plugins.
                    let result = pendon_plugin_marker::process(&events, &marker_options);
                    if let Some(hints) = pendon_plugin_marker::solid_hints(&marker_options) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "directive" => {
                    // §10.2/§10.3: directives are text constructs, so this plugin
                    // runs before markdown like the other construct plugins.
                    let result = pendon_plugin_directive::process(&events, &directive_options);
                    if let Some(hints) = pendon_plugin_directive::solid_hints(&directive_options) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "blockquote" => {
                    // §9.2: `>` stays plain Markdown; this plugin only binds the
                    // extras a quote may carry, so it runs before markdown re-lexes
                    // the stripped body as blocks.
                    let result = pendon_plugin_blockquote::process(&events, &blockquote_options);
                    if let Some(hints) = pendon_plugin_blockquote::solid_hints(&blockquote_options)
                    {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "section" => {
                    // §9.5: the section outline runs before markdown so it can see
                    // the raw heading text, the section decorator and the level
                    // markers (`>---<` would otherwise be a blockquote). Markdown
                    // then parses the interior of every `Section` it emits.
                    let result = pendon_plugin_section::process(&events, &section_options);
                    if let Some(hints) = pendon_plugin_section::solid_hints(&section_options) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "list" => {
                    // §9.3: the list markers stay plain Markdown; this plugin only
                    // binds a container decorator to the list below it, so it runs
                    // before markdown parses the list.
                    let result = pendon_plugin_list::process(&events, &list_options);
                    if let Some(hints) = pendon_plugin_list::solid_hints(&list_options) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "vicado" => {
                    used_vicado = true;
                    pendon_plugin_vicado::process(&events)
                }
                "markdown" => {
                    let result =
                        pendon_plugin_markdown::process_with_options(&events, task_markdown_opts);
                    markdown_ran = true;
                    if quiz_pending {
                        quiz_pending = false;
                        pendon_plugin_quiz::process(&result)
                    } else {
                        result
                    }
                }
                other => {
                    if let Some(spec) = custom_registry.get(other) {
                        let spec = spec.clone();
                        track_used_spec(&mut used_custom_specs, spec.clone());
                        pendon_plugin_custom::process_with_context(
                            &events,
                            &spec,
                            &inline_pipeline,
                            &mut document_context,
                        )
                    } else {
                        // §13: an unknown plugin name used to be ignored without a
                        // word. Report it once per name; the build still succeeds.
                        if reported_plugins.insert(other.to_string()) {
                            eprintln!(
                                "Warning: [{task_name}] unknown plugin '{other}' in task.plugin; \
                                 it is ignored (§13)"
                            );
                        }
                        events
                    }
                }
            };
        }
    }
    if quiz_pending {
        events = pendon_plugin_quiz::process(&events);
    }
    if used_quiz {
        builtin_hints.push(pendon_plugin_quiz::solid_hints());
    }
    if used_vicado {
        if let Some(override_hints) = vicado_hints_override {
            builtin_hints.push(override_hints.clone());
        } else {
            builtin_hints.push(pendon_plugin_vicado::solid_hints());
        }
    }
    // Finalize cite
    if cite_ran {
        let (processed_events, diagnostics) = document_context.finalize_citations(&events);
        events = processed_events;

        if !diagnostics.is_empty() {
            let insert_at = events
                .iter()
                .position(|e| matches!(e, Event::StartNode(NodeKind::Document)))
                .map(|i| i + 1)
                .unwrap_or(0);
            for (off, diag) in diagnostics.into_iter().enumerate() {
                events.insert(insert_at + off, diag);
            }
        }

        if let Some(hints) = pendon_plugin_cite::solid_hints(&cite_options) {
            builtin_hints.push(hints);
        }
    }
    if anchor_ran {
        if let Some(hints) = pendon_plugin_anchor::solid_hints(&anchor_options) {
            builtin_hints.push(hints);
        }
    }

    let pretty = task.pretty.unwrap_or(false);
    let rendered = match task.format.as_str() {
        "json" => pendon_renderer_json::render_to_string(&events).map(|s| maybe_pretty(&s, pretty)),
        "events" => pendon_renderer_events::render_events_to_string(&events)
            .map(|s| maybe_pretty(&s, pretty)),
        "ast" => {
            if pretty {
                pendon_renderer_ast::render_ast_to_string_pretty(&events)
            } else {
                pendon_renderer_ast::render_ast_to_string(&events)
            }
        }
        "html" => Ok(if pretty {
            pendon_renderer_html::render_html_pretty(&events)
        } else {
            pendon_renderer_html::render_html(&events)
        }),
        "solid" => {
            let hints = merge_solid_hints(&used_custom_specs, &builtin_hints);
            Ok(match hints.as_ref() {
                Some(h) => render_solid_with_hints(&events, Some(h)),
                None => pendon_renderer_solid::render_solid(&events),
            })
        }
        other => {
            eprintln!("Error: unsupported format in task: {}", other);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    };

    match rendered {
        Ok(out_str) => {
            if let Some(parent) = Path::new(out_path).parent() {
                let _ = fs::create_dir_all(parent);
            }

            // CONTENT-AWARE WRITE: check if the content is the same as existing file to avoid unnecessary writes
            let mut skipped_write = false;
            if let Ok(existing_content) = fs::read_to_string(out_path) {
                if existing_content == out_str {
                    // The content is the same, skip writing to disk, avoid frontend massive hot reload
                    skipped_write = true;
                }
            }

            if !skipped_write {
                if let Err(e) = fs::write(out_path, out_str.clone()) {
                    eprintln!("Error: cannot write output '{}': {}", out_path, e);
                    return ProcessResult {
                        success: false,
                        bytes_written: 0,
                        cache_entry: None,
                        skipped_write: false,
                    };
                }
            }

            let bytes_written = out_str.len();

            // Keep updating the cache entry even if the file was not written to disk
            let cache_entry = match create_cache_entry(task_name, path_str, out_path, deps) {
                Ok(entry) => Some((path_str.to_string(), entry)),
                Err(_) => None,
            };

            return ProcessResult {
                success: true,
                bytes_written,
                cache_entry,
                skipped_write,
            };
        }
        Err(e) => {
            eprintln!("Error: render failed: {}", e);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
                skipped_write: false,
            };
        }
    }
}
