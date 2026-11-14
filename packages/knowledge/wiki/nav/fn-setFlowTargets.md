<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`nav`](../nav.md) · [index](../index.md)

<a id="fn-setFlowTargets"></a>

### `og.nav.setFlowTargets`

Move a flow field's targets (rebuilds it).

```ts
og.nav.setFlowTargets(world: World, field: Navigation, targets: readonly Vector3[]): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `field` | `Navigation` | no | The flow field. |
| `targets` | `readonly Vector3[]` | no | The new targets. |

**Returns** `Result<void>`: Refused for an unknown flow field.

Binding symbol `og_nav_set_flow_targets` · publish builds: kept

```ogt
fn setFlowTargets {
  doc "Move a flow field's targets (rebuilds it)."
  param world World "The world."
  param field Navigation "The flow field."
  param targets List.Vector3 "The new targets."
  returns Result.Void "Refused for an unknown flow field."
}
```

<sub>Module `nav` line 552</sub>
