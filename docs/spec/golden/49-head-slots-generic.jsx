import { Card } from '@comp/Directive'

export default function PendonView() { return (<>
<p>The <code>[bracket]</code> / <code>(paren)</code> groups of a <code>::</code> or <code>==</code> head are the <strong>same two slots</strong>
a tagged head carries (§5.2, §6.1): each holds a bare, a quoted and a ticked
content, and nothing is read as &quot;one string per group&quot;. With no <code>field.*</code>
assignment every content keeps its qualified name, which a built-in element
writes as a dotted spread (§6.5):
</p>
<p><span {...{ "bracket.bare": "bare" }} {...{ "bracket.quote": "quoted" }} {...{ "bracket.tick": "ticked" }} {...{ "paren.bare": "pbare" }} {...{ "paren.quote": "pquoted" }} {...{ "paren.tick": "pticked" }} type="plain"> the whole head is text</span>
</p>
<p>A component entry sees the same six positions. A <code>field.*</code> assignment renames
exactly one of them (D-7), and the bracket&#39;s <code>|…</code> run contributes <strong>last</strong>, so a
field the head wrote itself wins (§5.4, D-9):
</p>
<p><Card bracket={{ quote: "quoted", tick: "ticked" }} class="base run" data-mark="yes" paren={{ bare: "pbare" }} slug="pticked" subtitle="pquoted" title="bare" type="card"> body</Card>
</p>
<p>The block form reads its head the same way (§10.3), and a position the entry does
not rename keeps its qualified name:
</p>
<Card bracket={{ quote: "quoted", tick: "ticked" }} paren={{ bare: "pbare" }} slug="pticked" subtitle="pquoted" title="bare" type="card"><p>block body
</p>
</Card>
</>); }
