<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`load`](../load.md) · [index](../index.md)

<a id="fn-effect"></a>

### `og.load.effect`

An effect material for og.Trail.effect and og.Beam.effect, from a cooked `.ogt` of `type effect` (the file emitters and composed effects name with `effect`): its textures, UV motion, erosion, fresnel and hook shader, loaded with it. Loaded once per path.

See `effect-files §2`.

```ts
og.load.effect(world: World, path: string): Result<EffectMaterial>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose renderer owns it. |
| `path` | `string` | no | The asset id, such as `fx/slash.ogt`. |

**Returns** `Result<EffectMaterial>`: Refused for a path not in the bundle, not an effect, or a texture or shader it cannot load.

Binding symbol `og_load_effect` · publish builds: kept

```ogt
fn effect {
  doc "An effect material for og.Trail.effect and og.Beam.effect, from a cooked `.ogt` of `type effect` (the file emitters and composed effects name with `effect`): its textures, UV motion, erosion, fresnel and hook shader, loaded with it. Loaded once per path."
  guide "effect-files §2"
  param world World "The world whose renderer owns it."
  param path String "The asset id, such as `fx/slash.ogt`."
  returns Result.EffectMaterial "Refused for a path not in the bundle, not an effect, or a texture or shader it cannot load."
}
```

<sub>Module `load` line 330</sub>
