<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-paused"></a>

### `og.ui.paused`

A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had.

```ts
og.ui.paused(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_paused` · publish builds: kept

```ogt
fn paused {
  doc "A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had."
  param world World "The world."
  returns bool
}
```

<sub>Module `ui` line 3939</sub>
