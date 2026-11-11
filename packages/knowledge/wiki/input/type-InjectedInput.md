<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="type-InjectedInput"></a>

### `InjectedInput` (union)

One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "key"` | [`KeyInput`](../input/type-KeyInput.md#type-KeyInput) | A key event. |
| `kind: "pointer"` | [`PointerInput`](../input/type-PointerInput.md#type-PointerInput) | A pointer event. |
| `kind: "wheel"` | [`WheelInput`](../input/type-WheelInput.md#type-WheelInput) | A wheel event. |
| `kind: "gamepad"` | [`GamepadInput`](../input/type-GamepadInput.md#type-GamepadInput) | A gamepad's state. |
| `kind: "text"` | [`TextEntry`](../input/type-TextEntry.md#type-TextEntry) | Typed or composed text. |
| `kind: "a11y"` | [`A11yRequest`](../input/type-A11yRequest.md#type-A11yRequest) | A screen reader's request on a UI control. |
| `kind: "pointerLock"` | [`PointerLockReport`](../input/type-PointerLockReport.md#type-PointerLockReport) | The host's word on the pointer lock (og.PointerLock.locked). |

```ogt
union InjectedInput {
  doc "One input event in the game's own vocabulary, as the probe's input() and playtest scripts carry it."
  tag kind
  case key KeyInput "A key event."
  case pointer PointerInput "A pointer event."
  case wheel WheelInput "A wheel event."
  case gamepad GamepadInput "A gamepad's state."
  case text TextEntry "Typed or composed text."
  case a11y A11yRequest "A screen reader's request on a UI control."
  case pointerLock PointerLockReport "The host's word on the pointer lock (og.PointerLock.locked)."
}
```

<sub>Module `input` line 751</sub>
