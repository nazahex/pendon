import { QuoteA } from '@comp/Quote'

export default function PendonView() { return (<>
<p>§9.2: a quote has <strong>no head of its own</strong>. The only way to configure one is the
<strong>decorator line</strong> directly above it (§9.1, a line touches the <code>&gt;</code> it decorates):
</p>
<blockquote class="outer">a plain quoted body</blockquote><p>A claimed <code>type</code> renders its component, and every extra lands on the quote node
(§11 rule 3):
</p>
<QuoteA class="q" id="q1" type="quoteA">quoted <em>text</em></QuoteA><p>An unclaimed type falls back to the built-in <code>&lt;blockquote&gt;</code> and still keeps every
extra (§11 rule 3, D8):
</p>
<blockquote class="z" type="quoteZ">an unclaimed type</blockquote><p>Anything written <strong>inside</strong> the <code>&gt;</code> is content, not a head: <code>@@</code> and the braces
are literal text there, and the body keeps the constructs and slots that its own
elements claim (§9.2):
</p>
<blockquote>@@quoteA&#123;.q&#125; this is not a head</blockquote>
</>); }
