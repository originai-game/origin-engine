<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-A11yActionKind"></a>

### `A11yActionKind` (enum)

What an assistive technology asks of a UI control.

A string: one of the variants below. The zero value is `"focus"`.

| Variant | Doc |
|---|---|
| `"focus"` | Move the UI's focus to it (with the focus ring). |
| `"activate"` | Press it, as Enter on it would (a text control takes the focus). |
| `"increment"` | Step an adjustable control (a slider, a stepper) up. |
| `"decrement"` | Step an adjustable control down. |

```ogt
enum A11yActionKind {
  doc "What an assistive technology asks of a UI control."
  variant focus "Move the UI's focus to it (with the focus ring)."
  variant activate "Press it, as Enter on it would (a text control takes the focus)."
  variant increment "Step an adjustable control (a slider, a stepper) up."
  variant decrement "Step an adjustable control down."
}
```

<sub>Module `input` line 731</sub>
