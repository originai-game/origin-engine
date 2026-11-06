<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-emitter"></a>

### `og.load.emitter`

A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles`: shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision (`collision <bounce> <friction> [depth | colliders]`: off the drawn depth, or off the scene's colliders on the GPU), billboard, mesh (`render mesh box | sphere | tetrahedron | "<model id>"`: each particle a small lit shape that tumbles with its rotation) or 2D sprite rendering, `lit true` (billboards and meshes shaded by the sun, the sky and local lights), `distortion <0..1>` (billboards that bend the scene behind them: heat haze, shock rings), `subEmitter <birth | death | collision> "<emitter id>" <count> [<inherit velocity>]` (up to 4: another emitter's particles born where this one's are born, die or bounce; one level deep) and `events <birth | death | collision>...` (og.ParticleEvent: its systems simulate in the tick). A sprite-mode emitter may draw from a texture array instead (`array "<id>" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures, array, mesh model and sub-emitters it names are loaded with it (a sub-emitter that fires its parent is refused); while the array's streaming group is not resident its systems draw nothing. Loaded once per path.

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
  doc "A particle emitter asset for ParticleSystem.emitter, from a cooked `.ogt` of `type particles`: shape, rate and bursts, lifetime, speed, size and colour over life, gravity, drag, noise, a texture and sprite sheet, blending, collision (`collision <bounce> <friction> [depth | colliders]`: off the drawn depth, or off the scene's colliders on the GPU), billboard, mesh (`render mesh box | sphere | tetrahedron | \"<model id>\"`: each particle a small lit shape that tumbles with its rotation) or 2D sprite rendering, `lit true` (billboards and meshes shaded by the sun, the sky and local lights), `distortion <0..1>` (billboards that bend the scene behind them: heat haze, shock rings), `subEmitter <birth | death | collision> \"<emitter id>\" <count> [<inherit velocity>]` (up to 4: another emitter's particles born where this one's are born, die or bounce; one level deep) and `events <birth | death | collision>...` (og.ParticleEvent: its systems simulate in the tick). A sprite-mode emitter may draw from a texture array instead (`array \"<id>\" [<layers>] [off | life <cycles> | fps <n> | random]`: each particle shows a layer by its age, at a rate, or one at random for its life; one draw per array whatever the layers; `round true` fades each particle to the soft dot's round edge). The textures, array, mesh model and sub-emitters it names are loaded with it (a sub-emitter that fires its parent is refused); while the array's streaming group is not resident its systems draw nothing. Loaded once per path."
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/sparks.ogt`."
  returns Result.ParticleEmitter "Refused for a path not in the bundle, or a texture, texture array, model or sub-emitter it cannot load."
}
```

<sub>`api.ogt` line 8176</sub>
