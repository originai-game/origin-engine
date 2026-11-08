<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-RadialItem"></a>

### `RadialItem` (desc struct)

A slice of a radial wheel.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | What it is. |
| `icon` | `string` | `""` | — | — | — | Its icon; empty is none. |
| `detail` | `string` | `""` | — | — | — | A line the centre shows while it is chosen. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but not choosable. |
| `cost` | `string` | `""` | — | — | — | A price under its label, a coin before it (a build wheel's 150); empty is none. |
| `costIcon` | `string` | `"coin"` | — | — | — | The cost's icon. |
| `unaffordable` | `boolean` | `false` | — | — | — | Its cost cannot be paid now: the cost in the danger hue. It stays choosable (the game refuses it and says why) unless disabled. |

```ogt
struct RadialItem {
  doc "A slice of a radial wheel."
  desc true
  field label String "What it is."
  field icon String "Its icon; empty is none."
  field detail String "A line the centre shows while it is chosen."
  field disabled bool "Shown but not choosable."
  field cost String "A price under its label, a coin before it (a build wheel's 150); empty is none."
  field costIcon String "The cost's icon." {
    default "coin"
  }
  field unaffordable bool "Its cost cannot be paid now: the cost in the danger hue. It stays choosable (the game refuses it and says why) unless disabled."
}
```

<sub>Module `ui` line 2311</sub>
