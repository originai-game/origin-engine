<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-mouse"></a>

### `og.input.mouse`

How the host reads the mouse: raw input asked for, granted or refused by the browser.

```ts
og.input.mouse(world: World): MouseInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `MouseInfo`.

Binding symbol `og_input_mouse` · publish builds: kept

```ogt
fn mouse {
  doc "How the host reads the mouse: raw input asked for, granted or refused by the browser."
  param world World "The world."
  returns MouseInfo
}
```

<sub>Module `input` line 575</sub>
