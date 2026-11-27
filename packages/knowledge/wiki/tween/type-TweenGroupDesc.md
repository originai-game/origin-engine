<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="type-TweenGroupDesc"></a>

### `TweenGroupDesc` (desc struct)

How a sequence or parallel group runs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `repeat` | `number` | `0` | ≥ -1 | — | — | Extra runs of the whole group: 0 once, -1 forever; each run replays the spans its tweens resolved in the first. |
| `unscaled` | `boolean` | `false` | — | — | — | Advance by real ticks, ignoring Time.scale and hit stop. |
| `tag` | `number` | `0` | — | — | — | A number of yours, carried by og.TweenCompleted. |

```ogt
struct TweenGroupDesc {
  doc "How a sequence or parallel group runs."
  desc true
  field repeat i32 "Extra runs of the whole group: 0 once, -1 forever; each run replays the spans its tweens resolved in the first." {
    min -1
  }
  field unscaled bool "Advance by real ticks, ignoring Time.scale and hit stop."
  field tag u32 "A number of yours, carried by og.TweenCompleted."
}
```

<sub>Module `tween` line 102</sub>
