---
title: "Universal Custom Syntax"
references:
  suryana-2026:
    id: suryana-2026
    type: book
    title: Masa Depan Rekayasa Perangkat Lunak
    authors:
      - firstName: Eko
        lastName: Suryana
    publisher: TechPress Indonesia
    publisherLocation: Jakarta
    issuedDate:
      year: 2026
    isbn: 978-602-0000-00-0
    language: id

  paper-smith:
    id: paper-smith
    type: journal
    title: Generative MDX to PDF Compilation Architectures
    authors:
      - firstName: John
        lastName: Smith
      - firstName: Jane
        lastName: Doe
    containerTitle: Journal of Web Engineering
    volume: "18"
    issue: "4"
    pages: 210-225
    issuedDate:
      year: 2025
      month: 8
    doi: 10.1016/j.jwe.2025.08.001
    language: en
---

## Image

~?!!h300w800[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg"} Exercitation qui **exercitation** $dolor$ {velit} _aliqua_ [[Anim Esta (Officia) | Anim]] consectetur [^^]("suryana-2026") voluptate [labore](/foo/bar^--$! "Buy Foo!") labore elit non esse occaecat. [^^]("suryana-2026", "hlm. 45") ::[ep2] Et deserunt sunt consectetur elit.::

## Anchor

[consectetur](https://foo.com/bar--!~$ "Title Foo")[.extra,.qur,#rew]{rel: "prefetch", hreflang: "en", qux: "rox"} Incididunt {cupidatat} quis et pariatur commodo laborum consectetur anim do minim anim in.

## Cite

Minim [^^]("paper-smith")[.paper,#smith]{rox:"hen"} esse do ut anim proident est qui $F$ magna non elit quis eiusmod dolore.

##[heading-slug] Heading

###[foo-bar][.hoo]{ qun: "anu" } Foo Bar Barosa

## Table

[Laporan [Penjualan](https://foo.com "Foo") 2026 [[Foo | Foo Bar]]][.striped,#sales-table,.foo]{sortable: "true", qux: true}
| Produk | Stok | Harga | Status |
| :---(200px)[.v-top] | :---:[.v-top] | ---:(30%)[.v-bottom] | :---: |
| [[Laptop (Pro) | Pro]] | Foo ![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg" } Bar | 15.000.000 | [Tersedia](/foo!~$) |
| Mouse [[Wireless]] | > | 250.000 { rox: "rox" } | ![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg" } |
| {Keyboard} Mekanikal[^^]("suryana-2026") | 0 [.text-red] | $850.000$ | Habis | -[.row-danger]
| ^ | 5 | 5.200.000 | ~?!h300w800[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg"} Exercitation qui **exercitation** dolor velit _aliqua_ [[Anim Esta (Officia) | Anim]] consectetur [^^]("suryana-2026") voluptate [labore](/foo/bar^--$! "Buy Foo!") labore elit non esse occaecat. [^^]("suryana-2026", "hlm. 45") |
|===|
| Total Inventaris | > | 21.300.000 | - |

[^^]("suryana-2026")

## Latex

$$((H \rightarrow O) \land \lnot O) \rightarrow \lnot H$$

Rumus ini dibaca: bila $H$ mengimplikasikan $O$ dan $O$ tidak terjadi, maka $H$ tidak benar. Kebalikannya tidak sah: dari $H \rightarrow O$ dan $O$ terjadi, kita tidak boleh menyimpulkan $H$ pasti benar, sebab {hipotesis} lain bisa menghasilkan $O$ yang sama. Karena itu, hipotesis yang lolos uji lazimnya disebut didukung, bukan terbukti.

<!-- {hipotesis} should be rendered as  &#123;hipotesis&#125-->

## Custom Plugins

:::note["Foo Bar"]("Bax Xo")

Ullamco excepteur adipisicing quis ullamco ea mollit et nulla sint non et id commodo commodo [^^]("suryana-2026", "hlm. 45")[.suyn,#surya]{rox:"hen"}::[ep3] Aliquip commodo commodo et ut reprehenderit qui magna laboris et.::.

> ! In amet deserunt consequat cupidatat laboris cupidatat.

:::

> ? Ea reprehenderit sint exercitation velit quis officia aliqua est consectetur tempor cupidatat qui sint.
>
> - Est deserunt excepteur minim in ad nisi.
> - Est deserunt excepteur minim in ad nisi.

:::error["Bux"]

- Ad excepteur nulla amet cupidatat aliqua.
- Aliqua nostrud mollit cillum reprehenderit deserunt nostrud voluptate eu duis officia incididunt excepteur.
- Velit adipisicing officia mollit aliquip anim cupidatat aute ad labore.

:::

## Lists

- Alpha
- Beta
- Gamma

1. Satu
2. Dua
3. Tiga

> Blockquote dengan list:
>
> - Alpha
> - Beta

:::note["List in a Parego block"]("Ordered")

1. Primero
2. Segundo
3. Tercero

:::

> ? List in a Hint block.
>
> - Est deserunt excepteur minim in ad nisi.
> - Aliqua nostrud mollit cillum reprehenderit deserunt nostrud.

:::error["List in an error block"]

- Ad excepteur nulla amet cupidatat aliqua.
- Velit adipisicing officia mollit aliquip anim cupidatat aute ad labore.

:::

[Lists in table cells][.striped,#list-table]{sortable: "true"}
| Jenis | Isi Sel |
| :---(160px) | :--- |
| Bullet | - Alpha\n- Beta\n- Gamma |
| Ordered | 1. Satu\n2. Dua\n3. Tiga |
| Nested | - ::[ep4] Aliquip commodo commodo.::\n - Nested satu\n - Nested dua |
| Mixed | Paragraf singkat.\n\n- Setelah paragraf\n- Item kedua |
