<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-NameplateDesc"></a>

### `NameplateDesc` (desc struct)

A nameplate over a unit.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | What it sits over. |
| `height` | `number` | `2` | — | `m` | — | World units above the target's point (its head). |
| `name` | `string` | `""` | — | — | — | The name. |
| `title` | `string` | `""` | — | — | — | A line under it (<Guild>, a role); empty is none. |
| `level` | `string` | `""` | — | — | — | Its level on a badge; empty is none. |
| `reaction` | `Reaction` | `"hostile"` | — | — | — | Its colour. |
| `health` | `number \| null` | `null` | — | `ratio` | — | Health 0 to 1 for a bar under the name; null draws none. |
| `shield` | `number` | `0` | — | `ratio` | — | An absorb over the health, 0 to 1. |
| `castName` | `string` | `""` | — | — | — | What it is casting; empty is nothing. |
| `castProgress` | `number` | `0` | — | `ratio` | — | How far the cast is. |
| `icon` | `string` | `""` | — | — | — | A class, quest or role icon before the name. |
| `unitClass` | `UnitClass` | `"normal"` | — | — | — | Elite, rare or boss: a dressed frame. |
| `targeted` | `boolean` | `false` | — | — | — | The player's target: larger, outlined, drawn over the rest. |
| `priority` | `number` | `0` | — | — | — | Who keeps its place when tags overlap: a nameplate ranks 100 over loot labels (the target 1000 more), and waypoints rank over every tag. |
| `lod` | `readonly number[]` | `[]` | — | — | — | Distances where it sheds detail: past the first the title goes, past the second the name goes and only the bar stays; empty is 25 and 50. |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | Hidden past this; 0 is 80. |
| `occlusion` | `WorldOcclusion` | `"none"` | — | — | — | Behind the scene; the default fades. |

```ogt
struct NameplateDesc {
  doc "A nameplate over a unit."
  desc true
  field target WorldTarget "What it sits over."
  field height f32 "World units above the target's point (its head)." {
    default 2
    unit m
  }
  field name String "The name."
  field title String "A line under it (<Guild>, a role); empty is none."
  field level String "Its level on a badge; empty is none."
  field reaction Reaction "Its colour."
  field health Option.f32 "Health 0 to 1 for a bar under the name; null draws none." {
    unit ratio
  }
  field shield f32 "An absorb over the health, 0 to 1." {
    unit ratio
  }
  field castName String "What it is casting; empty is nothing."
  field castProgress f32 "How far the cast is." {
    unit ratio
  }
  field icon String "A class, quest or role icon before the name."
  field unitClass UnitClass "Elite, rare or boss: a dressed frame."
  field targeted bool "The player's target: larger, outlined, drawn over the rest."
  field priority i32 "Who keeps its place when tags overlap: a nameplate ranks 100 over loot labels (the target 1000 more), and waypoints rank over every tag."
  field lod List.f32 "Distances where it sheds detail: past the first the title goes, past the second the name goes and only the bar stays; empty is 25 and 50."
  field maxDistance f32 "Hidden past this; 0 is 80." {
    unit m
    min 0
  }
  field occlusion WorldOcclusion "Behind the scene; the default fades."
}
```

<sub>Module `ui` line 6420</sub>
