<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="fn-screenTransition"></a>

### `og.ui.screenTransition`

A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade.

```ts
og.ui.screenTransition(id: string, screen: string, desc?: ScreenTransitionDesc): ScreenTransitionResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `screen` | `string` | no | The screen the game wants now. |
| `desc` | `ScreenTransitionDesc` | yes | Kind, duration, colour or texture, side. |

**Returns** `ScreenTransitionResult`.

Binding symbol `og_ui_screen_transition` · publish builds: kept

```ogt
fn screenTransition {
  doc "A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade."
  param id String "Its id."
  param screen String "The screen the game wants now."
  param desc ScreenTransitionDesc "Kind, duration, colour or texture, side."
  returns ScreenTransitionResult
}
```

<sub>`api.ogt` line 17589</sub>
