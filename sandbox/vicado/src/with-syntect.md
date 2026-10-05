# Vicado With Regular Codeblock

Dokumen ini menunjukkan Vicado berjalan bersamaan dengan code fence reguler.

## Vicado Fence

```typescript vicado [.panel, #vicado-one] {mount: "visible", foo: "bar", baz: 89, enabled: true}
function sum(a: number, b: number) {
  return a + b
}
```

## Regular Fence (Should Be Highlighted By Syntect)

```rust
fn main() {
    for i in 0..3 {
        println!("{}", i);
    }
}
```

```html
<section class="demo">
  <h2>Regular HTML block</h2>
  <p>This should stay as regular highlighted code.</p>
</section>
```

## Another Vicado Fence

```js vicado {mount: "lazy", compact: true}
const names = ["ana", "budi", "caca"]
console.log(names.map((n) => n.toUpperCase()))
```

## Regular Fence Again

```ts
type User = { id: number; name: string }
const users: User[] = [{ id: 1, name: "A" }]
console.log(users)
```
