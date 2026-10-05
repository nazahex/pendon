export default function PendonView() {
  return (
    <>
      <p>The construct head wins over the extras; the extras value is dropped (§6.2).</p>
      <p>
        <a href="/docs" title="Head title" type="anchorA">
          Head title wins
        </a>
      </p>
      <p>
        <code>#id</code> beats the extras slug, and an extras slug becomes the id when nothing else
        did (§6.2).
      </p>
      <p>
        <a href="/docs" id="explicit" type="anchorA">
          Explicit id
        </a>
      </p>
      <p>
        <a href="/docs" id="slug-only" type="anchorA">
          Slug only
        </a>
      </p>
      <p>
        <code>href</code> is owned by the link; the extras value is ignored (§7.2).
      </p>
      <p>
        <a href="/docs" type="anchorA">
          Href is owned
        </a>
      </p>
      <p>
        URL modifiers are parsed before the head and win over <code>target:</code>:
      </p>
      <p>
        <a href="/docs" rel="noopener" target="_blank" type="anchorA">
          Modifier wins
        </a>
      </p>
      <p>
        <a href="https://example.com/bar" rel="noopener" target="_self" type="anchorA">
          No modifier
        </a>
      </p>
      <p>
        <code>rel:</code> merges with the modifier result instead of replacing it:
      </p>
      <p>
        <a
          href="https://example.com/bar"
          hreflang="en"
          rel="noopener nofollow prefetch"
          target="_blank"
          type="anchorA">
          Rel merge
        </a>
      </p>
      <p>
        A marker with no configured entry keeps the built-in <code>&lt;a&gt;</code> element and
        carries the marker as its <code>type</code> attribute (§11 rule 3):
      </p>
      <p>
        <a class="hero" href="/docs" type="anchorZ">
          Unclaimed marker
        </a>
      </p>
      <p>
        <a href="/docs">No marker at all</a>
      </p>
    </>
  )
}
