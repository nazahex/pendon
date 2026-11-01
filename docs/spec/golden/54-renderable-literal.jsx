export default function PendonView() { return (<>
<p>A renderable is a value, and a value is only a value where the grammar says so.
<code>;;</code> in prose is punctuation — a pair of semicolons, nothing more:
</p>
<p>Prose keeps its ;; literal: no field was written here, so no field is read.
</p>
<p>A head is all-or-nothing (§4.3), so a span that never closes leaves the whole
head literal instead of swallowing the rest of the line:
</p>
<p><a href="/docs">Card</a>@@anchorA&#123;note: ;;never closed&#125;
</p>
<p><code>.class</code> and <code>#id</code> have a <strong>name</strong> grammar, not a value grammar (D-RF-6): <code>;</code> is
not a name character, so a head that writes one there is malformed and stays
text — the field is not silently dropped:
</p>
<p><a href="/docs">Card</a>@@anchorA&#123;.c;;x;;, note: text&#125;
</p>
<p>Quoting never decides a value&#39;s meaning, but a span must be the value&#39;s
<strong>whole body</strong>: <code>&quot;pre ;;a;; post&quot;</code> is text, while <code>&quot;;;a *payload*;;&quot;</code> is a payload:
</p>
<p><a body={(<>a <em>payload</em></>)} href="/docs" note="pre ;;a;; post" type="anchorA">Card</a>
</p>
<p><code>\</code> escapes the next character, and one level is all there is — <code>\;</code> is a
semicolon, so two escaped semicolons are payload text, and <code>\\</code> is a backslash:
</p>
<p><a href="/docs" note={(<>a ; semicolon and a \ backslash</>)} type="anchorA">Card</a>
</p>

</>); }
