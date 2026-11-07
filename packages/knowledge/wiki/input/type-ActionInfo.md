<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-ActionInfo"></a>

### `ActionInfo` (struct)

An action as a controls screen lists it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `kind` | `ActionKind` | `"button"` | — | — | — | Its kind. |
| `actionMap` | `string` | `""` | — | — | — | Its action map. |
| `label` | `string` | `""` | — | — | — | Its label (its name when it has none). |
| `bindings` | `readonly InputBinding[]` | `[]` | — | — | — | Its bindings now: the player's, else the defaults. |
| `defaults` | `readonly InputBinding[]` | `[]` | — | — | — | Its default bindings. |
| `locked` | `boolean` | `false` | — | — | — | The player cannot rebind it. |
| `toggle` | `boolean` | `false` | — | — | — | It toggles (the player's choice, else its desc's). |
| `rebound` | `boolean` | `false` | — | — | — | The player changed its bindings. |

```ogt
struct ActionInfo {
  doc "An action as a controls screen lists it."
  field name String "Its name."
  field kind ActionKind "Its kind."
  field actionMap String "Its action map."
  field label String "Its label (its name when it has none)."
  field bindings List.InputBinding "Its bindings now: the player's, else the defaults."
  field defaults List.InputBinding "Its default bindings."
  field locked bool "The player cannot rebind it."
  field toggle bool "It toggles (the player's choice, else its desc's)."
  field rebound bool "The player changed its bindings."
}
```

<sub>Module `input` line 157</sub>
