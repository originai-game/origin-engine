<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-emitter"></a>

### `og.load.emitter`

A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles`: its shape, emission, motion, look, rendering (billboards, meshes or 2D sprites), collision, sub-emitters and events. Every line runs the same in compute and on the CPU path. The textures, texture array, mesh model, effect material and sub-emitters it names are loaded with it (a sub-emitter that fires its parent is refused); while the array's streaming group is not resident its systems draw nothing. Loaded once per path.

See `loading §5`.

```ts
og.load.emitter(world: World, path: string): Result<ParticleEmitter>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `fx/sparks.ogt`. |

**Returns** `Result<ParticleEmitter>`: Refused for a path not in the bundle, or a texture, texture array, model or sub-emitter it cannot load.

Binding symbol `og_load_emitter` · publish builds: kept

```ogt
fn emitter {
  doc "A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles`: its shape, emission, motion, look, rendering (billboards, meshes or 2D sprites), collision, sub-emitters and events. Every line runs the same in compute and on the CPU path. The textures, texture array, mesh model, effect material and sub-emitters it names are loaded with it (a sub-emitter that fires its parent is refused); while the array's streaming group is not resident its systems draw nothing. Loaded once per path."
  guide "loading §5"
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/sparks.ogt`."
  returns Result.ParticleEmitter "Refused for a path not in the bundle, or a texture, texture array, model or sub-emitter it cannot load."
}
```

<sub>Module `load` line 282</sub>
