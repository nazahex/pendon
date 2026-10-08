import { OrderedDefault } from '@comp/content/List'
import { UnorderedA } from '@comp/content/List'
import { UnorderedDefault } from '@comp/content/List'

export default function PendonView() { return (<>
<p>A container decorator above a list decorates the <em>container</em>, not the items
(§9.3 L1); the decorator line touches the list it decorates and a typed head
routes through the marker&#39;s layer (§9.4):
</p>
<UnorderedA class={"u"} id={"l03"} type={"unorderedA"}><li>one</li>
<li>two</li>
</UnorderedA><p>The <strong>marker</strong> decides the layer, never the decorator&#39;s own type, and an
unclaimed type falls back to the layer default (§9.4, §11 rule 3):
</p>
<OrderedDefault class={"o"} start={1}><li>first</li>
<li>second</li>
</OrderedDefault><UnorderedDefault class={"z"} type={"unorderedZ"}><li>star one</li>
<li>star two</li>
</UnorderedDefault>
</>); }
