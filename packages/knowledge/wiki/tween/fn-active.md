<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-active"></a>

### `og.tween.active`

Whether a tween is still running.

```ts
og.tween.active(world: World, tween: Tween): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tween` | `Tween` | no | The handle. |

**Returns** `boolean`.

Binding symbol `og_tween_active` · publish builds: kept

```ogt
fn active {
  doc "Whether a tween is still running."
  param world World "The world."
  param tween Tween "The handle."
  returns bool
}
```

<sub>`api.ogt` line 12534</sub>
