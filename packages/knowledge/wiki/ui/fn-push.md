<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-push"></a>

### `og.ui.push`

Push a screen onto the layer stack (docs/ui.md §8.1), over the layers of its kind and under those of a later kind (hud, menu, modal, popover, toast, tooltip, system); the active layer remembers its focus. Callable anywhere (systems, commands, start); the stack is world state, so snapshots carry it.

```ts
og.ui.push(world: World, screen: string, vm?: unknown): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |
| `vm` | `unknown` | yes | Its viewmodel (null: its document's first fixture, else an empty object). |

**Returns** `Result<number>`: The layer's serial. Refused for a name no screen document has.

Binding symbol `og_ui_push` · publish builds: kept

```ogt
fn push {
  doc "Push a screen onto the layer stack (docs/ui.md §8.1), over the layers of its kind and under those of a later kind (hud, menu, modal, popover, toast, tooltip, system); the active layer remembers its focus. Callable anywhere (systems, commands, start); the stack is world state, so snapshots carry it."
  param world World "The world."
  param screen String "The screen's name."
  param vm Json "Its viewmodel (null: its document's first fixture, else an empty object)." {
    optional true
  }
  returns Result.u32 "The layer's serial. Refused for a name no screen document has."
}
```

<sub>Module `ui` line 8141</sub>
