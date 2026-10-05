## Extra Syntax

Sintaksis tambahan untuk plugin-img

### Width and Height

!w300h800[foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<img width="300" height="800" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
```

---

!!h600[foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<figure><img height="800" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></figure>
```

### Lazy Loading

?![lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

````html
<img loading="lazy" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />

---

!?[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)


```html
<img loading="lazy" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
````

---

?!![lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

```html
<figure><img loading="lazy" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bnch_egora7.webp" /></figure>
```

### Async Decoding

~![lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

````html
<img decoding="async" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />

---

!~[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)


```html
<img decoding="async" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" />
````

---

!!~[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

```html
<figure><img decoding="async" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bnch_egora7.webp" /></figure>
```

## Complete Combination

~?!!h300w800[lorem ipsum](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg"} Exercitation qui **exercitation** dolor velit _aliqua_ consectetur voluptate [consequat](/consequat) labore elit non esse occaecat.

Expected result:

```html
<figure id="custom-id"class="extra class or" data:foo="bar" data:baz="23" style="--wix:sum;--rotate:5deg;"><img decoding="async" loading="lazy" height="300" width="800" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /><figcaption>Exercitation qui <strong>exercitation</strong> dolor velit <em>aliqua</em> consectetur voluptate <a href="/consequat">consequat</a> labore elit non esse occaecat.</figcaption></figure>
```
