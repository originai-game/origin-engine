<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`vfx`](../vfx.md) · [index](../index.md)

<a id="fn-spawn"></a>

### `og.vfx.spawn`

Play a composed effect now: a new entity with og.Transform (at the desc's position and rotation, a child of its parent when it has one) and og.Vfx started at this tick (world.tick). A system calling it writes og.Transform, og.Vfx and, with a parent, og.Parent. The entity despawns when the effect ends (og.Vfx.despawn).

Also known as: play effect, spawn particle effect, play vfx, spawn effect at, Unity Instantiate particle prefab, Unreal SpawnSystemAtLocation, SpawnSystemAttached, Godot GPUParticles3D restart, Effekseer play

```ts
og.vfx.spawn(world: World, desc?: VfxSpawnDesc): Result<Entity>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `VfxSpawnDesc` | yes | The effect and where. |

**Returns** `Result<Entity>`: Refused for an asset not loaded in this world or a dead parent.

Binding symbol `og_vfx_spawn` · publish builds: kept

```ogt
fn spawn {
  doc "Play a composed effect now: a new entity with og.Transform (at the desc's position and rotation, a child of its parent when it has one) and og.Vfx started at this tick (world.tick). A system calling it writes og.Transform, og.Vfx and, with a parent, og.Parent. The entity despawns when the effect ends (og.Vfx.despawn)."
  aliases "play effect, spawn particle effect, play vfx, spawn effect at, Unity Instantiate particle prefab, Unreal SpawnSystemAtLocation, SpawnSystemAttached, Godot GPUParticles3D restart, Effekseer play"
  param world World "The world."
  param desc VfxSpawnDesc "The effect and where."
  returns Result.Entity "Refused for an asset not loaded in this world or a dead parent."
}
```

<sub>Module `vfx` line 81</sub>
