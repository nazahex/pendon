# Pendon

**High‑Performance Modular Markdown Engine**

Pendon adalah mesin parsing Markdown modular berperforma ekstrem yang dirancang sebagai _Markdown-as-DSL engine_ untuk kebutuhan frontend modern. Fokus utama Pendon adalah **DX (Developer Experience)**, **ukuran biner kecil**, dan **kemampuan kustomisasi sintaks tanpa batas**, terutama untuk kasus penggunaan UI/komponen frontend kompleks (SolidJS, React-like IR, dsb).

---

## 1. Visi & Filosofi

Pendon tidak diposisikan sebagai Markdown parser kompatibel CommonMark. Pendon adalah **mesin parsing terprogram**.

Prinsip desain utama:

- **Markdown sebagai DSL**, bukan dokumen statis
- **Event-driven parsing**, bukan AST monolitik
- **Plugin-first architecture**
- **Output-native**, bukan HTML-centric
- **Zero-cost abstraction** dan _streaming by default_

Pendon memungkinkan client mendefinisikan sendiri apa itu "Markdown" bagi domain mereka.

---

## 2. Target Use Case

Pendon cocok digunakan untuk:

- Markdown dengan banyak **custom block / inline syntax**
- Rendering ke **UI Component Tree** (SolidJS, custom renderer)
- Konversi ke **JSON / YAML / DSL internal**
- CMS headless
- Documentation engine dengan komponen interaktif

Tidak ditujukan untuk:

- Blog Markdown statis klasik
- Strict CommonMark compliance

---

## 3. Non‑Goals

- Tidak menjamin kompatibilitas penuh CommonMark
- Tidak berfokus pada HTML rendering
- Tidak menyediakan WYSIWYG editor

---

## 4. Arsitektur Tingkat Tinggi

```
UTF‑8 Input
   ↓
Lexer (byte‑level FSM)
   ↓
Parser Core (state machine)
   ↓
Event Stream (semantic)
   ↓
Plugins
 ├─ Syntax Plugins
 └─ Renderer Plugins
```

Pendon memisahkan **parsing**, **semantik**, dan **output** secara tegas.

---

## 5. Core Concepts

### 5.1 Event‑Driven Parsing

Parser Pendon tidak membangun AST secara default.

Ia menghasilkan _event stream_:

- `StartNode`
- `EndNode`
- `Text`
- `Component`
- `Attribute`

Renderer dapat:

- mengonsumsi event secara streaming
- atau membangun tree jika diperlukan

---

### 5.2 UTF‑8 Handling

- Validasi UTF‑8 dilakukan satu kali di awal
- Internal parsing menggunakan byte index
- `&str` hanya dibuat saat dibutuhkan

Menjamin keamanan tanpa mengorbankan performa.

---

## 6. Plugin Architecture

Pendon membedakan dua jenis plugin:

### 6.1 Syntax Plugin

Digunakan untuk mendefinisikan sintaks Markdown kustom.

Contoh:

- `:::alert`
- `[[UserCard]]`
- `==highlight==`

### 6.2 Renderer Plugin

Digunakan untuk mengubah event menjadi output tertentu:

- JSON
- YAML
- SolidJS Native IR
- Custom UI Tree

---

## 7. Plugin Authoring (Client Perspective)

Pendon menyediakan DSL deklaratif berbasis macro untuk authoring plugin.

### 7.1 Contoh Plugin Block

```rust
md_plugin! {
    name: "alert",

    block {
        start: ":::alert",
        end: ":::",

        attrs {
            type: String,
            title?: String,
        }

        render |ctx| {
            ctx.component("Alert")
               .prop("type", ctx.attr("type"))
               .children(ctx.children());
        }
    }
}
```

Client **tidak perlu** memahami lexer, token, atau state machine.

---

### 7.2 Inline Syntax

```rust
md_plugin! {
    name: "highlight",

    inline {
        marker: "==",

        render |ctx| {
            ctx.component("Highlight")
               .child(ctx.text());
        }
    }
}
```

---

## 8. Context API

Context (`ctx`) adalah satu‑satunya interface parsing yang dilihat client.

Disediakan API tingkat tinggi:

- `ctx.text()`
- `ctx.children()`
- `ctx.attr(name)`
- `ctx.component(name)`
- `ctx.emit_text()`

Context menjamin:

- keamanan
- isolasi plugin
- DX konsisten

---

## 9. Engine Usage Flow (DX Story)

### 9.1 Setup

```toml
[dependencies]
pendon = { version = "0.1", features = ["solid"] }
```

### 9.2 Register Plugin

```rust
let engine = Pendon::new()
    .plugin(AlertPlugin)
    .plugin(TabsPlugin);
```

### 9.3 Render

```rust
let output = engine
    .input(markdown)
    .render::<SolidRenderer>();
```

Renderer dapat diganti tanpa mengubah plugin sintaks.

---

## 10. Output Model

Pendon **tidak mengutamakan HTML**.

Contoh UI IR:

```json
{
  "type": "Alert",
  "props": { "type": "warning" },
  "children": ["Hello"]
}
```

Pendekatan ini memungkinkan:

- hydration langsung
- zero DOM parsing
- integrasi frontend maksimal

---

## 11. Konfigurasi

Pendon mendukung konfigurasi runtime dan build‑time.

### Runtime

```rust
Pendon::new()
  .config(|c| {
      c.strict = false;
      c.allow_unknown_blocks = true;
  })
```

### Build‑time

Menggunakan `cargo features`:

- `json`
- `yaml`
- `solid`

---

## 12. Error Handling

Pendon mengutamakan error yang ramah manusia.

Contoh:

```
Unclosed :::alert block
  ┌─ input.md:12:1
  │
12│ :::alert type=warning
  │ ^^^^^^^^^^^^^^^^^^^^
```

---

## 13. Performance Strategy

- Byte‑level lexer (FSM)
- Streaming parsing
- Minimal allocation
- `SmallVec`, arena allocator
- `panic = abort`
- LTO + strip

Pendon dirancang untuk:

- latency rendah
- memory footprint kecil
- predictable performance

---

## 14. Project Structure

```
pendon/
 ├─ core/
 │   ├─ lexer.rs
 │   ├─ parser.rs
 │   ├─ event.rs
 │   └─ context.rs
 ├─ plugin/
 ├─ renderer/
 ├─ tui/                # New User Interface Module
 │  ├── mod.rs          # Sub-module registration
 │  ├── assets.rs       # Icon, Glyph, and Character definitions (ASCII/Nerd)
 │  ├── locales.rs      # Collection of strings, error messages, and static text
 │  ├── templates.rs    # Rendering logic for output (e.g., table/list formats)
 │  ├── theme.rs        # Color definitions (Style) for UI consistency
 │  └── widgets/        # Specific UI components (progress bar, spinner)
 ├─ config.rs
 └─ lib.rs
```

---

## 15. Positioning

Pendon berada di persimpangan:

- Markdown engine
- DSL parser
- UI compiler frontend

Pendon bukan sekadar parser, tetapi **fondasi bahasa markup terprogram**.

---

## 16. Penutup

Pendon memberikan client **kontrol penuh atas bahasa dan output**, tanpa memaksa mereka memahami kompleksitas parsing.

Dengan pendekatan ini, Pendon siap menjadi:

> _"The engine behind next‑generation Markdown‑driven UI."_
