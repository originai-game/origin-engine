<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-Gamepad"></a>

### `og.Gamepad` (resource)

Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout: on the web the shell polls the pads each display frame, maps a pad the browser does not map (Firefox, Linux, many third-party pads) through its table, and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys.

Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods. Sticks and triggers are raw: dead zones are the actions' (og.input.setDeadzones). `devices` holds each pad's family as the host told it (og.input.padDevice).

Token `og.Gamepad`, registered under the schema name `og.Gamepad`; its value type is `Gamepad`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `connectedMask` | `number` | `0` | — | — | read only | Connected pads: bit `pad`. |
| `devices` | `number` | `0` | — | — | read only | Each pad's family, four bits per pad (pad p at bit 4p): an InputDevice index (1 xbox, 2 playstation, 3 switch, 4 gamepad), 0 when the host did not say. |
| `buttons0` | `number` | `0` | — | — | read only | Held buttons of pad 0. |
| `buttons1` | `number` | `0` | — | — | read only | Held buttons of pad 1. |
| `buttons2` | `number` | `0` | — | — | read only | Held buttons of pad 2. |
| `buttons3` | `number` | `0` | — | — | read only | Held buttons of pad 3. |
| `buttons4` | `number` | `0` | — | — | read only | Held buttons of pad 4. |
| `buttons5` | `number` | `0` | — | — | read only | Held buttons of pad 5. |
| `buttons6` | `number` | `0` | — | — | read only | Held buttons of pad 6. |
| `buttons7` | `number` | `0` | — | — | read only | Held buttons of pad 7. |
| `pressed0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went down this tick. |
| `pressed1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went down this tick. |
| `pressed2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went down this tick. |
| `pressed3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went down this tick. |
| `pressed4` | `number` | `0` | — | — | read only | Buttons of pad 4 that went down this tick. |
| `pressed5` | `number` | `0` | — | — | read only | Buttons of pad 5 that went down this tick. |
| `pressed6` | `number` | `0` | — | — | read only | Buttons of pad 6 that went down this tick. |
| `pressed7` | `number` | `0` | — | — | read only | Buttons of pad 7 that went down this tick. |
| `released0` | `number` | `0` | — | — | read only | Buttons of pad 0 that went up this tick. |
| `released1` | `number` | `0` | — | — | read only | Buttons of pad 1 that went up this tick. |
| `released2` | `number` | `0` | — | — | read only | Buttons of pad 2 that went up this tick. |
| `released3` | `number` | `0` | — | — | read only | Buttons of pad 3 that went up this tick. |
| `released4` | `number` | `0` | — | — | read only | Buttons of pad 4 that went up this tick. |
| `released5` | `number` | `0` | — | — | read only | Buttons of pad 5 that went up this tick. |
| `released6` | `number` | `0` | — | — | read only | Buttons of pad 6 that went up this tick. |
| `released7` | `number` | `0` | — | — | read only | Buttons of pad 7 that went up this tick. |
| `leftStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 0, -1 to 1, y down. |
| `leftStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 1, -1 to 1, y down. |
| `leftStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 2, -1 to 1, y down. |
| `leftStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 3, -1 to 1, y down. |
| `leftStick4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 4, -1 to 1, y down. |
| `leftStick5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 5, -1 to 1, y down. |
| `leftStick6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 6, -1 to 1, y down. |
| `leftStick7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left stick of pad 7, -1 to 1, y down. |
| `rightStick0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 0, -1 to 1, y down. |
| `rightStick1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 1, -1 to 1, y down. |
| `rightStick2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 2, -1 to 1, y down. |
| `rightStick3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 3, -1 to 1, y down. |
| `rightStick4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 4, -1 to 1, y down. |
| `rightStick5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 5, -1 to 1, y down. |
| `rightStick6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 6, -1 to 1, y down. |
| `rightStick7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The right stick of pad 7, -1 to 1, y down. |
| `triggers0` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 0, 0 to 1. |
| `triggers1` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 1, 0 to 1. |
| `triggers2` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 2, 0 to 1. |
| `triggers3` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 3, 0 to 1. |
| `triggers4` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 4, 0 to 1. |
| `triggers5` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 5, 0 to 1. |
| `triggers6` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 6, 0 to 1. |
| `triggers7` | `Vector2` | `{ x: 0, y: 0 }` | — | — | read only | The left (x) and right (y) triggers of pad 7, 0 to 1. |

```ogt
resource Gamepad {
  doc "Up to eight gamepads at the start of this tick (og-input's og.Gamepad, version 2), in the W3C standard layout: on the web the shell polls the pads each display frame, maps a pad the browser does not map (Firefox, Linux, many third-party pads) through its table, and sends their state through the input ring, so a pad is world state and replays; it also maps pads to keys for games that only read keys."
  doc "Button masks are bit `GamepadButton` index per pad; edges last exactly one tick, and a press and a release between two ticks leave both edges set. Read them through the methods. Sticks and triggers are raw: dead zones are the actions' (og.input.setDeadzones). `devices` holds each pad's family as the host told it (og.input.padDevice)."
  field connectedMask u32 "Connected pads: bit `pad`." {
    readonly true
  }
  field devices u32 "Each pad's family, four bits per pad (pad p at bit 4p): an InputDevice index (1 xbox, 2 playstation, 3 switch, 4 gamepad), 0 when the host did not say." {
    readonly true
  }
  field buttons0 u32 "Held buttons of pad 0." {
    readonly true
  }
  field buttons1 u32 "Held buttons of pad 1." {
    readonly true
  }
  field buttons2 u32 "Held buttons of pad 2." {
    readonly true
  }
  field buttons3 u32 "Held buttons of pad 3." {
    readonly true
  }
  field buttons4 u32 "Held buttons of pad 4." {
    readonly true
  }
  field buttons5 u32 "Held buttons of pad 5." {
    readonly true
  }
  field buttons6 u32 "Held buttons of pad 6." {
    readonly true
  }
  field buttons7 u32 "Held buttons of pad 7." {
    readonly true
  }
  field pressed0 u32 "Buttons of pad 0 that went down this tick." {
    readonly true
  }
  field pressed1 u32 "Buttons of pad 1 that went down this tick." {
    readonly true
  }
  field pressed2 u32 "Buttons of pad 2 that went down this tick." {
    readonly true
  }
  field pressed3 u32 "Buttons of pad 3 that went down this tick." {
    readonly true
  }
  field pressed4 u32 "Buttons of pad 4 that went down this tick." {
    readonly true
  }
  field pressed5 u32 "Buttons of pad 5 that went down this tick." {
    readonly true
  }
  field pressed6 u32 "Buttons of pad 6 that went down this tick." {
    readonly true
  }
  field pressed7 u32 "Buttons of pad 7 that went down this tick." {
    readonly true
  }
  field released0 u32 "Buttons of pad 0 that went up this tick." {
    readonly true
  }
  field released1 u32 "Buttons of pad 1 that went up this tick." {
    readonly true
  }
  field released2 u32 "Buttons of pad 2 that went up this tick." {
    readonly true
  }
  field released3 u32 "Buttons of pad 3 that went up this tick." {
    readonly true
  }
  field released4 u32 "Buttons of pad 4 that went up this tick." {
    readonly true
  }
  field released5 u32 "Buttons of pad 5 that went up this tick." {
    readonly true
  }
  field released6 u32 "Buttons of pad 6 that went up this tick." {
    readonly true
  }
  field released7 u32 "Buttons of pad 7 that went up this tick." {
    readonly true
  }
  field leftStick0 Vector2 "The left stick of pad 0, -1 to 1, y down." {
    readonly true
  }
  field leftStick1 Vector2 "The left stick of pad 1, -1 to 1, y down." {
    readonly true
  }
  field leftStick2 Vector2 "The left stick of pad 2, -1 to 1, y down." {
    readonly true
  }
  field leftStick3 Vector2 "The left stick of pad 3, -1 to 1, y down." {
    readonly true
  }
  field leftStick4 Vector2 "The left stick of pad 4, -1 to 1, y down." {
    readonly true
  }
  field leftStick5 Vector2 "The left stick of pad 5, -1 to 1, y down." {
    readonly true
  }
  field leftStick6 Vector2 "The left stick of pad 6, -1 to 1, y down." {
    readonly true
  }
  field leftStick7 Vector2 "The left stick of pad 7, -1 to 1, y down." {
    readonly true
  }
  field rightStick0 Vector2 "The right stick of pad 0, -1 to 1, y down." {
    readonly true
  }
  field rightStick1 Vector2 "The right stick of pad 1, -1 to 1, y down." {
    readonly true
  }
  field rightStick2 Vector2 "The right stick of pad 2, -1 to 1, y down." {
    readonly true
  }
  field rightStick3 Vector2 "The right stick of pad 3, -1 to 1, y down." {
    readonly true
  }
  field rightStick4 Vector2 "The right stick of pad 4, -1 to 1, y down." {
    readonly true
  }
  field rightStick5 Vector2 "The right stick of pad 5, -1 to 1, y down." {
    readonly true
  }
  field rightStick6 Vector2 "The right stick of pad 6, -1 to 1, y down." {
    readonly true
  }
  field rightStick7 Vector2 "The right stick of pad 7, -1 to 1, y down." {
    readonly true
  }
  field triggers0 Vector2 "The left (x) and right (y) triggers of pad 0, 0 to 1." {
    readonly true
  }
  field triggers1 Vector2 "The left (x) and right (y) triggers of pad 1, 0 to 1." {
    readonly true
  }
  field triggers2 Vector2 "The left (x) and right (y) triggers of pad 2, 0 to 1." {
    readonly true
  }
  field triggers3 Vector2 "The left (x) and right (y) triggers of pad 3, 0 to 1." {
    readonly true
  }
  field triggers4 Vector2 "The left (x) and right (y) triggers of pad 4, 0 to 1." {
    readonly true
  }
  field triggers5 Vector2 "The left (x) and right (y) triggers of pad 5, 0 to 1." {
    readonly true
  }
  field triggers6 Vector2 "The left (x) and right (y) triggers of pad 6, 0 to 1." {
    readonly true
  }
  field triggers7 Vector2 "The left (x) and right (y) triggers of pad 7, 0 to 1." {
    readonly true
  }
  fn connected {
    doc "Whether a pad is connected."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn down {
    doc "Whether a button is held."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn pressed {
    doc "Whether a button went down this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn released {
    doc "Whether a button went up this tick."
    param button GamepadButton "The button."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns bool
  }
  fn axis {
    doc "An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction)."
    param axis GamepadAxis "Which input."
    param pad u32 "The pad, 0 to 7; omitted is 0." {
      optional true
    }
    returns f32
  }
}
```

<sub>Module `input` line 987</sub>

<a id="member-Gamepad-connected"></a>

#### `Gamepad.connected`

Whether a pad is connected.

```ts
Gamepad.connected(pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_connected` · publish builds: kept

```ogt
fn connected {
  doc "Whether a pad is connected."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>Module `input` line 1140</sub>

<a id="member-Gamepad-down"></a>

#### `Gamepad.down`

Whether a button is held.

```ts
Gamepad.down(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a button is held."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>Module `input` line 1147</sub>

<a id="member-Gamepad-pressed"></a>

#### `Gamepad.pressed`

Whether a button went down this tick.

```ts
Gamepad.pressed(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a button went down this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>Module `input` line 1155</sub>

<a id="member-Gamepad-released"></a>

#### `Gamepad.released`

Whether a button went up this tick.

```ts
Gamepad.released(button: GamepadButton, pad?: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `button` | `GamepadButton` | no | The button. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `boolean`.

Binding symbol `og_gamepad_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a button went up this tick."
  param button GamepadButton "The button."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns bool
}
```

<sub>Module `input` line 1163</sub>

<a id="member-Gamepad-axis"></a>

#### `Gamepad.axis`

An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction).

```ts
Gamepad.axis(axis: GamepadAxis, pad?: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `axis` | `GamepadAxis` | no | Which input. |
| `pad` | `number` | yes | The pad, 0 to 7; omitted is 0. |

**Returns** `number`.

Binding symbol `og_gamepad_axis` · publish builds: kept

```ogt
fn axis {
  doc "An analog input, raw: a stick's x or y (-1 to 1, y down) or a trigger (0 to 1). Actions read sticks past their dead zones and y up (og.input.defineAction)."
  param axis GamepadAxis "Which input."
  param pad u32 "The pad, 0 to 7; omitted is 0." {
    optional true
  }
  returns f32
}
```

<sub>Module `input` line 1171</sub>
