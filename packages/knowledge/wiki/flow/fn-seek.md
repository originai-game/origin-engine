<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-seek"></a>

### `og.flow.seek`

Seek within the current segment (0 to its cut time, e.g., -5s rewind). Systems may call it.

```ts
og.flow.seek(world: World, flow: Entity, time: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `time` | `number` | no | Seconds into the segment, clamped to 0..cut. |

**Returns** `Result<void>`: Refused for an entity that is no flow, one that has ended, or a non-finite/negative time.

Binding symbol `og_flow_seek` · publish builds: kept

```ogt
fn seek {
  doc "Seek within the current segment (0 to its cut time, e.g., -5s rewind). Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param time f32 "Seconds into the segment, clamped to 0..cut."
  returns Result.Void "Refused for an entity that is no flow, one that has ended, or a non-finite/negative time."
}
```

<sub>Module `flow` line 381</sub>
