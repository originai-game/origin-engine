<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-LootLabelDesc"></a>

### `LootLabelDesc` (desc struct)

A label on an item lying in the world.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | The item. |
| `height` | `number` | `0.4` | — | `m` | — | World units above it. |
| `name` | `string` | `""` | — | — | — | The item's name. |
| `rarity` | `Rarity` | `"none"` | — | — | — | Its tier: colour, pips, and a beam from rare up. |
| `count` | `number` | `0` | — | — | — | A stack's count (×3); 0 and 1 show none. |
| `icon` | `string` | `""` | — | — | — | Its icon. |
| `beam` | `boolean` | `true` | — | — | — | A light beam over it (the default for rare and up). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | Hidden past this; 0 is 40. |

```ogt
struct LootLabelDesc {
  doc "A label on an item lying in the world."
  desc true
  field target WorldTarget "The item."
  field height f32 "World units above it." {
    default 0.4
    unit m
  }
  field name String "The item's name."
  field rarity Rarity "Its tier: colour, pips, and a beam from rare up."
  field count u32 "A stack's count (×3); 0 and 1 show none."
  field icon String "Its icon."
  field beam bool "A light beam over it (the default for rare and up)." {
    default true
  }
  field maxDistance f32 "Hidden past this; 0 is 40." {
    unit m
    min 0
  }
}
```

<sub>Module `ui` line 6733</sub>
