<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-UiActionTrigger"></a>

### `UiActionTrigger` (enum)

When a UI action fires.

A string: one of the variants below. The zero value is `"press"`.

| Variant | Doc |
|---|---|
| `"press"` | When a binding goes down. |
| `"release"` | When it comes up. |
| `"hold"` | When it has been held holdMs. |

```ogt
enum UiActionTrigger {
  doc "When a UI action fires."
  variant press "When a binding goes down."
  variant release "When it comes up."
  variant hold "When it has been held holdMs."
}
```

<sub>Module `ui` line 6087</sub>
