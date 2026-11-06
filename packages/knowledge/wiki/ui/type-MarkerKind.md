<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MarkerKind"></a>

### `MarkerKind` (enum)

What a map marker is.

A string: one of the variants below. The zero value is `"player"`.

| Variant | Doc |
|---|---|
| `"player"` | The player: an arrow by heading. |
| `"ally"` | An ally: a teammate's arrow in their colour. |
| `"enemy"` | An enemy: a danger diamond. |
| `"objective"` | An objective: a gold star, kept on the edge when out of range. |
| `"ping"` | A ping: a pulsing ring, kept on the edge. |
| `"loot"` | Loot: a small square in its rarity. |
| `"poi"` | A point of interest: its icon. |

```ogt
enum MarkerKind {
  doc "What a map marker is."
  variant player "The player: an arrow by heading."
  variant ally "An ally: a teammate's arrow in their colour."
  variant enemy "An enemy: a danger diamond."
  variant objective "An objective: a gold star, kept on the edge when out of range."
  variant ping "A ping: a pulsing ring, kept on the edge."
  variant loot "Loot: a small square in its rarity."
  variant poi "A point of interest: its icon."
}
```

<sub>`api.ogt` line 21928</sub>
