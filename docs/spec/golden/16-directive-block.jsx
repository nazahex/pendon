import { DirectiveDefault } from '@comp/content/Directive'
import { Note } from '@comp/content/Directive'

export default function PendonView() { return (<>
<p>A block directive owns its line and its body is parsed as block content (§10.3):
</p>
<Note type={"note"}><h1>A heading inside the directive</h1>
<ul>
<li>the first item</li>
<li>the second item</li>
</ul>
</Note><p>Nested fences close LIFO, so the innermost directive is the first to close
(§10.3):
</p>
<DirectiveDefault type={"outer"}><Note type={"note"}><p>inner body
</p>
</Note><p>trailing text of the outer
</p>
</DirectiveDefault><p>A fence with nothing open, and a fence without a type, stay literal text: there
is no anonymous block directive (§10.3, OPEN-B1):
</p>
<p>==
==5
</p>
<p>An unclosed directive is closed implicitly at the end of the input, with a
warning (§10.3):
</p>
<Note type={"note"}><p>this block never closes
</p>
</Note>
</>); }
