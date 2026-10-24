# RFC: plugin-cite Rework — Semantic Renaming & Export Separation

## Summary

This RFC proposes a rework of `pendon-plugin-cite` to resolve ambiguous configuration names, fix a critical concept collision around citation indexing, and separate citation/reference data from frontmatter into dedicated JS exports. These changes improve type-safety, enable correct bibliography backlinks, and make the plugin's API self-documenting.

**Status:** Proposed
**Breaking Changes:** Yes (Total Replacement)
**Release Stage:** Alpha

> ⚠️ **Alpha Stage Notice**
> This plugin is currently in **alpha**. All changes described in this RFC will be implemented as a **clean, total replacement**. There will be:
>
> - **No backward compatibility** with previous config or output formats
> - **No deprecation warnings** or transitional periods
> - **No changelog migration notes** for old field names
> - **No legacy code paths** retained in the codebase
>
> Users upgrading must adopt the new API in full. Old configurations will fail fast with validation errors rather than silently falling back to deprecated behavior.

---

## 1. Motivation

### 1.1 Ambiguous Configuration Names

The current config fields `prefix`, `class`, and `id_prefix` are too generic. They do not distinguish inline citation elements from bibliography targets, leading to confusion when configuring the plugin alongside other plugins that may use similar field names.

### 1.2 Index Concept Collision

A single `index` field currently serves two conflicting purposes:

- **Display ordinal** (`[1], [2]`) — deduplicated by `(ref_id + loc)`, values may repeat.
- **Unique anchor target** — must be unique per physical occurrence for bibliography backlinks.

When both share the same value, the bibliography can only link back to the _first_ occurrence of a deduplicated citation. This is a functional bug.

### 1.3 Frontmatter Data Coupling

Citation and reference data embedded in frontmatter loses type information, requires YAML re-parsing at runtime, and pollutes document metadata. Solid/JSX output should use typed `export const` bindings instead.

---

## 2. Proposed Changes

### 2.1 Config Field Renames (`[task.cite]`)

| Old Name    | New Name             | Description                                                       |
| :---------- | :------------------- | :---------------------------------------------------------------- |
| `prefix`    | `ref_id_prefix`      | Prefix for `href` linking to bibliography entries                 |
| `class`     | `citation_class`     | CSS class for the inline `<sup>` citation element                 |
| `id_prefix` | `citation_id_prefix` | Prefix for inline anchor IDs (backlink targets from bibliography) |

### 2.2 Citation Node Prop Splits

| Old Name | New Name          | Type   | Description                                                                                                      |
| :------- | :---------------- | :----- | :--------------------------------------------------------------------------------------------------------------- |
| `index`  | `citation_number` | number | Display ordinal `[1], [2]`. Deduplicated by `(ref_id + loc)`. Values may repeat.                                 |
| _(new)_  | `instance_index`  | number | Absolute counter per physical citation occurrence. Never resets or deduplicates. Used as unique backlink target. |

> ⚠️ `citation_number` and `instance_index` are independent. Example: same ref + same loc appearing in paragraph 1 and paragraph 5 → both have `citation_number: 1` but `instance_index: 1` and `instance_index: 3`.

#### Deduplication Semantics

`citation_number` is deduplicated by the **tuple `(ref_id, loc)`**, not by `ref_id` alone. This follows established academic citation standards (CSL, IEEE, Vancouver, Chicago):

- Same `ref_id` + same `loc` → same `citation_number` (valid deduplication)
- Same `ref_id` + different `loc` → **different** `citation_number`
- Same `ref_id` + `loc` present vs. absent → **different** `citation_number`

Implementations MUST NOT deduplicate based solely on `ref_id`. Doing so produces academically incorrect output where distinct locators collapse into a single citation number, breaking reference precision.

### 2.3 Data Export Separation

**Default behavior** (no config overrides needed):

```js
export const references = { /* ... */ };
export const citations = [ /* ... */ ];
```

**Optional override fields** in `[task.cite]`:

| Field               | Default        | Description                                             |
| :------------------ | :------------- | :------------------------------------------------------ |
| `references_export` | `"references"` | Override the exported variable name for references data |
| `citations_export`  | `"citations"`  | Override the exported variable name for citations data  |

Example with overrides:

```toml
[task.cite]
references_export = "bibliographyData" # export const bibliographyData = {...};
citations_export = "citationList" # export const citationList = [...];
```

**Constraints:**

- Values MUST be valid JavaScript identifiers. Reject with diagnostic error if invalid.
- Each field is independently optional; omitting one keeps its default.
- For `format = "html"`, data embedding strategy remains unchanged (e.g., `<script type="application/json">`). This change primarily targets `format = "solid"` and other JS/TS outputs.

### 2.4 Frontmatter Cleanup

After this change, `cites` and `references` arrays are **removed entirely** from frontmatter output. Frontmatter returns to containing only original document metadata (`title`, `date`, etc.).

### 2.5 Custom Component Template Update

Templates must be updated to use the new prop names:

```toml
# BEFORE
template = "<Citation index={attrs.index} id={attrs.id} ... />"

# AFTER
template = "<Citation reference={references[\"{attrs.ref_id}\"]} {...attrs} />"
```

`{...attrs}` is the portable form: raw attribute values are serialised as JSON
through the spread, so `citation_number` and `instance_index` reach the
component typed (`citation_number={1}`). A bare `{attrs.citation_number}`
interpolates as text (`number=1`), which is invalid JSX — templates are string
substitution, not an expression evaluator, so `id={'cite-' + attrs.instance_index}`
is emitted verbatim and must be written as a quoted slot
(`id="cite-{attrs.instance_index}"`) instead.

The `reference` binding is the one place §2.3 export renaming reaches a
template: it must name the configured `references_export`. The built-in
fallback template therefore follows that key rather than hard-coding
`references`.

### 2.6 Reference Export Scope

Before this change the export was always cited-only. A document that declares
its references in frontmatter, however, has authored them for that document — an
entry can be listed and cited nowhere and still belong in the output (a
"further reading" list). An external catalog is the opposite case: it is often a
large shared file, so shipping every uncited entry bloats each output.

**New optional field** in `[task.cite]`:

| Field              | Type                 | Default          | Description                                      |
| :----------------- | :------------------- | :--------------- | :----------------------------------------------- |
| `references_scope` | `"cited"` \| `"all"` | source-dependent | Which references the `references` export carries |

The default follows `reference_source` when the field is omitted:

| `reference_source`         | Default   |
| :------------------------- | :-------- |
| `frontmatter` / `internal` | `"all"`   |
| `external`                 | `"cited"` |

An explicit value always wins, whichever source is configured:

```toml
[task.cite]
# Force all external references to be exported
references_scope = "all"

# Or force only cited frontmatter references
references_scope = "cited"
```

**Constraints:**

- Only `"cited"` and `"all"` are accepted. Reject any other value with a
  diagnostic error listing the valid options.
- The setting affects **only** the `references` export. It has no effect on the
  `citations` export, on `citation_number` / `instance_index` assignment, or on
  diagnostics.

---

## 3. Output Examples

### Citations Data Structure

```json
[
  { "ref_id": "suryana-2016", "citation_number": 1, "instance_index": 1 },
  { "ref_id": "suryana-2016", "citation_number": 2, "loc": "page 27-35", "instance_index": 2 },
  { "ref_id": "suryana-2016", "citation_number": 1, "instance_index": 3 }
]
```

### Generated Solid Output (default export names)

```jsx
export const references = {
  "suryana-2016": { "title": "...", "authors": [...] }
};

export const citations = [
  { "ref_id": "suryana-2016", "citation_number": 1, "instance_index": 1 },
  { "ref_id": "suryana-2016", "citation_number": 2, "loc": "page 27-35", "instance_index": 2 },
  { "ref_id": "suryana-2016", "citation_number": 1, "instance_index": 3 }
];

export default function PendonView() { return (<> {/* ... */} </>); }
```

---

## 4. Implementation Checklist

- [x] Rename config fields in TOML parser & validation schema (§2.1)
- [x] Add `references_export` / `citations_export` config fields with JS identifier validation (§2.3)
- [x] Update `CitationContext` internals: separate display counter from instance counter (§2.2)
- [x] Remove `cites` and `references` from frontmatter serialization (§2.4)
- [x] Add `export const` generation for Solid/JS output formats (§2.3)
- [x] Update default HTML renderer to use new prop names (§2.2)
- [x] Update custom component template attribute mapping (§2.5)
- [x] Add `references_scope` (`cited` / `all`) with a source-dependent default (§2.6)
- [x] Update plugin README / documentation with all new names
- [x] Replace golden fixtures entirely with new output format (no diff against old format)
- [x] Delete all legacy config parsing code and deprecated prop mappings — the old
      keys (`prefix`, `class`, `id_prefix`, `references`, `citations`) are still
      _recognised_ only to fail loudly with a rename hint, never to drive behaviour
      (see `apps/cli/src/plugins.rs`, `check_legacy_cite_keys`).

---

## 5. Open Questions

None at this time. All naming decisions have been finalized through prior discussion.
