<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openTransition"></a>

### `og.ui.openTransition`

Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves.

```ts
og.ui.openTransition(id: string, show: boolean, desc?: TransitionDesc): Presence
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `show` | `boolean` | no | Whether it should be shown. |
| `desc` | `TransitionDesc` | yes | How it moves. |

**Returns** `Presence`.

Binding symbol `og_ui_open_transition` · publish builds: kept

```ogt
fn openTransition {
  doc "Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves."
  param id String "Its id."
  param show bool "Whether it should be shown."
  param desc TransitionDesc "How it moves."
  returns Presence
}
```

<sub>Module `ui` line 1273</sub>
