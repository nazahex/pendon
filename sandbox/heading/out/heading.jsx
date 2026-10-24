import DocHeading from "@/components/DocHeading";

export const frontmatter = {"Foo":"Bar"};
export const headings = [{"id":"lorem-isum","text":"Lorem Isum","level":1,"subheadings":[{"id":"foo-bar","text":"Foo Bar","level":2},{"id":"bar-foo","text":"Bar Foo","level":2}]},{"id":"bar-foo","text":"Markdown Comment","level":1},{"id":"hiz","text":"Hiz","level":1},{"id":"hiz","text":"Bind Comment","level":1}];
export default function PendonView() { return (<>
<section id="lorem-isum">
<DocHeading level={1} number={1} raw_title={"Lorem Isum"} >Lorem Isum
</DocHeading><section id="foo-bar">
<DocHeading level={2} number={"1.1."} raw_title={"Foo Bar"} >Foo Bar
</DocHeading></section>
<section id="bar-foo">
<DocHeading level={2} number={"1.2."} raw_title={"Bar Foo"} >Bar Foo
</DocHeading><p>&lt;!--
</p>
<h1>Markdown Comment</h1>
<p>--&gt;
</p>
</section>
</section>
<section id="hiz">
<DocHeading level={1} number={2} raw_title={"Hiz"} >Hiz
</DocHeading><p>&#123;&#123;&#123;toml[dataY]
</p>
<h1>Bind Comment</h1>
<p>&#125;&#125;&#125;
</p>
<pre><code>## Codefence Comment
</code></pre>
</section>

</>); }
