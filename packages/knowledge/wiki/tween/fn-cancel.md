<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`tween`](../tween.md) · [index](../index.md)

<a id="fn-cancel"></a>

### `og.tween.cancel`

Stop a tween where it is (no event).

Also known as: stop tween, kill tween

```ts
og.tween.cancel(world: World, tween: Tween): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tween` | `Tween` | no | The handle. |

**Returns** `boolean`: False when it was not running.

Binding symbol `og_tween_cancel` · publish builds: kept

```ogt
fn cancel {
  doc "Stop a tween where it is (no event)."
  aliases "stop tween, kill tween"
  param world World "The world."
  param tween Tween "The handle."
  returns bool "False when it was not running."
}
```

<sub>Module `tween` line 160</sub>
