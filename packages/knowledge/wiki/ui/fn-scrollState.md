<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-scrollState"></a>

### `og.ui.scrollState`

A scroll container's offset, view and whole content, by its full path (og.ui.lastPath after closing it): what a component virtualises its rows by, since og.ui.bounds is cut to the view and a row scrolled away has none. The offset is this tick's; the sizes are the last frame's. Null for a path that was no scroll container.

```ts
og.ui.scrollState(path: string): ScrollState | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | Its full path. |

**Returns** `ScrollState | null`.

Binding symbol `og_ui_scroll_state` · publish builds: kept

```ogt
fn scrollState {
  doc "A scroll container's offset, view and whole content, by its full path (og.ui.lastPath after closing it): what a component virtualises its rows by, since og.ui.bounds is cut to the view and a row scrolled away has none. The offset is this tick's; the sizes are the last frame's. Null for a path that was no scroll container."
  param path String "Its full path."
  returns Option.ScrollState
}
```

<sub>Module `ui` line 1443</sub>
