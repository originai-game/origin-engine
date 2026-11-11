<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`input`](../input.md) · [index](../index.md)

<a id="record-Pointers"></a>

### `og.Pointers` (resource)

Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id).

Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods.

Token `og.Pointers`, registered under the schema name `og.Pointers`; its value type is `Pointers`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `heldMask` | `number` | `0` | — | — | read only | Pointers held down, as a mask. |
| `pressedMask` | `number` | `0` | — | — | read only | Pointers that went down this tick. |
| `releasedMask` | `number` | `0` | — | — | read only | Pointers that went up this tick. |
| `releaseNextMask` | `number` | `0` | — | — | read only | Pointers a click releases at the end of the next tick. |
| `position0` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 0 is. |
| `position1` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 1 is. |
| `position2` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 2 is. |
| `position3` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 3 is. |
| `position4` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 4 is. |
| `start0` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 0 went down. |
| `start1` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 1 went down. |
| `start2` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 2 went down. |
| `start3` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 3 went down. |
| `start4` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only, worldspace `none` | Where pointer 4 went down. |
| `movement` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | read only | The mouse's relative movement this tick: the sum of pointer 0's movementX, movementY since the last tick (y down), zero again when the tick ends. It keeps coming while the pointer is locked (og.PointerLock) and the position stands still; headless, only injected movementX/Y moves it. See `input §2`. |

```ogt
resource Pointers {
  doc "Every pointer on the play surface at once, at the start of this tick (og-input's og.Pointers): a touch stick and a touch button can be held together. Pointer 0 is og.Input's pointer; ids 1 to 4 are further touches, in the order they landed (the lowest free id)."
  doc "Edges last exactly one tick, as og.Input's do. Masks are bit `id`; read them through the methods."
  field heldMask u32 "Pointers held down, as a mask." {
    readonly true
  }
  field pressedMask u32 "Pointers that went down this tick." {
    readonly true
  }
  field releasedMask u32 "Pointers that went up this tick." {
    readonly true
  }
  field releaseNextMask u32 "Pointers a click releases at the end of the next tick." {
    readonly true
  }
  field position0 Vector2 "Where pointer 0 is." {
    worldspace none
    readonly true
    unit cssPx
  }
  field position1 Vector2 "Where pointer 1 is." {
    worldspace none
    readonly true
    unit cssPx
  }
  field position2 Vector2 "Where pointer 2 is." {
    worldspace none
    readonly true
    unit cssPx
  }
  field position3 Vector2 "Where pointer 3 is." {
    worldspace none
    readonly true
    unit cssPx
  }
  field position4 Vector2 "Where pointer 4 is." {
    worldspace none
    readonly true
    unit cssPx
  }
  field start0 Vector2 "Where pointer 0 went down." {
    worldspace none
    readonly true
    unit cssPx
  }
  field start1 Vector2 "Where pointer 1 went down." {
    worldspace none
    readonly true
    unit cssPx
  }
  field start2 Vector2 "Where pointer 2 went down." {
    worldspace none
    readonly true
    unit cssPx
  }
  field start3 Vector2 "Where pointer 3 went down." {
    worldspace none
    readonly true
    unit cssPx
  }
  field start4 Vector2 "Where pointer 4 went down." {
    worldspace none
    readonly true
    unit cssPx
  }
  field movement Vector2 "The mouse's relative movement this tick: the sum of pointer 0's movementX, movementY since the last tick (y down), zero again when the tick ends. It keeps coming while the pointer is locked (og.PointerLock) and the position stands still; headless, only injected movementX/Y moves it." {
    guide "input §2"
    readonly true
    unit cssPx
  }
  fn down {
    doc "Whether a pointer is held."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn pressed {
    doc "Whether a pointer went down this tick."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn released {
    doc "Whether a pointer went up this tick."
    param id u32 "The pointer, 0 to 4."
    returns bool
  }
  fn position {
    doc "Where a pointer is, in CSS pixels."
    param id u32 "The pointer, 0 to 4."
    returns Vector2
  }
  fn start {
    doc "Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it."
    param id u32 "The pointer, 0 to 4."
    returns Vector2
  }
  fn count {
    doc "How many pointers are held."
    returns u32
  }
}
```

<sub>Module `input` line 888</sub>

<a id="member-Pointers-down"></a>

#### `Pointers.down`

Whether a pointer is held.

```ts
Pointers.down(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_down` · publish builds: kept

```ogt
fn down {
  doc "Whether a pointer is held."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>Module `input` line 958</sub>

<a id="member-Pointers-pressed"></a>

#### `Pointers.pressed`

Whether a pointer went down this tick.

```ts
Pointers.pressed(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_pressed` · publish builds: kept

```ogt
fn pressed {
  doc "Whether a pointer went down this tick."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>Module `input` line 963</sub>

<a id="member-Pointers-released"></a>

#### `Pointers.released`

Whether a pointer went up this tick.

```ts
Pointers.released(id: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `boolean`.

Binding symbol `og_pointers_released` · publish builds: kept

```ogt
fn released {
  doc "Whether a pointer went up this tick."
  param id u32 "The pointer, 0 to 4."
  returns bool
}
```

<sub>Module `input` line 968</sub>

<a id="member-Pointers-position"></a>

#### `Pointers.position`

Where a pointer is, in CSS pixels.

```ts
Pointers.position(id: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `Vector2`.

Binding symbol `og_pointers_position` · publish builds: kept

```ogt
fn position {
  doc "Where a pointer is, in CSS pixels."
  param id u32 "The pointer, 0 to 4."
  returns Vector2
}
```

<sub>Module `input` line 973</sub>

<a id="member-Pointers-start"></a>

#### `Pointers.start`

Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it.

```ts
Pointers.start(id: number): Vector2
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `number` | no | The pointer, 0 to 4. |

**Returns** `Vector2`.

Binding symbol `og_pointers_start` · publish builds: kept

```ogt
fn start {
  doc "Where a pointer went down, in CSS pixels: a touch control claims the pointers that started on it."
  param id u32 "The pointer, 0 to 4."
  returns Vector2
}
```

<sub>Module `input` line 978</sub>

<a id="member-Pointers-count"></a>

#### `Pointers.count`

How many pointers are held.

```ts
Pointers.count(): number
```

**Returns** `number`.

Binding symbol `og_pointers_count` · publish builds: kept

```ogt
fn count {
  doc "How many pointers are held."
  returns u32
}
```

<sub>Module `input` line 983</sub>
