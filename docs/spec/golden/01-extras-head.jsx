export default function PendonView() {
  return (
    <>
      <p>
        Every item kind of §5 in one head. <code>class</code> accumulates, <code>id</code>/
        <code>slug</code>/<code>title</code> and every prop is last-wins, and bare flags stay bare
        attributes (§6.3).
      </p>
      <p>
        <a
          bar="12"
          class="extra class"
          foo="bar"
          href="/docs/bar"
          id="id"
          isBar
          isFoo="true"
          style="--style-var: 2rem"
          title="Head title"
          type="anchorA">
          All item kinds
        </a>
      </p>
      <p>The worked example of §5.1 in full:</p>
      <p>
        <a
          bar="12"
          class="extra class"
          foo="bar"
          href="/docs"
          id="id"
          isBar
          isFoo="true"
          style="--style-var: 2rem"
          title="Title Foo"
          type="anchorB">
          Worked
        </a>
      </p>
      <p>
        Duplicates collapse to one attribute: the position of the first occurrence wins and the
        value of the last one does (§6.4).
      </p>
      <p>
        <a class="first second" foo="two" href="/docs" id="other" isFlag other type="anchorB">
          Duplicates
        </a>
      </p>
      <p>Escapes inside values: a comma, a quote and a backslash all survive.</p>
      <p>
        <a csv="a,b" href="/docs" path="dir\file" quote="say &quot;hi&quot;" type="anchorA">
          Escaping
        </a>
      </p>
      <p>
        A <code>class:</code> prop accumulates exactly like a <code>.x</code> item, head items first
        (§6.4).
      </p>
      <p>
        <a class="head tail" href="/docs" type="anchorA">
          Class prop
        </a>
      </p>
    </>
  )
}
