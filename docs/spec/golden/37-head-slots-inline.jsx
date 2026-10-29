export const frontmatter = {"references":{"book":{"id":"book","title":"A Book"}}};
export const references = {"book":{"id":"book","title":"A Book"}};
export const citations = [{"citation_number":1,"locator":"p. 12","reference_id":"book"}];
export default function PendonView() { return (<>
<p>§6.5/D-7: a claimed head is read <strong>whole</strong>. The preset names some of its
positions, and every other content becomes a bare field under its slot-qualified
name — so a head cannot lose a value for want of a preset row.
</p>
<p>The anchor presets <code>bracket.bare → inner_text</code>, <code>paren.bare → href</code> and
<code>paren.quote → title</code>; its other three positions keep their names:
</p>
<p>An <a {...{ "bracket.quote": "anchor quote" }} {...{ "bracket.tick": "anchor tick" }} href="/u" {...{ "paren.tick": "anchor paren tick" }} title="anchor title">anchor</a> inline.
</p>
<p>The image presets <code>bracket.bare → alt</code> and <code>paren.bare → src</code>. Its quote is <strong>not</strong>
the <code>&lt;img&gt;</code> title — that is the <em>extras</em> <code>&quot;quote&quot;</code> (§6.3) — so all four of the
remaining positions stay fields of their own:
</p>
<p>Mid-sentence <img alt="an image" {...{ "bracket.quote": "image quote" }} {...{ "bracket.tick": "image tick" }} {...{ "paren.quote": "image paren quote" }} {...{ "paren.tick": "image paren tick" }} src="img.png" /> stays inline.
</p>
<p>The citation presets <code>bracket.bare → reference_id</code> and <code>bracket.quote → locator</code>.
The built-in <code>&lt;sup&gt;</code> has no prop for the other four, so it writes them under the
<code>data-</code> prefix — <code>bracket.tick</code> becomes <code>data-bracket-tick</code>, the spelling a dotted
path takes when it cannot be a prop (§6.3, §7.3):
</p>
<p>A <sup class="cite-ref"><a href="#citeref-1-book" id="cite-1" data-bracket-tick="cite tick" data-paren-bare="paperback" data-paren-quote="cite paren quote" data-paren-tick="cite paren tick">[1]</a></sup> citation.
</p>
<p>None of the above is a diagnostic: an unpreset position is a field, not a mistake.
</p>

</>); }
