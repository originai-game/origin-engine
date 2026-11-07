<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-MirrorAxis"></a>

### `MirrorAxis` (enum)

The world axis a part is mirrored along, after its placement.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Not mirrored. |
| `"x"` | x negated: the mirror image across the yz plane. |
| `"y"` | y negated: across the xz plane. |
| `"z"` | z negated: across the xy plane. |

```ogt
enum MirrorAxis {
  doc "The world axis a part is mirrored along, after its placement."
  variant none "Not mirrored."
  variant x "x negated: the mirror image across the yz plane."
  variant y "y negated: across the xz plane."
  variant z "z negated: across the xy plane."
}
```

<sub>Module `render` line 3337</sub>
