import { QuoteA } from '@comp/Quote'

export default function PendonView() { return (<>
<p>§9.1/D-10: a decorator line reads its head in exactly one order — <code>@@type</code> →
slot → <code>&#123;specs&#125;</code> → <code>%%numbering</code> — and every part lands on the block it
decorates:
</p>
<QuoteA {...{ "bracket.bare": "intro" }} class="q" id="q1" k="v" {...{ "number.depth": 2 }} {...{ "number.raw": ["A", "A"] }} {...{ "number.type": "numberA" }} {...{ "paren.bare": "Aside title" }} type="quoteA">a quote configured by one decorator line</QuoteA><p>The <code>%%</code> run carries its own depth (<code>%%%</code> = depth 2), so the marker names a
<code>[[task.number.custom]]</code> counter rather than the element&#39;s (§10.5).
</p>
<p>The sigil, not the type, is what makes a line a decorator: the untyped form
carries a slot and a numbering marker just the same:
</p>
<blockquote {...{ "bracket.bare": "intro2" }} {...{ "number.depth": 1 }} {...{ "number.raw": ["B"] }} {...{ "number.type": "numberA" }}>a quote decorated without a type</blockquote>
</>); }
