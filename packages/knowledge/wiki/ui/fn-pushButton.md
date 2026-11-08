<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-pushButton"></a>

### `og.ui.pushButton`

A button (docs/ui.md §2, Controls): primary, secondary, ghost (outlined), quiet (text) or danger, small, medium or large, with icons, a badge, a loading spinner and a disabled reason. It lifts under the pointer and the focus and sinks as it is pressed (its interaction group, on the style's motion), and the style dresses it as `Button.base[<variant>][hover|pressed|focused|disabled][selected]` (a version 1 style's button art included), its label in the ink that reads on its fill. A click is a press and a release inside it, Enter, Space or the pad's A while it has the focus. Role button (pressed while selected, busy while loading). Probe: `<id>`, `<id>/Label`, `<id>/Icon`, `<id>/Badge`.

```ts
og.ui.pushButton(id: string, label: string, desc?: PushButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `label` | `string` | no | Its label. |
| `desc` | `PushButtonDesc` | yes | Its role, size, icons and states. |

**Returns** `Response`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn pushButton {
  doc "A button (docs/ui.md §2, Controls): primary, secondary, ghost (outlined), quiet (text) or danger, small, medium or large, with icons, a badge, a loading spinner and a disabled reason. It lifts under the pointer and the focus and sinks as it is pressed (its interaction group, on the style's motion), and the style dresses it as `Button.base[<variant>][hover|pressed|focused|disabled][selected]` (a version 1 style's button art included), its label in the ink that reads on its fill. A click is a press and a release inside it, Enter, Space or the pad's A while it has the focus. Role button (pressed while selected, busy while loading). Probe: `<id>`, `<id>/Label`, `<id>/Icon`, `<id>/Badge`."
  host false
  param id String "Its id."
  param label String "Its label."
  param desc PushButtonDesc "Its role, size, icons and states."
  returns Response
}
```

<sub>Module `ui` line 3451</sub>
