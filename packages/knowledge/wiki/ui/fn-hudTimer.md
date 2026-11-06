<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-hudTimer"></a>

### `og.ui.hudTimer`

A timer to the bar of fighting and battle royale HUDs: the time in tabular digits on the HUD chip, a ring for a countdown's share left, warn and then danger with a pulse and a tick sound each second of the last ones, tenths in the last ten seconds, dimmed and blinking while paused.

Parts: HudTimer.frame[normal|warn|critical|paused], HudTimer.ring, HudTimer.time, HudTimer.label. States: group time (normal, warn, critical, paused, done). Sounds: counterTick each of the last seconds, pitched up. Accessibility: timer named label, value the time. Probe ids: <id>, <id>/Time, <id>/Label.

```ts
og.ui.hudTimer(id: string, desc?: HudTimerDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `HudTimerDesc` | yes | The time. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn hudTimer {
  doc "A timer to the bar of fighting and battle royale HUDs: the time in tabular digits on the HUD chip, a ring for a countdown's share left, warn and then danger with a pulse and a tick sound each second of the last ones, tenths in the last ten seconds, dimmed and blinking while paused."
  doc "Parts: HudTimer.frame[normal|warn|critical|paused], HudTimer.ring, HudTimer.time, HudTimer.label. States: group time (normal, warn, critical, paused, done). Sounds: counterTick each of the last seconds, pitched up. Accessibility: timer named label, value the time. Probe ids: <id>, <id>/Time, <id>/Label."
  host false
  param id String "Its id."
  param desc HudTimerDesc "The time."
  returns Void
}
```

<sub>`api.ogt` line 22071</sub>
