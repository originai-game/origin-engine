<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiFrameMessage"></a>

### `UiFrameMessage` (struct)

Something the document of a remote `<iframe>` did (docs/ui.md §2.1 Frames, HTML's `postMessage` as a frame's events): it never reaches docEvents, so the game reads it here and decides what it does. A gameplay effect then goes through og.input.intent, so replays carry it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mount` | `string` | `""` | — | — | — | The embedding document's mount. |
| `frame` | `string` | `""` | — | — | — | The `<iframe>`'s path. |
| `origin` | `string` | `""` | — | — | — | The origin its document was loaded from, such as `https://events.example.com`: check it before acting. |
| `kind` | `string` | `""` | — | — | — | click, changed, select, emit, action or set, as a document event's kind. |
| `name` | `string` | `""` | — | — | — | The element's id (click), the prop (changed), the event or action's name. |
| `path` | `string` | `""` | — | — | — | The element's path inside the frame, after the iframe's. |
| `value` | `unknown` | `null` | — | — | — | The new value (changed, set), the row's index (select); null otherwise. |

```ogt
struct UiFrameMessage {
  doc "Something the document of a remote `<iframe>` did (docs/ui.md §2.1 Frames, HTML's `postMessage` as a frame's events): it never reaches docEvents, so the game reads it here and decides what it does. A gameplay effect then goes through og.input.intent, so replays carry it."
  field mount String "The embedding document's mount."
  field frame String "The `<iframe>`'s path."
  field origin String "The origin its document was loaded from, such as `https://events.example.com`: check it before acting."
  field kind String "click, changed, select, emit, action or set, as a document event's kind."
  field name String "The element's id (click), the prop (changed), the event or action's name."
  field path String "The element's path inside the frame, after the iframe's."
  field value Json "The new value (changed, set), the row's index (select); null otherwise."
}
```

<sub>Module `ui` line 1652</sub>
