<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-playTimeline"></a>

### `og.ui.playTimeline`

Play one timeline (by name) on an element: og.ui.play with a single step.

```ts
og.ui.playTimeline(path: string, timeline: string, desc?: PlayDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path; empty is the element built just before. |
| `timeline` | `string` | no | The timeline. |
| `desc` | `PlayDesc` | yes | How it runs. |

**Returns** `number`.

Binding symbol `og_ui_play_timeline` · publish builds: kept

```ogt
fn playTimeline {
  doc "Play one timeline (by name) on an element: og.ui.play with a single step."
  param path String "The element's full path; empty is the element built just before."
  param timeline String "The timeline."
  param desc PlayDesc "How it runs."
  returns u32
}
```

<sub>Module `ui` line 2953</sub>
