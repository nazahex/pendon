export default function PendonView() { return (<>
<p><code>plugin-bind</code> reads a <code>&#123;&#123;&#123;lang[var] … &#125;&#125;&#125;</code> data block <strong>before</strong> the lexer sees it
(ADR-0004): a payload&#39;s <code>#</code>, <code>-</code> and blank lines would otherwise be read as
markup. The block is removed from the text and every <code>$var</code> in an extras head
binds the parsed value as a real JavaScript prop (RFC §2).
</p>
<p>The whole object becomes one prop:
</p>
<p><a data={{"id":"usr_01","tags":["admin","dev"]}} href="/docs" type="anchorA">Card</a>
</p>
<p>A nested extras value binds a leaf; a block may be declared after its use:
</p>
<p><a href="/docs" meta={{"n":2,"rows":[{"aktif":true,"id":1,"nama":"Budi, S.T."},{"aktif":false,"id":2,"nama":"Siti"}]}} type="anchorA">Rows</a>
</p>
<p><code>jsonc</code> strips <code>//</code> comments, <code>yaml</code> expands merge keys, and <code>toml</code> keeps its
tables:
</p>
<p><a build={{"profile":{"release":true,"targets":["aarch64","x86_64"]}}} env={{"base":{"timeout":30},"dev":{"host":"localhost","timeout":30}}} href="/docs" theme={{"accent":"#0af","scale":[1,2]}} type="anchorA">Typed</a>
</p>
<p>An undefined <code>$ghost</code> warns and stays literal text:
</p>
<p><a data="$ghost" href="/docs" type="anchorA">Ghost</a>
</p>

</>); }
