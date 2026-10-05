---
title: "Dialog Plugin Demo"
charmap: ["Revan Juan", "a", "Stevano", "b"]
---

## Dialog

This is a naration. Excepteur anim _veniam_ sunt occaecat non enim sunt sunt pariatur aliquip sint. Reprehenderit sunt ut ut duis eu do deserunt tempor consectetur minim cupidatat cupidatat ad.

Revan Juan: "Consectetur eu minim _aute_ deserunt nulla amet elit." _(esse qui cupidatat ex exercitation qui minim tempor anim mollit)_
Stevano: _(dolore eiusmod ad proident)_ "Exercitation anim id culpa ex ea aliquip exercitation velit culpa. Aliquip occaecat ipsum fugiat eu nostrud velit sunt ex cillum."\\...\\"Enim velit anim sunt qui mollit."
Revan Juan: "In cillum cupidatat amet anim occaecat deserunt cupidatat."

Stevano: "Nulla quis duis excepteur sint Lorem dolor anim adipisicing non aliqua consequat sunt irure esse."

Veniam duis adipisicing deserunt Lorem esse occaecat anim sit aliqua Lorem dolore aliquip. Mollit sit nisi culpa minim deserunt voluptate laboris irure veniam exercitation. Eu occaecat consectetur adipisicing deserunt amet cillum ullamco. Ea commodo in reprehenderit adipisicing laboris Lorem consequat cillum aute occaecat ut est sunt. Ullamco sint nulla tempor non non mollit tempor aute. Non occaecat velit et excepteur cupidatat velit sint sit eu consectetur reprehenderit laborum est sint.

Rio: "Sunt exercitation veniam Lorem aliqua dolor esse officia." _(eiusmod minim)_\\"Sint non elit consectetur exercitation."

## Expected Output

```html
<p>This is a naration. Excepteur anim <em>veniam</em> sunt occaecat non enim sunt sunt pariatur aliquip sint. Reprehenderit sunt ut ut duis eu do deserunt tempor consectetur minim cupidatat cupidatat ad.</p>

<dl>
<dt class="a">Revan Juan</dt> <dd class="a"><q>Consectetur eu minim <em>aute</em> deserunt nulla amet elit.</q> <i>(esse qui cupidatat ex exercitation qui minim tempor anim mollit)</i></dd>
<dt class="b">Stevano</dt> <dd class="b"><i>(dolore eiusmod ad proident)</i> <q>Exercitation anim id culpa ex ea aliquip exercitation velit culpa. Aliquip occaecat ipsum fugiat eu nostrud velit sunt ex cillum.</q><br /><p>...</p><br /><q>Enim velit anim sunt qui mollit.</q></dd>
<dt class="a">Revan Juan</dt> <dd class="a"><q>In cillum cupidatat amet anim occaecat deserunt cupidatat.</q></dd>
</dl>


<dl>
<dt class="b">Stevano</dt> <dd class="b"><q>Nulla quis duis excepteur sint Lorem dolor anim adipisicing non aliqua consequat sunt irure esse..</q></dd>
</dl>

<p>Veniam duis adipisicing deserunt Lorem esse occaecat anim sit aliqua Lorem dolore aliquip. Mollit sit nisi culpa minim deserunt voluptate laboris irure veniam exercitation. Eu occaecat consectetur adipisicing deserunt amet cillum ullamco. Ea commodo in reprehenderit adipisicing laboris Lorem consequat cillum aute occaecat ut est sunt. Ullamco sint nulla tempor non non mollit tempor aute. Non occaecat velit et excepteur cupidatat velit sint sit eu consectetur reprehenderit laborum est sint.</p>

<dl>
<dt>Rio</dt> <dd><q>Sunt exercitation veniam Lorem aliqua dolor esse officia.</q> <i>(eiusmod minim)</i><br /><q>Sint non elit consectetur exercitation.</q></dd>
</dl>
```

## Penjalasan

- semua format markdown harus tetap bisa bekerja seperti biasa
- plugin ini hanya override format dialognya saja yang selalu di awali dengan `foo: bar`
- di dalamnya, semua yang digapit `""` dibungkus oleh `<q>` tanpa menambah kutip manual (kutip mengikuti default browser/CSS)
- di dalamnya, semua yang digapit `_()_` atau `*()*` wajib dibungkus oleh `<i>`, bukan `<em>`
- di dalamnya, semua yang tidak digapit apapun dibungkus oleh `<p>`
- di dalamnya, `//` dirender menjadi `<br />`
- 1 line break terpisah = dl baru
- `charmap` didaptkan dari plugin pendon micromatter
- karakter yang ada di dalam daftar charmap mendapatkan class spesial untuk dt dan ddnya.
- jika tidak ada di charmap, maka tidak ada class khusus
