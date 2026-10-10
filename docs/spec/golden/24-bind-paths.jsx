export default function PendonView() { return (<>
<p><code>plugin-bind</code> reaches <strong>one leaf</strong> of a bound value with a path
(<code>$config.db.host</code>, <code>$rows[0].nama</code>, <code>$config.limits[&quot;max-rows&quot;]</code>), merges a bound
object into a nested extras value with a <strong>spread</strong> item, and reads a payload from
an <strong>external file</strong> — RFC §2.5–§2.7.
</p>
<p>This fixture lives at <code>src/24-bind-paths.md</code> and its file payload at
<code>../data/rows.csv</code>, so resolving a relative path against the <strong>source file&#39;s</strong> own
directory (never the process CWD) is part of the contract, as is recording that
file as a cache dependency.
</p>
<p>A file payload binds exactly like a body payload:
</p>
<p>A path reaches one leaf. A miss stays literal with a warning, and a price is never
a reference at all — a <code>var</code> starts with a letter:
</p>
<p><a db="localhost" first="Budi, S.T." href="/docs" max={25} miss="$rows[9].nama" port={5432} price="$100" type="anchorA">Card</a>
</p>
<p>A spread supplies defaults, and the key written in the head wins over it:
</p>
<p><a href="/docs" theme={{"accent":"#0af","mode":"dark","scale":1.2}} type="anchorA">Theme</a>
</p>
<p>A spread item is <strong>copied, never walked</strong>: a <code>$…</code>-looking string inside the bound
object stays exactly as it is.
</p>
<p><a copy={{"n":2,"title":"a $brand string is data"}} href="/docs" type="anchorA">Copy</a>
</p>

</>); }
