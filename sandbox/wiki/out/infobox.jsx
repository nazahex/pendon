export default function PendonView() {
  return (
    <>
      <h2>Wiki Infobox Demo</h2>
      <aside class="infobox rox">
        <dl>
          <dt class="full h2">Some Title</dt>
          <dd class="full img">
            <img alt="Some Image" src="https://upload.wikimedia.org/wikipedia/en/b/bc/Wiki.png" />
          </dd>
          <dt>Foo</dt>
          <dd>
            <a href="/id/wiki/Baz" title="Baz">
              Baz
            </a>
          </dd>
          <dt>Bar Qux</dt>
          <dd>
            Fugiat ex <strong>exercitation</strong> aute nisi <em>incididunt</em> in et veniam ex id
            occaecat ex duis fugiat.
          </dd>
          <dd class="full hox nox">
            Est nisi <em>culpa</em> commodo cillum incididunt elit cupidatat consequat nulla dolore
            velit.
          </dd>
          <dt class="full h3">Sub Heading</dt>
          <dt class="cill elit">Cillum</dt>
          <dd class="cill elit">Elit</dd>
          <dt class="foo">Lorem</dt>
          <dd class="foo">Nisi veniam</dd>
          <dt>Nulla</dt>
          <dd>Dolore</dd>
          <dd class="full">
            Id amet cillum cupidatat{" "}
            <a href="/id/wiki/Nostrud" title="Nostrud">
              nostrud
            </a>{" "}
            nostrud ea pariatur exercitation laborum.
          </dd>
        </dl>
      </aside>
    </>
  )
}
