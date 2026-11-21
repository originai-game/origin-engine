<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui` module

The engine's UI: TSX components (@origingame/engine/ui's mount) styled by Tailwind classes and the style pack, laid out and drawn over the frame, and answered to the probe (dump_ui, touch_move_to). These calls configure it and read it back.

Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Input is decided against the previous tick's boxes, so a scripted touch_move_to and a click agree.

- **TypeScript**: functions are the namespace `og.ui`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module ui {
  doc "The engine's UI: TSX components (@origingame/engine/ui's mount) styled by Tailwind classes and the style pack, laid out and drawn over the frame, and answered to the probe (dump_ui, touch_move_to). These calls configure it and read it back."
  doc "Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Input is decided against the previous tick's boxes, so a scripted touch_move_to and a click agree."
  # declarations follow, each on its own page
```

<sub>Module `ui` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`TextAlign`](ui/type-TextAlign.md) | enum |  | How the lines of a label align. |
| [`Scheme`](ui/type-Scheme.md) | enum |  | The palette: games run on a stage, so dark is the default. |
| [`Insets`](ui/type-Insets.md) | struct |  | Padding or margins on four sides. |
| [`Viewport`](ui/type-Viewport.md) | struct |  | The play surface the UI lays out in, in CSS px and device pixels per CSS px. |
| [`UiDesc`](ui/type-UiDesc.md) | desc struct |  | How the UI looks: the scheme, its fonts, the contrast, the seed colours its palette is derived from, motion, transparency, the icon atlas, the style and the sounds. |
| [`UiSeeds`](ui/type-UiSeeds.md) | desc struct |  | The colours a UI palette is derived from (packages/design/tokens/seed.css): every ground, ink, accent, state, status and team colour follows them, each scheme keeping its text and team inks at 4.5:1,… |
| [`UiSounds`](ui/type-UiSounds.md) | desc struct |  | Sounds the UI plays through og-audio (the ui bus) on its events; null is silent. |
| [`UiTheme`](ui/type-UiTheme.md) | struct |  | What the UI draws with, read-only: the style pack's theme (the default pack with the game's over it) in the scheme and contrast in effect, in the design tokens' own names, and the player's settings i… |
| [`og.ui.configure`](ui/fn-configure.md) | function | `(world: World, desc?: UiDesc): Result<void>` | Set how the UI looks (the scheme, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. |
| [`og.ui.viewport`](ui/fn-viewport.md) | function | `(world: World): Viewport` | This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in CSS px and device pixels per CSS px, from what the host reports. |
| [`og.ui.theme`](ui/fn-theme.md) | function | `(world: World): UiTheme` | The style values the UI draws with. |
| [`TextSpan`](ui/type-TextSpan.md) | struct |  | A named attribute range in plain text for a store and the Svelte text spans binding. |
| [`Breakpoint`](ui/type-Breakpoint.md) | enum |  | The play surface's size class. |
| [`ColorVision`](ui/type-ColorVision.md) | enum |  | A colour vision deficiency (UiDesc.colorVision; CSS @media (-og-color-vision: <name>)). |
| [`ColorVisionFilter`](ui/type-ColorVisionFilter.md) | enum |  | What UiDesc.colorVision does to the frame. |
| [`Contrast`](ui/type-Contrast.md) | enum |  | The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour). |
| [`UiRect`](ui/type-UiRect.md) | struct |  | A rectangle of the play surface. |
| [`IconAtlasDesc`](ui/type-IconAtlasDesc.md) | desc struct |  | The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font). |
| [`UiInlineImage`](ui/type-UiInlineImage.md) | desc struct |  | A picture rich text draws inline by name ([img=<name>]). |
| [`UiPointer`](ui/type-UiPointer.md) | struct |  | The primary pointer as the UI reads it this tick. |
| [`og.ui.framePointer`](ui/fn-framePointer.md) | function | `(): UiPointer` | The primary pointer of the frame being built, without the world: for a component that hit-tests a shape it draws itself (a pie's wedges, a wheel's slices) by angle and radius rather than by rectangle… |
| [`og.ui.frameCursor`](ui/fn-frameCursor.md) | function | `(): Cursor` | The pointer's shape the last frame resolved (never auto): the `cursor-*` class of the TSX element the pointer is over, or of its nearest ancestor that names one, else default; none while the virtual … |
| [`og.ui.frameViewport`](ui/fn-frameViewport.md) | function | `(): Viewport` | The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI. |
| [`og.ui.hasIcon`](ui/fn-hasIcon.md) | function | `(name: string): boolean` | Whether the icon atlas has an icon of this name (components draw a fallback otherwise). |
| [`og.ui.seconds`](ui/fn-seconds.md) | function | `(): number` | The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock). |
| [`og.ui.frameDirection`](ui/fn-frameDirection.md) | function | `(): string` | The writing direction of the frame being built, `ltr` or `rtl` (og.i18n's locale sets it): code that draws its own arrows, chevrons or progress along a line mirrors them by it; layout mirrors rows an… |
| [`og.ui.frameTheme`](ui/fn-frameTheme.md) | function | `(): UiTheme` | The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the components and patterns). |
| [`VirtualCursorDesc`](ui/type-VirtualCursorDesc.md) | desc struct |  | A pad's virtual cursor (docs/ui.md §2.1). |
| [`og.ui.setVirtualCursor`](ui/fn-setVirtualCursor.md) | function | `(desc: VirtualCursorDesc \| null): void` | A pad's virtual cursor, for a screen built for the pointer (a map, a shop grid, a strategy HUD) played on a pad: pad 0's left stick moves it instead of the focus, slowing and sticking over controls; … |
| [`VirtualCursor`](ui/type-VirtualCursor.md) | struct |  | Where a pad's virtual cursor is. |
| [`og.ui.virtualCursor`](ui/fn-virtualCursor.md) | function | `(): VirtualCursor \| null` | The virtual cursor as the last frame left it; null when none is declared. |
| [`og.ui.feedback`](ui/fn-feedback.md) | function | `(event: string, rise: number): void` | Play a UI event's style feedback (docs/ui.md §3.1): a random sound from its set, never repeating the last, with volume and pitch from the set's ranges, and its haptic pattern at the player's strength… |
| [`og.ui.setHaptics`](ui/fn-setHaptics.md) | function | `(intensity: number): void` | The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. |
| [`og.ui.tick`](ui/fn-tick.md) | function | `(): number` | Frames the UI has built: its clock, for animations by the tick. |
| [`og.ui.previewViewport`](ui/fn-previewViewport.md) | function | `(world: World, viewport: Viewport \| null): void` | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. |
| [`Cursor`](ui/type-Cursor.md) | enum |  | The pointer's shape over an element (CSS cursor's keywords, CSS UI 4 §5.1, docs/ui.md §2), inherited by its subtree. |
| [`Notation`](ui/type-Notation.md) | enum |  | How a number is written. |
| [`NumberFormatDesc`](ui/type-NumberFormatDesc.md) | desc struct |  | How og.ui.formatNumber and the counters write a number. |
| [`og.ui.formatNumber`](ui/fn-formatNumber.md) | function | `(value: number, format?: NumberFormatDesc): string` | A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. |
| [`VectorArt`](ui/type-VectorArt.md) | handle |  | A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill. |
| [`UiMaterial`](ui/type-UiMaterial.md) | handle |  | A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through. |
| [`VectorArt()`](#column-VectorArt) | column kind |  | A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill. |
| [`UiMaterial()`](#column-UiMaterial) | column kind |  | A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint. |
| [`UiSource`](ui/type-UiSource.md) | interface |  | A store's stable checkpoint identity. |
| [`UiCollectionDesc`](ui/type-UiCollectionDesc.md) | struct |  | A keyed collection's identity rule. |
| [`UiScope`](ui/type-UiScope.md) | interface |  | Named presentation stores for one world, following the svelte/store contract. |
| [`og.ui.scope`](ui/fn-scope.md) | function | `(world: World, id: string): UiScope` | The entry to named svelte/store-compatible presentation data and retained document mounts. |
| [`UiActionTrigger`](ui/type-UiActionTrigger.md) | enum |  | When a UI action fires. |
| [`UiActionDesc`](ui/type-UiActionDesc.md) | desc struct |  | A named UI action (docs/ui.md §2.1) bound to og-input's controls: a component asks for an action, not a key. |
| [`UiDevice`](ui/type-UiDevice.md) | enum |  | The device a glyph set is for: the one the player used last. |
| [`UiGlyph`](ui/type-UiGlyph.md) | struct |  | An action's control on the device used last. |
| [`og.ui.defineAction`](ui/fn-defineAction.md) | function | `(world: World, desc?: UiActionDesc): Result<void>` | Define (or redefine) a UI action. |
| [`og.ui.actionFired`](ui/fn-actionFired.md) | function | `(name: string): boolean` | The action fired this tick: on its press, its release or once its hold is reached, as its trigger says (a document's `on action` fires on the top layer that takes input). |
| [`og.ui.actionHeld`](ui/fn-actionHeld.md) | function | `(name: string): boolean` | A hold action is held: down for its holdMs, or its toggle is on. |
| [`og.ui.glyph`](ui/fn-glyph.md) | function | `(world: World, action: string): UiGlyph \| null` | An action's glyph on the device used last: its first binding there. |
| [`og.ui.frameGlyph`](ui/fn-frameGlyph.md) | function | `(action: string): UiGlyph \| null` | An action's glyph on the device used last, as og.ui.glyph answers it, without the world: for a component that shows a prompt (a tab row's bumpers, the action bar) and only builds UI. |
| [`og.ui.device`](ui/fn-device.md) | function | `(world: World): UiDevice` | The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer. |
| [`og.ui.setDevice`](ui/fn-setDevice.md) | function | `(world: World, device: UiDevice): void` | Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on. |

## Column kinds

Each is a factory, `VectorArt(value?, options?)`, that makes one field of a component or resource layout.

| Column | Value | Typed array | Stride | Doc |
|---|---|---|---|---|
| <a id="column-VectorArt"></a>`VectorArt` | `VectorArt` | `Uint32Array` | 2 | A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill. |
| <a id="column-UiMaterial"></a>`UiMaterial` | `UiMaterial` | `Uint32Array` | 2 | A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint. |

```ogt
column VectorArt {
  doc "A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill."
  array Uint32Array
  stride 2
  value VectorArt
}
```

<sub>Module `ui` line 470</sub>

```ogt
column UiMaterial {
  doc "A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint."
  array Uint32Array
  stride 2
  value UiMaterial
}
```

<sub>Module `ui` line 477</sub>
