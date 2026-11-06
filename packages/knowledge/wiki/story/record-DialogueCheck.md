<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-DialogueCheck"></a>

### `og.DialogueCheck` (event)

An active check was rolled (d20 plus the stat against the target): reveal the roll. Sent the tick after.

Token `og.DialogueCheck`, registered under the schema name `og.DialogueCheck`; its value type is `DialogueCheck`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `conversation` | `number` | `0` | — | — | — | The conversation. |
| `stat` | `string` | `""` | — | — | — | The variable read as the stat. |
| `target` | `number` | `0` | — | — | — | What the roll plus the stat had to reach. |
| `roll` | `number` | `0` | — | — | — | The d20, 1 to 20. |
| `passed` | `boolean` | `false` | — | — | — | Whether roll + stat reached the target. |

```ogt
event DialogueCheck {
  doc "An active check was rolled (d20 plus the stat against the target): reveal the roll. Sent the tick after."
  field conversation u32 "The conversation."
  field stat String "The variable read as the stat."
  field target f32 "What the roll plus the stat had to reach."
  field roll u32 "The d20, 1 to 20."
  field passed bool "Whether roll + stat reached the target."
}
```

<sub>Module `story` line 31</sub>
