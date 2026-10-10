# Pendon plugin-bind

Mengubah data dari JSON, JSONC, YAML, TOML, Markdown Pendon, dan CSV menjadi json satu baris yang valid sebagai nilai properti JSX Solid.

## Definisi

- Data: data JSON, JSONC, YAML, TOML, Markdown Pendon (MDP), dan CSV yang ditulis di dalam `{{{ }}}`

## Mekanisme

### Mendefinisikan Data

Data ditulis dalam blok `{{{lang[var-name] ... }}}`.

- `lang` wajib diisi dengan salah satu ini: `json`, `jsonc`, `yaml`, `yml`, `toml`, `csv`, dan `mdp`.
- `var-name` wajib diisi. Karakter yang diperbolehkan: alpha-numerik upper or lower, `-` `_`
- Jika `lang` dan `var-name` tidak diisi atau tidak valid, semua blok data ini akan dihiraukan dan tidak dirender (pertimbangan: membiarkannya dirender akan beresiko merusak markdown keseluruhan yang mengakibatkan bug pada parser atau renderer), kemudian diagnostic warning dikeluarkan.
- Data wajib ditulis di baris baru. Menuliskannya di baris yang sama di dalam `{{{ }}}` maka data dianggap invalid, semua blok data ini akan dihiraukan dan tidak dirender, kemudian diagnostic warning dikeluarkan.
- Jika data kosong, beri ia nilai `null`, tidak ada warning.

## Mendefinisikan Data MDP

- Khusus untuk MDP:
  - Jika markdown ditulis dalam baris yang sama dengan `{{{ }}}`, maka ia dianggap dan dirender sebagai elemen inline (tidak dibungkus `<p>` maupun elemen block lain). Contoh:
  ```mdp
  {{{mdp[jsx-inline] Foo bar [lorem](https://lorem "Lorem")@@anchorL{ bax: 13 } dolor ::tagX sit:: amet. }}}
  ```
  - Jika ditulis dalam baris baru, maka akan dianggap, diperlakukan, dan dirender sebagai elemen block.
  - Sintaksis bind `{{{ }}}` di dalam bind `{{{ }}}` dianggap sebagai teks literal.

### Mengkonsumsi Data

- Hanya ada satu cara untuk mengkonsumsi data ini, yaitu mengirimkannya lewat `k: v` pada extras `{...}`, dengan memberikan `$` pada awal `var-name`.

### Render

Data akan dirender sebagai javascript object one-line yang dirimkan ke props terkait.

Contoh:

```mdp
```

```jsx
<Element >
```

## Contoh Singkat

Contoh lebih lengkap dan kompleks ada di `./sandbox/bind/`

```md
# Bind Sandbox

@@{ dataT: $f21 }
Amet magna do {minim} est $eu$ pariatur.

## JSON

{{{json[data-X]
{
"str_escapes": "Line\n\t\"quote\" \u2728",
"numbers": [ 0, -42, 3.14, 1e5, 9007199254740991 ],
"primitives": { "b": true, "n": null, "arr": [] },
"deep": [ [ [ { "k": [ 100, "txt", null ] } ] ] ],
"keys": { "": "empty", "123": "num", "a.b": "dot" },
"users": [ { "id": 1, "tags": [ "admin" ] } ]
}
}}}

@@{ dataX: $data-X }
Eiusmod qui sunt labore nisi.

## YAML

{{{yaml[foo_yi]
foo: bar

# 1. ANCHORS & MERGE

base: &base_env
timeout: 30
dev:
<<: *base_env
host: "localhost"

# 2. BLOCK SCALARS

text:
folded: >
Baris 1
Baris 2
literal: |-
Line 1
Line 2

# 3. ADVANCED TYPES

types:
str_int: !!str 123
bin: !!binary R3VydQ==
nums: [ 0x1A, 0b1010 ]
bools: [ true, "yes" ]
nested: { arr: [1, { a: "b" }] }
}}}

{{markerAx}}{yi-ro: $foo_yi}

Lorem {{markerDataX}}{data: $data-X, `marker-x`, --lenghth: "2rem", fooX: "roem"} ipsum.

## CSV

{{{csv[tabel21-karyawan4]
id,nama,meta,catatan,aktif
1,"Budi, S.T.",{"role":"admin"},"Baris 1",true
2,"Siti ""B""",{"tags":["qa"]},"Baris 1
Baris 2",false
}}}

{{markerAx}}{yi-ro: $foo_yi}

Lorem {{markerDataX}}{data: $data-X, `marker-x`, --lenghth: "2rem", fooX: "roem"} ipsum.

===directiveCSV("Tabel Karyawan"){ tabel: { karyawan: $tabel21-karyawan4, foo: { fooYi:$foo_yi } }, --color: "#ffa"}

Incididunt laborum culpa Lorem proident nulla consequat dolor adipisicing.

{{markerTOML}}{yi-ro: $f21}

===

## TOML

{{{toml[f21]

# 1. SCALARS & DATES

[primitives]
str_basic = "Hello\tWorld"
str_lit = 'C:\path'
numbers = [ 42, 0x1A, 3.14, inf, nan ]
bools = [ true, false ]
date = 1979-05-27T07:32:00Z

# 2. INLINE & KEYS

[inline]
table = { a = 1, b = "x" }
"quoted.key" = "val"
server.ip = "127.0.0.1"

# 3. ARRAY OF TABLES

[[users]]
id = 1
name = "Budi"
}}}

{{markerTOML2}}@@("Lom Umami"){"Lom", lom: $f21}
```
