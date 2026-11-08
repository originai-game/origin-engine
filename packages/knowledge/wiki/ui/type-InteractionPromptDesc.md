<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-InteractionPromptDesc"></a>

### `InteractionPromptDesc` (desc struct)

An interaction prompt over a thing.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | What it is over. |
| `height` | `number` | `1.2` | — | `m` | — | World units above the target's point. |
| `action` | `string` | `""` | — | — | — | The action whose glyph it shows, its binding on the device used last: a gameplay action's (og.input.glyph), else a UI action's (og.ui.glyph). |
| `verb` | `string` | `""` | — | — | — | What it does (Open, Talk, Pick up). |
| `name` | `string` | `""` | — | — | — | What it is (Chest); empty is none. |
| `hold` | `number \| null` | `null` | — | `ratio` | — | A hold's progress 0 to 1: a ring fills round the glyph; null is a press. |
| `disabled` | `boolean` | `false` | — | — | — | It cannot be used now (locked, out of reach): dimmed, with reason. |
| `reason` | `string` | `""` | — | — | — | Why it cannot (Locked). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | Hidden past this; 0 is 6. |

```ogt
struct InteractionPromptDesc {
  doc "An interaction prompt over a thing."
  desc true
  field target WorldTarget "What it is over."
  field height f32 "World units above the target's point." {
    default 1.2
    unit m
  }
  field action String "The action whose glyph it shows, its binding on the device used last: a gameplay action's (og.input.glyph), else a UI action's (og.ui.glyph)."
  field verb String "What it does (Open, Talk, Pick up)."
  field name String "What it is (Chest); empty is none."
  field hold Option.f32 "A hold's progress 0 to 1: a ring fills round the glyph; null is a press." {
    unit ratio
  }
  field disabled bool "It cannot be used now (locked, out of reach): dimmed, with reason."
  field reason String "Why it cannot (Locked)."
  field maxDistance f32 "Hidden past this; 0 is 6." {
    unit m
    min 0
  }
}
```

<sub>Module `ui` line 6478</sub>
