import { Bibliography } from '@comp/content/Marker'
import { NoteAside } from '@comp/content/Marker'

export default function PendonView() { return (<>
<p>The <code>type</code> of a marker is mandatory (§10.1) and is the routing key of the §11
component set. This fixture configures typed entries but no layer default, so an
unclaimed type falls back to the built-in <code>&lt;span&gt;</code> / <code>&lt;div&gt;</code>.
</p>
<p>Inline markers keep their paragraph: see <NoteAside type={"note"} />, <NoteAside type={"aside"} /> and the text after
them, plus an unclaimed one (<span type="unknown"></span>).
</p>
<p>The block form owns its line and carries the trailing text of that line as its
children, so a caption needs a template with <code>&#123;children&#125;</code> (§11 rule 6):
</p>
<Bibliography type={"bibliography"}>Every reference cited above.</Bibliography><p>An unclaimed block type renders the <code>&lt;div&gt;</code> fallback with the same children:
</p>
<div type="ghost">Trailing text of an unclaimed block marker.</div><p>The extras head must be adjacent, so only the first marker reads it:
<NoteAside class={"box"} id={"a1"} isOpen={true} type={"aside"} /> keeps its attributes, while
<NoteAside type={"aside"} /> @@asideAB&#123;.box&#125; does not.
</p>
<p>A marker without a legal type stays literal text: &#123;&#123;&#125;&#125;, &#123;&#123; note &#125;&#125;, &#123;&#123;a-b&#125;&#125; and
an unterminated &#123;&#123;note are all text.
</p>
<p>§6.1/§10.1: the groups sit between <code>&#125;&#125;</code> and an adjacent extras head and beat a
same-key body item (§6.2), while a malformed group is not consumed — the marker
still renders and the group stays literal text (§4.3):
</p>
<Bibliography class={"box"} slug={"refs"} title={"Cited sources"} type={"bibliography"}>Every reference cited above.</Bibliography><Bibliography type={"bibliography"}>[unclosed(&quot;Cited&quot;) still a caption.</Bibliography>
</>); }
