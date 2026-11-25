<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-advance"></a>

### `og.flow.advance`

End the segment playing now, as its cut would: with `outcome`, the node's `#outcome:<name>` choice; without, the choice taken in the window, else the default (or its own way on, for a node without choices). What a playable segment's game calls when the level is won or lost; it stops at its `flow.hold` until then. Systems may call it; og.flow cuts this tick.

Also known as: level complete, finish level, next segment, continue the story, Unreal Level Sequence end, Telltale playable section

```ts
og.flow.advance(world: World, flow: Entity, outcome?: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `outcome` | `string` | yes | The `#outcome` to take, such as `won`; omitted for a node without outcomes. |

**Returns** `Result<void>`: Refused for an entity that is no flow, one that has ended, an outcome its node has not, or none for a node that ends only by its outcomes.

Binding symbol `og_flow_advance` · publish builds: kept

```ogt
fn advance {
  doc "End the segment playing now, as its cut would: with `outcome`, the node's `#outcome:<name>` choice; without, the choice taken in the window, else the default (or its own way on, for a node without choices). What a playable segment's game calls when the level is won or lost; it stops at its `flow.hold` until then. Systems may call it; og.flow cuts this tick."
  aliases "level complete, finish level, next segment, continue the story, Unreal Level Sequence end, Telltale playable section"
  param world World "The world."
  param flow Entity "The flow."
  param outcome String "The `#outcome` to take, such as `won`; omitted for a node without outcomes." {
    optional true
  }
  returns Result.Void "Refused for an entity that is no flow, one that has ended, an outcome its node has not, or none for a node that ends only by its outcomes."
}
```

<sub>Module `flow` line 349</sub>
