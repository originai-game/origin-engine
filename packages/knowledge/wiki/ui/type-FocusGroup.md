<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-FocusGroup"></a>

### `FocusGroup` (desc struct)

A container whose focusable controls are one group for keys and pads (docs/ui.md §2.1): arrows move among its members first. At its edge a group that wraps goes round to its far side; one that does not leaves for the nearest control outside that way (the group around it first) and stays put when there is none. The zero value neither wraps nor remembers.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `wrap` | `boolean` | `false` | — | — | — | At its edge the focus wraps round inside it. |
| `memory` | `boolean` | `false` | — | — | — | Focus coming into it from outside lands on the member it last had. |

```ogt
struct FocusGroup {
  doc "A container whose focusable controls are one group for keys and pads (docs/ui.md §2.1): arrows move among its members first. At its edge a group that wraps goes round to its far side; one that does not leaves for the nearest control outside that way (the group around it first) and stays put when there is none. The zero value neither wraps nor remembers."
  desc true
  field wrap bool "At its edge the focus wraps round inside it."
  field memory bool "Focus coming into it from outside lands on the member it last had."
}
```

<sub>Module `ui` line 2566</sub>
