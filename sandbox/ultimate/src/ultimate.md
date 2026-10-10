---
# =============================================================================
# ULTIMATE.MD: COMBINATORIAL STRESS TEST FOR THE PENDON UNIFIED SYNTAX
# =============================================================================
# PURPOSE
#   One document that exercises every construct that reads the unified extras
#   head, every optional/partial form of that head, and the combinations and
#   nesting between plugins. It exists because per-plugin tests in "full
#   version only" (all items present) missed whole classes of bugs, e.g.
#   constructs written WITHOUT @@type, which is optional in the spec.
#   Paired with pendon.toml (3 tasks). Output is compared against a frozen
#   render; there is no assertion code, the diff IS the test.
#
# THIS IS NOT DOCUMENTATION
#   Prose is deliberately meaningless. Do not "improve" wording. Only syntax,
#   variants, combinations and nesting matter. Keep every case short.
#
# SOURCES OF TRUTH
#   docs/spec/SYNTAX.md  = frozen grammar (wins on any disagreement).
#   RFC (design history) = background only; may be outdated.
#   OPEN-* items in spec section 15 = known undecided behavior. Cases that
#   depend on them are marked "edge"; confirm against the spec before treating
#   a failure there as a bug.
#
# CANONICAL SYNTAX (maintainer decisions, keep this file consistent with them)
#   - `@@type` and `{...}` are TWO independent entities. Either may appear
#     alone, or both. When both appear, `@@type` comes first and they must be
#     adjacent: `@@type{...}`. A space between them makes the whole head
#     literal text.
#   - A type is ASCII alphanumeric only ([A-Za-z0-9], any case). Symbols,
#     including `-` and `_`, are never part of a type: they end it, and the
#     symbol plus everything after it is literal text. A type-only head
#     followed by punctuation stays valid: `[t](/a)@@anchorA.` renders the
#     anchor, then a literal ".".
#   - CANONICAL form of an extras head is bare braces: `[t](/a){.x}`.
#     `@@type` is only written when type routing is wanted.
#   - `@@{...}` (empty type) is allowed but NOT canonical. Stress test only,
#     in cases whose label ends with `n` (A07n, H10n...). Never the majority
#     of a section.
#   - Heads must TOUCH the construct they attach to. Only list and blockquote
#     are exempt (a space between marker, head and content is allowed there).
#   - Tables have no exception. Header/body cells: the head touches the cell's
#     opening `|` (`|{.x} text |`). Caption: `||{.c} caption ||`. Alignment
#     cells: the head goes AFTER the alignment code and touches it
#     (`:---(200px)@@cellA{.v}`). Trailing heads touch the last `|`
#     (`|{.row}`, `|===|{.foot}`).
#   - Table spans: a cell holding only `>` is merged into the cell on its right
#     (colspan); a cell holding only `^` is merged into the cell above
#     (rowspan). The cell holding the marker is sacrificed.
#   - Applies to every construct that uses `@@type`: anchor, heading, image and
#     figure, cite, wiki, table slots, blockquote, list decorators and items.
#   - Marker `{{type}}`, inline directive `::type` and block directive `==type`
#     carry the type inside their own fence, so they have no @@ form and no
#     `n` cases; their `{...}` follows the fence directly.
#   - Cite form is `[^^](ref "loc")` (unquoted ref, optional quoted loc). The
#     old forms `[^^]("id")`, `[^^]("id", "loc")` and the `loc=` prop are
#     removed and must stay literal, with no warning.
#   - The legacy extras form `[.class,#id]{k:v}` is removed and must stay
#     literal, with no warning.
#   - Pendon has NO setext headings (`Title` + `-----`). Use `#` headings for
#     every section break. `---` belongs to micromatter and `<hr />` only.
#   - Blockquote and list read the same unified head. A quote carries an inner
#     head right after `>` (§9.2, inner head over a decorator line); a list
#     carries a container **decorator line** above it that decorates the
#     container, never the items (§9.3 L1), the layer chosen by the marker
#     (§9.4). The Q and L sections exercise both.
#
# HOW THIS FILE WAS BUILT (repeat this method when extending it)
#   1. List every construct and every slot where an extras head can attach.
#   2. For EACH construct apply the variant matrix:
#        none | empty ({} / @@type{}) | type only (@@type) | X | XY | XXYZZ
#        | all item kinds | shuffled | typed | untyped | unclaimed type
#        | non-canonical @@{...} (label suffix n)
#      Item kinds: `slug`, "title", .class, #id, key: "str", key: 12,
#      key: true, --css-var: "x", bareFlag.
#   3. Cross-product by nesting: every inline construct inside every container
#      (heading, table cell, figure caption, list item, blockquote, directive),
#      and containers inside containers. KS (typed) and KSu (untyped) are the
#      two "kitchen sink" inline lines; reuse them, do not invent new ones.
#   4. Edge cases: literal/escaped/code (must NOT parse), malformed heads (must
#      stay literal, never abort the build), invalid types, spacing/adjacency
#      rules, delimiter collisions (| :: == {{ }} {} std::vector), duplicates,
#      Unicode, raw HTML.
#
# STRUCTURE (section id prefix -> construct)
#   A anchor   H heading   I image/figure   C cite   W wiki   M marker
#   T table    Q blockquote   L list   D inline directive   B block directive
#   N nesting matrix   E edge cases
#
# CONVENTIONS
#   - Every case has a stable label (A01, T05, N10...). Labels appear in the
#     rendered output and in bug reports. NEVER renumber or reuse a label.
#     Append new cases at the end of their section with the next free number;
#     the non-canonical twin of a case reuses its label plus `n`.
#   - RETIRED labels (do not reuse): H22 and E44 (setext, not supported).
#   - One case per paragraph/line, separated by blank lines, so one failure
#     cannot swallow its neighbours. Deliberate exceptions are labeled.
#   - Lists are separated from each other by `###` headings (no thematic
#     breaks). Keep the heading directly before the list or its decorator.
#   - HTML comments with expected behavior go only BETWEEN blocks, never inside
#     tables/lists/quotes.
#   - Type names (anchorA, headingX, cellB, bqA...) must match pendon.toml.
#     Some types (anchorZ, headingZ, unorderedZ, zzz...) are intentionally
#     UNCLAIMED in the config. Do not add config entries for them.
#   - The frontmatter `references` feed the C section. Changing ids there
#     requires updating every cite case.
#
# WHEN THE PROJECT CHANGES
#   New feature/plugin : add a section with the next letter prefix; apply the
#                        variant matrix; add it to KS/KSu and to the nesting
#                        matrix (N); add literal/malformed cases (E); add
#                        config entries in pendon.toml (see its header).
#   Syntax change      : update EVERY occurrence (grep the old form), update
#                        the edge cases that exercised the old rule, keep one
#                        case proving the old form is now literal or rejected.
#   Feature removed    : keep its cases, move them to E as "must stay literal".
#   Bug found          : add a minimal regression case in the matching section;
#                        never delete a failing case to make the diff green.
#   Always             : regenerate (pendon run -F), review the diff by hand
#                        for all 3 tasks, only then freeze the output.
#
# KNOWN UNCONFIRMED ASSUMPTIONS (resolve against the spec, then delete here)
#   - Leading digit in a type (`@@1{.x}`, E64): valid type or literal?
#   - Heads on a cell that holds `>` or `^` (T30), spans without a target
#     (T26), conflicting spans (T27), spans across tbody/tfoot (T29).
#   - A cell that starts with a marker touching `|` (T31).
#   - Spaced head in a delimiter row (T21).
#   - Decorator not directly above its list (L30, L40, L41, L46) and trailing
#     extras on table rows/tbody.
#   - THE RENDERED BODY STOPS AT L37. Everything from L40 on — including the
#     D, B, N and E sections — is missing from all three tasks: the extras head
#     of the H1 inside the L37 item swallows the rest of the source (the same
#     head scan also mis-renders Q24 in the subset task). Repro and status:
#     TODO.md, "Sandbox ultimate: the rendered body stops at L37".
# =============================================================================
title: "Unified Syntax Torture Demo"
references:
  suryana-2026:
    id: suryana-2026
    type: book
    title: Masa Depan Rekayasa Perangkat Lunak
    authors:
      - firstName: Eko
        lastName: Suryana
    publisher: TechPress Indonesia
    publisherLocation: Jakarta
    issuedDate:
      year: 2026
    isbn: 978-602-0000-00-0
    language: id
  paper-smith:
    id: paper-smith
    type: journal
    title: Generative MDX to PDF Compilation Architectures
    authors:
      - firstName: John
        lastName: Smith
    containerTitle: Journal of Web Engineering
    volume: "18"
    issue: "4"
    pages: 210-225
    issuedDate:
      year: 2025
      month: 8
    doi: 10.1016/j.jwe.2025.08.001
    language: en
  doe-2020:
    id: doe-2020
    type: book
    title: Edge Cases in Practice
    authors:
      - firstName: Jane
        lastName: Doe
    publisher: Test Press
    publisherLocation: Bandung
    issuedDate:
      year: 2020
    language: en
---

<!--
Legend: setiap kasus berlabel (A=anchor H=heading I=image C=cite W=wiki M=marker
T=table Q=blockquote L=list D=inline directive B=block directive N=nesting E=edge).
Kanonik: {...} tanpa @@. @@type hanya untuk routing. @@{...} = non-kanonik, label berakhiran n.
@@type dan {...} harus menempel; dipisah spasi = literal. Type = alfanumerik saja.
Sel tabel: head menempel ke | ; sel alignment: head setelah kode alignment. > = colspan, ^ = rowspan.
Varian extras: none | empty | type-only | X | XY | XXYZZ | all | shuffled | typed | untyped | unclaimed.
KS (typed) / KSu (untyped) = anchor + bold + italic + code + wiki + cite + marker + directive + math.
Label pensiun: H22, E44 (setext tidak didukung).
-->

#[s-top] Unified Demo

##[s-anchor] Anchor

A01 [plain](/a)

A02 [title](/a "T")

A03 [empty title](/a)

A04 [t](/a){}

A04n [t](/a)@@{}

A05 [t](/a)@@anchorA{}

A06 [t](/a)@@anchorA

A07 [t](/a){.x}

A07n [t](/a)@@{.x}

A08 [t](/a)@@anchorA{.x}

A09 [t](/a)@@anchorB{#i, .x}

A10 [t](/a)@@anchorC{.a, .b, #i, k: "v", n: 2}

A11 [t](/a)@@anchorA{`s-only`}

A12 [t](/a)@@anchorA{"Extras title"}

A13 [t](/a "Head title")@@anchorA{"Extras title"}

A14 [t](/a)@@anchorA{k: "v"}

A15 [t](/a)@@anchorA{n: 12} [t](/a)@@anchorA{n: -1.5}

A16 [t](/a)@@anchorA{b: true} [t](/a)@@anchorA{b: false}

A17 [t](/a)@@anchorA{--v: "2rem"}

A18 [t](/a)@@anchorA{isBar}

A19 [t](/a "Head")@@anchorA{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

A20 [t](/a)@@anchorB{isBar, --style-var: "2rem", foo: "bar", #id, "Title Foo", .extra, bar: 12, `slug-foo`, isFoo: true, .class}

A21 [t](/a)@@anchorB{`slug-extras`, #explicit, .from-extras}

A22 [t](/a)@@anchorZ{.hero}

A23 [abs](https://example.com/a?q=1&r=2#h)@@anchorA{.u}

A24 [rel](./a/b.md){.u}

A24n [rel](./a/b.md)@@{.u}

A25 [hash](#s-anchor){.u}

A26 [mail](mailto:a@b.co){.u}

A27 [mod](https://example.com/bar--!~$ "T")@@anchorB{rel: "prefetch", target: "_blank", hreflang: "en"}

A28 [mod2](https://example.com/bar--!;$)

A29 [paren](/a_(b)){.u}

A30 [**b** _i_ `c` $m$](/a)@@anchorA{.m}

A31 [a \[b\] c](/a){.e}

A32 [t](/a){k: "a, b} c", q: "x\"y", u: "日本 😀"}

A33 [a](/1){.a}[b](/2){.b}[c](/3)

A34 ([t](/a){.p}), [t](/a){.p}. **[t](/a){.p}** _[t](/a){.p}_

A35 [{{footnote}}{.k} in anchor](/a){.o} [::asideA d::](/a){.o}

A36 [t](/a){.a, .b, #i, k: "v", n: 2}

A36n [t](/a)@@{.a, .b, #i, k: "v", n: 2}

A37 [t](/a "Head"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

A37n [t](/a "Head")@@{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

A38 [t](/a){isBar, --style-var: "2rem", foo: "bar", #id, "Title Foo", .extra, bar: 12, `slug-foo`, isFoo: true, .class}

A39 [t](/a){#i, .x}

A40 [t](/a)@@anchorZ

A41 [t](/a){`s-only`}

A42 [t](/a){"Extras title"}

A43 [t](/a){k: "v"}

A44 [t](/a){--v: "2rem"}

A45 [t](/a){isBar}

A46 [t](/a "Head title"){"Extras title"}

A47 [t](/a){`slug-extras`, #explicit, .from-extras}

A48 [mod3](https://example.com/bar--!;$){rel: "prefetch"}

A49 [t](/a)@@anchorA.

A50 ([t](/a)@@anchorA)

A51 [t](/a)@@anchorA, [u](/b)@@anchorB; [v](/c)@@anchorA!

A52 [t](/a)@@anchor1{.x}

##[s-heading] Heading

###[h01] H01 slug

###("T02") H02 title

###@@headingX{.x} H03 type+X

### H04 plain

###[h05]("T05") H05 slug+title

###[h06]@@headingX{.x} H06 slug+X

###("T07")@@headingY{.x, #i} H07 title+XY

###[h08]("T08")@@headingX{.a, .b, #i, k: "v", n: 2} H08 XXYZZ

###[h09]("T09")@@headingX{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} H09 all

###{.x} H10 untyped

###@@{.x} H10n untyped non-canonical

###{} H11 empty

###@@{} H11n empty non-canonical

###@@headingZ{.x} H12 unclaimed

###@@headingX H13 type only

###[h14]("T14"){`slug-extras`, "Title extras", .x} H14 precedence

###[h14n]("T14n")@@{`slug-extras`, "Title extras", .x} H14n precedence non-canonical

#[h15]@@headingX{.l1} H15 level 1

##{.l2} H16 level 2

##@@{.l2n} H16n level 2 non-canonical

####[h17]@@headingY{.l4} H17 level 4

######("T18"){.l6} H18 level 6 (skips 5)

######("T18n")@@{.l6n} H18n level 6 non-canonical

###[h19]("T19")@@headingX{.k} H19 KS [a](/x "T")@@anchorA{.k} **b** _i_ `c` [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

<!-- H20: heading with empty text, typed -->

###@@headingX{.x}

###[h21] H21 closing hashes ###

<!-- H23: heading with empty text, untyped -->

###{.x}

###[h24]{.k} H24 KSu [a](/x "T"){.k} **b** _i_ `c` [[W]]{.k} [^^](paper-smith){.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

###[h25]("T25"){.a, #i, k: "v"} H25 slug+title+XXYZ untyped

###("T26"){.a} H26 title+X untyped

###[h27]{`slug-extras`, .a} H27 slug+X untyped, slug precedence

##[s-image] Image and figure

~!w800[I01](/i.png)

~!w800[I02](/i.png) Caption only

~!h300[I03](/i.png){}

~!h300[I03n](/i.png)@@{}

~!h300w800[I04](/i.png)@@imageX{.thumb}

~?!w800[I05](/i.png)@@imageX{.x, #i} Caption **b**

~?!h300w800[I06](/i.png)@@thumb{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Caption all

~?!!w800[I07](/i.png)

~?!!h300w800[I08](/i.png) Figure caption only

~?!!h300w800[I09](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)@@figureX{.wide, #fig-1, --rotate: "5deg", isLazy} Caption **b**

~?!!w800[I10](/i.png){`s-i10`}

~?!!w800[I10n](/i.png)@@{`s-i10n`}

~?!![I11](/i.png)@@figureX{}

<!-- I12: figure with empty alt, untyped -->

~?!!w800[](/i.png){.empty-alt}

~?!!h300[I13](/i.png)@@figureZ{.x} unclaimed

~!w100[I14](/i.png "Head T")@@imageX{"Extras T", .a, .b, #i, k: "v", n: 2}

~?!!w100[I15](/i.png)@@figureX{isA} Caption KS [a](/x)@@anchorA{.k} [^^](paper-smith)@@citeX{.k} [[W]]@@wikiX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$ **b**

![I16 plain markdown image](/i.png)

~!h300w800[I17](/i.png){.thumb}

~?!h300w800[I18](/i.png){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Caption all

~?!h300w800[I18n](/i.png)@@{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Caption all

~?!!h300w800[I19](/i.png){.wide, #fig-2, --rotate: "5deg", isLazy} Caption

~?!![I20](/i.png){}

~?!![I21](/i.png)@@figureX

~!w100[I22](/i.png "Head T"){"Extras T", .a}

~?!!w100[I23](/i.png){.f} Caption KSu [a](/x){.k} [^^](paper-smith){.k} [[W]]{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$ **b**

##[s-cite] Cite

<!-- Cite form: [^^](id "loc"). The old [^^]("id", "loc") is obsolete. -->

C01 Minim [^^](paper-smith) esse.

C02 Minim [^^](paper-smith "hlm. 55") esse.

C03 Minim [^^](paper-smith){} esse.

C03n Minim [^^](paper-smith)@@{} esse.

C04 Minim [^^](paper-smith)@@citeX{} esse.

C05 Minim [^^](paper-smith)@@citeX{.paper} esse.

C06 Minim [^^](paper-smith)@@citeX{.paper, #smith} esse.

C07 Minim [^^](suryana-2026 "hlm. 45")@@citeX{.a, .b, #i, note: "short", n: 2} esse.

C08 Minim [^^](doe-2020)@@citeX{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} esse.

C09 [^^](paper-smith "hlm. 1")@@citeX{loc: "dropped", .thin}

C10 [^^](missing-ref)@@citeX{.x}

C11 [^^](paper-smith) [^^](suryana-2026){.y}[^^](doe-2020)

C12 [^^](paper-smith) dan lagi [^^](paper-smith)@@citeX{.again}

C13 [^^]()

C14 [^^](paper-smith)@@citeZ{.x} unclaimed

C15 [^^](paper-smith)@@citeX

C16 Minim [^^](paper-smith){.paper} esse.

C17 Minim [^^](paper-smith "hlm. 7"){.paper, #smith2} esse.

C18 Minim [^^](suryana-2026 "hlm. 45"){.a, .b, #i2, note: "short", n: 2} esse.

C19 Minim [^^](doe-2020){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} esse.

C19n Minim [^^](doe-2020)@@{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} esse.

<!-- C20-C22: retired cite forms. Expected: plain literal text, no warning. -->

C20 [^^](paper-smith loc="p.1")

C21 [^^]("paper-smith")

C22 [^^]("paper-smith", "hlm. 55")

C23 Minim [^^](paper-smith)@@citeX. esse

##[s-wiki] Wiki

W01 [[Wireless]]

W02 [[Anim Esta (Officia) | Anim]]

W03 [[Wireless]]{}

W03n [[Wireless]]@@{}

W04 [[Wireless]]@@wikiX{.link}

W05 [[Wireless]]@@wikiX{.a, #i}

W06 [[Wireless]]@@wikiX{.a, .b, #i, k: "v", n: 2}

W07 [[Anim Esta]]@@wikiX{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

W08 [[Wireless]]@@wikiX{href: "/evil", title: "dropped"}

W09 [[Café Ünïcode 日本]]{.u}

W09n [[Café Ünïcode 日本]]@@{.u}

W10 [[a]]{.a}[[b]]{.b}[[c]]

W11 [[ ]] [[a|b|c]] [[]]

W12 [[Wireless]] @@wikiX{.spaced}

W13 [[Wireless]]@@wikiZ{.x}

W14 [[Wireless]]@@wikiX {.x} type and brace separated by space

W15 [[Wireless]] {.x} bare brace separated by space

W16 [[Wireless]]{.link}

W17 [[Wireless]]{.a, #i}

W18 [[Wireless]]{.a, .b, #i, k: "v", n: 2}

W19 [[Anim Esta]]{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

W19n [[Anim Esta]]@@{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

W20 [[Wireless]]{href: "/evil", title: "dropped"}

W21 [[Wireless]]@@wikiX.

##[s-marker] Marker

M01 inline {{footnote}} ipsum.

M02 inline {{footnote}}{.x} ipsum.

M03 inline {{footnote}}{`s`, .a, #i} ipsum.

M04 inline {{footnote}}{.a, .b, #i, k: "v", n: 2} ipsum.

M05 inline {{markerA}}{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} ipsum.

M06 {{footnote}}{}

{{bibliography}}

{{bibliography}}{.x}

{{markerA}} trailing text as children

{{markerA}}{.x, #i} trailing **text** [a](/x)@@anchorA{.k}

{{markerA}}{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

{{unknownType}} unclaimed falls to default

{{unknownType}}{.u}

M10 {{footnote}}{.a} dan {{footnote}} dan {{markerA}}{.b}{{markerA}}

M11 {{}} {{bad-type}} {{bad_type}} {{type1}} {{ spaced }} {{footnote}} {.spaced}

M12 {{Footnote}}{.upper} {{FOOTNOTE}}

<!-- §6.1/§10.1 groups: a bracket group directly after `}}` fills the marker's
bracket_key (slug) and a parentheses group fills parentheses_key (title). A bracket
group immediately followed by a parentheses group is also an anchor head, and this task
lists `anchor` before `marker`, so the anchor layer claims such a pair first (§6.1 layer
order) and the marker renders bare. -->

M13 {{markerA}}[m13] bracket group, trailing text as children

M14 {{markerA}}("M14") parentheses group, trailing text as children

M15 {{markerA}}[m15]@@asideA{.h} group then extras head

M16 {{markerA}}[m16( malformed group stays literal, marker still renders

M17 {{markerA}}[m17]("M17") anchor layer wins the pair, marker stays bare

M18 {{unknownType}}[m18] unclaimed type keeps the group

M19 inline {{footnote}}[m19] and {{markerA}}("M19") ipsum.

##[s-table] Table

<!--
Placement: heads touch the cell's opening | ; alignment cells take the head AFTER the
alignment code ; trailing heads touch the last | . A cell holding only > is merged into the
cell on its right (colspan) ; only ^ is merged into the cell above (rowspan).
-->

|-[t01]("Table 01")@@tableX{`slug-foo`, "Title Foo", .striped, .wide, #tbl, sortable: true, cols: 4, --gap: "2rem", isDense}-|
||@@captionX{.note, #cap} T01 caption **b** ||
|{.h1} H1 |@@cellA{.h2} H2 | H3 | H4 |
| :---(200px)@@cellA{.v-top} | :---:{.c} | ---:(30%) | :---@@cellB{.a, #i, k: "v", n: 2} |@@tbodyX{.tb}
|@@cellB{.lead} X | 15 | [Ada](/docs)@@anchorA{.u} | [[W]]@@wikiX{.w} |
| plain | > | [^^](paper-smith)@@citeX{.c} | {{footnote}}{.m} |@@rowB{.row-info}
|{} empty-extras | ::asideA{.k} d:: | $x^2$ | `c` |@@rowA{}
| a | b | c | d |{.untyped-row}
| \| escaped | | | |@@rowB{.a, #r, k: "v", n: 2}
| all-row | x | y | z |@@rowB{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
|@@cellA{.only-extras} | **b** _i_ | [a](/x "T")@@anchorA{.k} [^^](paper-smith)@@citeX{.k} |@@cellB{`s`, "T", .a, #i, k: "v", n: 1, b: true, --v: "1", flag} KS |
|===|@@tfootX{.total}
| Total | > | > | 1 |@@rowA{.f}
|@@cellA{.t} F2 | x | y | z |

T01n non-canonical @@{...} on every table slot:

|-[t01n]@@{.x}-|
||@@{.c} T01n caption ||
| A | B |
| :---@@{.v} | --- |@@{.tb}
|@@{.l} x | y |@@{.r}
|===|@@{.f}
| t | u |

T02 plain, no declaration, no caption, no extras:

| A     | B |
| ----- | - |
| 1     | 2 |
| 3     | 4 |
| ===   |   |
| total | 6 |

|| T03 caption only ||

| A | B |
| - | - |
| 1 | 2 |

|- -|

| T04 A | B |
| ----- | - |
| 1     | 2 |

|-[t05]-|

| T05 A | B |
| ----- | - |
| 1     | 2 |

|-("Table 06")-|

| T06 A | B |
| ----- | - |
| 1     | 2 |

|-{.x}-|

| T07 A | B |
| ----- | - |
| 1     | 2 |

|-@@{.x}-|

| T07n A | B |
| ------ | - |
| 1      | 2 |

|-@@tableX-|

| T08 A | B |
| ----- | - |
| 1     | 2 |

||{.c} T09 caption extras only ||

| A | B |
| - | - |
| 1 | 2 |

||@@{.c} T09n caption extras only ||

| A | B |
| - | - |
| 1 | 2 |

|-[t10]-|
|| T10 plain caption ||

| A | B |
| - | - |
| 1 | 2 |

|-[t11]("T11")@@tableZ{.x}-|
||@@captionZ{.y} T11 unclaimed ||

| A | B |
| - | - |
| 1 | 2 |

T12 header only, no body:

| A  |  B |
| :- | -: |

T13 footer without body rows:

| A   |
| --- |
| === |
| tot |

T14 ragged:

| A | B | C |
| - | - | - |
| 1 |   |   |
| 1 | 2 | 3 |
| > | > | > |

T15 alignment/width variants:

| a | b | c | d | e | f |
| --- | :--- | :---: | ---: | ---(10%) | :---:(120px) |
| 1 | 2 | 3 | 4 | 5 | 6 |

T16 one column:

| Only |
| ---- |
| x    |

T17 table directly after paragraph line

| A | B |
| - | - |
| 1 | 2 |

T18 untyped canonical on every table slot:

|-[t18]("Table 18"){`slug-foo`, "Title Foo", .striped, .wide, #tbl18, sortable: true, cols: 4, --gap: "2rem", isDense}-|
||{.note, #cap18} T18 caption **b** ||
|{.h1} H1 |{.h2} H2 | H3 | H4 |
| :---(200px){.v-top} | :---:{.c, #i} | ---:(30%){.a, .b, #i2, k: "v", n: 2} | :---{`s`, "T", .a, #i3, k: "v", n: 1, b: true, --v: "1", flag} |{.tb}
|{.lead} X | 15 | [Ada](/docs){.u} | [[W]]{.w} |
| plain | > | [^^](paper-smith){.c} | {{footnote}}{.m} |{.row-info}
|{} empty | ::asideA{.k} d:: | $x^2$ | `c` |{}
| a | b | c | d |{.a, #r, k: "v", n: 2}
| all-row | x | y | z |{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
|===|{.total}
| Total | > | > | 1 |{.f}
|{.t} F2 | x | y | z |

T19 type and brace separated by space: head stays literal:

|-[t19]@@tableX {.x}-|
||@@captionX {.y} T19 caption ||

| A              | B |
| -------------- | - |
| @@cellA {.z} 1 | 2 |

T20 head not touching the pipe: stays literal (no exception for tables):

|| {.c} T20 caption spaced ||

| A                | B                             |
| ---------------- | ----------------------------- |
| {.z} spaced cell | @@cellA{.y} spaced typed cell |

<!-- T21: spaced head in a delimiter row. Not a valid delimiter row, so the table may not form (edge). -->

| A | B |
| :--- {.v} | --- @@cellA{.w} |
| 1 | 2 |

T22 rowspan (^ merges into the cell above):

| A  | B | C |
| -- | - | - |
| r1 | x | y |
| ^  | p | q |
| ^  | s | t |

T23 colspan (> merges into the cell on its right), also in the header:

| A | > | C |
| - | - | - |
| 1 | > | 3 |
| x | y | z |

T24 chains and full-width spans:

| A | B | C | D        |
| - | - | - | -------- |
| > | > | > | all four |
| 1 | > | > | three    |
| > | 2 | 3 | 4        |

T25 colspan and rowspan forming a 2x2 block:

| A | B         | C |
| - | --------- | - |
| > | block 2x2 | z |
| ^ | ^         | y |
| p | q         | r |

<!-- T26: spans without a target: ^ in the header, ^ right under the header row, > in the last column (edge). -->

| A | B | ^ |
| - | - | - |
| ^ | 1 | > |
| a | b | c |

<!-- T27: conflicting/ambiguous spans: ^ under a merged cell, > next to ^ (edge, no expectation). -->

| A | B | C |
| - | - | - |
| 1 | > | 3 |
| > | ^ | 4 |
| ^ | ^ | > |

T28 lookalikes that must stay literal text:

| a > b | >= | >> | x^2 | ^^ | \> | \^ | > x | ^ y |
| ----- | -- | -- | --- | -- | -- | -- | --- | --- |
| 1     | 2  | 3  | 4   | 5  | 6  | 7  | 8   | 9   |

<!-- T29: spans in tfoot, including ^ right under the tbody/tfoot boundary (edge). -->

| A   | B | C |
| --- | - | - |
| 1   | 2 | 3 |
| === |   |   |
| ^   | > | t |
| ^   | x | y |

<!-- T30: heads on cells that hold > or ^, and a head on the target cell (edge, no expectation). -->

| A           | B                     | C |
| ----------- | --------------------- | - |
|{.s} >      | x                     | y |
| @@cellB{} > | x                     | y |
|{.s} ^      | x                     | y |
| a           |{.s} >                | y |
| >           | @@cellB{.span} merged | z |

T31 a cell starting with a marker touching the pipe (head vs marker, edge):

| A                  | B             | C |
| ------------------ | ------------- | - |
|{{footnote}}{.m} x |{{markerA}} y | z |
|{{footnote}}       | w             | v |

##[s-quote] Blockquote

<!-- §9.2: inner head (right after `>`) + a decorator line directly above the quote. The inner head wins; an unclaimed type falls back to the layer default (§11 rule 3). §6.1 groups: a bracket group becomes the inner head's slug and a parentheses group its title; a bracket group immediately followed by a parentheses group is an anchor head and the anchor layer runs first, so the head keeps only its type. -->

> Q01 plain
> continues

> {.x} Q02 untyped X

> @@{.x} Q02n untyped non-canonical

> @@bqA{} Q03 typed empty

> @@bqA{.x} Q04 X

> @@bqA{.x, #i} Q05 XY

> @@bqA{.a, .b, #i, k: "v", n: 2} Q06 XXYZZ

> @@bqB{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Q07 all

> @@bqZ{.x} Q08 unclaimed

> @@bqA Q09 type only

>@@bqA{.x} Q10 no space after >

> Q11 not first @@bqA{.x} literal

> @@bqA{.x}

> @@bqA{.outer} Q13 outer
>
>> @@bqB{.inner} Q13a inner
>>
>>> Q13b triple plain
>>> back to outer lazy
>>> lazy continuation without marker

> @@bqA{.q} Q14 mixed
>
> ### Q14 heading inside
>
> - @@liItem{.i} Q14 item
> - Q14 item 2
>
> | A | B |
> | - | - |
> | 1 | 2 |
>
> ```ts
> const q = "@@bqA{.literal}";
> ```
>
> $$x = y$$
>
> {{markerA}}{.q}

> @@bqA{.q15} KS [a](/x "T")@@anchorA{.k} **b** _i_ `c` [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

> {} Q16 untyped empty

> {.x, #i} Q17 untyped XY

> {.a, .b, #i, k: "v", n: 2} Q18 untyped XXYZZ

> {`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} Q19 untyped all

>{.x} Q20 untyped, no space after >

> Q21 not first {.x} literal

> {.x}

> @@bqA {.x} Q23 type and brace separated by space (literal)

> @@bqA[q25] Q25 bracket group on the inner head

> @@bqA[q26( Q26 malformed inner-head group stays literal, type falls back

> {.q24} KSu [a](/x "T"){.k} **b** _i_ `c` [[W]]{.k} [^^](paper-smith){.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

##[s-list] List

<!-- §9.3 L1: a container decorator line above a list decorates the container (§9.4 layer = the marker). The `list` item layer (L2/L3) is not wired yet, so the L10/L22 item heads stay literal for now. -->

###[l01] L01

- L01 plain
  - L01a
- L01b

###[l02] L02

{.u}
- L02 decorator untyped X
- L02b

###[l02n] L02n

@@{.u}
- L02n decorator non-canonical
- L02nb

###[l03] L03

@@unorderedA{.u, #l03}
- L03 star, XY
- L03b

###[l04] L04

@@compact{.a, .b, #l04, k: "v", n: 2}
- L04 plus, XXYZZ
- L04b

###[l05] L05

@@unorderedA{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}
- L05 all
- L05b

###[l06] L06

@@unorderedZ{.u}
- L06 unclaimed

###[l07] L07

{}
- L07 empty

###[l07n] L07n

@@{}
- L07n empty non-canonical

###[l08] L08

@@unorderedA
- L08 type only

###[l10] L10

- @@liItem{`alpha`} L10 typed item
  - {`beta`} L10a nested untyped
  - @@liItem{.a, #i} L10b XY
- @@check{.c, k: "v", n: 2} L11 XXYZZ
- @@data{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} L12 all
- @@liZ{.z} L13 unclaimed
- {} L14 empty
- @@{} L14n empty non-canonical
- @@liItem L15 type only
- L16 plain among extras
- {.only}
- @@liItem{.x}L17 glued
- {.x}L17u glued untyped
- @@liItem {.x} L17s type and brace separated by space (literal)

###[l20] L20

{`my-list`}
6. L20 start at 6
7. L20b

###[l20n] L20n

@@{`my-list-n`}
6. L20n start at 6 non-canonical
7. L20nb

###[l21] L21

1. L21 plain ol
2. L21b

###[l22] L22

@@orderedA{.o, #l22}
1. {.i} L22 container+item
2. @@liItem{.a, .b, #i2, k: "v", n: 2} L22b

###[l22n] L22n

@@{.o}
1. @@{.i} L22n non-canonical container+item

###[l24] L24

0. L24 start 0
1. L24b

###[l25] L25

{start: 3}
1. L25 start via extras (expect Warning)

###[l25n] L25n

@@{start: 3}
1. L25n start via non-canonical extras (expect Warning)

###[l26] L26

o. L26 retired o. syntax (expect plain paragraph)

###[l30] L30

@@unorderedA{.l1}
- L30 level 1
  @@orderedA{.l2}
  1. L30a decorator right under item text (edge)
     - L30a1
  2. L30b
- L33 level 1
  @@orderedA{.l2b}
  1. L33a decorator after blank, inside item
  2. L33b

###[l34] L34

- L34 multi-paragraph

  second paragraph KS [a](/x)@@anchorA{.k} **b** [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} ::asideA{.k} d::

  ```ts
  const l = "@@{.literal}";
  ```

- L35 table in item

  | A | B |
  | - | - |
  | 1 | 2 |

- L36 quote in item

  > @@bqA{.in-li} quoted

- L37 directive in item

  # ===asideA[s-l37]("L37"){.d}body

###[l40] L40

{.u}
- L40 blank line between decorator and list (edge: decorator not directly above)

###[l40n] L40n

@@{.u}
- L40n blank line between non-canonical decorator and list (edge)

###[l41] L41

@@orphan{.x}
L41 orphan decorator with no list below (edge)

###[l42] L42

{.u, #l42}
- L42 container untyped XY
- L42b

###[l43] L43

{.a, .b, #l43, k: "v", n: 2}

1. L43 container untyped XXYZZ, separated by linebreak
2. L43b

###[l44] L44

{`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

- {`slug-item`, "Title Item", .extra, .class, #id2, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} L44 container+item all untyped
- {.c, k: "v", n: 2} L45 item untyped XXYZZ

###[l46] L46

{.x}

L46 orphan untyped decorator with no list below (edge)

##[s-idir] Inline directive

D01 ::asideA text::

D02 ::asideA[s-d02] text::

D03 ::asideA("T03") text::

D04 ::asideA[s-d04]("T04") text::

D05 ::asideA{.x} text::

D06 ::asideA[s-d06]("T06"){.x} text::

D07 ::asideA{.a, .b, #i, k: "v", n: 2} text::

D08 ::asideA[s-foo]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar} text::

D09 :: untyped text ::

D10 ::{.x} untyped extras::

D11 ::[s-d11]("T11"){.x} untyped all::

D12 ::note[n12]("2"){.x} custom keys::

D13 ::zzz unclaimed::

D14 ::asideA::

D15 ::asideA ::

D16 ::asideA{k: "a::b"} colons in string::

D20 :::::::asideA a ::::::asideB b :::::asideA c ::::asideB d :::asideA e ::asideB f:: e2 ::: d2 :::: c2 ::::: b2 :::::: a2 :::::::

D21 ::asideA a:: dan ::asideB b:: dan ::asideA{.c} c::

D22 :x one colon: literal

D23 ::::::::asideA eight colons::::::::

D24 ::asideA{.o} [a](/x)@@anchorA{.k} [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} $x$ **b** `c`::

D25 ::asideA unterminated directive stays literal

D26 ::asideA soft
break inside::

D27 ::asideA a::::asideB b::

D30 false positives: std::vector, std::collections::HashMap, a::b::c, 12:30:45, http://x.co

D31 ::type-x invalid type:: ::type_x invalid::

D32 ::asideA{.o} [a](/x){.k} [[W]]{.k} [^^](paper-smith){.k} {{footnote}}{.k} $x$ **b** `c`:: untyped inner heads

##[s-bdir] Block directive

# ===asideAB01 typed plain

===asideA[s-b02]

B02 slug

===

# ===asideA("T03")B03 title

===asideA[s-b04]("T04"){.x}

B04 slug+title+X

===

===asideA{.a, .b, #i, k: "v", n: 2}

B05 XXYZZ

===

===asideB[s-foo]("Foo Bar"){`slug-foo`, "Title Foo", .extra, .class, #id, foo: "bar", bar: 12, isFoo: true, --style-var: "2rem", isBar}

B06 all

===

===[s-b07]("T07"){.x}

B07 untyped

===

=={.x}

B08 untyped min fence, extras only

==

==asideA

B09 min fence

==

=======asideA

B10 max fence

=======

===note[n11]("3")

B11 custom keys

===

===zzz

B12 unclaimed

===

===asideA

===

=======asideA

L7

======asideB

L6

=====note[n5]("2")

L5

====asideA

L4

===asideB

L3

==note

L2

==

L3b

===

L4b

====

L5b

=====

L6b

======

L7b

=======

===asideA[s-b20]("B20"){.mix}

###[b20-h]("B20 H")@@headingX{.k} Heading inside

@@unorderedA{.k}
- @@liItem{.k} item
- item

@@orderedA{.k}
3. a
4. b

> @@bqA{.k} quote

|-[b20-t]-|
||{.k} caption ||

| A             | B |
| ------------- | - |
| @@cellA{.k} 1 | 2 |

~?!!w100[B20](/i.png)@@figureX{.k} cap

{{markerA}}{.k}

KS [a](/x "T")@@anchorA{.k} **b** _i_ `c` [[W]]@@wikiX{.k} [^^](paper-smith)@@citeX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

$$a \to b$$

```rust
let x = "===asideA";
```

==asideB{.inner}

B20 inner block with inline ::asideA{.k} d:: and ::::asideB x :::asideA y::: z::::

==

===

> ===asideA[s-b21]
> B21 unterminated block inside quote (closed by container end)

<!-- B22 stray closer below (blank line before it) -->

==

===asideA[s-b23]{.mix}

###[b23-h]("B23 H"){.k} Heading untyped inside

{.k}
- {.k} item untyped
- item

> {.k} quote untyped

|-[b23-t]-|
||{.k} caption ||

| A      | B |
| ------ | - |
|{.k} 1 | 2 |

~?!!w100[B23](/i.png){.k} cap

KSu [a](/x "T"){.k} **b** _i_ `c` [[W]]{.k} [^^](paper-smith){.k} {{footnote}}{.k} ::asideA{.k} d:: $x$

===

##[s-nest] Nesting

N01 **[a _b_ `c`](/x)@@anchorA{.k}** _[[W]]@@wikiX{.k}_ ~~[x](/a){.s}~~ **[^^](paper-smith)@@citeX{.k}** _{{footnote}}{.k}_

N02 ::asideA{.o} **bold ::asideB{.i} italic [a](/x)@@anchorA{.k}:: end** after::

~?!!w100[N03](/i.png)@@figureX{.f} Cap [a](/x)@@anchorA{.k} [^^](paper-smith)@@citeX{.k} [[W]]@@wikiX{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$ **b**

N04 **[a _b_ `c`](/x){.k}** _[[W]]{.k}_ ~~[x](/a){.s}~~ **[^^](paper-smith){.k}** _{{footnote}}{.k}_

~?!!w100[N05](/i.png){.f} Cap KSu [a](/x){.k} [^^](paper-smith){.k} [[W]]{.k} {{footnote}}{.k} ::asideA{.k} d:: $x$ **b**

> @@bqA{.deep}
>
> - @@liItem{.l1} N10 L1 KS [a](/x)@@anchorA{.k} {{footnote}}{.k}
>   1. {.l2} N10 L2
>      ===asideA[n-1]("N"){.d}
>      N10 body ::asideB{.k} d::
>      | a                       | b             |
>      | ----------------------- | ------------- |
>      | KS [a](/x)@@anchorA{.k} | @@cellB{.k} z |
>      | ===                     |               |

- @@liItem{.l1} N11 table with images and extras

  |-[n11]-|
  | A                                   | B                                      |
  | ----------------------------------- | -------------------------------------- |
  | ~!w50[N11a](/i.png)@@imageX{.c} cap | ~?!!w50[N11b](/i.png)@@figureX{.c} cap |

- N12 figure in list

  ~?!!w100[N12](/i.png)@@figureX{.l} cap

> ~?!!w100[N13](/i.png)@@figureX{.q} cap in quote

===asideA[n-14]("N14"){.o}

> @@bqA{.in-dir}
>
> - @@liItem{.in-bq} item with ::asideB{.k} d::

===

|-[n15]-|

| @@cellA{.k} ::asideA{.k} [a](/x)@@anchorA{.k} d:: | @@cellB{.k2} {{footnote}}{.k} [[W]]@@wikiX{.k} $x$ [^^](paper-smith)@@citeX{.k} |
| ------------------------------------------------- | ------------------------------------------------------------------------------- |
| **[a](/x)@@anchorA{.k}**                          | `c`                                                                             |

> {.deep2}
>
> - {.l1} N16 L1 KSu [a](/x){.k} {{footnote}}{.k}
>   1. {.l2} N16 L2
>      ===asideA[n-16]("N"){.d}
>      N16 body ::asideB{.k} d::
>      | a               | b      |
>      | --------------- | ------ |
>      | KSu [a](/x){.k} |{.k} z |
>      | ===             |        |

- {.l1} N17 table with images, untyped

  |-[n17]-|
  | A                           | B                             |
  | --------------------------- | ----------------------------- |
  | ~!w50[N17a](/i.png){.c} cap | ~?!!w50[N17b](/i.png){.c} cap |

|-[n18]-|

|{.k} ::asideA{.k} [a](/x){.k} d:: |{.k2} {{footnote}}{.k} [[W]]{.k} $x$ [^^](paper-smith){.k} |
| --------------------------------- | ---------------------------------------------------------- |
| **[a](/x){.k}**                   | `c`                                                        |

##[s-edge] Edge cases

<!-- E00: everything in code must stay literal -->

```md
# @@anchorA{.x} {.x} {{footnote}}{.k} ::asideA x:: [[W]]@@wikiX{.k}[t](/a){.x}~?!!w1[a](/i.png)@@figureX{.f} cap===asideAx
```

E01 inline code: `[t](/a)@@anchorA{.x}` `[t](/a){.x}` `{{footnote}}{.k}` `::asideA x::` `[[W]]@@wikiX{.k}`

E02 escapes: \@@anchorA{.x} [t](/a)\@@anchorA{.y} [t](/a)\{.x} \{{footnote}} \::asideA x\:: \[[Wireless]] \~!w100[a](/i.png) \===asideA

E03 stray: @@ and @@ x and a@@b.co and @@1{.x} and {{ }} and {} and @@{

E03u stray bare braces: {.x} and { } and {} and text {.x} text

E10 [t](/a){foo: "bar" and the rest of the line is text.

E10n [t](/a)@@{foo: "bar" and the rest of the line is text.

E11 [t](/a) @@anchorA{.x} head separated by space

E12 [t](/a)@@anchorA {.x} brace separated by space

E13 [t](/a){foo bar}

E14 [t](/a){.x,}

E15 [t](/a){,.x}

E16 [t](/a){.x .y}

E17 [t](/a){.a, .a, #i, #j, k: 1, k: 2}

E18 [t](/a){k: "unterminated}

E19 [t](/a){k: }

E20 [t](/a){: "v"}

E21 [t](/a){--: "v"} [t](/a){.} [t](/a){#} [t](/a){``} [t](/a){""}

E22 [t](/a)@@type-x{.x}

E23 [t](/a)
@@anchorA{.x} head on next line

E24 [t](/a){k: "@@x{.y}"}

E25 [t](/a){.a,
.b} multi-line head

E26 [t](/a)@@anchorA{.x}@@anchorB{.y} double head

E27 [t](/a)@@anchorA{.x}{.y}

E28 [t](/a){data-x: "1", aria-label: "l"}

E29 [t](/a){k: 1e3, n: 0x10, m: .5, p: +1}

E30 [t](/a){--a b: "x"} [t](/a){`bad slug`} [t](/a){`UPPER_Slug`}

E31 [日本語](/a "Ünïcode 😀"){"Título", k: "é", .ünï}

E31n [日本語](/a "Ünïcode 😀")@@{"Título", k: "é", .ünï}

E32 ###[bad slug] heading with spaced slug

###[h-e33] E33 heading with [t](/a){.x} and bad head [u](/b){.y

<span class="raw">E40 raw html</span>{.x}

<span class="raw">E40n raw html</span>@@{.x}

<div class="raw">

E41 markdown inside html [a](/x){.k} **b**

</div>

<!-- E42 comment -->

E43 text before comment <!-- inline comment --> text after

E45 $inline $x \to y$ math$ and $$display$$

inline and \$escaped\$

E46 [t](/a) {.x} bare brace separated by space

E47 [t](/a)@@ {.x} at-signs separated from brace by space

E48 [t](/a)@@anchorA text after a type-only head

E49 [t](/a)@@anchorA{.x} {.y} second brace separated by space

E50 [t](/a)@@anchorA{k: "unterminated}

E51 [t](/a)
{.x} bare head on next line

E52 [t](/a){.x}{.y}

E53 [t](/a)@@anchorA{foo bar}

E54 [t](/a)@@anchorA{.x,}

E55 [t](/a)@@anchorA{.a, .a, #i, #j, k: 1, k: 2}

E56 [t](/a)@@anchorA.

E57 ([t](/a)@@anchorA)

E58 [t](/a)@@anchorA, [u](/b)@@anchorB; [v](/c)@@anchorA!

E59 [t](/a)@@anchorA{.x}.

E60 [t](/a)@@type_x{.x}

E61 [t](/a)@@Type{.x}

E62 [t](/a)@@TYPE{.x}

E63 [t](/a)@@type1{.x}

E64 [t](/a)@@1{.x}

E65 [t](/a)@@anchorA.x

<!-- E66-E69: retired legacy extras form [.class,#id]{k:v}. Expected: plain literal text, no warning. -->

E66 [t](/a)[.extra,.class,#id]{foo: "bar"}

~?!h300w800[E67](/i.png)[.extra,.class,#id]{foo: "bar"} Figcaption

###[h-e68][.extra,.class,#id]{foo: "bar"} E68 legacy heading

[E69 legacy caption][.extra,.class,#id]{foo: "bar"}
| Qoo | Roo |
| :---(200px)[.extra,.class,#id] | :---:[.v-top] |
| Foo -[.extra,.class,#id] | Bar { foo: "bar "} |

$$((H \rightarrow O) \land \lnot O) \rightarrow \lnot H$$

```rust
fn extras(head: &str) -> Option<Head> {
    parse_extras(head).ok()
}
```

```
no language @@{.literal} {.literal}
```

{{bibliography}}
