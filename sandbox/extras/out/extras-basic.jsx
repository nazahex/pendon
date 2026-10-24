import DirecA from '@comp/content/DirecA'
import MarkerA from '@comp/content/MarkerA'

export const frontmatter = {"fm1":"This is a frontmatter label"};
export default function PendonView() { return (<>
<h2>Extras Demo</h2>
<MarkerA bags={[{fruits: ["banana", "apple"]}, {vegs: "carrot"}]} content={{original: "Foo bar bur.", text: "Lorem Ipsum"}} labels={frontmatter.fm1} type="markerA" /><div bags="[&#123;&quot;fruits&quot;:[&quot;banana&quot;,&quot;apple&quot;]&#125;,&#123;&quot;vegs&quot;:&quot;carrot&quot;&#125;]" content="&#123;&quot;original&quot;:&quot;Foo bar bur.&quot;,&quot;text&quot;:&quot;Lorem Ipsum&quot;&#125;" labels={frontmatter.fm1} type="untyped"></div><MarkerA jsxBracket={valueA} type="markerA" />{/* plugin-bind */}
<DirecA token={{def: {theme: {accent: "#0af", scale: [1, 2]}}}} type="DirecA" /><DirecA bags={[{fruits: ["banana", "apple"]}, {vegs: "carrot"}]} labels={frontmatter.fm1} token={{content: {original: "Foo bar bur.", text: "Lorem Ipsum"}, def: {theme: {accent: "#0af", scale: [1, 2]}}}} type="DirecA" />
</>); }
