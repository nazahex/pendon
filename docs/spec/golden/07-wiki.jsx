export default function PendonView() {
  return (
    <>
      <p>
        <code>href</code> is produced by the plugin and is never overridable by extras (§7.5):
      </p>
      <p>
        <a
          class="link"
          href="/id/wiki/Anim_Esta_(Officia)"
          title="Anim Esta (Officia)"
          type="wikiX">
          Anim
        </a>
      </p>
      <p>
        <a href="/id/wiki/Wireless" title="Wireless">
          Wireless
        </a>{" "}
        and{" "}
        <a href="/id/wiki/Plain" title="Plain">
          Plain
        </a>{" "}
        follow one another without extras.
      </p>
      <p>
        An unclaimed marker keeps the built-in <code>&lt;a&gt;</code> element and carries the marker
        as its <code>type</code> attribute:
      </p>
      <p>
        <a class="hero" href="/id/wiki/Fallback" title="Fallback" type="wikiZ">
          Fallback
        </a>
      </p>
    </>
  )
}
