<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="type-GraphInterrupt"></a>

### `GraphInterrupt` (enum)

Which transitions may interrupt a cross-fade.

A string: one of the variants below. The zero value is `"any"`.

| Variant | Doc |
|---|---|
| `"any"` | Any eligible transition from the new state, including any-state transitions. |
| `"none"` | Complete this fade first. |
| `"source"` | Only transitions from the state being left. |
| `"next"` | Only transitions from the new state itself. |

```ogt
enum GraphInterrupt {
  doc "Which transitions may interrupt a cross-fade."
  variant any "Any eligible transition from the new state, including any-state transitions."
  variant none "Complete this fade first."
  variant source "Only transitions from the state being left."
  variant next "Only transitions from the new state itself."
}
```

<sub>Module `anim` line 703</sub>
