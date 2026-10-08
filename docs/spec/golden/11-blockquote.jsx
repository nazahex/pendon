import { QuoteA } from '@comp/content/Quote'

export default function PendonView() { return (<>
<p>§9.2: a quote carries its extras through an <strong>inner head</strong>, right after the <code>&gt;</code>
marker (a space between marker, head and content is allowed, §4.1):
</p>
<QuoteA class={"q"} id={"q1"} type={"quoteA"}>quoted <em>text</em></QuoteA><p>§9.2/§9.1: a <strong>decorator line</strong> directly above the quote decorates it too (the
line touches the <code>&gt;</code> it decorates, §9.1):
</p>
<blockquote class="outer">a plain quoted body</blockquote><p>§9.2: when both exist the inner head wins; an unclaimed type falls back to the
built-in <code>&lt;blockquote&gt;</code> and still keeps every extra (§11 rule 3, D8):
</p>
<blockquote class="z" type="quoteZ">inner wins</blockquote>
</>); }
