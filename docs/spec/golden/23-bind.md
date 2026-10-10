`plugin-bind` reads a `{{{lang[var] … }}}` data block **before** the lexer sees it
(ADR-0004): a payload's `#`, `-` and blank lines would otherwise be read as
markup. The block is removed from the text and every `$var` in an extras head
binds the parsed value as a real JavaScript prop (RFC §2).

{{{json[card]
{
  "id": "usr_01",
  "tags": [ "admin", "dev" ]
}
}}}

The whole object becomes one prop:

[Card](/docs)@@anchorA{data: $card}

A nested extras value binds a leaf; a block may be declared after its use:

[Rows](/docs)@@anchorA{meta: {rows: $rows, n: 2}}

{{{csv[rows]
id,nama,aktif
1,"Budi, S.T.",true
2,Siti,false
}}}

`jsonc` strips `//` comments, `yaml` expands merge keys, and `toml` keeps its
tables:

{{{jsonc[theme]
{
  // a JSONC line comment
  "accent": "#0af",
  "scale": [1, 2]
}
}}}

{{{yaml[env]
base: &base
  timeout: 30
dev:
  <<: *base
  host: "localhost"
}}}

{{{toml[build]
[profile]
release = true
targets = ["aarch64", "x86_64"]
}}}

[Typed](/docs)@@anchorA{theme: $theme, env: $env, build: $build}

An undefined `$ghost` warns and stays literal text:

[Ghost](/docs)@@anchorA{data: $ghost}
