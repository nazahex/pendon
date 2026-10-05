# JS Example

```js
import { log } from "console"; 
const data = [10, 20, null];
async function* hitung(a, b = 5) {
  for (let x of data) {
    if (x?.val ?? true) yield (a + b) * x;
  }
}
hitung(2).next().then(({ value }) => log(`Hasil: ${value}`));
```
