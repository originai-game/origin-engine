<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-openPath"></a>

### `og.ui.openPath`

The full path of the innermost open container (an open row, panel, anchor or world anchor), as bounds, visualState, forceState, playTimeline and the probe name it; empty at the top level. A component reads its root's path right after opening it, to name its parts in those calls.

```ts
og.ui.openPath(): string
```

**Returns** `string`.

Binding symbol `og_ui_open_path` · publish builds: kept

```ogt
fn openPath {
  doc "The full path of the innermost open container (an open row, panel, anchor or world anchor), as bounds, visualState, forceState, playTimeline and the probe name it; empty at the top level. A component reads its root's path right after opening it, to name its parts in those calls."
  returns String
}
```

<sub>Module `ui` line 2338</sub>
