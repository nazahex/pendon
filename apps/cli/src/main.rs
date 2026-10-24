use std::process::ExitCode;

use pendon_core::{parse, Options};
use pendon_plugin_cite::CiteOptions;
use pendon_plugin_markdown::MarkdownOptions;
use pendon_plugin_wiki::WikiOptions;
use pendon_renderer_json::render_to_string;
use pendon_renderer_solid::render_solid_with_hints;

mod cache;
mod chain;
mod cli;
mod components;
mod config;
mod hook;
mod plugins;
mod process;
mod run;
mod utils;

use cli::{parse_args, read_input};
use plugins::{merge_solid_hints, DocumentContext};
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

    // `--quiet` silences the report, so it also disables the `--tui` spinner
    // (`docs/spec/CLI-REPORT.md` §5).
    let use_tui = args.tui && !args.quiet && pendon_tui::is_interactive_stderr();

    let maybe_spinner = if use_tui {
        Some(pendon_tui::widgets::spinner::Spinner::start(
            "Reading input".to_string(),
            pendon_tui::Theme::default(),
        ))
    } else {
        None
    };

    let mut input = match read_input(&args) {
        Ok(s) => s,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };

    if let Some(sp) = maybe_spinner {
        sp.stop();
    }

    // `plugin-bind` scans `{{{lang[var] … }}}` data blocks **before** the lexer
    // sees them: a payload's lines (`#`, `-`, a blank line) would otherwise be
    // read as markup (ADR-0004, `docs/rfc/plugin-bind.md`).
    let bind = if plugins::has_plugin(args.plugin.as_deref(), "bind") {
        // A `(path)` payload resolves against the **source file's** directory
        // (§2.7); a document piped through stdin has none, and a `(path)` block
        // is then an error rather than a silent CWD-relative guess.
        let extracted = match args
            .input
            .as_deref()
            .and_then(|path| std::path::Path::new(path).parent())
        {
            Some(base) => pendon_plugin_bind::extract_with_base(&input, base),
            None => pendon_plugin_bind::extract(&input),
        };
        input = extracted.text.clone();
        Some(extracted)
    } else {
        None
    };

    let mut events = parse(
        &input,
        &Options {
            strict: args.strict,
            max_doc_bytes: args.max_doc_bytes,
            max_line_len: args.max_line_len,
            max_blank_run: args.max_blank_run,
        },
    );
    if let Some(bind) = bind.as_ref() {
        events.extend(bind.diagnostics.iter().cloned());
    }

    let markdown_opts = MarkdownOptions {
        allow_html: args.markdown_allow_html,
        strip_comments: args.markdown_strip_comments,
    };
    let wiki_opts = WikiOptions {
        link_prefix: args.wiki_link_prefix.clone(),
        latex: None,
    };

    let mut config = match chain::ChainConfig::new(
        "cli",
        args.plugin.as_deref(),
        format,
        markdown_opts,
        wiki_opts,
    ) {
        Ok(config) => config,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };
    // `--wiki-link-prefix` and `--latex` reach the wiki plugin through its
    // options, exactly as they do for a task.
    config.wiki.latex = config.latex;

    // Text stages first: a leading `---` block only becomes a node here, and the
    // citation context is built from the references it holds.
    events = chain::prelude(events, &config);

    let frontmatter = extract_frontmatter_for_cite(&events);
    let references = frontmatter
        .as_ref()
        .and_then(|data| data.get("references"))
        .cloned()
        .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::new()));
    let references = merge_refs_for_context(&references, None);
    let document_context = DocumentContext::new(pendon_plugin_cite::CitationContext::new(
        references,
        CiteOptions::default(),
    ));

    let pipeline = config.pipeline();
    let mut state =
        chain::ChainState::new(document_context, std::collections::HashMap::new(), pipeline);
    events = chain::run(events, &config, &mut state);

    // `resolve` runs after every plugin has emitted its extras attributes
    // (ADR-0004) and before the citation export, so a citation inside a bound
    // `mdp` payload belongs to the page's citation list.
    let events = match bind.as_ref() {
        Some(bind) => {
            let mut render = |markdown: &str, mode: pendon_plugin_bind::MdpMode| {
                chain::render_mdp(markdown, mode, &config, &mut state)
            };
            pendon_plugin_bind::resolve_with(&events, &bind.registry, &mut render)
        }
        None => events,
    };

    let events = if state.cite_ran {
        let (cite_diagnostics, citations, references, options) = state.context.finalize_citations();
        state.hints.push(pendon_plugin_cite::solid_hints(
            &options,
            &citations,
            &references,
        ));
        let mut events = events;
        events.extend(cite_diagnostics);
        events
    } else {
        events
    };

    chain::finish(&config, &mut state);

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
            let s = if args.pretty {
                pendon_renderer_html::render_html_pretty(&events)
            } else {
                pendon_renderer_html::render_html(&events)
            };
            println!("{}", s);
            if has_error {
                ExitCode::from(2)
            } else {
                ExitCode::SUCCESS
            }
        }
        "solid" => {
            let hints = merge_solid_hints(&state.used_specs, &state.hints);
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
