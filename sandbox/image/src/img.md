# Image Demo

## Markdown Vanila

### Basic

![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<img alt="Alternative Text" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
```

---

![](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<img alt="" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
```

### Figured

!![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp) Exercitation qui **exercitation** dolor velit _aliqua_ consectetur voluptate [consequat](/consequat) labore elit non esse occaecat.

Expected result:

```html
<figure><img alt="Alternative Text" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /><figcaption>Exercitation qui <strong>exercitation</strong> dolor velit <em>aliqua</em> consectetur voluptate <a href="/consequat">consequat</a> labore elit non esse occaecat.</figcaption></figure>
```

> Any markdown inline renderer must work properly.

---

!![](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<figure><img alt="" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></figure>
```

## plugin-img

Plugin pendon untuk sintaksis img lebih lanjut.

!![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg" }

Expected result:

```html
<figure id="custom-id"class="extra class or" data:foo="bar" data:baz="23" style="--wix:sum;--rotate:5deg;"><img alt="Alternative Text" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></figure>
```

---

![Aternative Text](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg" }

Expected result:

```html
<img alt="Alternative Text" id="custom-id"class="extra class or" data:foo="bar" data:baz="23" style="--wix:sum;--rotate:5deg;" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
```
