import DocHeading from "@/components/DocHeading";

export const frontmatter = {"Foo":"Bar"};
export const headings = [{"id":"lorem-isum","level":1,"subheadings":[{"id":"foo-bar","level":2,"text":"Foo Bar"},{"id":"bar-foo","level":2,"text":"Bar Foo"}],"text":"Lorem Isum"},{"id":"bar-foo","level":1,"text":"Markdown Comment"},{"id":"hiz","level":1,"text":"Hiz"},{"id":"hiz","level":1,"text":"Bind Comment"}];
export const outline = [{"depth":1,"inner_text":"Lorem Isum","level":"1","raw_title":"Lorem Isum"},{"depth":2,"inner_text":"Foo Bar","level":"2","raw_title":"Foo Bar"},{"depth":2,"inner_text":"Bar Foo","level":"2","raw_title":"Bar Foo"},{"depth":1,"inner_text":"Markdown Comment","level":"1"},{"depth":1,"inner_text":"Hiz","level":"1","raw_title":"Hiz"},{"depth":1,"inner_text":"Bind Comment","level":"1"}];
export default function PendonView() { return (<>
<section>
<p>This fixture pins the automatic <code>headings</code> export (§20) in the <strong>nested</strong> shape the
retired <code>plugin-extract-heading</code> used to inject. <code>renderer-solid</code> now builds that
outline from the AST, so <code>sandbox/heading</code> renders byte-identically without the
plugin. The <code>[[task.heading.extract]]</code> block is the plugin&#39;s replacement: the same
headings as a <strong>flat</strong> array with <code>depth</code>.
</p>
<p>A heading is a heading wherever the lexer finds one — inside a section, inside an
HTML comment, or inside a <code>&#123;&#123;&#123;…&#125;&#125;&#125;</code> block that no <code>bind</code> stage consumed. A code
fence is not:
</p>
</section>
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
