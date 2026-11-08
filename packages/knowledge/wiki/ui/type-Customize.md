<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Customize"></a>

### `Customize` (desc struct)

An element the player can move, scale and fade in the HUD edit mode (docs/ui.md §2.1): a HUD group, a touch stick or button. Its placement is kept per player under key (og.ui.hudLayout). The zero value allows all three, from half to twice its size and down to a fifth of its opacity.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `key` | `string` | `""` | — | — | — | Its name in the saved layout; empty is its path. Name it when its path may change between versions of the game. |
| `lockMove` | `boolean` | `false` | — | — | — | It stays where it is laid out. |
| `lockScale` | `boolean` | `false` | — | — | — | It keeps its size. |
| `lockOpacity` | `boolean` | `false` | — | — | — | It stays opaque. |
| `minScale` | `number` | `0` | ≥ 0 | `ratio` | — | The smallest it scales to; 0 is 0.5. |
| `maxScale` | `number` | `0` | ≥ 0 | `ratio` | — | The largest it scales to; 0 is 2. |
| `minOpacity` | `number` | `0` | 0 … 1 | `ratio` | — | The faintest it fades to; 0 is 0.2. |

```ogt
struct Customize {
  doc "An element the player can move, scale and fade in the HUD edit mode (docs/ui.md §2.1): a HUD group, a touch stick or button. Its placement is kept per player under key (og.ui.hudLayout). The zero value allows all three, from half to twice its size and down to a fifth of its opacity."
  desc true
  field key String "Its name in the saved layout; empty is its path. Name it when its path may change between versions of the game."
  field lockMove bool "It stays where it is laid out."
  field lockScale bool "It keeps its size."
  field lockOpacity bool "It stays opaque."
  field minScale f32 "The smallest it scales to; 0 is 0.5." {
    unit ratio
    min 0
  }
  field maxScale f32 "The largest it scales to; 0 is 2." {
    unit ratio
    min 0
  }
  field minOpacity f32 "The faintest it fades to; 0 is 0.2." {
    unit ratio
    min 0
    max 1
  }
}
```

<sub>Module `ui` line 4744</sub>
