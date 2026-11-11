<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`render`](../render.md) · [index](../index.md)

<a id="type-DisplaySettings"></a>

### `DisplaySettings` (desc struct)

The player's graphics settings, as a settings page shows them and og.render.setDisplay sets them: the graphics preset, the frame cap, battery saver, render scale and automatic quality, fullscreen and the GPU's power preference. Presentation only: none of it is in the world, so hashes, snapshots, saves and replays never see it. A native run draws every frame at render scale 1 whatever it says, though it draws the preset. og.render.display(world).settings is the whole object to save.

See `looks §17`.

A desc: every field may be omitted and takes its default; `{}` is valid.

#### Preset and frame rate

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `quality` | `GraphicsQuality \| null` | `null` | — | — | — | The player's graphics preset: its row of quality fields (og.render.applyQuality's table) is drawn in place of RenderSettings', which keep the game's values. Null draws the game's own RenderSettings quality. |
| `frameCap` | `FrameCap` | `"auto"` | — | — | — | The most frames a second (auto: 60 on phones, the display's rate on desktops). |
| `batterySaver` | `boolean` | `false` | — | — | — | Save power: at most 30 frames a second whatever frameCap says, the render scale at most 0.75, and automatic quality as on. For a phone on battery or a player who asks for it. |

#### Resolution and automatic quality

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `renderScale` | `number` | `1` | 0.5 … 2 | — | — | The 3D scene's resolution as a fraction of the display's: below 1 it is drawn smaller and scaled up (bilinearly, or temporally under TAA), above 1 supersampled; the UI, screen text and screen sprites stay at the display's resolution. With autoQuality it is the most automatic quality draws at. |
| `autoQuality` | `boolean` | `true` | — | — | — | Lower the render scale (and then, where autoPresetSteps allows, the graphics preset) while frames take longer than the frame cap's interval, and raise them again when there is room, with hysteresis so it never oscillates. False draws at renderScale and RenderSettings exactly. See `looks §17`. |
| `autoMinScale` | `number` | `0.5` | 0.5 … 2 | — | — | The least render scale automatic quality lowers to. |
| `autoPresetSteps` | `number` | `1` | 0 … 2 | — | — | Graphics preset steps automatic quality may take below the game's RenderSettings once the scale is at autoMinScale: 1 caps every quality field at the medium row, 2 at the low row; 0 never touches the quality. RenderSettings keeps its values: the cap applies when the frame is drawn. |

#### Cloth detail

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clothSubdivisions` | `number` | `0` | 0 … 8 | — | — | Draw segments per coarse Cloth or SoftBody edge where the device has compute and vertex storage: 0 selects two, 1 to 8 that many (its square of triangles per solve triangle, crack-free Phong tessellation). Points, snapshots and hashes are unchanged. Automatic quality caps it at two, then one. See `looks §17`. |

#### Window and GPU

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fullscreen` | `boolean` | `false` | — | — | — | The page fills the screen: the web shell asks for fullscreen (on the player's click on the settings page, or their next press when the browser wanted a gesture) and leaves it when false. The page leaving fullscreen itself (Escape) turns it off. |
| `powerPreference` | `PowerPreference` | `"highPerformance"` | — | — | — | The GPU the next load asks for (highPerformance or lowPower): the adapter is chosen once, at the boot, so a change shows DisplayInfo.restartNeeded until the page loads again. |

```ogt
struct DisplaySettings {
  doc "The player's graphics settings, as a settings page shows them and og.render.setDisplay sets them: the graphics preset, the frame cap, battery saver, render scale and automatic quality, fullscreen and the GPU's power preference. Presentation only: none of it is in the world, so hashes, snapshots, saves and replays never see it. A native run draws every frame at render scale 1 whatever it says, though it draws the preset. og.render.display(world).settings is the whole object to save."
  guide "looks §17"
  desc true
  group "Preset and frame rate"
  field quality Option.GraphicsQuality "The player's graphics preset: its row of quality fields (og.render.applyQuality's table) is drawn in place of RenderSettings', which keep the game's values. Null draws the game's own RenderSettings quality."
  field frameCap FrameCap "The most frames a second (auto: 60 on phones, the display's rate on desktops)."
  field batterySaver bool "Save power: at most 30 frames a second whatever frameCap says, the render scale at most 0.75, and automatic quality as on. For a phone on battery or a player who asks for it."
  group "Resolution and automatic quality"
  field renderScale f32 "The 3D scene's resolution as a fraction of the display's: below 1 it is drawn smaller and scaled up (bilinearly, or temporally under TAA), above 1 supersampled; the UI, screen text and screen sprites stay at the display's resolution. With autoQuality it is the most automatic quality draws at." {
    default 1
    min 0.5
    max 2
  }
  field autoQuality bool "Lower the render scale (and then, where autoPresetSteps allows, the graphics preset) while frames take longer than the frame cap's interval, and raise them again when there is room, with hysteresis so it never oscillates. False draws at renderScale and RenderSettings exactly." {
    guide "looks §17"
    default true
  }
  field autoMinScale f32 "The least render scale automatic quality lowers to." {
    default 0.5
    min 0.5
    max 2
  }
  field autoPresetSteps u32 "Graphics preset steps automatic quality may take below the game's RenderSettings once the scale is at autoMinScale: 1 caps every quality field at the medium row, 2 at the low row; 0 never touches the quality. RenderSettings keeps its values: the cap applies when the frame is drawn." {
    default 1
    min 0
    max 2
  }
  group "Cloth detail"
  field clothSubdivisions u32 "Draw segments per coarse Cloth or SoftBody edge where the device has compute and vertex storage: 0 selects two, 1 to 8 that many (its square of triangles per solve triangle, crack-free Phong tessellation). Points, snapshots and hashes are unchanged. Automatic quality caps it at two, then one." {
    guide "looks §17"
    default 0
    min 0
    max 8
  }
  group "Window and GPU"
  field fullscreen bool "The page fills the screen: the web shell asks for fullscreen (on the player's click on the settings page, or their next press when the browser wanted a gesture) and leaves it when false. The page leaving fullscreen itself (Escape) turns it off."
  field powerPreference PowerPreference "The GPU the next load asks for (highPerformance or lowPower): the adapter is chosen once, at the boot, so a change shows DisplayInfo.restartNeeded until the page loads again."
}
```

<sub>Module `render` line 2959</sub>
