<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-semantics"></a>

### `og.ui.semantics`

Give the element built just before (a container just closed, or a widget) a role, a name, a value and states for the accessibility tree and the probe's dump (dump_ui's role, name, value, states). Components built from containers call it so a screen reader meets a menu, a tree or a table, not rows of buttons. Answers false, with a frame warning, when no element was built before it or the role is unknown.

```ts
og.ui.semantics(desc?: SemanticsDesc): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `SemanticsDesc` | yes | Its role, name, value and states. |

**Returns** `boolean`.

Binding symbol `og_ui_semantics` · publish builds: kept

```ogt
fn semantics {
  doc "Give the element built just before (a container just closed, or a widget) a role, a name, a value and states for the accessibility tree and the probe's dump (dump_ui's role, name, value, states). Components built from containers call it so a screen reader meets a menu, a tree or a table, not rows of buttons. Answers false, with a frame warning, when no element was built before it or the role is unknown."
  param desc SemanticsDesc "Its role, name, value and states."
  returns bool
}
```

<sub>Module `ui` line 7285</sub>
