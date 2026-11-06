<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-BarkDesc"></a>

### `BarkDesc` (desc struct)

A bark to ask for.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | Who barks: its og.story.barks entry, `self.` in the bark's conditions and arguments. |
| `topic` | `string` | `""` | — | — | — | The topic: one of the bark's `topic`s, such as idle or alarm. |
| `dialogue` | `string` | `""` | — | — | — | Only this dialogue's barks (empty: every loaded dialogue's). |

```ogt
struct BarkDesc {
  doc "A bark to ask for."
  desc true
  field entity Entity "Who barks: its og.story.barks entry, `self.` in the bark's conditions and arguments."
  field topic String "The topic: one of the bark's `topic`s, such as idle or alarm."
  field dialogue String "Only this dialogue's barks (empty: every loaded dialogue's)."
}
```

<sub>Module `story` line 167</sub>
