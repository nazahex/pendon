import { MoveA } from '@comp/MoveA'

export default function PendonView() { return (<>
<p>§6.1: <code>field.specs.*</code> <strong>moves</strong> a position to a public field of the author&#39;s own.
The entry below assigns all five, so every content lands exactly where it was put
— no <code>class</code> / <code>id</code> / <code>style</code> rename (§6.3 rule 1) and no default path left
behind (§6.1):
</p>
<p><MoveA bar.id="john-foo" foo.class="price-detail" foo.style="--width: 200px" href="/pricing" slug="price_slug" title="Price Title" type="moveA">Moved</MoveA>
</p>
<p>The same item kinds on an entry that assigns nothing keep the default paths:
<code>class</code> / <code>id</code> / <code>style</code> are the DOM fields they name (§6.3), and the anchor reads
the tick as its id fallback (§6.2) and the quote as its <code>title</code>:
</p>
<p><a class="cta" href="/docs" id="main" style="--tone: red" title="Docs Title" type="anchorZ">Kept</a>
</p>
<p>A moved position is the author&#39;s own field, so an element does not read it:
<code>moveA</code>&#39;s tick is a plain <code>slug</code> attribute and never an id, while the unclaimed
type still falls back to it:
</p>
<p><a href="/x" id="slug-b" type="anchorZ">Fallback</a>
</p>

</>); }
