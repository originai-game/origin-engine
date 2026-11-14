<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-setNavigation"></a>

### `og.ui.setNavigation`

Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick.

```ts
og.ui.setNavigation(on: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `on` | `boolean` | no | On or off. |

**Returns** `void`.

Binding symbol `og_ui_set_navigation` · publish builds: kept

```ogt
fn setNavigation {
  doc "Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick."
  param on bool "On or off."
  returns Void
}
```

<sub>Module `ui` line 755</sub>
