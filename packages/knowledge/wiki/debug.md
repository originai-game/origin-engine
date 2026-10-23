<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `debug` module

Debug draw: lines, boxes, spheres, arrows, axes and labels in the world, each for a number of ticks, drawn over the finished frame (after every post effect, before the UI) so they read in every capture and never hide behind geometry; and the debug views (physics, navigation, bounds) the command channel's `debug:` actions switch.

Development only: not in publish builds, where every member does nothing and answers its default, so a game may leave its debug draws in. Debug draw is rendering: it never feeds back into the simulation, and its draws are not world state (a snapshot or a replay does not carry them).

- **TypeScript**: functions are the namespace `og.debug`; components, resources and events are tokens on `og`.
- **Publish builds**: dropped (members may override).
- **Skill**: `og-game`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module debug {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 13388</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`DebugDrawDesc`](#type-DebugDrawDesc) | desc struct | How a debug shape looks and how long it lasts. |
| [`og.debug.line`](#fn-line) | function | A line from one point to another. |
| [`og.debug.box`](#fn-box) | function | A box's twelve edges, as three.js Box3Helper (turned by desc.rotation). |
| [`og.debug.sphere`](#fn-sphere) | function | A sphere as its three great circles. |
| [`og.debug.arrow`](#fn-arrow) | function | An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target. |
| [`og.debug.axes`](#fn-axes) | function | A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read). |
| [`og.debug.text`](#fn-text) | function | A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name. |
| [`og.debug.clear`](#fn-clear) | function | Remove every debug draw that is still showing. |
| [`og.debug.setView`](#fn-setView) | function | Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone. |
| [`og.debug.view`](#fn-view) | function | The debug view shown. |
| [`og.debug.setEnabled`](#fn-setEnabled) | function | Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds. |
| [`og.debug.enabled`](#fn-enabled) | function | Whether debug draws and views show. |

## Functions

<a id="fn-line"></a>

### `og.debug.line`

A line from one point to another.

```ts
og.debug.line(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `from` | `Vector3` | no | Where it starts. |
| `to` | `Vector3` | no | Where it ends. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration. |

**Returns** `void`.

Binding symbol `og_debug_line` · publish builds: dropped

```ogt
fn line {
  doc "A line from one point to another."
  param world World "The world drawn in."
  param from Vector3 "Where it starts."
  param to Vector3 "Where it ends."
  param desc DebugDrawDesc "Colour, width, duration."
}
```

<sub>`api.ogt` line 13410</sub>

<a id="fn-box"></a>

### `og.debug.box`

A box's twelve edges, as three.js Box3Helper (turned by desc.rotation).

```ts
og.debug.box(world: World, center: Vector3, size: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `center` | `Vector3` | no | Its centre. |
| `size` | `Vector3` | no | Its extent along each axis. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration, rotation. |

**Returns** `void`.

Binding symbol `og_debug_box` · publish builds: dropped

```ogt
fn box {
  doc "A box's twelve edges, as three.js Box3Helper (turned by desc.rotation)."
  param world World "The world drawn in."
  param center Vector3 "Its centre."
  param size Vector3 "Its extent along each axis."
  param desc DebugDrawDesc "Colour, width, duration, rotation."
}
```

<sub>`api.ogt` line 13418</sub>

<a id="fn-sphere"></a>

### `og.debug.sphere`

A sphere as its three great circles.

```ts
og.debug.sphere(world: World, center: Vector3, radius: number, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `center` | `Vector3` | no | Its centre. |
| `radius` | `number` | no | Its radius. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration, rotation. |

**Returns** `void`.

Binding symbol `og_debug_sphere` · publish builds: dropped

```ogt
fn sphere {
  doc "A sphere as its three great circles."
  param world World "The world drawn in."
  param center Vector3 "Its centre."
  param radius f32 "Its radius."
  param desc DebugDrawDesc "Colour, width, duration, rotation."
}
```

<sub>`api.ogt` line 13426</sub>

<a id="fn-arrow"></a>

### `og.debug.arrow`

An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target.

```ts
og.debug.arrow(world: World, from: Vector3, to: Vector3, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `from` | `Vector3` | no | Its tail. |
| `to` | `Vector3` | no | Its tip. |
| `desc` | `DebugDrawDesc` | yes | Colour, width, duration. |

**Returns** `void`.

Binding symbol `og_debug_arrow` · publish builds: dropped

```ogt
fn arrow {
  doc "An arrow from one point to another, its head a fifth of its length (three.js ArrowHelper): a velocity, a force, a direction to a target."
  param world World "The world drawn in."
  param from Vector3 "Its tail."
  param to Vector3 "Its tip."
  param desc DebugDrawDesc "Colour, width, duration."
}
```

<sub>`api.ogt` line 13434</sub>

<a id="fn-axes"></a>

### `og.debug.axes`

A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read).

```ts
og.debug.axes(world: World, position: Vector3, size: number, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `position` | `Vector3` | no | The frame's origin. |
| `size` | `number` | no | Each axis's length. |
| `desc` | `DebugDrawDesc` | yes | Width, duration and the frame's rotation. |

**Returns** `void`.

Binding symbol `og_debug_axes` · publish builds: dropped

```ogt
fn axes {
  doc "A frame's three axes, as three.js AxesHelper: +x red, +y green, +z blue (desc.color is not read)."
  param world World "The world drawn in."
  param position Vector3 "The frame's origin."
  param size f32 "Each axis's length."
  param desc DebugDrawDesc "Width, duration and the frame's rotation."
}
```

<sub>`api.ogt` line 13442</sub>

<a id="fn-text"></a>

### `og.debug.text`

A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name.

```ts
og.debug.text(world: World, position: Vector3, text: string, desc?: DebugDrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `position` | `Vector3` | no | The point the label sits above. |
| `text` | `string` | no | The label. |
| `desc` | `DebugDrawDesc` | yes | Colour and duration (width is the text's size in CSS pixels, 14 where it is 2 or less). |

**Returns** `void`.

Binding symbol `og_debug_text` · publish builds: dropped

```ogt
fn text {
  doc "A label anchored at a world point, drawn in screen space at a constant size in the UI's body font (a world with no UI font draws none), centred above the point: an entity's state, a counter, a name."
  param world World "The world drawn in."
  param position Vector3 "The point the label sits above."
  param text String "The label."
  param desc DebugDrawDesc "Colour and duration (width is the text's size in CSS pixels, 14 where it is 2 or less)."
}
```

<sub>`api.ogt` line 13450</sub>

<a id="fn-clear"></a>

### `og.debug.clear`

Remove every debug draw that is still showing.

```ts
og.debug.clear(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |

**Returns** `void`.

Binding symbol `og_debug_clear` · publish builds: dropped

```ogt
fn clear {
  doc "Remove every debug draw that is still showing."
  param world World "The world drawn in."
}
```

<sub>`api.ogt` line 13458</sub>

<a id="fn-setView"></a>

### `og.debug.setView`

Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone.

```ts
og.debug.setView(world: World, view: DebugView): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `view` | `DebugView` | no | The view. |

**Returns** `void`.

Binding symbol `og_debug_set_view` · publish builds: dropped

```ogt
fn setView {
  doc "Show a debug view besides og.debug's draws (the command channel's `debug:<view>`); none shows the draws alone."
  param world World "The world drawn in."
  param view DebugView "The view."
}
```

<sub>`api.ogt` line 13463</sub>

<a id="fn-view"></a>

### `og.debug.view`

The debug view shown.

```ts
og.debug.view(world: World): DebugView
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `DebugView`.

Binding symbol `og_debug_view` · publish builds: dropped

```ogt
fn view {
  doc "The debug view shown."
  param world World "The world asked."
  returns DebugView
}
```

<sub>`api.ogt` line 13469</sub>

<a id="fn-setEnabled"></a>

### `og.debug.setEnabled`

Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds.

```ts
og.debug.setEnabled(world: World, enabled: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world drawn in. |
| `enabled` | `boolean` | no | Whether they show. |

**Returns** `void`.

Binding symbol `og_debug_set_enabled` · publish builds: dropped

```ogt
fn setEnabled {
  doc "Show or hide every debug draw and view (`debug:on`, `debug:off`); on by default outside publish builds."
  param world World "The world drawn in."
  param enabled bool "Whether they show."
}
```

<sub>`api.ogt` line 13475</sub>

<a id="fn-enabled"></a>

### `og.debug.enabled`

Whether debug draws and views show.

```ts
og.debug.enabled(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world asked. |

**Returns** `boolean`.

Binding symbol `og_debug_enabled` · publish builds: dropped

```ogt
fn enabled {
  doc "Whether debug draws and views show."
  param world World "The world asked."
  returns bool
}
```

<sub>`api.ogt` line 13481</sub>

## Types

<a id="type-DebugDrawDesc"></a>

### `DebugDrawDesc` (desc struct)

How a debug shape looks and how long it lasts.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Color` | `{ r: 1, g: 1, b: 0 }` | — | — | — | The linear colour. |
| `width` | `number` | `2` | ≥ 0 | — | — | Line width in CSS pixels. |
| `duration` | `number` | `0` | — | `ticks` | — | How many ticks after this one it stays; 0 is this frame only (draw it every tick it should show). |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The shape's rotation: a box's or a sphere's circles', the axes' frame. |

```ogt
struct DebugDrawDesc {
  doc "How a debug shape looks and how long it lasts."
  desc true
  field color Color "The linear colour." {
    default 1 1 0
  }
  field width f32 "Line width in CSS pixels." {
    default 2
    min 0
  }
  field duration u32 "How many ticks after this one it stays; 0 is this frame only (draw it every tick it should show)." {
    unit ticks
  }
  field rotation Quaternion "The shape's rotation: a box's or a sphere's circles', the axes' frame."
}
```

<sub>`api.ogt` line 13394</sub>
