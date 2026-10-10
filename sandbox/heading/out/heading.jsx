import DocHeading from "@/components/DocHeading";

export const frontmatter = {"title":"Heading Demo"};
export const headings = [{"id":"foo-bar","text":"Foo Bar Barosa","level":2},{"id":"sectionIDqux","text":"Qux Nos","level":2},{"id":"lorem-ipsum","text":"Lorem","level":2,"subheadings":[{"id":"ipsum","text":"Ipsum","level":3,"subheadings":[{"id":"slug-head","text":"Extras-routed heading","level":4}]},{"id":"heading-y","text":"Routed by the second marker","level":3}]},{"id":"unrouted","text":"An unclaimed marker keeps the layer default","level":2},{"id":"foo-poo","text":"Foo Poo","level":2}];
export default function PendonView() { return (<>
<section>
<p>Sunt quis labore dolor quis pariatur consectetur mollit Lorem sunt nisi do officia.
</p>
</section>
<section id="foo-bar">
<DocHeading class={"extra"} level={2} number={1} qux={"anu"} raw_title={"Foo Bar Barosa"} type={"heading"} >Foo Bar Barosa
</DocHeading><p>Cupidatat velit esse dolore pariatur dolor cillum quis tempor cillum qui.
</p>
</section>
<section id="sectionIDqux" rox="12">
<DocHeading level={2} nor={32} number={2} raw_title={"Qux Nos"} type={"heading"} >Qux Nos
</DocHeading><p>Aliquip sint ex tempor ipsum ut minim aliqua consectetur pariatur cillum.
</p>
</section>
<section id="lorem-ipsum">
<DocHeading level={2} number={3} raw_title={"Lorem"} >Lorem
</DocHeading><section id="ipsum">
<DocHeading level={3} number={"3.1."} raw_title={"Ipsum"} title={"Ipsah"} >Ipsum
</DocHeading><p><code>[slug]</code>, <code>(&quot;title&quot;)</code> and <code>@@type&#123;…&#125;</code> are all optional and independent (§7.4).
</p>
<hr />
<section id="slug-head" type="sectionX">
<DocHeading class={"fancy"} level={4} number={"3.1.1."} raw_title={"Extras-routed heading"} title={"Heading X"} type={"headingX"} >Extras-routed heading
</DocHeading><p>Magna qui elit commodo cupidatat amet et ipsum enim deserunt excepteur aute officia.
</p>
<section>
<p>Sit sit in ea ad eu.
</p>
<section>
<p>Voluptate quis irure et officia ipsum enim adipisicing aute elit.
</p>
<p>Cupidatat incididunt dolor eu excepteur est ut est esse.
</p>
<p>Est ad sunt qui aliquip pariatur aliqua est minim occaecat amet.
</p>
<p>&lt;!-- Velit consequat sit ullamco consectetur incididunt consectetur fugiat enim dolore. --&gt;
</p>
<hr />
</section>
</section>
</section>
</section>
<section id="heading-y">
<DocHeading level={3} number={"auto"} raw_title={"Routed by the second marker"} title={"Heading Y"} type={"headingY"} >Routed by the second marker
</DocHeading></section>
</section>
<section class="x spaarted-by-line-break" id="unrouted" type="sectionZ">
<DocHeading class={"x"} level={2} number={4} raw_title={"An unclaimed marker keeps the layer default"} type={"unknownZ"} >An unclaimed marker keeps the layer default
</DocHeading><p>Proident ipsum id labore duis magna commodo et enim reprehenderit duis sit nostrud.
</p>
</section>
<section>
<p>Id sunt id nisi occaecat.
</p>
<section>
</section>
<p>Est deserunt ipsum cupidatat non cupidatat ipsum magna reprehenderit culpa.
</p>
</section>
<section id="foo-poo">
<DocHeading level={2} number={5} raw_title={"Foo Poo"} >Foo Poo
</DocHeading><p>In ad aliquip ex ut eiusmod velit tempor sit ut elit nisi.
</p>
</section>

</>); }
