<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-StarRatingDesc"></a>

### `StarRatingDesc` (desc struct)

A row of stars earned.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `earned` | `number` | `0` | — | — | — | Stars earned. |
| `max` | `number` | `0` | — | — | — | Stars possible; 0 is 3. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | A star's size; 0 is 48 (the middle of three larger, as Angry Birds and Candy Crush do). |
| `revealAt` | `number \| null` | `null` | — | — | — | og.ui.tick() when the stars start landing in turn (a results screen); null shows them settled. |
| `staggerMs` | `number` | `0` | ≥ 0 | `ms` | — | Between stars landing; 0 is 320. |
| `name` | `string` | `"{earned} of {max} stars"` | — | — | — | What a screen reader says ({earned} of {max} stars). |

```ogt
struct StarRatingDesc {
  doc "A row of stars earned."
  desc true
  field earned u32 "Stars earned."
  field max u32 "Stars possible; 0 is 3."
  field size f32 "A star's size; 0 is 48 (the middle of three larger, as Angry Birds and Candy Crush do)." {
    unit cssPx
    min 0
  }
  field revealAt Option.u32 "og.ui.tick() when the stars start landing in turn (a results screen); null shows them settled."
  field staggerMs f32 "Between stars landing; 0 is 320." {
    unit ms
    min 0
  }
  field name String "What a screen reader says ({earned} of {max} stars)." {
    default "{earned} of {max} stars"
  }
}
```

<sub>Module `ui` line 5900</sub>
