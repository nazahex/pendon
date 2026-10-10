# pendon-renderer-events

Renders the raw **event stream** as JSON — the debug-friendly, lowest-level view
of what a pipeline produced (`--format events`). One JSON value per `Event`.

## API

```rust
use pendon_renderer_events::render_events_to_string;

let json = render_events_to_string(&events)?; // Result<String, serde_json::Error>
```

## Event shapes

Each line is a tagged object (`type` discriminates):

```json
{ "type": "Start", "node": "Paragraph" }
{ "type": "Attribute", "name": "class", "value": "lead" }
{ "type": "AttributeFlag", "name": "isFoo" }
{ "type": "Text", "text": "Hello" }
{ "type": "End", "node": "Paragraph" }
{ "type": "Diagnostic", "severity": "warning", "message": "..." }
```

Use this renderer when developing or debugging a plugin: it shows exactly what
the transform emitted, before any structural assembly.

## Usage

```bash
pendon --plugin markdown --format events --input ./doc.md
```

## License

MIT
