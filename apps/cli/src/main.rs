use std::process::ExitCode;

use pendon_core::{parse, Options};
use pendon_plugin_anchor::AnchorOptions;
use pendon_plugin_cite::CiteOptions;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_quiz::solid_hints as quiz_solid_hints;
use pendon_plugin_vicado::solid_hints as vicado_solid_hints;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_json::render_to_string;
use pendon_renderer_solid::{render_solid_with_hints, SolidRenderHints};

mod cache;
mod cli;
mod config;
mod plugins;
mod process;
mod run;
mod utils;

use cli::{parse_args, read_input};
use plugins::{
    build_anchor_options, build_context_inline_pipeline, load_custom_spec, merge_solid_hints,
    plugin_names, DocumentContext,
};
use utils::{extract_frontmatter_for_cite, maybe_pretty, merge_refs_for_context};

fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() == Some("run") {
        return run::run_from_config();
    }

    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };

    let format = args.format.as_deref().unwrap_or("json");

    let use_tui = args.tui && pendon_tui::is_interactive_stderr();
    let maybe_spinner = if use_tui {
        Some(pendon_tui::widgets::spinner::Spinner::start(
            "Reading input".to_string(),
            pendon_tui::Theme::default(),
        ))
    } else {
        None
    };

    let input = match read_input(&args) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };

    if let Some(sp) = maybe_spinner {
        sp.stop();
    }

    let mut events = parse(
        &input,
        &Options {
            strict: args.strict,
            max_doc_bytes: args.max_doc_bytes,
            max_line_len: args.max_line_len,
            max_blank_run: args.max_blank_run,
        },
    );

    if let Some(plugins) = args.plugin.as_deref() {
        if plugins::has_plugin(Some(plugins), "micromatter") {
            events = pendon_plugin_micromatter::process(&events);
        }
    }

    let markdown_opts = MarkdownOptions {
        allow_html: args.markdown_allow_html,
        strip_comments: args.markdown_strip_comments,
    };
    let enabled_plugins: std::collections::HashSet<&str> =
        plugin_names(args.plugin.as_deref()).into_iter().collect();
    let latex_options = if enabled_plugins.contains("latex") {
        Some(pendon_plugin_latex::LatexOptions {
            target: plugins::latex_target_for_format(format),
        })
    } else {
        None
    };
    let wiki_opts = WikiOptions {
        link_prefix: args.wiki_link_prefix.clone(),
        latex: latex_options,
    };

    let frontmatter = extract_frontmatter_for_cite(&events);
    let references = frontmatter
        .as_ref()
        .and_then(|data| data.get("references"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let references = merge_refs_for_context(&references, None);
    let mut document_context = DocumentContext::new(pendon_plugin_cite::CitationContext::new(
        references,
        CiteOptions::default(),
    ));
    let inline_pipeline = build_context_inline_pipeline(
        pendon_plugin_img::ImgOptions::default(),
        wiki_opts.clone(),
        build_anchor_options(None),
        latex_options,
        &enabled_plugins,
    );

    let mut used_custom_specs: Vec<pendon_plugin_custom::PluginSpec> = Vec::new();
    let mut custom_cache: std::collections::HashMap<String, pendon_plugin_custom::PluginSpec> =
        std::collections::HashMap::new();
    let mut builtin_hints: Vec<SolidRenderHints> = Vec::new();
    let mut used_quiz = false;
    let mut used_vicado = false;

    let events = if let Some(pstr) = args.plugin.as_deref() {
        let mut ev = events;
        let mut markdown_ran = false;
        let mut quiz_pending = false;
        for name in plugin_names(Some(pstr)) {
            if name == "micromatter" {
                continue;
            }
            if let Some(path) = name.strip_prefix("toml:") {
                let spec = match load_custom_spec(path, &mut custom_cache) {
                    Ok(spec) => spec,
                    Err(msg) => {
                        eprintln!("Error: {}", msg);
                        return ExitCode::from(2);
                    }
                };
                plugins::track_used_spec(&mut used_custom_specs, spec.clone());
                ev = pendon_plugin_custom::process_with_context(
                    &ev,
                    &spec,
                    &inline_pipeline,
                    &mut document_context,
                );
                continue;
            }

            if let Some(processed) =
                plugins::process_stateless_plugin(name, &ev, &wiki_opts, latex_options.as_ref())
            {
                ev = processed;
                continue;
            }

            ev = match name {
                "micromatter" => pendon_plugin_micromatter::process(&ev),
                "quiz" => {
                    used_quiz = true;
                    if markdown_ran {
                        pendon_plugin_quiz::process(&ev)
                    } else {
                        quiz_pending = true;
                        ev
                    }
                }
                "img" => pendon_plugin_img::process_with_context(
                    &ev,
                    &pendon_plugin_img::ImgOptions::default(),
                    &inline_pipeline,
                    &mut document_context,
                ),
                "table" => pendon_plugin_table::process_with_context(
                    &ev,
                    &pendon_plugin_table::TableOptions::default(),
                    &inline_pipeline,
                    &mut document_context,
                ),
                "heading" => pendon_plugin_heading::process(
                    &ev,
                    &pendon_plugin_heading::HeadingOptions::default(),
                ),
                "anchor" => pendon_plugin_anchor::process(&ev, &AnchorOptions::default()),
                "cite" => document_context.process_citations(&ev),
                "vicado" => {
                    used_vicado = true;
                    pendon_plugin_vicado::process(&ev)
                }
                "markdown" => {
                    let processed =
                        pendon_plugin_markdown::process_with_options(&ev, markdown_opts);
                    markdown_ran = true;
                    if quiz_pending {
                        quiz_pending = false;
                        pendon_plugin_quiz::process(&processed)
                    } else {
                        processed
                    }
                }
                other => {
                    if let Some(spec) = custom_cache.get(other) {
                        plugins::track_used_spec(&mut used_custom_specs, spec.clone());
                        pendon_plugin_custom::process_with_context(
                            &ev,
                            &spec,
                            &inline_pipeline,
                            &mut document_context,
                        )
                    } else {
                        ev
                    }
                }
            };
        }
        if quiz_pending {
            ev = pendon_plugin_quiz::process(&ev);
        }
        ev
    } else {
        events
    };

    let events = if enabled_plugins.contains("cite") {
        let (events, cite_diagnostics) = document_context.finalize_citations(&events);
        let mut events = events;
        events.extend(cite_diagnostics);
        events
    } else {
        events
    };

    if used_quiz {
        builtin_hints.push(quiz_solid_hints());
    }
    if used_vicado {
        builtin_hints.push(vicado_solid_hints());
    }

    let has_error = events.iter().any(|e| match e {
        pendon_core::Event::Diagnostic { severity, .. } => {
            matches!(severity, pendon_core::Severity::Error)
        }
        _ => false,
    });

    match format {
        "json" => match render_to_string(&events) {
            Ok(s) => {
                println!("{}", maybe_pretty(&s, args.pretty));
                if has_error {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("Error: failed to serialize JSON: {}", e);
                ExitCode::from(2)
            }
        },
        "events" => match pendon_renderer_events::render_events_to_string(&events) {
            Ok(s) => {
                println!("{}", maybe_pretty(&s, args.pretty));
                if has_error {
                    ExitCode::from(2)
                } else {
                    ExitCode::SUCCESS
                }
            }
            Err(e) => {
                eprintln!("Error: failed to serialize events JSON: {}", e);
                ExitCode::from(2)
            }
        },
        "ast" => {
            let res = if args.pretty {
                pendon_renderer_ast::render_ast_to_string_pretty(&events)
            } else {
                pendon_renderer_ast::render_ast_to_string(&events)
            };
            match res {
                Ok(s) => {
                    println!("{}", s);
                    if has_error {
                        ExitCode::from(2)
                    } else {
                        ExitCode::SUCCESS
                    }
                }
                Err(e) => {
                    eprintln!("Error: failed to serialize AST JSON: {}", e);
                    ExitCode::from(2)
                }
            }
        }
        "html" => {
            let s = pendon_renderer_html::render_html(&events);
            println!("{}", s);
            if has_error {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        "solid" => {
            let hints = merge_solid_hints(&used_custom_specs, &builtin_hints);
            let s = match hints.as_ref() {
                Some(h) => render_solid_with_hints(&events, Some(h)),
                None => pendon_renderer_solid::render_solid(&events),
            };
            println!("{}", s);
            if has_error {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        other => {
            eprintln!(
                "Error: unsupported format '{}'. Try --format json|events|ast|html|solid",
                other
            );
            ExitCode::from(2)
        }
    }
}
