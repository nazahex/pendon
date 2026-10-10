# Guides

Task-oriented how-tos. Each answers "how do I …" without restating the grammar
(that lives in [`../spec/SYNTAX.md`](../spec/SYNTAX.md)).

| Guide                                        | Use it to…                                             |
| -------------------------------------------- | ------------------------------------------------------ |
| [`cli.md`](cli.md)                           | Run the `pendon` binary, understand flags and formats  |
| [`plugin-authoring.md`](plugin-authoring.md) | Add or modify a plugin, wire config, pick a phase      |
| [`testing.md`](testing.md)                   | Write tests, add/re-freeze goldens, validate a sandbox |

Prefer the smallest relevant package test first, then widen to workspace and
sandbox validation (see `CONTRIBUTORS.md`).
