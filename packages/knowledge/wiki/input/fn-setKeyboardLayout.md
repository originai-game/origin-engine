<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setKeyboardLayout"></a>

### `og.input.setKeyboardLayout`

Set the keys the player's keyboard layout prints, for glyphs: bindings still name physical keys (KeyboardEvent.code), so only their glyphs change (Z Q S D on AZERTY). The web shell sets it; empty is the US layout (native runs). The host's, one for the page: no world state.

See `input §2`.

```ts
og.input.setKeyboardLayout(labels: readonly KeyLabel[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `labels` | `readonly KeyLabel[]` | no | Each key's printed label, in place of the last list. |

**Returns** `void`.

Binding symbol `og_input_set_keyboard_layout` · publish builds: kept

```ogt
fn setKeyboardLayout {
  doc "Set the keys the player's keyboard layout prints, for glyphs: bindings still name physical keys (KeyboardEvent.code), so only their glyphs change (Z Q S D on AZERTY). The web shell sets it; empty is the US layout (native runs). The host's, one for the page: no world state."
  guide "input §2"
  param labels List.KeyLabel "Each key's printed label, in place of the last list."
  returns Void
}
```

<sub>Module `input` line 556</sub>
