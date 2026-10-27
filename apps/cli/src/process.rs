use pendon_core::{Event, Options};
use pendon_plugin_custom::PluginSpec;
use pendon_plugin_extract::ExtractOutput;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_solid::{render_solid_with_hints, DataExport, SolidRenderHints};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::cache::{create_entry, write_atomically, CacheEntry};
use crate::chain::{self, ChainConfig, ChainState};
use crate::config::ConfigTask;
use crate::plugins::{
    build_anchor_options, build_bind_injects, build_blockquote_options, build_cite_options,
    build_directive_options, build_extract_options, build_heading_options, build_image_options,
    build_list_options, build_marker_options, build_number_options, build_section_options,
    build_table_options, has_plugin, merge_solid_hints, DocumentContext,
};
use crate::utils::{
    extract_frontmatter_for_cite, maybe_pretty, merge_refs_for_context, substitute_output,
};

pub struct ProcessResult {
    pub success: bool,
    pub bytes_written: usize,
    pub cache_entry: Option<CacheEntry>,
    pub skipped_write: bool,
}

/// The result of a file that cannot be rendered. A task error is reported on
/// stderr and renders nothing; it must not abort the other files of the run.
fn failure() -> ProcessResult {
    ProcessResult {
        success: false,
        bytes_written: 0,
        cache_entry: None,
        skipped_write: false,
    }
}

/// Unwraps a plugin-option builder (`build_*_options`), reporting its message the
/// way the CLI reports any task error.
macro_rules! plugin_options {
    ($call:expr) => {
        match $call {
            Ok(options) => options,
            Err(msg) => {
                eprintln!("Error: {}", msg);
                return failure();
            }
        }
    };
}

pub fn process_single_file(
    task: &ConfigTask,
    task_name: &str,
    config_hash: &str,
    path_str: &str,
    out_path: &str,
    map: &HashMap<String, String>,
    custom_registry: &HashMap<String, PluginSpec>,
    vicado_hints_override: Option<&SolidRenderHints>,
    task_markdown_opts: MarkdownOptions,
    task_wiki_opts: WikiOptions,
    mut deps: Vec<String>,
) -> ProcessResult {
    let mut input_text = match fs::read_to_string(path_str) {
        Ok(text) => text,
        Err(e) => {
            eprintln!("Error: cannot read input file '{}': {}", path_str, e);
            return failure();
        }
    };

    // `plugin-bind` scans `{{{lang[var] … }}}` data blocks **before** the lexer
    // sees them: a payload's lines (`#`, `-`, a blank line) would otherwise be
    // read as markup (ADR-0004, `docs/rfc/plugin-bind.md`).
    let bind = if has_plugin(task.plugin.as_deref(), "bind") {
        // A `(path)` payload resolves against the **source file's** directory,
        // never the process CWD (§2.7), so the depth of `path_str` does not
        // matter and `../` walks up from the document itself.
        let extracted = match Path::new(path_str).parent() {
            Some(base) => pendon_plugin_bind::extract_with_base(&input_text, base),
            None => pendon_plugin_bind::extract(&input_text),
        };
        input_text = extracted.text.clone();
        // Every external file is an input of this page: without this the cache
        // would not notice an edit to it (§2.7 decision 8).
        deps.extend(
            extracted
                .external_files
                .iter()
                .map(|path| path.display().to_string()),
        );
        Some(extracted)
    } else {
        None
    };

    // §6.1 of `workflows/numbering-extract-type-export.md`:
    // `[[task.bind.inject]]` publishes whole files as module-level exports. They
    // are resolved per page (dynamic segments substituted, source relative to
    // the project root) and recorded as dependencies exactly like a `(path)`
    // block's file, so editing one re-renders the page. Like every other
    // config-driven plugin, the section only applies when `bind` is enabled.
    let bind_injects = if has_plugin(task.plugin.as_deref(), "bind") {
        plugin_options!(build_bind_injects(task.bind.as_ref(), map, &mut deps))
    } else {
        Vec::new()
    };

    let opts = Options {
        strict: task.strict.unwrap_or(false),
        max_doc_bytes: task.max_doc_bytes,
        max_line_len: task.max_line_len,
        max_blank_run: task.max_blank_run,
    };
    let mut events = pendon_core::parse(&input_text, &opts);
    if let Some(bind) = bind.as_ref() {
        events.extend(bind.diagnostics.iter().cloned());
    }

    // The task's own options, over the plugin defaults. `table` and `heading`
    // are only built when the task enables them, so an unused table's config
    // never fails a page.
    let mut config = plugin_options!(ChainConfig::new(
        task_name,
        task.plugin.as_deref(),
        &task.format,
        task_markdown_opts,
        task_wiki_opts.clone(),
    ));
    config.vicado_hints = vicado_hints_override.cloned();
    config.image = plugin_options!(build_image_options(task.image.as_ref()));
    config.anchor = plugin_options!(build_anchor_options(task.anchor.as_ref()));
    config.marker = plugin_options!(build_marker_options(task.marker.as_ref()));
    config.directive = plugin_options!(build_directive_options(task.directive.as_ref()));
    config.list = plugin_options!(build_list_options(task.list.as_ref()));
    config.blockquote = plugin_options!(build_blockquote_options(task.blockquote.as_ref()));
    config.section = plugin_options!(build_section_options(task.section.as_ref()));
    if config.has("table") {
        config.table = plugin_options!(build_table_options(task.table.as_ref()));
    }
    if config.has("heading") {
        let mut heading = plugin_options!(build_heading_options(task.heading.as_ref()));
        heading.section_owns_id = config.has("section");
        config.heading = heading;
    }
    // ADR-0010: the numbering blocks are validated for every task, so a typo in
    // a `type` selector fails even before the plugin is enabled; the pass itself
    // only runs when `number` is in `task.plugin`.
    config.number = plugin_options!(build_number_options(task));
    // Same rule for extraction: the blocks are validated for every task, and the
    // pass itself only runs when `extract` is in `task.plugin`.
    config.extract = plugin_options!(build_extract_options(task));

    // Text stages first: a leading `---` block only becomes a node here, and the
    // citation context is built from the references it holds.
    events = chain::prelude(events, &config);

    let cite_options = plugin_options!(build_cite_options(
        task.cite.as_ref(),
        &task.input,
        path_str,
        Some(map)
    ));
    let frontmatter_data = extract_frontmatter_for_cite(&events);
    let front_refs = frontmatter_data
        .as_ref()
        .and_then(|d| d.get("references"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let merged_refs =
        merge_refs_for_context(&front_refs, cite_options.external_references.as_ref());
    let document_context = DocumentContext::new(pendon_plugin_cite::CitationContext::new(
        merged_refs,
        cite_options.clone(),
    ));

    let pipeline = config.pipeline();
    let mut state = ChainState::new(document_context, custom_registry.clone(), pipeline);

    events = chain::run(events, &config, &mut state);

    // `resolve` runs after every plugin has emitted its extras attributes
    // (ADR-0004) and **before** the citation export: an `mdp` payload may cite,
    // and the citation list is the page's, not the payload's.
    if let Some(bind) = bind.as_ref() {
        let mut render = |markdown: &str, mode: pendon_plugin_bind::MdpMode| {
            chain::render_mdp(markdown, mode, &config, &mut state)
        };
        events = pendon_plugin_bind::resolve_with(&events, &bind.registry, &mut render);
    }

    // Finalize cite
    if state.cite_ran {
        let (diagnostics, citations, references, options) = state.context.finalize_citations();

        if !diagnostics.is_empty() {
            let insert_at = events
                .iter()
                .position(|e| matches!(e, Event::StartNode(pendon_core::NodeKind::Document)))
                .map(|i| i + 1)
                .unwrap_or(0);
            for (off, diag) in diagnostics.into_iter().enumerate() {
                events.insert(insert_at + off, diag);
            }
        }

        state.hints.push(pendon_plugin_cite::solid_hints(
            &options,
            &citations,
            &references,
        ));
    }

    chain::finish(&config, &mut state);

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
            // ADR-0009 rule 5: only the `solid` format emits `export const`, so
            // the injected payloads join the hints here and nowhere else.
            if !bind_injects.is_empty() {
                state.hints.push(SolidRenderHints {
                    exports: bind_injects,
                    ..SolidRenderHints::default()
                });
            }
            // P.1.b: a block's `export` is the same `export const` mechanism, and
            // it is registered after the bind payloads so a `micromatter.extract`
            // payload wins over the automatic `frontmatter` export it duplicates.
            // Every other task format ignores `export` (docs/issues/plugin-extract.md).
            if !state.extract.exports.is_empty() {
                state.hints.push(SolidRenderHints {
                    exports: state
                        .extract
                        .exports
                        .iter()
                        .map(|(name, value)| {
                            DataExport::plugin("extract", name.clone(), value.clone())
                        })
                        .collect(),
                    ..SolidRenderHints::default()
                });
            }
            let hints = merge_solid_hints(&state.used_specs, &state.hints);
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

            // P.1.a: the extraction side files are written only after the page
            // itself rendered, so a failed page leaves no half-written data
            // behind. A path two pages resolve to is overwritten (last page wins).
            // They are outputs, so the cache record remembers them (T4.6): a
            // deleted side file makes the next run rewrite it.
            let side_outputs = match write_extract_files(&state.extract, map) {
                Ok(paths) => paths,
                Err(message) => {
                    eprintln!("Error: {message}");
                    return failure();
                }
            };

            // Keep updating the cache entry even if the file was not written to disk
            let cache_entry = create_entry(
                task_name,
                config_hash,
                path_str,
                out_path,
                deps,
                side_outputs,
            )
            .ok();

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

/// Writes every side file one document extracted (P.1.a of
/// `docs/issues/plugin-extract.md`), returning the paths it wrote.
///
/// A block's `output` is a config path like `output`: relative to the project
/// root, with the page's dynamic segments substituted per document. Two pages
/// that resolve to the same file overwrite it — the format is the block's, never
/// the extension's, so `format = "json"` writes JSON to a `.yaml` path.
///
/// Writing is atomic ([`cache::write_atomically`]): a reader never sees half a
/// JSON document, and a failed write is a task error instead of a truncated file.
fn write_extract_files(
    output: &ExtractOutput,
    vars: &HashMap<String, String>,
) -> Result<Vec<String>, String> {
    let mut written: Vec<String> = Vec::with_capacity(output.files.len());
    for file in &output.files {
        let path = substitute_output(&file.path, vars)
            .map_err(|cause| format!("task extract output '{}': {cause}", file.path))?;
        let rendered = file
            .render()
            .map_err(|cause| format!("task extract output '{path}': {cause}"))?;
        write_atomically(Path::new(&path), &rendered)
            .map_err(|e| format!("task extract output: {e}"))?;
        written.push(path);
    }
    Ok(written)
}
