<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-splitView"></a>

### `og.ui.splitView`

Two panes with a divider a drag moves and a press of the focused divider steps.

```ts
og.ui.splitView(id: string, desc: SplitViewDesc, start: UiBuild, end: UiBuild): SplitResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SplitViewDesc` | no | Direction, ratio and limits. |
| `start` | `UiBuild` | no | Builds the start pane. |
| `end` | `UiBuild` | no | Builds the end pane. |

**Returns** `SplitResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn splitView {
  doc "Two panes with a divider a drag moves and a press of the focused divider steps."
  host false
  param id String "Its id."
  param desc SplitViewDesc "Direction, ratio and limits."
  param start UiBuild "Builds the start pane."
  param end UiBuild "Builds the end pane."
  returns SplitResult
}
```

<sub>`api.ogt` line 18690</sub>
