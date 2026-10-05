# Pendon Plugin Wiki

An adapted Wiki syntaxes for Pendon.

## WikiLink

Syntax:

```
Nisi [[Anim Esta]] id id est officia.

Nisi [[Anim Esta (Officia) | Anim]] id id est officia.

Nisi [[anim esta]] id id est officia.
```

Rendered as:

```html
<p>Nisi <a href="/Anim_Esta" title="Anim Esta">Anim Esta</a> id id est officia.</p>

<p>Nisi <a href="/Anim_Esta_(Officia)" title="Anim Esta (Officia)">Anim</a> id id est officia.</p>

<p>Nisi <a href="/Anim_esta" title="Anim esta">anim esta</a> id id est officia.</p>
```

## Infobox

:::infobox[rox]

## Some Title

::[.img]
![Some Image](https://upload.wikimedia.org/wikipedia/en/b/bc/Wiki.png)
::

Foo = Baz
Bar Qux = Fugiat ex **exercitation** aute nisi _incididunt_ in et veniam ex id occaecat ex duis fugiat.

::[.hox,.nox]
Est nisi _culpa_ commodo cillum incididunt elit cupidatat consequat nulla dolore velit.
::

### Sub Heading

Cillum =[.cill,.elit] Elit
Lorem =[.foo] Nisi veniam
Nulla = Dolore

::
Id amet cillum cupidatat [[nostrud]] nostrud ea pariatur exercitation laborum.
::

:::

Rendered as:

```html
<aside class="infobox rox">
  <dl>
    <dt class="full h2">Some Title</dt>
    <dd class="full img"><img alt="Some Image" src="https://upload.wikimedia.org/wikipedia/en/b/bc/Wiki.png" /></dd>
    <dt>Foo</dt>
    <dd>Baz</dd>
    <dt>Bar Qux</dt>
    <dd>Fugiat ex <strong>exercitation</strong> aute nisi <em>incididunt</em> in et veniam ex id occaecat ex duis fugiat.</dd>
    <dd class="full hox nox">Est nisi <em>culpa</em> commodo cillum incididunt elit cupidatat consequat nulla dolore velit.</dd>
    <dt class="full h3">Sub Heading</dt>
    <dd class="cill elit">Elit</dd>
    <dt class="cill elit">Cillum</dt>
    <dt class="foo">Lorem</dt>
    <dd class="foo">Nisi veniam</dd>
    <dt>Nulla</dt>
    <dd>Dolore</dd>
    <dd class="full">Id amet cillum cupidatat <a href="Nostrud" title="Nostrud">nostrud</a> nostrud ea pariatur exercitation laborum.</dd>
  </dl>
</aside>
```

## Terms

- This plugin should work well with vanila Pendon and other plugins. Anything inside `dl > dt` and `dl > dd` must be able to be processed by vanila and another plugin.
