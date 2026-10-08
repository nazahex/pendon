export default function PendonView() { return (<>
<section>
<p>§9.5: a decorator line above a heading decorates the <strong>section</strong>, not the
heading — the heading already carries its own extras on the <code>#</code> run, so a second
head above it would be redundant:
</p>
</section>
<section id="slug-section" type="sectionA">
<h2 class="lead" title="Head title" type="headingX">Section A</h2>
<p>The section id is the first of <code>#sectionID</code> &gt; <code>slug-section</code> &gt; <code>[slug]</code> &gt; extras
slug &gt; the slug of the title; the heading yields its own id to the section.
</p>
<section>
<p>A <code>&lt;---&gt;</code> marker deepens the outline without waiting for a deeper heading.
</p>
</section>
<p><code>&gt;---&lt;</code> closes the innermost section again.
</p>
</section>
<section id="bare-section-b">
<h2>Bare Section B</h2>
<p>Body of section B.
</p>
</section>

</>); }
