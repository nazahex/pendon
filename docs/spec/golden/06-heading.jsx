import { HeadingDefault } from '@comp/Heading'
import { HeadingXY } from '@comp/Heading'

export const headings = [{"id":"slug-head","level":2,"text":"Extras-routed heading"},{"id":"heading-y","level":2,"text":"Routed by the second marker"},{"id":"unrouted","level":2,"text":"An unclaimed marker keeps the layer default"},{"id":"a-plain-heading-without-extras","level":2,"text":"A plain heading without extras"}];
export default function PendonView() { return (<>
<p><code>[slug]</code>, <code>(&quot;title&quot;)</code> and <code>@@type&#123;…&#125;</code> are all optional and independent (§7.4).
</p>
<HeadingXY depth={"2"} type={"headingX"} class="fancy" id="slug-head" slug="slug-head" title="Heading X">Extras-routed heading</HeadingXY><HeadingXY depth={"2"} type={"headingY"} number={"auto"} id="heading-y" slug="heading-y" title="Heading Y">Routed by the second marker</HeadingXY><HeadingDefault class="x" depth={2} id="unrouted" slug="unrouted" type="unknownZ">An unclaimed marker keeps the layer default</HeadingDefault><HeadingDefault depth={2} id="a-plain-heading-without-extras" slug="a-plain-heading-without-extras">A plain heading without extras</HeadingDefault>
</>); }
