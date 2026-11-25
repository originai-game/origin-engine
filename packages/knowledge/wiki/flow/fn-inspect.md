<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`flow`](../flow.md) · [index](../index.md)

<a id="fn-inspect"></a>

### `og.flow.inspect`

Trigger an active `inspect` hotspot by id without cutting: marks it inspected and sends og.FlowInspected. Systems may call it.

```ts
og.flow.inspect(world: World, flow: Entity, hotspot: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `flow` | `Entity` | no | The flow. |
| `hotspot` | `string` | no | The hotspot's id. |

**Returns** `Result<void>`: Refused for an entity that is no flow, or an unknown/inactive inspect hotspot.

Binding symbol `og_flow_inspect` · publish builds: kept

```ogt
fn inspect {
  doc "Trigger an active `inspect` hotspot by id without cutting: marks it inspected and sends og.FlowInspected. Systems may call it."
  param world World "The world."
  param flow Entity "The flow."
  param hotspot String "The hotspot's id."
  returns Result.Void "Refused for an entity that is no flow, or an unknown/inactive inspect hotspot."
}
```

<sub>Module `flow` line 390</sub>
