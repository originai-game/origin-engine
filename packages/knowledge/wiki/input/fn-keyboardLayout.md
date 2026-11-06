<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-keyboardLayout"></a>

### `og.input.keyboardLayout`

The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout).

```ts
og.input.keyboardLayout(): readonly KeyLabel[]
```

**Returns** `readonly KeyLabel[]`.

Binding symbol `og_input_keyboard_layout` · publish builds: kept

```ogt
fn keyboardLayout {
  doc "The keys the layout prints as og.input.setKeyboardLayout last set them, sorted by code (empty: the US layout)."
  returns List.KeyLabel
}
```

<sub>Module `input` line 508</sub>
