<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `tween` module

Tweens: animate any numeric field of a component (og.Transform.position, og.Sprite.color, og.Sprite.opacity, a game's own f32) from its value to a target over time with an easing curve, a delay, loops and yoyo; chain them in sequences or run them in parallel. Advanced by the engine's og.tween system after the game's systems and before og.anim and physics, by scaled time (slow motion, hit stop) unless unscaled.

Running tweens are world state (the og.tween state block): snapshots, restore and the world hash cover them. og.TweenCompleted reports a tween or group that ends on its own; a cancelled one, or one whose entity or component went away, ends silently.

Feel presets are tweens too: og.tween.squash (squash and stretch that recovers) and og.tween.flash (a hit flash that fades).

- **TypeScript**: functions are the namespace `og.tween`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-feel`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module tween {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 8649</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Tween`](#type-Tween) | handle | A running tween or group. |
| [`Tween()`](#column-Tween) | column kind | A Tween handle field. |
| [`Ease`](#type-Ease) | enum | An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end. |
| [`TweenDesc`](#type-TweenDesc) | desc struct | One tween of one field. |
| [`TweenGroupDesc`](#type-TweenGroupDesc) | desc struct | How a sequence or parallel group runs. |
| [`og.TweenCompleted`](#record-TweenCompleted) | event | A tween or group ended on its own (read in the tick after). |
| [`SquashDesc`](#type-SquashDesc) | desc struct | Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest. |
| [`FlashDesc`](#type-FlashDesc) | desc struct | A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0. |
| [`og.tween.to`](#fn-to) | function | Start a tween. Systems may call it; it first moves in this tick's og.tween run. |
| [`og.tween.sequence`](#fn-sequence) | function | Start tweens one after another (each with its own delay, duration and loops); the time one leaves over goes to the next. |
| [`og.tween.parallel`](#fn-parallel) | function | Start tweens together; the group ends with the last one. |
| [`og.tween.cancel`](#fn-cancel) | function | Stop a tween where it is (no event). |
| [`og.tween.cancelEntity`](#fn-cancelEntity) | function | Stop every tween of an entity (no events), say before starting a new reaction. |
| [`og.tween.active`](#fn-active) | function | Whether a tween is still running. |
| [`og.tween.squash`](#fn-squash) | function | Squash and stretch the entity's og.Transform.scale (cancels its earlier squash). |
| [`og.tween.flash`](#fn-flash) | function | Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash. |

## Functions

<a id="fn-to"></a>

### `og.tween.to`

Start a tween. Systems may call it; it first moves in this tick's og.tween run.

```ts
og.tween.to(world: World, desc?: TweenDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `TweenDesc` | yes | The field, the values and the timing. |

**Returns** `Result<Tween>`: Refused for a dead entity, a component it lacks, a field that is not numeric, or values of the wrong length.

Binding symbol `og_tween_to` · publish builds: kept

```ogt
fn to {
  doc "Start a tween. Systems may call it; it first moves in this tick's og.tween run."
  param world World "The world."
  param desc TweenDesc "The field, the values and the timing."
  returns Result.Tween "Refused for a dead entity, a component it lacks, a field that is not numeric, or values of the wrong length."
}
```

<sub>`api.ogt` line 8774</sub>

<a id="fn-sequence"></a>

### `og.tween.sequence`

Start tweens one after another (each with its own delay, duration and loops); the time one leaves over goes to the next.

```ts
og.tween.sequence(world: World, steps: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `steps` | `readonly TweenDesc[]` | no | The tweens, in order. |
| `desc` | `TweenGroupDesc` | yes | Loops, scaling and the tag. |

**Returns** `Result<Tween>`.

Binding symbol `og_tween_sequence` · publish builds: kept

```ogt
fn sequence {
  doc "Start tweens one after another (each with its own delay, duration and loops); the time one leaves over goes to the next."
  param world World "The world."
  param steps List.TweenDesc "The tweens, in order."
  param desc TweenGroupDesc "Loops, scaling and the tag." {
    optional true
  }
  returns Result.Tween
}
```

<sub>`api.ogt` line 8781</sub>

<a id="fn-parallel"></a>

### `og.tween.parallel`

Start tweens together; the group ends with the last one.

```ts
og.tween.parallel(world: World, tweens: readonly TweenDesc[], desc?: TweenGroupDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tweens` | `readonly TweenDesc[]` | no | The tweens. |
| `desc` | `TweenGroupDesc` | yes | Loops, scaling and the tag. |

**Returns** `Result<Tween>`.

Binding symbol `og_tween_parallel` · publish builds: kept

```ogt
fn parallel {
  doc "Start tweens together; the group ends with the last one."
  param world World "The world."
  param tweens List.TweenDesc "The tweens."
  param desc TweenGroupDesc "Loops, scaling and the tag." {
    optional true
  }
  returns Result.Tween
}
```

<sub>`api.ogt` line 8791</sub>

<a id="fn-cancel"></a>

### `og.tween.cancel`

Stop a tween where it is (no event).

```ts
og.tween.cancel(world: World, tween: Tween): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tween` | `Tween` | no | The handle. |

**Returns** `boolean`: False when it was not running.

Binding symbol `og_tween_cancel` · publish builds: kept

```ogt
fn cancel {
  doc "Stop a tween where it is (no event)."
  param world World "The world."
  param tween Tween "The handle."
  returns bool "False when it was not running."
}
```

<sub>`api.ogt` line 8801</sub>

<a id="fn-cancelEntity"></a>

### `og.tween.cancelEntity`

Stop every tween of an entity (no events), say before starting a new reaction.

```ts
og.tween.cancelEntity(world: World, entity: Entity): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `entity` | `Entity` | no | The entity. |

**Returns** `number`: How many stopped.

Binding symbol `og_tween_cancel_entity` · publish builds: kept

```ogt
fn cancelEntity {
  doc "Stop every tween of an entity (no events), say before starting a new reaction."
  param world World "The world."
  param entity Entity "The entity."
  returns u32 "How many stopped."
}
```

<sub>`api.ogt` line 8808</sub>

<a id="fn-active"></a>

### `og.tween.active`

Whether a tween is still running.

```ts
og.tween.active(world: World, tween: Tween): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `tween` | `Tween` | no | The handle. |

**Returns** `boolean`.

Binding symbol `og_tween_active` · publish builds: kept

```ogt
fn active {
  doc "Whether a tween is still running."
  param world World "The world."
  param tween Tween "The handle."
  returns bool
}
```

<sub>`api.ogt` line 8815</sub>

<a id="fn-squash"></a>

### `og.tween.squash`

Squash and stretch the entity's og.Transform.scale (cancels its earlier squash).

```ts
og.tween.squash(world: World, desc?: SquashDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `SquashDesc` | yes | The entity and the impact. |

**Returns** `Result<Tween>`: Refused for an entity without og.Transform.

Binding symbol `og_tween_squash` · publish builds: kept

```ogt
fn squash {
  doc "Squash and stretch the entity's og.Transform.scale (cancels its earlier squash)."
  param world World "The world."
  param desc SquashDesc "The entity and the impact."
  returns Result.Tween "Refused for an entity without og.Transform."
}
```

<sub>`api.ogt` line 8822</sub>

<a id="fn-flash"></a>

### `og.tween.flash`

Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash.

```ts
og.tween.flash(world: World, desc?: FlashDesc): Result<Tween>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `FlashDesc` | yes | The entity and the flash. |

**Returns** `Result<Tween>`: Refused for an entity with none of og.Sprite, og.Mesh and og.Model.

Binding symbol `og_tween_flash` · publish builds: kept

```ogt
fn flash {
  doc "Flash the entity's og.Sprite, its og.Mesh, or its og.Model (a glTF instance: this instance only, the others sharing its materials stay as they are). Cancels its earlier flash."
  param world World "The world."
  param desc FlashDesc "The entity and the flash."
  returns Result.Tween "Refused for an entity with none of og.Sprite, og.Mesh and og.Model."
}
```

<sub>`api.ogt` line 8829</sub>

## Components, resources and events

<a id="record-TweenCompleted"></a>

### `og.TweenCompleted` (event)

A tween or group ended on its own (read in the tick after).

Token `og.TweenCompleted`, registered under the schema name `og.TweenCompleted`; its value type is `TweenCompleted`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tween` | `Tween` | `0` | — | — | — | The handle og.tween.to, sequence, parallel, squash or flash answered. |
| `entity` | `Entity` | `0` | — | — | — | The entity of its (first) tween. |
| `tag` | `number` | `0` | — | — | — | Its tag. |

```ogt
event TweenCompleted {
  doc "A tween or group ended on its own (read in the tick after)."
  field tween Tween "The handle og.tween.to, sequence, parallel, squash or flash answered."
  field entity Entity "The entity of its (first) tween."
  field tag u32 "Its tag."
}
```

<sub>`api.ogt` line 8727</sub>

## Types

<a id="type-Tween"></a>

### `Tween` (handle)

A running tween or group.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle Tween "A running tween or group."
```

<sub>`api.ogt` line 8655</sub>

<a id="type-Ease"></a>

### `Ease` (enum)

An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end.

A string: one of the variants below. The zero value is `"quadOut"`.

| Variant | Doc |
|---|---|
| `"quadOut"` | Gentle ease out: the default for most motion. |
| `"linear"` | Constant speed: scrolling, timers, colour cycles. |
| `"quadIn"` | Gentle ease in. |
| `"quadInOut"` | Gentle in and out. |
| `"cubicIn"` | Ease in. |
| `"cubicOut"` | Ease out: UI sliding in, things settling. |
| `"cubicInOut"` | In and out: camera moves, deliberate motion. |
| `"quartIn"` | Strong ease in. |
| `"quartOut"` | Strong ease out. |
| `"quartInOut"` | Strong in and out. |
| `"expoIn"` | Exponential ease in: things falling away, warp-outs. |
| `"expoOut"` | Exponential ease out: snappy arrivals. |
| `"expoInOut"` | Exponential in and out. |
| `"backIn"` | Pulls back first, then goes: anticipation. |
| `"backOut"` | Overshoots, then settles: pops, squash recovery, buttons. |
| `"backInOut"` | Pulls back and overshoots. |
| `"elasticIn"` | Rings before leaving. |
| `"elasticOut"` | Rings after arriving: springy, cartoon impacts. |
| `"elasticInOut"` | Rings at both ends. |
| `"bounceIn"` | Bounces before leaving. |
| `"bounceOut"` | Bounces at the end: drops, landings. |
| `"bounceInOut"` | Bounces at both ends. |

```ogt
enum Ease {
  doc "An easing curve from the start (0) to the end (1). In starts slow, out ends slow, inOut both. back overshoots, elastic rings, bounce bounces at the end."
  variant quadOut "Gentle ease out: the default for most motion."
  variant linear "Constant speed: scrolling, timers, colour cycles."
  variant quadIn "Gentle ease in."
  variant quadInOut "Gentle in and out."
  variant cubicIn "Ease in."
  variant cubicOut "Ease out: UI sliding in, things settling."
  variant cubicInOut "In and out: camera moves, deliberate motion."
  variant quartIn "Strong ease in."
  variant quartOut "Strong ease out."
  variant quartInOut "Strong in and out."
  variant expoIn "Exponential ease in: things falling away, warp-outs."
  variant expoOut "Exponential ease out: snappy arrivals."
  variant expoInOut "Exponential in and out."
  variant backIn "Pulls back first, then goes: anticipation."
  variant backOut "Overshoots, then settles: pops, squash recovery, buttons."
  variant backInOut "Pulls back and overshoots."
  variant elasticIn "Rings before leaving."
  variant elasticOut "Rings after arriving: springy, cartoon impacts."
  variant elasticInOut "Rings at both ends."
  variant bounceIn "Bounces before leaving."
  variant bounceOut "Bounces at the end: drops, landings."
  variant bounceInOut "Bounces at both ends."
}
```

<sub>`api.ogt` line 8664</sub>

<a id="type-TweenDesc"></a>

### `TweenDesc` (desc struct)

One tween of one field.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity whose component is tweened. |
| `component` | `AnyComponent \| null` | `null` | — | — | — | The component token (og.Transform, og.Sprite, a game's own); null is og.Transform. |
| `field` | `string` | `""` | — | — | — | The field, or one lane of it: `position`, `position.y`, `scale`, `color.r`, `opacity`. f32, f64, i32 and u32 fields (rounded), vectors, colours and quaternions (as a rotation) tween; bool, enum and handle fields do not. |
| `to` | `readonly number[]` | `[]` | — | — | — | The end value: one number for a scalar or for every lane, or one per lane. |
| `from` | `readonly number[] \| null` | `null` | — | — | — | The start value, written from the tick the tween is started and held through its delay; null takes the field's value when the tween starts (after its delay). |
| `relative` | `boolean` | `false` | — | — | — | `to` is added to the start value. |
| `duration` | `number` | `0.3` | ≥ 0 | `s` | — | Seconds from start to end (one cycle). |
| `delay` | `number` | `0` | ≥ 0 | `s` | — | Seconds before it starts. |
| `ease` | `Ease` | `"quadOut"` | — | — | — | The curve. |
| `loops` | `number` | `0` | ≥ -1 | — | — | Extra cycles: 0 plays once, n repeats n more times, -1 forever. |
| `yoyo` | `boolean` | `false` | — | — | — | Every other cycle plays backwards (pulses, bobbing). |
| `unscaled` | `boolean` | `false` | — | — | — | Advance by real ticks, ignoring Time.scale and hit stop (UI motion). |
| `tag` | `number` | `0` | — | — | — | A number of yours, carried by og.TweenCompleted. |

```ogt
struct TweenDesc {
  doc "One tween of one field."
  desc true
  field entity Entity "The entity whose component is tweened."
  field component Option.AnyComponent "The component token (og.Transform, og.Sprite, a game's own); null is og.Transform."
  field field String "The field, or one lane of it: `position`, `position.y`, `scale`, `color.r`, `opacity`. f32, f64, i32 and u32 fields (rounded), vectors, colours and quaternions (as a rotation) tween; bool, enum and handle fields do not."
  field to List.f32 "The end value: one number for a scalar or for every lane, or one per lane."
  field from Option.List.f32 "The start value, written from the tick the tween is started and held through its delay; null takes the field's value when the tween starts (after its delay)."
  field relative bool "`to` is added to the start value."
  field duration f32 "Seconds from start to end (one cycle)." {
    default 0.3
    min 0
    unit s
  }
  field delay f32 "Seconds before it starts." {
    min 0
    unit s
  }
  field ease Ease "The curve."
  field loops i32 "Extra cycles: 0 plays once, n repeats n more times, -1 forever." {
    min -1
  }
  field yoyo bool "Every other cycle plays backwards (pulses, bobbing)."
  field unscaled bool "Advance by real ticks, ignoring Time.scale and hit stop (UI motion)."
  field tag u32 "A number of yours, carried by og.TweenCompleted."
}
```

<sub>`api.ogt` line 8690</sub>

<a id="type-TweenGroupDesc"></a>

### `TweenGroupDesc` (desc struct)

How a sequence or parallel group runs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `loops` | `number` | `0` | ≥ -1 | — | — | Extra runs of the whole group: 0 once, -1 forever; each run replays the spans its tweens resolved in the first. |
| `unscaled` | `boolean` | `false` | — | — | — | Advance by real ticks, ignoring Time.scale and hit stop. |
| `tag` | `number` | `0` | — | — | — | A number of yours, carried by og.TweenCompleted. |

```ogt
struct TweenGroupDesc {
  doc "How a sequence or parallel group runs."
  desc true
  field loops i32 "Extra runs of the whole group: 0 once, -1 forever; each run replays the spans its tweens resolved in the first." {
    min -1
  }
  field unscaled bool "Advance by real ticks, ignoring Time.scale and hit stop."
  field tag u32 "A number of yours, carried by og.TweenCompleted."
}
```

<sub>`api.ogt` line 8717</sub>

<a id="type-SquashDesc"></a>

### `SquashDesc` (desc struct)

Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `impact` | `Partial<Vector3>` | `{}` | — | — | — | The x, y and z factors at the impact: (1.25, 0.8) squashes (a landing, a hit), (0.8, 1.25) stretches (a jump). z is for 3D models (a squash bulges in x and z: (1.15, 0.85, 1.15)); left out or 0 it stays at rest, so a sprite's { x, y } is unchanged. A component left out takes its default: x 1.25, y 0.8, z 0. |
| `rest` | `Vector3` | `{ x: 1, y: 1, z: 1 }` | — | — | — | The scale it recovers to. |
| `duration` | `number` | `0.18` | ≥ 0 | `s` | — | Seconds to recover. |
| `ease` | `Ease` | `"backOut"` | — | — | — | The recovery curve; backOut overshoots a little, elasticOut wobbles. |

```ogt
struct SquashDesc {
  doc "Squash and stretch: the entity's og.Transform.scale jumps to rest times impact and eases back to rest."
  desc true
  field entity Entity "The entity."
  field impact Partial.Vector3 "The x, y and z factors at the impact: (1.25, 0.8) squashes (a landing, a hit), (0.8, 1.25) stretches (a jump). z is for 3D models (a squash bulges in x and z: (1.15, 0.85, 1.15)); left out or 0 it stays at rest, so a sprite's { x, y } is unchanged. A component left out takes its default: x 1.25, y 0.8, z 0."
  field rest Vector3 "The scale it recovers to." {
    default 1 1 1
  }
  field duration f32 "Seconds to recover." {
    default 0.18
    min 0
    unit s
  }
  field ease Ease "The recovery curve; backOut overshoots a little, elasticOut wobbles." {
    default backOut
  }
}
```

<sub>`api.ogt` line 8734</sub>

<a id="type-FlashDesc"></a>

### `FlashDesc` (desc struct)

A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity` | `0` | — | — | — | The entity. |
| `color` | `Color` | `{ r: 1, g: 1, b: 1 }` | — | — | — | The flash colour (white for a hit, red for damage taken). |
| `amount` | `number` | `1` | 0 … 1 | — | — | How far it starts towards the colour. |
| `duration` | `number` | `0.12` | ≥ 0 | `s` | — | Seconds to fade. |
| `ease` | `Ease` | `"quadIn"` | — | — | — | The fade's curve; quadIn holds the flash, then drops it. |

```ogt
struct FlashDesc {
  doc "A hit flash: the entity's og.Sprite (else og.Mesh, else og.Model) flashColor set and its flash faded from amount to 0."
  desc true
  field entity Entity "The entity."
  field color Color "The flash colour (white for a hit, red for damage taken)." {
    default 1 1 1
  }
  field amount f32 "How far it starts towards the colour." {
    default 1
    min 0
    max 1
  }
  field duration f32 "Seconds to fade." {
    default 0.12
    min 0
    unit s
  }
  field ease Ease "The fade's curve; quadIn holds the flash, then drops it." {
    default quadIn
  }
}
```

<sub>`api.ogt` line 8752</sub>

## Column kinds

Each is a factory, `Tween(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-Tween"></a>`Tween` | `Tween` | `Uint32Array` | 2 | A Tween handle field. |

```ogt
column Tween {
  doc "A Tween handle field."
  array Uint32Array
  stride 2
  value Tween
}
```

<sub>`api.ogt` line 8657</sub>
