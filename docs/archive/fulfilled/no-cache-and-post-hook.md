### Per-Task Cache Disable

We already have a built-in CLI for bypassing the cache, but the `pendon.toml` configuration is still missing. Therefore, we need to create it.

Add a new optional boolean config field to completely bypass caching for a specific task.

| Field      | Type | Default | Description                                              |
| :--------- | :--- | :------ | :------------------------------------------------------- |
| `no_cache` | bool | `false` | When `true`, disables all caching for this task entirely |

When `no_cache = true`:

- The task always executes from scratch, regardless of input file timestamps or content hashes.
- No entries are read from or written to `.pendon/cache/` for this task.
- Other tasks in the same pipeline remain unaffected; caching is disabled **only** for the task where this field is set.

```toml
[[task]]
name = "Cite Sandbox (Solid)"
input = "./src/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "micromatter,cite,markdown"
format = "solid"
no_cache = true
```

---

### Per-Task Post-Hook Commands

Add a new optional config field to execute arbitrary shell commands after a task completes successfully. This enables autoformatting, linting, asset optimization, and integration with external tooling without wrapping Pendon in external scripts.

| Field       | Type               | Default  | Description                                              |
| :---------- | :----------------- | :------- | :------------------------------------------------------- |
| `post_hook` | string \| string[] | _(none)_ | Shell command(s) to run after successful task completion |

**Behavior:**

- Accepts a single string or an array of strings. Array entries execute **sequentially** in declared order.
- Each command runs with the working directory set to the project root (where `pendon.toml` resides).
- Commands only execute if the task completed **without errors**. Failed tasks skip post-hooks entirely.
- If any post-hook command exits with non-zero status, the task is marked as failed and subsequent hooks in the array are aborted.
- Standard output and standard error from hook commands are forwarded to Pendon's console output prefixed with `[post_hook:<task_name>]`.

```toml
[[task]]
name = "Cite Sandbox (Solid)"
input = "./src/[slug].mdp"
output = "./out/[slug].jsx"
plugin = "micromatter,cite,markdown"
format = "solid"
# Single command
post_hook = "prettier --write ./out/[slug].jsx"

[[task]]
name = "Cite Sandbox (HTML)"
input = "./src/[slug].mdp"
output = "./out/[slug].html"
plugin = "micromatter,cite,markdown"
format = "html"
# Multiple sequential commands
post_hook = [
  "prettier --write ./out/[slug].html",
  "html-minifier-terser -o ./out/[slug].min.html ./out/[slug].html",
]
```

**Constraints:**

- Hook commands are executed via the system shell (`sh -c` on Unix, `cmd /C` on Windows). Cross-platform compatibility is the user's responsibility.
- Output path placeholders (`[slug]`, etc.) in hook command strings are expanded using the same substitution rules as `input`/`output` fields.
- Hooks are NOT executed during dry-run or validation-only modes.
