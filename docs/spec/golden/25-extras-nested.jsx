import MarkerA from '@comp/MarkerA'

export const frontmatter = {"fm1":"This is a frontmatter label"};
export const headings = [{"id":"nested-extras","level":2,"text":"Nested extras"}];
export default function PendonView() { return (<>
<h2>Nested extras</h2>
<MarkerA bags={[{fruits: ["banana", "apple"]}, {vegs: "carrot"}]} content={{original: "Foo bar bur.", text: "Lorem Ipsum"}} labels={frontmatter.fm1} type="markerA" /><MarkerA jsxBracket={valueA} type="markerA" /><div token={{def: {theme: {accent: "#0af", scale: [1, 2]}}}} type="DirecA"><p>Lorem ipsum.
</p>
</div><div bags="[&#123;&quot;fruits&quot;:[&quot;banana&quot;,&quot;apple&quot;]&#125;,&#123;&quot;vegs&quot;:&quot;carrot&quot;&#125;]" content="&#123;&quot;original&quot;:&quot;Foo bar bur.&quot;,&quot;text&quot;:&quot;Lorem Ipsum&quot;&#125;" labels={frontmatter.fm1} token={{def: {theme: {accent: "#0af", scale: [1, 2]}}}} type="DirecA"><p>Lorem ipsum.
</p>
</div>
</>); }
