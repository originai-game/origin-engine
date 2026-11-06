<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`anim`](../anim.md) · [index](../index.md)

<a id="fn-rig2dInfo"></a>

### `og.anim.rig2dInfo`

What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load).

```ts
og.anim.rig2dInfo(world: World, rig: Rig2D): Result<Rig2DInfo>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose animation owns the rig. |
| `rig` | `Rig2D` | no | The rig. |

**Returns** `Result<Rig2DInfo>`: Refused for a handle og.load.rig2d did not answer.

Binding symbol `og_anim_rig2d_info` · publish builds: kept

```ogt
fn rig2dInfo {
  doc "What a loaded 2D rig holds: its bones, slots, IK constraints and clips, by the handle Skeleton2D.rig keeps (og.load.rig2d answers the same on load)."
  param world World "The world whose animation owns the rig."
  param rig Rig2D "The rig."
  returns Result.Rig2DInfo "Refused for a handle og.load.rig2d did not answer."
}
```

<sub>`api.ogt` line 12337</sub>
