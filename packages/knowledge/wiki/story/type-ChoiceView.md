<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-ChoiceView"></a>

### `ChoiceView` (struct)

A choice the node offers (hidden ones are left out).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | What og.story.choose takes: its place among the node's choices, hidden ones counted. |
| `id` | `string` | `""` | — | — | — | Its id within the node. |
| `text` | `string` | `""` | — | — | — | Its text, resolved in the active locale. |
| `spans` | `readonly TextSpan[]` | `[]` | — | — | — | Yarn attributes on the plain text, in Unicode scalar offsets; unknown attributes are preserved. |
| `key` | `string` | `""` | — | — | — | Its i18n key. |
| `args` | `unknown` | `null` | — | — | — | The arguments its text was resolved with. |
| `enabled` | `boolean` | `false` | — | — | — | Whether it can be chosen now (its `if` holds and a passive check passes). |
| `reason` | `string` | `""` | — | — | — | Why it is disabled, resolved in the active locale (empty without a `reason`). |
| `reasonKey` | `string` | `""` | — | — | — | The reason's i18n key. |
| `reasonArgs` | `unknown` | `null` | — | — | — | The arguments the reason was resolved with. |
| `visited` | `boolean` | `false` | — | — | — | Chosen before in this conversation (drawn muted). |
| `kind` | `string` | `""` | — | — | — | The game's word for its icon: leave, trade, attack, lie... |
| `check` | `CheckView \| null` | `null` | — | — | — | Its check, if it has one. |

```ogt
struct ChoiceView {
  doc "A choice the node offers (hidden ones are left out)."
  field index u32 "What og.story.choose takes: its place among the node's choices, hidden ones counted."
  field id String "Its id within the node."
  field text String "Its text, resolved in the active locale."
  field spans List.TextSpan "Yarn attributes on the plain text, in Unicode scalar offsets; unknown attributes are preserved."
  field key String "Its i18n key."
  field args Json "The arguments its text was resolved with."
  field enabled bool "Whether it can be chosen now (its `if` holds and a passive check passes)."
  field reason String "Why it is disabled, resolved in the active locale (empty without a `reason`)."
  field reasonKey String "The reason's i18n key."
  field reasonArgs Json "The arguments the reason was resolved with."
  field visited bool "Chosen before in this conversation (drawn muted)."
  field kind String "The game's word for its icon: leave, trade, attack, lie..."
  field check Option.CheckView "Its check, if it has one."
}
```

<sub>Module `story` line 112</sub>
