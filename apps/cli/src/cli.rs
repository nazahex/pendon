use std::fs;
use std::io::{self, Read};

#[derive(Debug, Default)]
pub struct CliArgs {
    pub input: Option<String>,
    pub format: Option<String>,
    pub strict: bool,
    pub pretty: bool,
    pub tui: bool,
    pub max_doc_bytes: Option<usize>,
    pub max_line_len: Option<usize>,
    pub max_blank_run: Option<usize>,
    pub plugin: Option<String>,
    pub markdown_allow_html: bool,
    pub wiki_link_prefix: Option<String>,
}

pub fn parse_args() -> Result<CliArgs, String> {
    use pico_args::Arguments;

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
    let markdown_allow_html: bool = pargs.contains("--markdown-allow-html");
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
    let wiki_link_prefix: Option<String> = pargs
        .opt_value_from_str("--wiki-link-prefix")
        .map_err(|e| e.to_string())?;

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
        markdown_allow_html,
        wiki_link_prefix,
    })
}

pub fn read_input(args: &CliArgs) -> Result<String, String> {
    if let Some(path) = &args.input {
        match fs::read_to_string(path) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("cannot read file '{}': {}", path, e)),
        }
    } else {
        let mut buf = String::new();
        let mut stdin = io::stdin();
        if let Err(e) = stdin.read_to_string(&mut buf) {
            return Err(format!("failed to read stdin: {}", e));
        }
        Ok(buf)
    }
}
