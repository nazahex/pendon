# Clean-up cite (`stable_fields`, no built-in `locator`)

**Status: landed (2026-10-31).** `plugin-cite` presets only `bracket.bare →
reference_id`; the bracket's quote is an ordinary field, so the plugin has no
`locator`. `[task.cite] stable_fields` names the public field paths (before
`field.override`) whose values make `citation_number` stable — default
`["reference_id"]`, `[]` = one number per occurrence, a field a citation does not
carry counted as absent. A `citations` row is `{reference_id, citation_number}`.
Spec: [`spec/SYNTAX.md`](../spec/SYNTAX.md) §7.3/§20,
[`spec/FIELDS-INVENTORY.md`](../spec/FIELDS-INVENTORY.md) §4/§12; golden
`48-cite-stable-fields`. No warning, no migration, no compatibility key (D-16).

- Currently, `plugin-cite` `citation_number` is stabilized via `reference_id` and `locator` fields. The problem is that the form of `locator` can vary for each user. Or users might want the `reference_id` and `locator` pair not to make `citation_number` stable. Or users might want various additional field combinations so that `citation_number` is stable.
- This change adds an optional setting for `task.cite` in the form of `stable_fields` whose value is an array of strings of the fields intended to make `citation_number` stable.

Example:

```toml
[task.cite]
stable_fields = ["reference_id", "locator_type", "locator_value", "author"]

[[task.cite.custom]]
type = ["citeX"]
name = "Cite"
imports = "import { Cite } from '@comp/Cite'"
template = "<Cite reference={references[\"{attrs.ref_id}\"]} {...attrs} />"

field.bracket.bare = "reference_id"
field.paren.bare = "locator_type"
field.paren.quote = "locator_value"
field.specs.quote = "author"
field.override.reference_id = "ref_id_overriden"

[[task.cite.custom]]
type = ["citeY"]

field.specs.tick = "author"
field.paren.quote = "reference_id"
```

- Thus, the built-in `locator` field in `plugin-cite` is now completely discarded. Its implementation is left entirely to the user.

This change is made fully without nostalgia: without warning, migration thingy, backward support.
