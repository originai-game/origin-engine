<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-ActionDesc"></a>

### `ActionDesc` (desc struct)

A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. Actions are read once per tick, before the game's systems, from og.Input, og.Pointers and og.Gamepad, so they replay with them; the player rebinds them (og.input.rebind, og.input.startCapture) and a settings screen lists them (og.input.actions).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `ActionKind` | `"button"` | — | — | — | Button, one axis or two. |
| `context` | `string` | `"gameplay"` | — | — | — | Its context (og.input.setContexts): `gameplay`, `menu`, `vehicle`, or the game's own. |
| `label` | `string` | `""` | — | — | — | What a settings screen and prompts call it (empty: its name). |
| `bindings` | `readonly InputBinding[]` | `[]` | — | — | — | The default bindings, in slot order: the player's rebinding replaces a slot, reset brings these back. |
| `threshold` | `number` | `0.5` | 0.01 … 1 | — | — | Where an analog value presses it (a button; and an axis's down). |
| `deadzone` | `number` | `0` | 0 … 0.95 | — | — | A dead zone on its combined value (radial for two axes), after each binding's own. |
| `holdMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it is down before `held` (0: never), counted in ticks. |
| `locked` | `boolean` | `false` | — | — | — | The player cannot rebind it (pause, a debug key). |
| `toggle` | `boolean` | `false` | — | — | — | A button whose press turns it on and the next press off (a hold's alternative, XAG 107); the player may change it (og.input.setToggle). |

```ogt
struct ActionDesc {
  doc "A gameplay action: a name the game reads instead of keys (`jump`, `move`, `look`), its kind and default bindings over keys, the mouse, pads and touch. Actions are read once per tick, before the game's systems, from og.Input, og.Pointers and og.Gamepad, so they replay with them; the player rebinds them (og.input.rebind, og.input.startCapture) and a settings screen lists them (og.input.actions)."
  desc true
  field name String "Its name."
  field kind ActionKind "Button, one axis or two."
  field context String "Its context (og.input.setContexts): `gameplay`, `menu`, `vehicle`, or the game's own." {
    default "gameplay"
  }
  field label String "What a settings screen and prompts call it (empty: its name)."
  field bindings List.InputBinding "The default bindings, in slot order: the player's rebinding replaces a slot, reset brings these back."
  field threshold f32 "Where an analog value presses it (a button; and an axis's down)." {
    default 0.5
    min 0.01
    max 1
  }
  field deadzone f32 "A dead zone on its combined value (radial for two axes), after each binding's own." {
    min 0
    max 0.95
  }
  field holdMs f32 "How long it is down before `held` (0: never), counted in ticks." {
    min 0
    unit ms
  }
  field locked bool "The player cannot rebind it (pause, a debug key)."
  field toggle bool "A button whose press turns it on and the next press off (a hold's alternative, XAG 107); the player may change it (og.input.setToggle)."
}
```

<sub>`api.ogt` line 2094</sub>
