<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`physics`](../physics.md) · [index](../index.md)

<a id="fn-resizeCharacter"></a>

### `og.physics.resizeCharacter`

Resize a character capsule after checking headroom and the final volume. Keeps its feet fixed by adjusting Collider.offset, preserves body velocity and joints, and returns false without changing anything when blocked. Works in both physics profiles, inside a system or between ticks; inside a system read Transform, Parent and the controller and write Collider.

```ts
og.physics.resizeCharacter(world: World, entity: Entity, height: number, radius: number): Result<boolean>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | An entity with CharacterController or PlatformerController and a capsule Collider. |
| `height` | `number` | no | New cylindrical segment height (as Collider.height), excluding caps; finite and >= 0. |
| `radius` | `number` | no | New capsule radius; finite and > 0. |

**Returns** `Result<boolean>`: True when resized, false when blocked; refused for invalid sizes or a non-capsule/non-controller entity.

Binding symbol `og_physics_resize_character` · publish builds: kept

```ogt
fn resizeCharacter {
  doc "Resize a character capsule after checking headroom and the final volume. Keeps its feet fixed by adjusting Collider.offset, preserves body velocity and joints, and returns false without changing anything when blocked. Works in both physics profiles, inside a system or between ticks; inside a system read Transform, Parent and the controller and write Collider."
  param world World "The world."
  param entity Entity "An entity with CharacterController or PlatformerController and a capsule Collider."
  param height f32 "New cylindrical segment height (as Collider.height), excluding caps; finite and >= 0."
  param radius f32 "New capsule radius; finite and > 0."
  returns Result.bool "True when resized, false when blocked; refused for invalid sizes or a non-capsule/non-controller entity."
}
```

<sub>Module `physics` line 891</sub>
