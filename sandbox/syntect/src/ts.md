# TypeScript Demo

```ts
// Imports & exports
import type { User, Role } from "./types";
import { readFile as rf, writeFile } from "fs/promises";
export interface Person {
  id: number;
  name: string;
}
export type Id = number | string;
export enum Status {
  Active = "A",
  Disabled = "D",
}
export const VERSION: string = "1.0.0";

// Declarations & modifiers
class Base<T = unknown> {
  protected value: T;
  constructor(v: T) {
    this.value = v;
  }
}
class UserService extends Base<Person> {
  #secret: string = "x";
  constructor(private store: Map<Id, Person>) {
    super({ id: 0, name: "" });
  }
  async get(id: Id): Promise<Person | undefined> {
    return this.store.get(id ?? 0);
  }
  set(id: Id, person: Person): void {
    this.store.set(id, person);
  }
}

// Functions, parameters, overloading
function add(a: number, b: number): number;
function add(a: string, b: string): string;
function add(a: any, b: any) {
  return a + b;
}

// Variables & object literals
let count: number = 0;
const user: Person = { id: 1, name: "Ada" };
const arr: Array<number> = [1, 2, 3];

// Template strings & expressions
const msg = `User: ${user.name}, ids: ${arr.join(",")}`;

// Operators & punctuation
count += 1;
count *= 2;
count -= 3;
count /= 4;
const ok = count > 0 && (arr?.length ?? 0) === 3;
const bit = (1 << 2) | ((8 & 3) ^ 1);

// Type assertions & narrowing
const maybe: unknown = "x";
if (typeof maybe === "string") {
  const len = (maybe as string).length;
}

// Generics & constraints
function map<T, U>(xs: T[], f: (x: T) => U): U[] {
  return xs.map(f);
}

type Pair<A, B> = [A, B];
const p: Pair<number, string> = [42, "answer"];

// Union/Intersection, Literal types
type U = string | number | boolean;
type I = { a: number } & { b: string };
const flag: true = true;

// Mapped types & conditional types
type Readonly<T> = { readonly [K in keyof T]: T[K] };
type IfAny<T, Y, N> = 0 extends 1 & T ? Y : N;

// Tuple, destructuring, spread
const tuple: [number, string] = [7, "seven"];
const [n, s] = tuple;
const obj = { ...user, role: "admin" as Role };

// Labels, loops, control flow
outer: for (let i = 0; i < 3; i++) {
  inner: for (let j = 0; j < 2; j++) {
    if (i + j > 3) break outer;
    continue inner;
  }
}

// Try/catch/finally
try {
  count = await (await rf("file.txt", "utf8")).length;
} catch (e) {
  console.error(e);
} finally {
  await writeFile("out.txt", msg);
}

// Type guards & predicates
function isPerson(x: unknown): x is Person {
  return typeof x === "object" && x !== null && "id" in (x as any);
}

// Namespace (TSX ignored here)
namespace NS {
  export const K = 1;
}
console.log(NS.K);
```
