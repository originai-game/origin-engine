<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-PointerAction"></a>

### `PointerAction` (enum)

What an injected pointer event does.

A string: one of the variants below. The zero value is `"click"`.

| Variant | Doc |
|---|---|
| `"click"` | Down, then up on the next tick, at the point. |
| `"move"` | A move to the point. |
| `"down"` | A press at the point. |
| `"up"` | A release at the point. |

```ogt
enum PointerAction {
  doc "What an injected pointer event does."
  variant click "Down, then up on the next tick, at the point."
  variant move "A move to the point."
  variant down "A press at the point."
  variant up "A release at the point."
}
```

<sub>Module `input` line 573</sub>
