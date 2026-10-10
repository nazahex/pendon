`plugin-bind` reaches **one leaf** of a bound value with a path
(`$config.db.host`, `$rows[0].nama`, `$config.limits["max-rows"]`), merges a bound
object into a nested extras value with a **spread** item, and reads a payload from
an **external file** — RFC §2.5–§2.7.

This fixture lives at `src/24-bind-paths.md` and its file payload at
`../data/rows.csv`, so resolving a relative path against the **source file's** own
directory (never the process CWD) is part of the contract, as is recording that
file as a cache dependency.

{{{json[config]
{
  "db": { "host": "localhost", "port": 5432 },
  "limits": { "max-rows": 25 }
}
}}}

{{{json[brand]
{
  "accent": "#0af",
  "mode": "dark",
  "scale": 1.0
}
}}}

A file payload binds exactly like a body payload:

{{{csv[rows](../data/rows.csv)}}}

A path reaches one leaf. A miss stays literal with a warning, and a price is never
a reference at all — a `var` starts with a letter:

[Card](/docs)@@anchorA{db: $config.db.host, port: $config.db.port, first: $rows[0].nama, max: $config.limits["max-rows"], miss: $rows[9].nama, price: $100}

A spread supplies defaults, and the key written in the head wins over it:

[Theme](/docs)@@anchorA{theme: {...$brand, scale: 1.2}}

A spread item is **copied, never walked**: a `$…`-looking string inside the bound
object stays exactly as it is.

{{{json[copy]
{
  "title": "a $brand string is data",
  "n": 1
}
}}}

[Copy](/docs)@@anchorA{copy: {...$copy, n: 2}}
