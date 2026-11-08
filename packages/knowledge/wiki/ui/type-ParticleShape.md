<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-ParticleShape"></a>

### `ParticleShape` (enum)

A particle's shape.

A string: one of the variants below. The zero value is `"square"`.

| Variant | Doc |
|---|---|
| `"square"` | A rectangle of size (a long thin one is a streak). |
| `"circle"` | A disc. |
| `"diamond"` | A diamond. |

```ogt
enum ParticleShape {
  doc "A particle's shape."
  variant square "A rectangle of size (a long thin one is a streak)."
  variant circle "A disc."
  variant diamond "A diamond."
}
```

<sub>Module `ui` line 3234</sub>
