<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MeterKind"></a>

### `MeterKind` (enum)

What a meter measures, which sets its colour.

A string: one of the variants below. The zero value is `"health"`.

| Variant | Doc |
|---|---|
| `"health"` | health; warn under 30 %, danger under 15 %. |
| `"mana"` | mana. |
| `"xp"` | xp. |
| `"primary"` | primary. |

```ogt
enum MeterKind {
  doc "What a meter measures, which sets its colour."
  variant health "health; warn under 30 %, danger under 15 %."
  variant mana "mana."
  variant xp "xp."
  variant primary "primary."
}
```

<sub>Module `ui` line 829</sub>
