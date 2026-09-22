use pendon_core::{Event, NodeKind, Options, Pipeline};
use pendon_plugin_custom::PluginSpec;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_solid::{render_solid_with_hints, SolidRenderHints};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::cache::{create_cache_entry, CacheFile};
use crate::config::ConfigTask;
use crate::plugins::{
    build_anchor_options, build_cite_options, build_table_options, merge_solid_hints,
    track_used_spec,
};
use crate::utils::{extract_frontmatter_for_cite, maybe_pretty, merge_refs_for_context};

pub struct ProcessResult {
    pub success: bool,
    pub bytes_written: usize,
    pub cache_entry: Option<(String, CacheFile)>,
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
        if pstr
            .split(',')
            .map(|s| s.trim())
            .any(|s| s == "micromatter")
        {
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
    let cite_ctx = pendon_plugin_cite::CitationContext::new(merged_refs, cite_options.clone());

    let cite_ctx_shared = std::sync::Arc::new(std::sync::Mutex::new(cite_ctx));

    // Build Inline Pipeline
    let mut inline_pipeline = Pipeline::new();

    let img_opts_for_pipeline = task.img.clone().unwrap_or_default();

    let mut img_inner_pipeline = Pipeline::new();
    let cite_inner = cite_ctx_shared.clone();
    img_inner_pipeline.add(move |ev: Vec<Event>| cite_inner.lock().unwrap().process_events(&ev));
    let wiki_inner = task_wiki_opts.clone();
    img_inner_pipeline.add(move |ev: Vec<Event>| {
        pendon_plugin_wiki::process_with_options(&ev, wiki_inner.clone())
    });
    let anchor_inner = build_anchor_options(task.anchor.as_ref());
    img_inner_pipeline.add(move |ev: Vec<Event>| pendon_plugin_anchor::process(&ev, &anchor_inner));

    let img_opts_clone = img_opts_for_pipeline.clone();
    inline_pipeline.add(move |ev: Vec<Event>| {
        pendon_plugin_img::process(&ev, &img_opts_clone, &img_inner_pipeline)
    });

    let cite_for_pipeline = cite_ctx_shared.clone();
    inline_pipeline
        .add(move |ev: Vec<Event>| cite_for_pipeline.lock().unwrap().process_events(&ev));

    let wiki_opts = task_wiki_opts.clone();
    inline_pipeline.add(move |ev: Vec<Event>| {
        pendon_plugin_wiki::process_with_options(&ev, wiki_opts.clone())
    });

    let anchor_opts = build_anchor_options(task.anchor.as_ref());
    inline_pipeline.add(move |ev: Vec<Event>| pendon_plugin_anchor::process(&ev, &anchor_opts));

    // Run remaining plugins
    let mut used_custom_specs: Vec<PluginSpec> = Vec::new();
    let mut builtin_hints: Vec<SolidRenderHints> = Vec::new();
    let mut used_quiz = false;
    let mut used_vicado = false;
    let anchor_options = build_anchor_options(task.anchor.as_ref());
    let mut anchor_ran = false;
    let mut cite_ran = false;
    let mut markdown_ran = false;
    let mut quiz_pending = false;
    let mut custom_cache: HashMap<String, PluginSpec> = HashMap::new();

    if let Some(pstr) = task.plugin.as_deref() {
        for name in pstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if name == "micromatter" {
                continue;
            }

            if let Some(path) = name.strip_prefix("toml:") {
                let spec = match custom_cache.get(path) {
                    Some(existing) => existing.clone(),
                    None => match pendon_plugin_custom::load_spec_from_path(path) {
                        Ok(s) => {
                            custom_cache.insert(path.to_string(), s.clone());
                            s
                        }
                        Err(msg) => {
                            eprintln!("Error: {}", msg);
                            return ProcessResult {
                                success: false,
                                bytes_written: 0,
                                cache_entry: None,
                            };
                        }
                    },
                };
                track_used_spec(&mut used_custom_specs, spec.clone());
                events = pendon_plugin_custom::process(&events, &spec);
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
                "dialog" => pendon_plugin_dialog::process(&events),
                "img" => {
                    let img_opts = task.img.clone().unwrap_or_default();
                    let result = pendon_plugin_img::process(&events, &img_opts, &inline_pipeline);
                    if let Some(hints) = pendon_plugin_img::solid_hints(&img_opts) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "table" => {
                    let table_opts = build_table_options(task.table.as_ref());
                    let result =
                        pendon_plugin_table::process(&events, &table_opts, &inline_pipeline);
                    if let Some(hints) = pendon_plugin_table::solid_hints(&table_opts) {
                        builtin_hints.push(hints);
                    }
                    result
                }
                "heading" => {
                    let opts = task.heading.clone().unwrap_or_default();
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
                    cite_ctx_shared.lock().unwrap().process_events(&events)
                }
                "latex" => pendon_plugin_latex::process(&events),
                "wiki" => pendon_plugin_wiki::process_with_options(&events, task_wiki_opts.clone()),
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
                "sectionize" => pendon_plugin_sectionize::process(&events),
                "extract-heading" => pendon_plugin_extract_heading::process(&events),
                "syntect" => pendon_plugin_codeblock_syntect::process(&events),
                other => {
                    if let Some(spec) = custom_registry.get(other) {
                        let spec = spec.clone();
                        track_used_spec(&mut used_custom_specs, spec.clone());
                        pendon_plugin_custom::process(&events, &spec)
                    } else {
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
        let ctx = cite_ctx_shared.lock().unwrap();
        let final_cites = ctx.get_cites();
        let final_refs = ctx.get_used_references();
        let cite_opts_final = ctx.options().clone();
        drop(ctx);

        let diagnostics = cite_ctx_shared.lock().unwrap().drain_diagnostics();

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

        events = cite_ctx_shared
            .lock()
            .unwrap()
            .replace_section_markers(&events);

        pendon_plugin_cite::update_frontmatter_in_events(
            &mut events,
            &final_cites,
            &final_refs,
            &cite_opts_final,
        );

        if let Some(hints) = pendon_plugin_cite::solid_hints(&cite_opts_final) {
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
            };
        }
    };

    match rendered {
        Ok(out_str) => {
            if let Some(parent) = Path::new(out_path).parent() {
                let _ = fs::create_dir_all(parent);
            }
            if let Err(e) = fs::write(out_path, out_str.clone()) {
                eprintln!("Error: cannot write output '{}': {}", out_path, e);
                return ProcessResult {
                    success: false,
                    bytes_written: 0,
                    cache_entry: None,
                };
            } else {
                let bytes_written = out_str.len();

                let cache_entry = match create_cache_entry(task_name, path_str, out_path, deps) {
                    Ok(entry) => Some((path_str.to_string(), entry)),
                    Err(_) => None,
                };

                return ProcessResult {
                    success: true,
                    bytes_written,
                    cache_entry,
                };
            }
        }
        Err(e) => {
            eprintln!("Error: render failed: {}", e);
            return ProcessResult {
                success: false,
                bytes_written: 0,
                cache_entry: None,
            };
        }
    }
}
