<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-PresenceSpec"></a>

### `PresenceSpec` (desc struct)

How an element arrives or leaves (docs/ui.md §3.1): the values it enters from or exits to, and how it moves between them and its own.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `values` | `readonly PropValue[]` | `[]` | — | — | — | Where it starts (entering) or ends (leaving). |
| `kind` | `TransitionKind \| null` | `null` | — | — | — | A transition kind's values in place of values: what the kind enters from and exits to. |
| `distance` | `number` | `0` | — | `cssPx` | — | A slide kind's distance (0: 24). |
| `transition` | `Transition \| null` | `null` | — | — | — | How it moves; null is the style's enter or exit token. |
| `delayMs` | `number` | `0` | — | `ms` | — | Wait this long before moving (a hand-built stagger). |

```ogt
struct PresenceSpec {
  doc "How an element arrives or leaves (docs/ui.md §3.1): the values it enters from or exits to, and how it moves between them and its own."
  desc true
  field values List.PropValue "Where it starts (entering) or ends (leaving)."
  field kind Option.TransitionKind "A transition kind's values in place of values: what the kind enters from and exits to."
  field distance f32 "A slide kind's distance (0: 24)." {
    unit cssPx
  }
  field transition Option.Transition "How it moves; null is the style's enter or exit token."
  field delayMs f32 "Wait this long before moving (a hand-built stagger)." {
    unit ms
  }
}
```

<sub>Module `ui` line 4763</sub>
