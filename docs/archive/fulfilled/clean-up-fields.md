# Clean-up fields (specs neutrality, universal `id`, no hardcoded cite output)

**Status: landed (2026-10-31).** `specs.tick` / `specs.quote` carry no default
meaning, `class` / `id` / `style` are universal DOM fields (only `section`'s
derived `id` is in a preset), `plugin-heading` derives no id, `plugin-cite` has no
`cite-id` / `CITATION_CLASS` / `REF_HREF_PREFIX` / `CITATION_ANCHOR_PREFIX`.
Spec: [`spec/FIELDS-INVENTORY.md`](../spec/FIELDS-INVENTORY.md) §0/§12 and
[`spec/SYNTAX.md`](../spec/SYNTAX.md) §6.2/§6.3.

- `specs.quote` and `specs.tick` MUST NOT have a default meaning, the current implementation interprets them as `slug` and `title`, which is wrong. `specs.quote` and `specs.tick` MUST be treated the same as `<slot>.<quote|tick>`. The meaning is left entirely to the user via preset assignments which are now implemented and working properly.
- Thus, `id` in all components as registered in FIELDS-INVENTORY.md must not originate from `specs.tick`. `id` by default comes purely from specs.id #id.
- However, since `id` comes from extras (specs), it MUST NOT be included in fields-inventory and in preset fields. We must remove them from there and plugin internal specific processor (including in heading and cite), everything originating from specs must remain universal and free from any plugin's default treatment.
- Remove `id` in plugin-heading. Even though plugin-section is active, `id` is held by section, not heading. Heading must not have a specific internal `id` mechanism. `id` on it MUST use the universal mechanism like the others.
- Completely remove `cite-id` in plugin-cite, it MUST use the universal mechanism like the others, the id implementation is left entirely to the user.
- All specific (non-universal) `id` implementations must be completely discarded from all plugins EXCEPT plugin-section. `id` must be universal because it resides in extras (specs `#foo`).
- Completely remove CITATION_CLAS, REF_HREF_PREFIX, CITATION_ANCHOR_PREFIX from plugin-cite. Let the rendered output be pure without any prefix and default values from Pendon. Users who need default classes and prefixes can use custom elements whose implementation has already been provided and is working well. Pendon MUST NOT provide that hardcoded.

These changes are made without nostalgia, without migration thingy, without warning.
