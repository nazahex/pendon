import { Note } from '@comp/Marker'

export const frontmatter = {"references":{"book":{"id":"book","title":"A Book","type":"book"}}};
export const references = {"book":{"id":"book","title":"A Book","type":"book"}};
export default function PendonView() { return (<>
<p>A field&#39;s value can be a <strong>document</strong> instead of text (§6.7). <code>;;…;;</code> is the
inline form: the payload is parsed and run through the page&#39;s own chain, and the
field carries the node list that run produced — the same fragment a
<code>&#123;&#123;&#123;mdp[…] …&#125;&#125;&#125;</code> binding writes, so no renderer had to learn a second shape.
</p>
<p>A marker&#39;s extras head is a field position like any other, and the citation
inside it is the page&#39;s citation — one list, one numbering:
</p>
<Note note={(<>lorem <em>ipsum</em>, a <a href="/Wiki" title="Wiki">Wiki</a> link and <sup><a href="#1-book" id="cite-1">[1]</a></sup></>)} type="note"></Note><p>An element&#39;s own positions take one too. <code>note</code> is preserved verbatim here, so
the value reaches the built-in <code>&lt;a&gt;</code> as a prop:
</p>
<p><a href="/docs" note={(<>Again <sup><a href="#1-book" id="cite-2" data-bracket-quote="p. 2">[1]</a></sup>.</>)} title="Docs" type="anchorA">Card</a>
</p>
<p>The second citation of <code>book</code> keeps the number the first one was given, because
<code>citation_number</code> is an identity ordinal, not a counter of occurrences:
</p>
<p><a body={(<><em>value</em> in a spec position</>)} href="/docs" type="anchorA">Slot</a>
</p>

</>); }
