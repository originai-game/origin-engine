<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui` module

The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.

The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.

Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.

- **TypeScript**: functions are the namespace `og.ui`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and name](index.md).

```ogt
module ui {
  doc "The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it."
  doc "The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick."
  doc "Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi(\"Slot\", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it."
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 2153</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Direction`](#type-Direction) | enum | Which way a container lays out its children. |
| [`Align`](#type-Align) | enum | Where children sit along the main axis. |
| [`Cross`](#type-Cross) | enum | Where children sit across the main axis. |
| [`TextAlign`](#type-TextAlign) | enum | How the lines of a label align. |
| [`FontRole`](#type-FontRole) | enum | Which configured font text uses; display and mono fall back to body. |
| [`ButtonVariant`](#type-ButtonVariant) | enum | Button looks, as the design language's buttons. |
| [`ControlSize`](#type-ControlSize) | enum | Control heights from the density (h-control-sm, -md, -lg). |
| [`Scheme`](#type-Scheme) | enum | The palette: games run on a stage, so dark is the default. |
| [`DensityMode`](#type-DensityMode) | enum | Control sizes: auto picks touch sizes on a coarse pointer or a play surface under 768 px, compact otherwise. |
| [`ThemeColor`](#type-ThemeColor) | enum | A semantic colour of the design tokens (tokens.json `color`, in the current scheme). |
| [`TypeStepName`](#type-TypeStepName) | enum | A step of the type ramp (tokens.json `font.scale`). |
| [`SizeRange`](#type-SizeRange) | struct | The bounds of a fit or grow size. |
| [`FixedSize`](#type-FixedSize) | struct | A fixed size. |
| [`PercentSize`](#type-PercentSize) | struct | A size relative to the parent. |
| [`Size`](#type-Size) | union | A size along one axis. Absent (null) is fit: the content's size. |
| [`Insets`](#type-Insets) | struct | Padding or margins on four sides. |
| [`Paint`](#type-Paint) | desc struct | A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white. |
| [`ElementDesc`](#type-ElementDesc) | desc struct | A container: its layout and its look. |
| [`LabelDesc`](#type-LabelDesc) | desc struct | How a label draws its text. |
| [`ButtonDesc`](#type-ButtonDesc) | desc struct | A button's look. |
| [`PanelDesc`](#type-PanelDesc) | desc struct | A card: surface, the card radius, padded and spaced by the tokens. |
| [`ScrollDesc`](#type-ScrollDesc) | desc struct | A vertical scroll container: the wheel or a drag scrolls it, it clips its content, a capsule indicator shows the position. |
| [`ImageDesc`](#type-ImageDesc) | desc struct | A texture region, or a solid rectangle without a texture. |
| [`ProgressDesc`](#type-ProgressDesc) | desc struct | A progress bar. |
| [`SliderDesc`](#type-SliderDesc) | desc struct | A slider's range and width. |
| [`FieldDesc`](#type-FieldDesc) | desc struct | A one-line text field. |
| [`Response`](#type-Response) | struct | What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick. |
| [`SliderResult`](#type-SliderResult) | struct | A slider's response and its value after this tick's input. |
| [`ToggleResult`](#type-ToggleResult) | struct | A toggle's response and its state after this tick's input. |
| [`FieldResult`](#type-FieldResult) | struct | A text field's response and its text after this tick's input. |
| [`Viewport`](#type-Viewport) | struct | The play surface the UI lays out in. |
| [`UiDesc`](#type-UiDesc) | desc struct | How the UI looks: the scheme, the density and its fonts. |
| [`TypeStep`](#type-TypeStep) | struct | One step of the type ramp. |
| [`ControlSizes`](#type-ControlSizes) | struct | The control, row and header heights and the gutter of the density in effect. |
| [`UiTheme`](#type-UiTheme) | struct | The design tokens the UI draws with, read-only. |
| [`UiBuild`](#type-UiBuild) | callback | Builds a container's children: ui calls made inside it nest under the container. |
| [`og.ui.configure`](#fn-configure) | function | Set the scheme, the density and the fonts, typically in GameDesc.start. Takes effect from the next tick. |
| [`og.ui.viewport`](#fn-viewport) | function | This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them. |
| [`og.ui.theme`](#fn-theme) | function | The design tokens the UI draws with. |
| [`og.ui.open`](#fn-open) | function | Open a container; the calls until its close() build its children. |
| [`og.ui.openPanel`](#fn-openPanel) | function | Open a card, closed by close(): og.ui.panel without the callback. |
| [`og.ui.openScroll`](#fn-openScroll) | function | Open a vertical scroll container, closed by close(): og.ui.scroll without the callback. |
| [`og.ui.close`](#fn-close) | function | Close the open container. |
| [`og.ui.row`](#fn-row) | function | A row built by `build`: open(id, {...desc, direction: row}), build(), close(). |
| [`og.ui.column`](#fn-column) | function | A column built by `build`: open(id, {...desc, direction: column}), build(), close(). |
| [`og.ui.panel`](#fn-panel) | function | A card built by `build`: openPanel(id, desc), build(), close(). |
| [`og.ui.scroll`](#fn-scroll) | function | A vertical scroll container built by `build`: openScroll(id, desc), build(), close(). |
| [`og.ui.label`](#fn-label) | function | MSDF text that wraps at its width (word breaks, any CJK character, long words between characters) unless nowrap. The probe reads its text. |
| [`og.ui.button`](#fn-button) | function | A capsule button with a label; `clicked` is a press and a release inside it. The probe reads its label. |
| [`og.ui.image`](#fn-image) | function | A texture region, or a solid rectangle. |
| [`og.ui.progress`](#fn-progress) | function | A capsule track filled to a fraction. |
| [`og.ui.slider`](#fn-slider) | function | A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick. |
| [`og.ui.toggle`](#fn-toggle) | function | A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label. |
| [`og.ui.textField`](#fn-textField) | function | One line of text: a press focuses it with the caret at the press, typed text inserts at the caret, Backspace, Delete, arrows, Home and End edit, Enter submits and leaves, Escape leaves. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty. |
| [`og.ui.tooltip`](#fn-tooltip) | function | A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text. |
| [`og.ui.idi`](#fn-idi) | function | The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`. |

## Functions

<a id="fn-configure"></a>

### `og.ui.configure`

Set the scheme, the density and the fonts, typically in GameDesc.start. Takes effect from the next tick.

```ts
og.ui.configure(world: World, desc?: UiDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI is configured. |
| `desc` | `UiDesc` | yes | The scheme, density and fonts. |

**Returns** `Result<void>`: Refused for a dead font.

Binding symbol `og_ui_configure` · publish builds: kept

```ogt
fn configure {
  doc "Set the scheme, the density and the fonts, typically in GameDesc.start. Takes effect from the next tick."
  param world World "The world whose UI is configured."
  param desc UiDesc "The scheme, density and fonts."
  returns Result.Void "Refused for a dead font."
}
```

<sub>`api.ogt` line 2567</sub>

<a id="fn-viewport"></a>

### `og.ui.viewport`

This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them.

```ts
og.ui.viewport(world: World): Viewport
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Viewport`.

Binding symbol `og_ui_viewport` · publish builds: kept

```ogt
fn viewport {
  doc "This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them."
  param world World "The world."
  returns Viewport
}
```

<sub>`api.ogt` line 2574</sub>

<a id="fn-theme"></a>

### `og.ui.theme`

The design tokens the UI draws with.

```ts
og.ui.theme(world: World): UiTheme
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `UiTheme`.

Binding symbol `og_ui_theme` · publish builds: kept

```ogt
fn theme {
  doc "The design tokens the UI draws with."
  param world World "The world."
  returns UiTheme
}
```

<sub>`api.ogt` line 2580</sub>

<a id="fn-open"></a>

### `og.ui.open`

Open a container; the calls until its close() build its children.

```ts
og.ui.open(id: string, desc?: ElementDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id, unique among its siblings. |
| `desc` | `ElementDesc` | yes | Its layout and look. |

**Returns** `void`.

Binding symbol `og_ui_open` · publish builds: kept

```ogt
fn open {
  doc "Open a container; the calls until its close() build its children."
  param id String "Its id, unique among its siblings."
  param desc ElementDesc "Its layout and look."
  returns Void
}
```

<sub>`api.ogt` line 2586</sub>

<a id="fn-openPanel"></a>

### `og.ui.openPanel`

Open a card, closed by close(): og.ui.panel without the callback.

```ts
og.ui.openPanel(id: string, desc?: PanelDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PanelDesc` | yes | Its layout and look. |

**Returns** `void`.

Binding symbol `og_ui_open_panel` · publish builds: kept

```ogt
fn openPanel {
  doc "Open a card, closed by close(): og.ui.panel without the callback."
  param id String "Its id."
  param desc PanelDesc "Its layout and look."
  returns Void
}
```

<sub>`api.ogt` line 2593</sub>

<a id="fn-openScroll"></a>

### `og.ui.openScroll`

Open a vertical scroll container, closed by close(): og.ui.scroll without the callback.

```ts
og.ui.openScroll(id: string, desc?: ScrollDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ScrollDesc` | yes | Its size and spacing. |

**Returns** `void`.

Binding symbol `og_ui_open_scroll` · publish builds: kept

```ogt
fn openScroll {
  doc "Open a vertical scroll container, closed by close(): og.ui.scroll without the callback."
  param id String "Its id."
  param desc ScrollDesc "Its size and spacing."
  returns Void
}
```

<sub>`api.ogt` line 2600</sub>

<a id="fn-close"></a>

### `og.ui.close`

Close the open container.

```ts
og.ui.close(): Response
```

**Returns** `Response`: Its response, for an interactive container; the zero response otherwise.

Binding symbol `og_ui_close` · publish builds: kept

```ogt
fn close {
  doc "Close the open container."
  returns Response "Its response, for an interactive container; the zero response otherwise."
}
```

<sub>`api.ogt` line 2607</sub>

<a id="fn-row"></a>

### `og.ui.row`

A row built by `build`: open(id, {...desc, direction: row}), build(), close().

```ts
og.ui.row(id: string, desc: ElementDesc, build: UiBuild): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ElementDesc` | no | Its layout and look; direction is row. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `Response`: Its response, for an interactive row.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn row {
  doc "A row built by `build`: open(id, {...desc, direction: row}), build(), close()."
  host false
  param id String "Its id."
  param desc ElementDesc "Its layout and look; direction is row."
  param build UiBuild "Makes its children."
  returns Response "Its response, for an interactive row."
}
```

<sub>`api.ogt` line 2612</sub>

<a id="fn-column"></a>

### `og.ui.column`

A column built by `build`: open(id, {...desc, direction: column}), build(), close().

```ts
og.ui.column(id: string, desc: ElementDesc, build: UiBuild): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ElementDesc` | no | Its layout and look; direction is column. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `Response`: Its response, for an interactive column.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn column {
  doc "A column built by `build`: open(id, {...desc, direction: column}), build(), close()."
  host false
  param id String "Its id."
  param desc ElementDesc "Its layout and look; direction is column."
  param build UiBuild "Makes its children."
  returns Response "Its response, for an interactive column."
}
```

<sub>`api.ogt` line 2621</sub>

<a id="fn-panel"></a>

### `og.ui.panel`

A card built by `build`: openPanel(id, desc), build(), close().

```ts
og.ui.panel(id: string, desc: PanelDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PanelDesc` | no | Its layout and look. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn panel {
  doc "A card built by `build`: openPanel(id, desc), build(), close()."
  host false
  param id String "Its id."
  param desc PanelDesc "Its layout and look."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>`api.ogt` line 2630</sub>

<a id="fn-scroll"></a>

### `og.ui.scroll`

A vertical scroll container built by `build`: openScroll(id, desc), build(), close().

```ts
og.ui.scroll(id: string, desc: ScrollDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ScrollDesc` | no | Its size and spacing. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn scroll {
  doc "A vertical scroll container built by `build`: openScroll(id, desc), build(), close()."
  host false
  param id String "Its id."
  param desc ScrollDesc "Its size and spacing."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>`api.ogt` line 2639</sub>

<a id="fn-label"></a>

### `og.ui.label`

MSDF text that wraps at its width (word breaks, any CJK character, long words between characters) unless nowrap. The probe reads its text.

```ts
og.ui.label(id: string, text: string, desc?: LabelDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The text. |
| `desc` | `LabelDesc` | yes | Type step, ink, alignment, wrapping, width and font. |

**Returns** `void`.

Binding symbol `og_ui_label` · publish builds: kept

```ogt
fn label {
  doc "MSDF text that wraps at its width (word breaks, any CJK character, long words between characters) unless nowrap. The probe reads its text."
  param id String "Its id."
  param text String "The text."
  param desc LabelDesc "Type step, ink, alignment, wrapping, width and font."
  returns Void
}
```

<sub>`api.ogt` line 2648</sub>

<a id="fn-button"></a>

### `og.ui.button`

A capsule button with a label; `clicked` is a press and a release inside it. The probe reads its label.

```ts
og.ui.button(id: string, text: string, desc?: ButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label. |
| `desc` | `ButtonDesc` | yes | Variant, size, disabled and width. |

**Returns** `Response`.

Binding symbol `og_ui_button` · publish builds: kept

```ogt
fn button {
  doc "A capsule button with a label; `clicked` is a press and a release inside it. The probe reads its label."
  param id String "Its id."
  param text String "The label."
  param desc ButtonDesc "Variant, size, disabled and width."
  returns Response
}
```

<sub>`api.ogt` line 2656</sub>

<a id="fn-image"></a>

### `og.ui.image`

A texture region, or a solid rectangle.

```ts
og.ui.image(id: string, desc?: ImageDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ImageDesc` | yes | Texture, region, tint and size. |

**Returns** `void`.

Binding symbol `og_ui_image` · publish builds: kept

```ogt
fn image {
  doc "A texture region, or a solid rectangle."
  param id String "Its id."
  param desc ImageDesc "Texture, region, tint and size."
  returns Void
}
```

<sub>`api.ogt` line 2664</sub>

<a id="fn-progress"></a>

### `og.ui.progress`

A capsule track filled to a fraction.

```ts
og.ui.progress(id: string, value: number, desc?: ProgressDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | How full it is, from 0 to 1. |
| `desc` | `ProgressDesc` | yes | Its width. |

**Returns** `void`.

Binding symbol `og_ui_progress` · publish builds: kept

```ogt
fn progress {
  doc "A capsule track filled to a fraction."
  param id String "Its id."
  param value f32 "How full it is, from 0 to 1."
  param desc ProgressDesc "Its width."
  returns Void
}
```

<sub>`api.ogt` line 2671</sub>

<a id="fn-slider"></a>

### `og.ui.slider`

A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick.

```ts
og.ui.slider(id: string, value: number, desc?: SliderDesc): SliderResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `SliderDesc` | yes | Range, step and width. |

**Returns** `SliderResult`.

Binding symbol `og_ui_slider` · publish builds: kept

```ogt
fn slider {
  doc "A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick."
  param id String "Its id."
  param value f32 "The value from game state."
  param desc SliderDesc "Range, step and width."
  returns SliderResult
}
```

<sub>`api.ogt` line 2679</sub>

<a id="fn-toggle"></a>

### `og.ui.toggle`

A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label.

```ts
og.ui.toggle(id: string, on: boolean, label: string): ToggleResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `on` | `boolean` | no | The state from game state. |
| `label` | `string` | no | The label. |

**Returns** `ToggleResult`.

Binding symbol `og_ui_toggle` · publish builds: kept

```ogt
fn toggle {
  doc "A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label."
  param id String "Its id."
  param on bool "The state from game state."
  param label String "The label."
  returns ToggleResult
}
```

<sub>`api.ogt` line 2687</sub>

<a id="fn-textField"></a>

### `og.ui.textField`

One line of text: a press focuses it with the caret at the press, typed text inserts at the caret, Backspace, Delete, arrows, Home and End edit, Enter submits and leaves, Escape leaves. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

```ts
og.ui.textField(id: string, value: string, desc?: FieldDesc): FieldResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `FieldDesc` | yes | Placeholder, width and length limit. |

**Returns** `FieldResult`.

Binding symbol `og_ui_text_field` · publish builds: kept

```ogt
fn textField {
  doc "One line of text: a press focuses it with the caret at the press, typed text inserts at the caret, Backspace, Delete, arrows, Home and End edit, Enter submits and leaves, Escape leaves. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc FieldDesc "Placeholder, width and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 2695</sub>

<a id="fn-tooltip"></a>

### `og.ui.tooltip`

A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text.

```ts
og.ui.tooltip(text: string): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `text` | `string` | no | The text. |

**Returns** `void`.

Binding symbol `og_ui_tooltip` · publish builds: kept

```ogt
fn tooltip {
  doc "A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text."
  param text String "The text."
  returns Void
}
```

<sub>`api.ogt` line 2703</sub>

<a id="fn-idi"></a>

### `og.ui.idi`

The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`.

```ts
og.ui.idi(name: string, index: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The name. |
| `index` | `number` | no | The index. |

**Returns** `string`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn idi {
  doc "The id of the index-th of several siblings of one name: `idi(\"Slot\", 3)` is `Slot#3`."
  host false
  param name String "The name."
  param index u32 "The index."
  returns String
}
```

<sub>`api.ogt` line 2709</sub>

## Types

<a id="type-Direction"></a>

### `Direction` (enum)

Which way a container lays out its children.

A string: one of the variants below. The zero value is `"row"`.

| Variant | Doc |
|---|---|
| `"row"` | Left to right. |
| `"column"` | Top to bottom. |

```ogt
enum Direction {
  doc "Which way a container lays out its children."
  variant row "Left to right."
  variant column "Top to bottom."
}
```

<sub>`api.ogt` line 2158</sub>

<a id="type-Align"></a>

### `Align` (enum)

Where children sit along the main axis.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | At the start. |
| `"center"` | Centred. |
| `"end"` | At the end. |
| `"spaceBetween"` | The free space split evenly between children. |

```ogt
enum Align {
  doc "Where children sit along the main axis."
  variant start "At the start."
  variant center "Centred."
  variant end "At the end."
  variant spaceBetween "The free space split evenly between children."
}
```

<sub>`api.ogt` line 2164</sub>

<a id="type-Cross"></a>

### `Cross` (enum)

Where children sit across the main axis.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | At the start. |
| `"center"` | Centred. |
| `"end"` | At the end. |
| `"stretch"` | Stretched to the container's inner size. |

```ogt
enum Cross {
  doc "Where children sit across the main axis."
  variant start "At the start."
  variant center "Centred."
  variant end "At the end."
  variant stretch "Stretched to the container's inner size."
}
```

<sub>`api.ogt` line 2172</sub>

<a id="type-TextAlign"></a>

### `TextAlign` (enum)

How the lines of a label align.

A string: one of the variants below. The zero value is `"start"`.

| Variant | Doc |
|---|---|
| `"start"` | To the start (left). |
| `"center"` | Centred. |
| `"end"` | To the end (right). |

```ogt
enum TextAlign {
  doc "How the lines of a label align."
  variant start "To the start (left)."
  variant center "Centred."
  variant end "To the end (right)."
}
```

<sub>`api.ogt` line 2180</sub>

<a id="type-FontRole"></a>

### `FontRole` (enum)

Which configured font text uses; display and mono fall back to body.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | The body font. |
| `"display"` | The display font. |
| `"mono"` | The monospaced font. |

```ogt
enum FontRole {
  doc "Which configured font text uses; display and mono fall back to body."
  variant body "The body font."
  variant display "The display font."
  variant mono "The monospaced font."
}
```

<sub>`api.ogt` line 2187</sub>

<a id="type-ButtonVariant"></a>

### `ButtonVariant` (enum)

Button looks, as the design language's buttons.

A string: one of the variants below. The zero value is `"secondary"`.

| Variant | Doc |
|---|---|
| `"secondary"` | A neutral fill (the default). |
| `"primary"` | The amber action: one per screen. |
| `"quiet"` | Text until hovered. |
| `"danger"` | A destructive action: danger ink on a neutral fill. |

```ogt
enum ButtonVariant {
  doc "Button looks, as the design language's buttons."
  variant secondary "A neutral fill (the default)."
  variant primary "The amber action: one per screen."
  variant quiet "Text until hovered."
  variant danger "A destructive action: danger ink on a neutral fill."
}
```

<sub>`api.ogt` line 2194</sub>

<a id="type-ControlSize"></a>

### `ControlSize` (enum)

Control heights from the density (h-control-sm, -md, -lg).

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | The medium control height. |
| `"small"` | The small control height. |
| `"large"` | The large control height. |

```ogt
enum ControlSize {
  doc "Control heights from the density (h-control-sm, -md, -lg)."
  variant medium "The medium control height."
  variant small "The small control height."
  variant large "The large control height."
}
```

<sub>`api.ogt` line 2202</sub>

<a id="type-Scheme"></a>

### `Scheme` (enum)

The palette: games run on a stage, so dark is the default.

A string: one of the variants below. The zero value is `"dark"`.

| Variant | Doc |
|---|---|
| `"dark"` | The dark palette. |
| `"light"` | The light palette. |

```ogt
enum Scheme {
  doc "The palette: games run on a stage, so dark is the default."
  variant dark "The dark palette."
  variant light "The light palette."
}
```

<sub>`api.ogt` line 2209</sub>

<a id="type-DensityMode"></a>

### `DensityMode` (enum)

Control sizes: auto picks touch sizes on a coarse pointer or a play surface under 768 px, compact otherwise.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Touch sizes on a coarse pointer or a narrow surface, compact otherwise. |
| `"compact"` | Compact controls. |
| `"comfortable"` | Comfortable controls. |
| `"touch"` | Touch-sized controls. |

```ogt
enum DensityMode {
  doc "Control sizes: auto picks touch sizes on a coarse pointer or a play surface under 768 px, compact otherwise."
  variant auto "Touch sizes on a coarse pointer or a narrow surface, compact otherwise."
  variant compact "Compact controls."
  variant comfortable "Comfortable controls."
  variant touch "Touch-sized controls."
}
```

<sub>`api.ogt` line 2215</sub>

<a id="type-ThemeColor"></a>

### `ThemeColor` (enum)

A semantic colour of the design tokens (tokens.json `color`, in the current scheme).

A string: one of the variants below. The zero value is `"accent"`.

| Variant | Doc |
|---|---|
| `"accent"` | accent: the amber action colour. |
| `"accentActive"` | accent-active. |
| `"accentFg"` | accent-fg: ink on the accent. |
| `"accentHover"` | accent-hover. |
| `"accentInk"` | accent-ink: the accent as text. |
| `"accentSoft"` | accent-soft: a translucent accent wash. |
| `"bg"` | bg: the page. |
| `"bgSubtle"` | bg-subtle. |
| `"border"` | border. |
| `"borderStrong"` | border-strong. |
| `"danger"` | danger. |
| `"fg"` | fg: the primary ink. |
| `"fgInverse"` | fg-inverse. |
| `"fgMuted"` | fg-muted. |
| `"fgSubtle"` | fg-subtle: placeholders. |
| `"fill"` | fill: neutral control fills. |
| `"fillStrong"` | fill-strong. |
| `"focus"` | focus: the focus ring. |
| `"info"` | info. |
| `"materialControl"` | material-control. |
| `"materialControlHover"` | material-control-hover. |
| `"materialEdge"` | material-edge. |
| `"materialGlow"` | material-glow. |
| `"materialRegular"` | material-regular. |
| `"materialRimSoft"` | material-rim-soft. |
| `"materialSheen"` | material-sheen. |
| `"materialShine"` | material-shine. |
| `"materialThick"` | material-thick. |
| `"materialThin"` | material-thin. |
| `"scrim"` | scrim. |
| `"shadow"` | shadow. |
| `"sheen"` | sheen. |
| `"stage"` | stage: the game stage. |
| `"stageBorder"` | stage-border. |
| `"stageDanger"` | stage-danger. |
| `"stageFg"` | stage-fg. |
| `"stageMuted"` | stage-muted. |
| `"stageRaised"` | stage-raised. |
| `"stageSurface"` | stage-surface. |
| `"success"` | success. |
| `"surface"` | surface: cards and panels. |
| `"surfaceHover"` | surface-hover. |
| `"surfaceRaised"` | surface-raised: tooltips. |
| `"thumb"` | thumb: slider and switch thumbs. |
| `"warn"` | warn. |

```ogt
enum ThemeColor {
  doc "A semantic colour of the design tokens (tokens.json `color`, in the current scheme)."
  variant accent "accent: the amber action colour."
  variant accentActive "accent-active."
  variant accentFg "accent-fg: ink on the accent."
  variant accentHover "accent-hover."
  variant accentInk "accent-ink: the accent as text."
  variant accentSoft "accent-soft: a translucent accent wash."
  variant bg "bg: the page."
  variant bgSubtle "bg-subtle."
  variant border "border."
  variant borderStrong "border-strong."
  variant danger "danger."
  variant fg "fg: the primary ink."
  variant fgInverse "fg-inverse."
  variant fgMuted "fg-muted."
  variant fgSubtle "fg-subtle: placeholders."
  variant fill "fill: neutral control fills."
  variant fillStrong "fill-strong."
  variant focus "focus: the focus ring."
  variant info "info."
  variant materialControl "material-control."
  variant materialControlHover "material-control-hover."
  variant materialEdge "material-edge."
  variant materialGlow "material-glow."
  variant materialRegular "material-regular."
  variant materialRimSoft "material-rim-soft."
  variant materialSheen "material-sheen."
  variant materialShine "material-shine."
  variant materialThick "material-thick."
  variant materialThin "material-thin."
  variant scrim "scrim."
  variant shadow "shadow."
  variant sheen "sheen."
  variant stage "stage: the game stage."
  variant stageBorder "stage-border."
  variant stageDanger "stage-danger."
  variant stageFg "stage-fg."
  variant stageMuted "stage-muted."
  variant stageRaised "stage-raised."
  variant stageSurface "stage-surface."
  variant success "success."
  variant surface "surface: cards and panels."
  variant surfaceHover "surface-hover."
  variant surfaceRaised "surface-raised: tooltips."
  variant thumb "thumb: slider and switch thumbs."
  variant warn "warn."
}
```

<sub>`api.ogt` line 2223</sub>

<a id="type-TypeStepName"></a>

### `TypeStepName` (enum)

A step of the type ramp (tokens.json `font.scale`).

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | text-body, the default. |
| `"xs"` | text-xs. |
| `"meta"` | text-meta. |
| `"caption"` | text-caption. |
| `"sm"` | text-sm. |
| `"reading"` | text-reading. |
| `"base"` | text-base. |
| `"lg"` | text-lg. |
| `"xl"` | text-xl. |
| `"xl2"` | text-2xl. |
| `"xl3"` | text-3xl. |
| `"xl4"` | text-4xl. |
| `"xl5"` | text-5xl. |
| `"xl6"` | text-6xl. |
| `"xl7"` | text-7xl. |
| `"xl8"` | text-8xl. |
| `"displaySm"` | text-display-sm. |
| `"displayMd"` | text-display-md. |
| `"displayLg"` | text-display-lg. |
| `"displayXl"` | text-display-xl. |
| `"displayXl2"` | text-display-2xl. |

```ogt
enum TypeStepName {
  doc "A step of the type ramp (tokens.json `font.scale`)."
  variant body "text-body, the default."
  variant xs "text-xs."
  variant meta "text-meta."
  variant caption "text-caption."
  variant sm "text-sm."
  variant reading "text-reading."
  variant base "text-base."
  variant lg "text-lg."
  variant xl "text-xl."
  variant xl2 "text-2xl."
  variant xl3 "text-3xl."
  variant xl4 "text-4xl."
  variant xl5 "text-5xl."
  variant xl6 "text-6xl."
  variant xl7 "text-7xl."
  variant xl8 "text-8xl."
  variant displaySm "text-display-sm."
  variant displayMd "text-display-md."
  variant displayLg "text-display-lg."
  variant displayXl "text-display-xl."
  variant displayXl2 "text-display-2xl."
}
```

<sub>`api.ogt` line 2272</sub>

<a id="type-SizeRange"></a>

### `SizeRange` (struct)

The bounds of a fit or grow size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | `cssPx` | optional | The smallest size. |
| `max` | `number` | `0` | — | `cssPx` | optional | The largest size; absent is unbounded. |

```ogt
struct SizeRange {
  doc "The bounds of a fit or grow size."
  field min f32 "The smallest size." {
    optional true
    unit cssPx
  }
  field max f32 "The largest size; absent is unbounded." {
    optional true
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2297</sub>

<a id="type-FixedSize"></a>

### `FixedSize` (struct)

A fixed size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `px` | `number` | `0` | — | `cssPx` | — | The size. |

```ogt
struct FixedSize {
  doc "A fixed size."
  field px f32 "The size." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2309</sub>

<a id="type-PercentSize"></a>

### `PercentSize` (struct)

A size relative to the parent.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fraction` | `number` | `0` | — | `ratio` | — | The fraction of the parent's inner size. |

```ogt
struct PercentSize {
  doc "A size relative to the parent."
  field fraction f32 "The fraction of the parent's inner size." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 2316</sub>

<a id="type-Size"></a>

### `Size` (union)

A size along one axis. Absent (null) is fit: the content's size.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "fit"` | [`SizeRange`](#type-SizeRange) | The content's size, within the range. |
| `kind: "grow"` | [`SizeRange`](#type-SizeRange) | The content's size, then a share of the parent's free space, within the range. |
| `kind: "fixed"` | [`FixedSize`](#type-FixedSize) | Exactly this size. |
| `kind: "percent"` | [`PercentSize`](#type-PercentSize) | A fraction of the parent's inner size. |

```ogt
union Size {
  doc "A size along one axis. Absent (null) is fit: the content's size."
  tag kind
  case fit SizeRange "The content's size, within the range."
  case grow SizeRange "The content's size, then a share of the parent's free space, within the range."
  case fixed FixedSize "Exactly this size."
  case percent PercentSize "A fraction of the parent's inner size."
}
```

<sub>`api.ogt` line 2323</sub>

<a id="type-Insets"></a>

### `Insets` (struct)

Padding or margins on four sides.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `left` | `number` | `0` | — | `cssPx` | — | The left side. |
| `top` | `number` | `0` | — | `cssPx` | — | The top side. |
| `right` | `number` | `0` | — | `cssPx` | — | The right side. |
| `bottom` | `number` | `0` | — | `cssPx` | — | The bottom side. |

```ogt
struct Insets {
  doc "Padding or margins on four sides."
  field left f32 "The left side." {
    unit cssPx
  }
  field top f32 "The top side." {
    unit cssPx
  }
  field right f32 "The right side." {
    unit cssPx
  }
  field bottom f32 "The bottom side." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2332</sub>

<a id="type-Paint"></a>

### `Paint` (desc struct)

A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `token` | `ThemeColor \| null` | `null` | — | — | — | A semantic colour of the theme. |
| `color` | `Color \| null` | `null` | — | — | — | An sRGB colour as authored, when no token is given. |
| `alpha` | `number` | `1` | 0 … 1 | — | — | The opacity. |

```ogt
struct Paint {
  doc "A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white."
  desc true
  field token Option.ThemeColor "A semantic colour of the theme."
  field color Option.Color "An sRGB colour as authored, when no token is given."
  field alpha f32 "The opacity." {
    default 1
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 2348</sub>

<a id="type-ElementDesc"></a>

### `ElementDesc` (desc struct)

A container: its layout and its look.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `wrap` | `boolean` | `false` | — | — | — | Break a row into lines at its inner width. |
| `scroll` | `boolean` | `false` | — | — | — | Scroll its content vertically and clip it. |
| `interactive` | `boolean` | `false` | — | — | — | Report a Response from close(), as a button does. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |

```ogt
struct ElementDesc {
  doc "A container: its layout and its look."
  desc true
  field direction Direction "Which way it lays out its children."
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Insets "Space inside its edges."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field wrap bool "Break a row into lines at its inner width."
  field scroll bool "Scroll its content vertically and clip it."
  field interactive bool "Report a Response from close(), as a button does."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2360</sub>

<a id="type-LabelDesc"></a>

### `LabelDesc` (desc struct)

How a label draws its text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is fg. |
| `align` | `TextAlign` | `"start"` | — | — | — | How the lines align. |
| `nowrap` | `boolean` | `false` | — | — | — | Keep to one line (breaking only at `\n`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font. |

```ogt
struct LabelDesc {
  doc "How a label draws its text."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is fg."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
}
```

<sub>`api.ogt` line 2381</sub>

<a id="type-ButtonDesc"></a>

### `ButtonDesc` (desc struct)

A button's look.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `variant` | `ButtonVariant` | `"secondary"` | — | — | — | The look. |
| `size` | `ControlSize` | `"medium"` | — | — | — | The control height. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the label. |

```ogt
struct ButtonDesc {
  doc "A button's look."
  desc true
  field variant ButtonVariant "The look."
  field size ControlSize "The control height."
  field disabled bool "Neutral and inert."
  field width Option.Size "Its width; null fits the label."
}
```

<sub>`api.ogt` line 2392</sub>

<a id="type-PanelDesc"></a>

### `PanelDesc` (desc struct)

A card: surface, the card radius, padded and spaced by the tokens.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `row` | `boolean` | `false` | — | — | — | Lay children left to right (the default is top to bottom). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets \| null` | `null` | — | — | — | Space inside; null is space.lg on every side. |
| `gap` | `number \| null` | `null` | — | — | — | Space between children; null is space.md. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null is surface. |

```ogt
struct PanelDesc {
  doc "A card: surface, the card radius, padded and spaced by the tokens."
  desc true
  field row bool "Lay children left to right (the default is top to bottom)."
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Option.Insets "Space inside; null is space.lg on every side."
  field gap Option.f32 "Space between children; null is space.md."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null is surface."
}
```

<sub>`api.ogt` line 2401</sub>

<a id="type-ScrollDesc"></a>

### `ScrollDesc` (desc struct)

A vertical scroll container: the wheel or a drag scrolls it, it clips its content, a capsule indicator shows the position.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |

```ogt
struct ScrollDesc {
  doc "A vertical scroll container: the wheel or a drag scrolls it, it clips its content, a capsule indicator shows the position."
  desc true
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Insets "Space inside."
  field gap f32 "Space between children." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2414</sub>

<a id="type-ImageDesc"></a>

### `ImageDesc` (desc struct)

A texture region, or a solid rectangle without a texture.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The texture; null draws a solid rectangle of the tint. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies the texture; null is white. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits. |

```ogt
struct ImageDesc {
  doc "A texture region, or a solid rectangle without a texture."
  desc true
  field texture Option.Texture "The texture; null draws a solid rectangle of the tint."
  field uv Option.Array.f32.4 "The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture."
  field tint Option.Paint "Multiplies the texture; null is white."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
}
```

<sub>`api.ogt` line 2425</sub>

<a id="type-ProgressDesc"></a>

### `ProgressDesc` (desc struct)

A progress bar.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows into the free space. |

```ogt
struct ProgressDesc {
  doc "A progress bar."
  desc true
  field width Option.Size "Its width; null grows into the free space."
}
```

<sub>`api.ogt` line 2435</sub>

<a id="type-SliderDesc"></a>

### `SliderDesc` (desc struct)

A slider's range and width.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The smallest value; min equal to max (the zero value) means 0 to 1. |
| `max` | `number` | `0` | — | — | — | The largest value. |
| `step` | `number` | `0` | ≥ 0 | — | — | Snap to multiples of step from min; 0 is continuous. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |

```ogt
struct SliderDesc {
  doc "A slider's range and width."
  desc true
  field min f32 "The smallest value; min equal to max (the zero value) means 0 to 1."
  field max f32 "The largest value."
  field step f32 "Snap to multiples of step from min; 0 is continuous." {
    min 0
  }
  field width Option.Size "Its width; null grows."
}
```

<sub>`api.ogt` line 2441</sub>

<a id="type-FieldDesc"></a>

### `FieldDesc` (desc struct)

A one-line text field.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown, dimmed, while the field is empty. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `maxChars` | `number` | `0` | — | — | — | The most characters it takes; 0 is no limit. |

```ogt
struct FieldDesc {
  doc "A one-line text field."
  desc true
  field placeholder String "Shown, dimmed, while the field is empty."
  field width Option.Size "Its width; null grows."
  field maxChars u32 "The most characters it takes; 0 is no limit."
}
```

<sub>`api.ogt` line 2452</sub>

<a id="type-Response"></a>

### `Response` (struct)

What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hovered` | `boolean` | `false` | — | — | — | The pointer is over it and no other element holds the press. |
| `pressed` | `boolean` | `false` | — | — | — | The press that started on it is still held. |
| `clicked` | `boolean` | `false` | — | — | — | Pressed and released inside it. |
| `changed` | `boolean` | `false` | — | — | — | Its value changed (slider, toggle, text field). |
| `focused` | `boolean` | `false` | — | — | — | It has keyboard focus (text field). |
| `submitted` | `boolean` | `false` | — | — | — | Enter was pressed in it (text field). |

```ogt
struct Response {
  doc "What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick."
  field hovered bool "The pointer is over it and no other element holds the press."
  field pressed bool "The press that started on it is still held."
  field clicked bool "Pressed and released inside it."
  field changed bool "Its value changed (slider, toggle, text field)."
  field focused bool "It has keyboard focus (text field)."
  field submitted bool "Enter was pressed in it (text field)."
}
```

<sub>`api.ogt` line 2460</sub>

<a id="type-SliderResult"></a>

### `SliderResult` (struct)

A slider's response and its value after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false }` | — | — | — | What happened to it. |
| `value` | `number` | `0` | — | — | — | The value, to keep in game state and pass back next tick. |

```ogt
struct SliderResult {
  doc "A slider's response and its value after this tick's input."
  field response Response "What happened to it."
  field value f32 "The value, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 2470</sub>

<a id="type-ToggleResult"></a>

### `ToggleResult` (struct)

A toggle's response and its state after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false }` | — | — | — | What happened to it. |
| `on` | `boolean` | `false` | — | — | — | The state, to keep in game state and pass back next tick. |

```ogt
struct ToggleResult {
  doc "A toggle's response and its state after this tick's input."
  field response Response "What happened to it."
  field on bool "The state, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 2476</sub>

<a id="type-FieldResult"></a>

### `FieldResult` (struct)

A text field's response and its text after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false }` | — | — | — | What happened to it. |
| `value` | `string` | `""` | — | — | — | The text, to keep in game state and pass back next tick. |

```ogt
struct FieldResult {
  doc "A text field's response and its text after this tick's input."
  field response Response "What happened to it."
  field value String "The text, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 2482</sub>

<a id="type-Viewport"></a>

### `Viewport` (struct)

The play surface the UI lays out in.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |
| `dpr` | `number` | `1` | — | — | — | Device pixels per CSS pixel; rectangles snap to device pixels. |
| `safeArea` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | The device's unsafe margins (notch, home indicator); the root pads by them. |
| `coarsePointer` | `boolean` | `false` | — | — | — | A touch screen or another coarse pointer. |

```ogt
struct Viewport {
  doc "The play surface the UI lays out in."
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
  field dpr f32 "Device pixels per CSS pixel; rectangles snap to device pixels." {
    default 1
  }
  field safeArea Insets "The device's unsafe margins (notch, home indicator); the root pads by them."
  field coarsePointer bool "A touch screen or another coarse pointer."
}
```

<sub>`api.ogt` line 2488</sub>

<a id="type-UiDesc"></a>

### `UiDesc` (desc struct)

How the UI looks: the scheme, the density and its fonts.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette. |
| `density` | `DensityMode` | `"auto"` | — | — | — | The control sizes. |
| `font` | `Font \| null` | `null` | — | — | — | The body font, from og.load.font; null measures text at half an em per character and draws none. |
| `displayFont` | `Font \| null` | `null` | — | — | — | The display font; null uses the body font. |
| `monoFont` | `Font \| null` | `null` | — | — | — | The monospaced font; null uses the body font. |

```ogt
struct UiDesc {
  doc "How the UI looks: the scheme, the density and its fonts."
  desc true
  field scheme Scheme "The palette."
  field density DensityMode "The control sizes."
  field font Option.Font "The body font, from og.load.font; null measures text at half an em per character and draws none."
  field displayFont Option.Font "The display font; null uses the body font."
  field monoFont Option.Font "The monospaced font; null uses the body font."
}
```

<sub>`api.ogt` line 2503</sub>

<a id="type-TypeStep"></a>

### `TypeStep` (struct)

One step of the type ramp.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | — | `cssPx` | — | The font size. |
| `lineHeight` | `number` | `0` | — | `cssPx` | — | The leading of Latin text. |
| `trackingEm` | `number` | `0` | — | — | — | Latin tracking, in em. |
| `cjkLineHeight` | `number` | `0` | — | `cssPx` | — | The leading text with Chinese, Japanese or Korean characters takes. |

```ogt
struct TypeStep {
  doc "One step of the type ramp."
  field size f32 "The font size." {
    unit cssPx
  }
  field lineHeight f32 "The leading of Latin text." {
    unit cssPx
  }
  field trackingEm f32 "Latin tracking, in em."
  field cjkLineHeight f32 "The leading text with Chinese, Japanese or Korean characters takes." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2513</sub>

<a id="type-ControlSizes"></a>

### `ControlSizes` (struct)

The control, row and header heights and the gutter of the density in effect.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `controlSm` | `number` | `0` | — | `cssPx` | — | h-control-sm. |
| `controlMd` | `number` | `0` | — | `cssPx` | — | h-control-md. |
| `controlLg` | `number` | `0` | — | `cssPx` | — | h-control-lg. |
| `gutter` | `number` | `0` | — | `cssPx` | — | The gutter. |
| `header` | `number` | `0` | — | `cssPx` | — | The header height. |
| `row` | `number` | `0` | — | `cssPx` | — | The row height. |
| `section` | `number` | `0` | — | `cssPx` | — | The section spacing. |

```ogt
struct ControlSizes {
  doc "The control, row and header heights and the gutter of the density in effect."
  field controlSm f32 "h-control-sm." {
    unit cssPx
  }
  field controlMd f32 "h-control-md." {
    unit cssPx
  }
  field controlLg f32 "h-control-lg." {
    unit cssPx
  }
  field gutter f32 "The gutter." {
    unit cssPx
  }
  field header f32 "The header height." {
    unit cssPx
  }
  field row f32 "The row height." {
    unit cssPx
  }
  field section f32 "The section spacing." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 2527</sub>

<a id="type-UiTheme"></a>

### `UiTheme` (struct)

The design tokens the UI draws with, read-only.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette in use. |
| `palette` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored). |
| `space` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels. |
| `radius` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The radius scale by name (control, field, block, card, panel, sheet, button), in CSS pixels. |
| `typeRamp` | `Readonly<Record<string, TypeStep>>` | `{}` | — | — | — | Each TypeStepName's step. |
| `sizes` | `ControlSizes` | `{ controlSm: 0, controlMd: 0, controlLg: 0, gutter: 0, header: 0, row: 0, section: 0 }` | — | — | — | The control sizes of the density in effect. |
| `durationsMs` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds. |

```ogt
struct UiTheme {
  doc "The design tokens the UI draws with, read-only."
  field scheme Scheme "The palette in use."
  field palette Map.String "Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored)."
  field space Map.f32 "The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels."
  field radius Map.f32 "The radius scale by name (control, field, block, card, panel, sheet, button), in CSS pixels."
  field typeRamp Map.TypeStep "Each TypeStepName's step."
  field sizes ControlSizes "The control sizes of the density in effect."
  field durationsMs Map.f32 "The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds."
}
```

<sub>`api.ogt` line 2552</sub>

<a id="type-UiBuild"></a>

### `UiBuild` (callback)

Builds a container's children: ui calls made inside it nest under the container.

Game code supplies this function.

```ts
type UiBuild = () => void
```

```ogt
callback UiBuild {
  doc "Builds a container's children: ui calls made inside it nest under the container."
}
```

<sub>`api.ogt` line 2563</sub>
