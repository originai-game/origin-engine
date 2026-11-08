<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-TimelineAction"></a>

### `TimelineAction` (union)

What an event does when a play reaches it.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "sound"` | [`SoundEvent`](../ui/type-SoundEvent.md#type-SoundEvent) | A UI sound. |
| `kind: "haptic"` | [`HapticEvent`](../ui/type-HapticEvent.md#type-HapticEvent) | A haptic. |
| `kind: "burst"` | [`BurstEvent`](../ui/type-BurstEvent.md#type-BurstEvent) | A burst. |
| `kind: "state"` | [`StateEvent`](../ui/type-StateEvent.md#type-StateEvent) | A state change. |
| `kind: "callback"` | [`CallbackEvent`](../ui/type-CallbackEvent.md#type-CallbackEvent) | A callback. |

```ogt
union TimelineAction {
  doc "What an event does when a play reaches it."
  tag kind
  case sound SoundEvent "A UI sound."
  case haptic HapticEvent "A haptic."
  case burst BurstEvent "A burst."
  case state StateEvent "A state change."
  case callback CallbackEvent "A callback."
}
```

<sub>Module `ui` line 2390</sub>
