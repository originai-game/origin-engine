<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-motionEvents"></a>

### `og.ui.motionEvents`

The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps.

```ts
og.ui.motionEvents(): readonly UiMotionEvent[]
```

**Returns** `readonly UiMotionEvent[]`.

Binding symbol `og_ui_motion_events` · publish builds: kept

```ogt
fn motionEvents {
  doc "The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps."
  returns List.UiMotionEvent
}
```

<sub>Module `ui` line 4498</sub>
