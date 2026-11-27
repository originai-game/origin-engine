<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`game`](../game.md) · [index](../index.md)

<a id="type-ProjectProfile"></a>

### `ProjectProfile` (enum)

The normalized startup profile. A 2.5d creation uses 3d.

A string: one of the variants below. The zero value is `"3d"`.

| Variant | Doc |
|---|---|
| `"3d"` | Three-dimensional simulation and rendering. |
| `"2d"` | Two-dimensional simulation and rendering. |

```ogt
enum ProjectProfile {
  doc "The normalized startup profile. A 2.5d creation uses 3d."
  variant "3d" "Three-dimensional simulation and rendering."
  variant "2d" "Two-dimensional simulation and rendering."
}
```

<sub>Module `game` line 6</sub>
