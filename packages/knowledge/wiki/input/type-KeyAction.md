<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-KeyAction"></a>

### `KeyAction` (enum)

What an injected key event does.

A string: one of the variants below. The zero value is `"press"`.

| Variant | Doc |
|---|---|
| `"press"` | Down, then up on the next tick. |
| `"down"` | Held down until an up. |
| `"up"` | Released. |

```ogt
enum KeyAction {
  doc "What an injected key event does."
  variant press "Down, then up on the next tick."
  variant down "Held down until an up."
  variant up "Released."
}
```

<sub>Module `input` line 566</sub>
