import DocHeading from "@/components/DocHeading";

export const frontmatter = {"Foo":"Bar"};
export const headings = [{"id":"lorem-isum","level":1,"subheadings":[{"id":"foo-bar","level":2,"text":"Foo Bar"},{"id":"bar-foo","level":2,"text":"Bar Foo"}],"text":"Lorem Isum"},{"id":"bar-foo","level":1,"text":"Markdown Comment"},{"id":"hiz","level":1,"text":"Hiz"},{"id":"hiz","level":1,"text":"Bind Comment"}];
export default function PendonView() { return (<>
<section id="lorem-isum">
<DocHeading level={1} number={["1"]} raw_title="Lorem Isum" >Lorem Isum</DocHeading><section id="foo-bar">
<DocHeading level={2} number={["1", "1"]} raw_title="Foo Bar" >Foo Bar</DocHeading></section>
<section id="bar-foo">
<DocHeading level={2} number={["1", "2"]} raw_title="Bar Foo" >Bar Foo</DocHeading><p>&lt;!--
</p>
<h1>Markdown Comment</h1>
<p>--&gt;
</p>
</section>
</section>
<section id="hiz">
<DocHeading level={1} number={["2"]} raw_title="Hiz" >Hiz</DocHeading><p>&#123;&#123;&#123;toml[dataY]
</p>
<h1>Bind Comment</h1>
<p>&#125;&#125;&#125;
</p>
<pre><code>## Codefence Comment
</code></pre>
</section>

</>); }
