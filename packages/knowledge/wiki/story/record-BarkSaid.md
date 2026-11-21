<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="record-BarkSaid"></a>

### `og.BarkSaid` (event)

An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. Sent the tick after it was asked for.

Token `og.BarkSaid`, registered under the schema name `og.BarkSaid`; its value type is `BarkSaid`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barked. |
| `bark` | `string` | `""` | — | — | — | The bark's id in its dialogue. |
| `key` | `string` | `""` | — | — | — | The line's i18n key. |
| `seq` | `number` | `0` | — | — | — | The line's number (BarkView.seq). |

```ogt
event BarkSaid {
  doc "An entity barked (og.story.bark or og.story.ambient): show its line over it with og.story.barks. Sent the tick after it was asked for."
  field entity Entity "Who barked."
  field bark String "The bark's id in its dialogue."
  field key String "The line's i18n key."
  field seq u32 "The line's number (BarkView.seq)."
}
```

<sub>Module `story` line 62</sub>
