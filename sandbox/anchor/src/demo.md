## Example

Aliqua aute nulla [excepteur](https://google.com) nostrud eu voluptate nulla nisi [proident](/foo/bar) ullamco cillum.

Ea veniam [veniam](https://anu.org! "Anu") id in adipisicing culpa irure ad nulla incididunt [mollit](http://example.com^ "Examplae Incididunt") excepteur sit.

## Proposal

### Custom Syntax

- Jika link diawali dengan `http://`, atau `https://`, atau domain website apapun, maka anchor otomatis mendapatkan `target="_blank" rel="noopener"`
- `^` di akhir link berfungsi untuk membubuhkan `target="_blank" rel="noopener"` secara paksa.
- `~` di akhir link berfungsi untuk membubuhkan `target="_self"` secara paksa.
- `!` di akhir link berfungsi untuk membubuhkan `rel="nofollow"`.
- `--` di akhir link berfungsi untuk membubuhkan `rel="noreferrer"`.
- `$` di akhir link berfungsi untuk membubuhkan `rel="sponsored"`.
- `;;` di akhir link berfungsi untuk membubuhkan `rel="ugc"`.
- Semua simbol bisa digunakan semuanya atau sebagian dan tidak ada aturan urutan penggunaan.

### Extra Attr

Aliquip [consectetur](https://foo.com/bar--! "Title Foo"){rel: "prefetch", hreflang: "en", qux: "rox"} magna velit cupidatat sint ad qui aliquip.

Mollit [labore](/foo/bar^--$! "Buy Foo!"){rel: "sponsored"} anim ipsum in ullamco.

Expected Output:

```html
<p>Aliquip <a href="https://foo.com/bar" target="_blank" title="Title Foo" rel="nooopener noreferrer nofollow prefetch" hreflang="en" qux="rox">consectetur</a> magna velit cupidatat sint ad qui aliquip.</p>

<p>Mollit <a href="/foo/bar" title="Buy Foo!" target="_blank" rel="noopener noreferrer sponsored nofollow">labore</a> anim ipsum in ullamco.</p>
```

### Custom Node

Buat supaya dev (Solid terutama) bisa mengurus output Jsx sendiri dengan komponen custom. Cara kerjanya mirip dengan custom component pada `plugin-cite`
