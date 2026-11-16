<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-startViewTransition"></a>

### `og.ui.startViewTransition`

Start an element-scoped view transition from its last rendered subtree to its state from this tick onward; change that state in the same tick. Replaces a transition already running on this element. Returns false without starting when the last frame drew nothing at the path or reduced motion is enabled.

See `screens §8`.

```ts
og.ui.startViewTransition(path: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path (og.ui.focus, the probe's dump). |

**Returns** `boolean`.

Binding symbol `og_ui_start_view_transition` · publish builds: kept

```ogt
fn startViewTransition {
  doc "Start an element-scoped view transition from its last rendered subtree to its state from this tick onward; change that state in the same tick. Replaces a transition already running on this element. Returns false without starting when the last frame drew nothing at the path or reduced motion is enabled."
  guide "screens §8"
  param path String "The element's full path (og.ui.focus, the probe's dump)."
  returns bool
}
```

<sub>Module `ui` line 516</sub>
