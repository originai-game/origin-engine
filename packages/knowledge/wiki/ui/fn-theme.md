<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-theme"></a>

### `og.ui.theme`

The style values the UI draws with.

```ts
og.ui.theme(world: World): UiTheme
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `UiTheme`.

Binding symbol `og_ui_theme` · publish builds: kept

```ogt
fn theme {
  doc "The style values the UI draws with."
  param world World "The world."
  returns UiTheme
}
```

<sub>Module `ui` line 573</sub>
