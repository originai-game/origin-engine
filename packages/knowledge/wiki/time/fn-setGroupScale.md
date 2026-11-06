<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`time`](../time.md) · [index](../index.md)

<a id="fn-setGroupScale"></a>

### `og.time.setGroupScale`

Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. Entities with og.TimeGroup in it animate, tween and walk at the new pace from the system after the call; physics stays on the global scale. Outside a system the first call registers og.TimeGroups; inside one it must be registered already (name og.TimeGroup or og.TimeGroups in a system's reads or writes, or call it once in setup).

Also known as: local time scale, per-group slow motion, bullet time for some entities, freeze group, Godot process mode

```ts
og.time.setGroupScale(world: World, group: number, scale: number): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `group` | `number` | no | The group, 0 to 15. |
| `scale` | `number` | no | 0 to 10: 1 is the global pace. |

**Returns** `Result<void>`: Refused for a group past 15, a scale outside 0..10, or inside a system before og.TimeGroups is registered.

Binding symbol `og_time_set_group_scale` · publish builds: kept

```ogt
fn setGroupScale {
  doc "Set one time group's scale on top of the global one: 0.25 a quarter speed, 0 frozen, 1 the global pace. Entities with og.TimeGroup in it animate, tween and walk at the new pace from the system after the call; physics stays on the global scale. Outside a system the first call registers og.TimeGroups; inside one it must be registered already (name og.TimeGroup or og.TimeGroups in a system's reads or writes, or call it once in setup)."
  aliases "local time scale, per-group slow motion, bullet time for some entities, freeze group, Godot process mode"
  param world World "The world."
  param group u32 "The group, 0 to 15."
  param scale f32 "0 to 10: 1 is the global pace."
  returns Result.Void "Refused for a group past 15, a scale outside 0..10, or inside a system before og.TimeGroups is registered."
}
```

<sub>Module `time` line 70</sub>
