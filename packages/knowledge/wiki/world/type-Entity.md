<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`world`](../world.md) · [index](../index.md)

<a id="type-Entity"></a>

### `Entity` (handle)

An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number.

Generations start at 1, so 0 is never a live entity.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Entity {
  doc "An entity: an index and a generation. The value is index + generation * 2^32, with the generation below 2^21, so it stays an exact JavaScript number."
  doc "Generations start at 1, so 0 is never a live entity."
}
```

<sub>`api.ogt` line 861</sub>
