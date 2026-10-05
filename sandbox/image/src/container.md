## Extra Container

### Paragraph

p!w300h800[foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg"}

Expected result:

```html
<p id="custom-id" class="extra class or" data:foo="bar" data:baz="23" style="--wix:sum;--rotate:5deg;"><img width="300" height="800" alt="lorem ipsum" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></p>
```

---

p![foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<p><img alt="foo" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></p>
```

### Division

d![foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)

Expected result:

```html
<p><img alt="foo" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></p>
```

---

d~!w300h800[foo](https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp)[.extra,.class,.or,#custom-id]{foo: "bar", baz: 23, --wix: "sum", --rotate: "5deg"}

Expected result:

```html
<div  id="custom-id" class="extra class or" data:foo="bar" data:baz="23" style="--wix:sum;--rotate:5deg;"><img decoding="async" width="300" height="800" alt="foo" src="https://res.cloudinary.com/ddbrg3jf1/image/upload/v1773735344/revano_bench_egora7.webp" /></div>
```
