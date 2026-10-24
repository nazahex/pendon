# ADR-0006 — One cache record per `(task, input)`

- Status: **Accepted / implemented**
- Related: [`../rfc/no-cache-and-post-hook.md`](../rfc/no-cache-and-post-hook.md)
  (the same run loop owns cache reads/writes), [`../guides/cli.md`](../guides/cli.md#cache)
- Evidence: `apps/cli/src/cache.rs` (`mod tests`),
  `apps/cli/tests/cli_spec.rs` (`run_task_two_tasks_over_one_input_both_stay_cached`,
  `run_task_a_deleted_output_is_rebuilt`, `run_task_no_cache_leaves_other_tasks_cached`,
  `run_task_the_no_cache_flag_records_nothing`)

## Context

The whole cache lived in one file, `.pendon/cache.toml`, and it was wrong in two
independent ways.

**1. It could not represent two tasks over one input.** `files` was a map keyed by
input path, and each entry carried a single `task` name. A second task rendering
the same input overwrote the first task's entry; the first task then looked its
entry up, followed `entry.task` to the _other_ task's descriptor, found a
different `config_hash`, and re-rendered. So every run re-rendered both tasks and
re-ran their `post_hook`s. [`sandbox/cite`](../../sandbox/cite/pendon.toml)
(Solid + HTML over `./src/[slug].mdp`) hit this on every run.

The indirection was the bug: `file → task name → task descriptor → config hash`.
A record that wants to say "this was rendered with config _h_" should simply
store _h_.

**2. It did not scale with the project.** Every run parsed the entire manifest and
re-serialised the entire manifest, so a 10 000-file project paid O(_all_ files) in
TOML parse and write in order to record O(_changed_) work. A single interrupted
write lost every record at once.

## Decision

**The cache is a directory of independent records, one per `(task, input)`.**

```
.pendon/cache/<first 2 hex>/<sha256(task + U+001F + input)>.toml
```

1. **The key is the task _and_ the input.** Two tasks over one input keep two
   records; nothing can overwrite anything else.
2. **A record is self-contained** — it carries everything needed to judge it, so
   there is no task table to consult:

   ```toml
   version = 1
   task = "Json"
   input = "./src/doc-12.md"
   output = "./out/doc-12.json"
   config_hash = "d897…"
   input_hash = "4596…"
   output_hash = "9175…"
   mtime = 1792647798

   [[deps]]
   path = "pendon.toml"
   hash = "681d…"
   ```

3. **Fresh** means: the record parses, its `version` matches, its `config_hash`
   matches the task's current hash, its `output` is the path the task would write
   _now_ and that file still exists, the input hash matches, and every dependency
   hash matches. The output checks are new — a record used to call a deleted
   output "up to date", so `rm -rf out/` produced nothing on the next run.
4. **Writes are atomic per record** (sibling temp file + `rename`). A crash
   damages at most one record, and a reader never sees a half-written one.
5. **A record that cannot be parsed, or that comes from another schema version,
   is deleted and treated as absent.** The cost is one re-render, never a wrong
   "up to date"; an upgrade never needs a manual `--clean-cache`.
6. **`.pendon` is pruned from the task walk** (`WalkDir::filter_entry`), so the
   cache can never be matched by an input glob or cost the walk thousands of
   stats.
7. **Legacy layout**: a pre-v1 `.pendon/cache.toml` is removed on first open and
   by `--clean-cache`; the project rebuilds cold exactly once.

## Consequences

- Recording costs O(files rendered); a warm run writes nothing at all. On a
  2 000-file project with two tasks (4 000 records), a warm run wrote **0**
  records and reported `2000 cached, 0 rebuilt` per task; after editing one file
  it rewrote exactly the 2 records for that file.
- Lookup is one small read per matched input (plus one per dependency), not a
  whole-project parse.
- A file may be cached for any number of tasks; `no_cache` removes exactly one
  task's records and leaves the others intact.
- Cost: inodes. 4 000 records ≈ 16 MB on disk at a 4 kB block size (record
  content ≈ 350 B). That is the price of incremental writes; `--clean-cache`
  reclaims it.
- Nothing prunes records for inputs or tasks that disappeared. They are inert
  (never read unless an input matches again) and `--clean-cache` removes them;
  task-aware pruning belongs to a future `pendon cache` command.

## Alternatives rejected

| Option                                                 | Why not                                                                                                                  |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ |
| Keep one manifest, key it `task::input`                | Fixes the collision but keeps O(all files) parse + write per run and the single-file blast radius.                       |
| One manifest as an append-only JSONL log               | Still O(n) to load, not greppable per file, and needs compaction before it can shrink.                                   |
| SQLite (`rusqlite`)                                    | A binary blob in a text-first repo, a new dependency and locking questions, for a hot path that is one `read_to_string`. |
| Sidecar file next to each output (`out/x.html.pendon`) | Pollutes the artifact directory (outputs are meant to be deployable as-is) and dies together with `out/`.                |
| A fingerprint directory per task (Cargo-style)         | Better than one file, but still one file per _task_ to rewrite — not per file.                                           |
| Key by input only, drop the task from the record       | Two tasks over one input would share a verdict they must not share: different plugins, formats and hashes.               |
| Keep the manifest, just add a `version` field          | Papers over a structure that cannot hold the data; the collision remains.                                                |

## Evidence

- `apps/cli/src/cache.rs` `mod tests`: round-trip, two tasks over one input,
  invalidation by config hash / input / dependency (including a missing
  dependency) / missing output, and a record from a foreign schema being dropped.
- `apps/cli/tests/cli_spec.rs`: `run_task_two_tasks_over_one_input_both_stay_cached`
  (the bug: two records, each `post_hook` runs exactly once),
  `run_task_a_deleted_output_is_rebuilt`, `run_task_no_cache_leaves_other_tasks_cached`,
  `run_task_the_no_cache_flag_records_nothing`.
