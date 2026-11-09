<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-a11y"></a>

### `og.ui.a11y`

The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements.

```ts
og.ui.a11y(): A11yTree
```

**Returns** `A11yTree`.

Binding symbol `og_ui_a11y` · publish builds: kept

```ogt
fn a11y {
  doc "The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements."
  returns A11yTree
}
```

<sub>Module `ui` line 736</sub>
