<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DisplayInfo"></a>

### `DisplayInfo` (struct)

What the display draws now (og.render.display): the settings, and what they resolved to on this device.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `settings` | `DisplaySettings` | `{}` | — | — | — | The settings og.render.setDisplay last set (the defaults before). |
| `presenting` | `boolean` | `false` | — | — | — | Whether this host presents to a display (the web canvas): false on a native run, where nothing below moves from its default. |
| `maxFps` | `number` | `0` | — | — | — | The frame cap resolved: 30, 60, or 0 for the display's rate. |
| `displayHz` | `number` | `0` | — | — | — | The display's refresh rate as measured (requestAnimationFrame's interval); 0 where unknown. |
| `renderScale` | `number` | `0` | — | — | — | The render scale the scene is drawn at now (automatic quality's, within its bounds). |
| `presetSteps` | `number` | `0` | — | — | — | The preset steps automatic quality has taken below RenderSettings (0: none). |
| `sceneWidth` | `number` | `0` | — | — | — | The 3D scene's width in device pixels (the display's times the render scale). |
| `sceneHeight` | `number` | `0` | — | — | — | Its height. |
| `frameMs` | `number` | `0` | — | — | — | The frame time automatic quality reads, smoothed: the GPU's where timed, else the interval between frames; 0 before a frame. |
| `deviceLosses` | `number` | `0` | — | — | — | How many times the GPU device was lost and the renderer made again on a new one (iOS Safari loses it after the page was in the background or under memory pressure). The game goes on: the next frame draws the world as it is, every mesh and texture uploaded again from the loaded assets. |
| `gpu` | `GpuMemory \| null` | `null` | — | — | — | The GPU memory the renderer holds; null without a renderer. |
| `vsync` | `boolean` | `false` | — | — | — | Frames are presented on the display's refresh (no tearing): always on a presenting host, since a browser has no switch for it; a settings page shows V-Sync as on and fixed. False on a native run, which presents nothing. |
| `fullscreen` | `boolean` | `false` | — | — | — | The page is fullscreen now, as the host last reported it. |
| `powerPreference` | `PowerPreference` | `"highPerformance"` | — | — | — | The power preference the GPU adapter was made with at the boot (highPerformance on a native run). |
| `restartNeeded` | `boolean` | `false` | — | — | — | settings.powerPreference differs from the adapter's: it applies on the next load (a settings page says so). |

```ogt
struct DisplayInfo {
  doc "What the display draws now (og.render.display): the settings, and what they resolved to on this device."
  field settings DisplaySettings "The settings og.render.setDisplay last set (the defaults before)."
  field presenting bool "Whether this host presents to a display (the web canvas): false on a native run, where nothing below moves from its default."
  field maxFps u32 "The frame cap resolved: 30, 60, or 0 for the display's rate."
  field displayHz f32 "The display's refresh rate as measured (requestAnimationFrame's interval); 0 where unknown."
  field renderScale f32 "The render scale the scene is drawn at now (automatic quality's, within its bounds)."
  field presetSteps u32 "The preset steps automatic quality has taken below RenderSettings (0: none)."
  field sceneWidth u32 "The 3D scene's width in device pixels (the display's times the render scale)."
  field sceneHeight u32 "Its height."
  field frameMs f32 "The frame time automatic quality reads, smoothed: the GPU's where timed, else the interval between frames; 0 before a frame."
  field deviceLosses u32 "How many times the GPU device was lost and the renderer made again on a new one (iOS Safari loses it after the page was in the background or under memory pressure). The game goes on: the next frame draws the world as it is, every mesh and texture uploaded again from the loaded assets."
  field gpu Option.GpuMemory "The GPU memory the renderer holds; null without a renderer."
  field vsync bool "Frames are presented on the display's refresh (no tearing): always on a presenting host, since a browser has no switch for it; a settings page shows V-Sync as on and fixed. False on a native run, which presents nothing."
  field fullscreen bool "The page is fullscreen now, as the host last reported it."
  field powerPreference PowerPreference "The power preference the GPU adapter was made with at the boot (highPerformance on a native run)."
  field restartNeeded bool "settings.powerPreference differs from the adapter's: it applies on the next load (a settings page says so)."
}
```

<sub>Module `render` line 2813</sub>
