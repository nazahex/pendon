import { OrderedDefault } from '@comp/content/List'
import { QuoteA } from '@comp/content/Quote'
import { UnorderedA } from '@comp/content/List'
import { UnorderedDefault } from '@comp/content/List'

export default function PendonView() { return (<>
<h1>List &amp; Blockquote Decorators</h1>
<p>One decorator line, two owners: a line directly above a <strong>blockquote</strong> decorates
the quote (§9.2), a line directly above a <strong>list</strong> decorates the whole list
container (§9.3 L1). This fixture drives both layers end to end.
</p>
<h2>Blockquote</h2>
<p>A decorator line directly above the quote decorates it:
</p>
<QuoteA class={"q"} id={"q1"} type={"bqA"}>a quoted <em>body</em></QuoteA><p>When both a decorator line and an inner head are present the <strong>inner head wins</strong>;
an unclaimed type falls back to the built-in <code>&lt;blockquote&gt;</code> and still keeps every
extra (§9.2, §11 rule 3, D8):
</p>
<blockquote class="z" type="quoteZ">inner head wins</blockquote><p>A quote with neither a head nor a decorator line is left alone:
</p>
<blockquote>a plain quote</blockquote><h2>List container</h2>
<p>A container decorator above a list decorates the <em>container</em>, never the items
(§9.3 L1). The <strong>marker decides the layer</strong> (§9.4), the decorator&#39;s type only
selects the component:
</p>
<UnorderedA class={"u"} id={"l1"} type={"unorderedA"}><li>one</li>
<li>two</li>
</UnorderedA><p>An untyped decorator therefore routes through the marker&#39;s layer default:
</p>
<OrderedDefault class={"o"} start={1}><li>first</li>
<li>second</li>
</OrderedDefault><p>An unclaimed type falls back to the layer default too:
</p>
<UnorderedDefault class={"z"} type={"unorderedZ"}><li>star one</li>
<li>star two</li>
</UnorderedDefault>
</>); }
