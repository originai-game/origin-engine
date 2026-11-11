<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`system`](../system.md) · [index](../index.md)

<a id="type-StateWhen"></a>

### `StateWhen` (enum)

When a system bound to a state machine's value runs.

A string: one of the variants below. The zero value is `"during"`.

| Variant | Doc |
|---|---|
| `"during"` | Every tick while the state machine holds the value. |
| `"enter"` | Once, as the tick begins, when the machine enters the value (the first tick, for its initial value). |
| `"exit"` | Once, as the tick begins, when the machine leaves the value. |

```ogt
enum StateWhen {
  doc "When a system bound to a state machine's value runs."
  variant during "Every tick while the state machine holds the value."
  variant enter "Once, as the tick begins, when the machine enters the value (the first tick, for its initial value)."
  variant exit "Once, as the tick begins, when the machine leaves the value."
}
```

<sub>Module `system` line 7</sub>
