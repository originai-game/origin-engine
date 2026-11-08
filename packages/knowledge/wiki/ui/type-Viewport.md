<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

Module [`ui`](../ui.md) · [index](../index.md)

<a id="type-Viewport"></a>

### `Viewport` (struct)

The play surface the UI lays out in, in og-ui units (CSS px over UiTheme.unit) and device pixels per unit.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |
| `dpr` | `number` | `1` | — | — | — | Device pixels per CSS pixel; rectangles snap to device pixels. |
| `safeArea` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | The device's unsafe margins (notch, home indicator); the root pads by them, and CSS env(safe-area-inset-*) reads them. |
| `smallWidth` | `number` | `0` | — | `cssPx` | optional | The surface's width with the browser's own controls (address bar, toolbars) shown, as CSS's small viewport: what svw, svi and svmin read; vw and dvw read width. Omitted or 0 is width (native hosts have no such controls). |
| `smallHeight` | `number` | `0` | — | `cssPx` | optional | Its height with the browser's controls shown (CSS svh); omitted or 0 is height. |
| `largeWidth` | `number` | `0` | — | `cssPx` | optional | Its width with the browser's controls hidden (CSS lvw); omitted or 0 is width. |
| `largeHeight` | `number` | `0` | — | `cssPx` | optional | Its height with the browser's controls hidden (CSS lvh); omitted or 0 is height. |
| `coarsePointer` | `boolean` | `false` | — | — | — | A touch screen or another coarse pointer. |
| `reducedMotion` | `boolean` | `false` | — | — | optional | The page prefers reduced motion (prefers-reduced-motion: reduce, from the web shell): the UI's theme only fades, whatever UiDesc.reducedMotion says. Omitted is false. |
| `highContrast` | `boolean` | `false` | — | — | optional | The page prefers more contrast (prefers-contrast: more, from the web shell): the UI's theme is the high-contrast one, whatever UiDesc.contrast says. Omitted is false. |
| `reducedTransparency` | `boolean` | `false` | — | — | optional | The page prefers less transparency (prefers-reduced-transparency: reduce, from the web shell): glass parts are opaque surfaces and the scrim and translucent fills flat, whatever UiDesc.reducedTransparency says (UiTheme.reducedTransparency). Omitted is false. |
| `macKeys` | `boolean` | `false` | — | — | optional | Text controls take macOS's editing keys: Command with A, C, X and Z, Command+←/→ to the line's ends (↑/↓ the text's), Option+←/→ and Option+Backspace by words, Control+A and E to the line's ends. Off: Control takes the shortcuts and moves by words (Alt too), Home and End go to the line's ends. The web shell sets it from the browser's platform (Apple's); a native host sets it on macOS, so a script that holds Ctrl or Meta edits as a person on that OS would (og.ui.previewViewport pins either). Omitted is false. |
| `covered` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | optional | Where the host's own control covers the surface: on a published page the web shell's round Back to the Portal, top left (about 12, 12 to 52, 52 CSS px; its name shows beside it for the first seconds of play). Anchored groups keep out of it, moved along the edge they sit on (a top-left group to its right), so the HUD templates never draw under it; a game drawing its own corner reads it from og.ui.frameViewport. Empty (omitted) is none: native hosts and pages without Back. |
| `breakpoint` | `Breakpoint` | `"desktop"` | — | — | — | Its size class: phone portrait or landscape, tablet, desktop. |
| `hud` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | optional | The HUD band (read only): the safe area at most UiDesc.hudMaxAspect times its height wide, centred, where anchored groups and toasts lay out; a component that places itself (a HUD of the game's own) reads it so it sits with them on an ultrawide. Ignored where a host passes a viewport in. |

```ogt
struct Viewport {
  doc "The play surface the UI lays out in, in og-ui units (CSS px over UiTheme.unit) and device pixels per unit."
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
  field dpr f32 "Device pixels per CSS pixel; rectangles snap to device pixels." {
    default 1
  }
  field safeArea Insets "The device's unsafe margins (notch, home indicator); the root pads by them, and CSS env(safe-area-inset-*) reads them."
  field smallWidth f32 "The surface's width with the browser's own controls (address bar, toolbars) shown, as CSS's small viewport: what svw, svi and svmin read; vw and dvw read width. Omitted or 0 is width (native hosts have no such controls)." {
    unit cssPx
    optional true
  }
  field smallHeight f32 "Its height with the browser's controls shown (CSS svh); omitted or 0 is height." {
    unit cssPx
    optional true
  }
  field largeWidth f32 "Its width with the browser's controls hidden (CSS lvw); omitted or 0 is width." {
    unit cssPx
    optional true
  }
  field largeHeight f32 "Its height with the browser's controls hidden (CSS lvh); omitted or 0 is height." {
    unit cssPx
    optional true
  }
  field coarsePointer bool "A touch screen or another coarse pointer."
  field reducedMotion bool "The page prefers reduced motion (prefers-reduced-motion: reduce, from the web shell): the UI's theme only fades, whatever UiDesc.reducedMotion says. Omitted is false." {
    optional true
  }
  field highContrast bool "The page prefers more contrast (prefers-contrast: more, from the web shell): the UI's theme is the high-contrast one, whatever UiDesc.contrast says. Omitted is false." {
    optional true
  }
  field reducedTransparency bool "The page prefers less transparency (prefers-reduced-transparency: reduce, from the web shell): glass parts are opaque surfaces and the scrim and translucent fills flat, whatever UiDesc.reducedTransparency says (UiTheme.reducedTransparency). Omitted is false." {
    optional true
  }
  field macKeys bool "Text controls take macOS's editing keys: Command with A, C, X and Z, Command+←/→ to the line's ends (↑/↓ the text's), Option+←/→ and Option+Backspace by words, Control+A and E to the line's ends. Off: Control takes the shortcuts and moves by words (Alt too), Home and End go to the line's ends. The web shell sets it from the browser's platform (Apple's); a native host sets it on macOS, so a script that holds Ctrl or Meta edits as a person on that OS would (og.ui.previewViewport pins either). Omitted is false." {
    optional true
  }
  field covered UiRect "Where the host's own control covers the surface: on a published page the web shell's round Back to the Portal, top left (about 12, 12 to 52, 52 CSS px; its name shows beside it for the first seconds of play). Anchored groups keep out of it, moved along the edge they sit on (a top-left group to its right), so the HUD templates never draw under it; a game drawing its own corner reads it from og.ui.frameViewport. Empty (omitted) is none: native hosts and pages without Back." {
    optional true
  }
  field breakpoint Breakpoint "Its size class: phone portrait or landscape, tablet, desktop."
  field hud UiRect "The HUD band (read only): the safe area at most UiDesc.hudMaxAspect times its height wide, centred, where anchored groups and toasts lay out; a component that places itself (a HUD of the game's own) reads it so it sits with them on an ultrawide. Ignored where a host passes a viewport in." {
    optional true
  }
}
```

<sub>Module `ui` line 327</sub>
