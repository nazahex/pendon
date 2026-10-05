## Highlight Bagian Kode (Inline/Range)

Contoh sintaks:

```html .foo .boz "<small>" "indah" "b/>"
<p>Lorem consequat <b>mollit commodo</b> cillum <small>labore</small> indah ut ex nisi excepteur ad nostrud quis dolore.</p>
```

Rekomendasi tag: `<strong>`

## Highlight Baris

### Plain

Contoh sintaks:

```js .wrap {1} {3-5}
import { log } from "console"; 
const data = [10, 20, null];
async function* hitung(a, b = 5) {
  for (let x of data) {
    if (x?.val ?? true) yield (a + b) * x;
  }
}
hitung(2).next().then(({ value }) => log(`Hasil: ${value}`));
```

Rekomendasi tag: `<p class="mark">`
Catatan: 1-index based

### Insert

Contoh sintaks:

```js ins={1} ins={3-5} .bar
import { log } from "console"; 
const data = [10, 20, null];
async function* hitung(a, b = 5) {
  for (let x of data) {
    if (x?.val ?? true) yield (a + b) * x;
  }
}
hitung(2).next().then(({ value }) => log(`Hasil: ${value}`));
```

Rekomendasi tag: `<p class="ins">`

### Delete

Contoh sintaks:

```js del={1} .dux del={3-5}
import { log } from "console"; 
const data = [10, 20, null];
async function* hitung(a, b = 5) {
  for (let x of data) {
    if (x?.val ?? true) yield (a + b) * x;
  }
}
hitung(2).next().then(({ value }) => log(`Hasil: ${value}`));
```

Rekomendasi tag: `<p class="del">`

## Kombinasi

Contoh sintaks:

```js "log" .bar "unction*" ins={1} del={3-4}
import { log } from "console"; 
const data = [10, 20, null];
async function* hitung(a, b = 5) {
  for (let x of data) {
    if (x?.val ?? true) yield (a + b) * x;
  }
}
hitung(2).next().then(({ value }) => log(`Hasil: ${value}`));
```
