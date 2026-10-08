import { DirectiveDefault } from '@comp/content/Directive'
import { Note } from '@comp/content/Directive'

export default function PendonView() { return (<>
<p>Inline directives keep their paragraph (§10.2) and their content is inline
content. The head <code>[bracket]</code> / <code>(&quot;paren&quot;)</code> slots map to <code>id</code> / <code>level</code>, and the
backticked / quoted extras items map to <code>author</code> / <code>summary</code> (§6.1, §10.2, §10.4):
</p>
<p><Note author={"Jane Doe"} class={"box"} id={"note-1"} isOpen={true} level={"A level"} summary={"A summary"} type={"note"}> some <em>inline</em> content</Note> and the
text after it stays inline.
</p>
<p>An unclaimed type routes to the layer default (§11 rule 3):
</p>
<p><DirectiveDefault type={"aside"}> this one is not typed</DirectiveDefault> but it still becomes a node.
</p>
<p>The closing run must be at least as long as the opening run, so a directive nests
and an inner <code>::</code> closes before the outer one (§10.2):
</p>
<p><DirectiveDefault type={"outer"}> a <Note type={"note"}> b</Note> c</DirectiveDefault> trailing text stays outside.
</p>
<p>A bare <code>::</code> and an unterminated opener stay literal text (§4.3):
</p>
<p>:: alone, and ::note no close here.
</p>

</>); }
