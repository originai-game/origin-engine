<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-MassKind"></a>

### `MassKind` (enum)

What a mass element template draws.

A string: one of the variants below. The zero value is `"number"`.

| Variant | Doc |
|---|---|
| `"number"` | The instance's value as a number (a damage number, a score). |
| `"text"` | One of strings, by the instance's value (a nameplate, a loot label). |
| `"bar"` | A bar filled to the instance's value 0–1 with a ghost at its ghost (an overhead health bar). |
| `"icon"` | Frame value of frames on texture (a status icon). |
| `"particle"` | A particle of particleShape (confetti, sparks). |

```ogt
enum MassKind {
  doc "What a mass element template draws."
  variant number "The instance's value as a number (a damage number, a score)."
  variant text "One of strings, by the instance's value (a nameplate, a loot label)."
  variant bar "A bar filled to the instance's value 0–1 with a ghost at its ghost (an overhead health bar)."
  variant icon "Frame value of frames on texture (a status icon)."
  variant particle "A particle of particleShape (confetti, sparks)."
}
```

<sub>Module `ui` line 7482</sub>
