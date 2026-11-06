<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-DensityMode"></a>

### `DensityMode` (enum)

Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Touch sizes on a coarse pointer or a narrow surface, mouse sizes otherwise. |
| `"mouse"` | A mouse and keyboard: the tightest controls. |
| `"pad"` | A gamepad across a room: larger controls, rows and gutters that read from a couch. |
| `"touch"` | A finger: every control at least 44 CSS px. |

```ogt
enum DensityMode {
  doc "Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise."
  variant auto "Touch sizes on a coarse pointer or a narrow surface, mouse sizes otherwise."
  variant mouse "A mouse and keyboard: the tightest controls."
  variant pad "A gamepad across a room: larger controls, rows and gutters that read from a couch."
  variant touch "A finger: every control at least 44 CSS px."
}
```

<sub>`api.ogt` line 15705</sub>
