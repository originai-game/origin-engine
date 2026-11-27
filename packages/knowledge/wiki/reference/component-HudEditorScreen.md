<!-- Generated from crates/og-api/api.ogt and checked input/component declarations by og api gen. -->

<a id="component-HudEditorScreen"></a>
### `HudEditorScreen`

The HUD editor's frame: a hint at the top until the player moves something, an interactive safe-area stage with pointer-draggable and keyboard-nudgeable HUD regions, an inspector dock for scale, opacity and visibility, and any custom editor content the caller supplies.

`import { HudEditorScreen } from "@origingame/engine/ui/components"` (kit `screens`, `packages/engine/src/ui/screens/hud-editor.tsx`).

| Prop | Type | What |
|---|---|---|
| `class?` | `string \| undefined` | Classes after the screen's own (a placement, an inset): the root fills the layer it is mounted in by default. |
| `showHint?` | `boolean` | The hint at the top shows (false once the player has moved something). |
| `regions?` | `ReadonlyArray<HudRegion> \| undefined` | Editable HUD regions; when provided, renders interactive draggable boxes and an inspector dock. |
| `selected?` | `string \| undefined` | Currently selected region id; omit to let the screen track selection locally. |
| `safeMargin?` | `boolean \| undefined` | Draw the safe-area guide frame around the editable stage (default true when regions are given). |
| `gridSize?` | `number \| undefined` | Snap step for dragging and keyboard nudging: values >= 1 snap in CSS px; values in (0, 1) snap in normalized fraction; 0 disables snapping. Default 0.025. |
| `onSelect?` | `((e: UiEvent, id: string) => void) \| undefined` | Called when a region is selected (or cleared with ""). |
| `onRegionChange?` | `((e: UiEvent, region: HudRegion) => void) \| undefined` | Called whenever a region is dragged, nudged, scaled, faded or toggled. |
| `onReset?` | `((e: UiEvent) => void) \| undefined` | Called when the player presses Reset. |
| `onDone?` | `((e: UiEvent) => void) \| undefined` | Called when the player presses Done or the `back` action. |
| `editorEditor?` | `(() => JSX.Element) \| undefined` | Content the caller supplies (was the editorEditor snippet). |
| `renderRegion?` | `((region: HudRegion) => JSX.Element) \| undefined` | Draws the live HUD element inside each region's tile, so the player places what they will see. |

Each `HudRegion`:

| Field | Type | What |
|---|---|---|
| `id` | `string` |  |
| `label` | `string` | Human-readable name shown on the editable tile and in the inspector. |
| `x` | `number` | Horizontal centre across the editable safe area, 0 to 1. |
| `y` | `number` | Vertical centre across the editable safe area, 0 to 1. |
| `width?` | `number \| undefined` | Optional width fraction of the safe area, 0 to 1. |
| `height?` | `number \| undefined` | Optional height fraction of the safe area, 0 to 1. |
| `scale?` | `number \| undefined` | Display scale factor (default 1, inspector range 0.5 to 1.5). |
| `opacity?` | `number \| undefined` | Display opacity (default 1, inspector range 0.2 to 1). |
| `hidden?` | `boolean \| undefined` | When true, the HUD element is hidden in gameplay and shown dimmed in the editor. |

Pack slots: `Panel.frame`, `Screen.frame`.

Fixtures: `default`, `regions`.

Actions: `back` (Done).
