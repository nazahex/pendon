export const headings = [{"id":"slug-section","level":1,"subheadings":[{"id":"bare-section-b","level":2,"text":"Bare Section B"}],"text":"Section A"}];
export default function PendonView() { return (<>
<section depth="0">
<p>§9.5: a decorator line above a heading decorates the <strong>section</strong>, not the
heading — the heading already carries its own extras on the <code>#</code> run, so a second
head above it would be redundant:
</p>
</section>
<section depth="2" id="slug-section" type="sectionA">
<h1 class="lead" title="Head title" type="headingX">Section A</h1>
<p>The section id is the first of <code>#sectionID</code> &gt; <code>slug-section</code> &gt; <code>[slug]</code> &gt; extras
slug &gt; the slug of the title; the heading yields its own id to the section.
</p>
<section depth="3">
<p>A <code>&lt;---&gt;</code> marker deepens the outline without waiting for a deeper heading.
</p>
</section>
<p><code>&gt;---&lt;</code> closes the innermost section again.
</p>
</section>
<section depth="2" id="bare-section-b">
<h2>Bare Section B</h2>
<p>Body of section B.
</p>
</section>

</>); }
