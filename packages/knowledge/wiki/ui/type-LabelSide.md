<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LabelSide"></a>

### `LabelSide` (enum)

Which side of a switch or a check its label sits.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | Before it, the switch at the row's end (a settings row: the default for a switch). |
| `"end"` | After it (the default for a check). |

```ogt
enum LabelSide {
  doc "Which side of a switch or a check its label sits."
  variant start "Before it, the switch at the row's end (a settings row: the default for a switch)."
  variant end "After it (the default for a check)."
}
```

<sub>Module `ui` line 3522</sub>
