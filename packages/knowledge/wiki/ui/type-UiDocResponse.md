<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiDocResponse"></a>

### `UiDocResponse` (struct)

What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its root's path; empty when it was not built. |
| `events` | `readonly UiDocEvent[]` | `[]` | — | — | — | Its events. |
| `props` | `unknown` | `null` | — | — | — | A component's props after its writes (a screen's viewmodel is og.ui.screenVm's). |

```ogt
struct UiDocResponse {
  doc "What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes."
  field path String "Its root's path; empty when it was not built."
  field events List.UiDocEvent "Its events."
  field props Json "A component's props after its writes (a screen's viewmodel is og.ui.screenVm's)."
}
```

<sub>Module `ui` line 8003</sub>
