import DocHeading from "@/components/DocHeading";

export const headings = [{"id":"foo-bar","text":"Foo Bar Barosa","level":2},{"id":"qux","text":"Qux Nos","level":2},{"id":"lorem-ipsum","text":"Lorem","level":2,"subheadings":[{"id":"ipsum","text":"Ipsum","level":3}]}];
export default function PendonView() { return (<>
<section id="foo-bar">
<DocHeading level={2} id="foo-bar" class="extra" number="1." qux={anu} nor={} >Foo Bar Barosa</DocHeading><p>Cupidatat velit esse dolore pariatur dolor cillum quis tempor cillum qui.
</p>
</section>
<section id="qux">
<DocHeading level={2} id="qux" class="" number="2." qux={} nor={32} >Qux Nos</DocHeading><p>Aliquip sint ex tempor ipsum ut minim aliqua consectetur pariatur cillum.
</p>
</section>
<section id="lorem-ipsum">
<DocHeading level={2} id="lorem-ipsum" class="" number="3." qux={} nor={} >Lorem</DocHeading><section id="ipsum">
<DocHeading level={3} id="ipsum" class="" number="3.1." qux={} nor={} >Ipsum</DocHeading></section>
</section>

</>); }
