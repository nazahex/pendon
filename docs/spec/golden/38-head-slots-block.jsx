export const headings = [{"id":"heading","level":1,"text":"Heading text"}];
export default function PendonView() { return (<>
<p>§6.5/D-7: the block-level heads read whole too. A heading presets
<code>bracket.bare → slug</code> and <code>paren.quote → title</code>; its depth is the <code>#</code> run, not a
position (D-11), and the four contents left over are fields:
</p>
<h1 {...{ "bracket.quote": "heading quote" }} {...{ "bracket.tick": "heading tick" }} id="heading" {...{ "paren.bare": "hp" }} {...{ "paren.tick": "heading paren tick" }} title="heading paren quote">Heading text</h1>
<p>A head alone on its line is a <strong>block image</strong> (§7.1): the <code>&lt;img&gt;</code> replaces the
paragraph, and the six positions are the inline form&#39;s six:
</p>
<img alt="block alt" {...{ "bracket.quote": "block quote" }} {...{ "bracket.tick": "block tick" }} {...{ "paren.quote": "block paren quote" }} {...{ "paren.tick": "block paren tick" }} src="block.png" />
<p>The block form drops nothing either — <code>bracket.bare</code> and <code>paren.bare</code> are the
preset&#39;s two rows, and the other four are the <code>&lt;img&gt;</code>&#39;s own fields (§7.1).
</p>

</>); }
