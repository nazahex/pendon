import { HeadingFallback } from '@comp/Fallback'
import { HeadingTypeA } from '@comp/SpecialA'
import { HeadingUntyped } from '@comp/Untyped'

export default function PendonView() { return (<>
<p><code>type</code> in a <code>[task.&lt;layer&gt;.custom]</code> entry is a <strong>selector</strong>, and the renderer
resolves every layer <strong>once per document</strong> against the types the document
actually occurs (<code>SYNTAX.md</code> §11, ADR-0008 <code>D-3a</code>). This fixture reproduces
<code>docs/issues/type-upgrade.md</code> R.1–R.4 verbatim:
</p>
<ul>
<li>R.1 <code>type = [&quot;*&quot;]</code> — every occurring type, specificity 1;</li>
<li>R.2 <code>type = [&quot;TypeA&quot;]</code> — an exact literal, specificity 2;</li>
<li>R.3 <code>type = [&quot;_&quot;]</code> — <em>the remainder</em>: every occurring type that no literal or <code>?</code> entry of the layer defines, specificity 1;</li>
<li>R.4 <code>type = &quot;?&quot;</code> — the untyped nodes, specificity 2;</li>
<li><code>defined</code> is computed from <strong>all</strong> entries of the layer up front, so R.4&#39;s <code>?</code> removes untyped from R.3 even though R.4 is written after it, and R.1&#39;s <code>*</code> defines nothing.</li>
</ul>
<HeadingTypeA modern variant id="heading-a" level={3} raw_title="`TypeA` — the literal (R.2) beats `*` (R.1)" type="TypeA"><code>TypeA</code> — the literal (R.2) beats <code>*</code> (R.1)</HeadingTypeA><HeadingFallback dark theme id="heading-b" level={3} raw_title="`TypeB` — no entry names it, so it lands in `_` (R.3)" type="TypeB"><code>TypeB</code> — no entry names it, so it lands in <code>_</code> (R.3)</HeadingFallback><HeadingFallback dark theme id="heading-c" level={3} raw_title="`TypeC` — also `_`; R.3 is later than R.1 at equal specificity" type="TypeC"><code>TypeC</code> — also <code>_</code>; R.3 is later than R.1 at equal specificity</HeadingFallback><HeadingFallback dark theme id="heading-d" level={3} raw_title="`TypeD` — a type nobody wrote down still occurs, so it is `_` too" type="TypeD"><code>TypeD</code> — a type nobody wrote down still occurs, so it is <code>_</code> too</HeadingFallback><HeadingUntyped strictMode="{true}" level={2} raw_title="A heading with no `@@` head is untyped" slug="a-heading-with-no-head-is-untyped">A heading with no <code>@@</code> head is untyped</HeadingUntyped><p>It is the <code>?</code> of R.4 — and because <code>?</code> <strong>defines</strong> untyped, the <code>_</code> of R.3 never
sees it.
</p>
<HeadingUntyped strictMode="{true}" level={2} raw_title="`*` (R.1) never wins here" slug="r-1-never-wins-here"><code>*</code> (R.1) never wins here</HeadingUntyped><p>Every occurring type is claimed by R.2, R.3 or R.4, so <code>HeadingDefault</code> is never
used and its import never appears in the output — that is the point of the
fixture.
</p>

</>); }
