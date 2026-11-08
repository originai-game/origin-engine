<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiDocEvent"></a>

### `UiDocEvent` (struct)

Something a document did (docs/ui.md §2.1): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back is `emit`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mount` | `string` | `""` | — | — | — | The document's mount: the screen's name, or a component instance's path. |
| `path` | `string` | `""` | — | — | — | The element's path. |
| `rowKey` | `string` | `""` | — | — | — | Stable collection key of the innermost each row; empty outside a row. Never an index. |
| `kind` | `string` | `""` | — | — | — | click, changed, select, emit, action, set, back, or activate and deactivate (a layer became or stopped being the active one). |
| `name` | `string` | `""` | — | — | — | The element's id (click), the prop (changed), the list (select), the event or action's name, or the viewmodel path a `set` wrote. |
| `value` | `unknown` | `null` | — | — | — | The new value (changed, set), the row's index (select); null otherwise. |

```ogt
struct UiDocEvent {
  doc "Something a document did (docs/ui.md §2.1): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back is `emit`."
  field mount String "The document's mount: the screen's name, or a component instance's path."
  field path String "The element's path."
  field rowKey String "Stable collection key of the innermost each row; empty outside a row. Never an index."
  field kind String "click, changed, select, emit, action, set, back, or activate and deactivate (a layer became or stopped being the active one)."
  field name String "The element's id (click), the prop (changed), the list (select), the event or action's name, or the viewmodel path a `set` wrote."
  field value Json "The new value (changed, set), the row's index (select); null otherwise."
}
```

<sub>Module `ui` line 7897</sub>
