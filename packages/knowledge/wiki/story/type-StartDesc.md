<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`story`](../story.md) · [index](../index.md)

<a id="type-StartDesc"></a>

### `StartDesc` (desc struct)

A conversation to start.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `dialogue` | `string` | `""` | — | — | — | The dialogue's name (StoryAsset.name). |
| `node` | `string` | `""` | — | — | — | The node to start at (empty: the dialogue's `start`). |
| `cast` | `unknown` | `null` | — | — | — | Speaker ids to the entities playing them, `{ smith: entity }`: SpeakerView.entity (for a speech bubble or a camera). |
| `vars` | `unknown` | `null` | — | — | — | Starting values for the dialogue's own `var`s, `{ mood: 2 }`. |

```ogt
struct StartDesc {
  doc "A conversation to start."
  desc true
  field dialogue String "The dialogue's name (StoryAsset.name)."
  field node String "The node to start at (empty: the dialogue's `start`)."
  field cast Json "Speaker ids to the entities playing them, `{ smith: entity }`: SpeakerView.entity (for a speech bubble or a camera)."
  field vars Json "Starting values for the dialogue's own `var`s, `{ mood: 2 }`."
}
```

<sub>`api.ogt` line 14458</sub>
