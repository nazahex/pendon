use std::process::ExitCode;

use pendon_core::{parse, Options};
use pendon_plugin_anchor::AnchorOptions;
use pendon_plugin_cite::CiteOptions;
use pendon_plugin_img::ImgOptions;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_quiz::solid_hints as quiz_solid_hints;
use pendon_plugin_table::TableOptions;
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
use plugins::merge_solid_hints;
use utils::maybe_pretty;

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

    let events = parse(
        &input,
        &Options {
            strict: args.strict,
            max_doc_bytes: args.max_doc_bytes,
            max_line_len: args.max_line_len,
            max_blank_run: args.max_blank_run,
        },
    );

    let markdown_opts = MarkdownOptions {
        allow_html: args.markdown_allow_html,
        strip_comments: args.markdown_strip_comments,
    };
    let wiki_opts = WikiOptions {
        link_prefix: args.wiki_link_prefix.clone(),
    };

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
        for name in pstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
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
                            return ExitCode::from(2);
                        }
                    },
                };
                plugins::track_used_spec(&mut used_custom_specs, spec.clone());
                ev = pendon_plugin_custom::process(&ev, &spec);
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
                "dialog" => pendon_plugin_dialog::process(&ev),
                "img" => {
                    let empty_pipeline = pendon_core::Pipeline::default();
                    pendon_plugin_img::process(&ev, &ImgOptions::default(), &empty_pipeline)
                }
                "table" => {
                    let empty_pipeline = pendon_core::Pipeline::default();
                    pendon_plugin_table::process(&ev, &TableOptions::default(), &empty_pipeline)
                }
                "heading" => pendon_plugin_heading::process(
                    &ev,
                    &pendon_plugin_heading::HeadingOptions::default(),
                ),
                "anchor" => pendon_plugin_anchor::process(&ev, &AnchorOptions::default()),
                "cite" => pendon_plugin_cite::process(&ev, &CiteOptions::default()),
                "latex" => pendon_plugin_latex::process(&ev),
                "wiki" => pendon_plugin_wiki::process_with_options(&ev, wiki_opts.clone()),
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
                "sectionize" => pendon_plugin_sectionize::process(&ev),
                "extract-heading" => pendon_plugin_extract_heading::process(&ev),
                "syntect" => pendon_plugin_codeblock_syntect::process(&ev),
                other => {
                    if let Some(spec) = custom_cache.get(other) {
                        plugins::track_used_spec(&mut used_custom_specs, spec.clone());
                        pendon_plugin_custom::process(&ev, spec)
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
