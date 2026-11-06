<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-squash"></a>

### `og.tween.squash`

Squash and stretch the entity's og.Transform.scale (cancels its earlier squash).

```ts
og.tween.squash(world: World, desc?: SquashDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SquashDesc` | yes | The entity and the impact. |

**Returns** `Result<Tween>`: Refused for an entity without og.Transform.

Binding symbol `og_tween_squash` · publish builds: kept

```ogt
fn squash {
  doc "Squash and stretch the entity's og.Transform.scale (cancels its earlier squash)."
  param world World "The world."
  param desc SquashDesc "The entity and the impact."
  returns Result.Tween "Refused for an entity without og.Transform."
}
```

<sub>`api.ogt` line 12549</sub>
