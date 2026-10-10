# pendon-renderer-json

Renders the event stream to a concatenated **text IR** in JSON — a quick,
lightweight preview of the document's text content (`--format json`, the default).

## API

```rust
use pendon_renderer_json::render_to_string;

let json = render_to_string(&events)?; // Result<String, serde_json::Error>
```

## What it is for

A compact preview: it concatenates the text of the stream so you can eyeball what
a pipeline produced without the full AST. For structure, use
[`pendon-renderer-ast`](../renderer-ast); for the raw event stream, use
[`pendon-renderer-events`](../renderer-events).

Container attributes are **not** dropped: a bare flag reaches JSON as `true` and
list/blockquote/paragraph/code-fence attributes survive (verified by the §16
fixture 17/18 test in this crate).

## Usage

```bash
pendon --format json < ./doc.md
```

## License

MIT
