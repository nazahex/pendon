## 1. Definisi Resmi: “YAML Flat Subset”

Yang kita **izinkan**:

- `bool` → `true | false`
- `number` → `int | float`
- `string` → quoted `"..."` atau bareword
- `array` → `[a, b, c]`

Yang kita **bunuh tanpa ampun**:

- ❌ Indentation
- ❌ Nested object
- ❌ Multiline
- ❌ Anchor / alias
- ❌ Implicit truthy (`yes`, `on`, dll)
- ❌ Date auto-parse

📌 Ini bukan YAML 1.2
📌 Ini **YAML-inspired config**, jauh lebih aman.

---

## 2. Contoh Format Final (Clean & Predictable)

```text
---
title: "Frontmatter Tanpa Drama"
draft: false
views: 1234
rating: 4.75
tags: ["markdown", "engine", "perf"]
authors: ["kaz", "team"]
lang: id
---
```

**Catatan penting:**

- Satu baris = satu key
- Tidak ada konteks struktural
- Semua bisa di-scan line-by-line

---

## 3. Grammar Minimal (EBNF, kecil & tajam)

```ebnf
frontmatter  = "---" newline { entry } "---"
entry        = key ":" value newline
key          = /[a-zA-Z_][a-zA-Z0-9_-]*/
value        = boolean | number | string | array

boolean      = "true" | "false"
number       = int | float
int          = ["-"] digit { digit }
float        = ["-"] digit { digit } "." digit { digit }

string       = quoted | bare
quoted       = '"' { char | escape } '"'
bare         = /[^#\n\r]+/

array        = "[" [ value { "," value } ] "]"
```

🔥 Grammar sekecil ini:

- Cocok untuk hand-written parser
- Cocok untuk SIMD scanning
- Cocok untuk Zig tanpa heap

---

## 4. Parsing Strategy (Single Pass, Zero Drama)

### Step-by-step mesin parsing:

1. **Peek 4 byte pertama**

   - Kalau `---\n` → masuk mode frontmatter
   - Kalau tidak → skip total

2. **Scan per baris**

   - Cari `:`
   - Split `key | raw_value`

3. **Trim whitespace minimal**

   - Tidak perlu normalize banyak-banyak

4. **Dispatch value parser**

   - `[` → array
   - `"` → quoted string
   - digit / `-` → number
   - `true|false` → bool
   - selain itu → bare string

📌 Tidak perlu AST besar
📌 Tidak perlu backtracking
📌 Tidak perlu lookahead panjang

---

## 5. Array Rules (penting biar konsisten)

Aku **sangat menyarankan** aturan ini:

### ✅ Array HARUS satu tipe

```text
tags: ["a", "b", "c"]   ✔
scores: [1, 2, 3]       ✔
mixed: [1, "a"]         ✘ (reject)
```

**Kenapa?**

- Type inference simpel
- Data model stabil
- Query jauh lebih cepat

Kalau tuan mau longgar → buat flag parser, tapi default strict lebih sehat.

---

## 6. String: Bare vs Quoted (detail kecil yang krusial)

### Bare string:

```text
slug: hello-world
lang: id
```

Rules:

- Stop di `#` (comment)
- Trim trailing whitespace
- Tidak boleh mengandung `[` `]` `,`

### Quoted string:

```text
title: "Hello, World!"
```

Rules:

- Support escape minimal:

  - `\"`
  - `\\`
  - `\n` (opsional)

📌 Jangan support multiline — ini **frontmatter**, bukan novel.

---

## 7. Comment Handling (opsional tapi worth it)

```text
views: 1234  # analytics only
```

Rule:

- `#` hanya valid **di luar quoted string**
- Setelah parse value → ignore rest of line

Parsing tetap cepat, UX naik.

---

## 8. Error Handling (penting buat DX)

Tolong jangan “silent fail”.

Contoh error yang sehat:

```text
[frontmatter] invalid number at line 4: 12.3.4
[frontmatter] mixed array types at key 'tags'
[frontmatter] invalid boolean: TRUE (lowercase only)
```

📌 Strict ≠ kejam
📌 Strict = predictable

---

## 9. Data Model Ideal (engine side)

Contoh model netral bahasa:

```text
Value =
  Bool(bool)
| Int(i64)
| Float(f64)
| String(slice)
| Array(Vec<Value>)
```

Tapi karena array satu tipe:

```text
Array =
  Bool[]
| Int[]
| Float[]
| String[]
```

🚀 Jauh lebih cepat dibanding `Vec<Value>`.

---

## 10. Performa Micro-Optimizations (yang sering dilupakan)

Detail kecil tapi berdampak:

- Jangan lowercase key/value → compare literal
- Pre-check `true` & `false` via length
- Number parse tanpa `strtod` (manual)
- Array parse tanpa split → state machine

💡 Dengan ini:

- Parsing bisa < **50ns/entry**
- Zero allocation untuk string (slice)

---

## 11. Ringkasan Keputusan Desain (biar solid)

✔ YAML-style key/value
✔ Tanpa indent
✔ Subset tipe eksplisit
✔ Strict grammar
✔ Single-pass parsing
✔ Human-friendly
✔ Engine-friendly

> Ini **bukan YAML**, ini **format metadata yang waras** 😄
