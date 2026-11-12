<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-timeline"></a>

### `og.load.timeline`

Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel.

```ts
og.load.timeline(world: World, path: string): Result<TimelineAsset>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The timeline's asset id, such as `timelines/intro.ogt`. |

**Returns** `Result<TimelineAsset>`: Refused for a path not in the bundle (run og cook) or a file that is not a timeline.

Binding symbol `og_load_timeline` · publish builds: kept

```ogt
fn timeline {
  doc "Load a timeline: an `.ogt` of `type timeline` (the og.timeline module's file: duration, paths and tracks of keys), cooked with its caption files imported. Loaded once per path; play it with an og.Timeline whose asset is the handle, or `timeline:play:<name>` on the command channel."
  param world World "The world."
  param path String "The timeline's asset id, such as `timelines/intro.ogt`."
  returns Result.TimelineAsset "Refused for a path not in the bundle (run og cook) or a file that is not a timeline."
}
```

<sub>Module `load` line 465</sub>
