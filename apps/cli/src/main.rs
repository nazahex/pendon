use std::fs;
use std::io::{self, Read};
use std::process::ExitCode;

use pendon_core::{parse, Options};
use pendon_plugin_custom::{load_index_from_path, load_spec_from_path, PluginSpec};
use pendon_renderer_json::render_to_string;
use pendon_renderer_solid::{
    render_solid_with_hints, ComponentTemplate, ImportEntry, SolidRenderHints,
};
use pico_args::Arguments;
use regex::Regex;
use serde::Deserialize;
use serde_json;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use walkdir::WalkDir;

#[derive(Debug, Default)]
struct CliArgs {
    input: Option<String>,
    format: Option<String>,
    strict: bool,
    pretty: bool,
    tui: bool,
    max_doc_bytes: Option<usize>,
    max_line_len: Option<usize>,
    max_blank_run: Option<usize>,
    plugin: Option<String>,
}

fn parse_args() -> Result<CliArgs, String> {
    let mut pargs = Arguments::from_env();

    let input: Option<String> = pargs
        .opt_value_from_str(["-i", "--input"])
        .map_err(|e| e.to_string())?;
    let format: Option<String> = pargs
        .opt_value_from_str(["-f", "--format"])
        .map_err(|e| e.to_string())?;
    let strict: bool = pargs.contains("--strict");
    let pretty: bool = pargs.contains("--pretty");
    let tui: bool = pargs.contains("--tui");
    let max_doc_bytes: Option<usize> = pargs
        .opt_value_from_str("--max-doc-bytes")
        .map_err(|e| e.to_string())?;
    let max_line_len: Option<usize> = pargs
        .opt_value_from_str("--max-line-len")
        .map_err(|e| e.to_string())?;
    let max_blank_run: Option<usize> = pargs
        .opt_value_from_str("--max-blank-run")
        .map_err(|e| e.to_string())?;
    let plugin: Option<String> = pargs
        .opt_value_from_str("--plugin")
        .map_err(|e| e.to_string())?;

    // Ensure no unexpected free arguments
    let rest = pargs.finish();
    if !rest.is_empty() {
        return Err(format!("Unexpected arguments: {:?}", rest));
    }

    Ok(CliArgs {
        input,
        format,
        strict,
        pretty,
        tui,
        max_doc_bytes,
        max_line_len,
        max_blank_run,
        plugin,
    })
}

fn read_input(args: &CliArgs) -> Result<String, String> {
    if let Some(path) = &args.input {
        match fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("cannot read file '{}': {}", path, e)),
        }
    } else {
        let mut buf = String::new();
        let mut stdin = io::stdin();
        // Read all of stdin; callers should pipe data when not using --input
        if let Err(e) = stdin.read_to_string(&mut buf) {
            return Err(format!("failed to read stdin: {}", e));
        }
        Ok(buf)
    }
}

fn main() -> ExitCode {
    // Subcommand: "run" (config-driven batch); internal tools are separated
    if std::env::args().nth(1).as_deref() == Some("run") {
        return run_from_config();
    }

    let args = match parse_args() {
        Ok(a) => a,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };

    // Default format is json; other formats can be added later.
    let format = args.format.as_deref().unwrap_or("json");

    // Optional TUI: spinner around input reading when interactive
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

    // Parse to events (MVP: full text as a single Text event)
    let events = parse(
        &input,
        &Options {
            strict: args.strict,
            max_doc_bytes: args.max_doc_bytes,
            max_line_len: args.max_line_len,
            max_blank_run: args.max_blank_run,
        },
    );

    let mut used_custom_specs: Vec<PluginSpec> = Vec::new();
    let mut custom_cache: HashMap<String, PluginSpec> = HashMap::new();

    // Optional plugin processing (supports comma-separated list and toml:foo.toml entries)
    let events = if let Some(pstr) = args.plugin.as_deref() {
        let mut ev = events;
        for name in pstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty()) {
            if let Some(path) = name.strip_prefix("toml:") {
                let spec = match custom_cache.get(path) {
                    Some(existing) => existing.clone(),
                    None => match load_spec_from_path(path) {
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
                track_used_spec(&mut used_custom_specs, spec.clone());
                ev = pendon_plugin_custom::process(&ev, &spec);
                continue;
            }

            ev = match name {
                "micomatter" => pendon_plugin_micomatter::process(&ev),
                "markdown" => pendon_plugin_markdown::process(&ev),
                "sectionize" => pendon_plugin_sectionize::process(&ev),
                "extract-heading" => pendon_plugin_extract_heading::process(&ev),
                "codeblock-syntect" => pendon_plugin_codeblock_syntect::process(&ev),
                other => {
                    if let Some(spec) = custom_cache.get(other) {
                        track_used_spec(&mut used_custom_specs, spec.clone());
                        pendon_plugin_custom::process(&ev, spec)
                    } else {
                        ev
                    }
                }
            };
        }
        ev
    } else {
        events
    };

    // Determine if any Error diagnostics are present
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
            let hints = if used_custom_specs.is_empty() {
                None
            } else {
                Some(build_solid_hints(&used_custom_specs))
            };
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

#[derive(Debug, Deserialize)]
struct ConfigTask {
    name: Option<String>,
    input: String,
    output: String,
    plugin: Option<String>,
    format: String,
    pretty: Option<bool>,
    strict: Option<bool>,
    max_doc_bytes: Option<usize>,
    max_line_len: Option<usize>,
    max_blank_run: Option<usize>,
}

#[derive(Debug, Deserialize, Default)]
#[allow(dead_code)]
struct PluginCustomSection {
    source: Option<Vec<String>>,
    order: Option<Vec<String>>,
    enable_unsafe_hooks: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct PendonConfig {
    #[serde(rename = "task")]
    tasks: Vec<ConfigTask>,
    #[serde(rename = "plugin-custom")]
    plugin_custom: Option<PluginCustomSection>,
}

fn run_from_config() -> ExitCode {
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

    let custom_registry = match load_custom_registry(cfg.plugin_custom.as_ref()) {
        Ok(map) => map,
        Err(msg) => {
            eprintln!("Error: {}", msg);
            return ExitCode::from(2);
        }
    };
    let mut custom_cache: HashMap<String, PluginSpec> = HashMap::new();

    let theme = pendon_tui::Theme::default();
    if pendon_tui::is_interactive_stderr() {
        pendon_tui::render_status_line("Scanning source files...", theme);
    }

    let mut exit = ExitCode::SUCCESS;
    for task in cfg.tasks.iter() {
        let re = match input_pattern_to_regex(&task.input) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error: invalid input pattern '{}': {}", task.input, e);
                exit = ExitCode::from(2);
                continue;
            }
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
                match fs::read_to_string(entry.path()) {
                    Ok(input_text) => {
                        let opts = Options {
                            strict: task.strict.unwrap_or(false),
                            max_doc_bytes: task.max_doc_bytes,
                            max_line_len: task.max_line_len,
                            max_blank_run: task.max_blank_run,
                        };
                        let mut events = parse(&input_text, &opts);
                        let mut used_custom_specs: Vec<PluginSpec> = Vec::new();
                        if let Some(pstr) = task.plugin.as_deref() {
                            for name in pstr.split(',').map(|s| s.trim()).filter(|s| !s.is_empty())
                            {
                                if let Some(path) = name.strip_prefix("toml:") {
                                    let spec = match custom_cache.get(path) {
                                        Some(existing) => existing.clone(),
                                        None => match load_spec_from_path(path) {
                                            Ok(s) => {
                                                custom_cache.insert(path.to_string(), s.clone());
                                                s
                                            }
                                            Err(msg) => {
                                                eprintln!("Error: {}", msg);
                                                exit = ExitCode::from(2);
                                                break;
                                            }
                                        },
                                    };
                                    track_used_spec(&mut used_custom_specs, spec.clone());
                                    events = pendon_plugin_custom::process(&events, &spec);
                                    continue;
                                }

                                match name {
                                    "micomatter" => {
                                        events = pendon_plugin_micomatter::process(&events);
                                    }
                                    "markdown" => {
                                        events = pendon_plugin_markdown::process(&events);
                                    }
                                    "sectionize" => {
                                        events = pendon_plugin_sectionize::process(&events);
                                    }
                                    "extract-heading" => {
                                        events = pendon_plugin_extract_heading::process(&events);
                                    }
                                    "codeblock-syntect" => {
                                        events = pendon_plugin_codeblock_syntect::process(&events);
                                    }
                                    other => {
                                        if let Some(spec) = custom_registry.get(other) {
                                            let spec = spec.clone();
                                            track_used_spec(&mut used_custom_specs, spec.clone());
                                            events = pendon_plugin_custom::process(&events, &spec);
                                        }
                                    }
                                }
                            }
                        }
                        if exit != ExitCode::SUCCESS {
                            continue;
                        }
                        // CSS options are handled in renderer selection below
                        let pretty = task.pretty.unwrap_or(false);
                        let rendered = match task.format.as_str() {
                            "json" => pendon_renderer_json::render_to_string(&events)
                                .map(|s| maybe_pretty(&s, pretty)),
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
                                let hints = if used_custom_specs.is_empty() {
                                    None
                                } else {
                                    Some(build_solid_hints(&used_custom_specs))
                                };
                                Ok(match hints.as_ref() {
                                    Some(h) => render_solid_with_hints(&events, Some(h)),
                                    None => pendon_renderer_solid::render_solid(&events),
                                })
                            }
                            other => {
                                eprintln!("Error: unsupported format in task: {}", other);
                                exit = ExitCode::from(2);
                                continue;
                            }
                        };
                        match rendered {
                            Ok(out_str) => {
                                if let Some(parent) = Path::new(&out_path).parent() {
                                    let _ = fs::create_dir_all(parent);
                                }
                                if let Err(e) = fs::write(&out_path, out_str.clone()) {
                                    eprintln!("Error: cannot write output '{}': {}", out_path, e);
                                    exit = ExitCode::from(2);
                                } else {
                                    total_bytes += out_str.len();
                                }
                            }
                            Err(e) => {
                                eprintln!("Error: render failed: {}", e);
                                exit = ExitCode::from(2);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Error: cannot read input file '{}': {}", path_str, e);
                        exit = ExitCode::from(2);
                    }
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
            if let Some(name) = task.name.as_deref() {
                let items = [
                    ("name", name),
                    ("format", fmt_text),
                    ("size", size_text.as_str()),
                    ("total", total_text.as_str()),
                ];
                pendon_tui::render_kv_list("› Wrote:", &items, theme);
            } else {
                let items = [
                    ("input", task.input.as_str()),
                    ("output", task.output.as_str()),
                    ("format", fmt_text),
                    ("size", size_text.as_str()),
                    ("total", total_text.as_str()),
                ];
                pendon_tui::render_kv_list("› Wrote:", &items, theme);
            }
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

fn input_pattern_to_regex(pattern: &str) -> Result<Regex, String> {
    // Convert ./content/[id]/[lang]/[...slug].md to regex capturing groups
    let mut re = String::from("^");
    let mut i = 0;
    let bytes: Vec<char> = pattern.chars().collect();
    while i < bytes.len() {
        if bytes[i] == '[' {
            if let Some(end) = bytes[i + 1..].iter().position(|&c| c == ']') {
                let end_idx = i + 1 + end;
                let name: String = bytes[i + 1..end_idx].iter().collect();
                if name.starts_with("...") {
                    re.push_str("(.+)");
                } else {
                    re.push_str("([^/]+)");
                }
                i = end_idx + 1;
                continue;
            } else {
                return Err("unclosed bracket".to_string());
            }
        }
        let ch = bytes[i];
        match ch {
            '.' | '+' | '*' | '?' | '^' | '$' | '(' | ')' | '|' | '{' | '}' | '\\' => {
                re.push('\\');
                re.push(ch);
            }
            _ => re.push(ch),
        }
        i += 1;
    }
    re.push('$');
    Regex::new(&re).map_err(|e| e.to_string())
}

fn captures_to_map(
    pattern: &str,
    caps: &regex::Captures,
) -> Result<HashMap<String, String>, String> {
    let mut map = HashMap::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    let mut group = 1;
    while i < chars.len() {
        if chars[i] == '[' {
            let end = chars[i + 1..]
                .iter()
                .position(|&c| c == ']')
                .ok_or_else(|| "unclosed bracket".to_string())?
                + i
                + 1;
            let name: String = chars[i + 1..end].iter().collect();
            let val = caps
                .get(group)
                .ok_or_else(|| "missing capture".to_string())?
                .as_str()
                .to_string();
            map.insert(name, val);
            group += 1;
            i = end + 1;
        } else {
            i += 1;
        }
    }
    Ok(map)
}

fn substitute_output(pattern: &str, vars: &HashMap<String, String>) -> Result<String, String> {
    let mut out = String::new();
    let mut i = 0;
    let chars: Vec<char> = pattern.chars().collect();
    while i < chars.len() {
        if chars[i] == '[' {
            let end = chars[i + 1..]
                .iter()
                .position(|&c| c == ']')
                .ok_or_else(|| "unclosed bracket".to_string())?
                + i
                + 1;
            let name: String = chars[i + 1..end].iter().collect();
            let key = name.trim_start_matches("...").to_string();
            let val = vars
                .get(&name)
                .or_else(|| vars.get(&key))
                .ok_or_else(|| format!("missing var {}", name))?;
            out.push_str(val);
            i = end + 1;
        } else {
            out.push(chars[i]);
            i += 1;
        }
    }
    Ok(out)
}

fn track_used_spec(list: &mut Vec<PluginSpec>, spec: PluginSpec) {
    if !list.iter().any(|s| s.name == spec.name) {
        list.push(spec);
    }
}

fn build_solid_hints(specs: &[PluginSpec]) -> SolidRenderHints {
    let mut hints = SolidRenderHints::default();
    let mut seen_templates: HashSet<(String, Option<String>)> = HashSet::new();

    for spec in specs {
        if let Some(renderer) = spec.renderer.as_ref().and_then(|r| r.solid.as_ref()) {
            for val in &renderer.imports {
                match val {
                    toml::Value::String(s) => hints.imports.push(ImportEntry::Raw(s.clone())),
                    toml::Value::Table(tbl) => {
                        if let Some(module) = tbl.get("module").and_then(|v| v.as_str()) {
                            let default = tbl
                                .get("default")
                                .and_then(|v| v.as_str())
                                .map(|s| s.to_string());
                            let names = tbl
                                .get("names")
                                .and_then(|v| v.as_array())
                                .map(|arr| {
                                    arr.iter()
                                        .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                        .collect::<Vec<String>>()
                                })
                                .unwrap_or_default();
                            hints.imports.push(ImportEntry::Structured {
                                module: module.to_string(),
                                default,
                                names,
                            });
                        }
                    }
                    _ => {}
                }
            }

            if let Some(tpl) = &renderer.component_template {
                let node_type = spec
                    .ast
                    .as_ref()
                    .and_then(|a| a.node.clone())
                    .unwrap_or_else(|| spec.name.clone());
                let node_name = spec.ast.as_ref().and_then(|a| a.node_name.clone());
                let key = (node_type.clone(), node_name.clone());
                if seen_templates.insert(key) {
                    hints.templates.push(ComponentTemplate {
                        node_type,
                        node_name,
                        template: tpl.clone(),
                    });
                }
            }
        }
    }

    hints
}

fn load_custom_registry(
    cfg: Option<&PluginCustomSection>,
) -> Result<HashMap<String, PluginSpec>, String> {
    let mut map: HashMap<String, PluginSpec> = HashMap::new();
    let Some(cfg) = cfg else {
        return Ok(map);
    };
    if let Some(sources) = &cfg.source {
        for src in sources {
            let plugins = load_index_from_path(src)?;
            for plugin in plugins {
                map.insert(plugin.id, plugin.spec);
            }
        }
    }
    Ok(map)
}

fn maybe_pretty(s: &str, pretty: bool) -> String {
    if !pretty {
        return s.to_string();
    }
    match serde_json::from_str::<serde_json::Value>(s) {
        Ok(v) => serde_json::to_string_pretty(&v).unwrap_or_else(|_| s.to_string()),
        Err(_) => s.to_string(),
    }
}

// CSS injection handled directly in renderer-html functions
