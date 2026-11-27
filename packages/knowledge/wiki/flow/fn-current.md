<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-current"></a>

### `og.flow.current`

The flow project.ts's `flow` object started before GameDesc.start (the engine loads its dialogue, prepares it and starts it, and its FlowScreen draws it unless flow.screen is game): what a playable segment's game passes to og.flow.advance.

Also known as: the game's flow, main flow, declared flow, story flow entity

```ts
og.flow.current(world: World): Entity | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Entity | null`: Null without a `flow` line, or once og.flow.stop ended it.

Binding symbol `og_flow_current` · publish builds: kept

```ogt
fn current {
  doc "The flow project.ts's `flow` object started before GameDesc.start (the engine loads its dialogue, prepares it and starts it, and its FlowScreen draws it unless flow.screen is game): what a playable segment's game passes to og.flow.advance."
  aliases "the game's flow, main flow, declared flow, story flow entity"
  param world World "The world."
  returns Option.Entity "Null without a `flow` line, or once og.flow.stop ended it."
}
```

<sub>Module `flow` line 327</sub>
