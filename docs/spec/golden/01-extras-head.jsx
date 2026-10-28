export default function PendonView() { return (<>
<p>Every item kind of §5 in one head. <code>class</code> accumulates, and bare flags stay bare
attributes (§6.3).
</p>
<p><a bar="12" class="extra class" foo="bar" href="/docs/bar" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Head title" type="anchorA">All item kinds</a>
</p>
<p>The worked example of §5.1 in full:
</p>
<p><a bar="12" class="extra class" foo="bar" href="/docs" id="id" isBar isFoo="true" style="--style-var: 2rem" title="Title Foo" type="anchorB">Worked</a>
</p>
<p>Duplicates are an error, not a merge: the first occurrence keeps both its
position and its value (§6.4, D-12).
</p>
<p><a class="first second" foo="one" href="/docs" id="dup" isFlag other type="anchorB">Duplicates</a>
</p>
<p>The same holds for the positional items, whatever the spelling.
</p>
<p><a href="/docs" id="one" title="a" type="anchorA">Dup positional</a>
</p>
<p>Escapes inside values: a comma, a quote and a backslash all survive.
</p>
<p><a csv="a,b" href="/docs" path="dir\file" quote="say &quot;hi&quot;" type="anchorA">Escaping</a>
</p>
<p>A <code>class:</code> prop accumulates exactly like a <code>.x</code> item, head items first (§6.4).
</p>
<p><a class="head tail" href="/docs" type="anchorA">Class prop</a>
</p>

</>); }
