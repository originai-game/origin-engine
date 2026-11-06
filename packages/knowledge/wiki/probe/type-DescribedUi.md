<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`probe`](../probe.md) · [index](../index.md)

<a id="type-DescribedUi"></a>

### `DescribedUi` (struct)

One UI element as describe lists it (the tree dump_ui writes).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `parent` | `string` | `""` | — | — | — | Its parent's id. |
| `kind` | `string` | `""` | — | — | — | What made it. |
| `role` | `string` | `""` | — | — | — | Its accessibility role. |
| `text` | `string` | `""` | — | — | — | The text it shows. |
| `value` | `string` | `""` | — | — | — | Its value as text. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states. |
| `opaque` | `boolean` | `false` | — | — | — | It draws an opaque background over its whole box: the entities under it count it in their occlusion. |
| `box` | `ScreenBox` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | Its rect. |

```ogt
struct DescribedUi {
  doc "One UI element as describe lists it (the tree dump_ui writes)."
  field id String "Its id."
  field parent String "Its parent's id."
  field kind String "What made it."
  field role String "Its accessibility role."
  field text String "The text it shows."
  field value String "Its value as text."
  field states List.String "Its states."
  field opaque bool "It draws an opaque background over its whole box: the entities under it count it in their occlusion."
  field box ScreenBox "Its rect."
}
```

<sub>`api.ogt` line 24497</sub>
