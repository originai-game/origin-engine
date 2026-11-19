<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`shared`](../shared.md) · [index](../index.md)

<a id="fn-teleport"></a>

### `og.shared.teleport`

Move this avatar anywhere at once (a map click, a portal): its og.Transform is set and the next proposal is a teleport, which the server takes past the speed rule when its policy allows, and remote clients draw at once instead of moving it across.

See `shared-worlds §4`.

```ts
og.shared.teleport(world: World, position: Vector3): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `position` | `Vector3` | no | Where, in world space. |

**Returns** `Result<void>`: Refused without an og.SharedAvatar entity or for a position that is not finite.

Binding symbol `og_shared_teleport` · publish builds: kept

```ogt
fn teleport {
  doc "Move this avatar anywhere at once (a map click, a portal): its og.Transform is set and the next proposal is a teleport, which the server takes past the speed rule when its policy allows, and remote clients draw at once instead of moving it across."
  guide "shared-worlds §4"
  param world World "The world."
  param position Vector3 "Where, in world space."
  returns Result.Void "Refused without an og.SharedAvatar entity or for a position that is not finite."
}
```

<sub>Module `shared` line 185</sub>
