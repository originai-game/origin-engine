<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-screen"></a>

### `og.ui.screen`

Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots after this call (og.ui.openSlot). Its bindings read the viewmodel every tick, a Capitalised root (`Wallet.gold`) the resource as of the last tick; two-way controls write into it (each write is a `set` event). Layers code does not mount are built at the frame's end with the viewmodel they have.

```ts
og.ui.screen<S extends ScreenName>(name: S, vm?: ScreenProps<S> | null): ScreenResponse<S>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `S` | no | The screen's name (its `name` line); a name `og ui types` declared types its viewmodel and events. |
| `vm` | `ScreenProps<S> \| null` | yes | Its viewmodel: an object (null keeps the one it has). |

**Returns** `ScreenResponse<S>`.

Binding symbol `og_ui_screen` · publish builds: kept

```ogt
fn screen {
  doc "Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots after this call (og.ui.openSlot). Its bindings read the viewmodel every tick, a Capitalised root (`Wallet.gold`) the resource as of the last tick; two-way controls write into it (each write is a `set` event). Layers code does not mount are built at the frame's end with the viewmodel they have."
  generic S ScreenName
  param name S "The screen's name (its `name` line); a name `og ui types` declared types its viewmodel and events."
  param vm Option.ScreenProps.S "Its viewmodel: an object (null keeps the one it has)." {
    optional true
  }
  returns ScreenResponse.S
}
```

<sub>Module `ui` line 8014</sub>
