<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-KeyBindingRowDesc"></a>

### `KeyBindingRowDesc` (desc struct)

A controls screen's row for one action: its name, its bindings on a device, and their capture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action (og.input.defineAction). |
| `label` | `string` | `""` | — | — | — | What it shows; empty is the action's label. |
| `device` | `InputDevice \| null` | `null` | — | — | — | Whose bindings it shows and captures: keyboard and mouse, or a pad (its family's glyphs); null is the device used last (it follows live). |
| `slots` | `number` | `2` | 1 … 4 | — | — | How many bindings it shows on that device (primary, secondary). |
| `reset` | `boolean` | `true` | — | — | — | A reset button while the player changed it. |
| `timeoutMs` | `number` | `8000` | — | `ms` | — | A capture cancels itself after this long (0: never). |
| `waitingLabel` | `string` | `""` | — | — | — | What a slot says while it waits; empty is `Press a key…` (`Press a button…` on a pad). |
| `cancelHint` | `string` | `""` | — | — | — | The hint beside it; empty is `Esc to cancel` (the pad's back glyph on a pad). |
| `emptyLabel` | `string` | `""` | — | — | — | What an unbound slot says; empty is `—`. |
| `conflictLabel` | `string` | `""` | — | — | — | The conflict line, `{control}` and `{action}` filled; empty is `{control} is used by {action}`. |
| `swapLabel` | `string` | `""` | — | — | — | The swap button; empty is `Swap`. |
| `replaceLabel` | `string` | `""` | — | — | — | The replace button; empty is `Replace`. |
| `cancelLabel` | `string` | `""` | — | — | — | The cancel button; empty is `Cancel`. |
| `resetLabel` | `string` | `""` | — | — | — | The reset button's name; empty is `Reset to default`. |

```ogt
struct KeyBindingRowDesc {
  doc "A controls screen's row for one action: its name, its bindings on a device, and their capture."
  desc true
  field action String "The action (og.input.defineAction)."
  field label String "What it shows; empty is the action's label."
  field device Option.InputDevice "Whose bindings it shows and captures: keyboard and mouse, or a pad (its family's glyphs); null is the device used last (it follows live)."
  field slots u32 "How many bindings it shows on that device (primary, secondary)." {
    default 2
    min 1
    max 4
  }
  field reset bool "A reset button while the player changed it." {
    default true
  }
  field timeoutMs f32 "A capture cancels itself after this long (0: never)." {
    default 8000
    unit ms
  }
  field waitingLabel String "What a slot says while it waits; empty is `Press a key…` (`Press a button…` on a pad)."
  field cancelHint String "The hint beside it; empty is `Esc to cancel` (the pad's back glyph on a pad)."
  field emptyLabel String "What an unbound slot says; empty is `—`."
  field conflictLabel String "The conflict line, `{control}` and `{action}` filled; empty is `{control} is used by {action}`."
  field swapLabel String "The swap button; empty is `Swap`."
  field replaceLabel String "The replace button; empty is `Replace`."
  field cancelLabel String "The cancel button; empty is `Cancel`."
  field resetLabel String "The reset button's name; empty is `Reset to default`."
}
```

<sub>Module `ui` line 3860</sub>
