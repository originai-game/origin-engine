<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-PointerLockReport"></a>

### `PointerLockReport` (struct)

The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own; og.PointerLock.locked follows it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `locked` | `boolean` | `false` | — | — | — | Locked now (true) or released (false). |

```ogt
struct PointerLockReport {
  doc "The host reports whether the pointer is locked to the play surface now (an engine extension of protocol 5): the web shell sends one from the page's pointerlockchange, and a playtest can send its own; og.PointerLock.locked follows it."
  field locked bool "Locked now (true) or released (false)."
}
```

<sub>Module `input` line 640</sub>
