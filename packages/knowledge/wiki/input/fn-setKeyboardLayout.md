<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="fn-setKeyboardLayout"></a>

### `og.input.setKeyboardLayout`

The keys the player's keyboard layout prints, for glyphs: the web shell sets it from navigator.keyboard.getLayoutMap() (Chrome, Edge), and where the browser has none from the characters the player's unmodified keys type as they are pressed (Firefox, Safari: a key reads as on a US layout until it is pressed once). Bindings still name physical keys (KeyboardEvent.code), so WASD is the same four keys on every layout; only their glyphs change (Z Q S D on AZERTY). Empty is the US layout (native runs). The host's, one for the page: no world state.

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
  doc "The keys the player's keyboard layout prints, for glyphs: the web shell sets it from navigator.keyboard.getLayoutMap() (Chrome, Edge), and where the browser has none from the characters the player's unmodified keys type as they are pressed (Firefox, Safari: a key reads as on a US layout until it is pressed once). Bindings still name physical keys (KeyboardEvent.code), so WASD is the same four keys on every layout; only their glyphs change (Z Q S D on AZERTY). Empty is the US layout (native runs). The host's, one for the page: no world state."
  param labels List.KeyLabel "Each key's printed label, in place of the last list."
  returns Void
}
```

<sub>Module `input` line 502</sub>
