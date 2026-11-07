<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-back"></a>

### `og.ui.back`

Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through.

```ts
og.ui.back(): boolean
```

**Returns** `boolean`.

Binding symbol `og_ui_back` · publish builds: kept

```ogt
fn back {
  doc "Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through."
  returns bool
}
```

<sub>Module `ui` line 1570</sub>
