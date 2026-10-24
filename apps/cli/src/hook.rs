use std::collections::HashMap;
use std::process::{Command, Output};

use crate::utils::substitute_output;

/// Runs a task's `post_hook` commands in declared order.
///
/// `commands` are raw config strings; every command is expanded with the same
/// `[slug]`-style substitution as `input` / `output` before the shell sees it.
/// The first command that exits non-zero aborts the rest and is reported as an
/// error, which marks the task failed in the caller.
///
/// The working directory is the process CWD, i.e. the project root where
/// `pendon.toml` was read from.
///
/// There is no dry-run / validation-only mode in the config runner today; when
/// one is added it must **not** call this function (hooks have side effects).
pub fn run_post_hooks(
    task_name: &str,
    commands: &[&str],
    vars: &HashMap<String, String>,
) -> Result<(), String> {
    for command in commands {
        let expanded = substitute_output(command, vars)
            .map_err(|e| format!("post_hook of task '{task_name}' has a bad placeholder: {e}"))?;

        let output = shell_command(&expanded)
            .output()
            .map_err(|e| format!("cannot run post_hook `{expanded}` of task '{task_name}': {e}"))?;

        forward(task_name, &output);

        if !output.status.success() {
            return Err(format!(
                "post_hook `{expanded}` of task '{task_name}' failed ({}); remaining hooks were aborted",
                describe_status(&output)
            ));
        }
    }

    Ok(())
}

/// Forwards the hook's output to Pendon's report stream (stderr), one prefixed
/// line at a time. Hook output is not suppressed by `--quiet`: `--quiet` hides
/// the `run` report, not the tooling a task deliberately runs.
fn forward(task_name: &str, output: &Output) {
    forward_stream(task_name, &output.stdout);
    forward_stream(task_name, &output.stderr);
}

fn forward_stream(task_name: &str, bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    for line in text.lines() {
        eprintln!("[post_hook:{}] {}", task_name, line);
    }
}

fn describe_status(output: &Output) -> String {
    match output.status.code() {
        Some(code) => format!("exited with status {}", code),
        None => "was terminated by a signal".to_string(),
    }
}

/// Hooks run through the system shell so users can pipe, redirect and chain
/// commands. Cross-platform portability of a hook is the author's concern
/// (`sh -c` on Unix, `cmd /C` on Windows).
#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("cmd");
    cmd.arg("/C").arg(command);
    cmd
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);
    cmd
}
