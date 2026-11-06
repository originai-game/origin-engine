<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="fn-info"></a>

### `og.timeline.info`

What a loaded timeline holds.

```ts
og.timeline.info(world: World, asset: TimelineAsset): Result<TimelineInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `asset` | `TimelineAsset` | no | The timeline. |

**Returns** `Result<TimelineInfo>`: Refused for a timeline not loaded in this world.

Binding symbol `og_timeline_info` · publish builds: kept

```ogt
fn info {
  doc "What a loaded timeline holds."
  param world World "The world."
  param asset TimelineAsset "The timeline."
  returns Result.TimelineInfo "Refused for a timeline not loaded in this world."
}
```

<sub>`api.ogt` line 26217</sub>
