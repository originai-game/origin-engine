<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="record-CameraView"></a>

### `og.CameraView` (component)

A secondary camera, as three.js's setRenderTarget or setViewport: it draws the scene from the entity's MatrixWorld (looking down its local -Z) into a render target, or without one into a viewport rect over the finished image and under the UI. Views draw lowest order first, all before the frame.

It draws what the main camera draws on its own layers. An ldr target and a viewport carry its exposure and the frame's tone curve; an hdr target carries linear radiance. Neither runs the frame's post effects; screen sprites, the UI, debug draw, trails and 2D fluids stay on the main frame.

See `render §11`.

Also known as: picture-in-picture, rear-view mirror, split screen, second camera, minimap camera

Token `og.CameraView`, registered under the schema name `og.CameraView`; its value type is `CameraView`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `RenderTarget` | `0` | — | — | — | The target it draws into; 0 draws into the viewport rect of the frame. |
| `viewportX` | `number` | `0` | 0 … 1 | — | — | The viewport's left edge, as a fraction of the frame's width (without a target). |
| `viewportY` | `number` | `0` | 0 … 1 | — | — | The viewport's top edge, as a fraction of the frame's height. |
| `viewportWidth` | `number` | `1` | 0 … 1 | — | — | The viewport's width, as a fraction of the frame's. |
| `viewportHeight` | `number` | `1` | 0 … 1 | — | — | The viewport's height, as a fraction of the frame's. |
| `fov` | `number` | `50` | 1 … 179 | `deg` | tunable | The vertical field of view, as og.Camera.fov. |
| `near` | `number` | `0.1` | ≥ 0.001 | `m` | — | The near clipping plane. |
| `far` | `number` | `2000` | ≥ 0.01 | `m` | — | The far clipping plane. |
| `zoom` | `number` | `1` | 0.01 … 100 | — | — | The zoom factor, as og.Camera.zoom. |
| `orthographic` | `boolean` | `false` | — | — | — | An orthographic projection of `height` world units. |
| `height` | `number` | `10` | ≥ 0.01 | `m` | — | The visible world height of the orthographic projection. |
| `exposure` | `number` | `1` | 0 … 16 | — | tunable | Its exposure before tone mapping (an ldr target and a viewport). |
| `layers` | `number` | `1` | — | — | — | Three.js layers, as og.Camera.layers: it draws the objects whose layers share a bit with it. |
| `order` | `number` | `0` | — | — | — | Views draw in ascending order (ties in entity order), all before the frame. |
| `clear` | `boolean` | `false` | — | — | — | Clear to clearColor instead of drawing the scene's background (sky or colour). |
| `clearColor` | `Color` | `{ r: 0, g: 0, b: 0 }` | — | — | — | The linear colour it clears to with clear. |
| `every` | `number` | `1` | — | — | — | Draw once every this many ticks (when the tick is a multiple of it); the target keeps its image between. 0 and 1 draw every frame: a security monitor at 4 costs a quarter. |
| `active` | `boolean` | `true` | — | — | — | Whether it draws. |

```ogt
component CameraView {
  aliases "picture-in-picture, rear-view mirror, split screen, second camera, minimap camera"
  doc "A secondary camera, as three.js's setRenderTarget or setViewport: it draws the scene from the entity's MatrixWorld (looking down its local -Z) into a render target, or without one into a viewport rect over the finished image and under the UI. Views draw lowest order first, all before the frame."
  doc "It draws what the main camera draws on its own layers. An ldr target and a viewport carry its exposure and the frame's tone curve; an hdr target carries linear radiance. Neither runs the frame's post effects; screen sprites, the UI, debug draw, trails and 2D fluids stay on the main frame."
  guide "render §11"
  field target RenderTarget "The target it draws into; 0 draws into the viewport rect of the frame."
  field viewportX f32 "The viewport's left edge, as a fraction of the frame's width (without a target)." {
    min 0
    max 1
  }
  field viewportY f32 "The viewport's top edge, as a fraction of the frame's height." {
    min 0
    max 1
  }
  field viewportWidth f32 "The viewport's width, as a fraction of the frame's." {
    default 1
    min 0
    max 1
  }
  field viewportHeight f32 "The viewport's height, as a fraction of the frame's." {
    default 1
    min 0
    max 1
  }
  field fov f32 "The vertical field of view, as og.Camera.fov." {
    default 50
    min 1
    max 179
    unit deg
    tunable true
  }
  field near f32 "The near clipping plane." {
    default 0.1
    min 0.001
    unit m
  }
  field far f32 "The far clipping plane." {
    default 2000
    min 0.01
    unit m
  }
  field zoom f32 "The zoom factor, as og.Camera.zoom." {
    default 1
    min 0.01
    max 100
  }
  field orthographic bool "An orthographic projection of `height` world units."
  field height f32 "The visible world height of the orthographic projection." {
    default 10
    min 0.01
    unit m
  }
  field exposure f32 "Its exposure before tone mapping (an ldr target and a viewport)." {
    default 1
    min 0
    max 16
    tunable true
  }
  field layers u32 "Three.js layers, as og.Camera.layers: it draws the objects whose layers share a bit with it." {
    default 1
  }
  field order i32 "Views draw in ascending order (ties in entity order), all before the frame."
  field clear bool "Clear to clearColor instead of drawing the scene's background (sky or colour)."
  field clearColor Color "The linear colour it clears to with clear." {
    default 0 0 0
  }
  field every u32 "Draw once every this many ticks (when the tick is a multiple of it); the target keeps its image between. 0 and 1 draw every frame: a security monitor at 4 costs a quarter." {
    default 1
  }
  field active bool "Whether it draws." {
    default true
  }
}
```

<sub>Module `render` line 3962</sub>
