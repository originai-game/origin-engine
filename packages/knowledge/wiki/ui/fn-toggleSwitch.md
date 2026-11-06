<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-toggleSwitch"></a>

### `og.ui.toggleSwitch`

A switch: a track and a knob that slides from its start to its end as it turns on (its selection group, on the style's motion), the track filling with primary, with a label and a description; the whole row takes a press, Enter, Space or A. Plays toggleOn and toggleOff. Parts: `Toggle.track[on|off][hover|pressed|focused|disabled]`, `Toggle.knob[on]`, `Toggle.row[hover|pressed|focused|disabled]` (the whole row, which takes the press). Role switch, checked when on. Probe: `<id>` (its label as name), `<id>/Track`, `<id>/Track/Knob`.

```ts
og.ui.toggleSwitch(id: string, on: boolean, desc?: ToggleSwitchDesc): SwitchResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `on` | `boolean` | no | Its state from game state. |
| `desc` | `ToggleSwitchDesc` | yes | Its label and states. |

**Returns** `SwitchResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn toggleSwitch {
  doc "A switch: a track and a knob that slides from its start to its end as it turns on (its selection group, on the style's motion), the track filling with primary, with a label and a description; the whole row takes a press, Enter, Space or A. Plays toggleOn and toggleOff. Parts: `Toggle.track[on|off][hover|pressed|focused|disabled]`, `Toggle.knob[on]`, `Toggle.row[hover|pressed|focused|disabled]` (the whole row, which takes the press). Role switch, checked when on. Probe: `<id>` (its label as name), `<id>/Track`, `<id>/Track/Knob`."
  host false
  param id String "Its id."
  param on bool "Its state from game state."
  param desc ToggleSwitchDesc "Its label and states."
  returns SwitchResult
}
```

<sub>`api.ogt` line 19391</sub>
