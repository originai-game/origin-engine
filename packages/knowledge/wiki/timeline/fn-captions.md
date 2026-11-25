<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`timeline`](../timeline.md) · [index](../index.md)

<a id="fn-captions"></a>

### `og.timeline.captions`

The captions the world's timelines are saying as of the last tick, in the timelines' order and then their tracks': what a subtitles screen shows (the built-in Subtitles takes them as `captions`). Nothing draws them otherwise.

```ts
og.timeline.captions(world: World): readonly TimelineCaption[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly TimelineCaption[]`: Empty when none is showing.

Binding symbol `og_timeline_captions` · publish builds: kept

```ogt
fn captions {
  doc "The captions the world's timelines are saying as of the last tick, in the timelines' order and then their tracks': what a subtitles screen shows (the built-in Subtitles takes them as `captions`). Nothing draws them otherwise."
  param world World "The world."
  returns List.TimelineCaption "Empty when none is showing."
}
```

<sub>Module `timeline` line 82</sub>
