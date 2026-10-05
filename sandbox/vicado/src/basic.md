# Vicado Basic

Dokumen ini berisi contoh Vicado minimal dan lanjutan.

## Minimal

```typescript vicado
const a = 1
const b = 2
console.log(a + b)
```

## Full Props

```tsx vicado [.hero, is-vicado, #editor-main] {mount: "visible", theme: "solarized", lineNumbers: true, tabSize: 2, fontScale: 1.1}
export function greet(name: string) {
  return `hello ${name}`
}
```

## Multiple Languages

```javascript vicado {mount: "defer", cache: true}
const now = new Date()
console.log(now.toISOString())
```

```css vicado [.snippet] {mount: "visible"}
.button {
  color: #222;
  border: 1px solid #888;
}
```
