<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-humanoid"></a>

### `og.anim.humanoid`

The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it.

```ts
og.anim.humanoid(world: World, desc?: HumanoidDesc): Result<Humanoid>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the skeleton. |
| `desc` | `HumanoidDesc` | yes | The skeleton and its rig. |

**Returns** `Result<Humanoid>`: Refused for an unregistered skeleton or rig, or a rig naming a node the skeleton lacks.

Binding symbol `og_anim_humanoid` · publish builds: kept

```ogt
fn humanoid {
  doc "The standard humanoid bones of a skeleton, as retargeting sees them: to check a rig before retargeting onto it."
  param world World "The world whose animation owns the skeleton."
  param desc HumanoidDesc "The skeleton and its rig."
  returns Result.Humanoid "Refused for an unregistered skeleton or rig, or a rig naming a node the skeleton lacks."
}
```

<sub>Module `anim` line 618</sub>
