<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-play"></a>

### `og.ui.play`

Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). Returns the play's id (0: no element, or a choreo of no known kind).

```ts
og.ui.play(path: string, choreo: Choreo, desc?: PlayDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path; empty is the element built just before. |
| `choreo` | `Choreo` | no | What plays. |
| `desc` | `PlayDesc` | yes | How it runs. |

**Returns** `number`.

Binding symbol `og_ui_play` · publish builds: kept

```ogt
fn play {
  doc "Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). Returns the play's id (0: no element, or a choreo of no known kind)."
  param path String "The element's full path; empty is the element built just before."
  param choreo Choreo "What plays."
  param desc PlayDesc "How it runs."
  returns u32
}
```

<sub>Module `ui` line 4980</sub>
