<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-startViewTransition"></a>

### `og.ui.startViewTransition`

Start a view transition scoped to the element at a full path, like CSS's element.startViewTransition(): the shared elements (`view-transition-name`) in its subtree as the last frame drew them are the old view, and what the subtree draws from this tick on is the new one, so change the state it shows in the same tick. Each name morphs as a `::view-transition-group(name)` drawn just after the element, over its children and outside its clip, styled by the `::view-transition-*` rules of the element's document (the user agent's 0.25s cross-fade and move without any). Transitions on different elements run at once, beside a screen's; one already running on this element is replaced. Names under a `view-transition-scope: all` element below it are left to that element's transitions. Returns false, starting none, when the last frame drew nothing at the path or under reduced motion.

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
  doc "Start a view transition scoped to the element at a full path, like CSS's element.startViewTransition(): the shared elements (`view-transition-name`) in its subtree as the last frame drew them are the old view, and what the subtree draws from this tick on is the new one, so change the state it shows in the same tick. Each name morphs as a `::view-transition-group(name)` drawn just after the element, over its children and outside its clip, styled by the `::view-transition-*` rules of the element's document (the user agent's 0.25s cross-fade and move without any). Transitions on different elements run at once, beside a screen's; one already running on this element is replaced. Names under a `view-transition-scope: all` element below it are left to that element's transitions. Returns false, starting none, when the last frame drew nothing at the path or under reduced motion."
  param path String "The element's full path (og.ui.focus, the probe's dump)."
  returns bool
}
```

<sub>Module `ui` line 517</sub>
