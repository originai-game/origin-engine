<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UnitClass"></a>

### `UnitClass` (enum)

A unit's standing, which dresses its frame.

A string: one of the variants below. The zero value is `"normal"`.

| Variant | Doc |
|---|---|
| `"normal"` | A plain unit. |
| `"elite"` | An elite: a gold dragon on its portrait's ring. |
| `"rare"` | A rare: a silver one. |
| `"boss"` | A boss: a skull and a heavier frame. |

```ogt
enum UnitClass {
  doc "A unit's standing, which dresses its frame."
  variant normal "A plain unit."
  variant elite "An elite: a gold dragon on its portrait's ring."
  variant rare "A rare: a silver one."
  variant boss "A boss: a skull and a heavier frame."
}
```

<sub>Module `ui` line 6153</sub>
