<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui` module

The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, themes it with the platform's design tokens, draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.

The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.

Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.

- **TypeScript**: functions are the namespace `og.ui`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Skill**: `og-ui`, which teaches the module (how much of it to use, and how to prove it).
- **Index**: [every module and name](index.md).

```ogt
module ui {
  # declarations follow, each quoted below
```

<sub>`api.ogt` line 7099</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Direction`](#type-Direction) | enum | Which way a container lays out its children. |
| [`Align`](#type-Align) | enum | Where children sit along the main axis. |
| [`Cross`](#type-Cross) | enum | Where children sit across the main axis. |
| [`TextAlign`](#type-TextAlign) | enum | How the lines of a label align. |
| [`FontRole`](#type-FontRole) | enum | Which configured font text uses; display and mono fall back to body, rounded to display, then body. |
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
| [`ScrollAxes`](#type-ScrollAxes) | enum | Which ways a scroll container scrolls. |
| [`ScrollDesc`](#type-ScrollDesc) | desc struct | A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it. |
| [`ImageDesc`](#type-ImageDesc) | desc struct | A texture region, or a solid rectangle without a texture. |
| [`ProgressDesc`](#type-ProgressDesc) | desc struct | A progress bar. |
| [`SliderDesc`](#type-SliderDesc) | desc struct | A slider's range and width. |
| [`FieldDesc`](#type-FieldDesc) | desc struct | A one-line text field. |
| [`TextAreaDesc`](#type-TextAreaDesc) | desc struct | A multi-line text area. |
| [`Response`](#type-Response) | struct | What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick. |
| [`SliderResult`](#type-SliderResult) | struct | A slider's response and its value after this tick's input. |
| [`ToggleResult`](#type-ToggleResult) | struct | A toggle's response and its state after this tick's input. |
| [`FieldResult`](#type-FieldResult) | struct | A text field's response and its text after this tick's input. |
| [`Viewport`](#type-Viewport) | struct | The play surface the UI lays out in. |
| [`UiDesc`](#type-UiDesc) | desc struct | How the UI looks: the scheme, the density, its fonts, the contrast, the game's accent, motion and the icon atlas. |
| [`UiSounds`](#type-UiSounds) | desc struct | Sounds the UI plays through og-audio (the ui bus) on its events; null is silent. |
| [`SkinPart`](#type-SkinPart) | enum | An element class a skin dresses with a nine-slice frame. |
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
| [`RichTextDesc`](#type-RichTextDesc) | desc struct | How a rich text paragraph draws: its defaults, which its markup's spans override. |
| [`RichTextResult`](#type-RichTextResult) | struct | What a rich text's links did this tick. |
| [`og.ui.richText`](#fn-richText) | function | A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the skin), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning. |
| [`og.ui.label`](#fn-label) | function | MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text. |
| [`og.ui.button`](#fn-button) | function | A capsule button with a label; `clicked` is a press and a release inside it. The probe reads its label. |
| [`og.ui.image`](#fn-image) | function | A texture region, or a solid rectangle. |
| [`og.ui.progress`](#fn-progress) | function | A capsule track filled to a fraction. |
| [`og.ui.slider`](#fn-slider) | function | A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick. |
| [`og.ui.toggle`](#fn-toggle) | function | A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label. |
| [`og.ui.textField`](#fn-textField) | function | One line of text: a press focuses it with the caret at the press and a drag selects; typed text (og.input's TextInput events, else the keys) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects; Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty. |
| [`og.ui.textArea`](#fn-textArea) | function | Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, the text wrapped between words at the area's width and scrolled to keep the caret's line in view. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty. |
| [`og.ui.tooltip`](#fn-tooltip) | function | A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text. |
| [`og.ui.idi`](#fn-idi) | function | The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`. |
| [`Anchor`](#type-Anchor) | enum | A point of the safe area an anchored group sits at. |
| [`TransitionKind`](#type-TransitionKind) | enum | How a transition moves its content in and out. Under reduced motion every kind only fades. |
| [`Tone`](#type-Tone) | enum | A status tone, as the design language's badges and toasts. |
| [`MeterKind`](#type-MeterKind) | enum | What a meter measures, which sets its colour. |
| [`Breakpoint`](#type-Breakpoint) | enum | The play surface's size class. |
| [`Contrast`](#type-Contrast) | enum | The theme variant: high contrast remaps muted inks, fills and borders to stronger tokens and outlines every control (no new colour). |
| [`HueName`](#type-HueName) | enum | One of the twelve category hues of the tokens, for a game's accent. |
| [`NavIntent`](#type-NavIntent) | enum | A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give. |
| [`UiRect`](#type-UiRect) | struct | A rectangle of the play surface. |
| [`IconAtlasDesc`](#type-IconAtlasDesc) | desc struct | The icon atlas: a texture of equal cells in a grid, named in reading order. |
| [`AnchorDesc`](#type-AnchorDesc) | desc struct | An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls). |
| [`TransitionDesc`](#type-TransitionDesc) | desc struct | How a transition moves, timed in ticks from the motion tokens. |
| [`Presence`](#type-Presence) | struct | Where a transition or dialog is this tick. |
| [`ModalDesc`](#type-ModalDesc) | desc struct | A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked. |
| [`ModalResult`](#type-ModalResult) | struct | A dialog's answer this tick. |
| [`SpotlightDesc`](#type-SpotlightDesc) | desc struct | A spotlight: the surface dimmed except a hole around a target. |
| [`ListDesc`](#type-ListDesc) | desc struct | A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row. |
| [`ListRange`](#type-ListRange) | struct | The rows (or cells) to build this tick: first up to, not including, last. |
| [`ListResult`](#type-ListResult) | struct | What happened in a list or grid this tick. |
| [`GridDesc`](#type-GridDesc) | desc struct | A virtualised grid of equal cells (inventories, level select), scrolling vertically. |
| [`TabsDesc`](#type-TabsDesc) | desc struct | A segmented control of tabs. |
| [`ChoiceResult`](#type-ChoiceResult) | struct | A choice control's response and the chosen index after this tick's input. |
| [`SelectDesc`](#type-SelectDesc) | desc struct | A select: a trigger that opens a menu of options. |
| [`StepperDesc`](#type-StepperDesc) | desc struct | A number stepper's range, step and digits. |
| [`StepperResult`](#type-StepperResult) | struct | A stepper's response and value after this tick's input. |
| [`RadioDesc`](#type-RadioDesc) | desc struct | How a radio group lays out. |
| [`CheckListResult`](#type-CheckListResult) | struct | A checkbox list's response and values after this tick's input. |
| [`ToastDesc`](#type-ToastDesc) | desc struct | A toast's tone and time. |
| [`ChipDesc`](#type-ChipDesc) | desc struct | A chip's state. |
| [`MeterDesc`](#type-MeterDesc) | desc struct | A meter: a capsule track with a fill that animates its changes by the tick. |
| [`CooldownDesc`](#type-CooldownDesc) | desc struct | A cooldown ring's size and centre. |
| [`IconDesc`](#type-IconDesc) | desc struct | An icon's size and ink. |
| [`MapMarker`](#type-MapMarker) | desc struct | A marker on a mini-map. |
| [`MinimapDesc`](#type-MinimapDesc) | desc struct | A mini-map frame. |
| [`DialogueDesc`](#type-DialogueDesc) | desc struct | A dialogue box: a portrait slot, the speaker and the line typed out by the tick. |
| [`DialogueResult`](#type-DialogueResult) | struct | A dialogue box's answer this tick. |
| [`JoystickDesc`](#type-JoystickDesc) | desc struct | A virtual joystick. |
| [`JoystickResult`](#type-JoystickResult) | struct | A joystick's vector this tick. |
| [`DpadDesc`](#type-DpadDesc) | desc struct | A d-pad. |
| [`DpadResult`](#type-DpadResult) | struct | Which arms of a d-pad are held this tick. |
| [`ActionButtonDesc`](#type-ActionButtonDesc) | desc struct | A round touch action button (jump, fire). |
| [`SkeletonDesc`](#type-SkeletonDesc) | desc struct | A skeleton: the neutral fill standing in for content that is loading. |
| [`UiItemBuild`](#type-UiItemBuild) | callback | Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it. |
| [`og.ui.breakpoint`](#fn-breakpoint) | function | The size class of the play surface of the frame being built (desktop outside a frame). |
| [`og.ui.frameViewport`](#fn-frameViewport) | function | The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI. |
| [`og.ui.hasIcon`](#fn-hasIcon) | function | Whether the icon atlas has an icon of this name (templates draw a fallback otherwise). |
| [`og.ui.seconds`](#fn-seconds) | function | The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock). |
| [`og.ui.frameTheme`](#fn-frameTheme) | function | The design tokens of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates). |
| [`og.ui.openAnchor`](#fn-openAnchor) | function | Open an anchored group, closed by close(): og.ui.anchor without the callback. |
| [`og.ui.anchor`](#fn-anchor) | function | An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls. |
| [`og.ui.openTransition`](#fn-openTransition) | function | Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves. |
| [`og.ui.transition`](#fn-transition) | function | Content that enters and leaves: fade, slide or scale by the tick from the motion tokens. `build` runs while it is visible. |
| [`og.ui.openModal`](#fn-openModal) | function | Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`. |
| [`og.ui.closeModal`](#fn-closeModal) | function | Close a dialog opened with openModal (its panel and backdrop). |
| [`og.ui.modal`](#fn-modal) | function | A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`. |
| [`og.ui.spotlight`](#fn-spotlight) | function | A spotlight over everything (a tutorial step): the scrim on four sides of the hole, an amber edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`. |
| [`og.ui.bounds`](#fn-bounds) | function | Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it. |
| [`og.ui.openList`](#fn-openList) | function | Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList(). |
| [`og.ui.openGrid`](#fn-openGrid) | function | Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList(). |
| [`og.ui.listItem`](#fn-listItem) | function | Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`. |
| [`og.ui.gridItem`](#fn-gridItem) | function | Open cell `index` of the open grid (`Cell#index`): a slot on fill, focusable; its content follows until close(). Drops answer `dropped` as listItem's. |
| [`og.ui.closeList`](#fn-closeList) | function | Close a list or grid opened with openList or openGrid. |
| [`og.ui.dragging`](#fn-dragging) | function | The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer. |
| [`og.ui.list`](#fn-list) | function | A virtualised list built by `build(index)` for the rows in view. |
| [`og.ui.grid`](#fn-grid) | function | A virtualised grid built by `build(index)` for the cells in view. |
| [`og.ui.tabs`](#fn-tabs) | function | A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels. |
| [`og.ui.select`](#fn-select) | function | A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it. |
| [`og.ui.stepper`](#fn-stepper) | function | A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused. |
| [`og.ui.radio`](#fn-radio) | function | A radio group of `Option#i` rows: a ring, filled amber when chosen, and a label. |
| [`og.ui.checkbox`](#fn-checkbox) | function | A checkbox with a label; a press flips it. |
| [`og.ui.checkboxList`](#fn-checkboxList) | function | Checkboxes in a column (`Option#i`). |
| [`og.ui.chip`](#fn-chip) | function | A chip: a capsule on fill; selected inverts to fg. For filters and tags. |
| [`og.ui.badge`](#fn-badge) | function | A badge: a small status capsule in its tone. |
| [`og.ui.divider`](#fn-divider) | function | A hairline in the border colour across its parent. |
| [`og.ui.spacer`](#fn-spacer) | function | Empty space. |
| [`og.ui.keyValue`](#fn-keyValue) | function | A key (fg-muted) and its value (fg, at the end) on one row: stats, settings summaries, receipts. |
| [`og.ui.skeleton`](#fn-skeleton) | function | A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion). |
| [`og.ui.spinner`](#fn-spinner) | function | A loading spinner: eight dots around a circle, the bright one turning by the tick. |
| [`og.ui.icon`](#fn-icon) | function | An icon of the atlas (UiDesc.icons) by name; an unknown name draws an empty slot and warns. The probe reads the name. |
| [`og.ui.meter`](#fn-meter) | function | A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick. |
| [`og.ui.cooldown`](#fn-cooldown) | function | A cooldown: a disc whose amber ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not). |
| [`og.ui.minimap`](#fn-minimap) | function | A mini-map frame: the map in a border-strong frame, markers as dots (the player amber and ringed), an N mark and a zone label. |
| [`og.ui.dialogue`](#fn-dialogue) | function | A dialogue box: a portrait slot, the speaker in amber ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown. |
| [`og.ui.joystick`](#fn-joystick) | function | A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time. |
| [`og.ui.dpad`](#fn-dpad) | function | A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px. |
| [`og.ui.actionButton`](#fn-actionButton) | function | A round touch action button: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px. |
| [`og.ui.toast`](#fn-toast) | function | Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it. |
| [`og.ui.dismissToast`](#fn-dismissToast) | function | Start a toast's exit. |
| [`og.ui.navigate`](#fn-navigate) | function | Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. |
| [`og.ui.back`](#fn-back) | function | Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. |
| [`og.ui.setPlayers`](#fn-setPlayers) | function | Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it. |
| [`og.ui.focus`](#fn-focus) | function | The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none. |
| [`A11yNode`](#type-A11yNode) | struct | One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out). |
| [`A11yTree`](#type-A11yTree) | struct | The last frame's accessibility tree, built from the same nodes the UI draws and takes input on. |
| [`og.ui.a11y`](#fn-a11y) | function | The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements. |
| [`og.ui.setNavigation`](#fn-setNavigation) | function | Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick. |
| [`og.ui.memory`](#fn-memory) | function | A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped. |
| [`og.ui.remember`](#fn-remember) | function | Set a number memory (see memory). |
| [`og.ui.tick`](#fn-tick) | function | Frames the UI has built: its clock, for animations by the tick. |
| [`CounterDesc`](#type-CounterDesc) | desc struct | A number that counts to its value. |
| [`og.ui.counter`](#fn-counter) | function | A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a skin with motion.pop bumps it when it lands. Scores, currency, rewards. |
| [`BurstDesc`](#type-BurstDesc) | desc struct | Rays and sparks thrown from an element. |
| [`og.ui.burst`](#fn-burst) | function | Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion. |
| [`ShakeDesc`](#type-ShakeDesc) | desc struct | A shake of an element: three sideways swings decaying to rest. |
| [`og.ui.shake`](#fn-shake) | function | Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the skin's motion.shake is set. |
| [`ScreenWipe`](#type-ScreenWipe) | enum | How a screen transition covers the old screen and reveals the new one. |
| [`WipeSide`](#type-WipeSide) | enum | Where a wipe or a slide comes from. |
| [`ScreenTransitionDesc`](#type-ScreenTransitionDesc) | desc struct | How a screen transition looks. |
| [`ScreenTransitionResult`](#type-ScreenTransitionResult) | struct | Where a screen transition is this tick. |
| [`og.ui.screenTransition`](#fn-screenTransition) | function | A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade. |
| [`og.ui.previewViewport`](#fn-previewViewport) | function | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`). |
| [`GraphicsQuality`](#type-GraphicsQuality) | enum | A graphics quality preset. |
| [`UiAction`](#type-UiAction) | callback | A template's action, by its id (a menu item, `buy:<item>`, `share`). |
| [`SettingsChanged`](#type-SettingsChanged) | callback | The settings after a change: persist them (cloud save, local storage through the shell) and apply them. |
| [`MenuItem`](#type-MenuItem) | desc struct | An item of a menu: an id the game acts on and the label a player reads. |
| [`MenuResult`](#type-MenuResult) | struct | What a menu-like template answered this tick. |
| [`MainMenuDesc`](#type-MainMenuDesc) | desc struct | A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone. |
| [`PauseMenuDesc`](#type-PauseMenuDesc) | desc struct | A pause screen: a dialog over the frozen game with Resume first. Back resumes. |
| [`KeyBinding`](#type-KeyBinding) | desc struct | A control the settings screen lists: an action and the key bound to it. |
| [`LanguageOption`](#type-LanguageOption) | desc struct | A language the settings screen offers, in its own name. |
| [`GameSettings`](#type-GameSettings) | desc struct | The settings most games share. The game keeps them (a resource, a save) and passes them in every tick. |
| [`SettingsDesc`](#type-SettingsDesc) | desc struct | A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks. |
| [`SettingsResult`](#type-SettingsResult) | struct | The settings screen's answer this tick. |
| [`Ability`](#type-Ability) | desc struct | An ability on a HUD: a cooldown ring with its key. |
| [`PlatformerHudDesc`](#type-PlatformerHudDesc) | desc struct | A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button. |
| [`HudResult`](#type-HudResult) | struct | What a HUD answered this tick. |
| [`ShooterHudDesc`](#type-ShooterHudDesc) | desc struct | A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map. |
| [`PuzzleHudDesc`](#type-PuzzleHudDesc) | desc struct | A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer. |
| [`Currency`](#type-Currency) | desc struct | A currency an idle game counts. |
| [`Generator`](#type-Generator) | desc struct | A generator an idle game lists: level, cost, progress. |
| [`IdleHudDesc`](#type-IdleHudDesc) | desc struct | An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list. |
| [`RacingHudDesc`](#type-RacingHudDesc) | desc struct | A racing HUD: position and lap top left, the times top right, the speedometer and gear bottom right, the boost meter, an optional mini-map. |
| [`DialogueLine`](#type-DialogueLine) | desc struct | One line of a cutscene. |
| [`CutsceneDesc`](#type-CutsceneDesc) | desc struct | A dialogue scene: letterbox bars, the dialogue box at the bottom, Skip at the top right. |
| [`CutsceneResult`](#type-CutsceneResult) | struct | A cutscene's answer this tick. |
| [`ShopItem`](#type-ShopItem) | desc struct | An item a shop sells. |
| [`ShopDesc`](#type-ShopDesc) | desc struct | A shop: the balance, category tabs, a grid of items with prices, a detail panel with Buy (disabled when the balance is short). |
| [`ShopResult`](#type-ShopResult) | struct | A shop's answer this tick. |
| [`InventoryItem`](#type-InventoryItem) | desc struct | What an inventory slot holds. |
| [`InventoryDesc`](#type-InventoryDesc) | desc struct | An inventory: a grid of slots, items dragged between them, a detail panel with Use. |
| [`SlotMove`](#type-SlotMove) | struct | An item dragged from one slot to another. |
| [`InventoryResult`](#type-InventoryResult) | struct | An inventory's answer this tick. |
| [`LevelInfo`](#type-LevelInfo) | desc struct | A level in a level select. |
| [`LevelSelectDesc`](#type-LevelSelectDesc) | desc struct | A level select: a grid of level cards with stars and locks, the total stars. |
| [`LevelSelectResult`](#type-LevelSelectResult) | struct | A level select's answer this tick. |
| [`StatRow`](#type-StatRow) | desc struct | A labelled value on a results screen. |
| [`ResultsDesc`](#type-ResultsDesc) | desc struct | A results screen: the title, stars that land one by one, the score counting up, the best (a badge for a new best), stats, actions and a share hook. |
| [`LeaderboardEntry`](#type-LeaderboardEntry) | desc struct | A row of a leaderboard. |
| [`LeaderboardDesc`](#type-LeaderboardDesc) | desc struct | A leaderboard: scope tabs, a virtualised list of ranks (medal colours for 1 to 3, the player highlighted), and honest loading, error and empty states. |
| [`LeaderboardResult`](#type-LeaderboardResult) | struct | A leaderboard's answer this tick. |
| [`TutorialStep`](#type-TutorialStep) | desc struct | A step of a tutorial: the element to point at, a title and a line. |
| [`TutorialDesc`](#type-TutorialDesc) | desc struct | An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count. |
| [`TutorialResult`](#type-TutorialResult) | struct | A tutorial's answer this tick. |
| [`LoadingDesc`](#type-LoadingDesc) | desc struct | A loading screen: the title, a real progress bar (or a spinner while unknown) and tips that change by the tick. |
| [`ConfirmDesc`](#type-ConfirmDesc) | desc struct | A confirm dialog: a title, a consequence sentence, Cancel and the action. |
| [`ConfirmResult`](#type-ConfirmResult) | struct | A confirm dialog's answer this tick. |
| [`AlertDesc`](#type-AlertDesc) | desc struct | An alert: a title, a message and one button. |
| [`og.ui.mainMenu`](#fn-mainMenu) | function | The title screen template. Returns the item chosen this tick. |
| [`og.ui.pauseMenu`](#fn-pauseMenu) | function | The pause screen template: a dialog with Resume first; Back answers `resume`. |
| [`og.ui.settingsScreen`](#fn-settingsScreen) | function | The settings template: Audio (volumes, mute), Graphics (quality, frame rate, reduced motion, contrast), Controls (sensitivity, invert, vibration, bindings) and Language. Pass the settings in every tick and keep what it answers. |
| [`og.ui.platformerHud`](#fn-platformerHud) | function | The platformer HUD template. |
| [`og.ui.shooterHud`](#fn-shooterHud) | function | The shooter HUD template. |
| [`og.ui.puzzleHud`](#fn-puzzleHud) | function | The puzzle HUD template. |
| [`og.ui.idleHud`](#fn-idleHud) | function | The idle and incremental HUD template: answers `tap` and `buy:<id>` actions. |
| [`og.ui.racingHud`](#fn-racingHud) | function | The racing HUD template. |
| [`og.ui.cutscene`](#fn-cutscene) | function | The dialogue and cutscene template: letterbox, the dialogue box, Skip. |
| [`og.ui.shopScreen`](#fn-shopScreen) | function | The shop template. |
| [`og.ui.inventoryScreen`](#fn-inventoryScreen) | function | The inventory template: a grid of slots with drag and drop. |
| [`og.ui.levelSelect`](#fn-levelSelect) | function | The level select template. |
| [`og.ui.resultsScreen`](#fn-resultsScreen) | function | The results template: stars, the score counting up, the best, stats, actions and Share. |
| [`og.ui.leaderboardScreen`](#fn-leaderboardScreen) | function | The leaderboard template. |
| [`og.ui.tutorialOverlay`](#fn-tutorialOverlay) | function | The onboarding template: a spotlight and a callout per step. |
| [`og.ui.loadingScreen`](#fn-loadingScreen) | function | The loading template. |
| [`og.ui.confirmDialog`](#fn-confirmDialog) | function | The confirm dialog template. |
| [`og.ui.alertDialog`](#fn-alertDialog) | function | The alert template. Returns true the tick it is dismissed. |

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

<sub>`api.ogt` line 7575</sub>

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

<sub>`api.ogt` line 7582</sub>

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

<sub>`api.ogt` line 7588</sub>

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

<sub>`api.ogt` line 7594</sub>

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

<sub>`api.ogt` line 7601</sub>

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

<sub>`api.ogt` line 7608</sub>

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

<sub>`api.ogt` line 7615</sub>

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

<sub>`api.ogt` line 7620</sub>

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

<sub>`api.ogt` line 7629</sub>

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

<sub>`api.ogt` line 7638</sub>

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

<sub>`api.ogt` line 7647</sub>

<a id="fn-richText"></a>

### `og.ui.richText`

A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the skin), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning.

```ts
og.ui.richText(id: string, markup: string, desc?: RichTextDesc): RichTextResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `markup` | `string` | no | The markup. |
| `desc` | `RichTextDesc` | yes | Defaults for its spans: type step, ink, alignment, wrapping, width and font. |

**Returns** `RichTextResult`.

Binding symbol `og_ui_rich_text` · publish builds: kept

```ogt
fn richText {
  doc "A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the skin), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning."
  param id String "Its id."
  param markup String "The markup."
  param desc RichTextDesc "Defaults for its spans: type step, ink, alignment, wrapping, width and font."
  returns RichTextResult
}
```

<sub>`api.ogt` line 7674</sub>

<a id="fn-label"></a>

### `og.ui.label`

MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text.

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
  doc "MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text."
  param id String "Its id."
  param text String "The text."
  param desc LabelDesc "Type step, ink, alignment, wrapping, width and font."
  returns Void
}
```

<sub>`api.ogt` line 7682</sub>

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

<sub>`api.ogt` line 7690</sub>

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

<sub>`api.ogt` line 7698</sub>

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

<sub>`api.ogt` line 7705</sub>

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

<sub>`api.ogt` line 7713</sub>

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

<sub>`api.ogt` line 7721</sub>

<a id="fn-textField"></a>

### `og.ui.textField`

One line of text: a press focuses it with the caret at the press and a drag selects; typed text (og.input's TextInput events, else the keys) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects; Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

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
  doc "One line of text: a press focuses it with the caret at the press and a drag selects; typed text (og.input's TextInput events, else the keys) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects; Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc FieldDesc "Placeholder, width and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 7729</sub>

<a id="fn-textArea"></a>

### `og.ui.textArea`

Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, the text wrapped between words at the area's width and scrolled to keep the caret's line in view. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

```ts
og.ui.textArea(id: string, value: string, desc?: TextAreaDesc): FieldResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `string` | no | The text from game state. |
| `desc` | `TextAreaDesc` | yes | Placeholder, width, rows and length limit. |

**Returns** `FieldResult`.

Binding symbol `og_ui_text_area` · publish builds: kept

```ogt
fn textArea {
  doc "Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, the text wrapped between words at the area's width and scrolled to keep the caret's line in view. Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc TextAreaDesc "Placeholder, width, rows and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 7737</sub>

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

<sub>`api.ogt` line 7745</sub>

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

<sub>`api.ogt` line 7751</sub>

<a id="fn-breakpoint"></a>

### `og.ui.breakpoint`

The size class of the play surface of the frame being built (desktop outside a frame).

```ts
og.ui.breakpoint(): Breakpoint
```

**Returns** `Breakpoint`.

Binding symbol `og_ui_breakpoint` · publish builds: kept

```ogt
fn breakpoint {
  doc "The size class of the play surface of the frame being built (desktop outside a frame)."
  returns Breakpoint
}
```

<sub>`api.ogt` line 8226</sub>

<a id="fn-frameViewport"></a>

### `og.ui.frameViewport`

The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI.

```ts
og.ui.frameViewport(): Viewport
```

**Returns** `Viewport`.

Binding symbol `og_ui_frame_viewport` · publish builds: kept

```ogt
fn frameViewport {
  doc "The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI."
  returns Viewport
}
```

<sub>`api.ogt` line 8231</sub>

<a id="fn-hasIcon"></a>

### `og.ui.hasIcon`

Whether the icon atlas has an icon of this name (templates draw a fallback otherwise).

```ts
og.ui.hasIcon(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The icon's name. |

**Returns** `boolean`.

Binding symbol `og_ui_has_icon` · publish builds: kept

```ogt
fn hasIcon {
  doc "Whether the icon atlas has an icon of this name (templates draw a fallback otherwise)."
  param name String "The icon's name."
  returns bool
}
```

<sub>`api.ogt` line 8236</sub>

<a id="fn-seconds"></a>

### `og.ui.seconds`

The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock).

```ts
og.ui.seconds(): number
```

**Returns** `number`.

Binding symbol `og_ui_seconds` · publish builds: kept

```ogt
fn seconds {
  doc "The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock)."
  returns f64
}
```

<sub>`api.ogt` line 8242</sub>

<a id="fn-frameTheme"></a>

### `og.ui.frameTheme`

The design tokens of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates).

```ts
og.ui.frameTheme(): UiTheme
```

**Returns** `UiTheme`.

Binding symbol `og_ui_frame_theme` · publish builds: kept

```ogt
fn frameTheme {
  doc "The design tokens of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates)."
  returns UiTheme
}
```

<sub>`api.ogt` line 8247</sub>

<a id="fn-openAnchor"></a>

### `og.ui.openAnchor`

Open an anchored group, closed by close(): og.ui.anchor without the callback.

```ts
og.ui.openAnchor(id: string, desc?: AnchorDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `AnchorDesc` | yes | Where it sits and how it lays out. |

**Returns** `void`.

Binding symbol `og_ui_open_anchor` · publish builds: kept

```ogt
fn openAnchor {
  doc "Open an anchored group, closed by close(): og.ui.anchor without the callback."
  param id String "Its id; the probe path is the id alone."
  param desc AnchorDesc "Where it sits and how it lays out."
  returns Void
}
```

<sub>`api.ogt` line 8252</sub>

<a id="fn-anchor"></a>

### `og.ui.anchor`

An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls.

```ts
og.ui.anchor(id: string, desc: AnchorDesc, build: UiBuild): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AnchorDesc` | no | Where it sits and how it lays out. |
| `build` | `UiBuild` | no | Makes its children. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn anchor {
  doc "An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls."
  host false
  param id String "Its id."
  param desc AnchorDesc "Where it sits and how it lays out."
  param build UiBuild "Makes its children."
  returns Void
}
```

<sub>`api.ogt` line 8259</sub>

<a id="fn-openTransition"></a>

### `og.ui.openTransition`

Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves.

```ts
og.ui.openTransition(id: string, show: boolean, desc?: TransitionDesc): Presence
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `show` | `boolean` | no | Whether it should be shown. |
| `desc` | `TransitionDesc` | yes | How it moves. |

**Returns** `Presence`.

Binding symbol `og_ui_open_transition` · publish builds: kept

```ogt
fn openTransition {
  doc "Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves."
  param id String "Its id."
  param show bool "Whether it should be shown."
  param desc TransitionDesc "How it moves."
  returns Presence
}
```

<sub>`api.ogt` line 8268</sub>

<a id="fn-transition"></a>

### `og.ui.transition`

Content that enters and leaves: fade, slide or scale by the tick from the motion tokens. `build` runs while it is visible.

```ts
og.ui.transition(id: string, show: boolean, desc: TransitionDesc, build: UiBuild): Presence
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `show` | `boolean` | no | Whether it should be shown. |
| `desc` | `TransitionDesc` | no | How it moves. |
| `build` | `UiBuild` | no | Makes its content. |

**Returns** `Presence`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn transition {
  doc "Content that enters and leaves: fade, slide or scale by the tick from the motion tokens. `build` runs while it is visible."
  host false
  param id String "Its id."
  param show bool "Whether it should be shown."
  param desc TransitionDesc "How it moves."
  param build UiBuild "Makes its content."
  returns Presence
}
```

<sub>`api.ogt` line 8276</sub>

<a id="fn-openModal"></a>

### `og.ui.openModal`

Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`.

```ts
og.ui.openModal(id: string, open: boolean, desc?: ModalDesc): ModalResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it should be open. |
| `desc` | `ModalDesc` | yes | Title, width and dismissal rules. |

**Returns** `ModalResult`.

Binding symbol `og_ui_open_modal` · publish builds: kept

```ogt
fn openModal {
  doc "Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`."
  param id String "Its id."
  param open bool "Whether it should be open."
  param desc ModalDesc "Title, width and dismissal rules."
  returns ModalResult
}
```

<sub>`api.ogt` line 8286</sub>

<a id="fn-closeModal"></a>

### `og.ui.closeModal`

Close a dialog opened with openModal (its panel and backdrop).

```ts
og.ui.closeModal(): void
```

**Returns** `void`.

Binding symbol `og_ui_close_modal` · publish builds: kept

```ogt
fn closeModal {
  doc "Close a dialog opened with openModal (its panel and backdrop)."
  returns Void
}
```

<sub>`api.ogt` line 8294</sub>

<a id="fn-modal"></a>

### `og.ui.modal`

A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`.

```ts
og.ui.modal(id: string, open: boolean, desc: ModalDesc, build: UiBuild): ModalResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `open` | `boolean` | no | Whether it should be open. |
| `desc` | `ModalDesc` | no | Title, width and dismissal rules. |
| `build` | `UiBuild` | no | Makes its content. |

**Returns** `ModalResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn modal {
  doc "A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`."
  host false
  param id String "Its id."
  param open bool "Whether it should be open."
  param desc ModalDesc "Title, width and dismissal rules."
  param build UiBuild "Makes its content."
  returns ModalResult
}
```

<sub>`api.ogt` line 8299</sub>

<a id="fn-spotlight"></a>

### `og.ui.spotlight`

A spotlight over everything (a tutorial step): the scrim on four sides of the hole, an amber edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`.

```ts
og.ui.spotlight(id: string, hole: UiRect | null, desc?: SpotlightDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `hole` | `UiRect \| null` | no | The rectangle to leave lit, as og.ui.bounds answers; null dims everything. |
| `desc` | `SpotlightDesc` | yes | Room around the hole. |

**Returns** `Response`.

Binding symbol `og_ui_spotlight` · publish builds: kept

```ogt
fn spotlight {
  doc "A spotlight over everything (a tutorial step): the scrim on four sides of the hole, an amber edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`."
  param id String "Its id."
  param hole Option.UiRect "The rectangle to leave lit, as og.ui.bounds answers; null dims everything."
  param desc SpotlightDesc "Room around the hole."
  returns Response
}
```

<sub>`api.ogt` line 8309</sub>

<a id="fn-bounds"></a>

### `og.ui.bounds`

Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it.

```ts
og.ui.bounds(path: string): UiRect | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The id or path. |

**Returns** `UiRect | null`.

Binding symbol `og_ui_bounds` · publish builds: kept

```ogt
fn bounds {
  doc "Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it."
  param path String "The id or path."
  returns Option.UiRect
}
```

<sub>`api.ogt` line 8317</sub>

<a id="fn-openList"></a>

### `og.ui.openList`

Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList().

```ts
og.ui.openList(id: string, desc?: ListDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ListDesc` | yes | Row count and size. |

**Returns** `ListRange`.

Binding symbol `og_ui_open_list` · publish builds: kept

```ogt
fn openList {
  doc "Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList()."
  param id String "Its id."
  param desc ListDesc "Row count and size."
  returns ListRange
}
```

<sub>`api.ogt` line 8323</sub>

<a id="fn-openGrid"></a>

### `og.ui.openGrid`

Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList().

```ts
og.ui.openGrid(id: string, desc?: GridDesc): ListRange
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `GridDesc` | yes | Cell count, size and columns. |

**Returns** `ListRange`.

Binding symbol `og_ui_open_grid` · publish builds: kept

```ogt
fn openGrid {
  doc "Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList()."
  param id String "Its id."
  param desc GridDesc "Cell count, size and columns."
  returns ListRange
}
```

<sub>`api.ogt` line 8330</sub>

<a id="fn-listItem"></a>

### `og.ui.listItem`

Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`.

```ts
og.ui.listItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (fill-strong). |

**Returns** `Response`.

Binding symbol `og_ui_list_item` · publish builds: kept

```ogt
fn listItem {
  doc "Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`."
  param index u32 "Its index."
  param selected bool "Drawn selected (fill-strong)."
  returns Response
}
```

<sub>`api.ogt` line 8337</sub>

<a id="fn-gridItem"></a>

### `og.ui.gridItem`

Open cell `index` of the open grid (`Cell#index`): a slot on fill, focusable; its content follows until close(). Drops answer `dropped` as listItem's.

```ts
og.ui.gridItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (fill-strong with an amber bar). |

**Returns** `Response`.

Binding symbol `og_ui_grid_item` · publish builds: kept

```ogt
fn gridItem {
  doc "Open cell `index` of the open grid (`Cell#index`): a slot on fill, focusable; its content follows until close(). Drops answer `dropped` as listItem's."
  param index u32 "Its index."
  param selected bool "Drawn selected (fill-strong with an amber bar)."
  returns Response
}
```

<sub>`api.ogt` line 8344</sub>

<a id="fn-closeList"></a>

### `og.ui.closeList`

Close a list or grid opened with openList or openGrid.

```ts
og.ui.closeList(): Response
```

**Returns** `Response`.

Binding symbol `og_ui_close_list` · publish builds: kept

```ogt
fn closeList {
  doc "Close a list or grid opened with openList or openGrid."
  returns Response
}
```

<sub>`api.ogt` line 8351</sub>

<a id="fn-dragging"></a>

### `og.ui.dragging`

The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer.

```ts
og.ui.dragging(id: string): number | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The list's or grid's id. |

**Returns** `number | null`.

Binding symbol `og_ui_dragging` · publish builds: kept

```ogt
fn dragging {
  doc "The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer."
  param id String "The list's or grid's id."
  returns Option.u32
}
```

<sub>`api.ogt` line 8356</sub>

<a id="fn-list"></a>

### `og.ui.list`

A virtualised list built by `build(index)` for the rows in view.

```ts
og.ui.list(id: string, desc: ListDesc, build: UiItemBuild): ListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ListDesc` | no | Row count, size and the selected row. |
| `build` | `UiItemBuild` | no | Makes one row's content. |

**Returns** `ListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn list {
  doc "A virtualised list built by `build(index)` for the rows in view."
  host false
  param id String "Its id."
  param desc ListDesc "Row count, size and the selected row."
  param build UiItemBuild "Makes one row's content."
  returns ListResult
}
```

<sub>`api.ogt` line 8362</sub>

<a id="fn-grid"></a>

### `og.ui.grid`

A virtualised grid built by `build(index)` for the cells in view.

```ts
og.ui.grid(id: string, desc: GridDesc, build: UiItemBuild): ListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `GridDesc` | no | Cell count, size, columns and the selected cell. |
| `build` | `UiItemBuild` | no | Makes one cell's content. |

**Returns** `ListResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn grid {
  doc "A virtualised grid built by `build(index)` for the cells in view."
  host false
  param id String "Its id."
  param desc GridDesc "Cell count, size, columns and the selected cell."
  param build UiItemBuild "Makes one cell's content."
  returns ListResult
}
```

<sub>`api.ogt` line 8371</sub>

<a id="fn-tabs"></a>

### `og.ui.tabs`

A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels.

```ts
og.ui.tabs(id: string, items: readonly string[], selected: number, desc?: TabsDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `items` | `readonly string[]` | no | The tab labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `TabsDesc` | yes | Fill and width. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_tabs` · publish builds: kept

```ogt
fn tabs {
  doc "A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels."
  param id String "Its id."
  param items List.String "The tab labels."
  param selected u32 "The chosen index from game state."
  param desc TabsDesc "Fill and width."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 8380</sub>

<a id="fn-select"></a>

### `og.ui.select`

A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it.

```ts
og.ui.select(id: string, options: readonly string[], selected: number, desc?: SelectDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `options` | `readonly string[]` | no | The option labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `SelectDesc` | yes | Placeholder, width, disabled. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_select` · publish builds: kept

```ogt
fn select {
  doc "A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it."
  param id String "Its id."
  param options List.String "The option labels."
  param selected u32 "The chosen index from game state."
  param desc SelectDesc "Placeholder, width, disabled."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 8389</sub>

<a id="fn-stepper"></a>

### `og.ui.stepper`

A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused.

```ts
og.ui.stepper(id: string, value: number, desc?: StepperDesc): StepperResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value from game state. |
| `desc` | `StepperDesc` | yes | Range, step and digits. |

**Returns** `StepperResult`.

Binding symbol `og_ui_stepper` · publish builds: kept

```ogt
fn stepper {
  doc "A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused."
  param id String "Its id."
  param value f32 "The value from game state."
  param desc StepperDesc "Range, step and digits."
  returns StepperResult
}
```

<sub>`api.ogt` line 8398</sub>

<a id="fn-radio"></a>

### `og.ui.radio`

A radio group of `Option#i` rows: a ring, filled amber when chosen, and a label.

```ts
og.ui.radio(id: string, options: readonly string[], selected: number, desc?: RadioDesc): ChoiceResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `options` | `readonly string[]` | no | The option labels. |
| `selected` | `number` | no | The chosen index from game state. |
| `desc` | `RadioDesc` | yes | Row or column. |

**Returns** `ChoiceResult`.

Binding symbol `og_ui_radio` · publish builds: kept

```ogt
fn radio {
  doc "A radio group of `Option#i` rows: a ring, filled amber when chosen, and a label."
  param id String "Its id."
  param options List.String "The option labels."
  param selected u32 "The chosen index from game state."
  param desc RadioDesc "Row or column."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 8406</sub>

<a id="fn-checkbox"></a>

### `og.ui.checkbox`

A checkbox with a label; a press flips it.

```ts
og.ui.checkbox(id: string, on: boolean, label: string): ToggleResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `on` | `boolean` | no | The state from game state. |
| `label` | `string` | no | The label. |

**Returns** `ToggleResult`.

Binding symbol `og_ui_checkbox` · publish builds: kept

```ogt
fn checkbox {
  doc "A checkbox with a label; a press flips it."
  param id String "Its id."
  param on bool "The state from game state."
  param label String "The label."
  returns ToggleResult
}
```

<sub>`api.ogt` line 8415</sub>

<a id="fn-checkboxList"></a>

### `og.ui.checkboxList`

Checkboxes in a column (`Option#i`).

```ts
og.ui.checkboxList(id: string, labels: readonly string[], values: readonly boolean[]): CheckListResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `labels` | `readonly string[]` | no | The labels. |
| `values` | `readonly boolean[]` | no | The states from game state. |

**Returns** `CheckListResult`.

Binding symbol `og_ui_checkbox_list` · publish builds: kept

```ogt
fn checkboxList {
  doc "Checkboxes in a column (`Option#i`)."
  param id String "Its id."
  param labels List.String "The labels."
  param values List.bool "The states from game state."
  returns CheckListResult
}
```

<sub>`api.ogt` line 8423</sub>

<a id="fn-chip"></a>

### `og.ui.chip`

A chip: a capsule on fill; selected inverts to fg. For filters and tags.

```ts
og.ui.chip(id: string, text: string, desc?: ChipDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label. |
| `desc` | `ChipDesc` | yes | Selected and disabled. |

**Returns** `Response`.

Binding symbol `og_ui_chip` · publish builds: kept

```ogt
fn chip {
  doc "A chip: a capsule on fill; selected inverts to fg. For filters and tags."
  param id String "Its id."
  param text String "The label."
  param desc ChipDesc "Selected and disabled."
  returns Response
}
```

<sub>`api.ogt` line 8431</sub>

<a id="fn-badge"></a>

### `og.ui.badge`

A badge: a small status capsule in its tone.

```ts
og.ui.badge(id: string, text: string, tone: Tone): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | The label (a count, a word). |
| `tone` | `Tone` | no | Its tone. |

**Returns** `void`.

Binding symbol `og_ui_badge` · publish builds: kept

```ogt
fn badge {
  doc "A badge: a small status capsule in its tone."
  param id String "Its id."
  param text String "The label (a count, a word)."
  param tone Tone "Its tone."
  returns Void
}
```

<sub>`api.ogt` line 8439</sub>

<a id="fn-divider"></a>

### `og.ui.divider`

A hairline in the border colour across its parent.

```ts
og.ui.divider(id: string, vertical: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `vertical` | `boolean` | no | Stand it up (in a row). |

**Returns** `void`.

Binding symbol `og_ui_divider` · publish builds: kept

```ogt
fn divider {
  doc "A hairline in the border colour across its parent."
  param id String "Its id."
  param vertical bool "Stand it up (in a row)."
  returns Void
}
```

<sub>`api.ogt` line 8447</sub>

<a id="fn-spacer"></a>

### `og.ui.spacer`

Empty space.

```ts
og.ui.spacer(id: string, size: Size | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `size` | `Size \| null` | no | Its size along both axes; null grows into the free space. |

**Returns** `void`.

Binding symbol `og_ui_spacer` · publish builds: kept

```ogt
fn spacer {
  doc "Empty space."
  param id String "Its id."
  param size Option.Size "Its size along both axes; null grows into the free space."
  returns Void
}
```

<sub>`api.ogt` line 8454</sub>

<a id="fn-keyValue"></a>

### `og.ui.keyValue`

A key (fg-muted) and its value (fg, at the end) on one row: stats, settings summaries, receipts.

```ts
og.ui.keyValue(id: string, key: string, value: string, mono: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `key` | `string` | no | The key. |
| `value` | `string` | no | The value. |
| `mono` | `boolean` | no | The value in the monospaced font (prices, ids). |

**Returns** `void`.

Binding symbol `og_ui_key_value` · publish builds: kept

```ogt
fn keyValue {
  doc "A key (fg-muted) and its value (fg, at the end) on one row: stats, settings summaries, receipts."
  param id String "Its id."
  param key String "The key."
  param value String "The value."
  param mono bool "The value in the monospaced font (prices, ids)."
  returns Void
}
```

<sub>`api.ogt` line 8461</sub>

<a id="fn-skeleton"></a>

### `og.ui.skeleton`

A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion).

```ts
og.ui.skeleton(id: string, desc?: SkeletonDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SkeletonDesc` | yes | Width, line height and lines. |

**Returns** `void`.

Binding symbol `og_ui_skeleton` · publish builds: kept

```ogt
fn skeleton {
  doc "A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion)."
  param id String "Its id."
  param desc SkeletonDesc "Width, line height and lines."
  returns Void
}
```

<sub>`api.ogt` line 8470</sub>

<a id="fn-spinner"></a>

### `og.ui.spinner`

A loading spinner: eight dots around a circle, the bright one turning by the tick.

```ts
og.ui.spinner(id: string, size: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `size` | `number` | no | Its diameter in CSS px; 0 is the small control height. |

**Returns** `void`.

Binding symbol `og_ui_spinner` · publish builds: kept

```ogt
fn spinner {
  doc "A loading spinner: eight dots around a circle, the bright one turning by the tick."
  param id String "Its id."
  param size f32 "Its diameter in CSS px; 0 is the small control height."
  returns Void
}
```

<sub>`api.ogt` line 8477</sub>

<a id="fn-icon"></a>

### `og.ui.icon`

An icon of the atlas (UiDesc.icons) by name; an unknown name draws an empty slot and warns. The probe reads the name.

```ts
og.ui.icon(id: string, name: string, desc?: IconDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `name` | `string` | no | The icon's name. |
| `desc` | `IconDesc` | yes | Size and ink. |

**Returns** `void`.

Binding symbol `og_ui_icon` · publish builds: kept

```ogt
fn icon {
  doc "An icon of the atlas (UiDesc.icons) by name; an unknown name draws an empty slot and warns. The probe reads the name."
  param id String "Its id."
  param name String "The icon's name."
  param desc IconDesc "Size and ink."
  returns Void
}
```

<sub>`api.ogt` line 8484</sub>

<a id="fn-meter"></a>

### `og.ui.meter`

A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick.

```ts
og.ui.meter(id: string, value: number, max: number, desc?: MeterDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The current value. |
| `max` | `number` | no | The full value. |
| `desc` | `MeterDesc` | yes | Kind, label, size and notches. |

**Returns** `void`.

Binding symbol `og_ui_meter` · publish builds: kept

```ogt
fn meter {
  doc "A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick."
  param id String "Its id."
  param value f32 "The current value."
  param max f32 "The full value."
  param desc MeterDesc "Kind, label, size and notches."
  returns Void
}
```

<sub>`api.ogt` line 8492</sub>

<a id="fn-cooldown"></a>

### `og.ui.cooldown`

A cooldown: a disc whose amber ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not).

```ts
og.ui.cooldown(id: string, remaining: number, desc?: CooldownDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `remaining` | `number` | no | How much of the cooldown is left, 0 to 1: 1 just used, 0 ready. |
| `desc` | `CooldownDesc` | yes | Size, label and icon. |

**Returns** `Response`.

Binding symbol `og_ui_cooldown` · publish builds: kept

```ogt
fn cooldown {
  doc "A cooldown: a disc whose amber ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not)."
  param id String "Its id."
  param remaining f32 "How much of the cooldown is left, 0 to 1: 1 just used, 0 ready."
  param desc CooldownDesc "Size, label and icon."
  returns Response
}
```

<sub>`api.ogt` line 8501</sub>

<a id="fn-minimap"></a>

### `og.ui.minimap`

A mini-map frame: the map in a border-strong frame, markers as dots (the player amber and ringed), an N mark and a zone label.

```ts
og.ui.minimap(id: string, desc?: MinimapDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MinimapDesc` | yes | Map, markers and label. |

**Returns** `void`.

Binding symbol `og_ui_minimap` · publish builds: kept

```ogt
fn minimap {
  doc "A mini-map frame: the map in a border-strong frame, markers as dots (the player amber and ringed), an N mark and a zone label."
  param id String "Its id."
  param desc MinimapDesc "Map, markers and label."
  returns Void
}
```

<sub>`api.ogt` line 8509</sub>

<a id="fn-dialogue"></a>

### `og.ui.dialogue`

A dialogue box: a portrait slot, the speaker in amber ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown.

```ts
og.ui.dialogue(id: string, desc?: DialogueDesc): DialogueResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DialogueDesc` | yes | Speaker, text, portrait and speed. |

**Returns** `DialogueResult`.

Binding symbol `og_ui_dialogue` · publish builds: kept

```ogt
fn dialogue {
  doc "A dialogue box: a portrait slot, the speaker in amber ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown."
  param id String "Its id."
  param desc DialogueDesc "Speaker, text, portrait and speed."
  returns DialogueResult
}
```

<sub>`api.ogt` line 8516</sub>

<a id="fn-joystick"></a>

### `og.ui.joystick`

A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time.

```ts
og.ui.joystick(id: string, desc?: JoystickDesc): JoystickResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `JoystickDesc` | yes | Size and dead zone. |

**Returns** `JoystickResult`.

Binding symbol `og_ui_joystick` · publish builds: kept

```ogt
fn joystick {
  doc "A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time."
  param id String "Its id."
  param desc JoystickDesc "Size and dead zone."
  returns JoystickResult
}
```

<sub>`api.ogt` line 8523</sub>

<a id="fn-dpad"></a>

### `og.ui.dpad`

A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px.

```ts
og.ui.dpad(id: string, desc?: DpadDesc): DpadResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `DpadDesc` | yes | Its size. |

**Returns** `DpadResult`.

Binding symbol `og_ui_dpad` · publish builds: kept

```ogt
fn dpad {
  doc "A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px."
  param id String "Its id."
  param desc DpadDesc "Its size."
  returns DpadResult
}
```

<sub>`api.ogt` line 8530</sub>

<a id="fn-actionButton"></a>

### `og.ui.actionButton`

A round touch action button: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px.

```ts
og.ui.actionButton(id: string, text: string, desc?: ActionButtonDesc): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `text` | `string` | no | Its label (and what the probe reads). |
| `desc` | `ActionButtonDesc` | yes | Size, primary and icon. |

**Returns** `Response`.

Binding symbol `og_ui_action_button` · publish builds: kept

```ogt
fn actionButton {
  doc "A round touch action button: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px."
  param id String "Its id."
  param text String "Its label (and what the probe reads)."
  param desc ActionButtonDesc "Size, primary and icon."
  returns Response
}
```

<sub>`api.ogt` line 8537</sub>

<a id="fn-toast"></a>

### `og.ui.toast`

Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it.

```ts
og.ui.toast(world: World, text: string, desc?: ToastDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI shows it. |
| `text` | `string` | no | The message: a few words. |
| `desc` | `ToastDesc` | yes | Tone and time. |

**Returns** `number`: Its id, for dismissToast.

Binding symbol `og_ui_toast` · publish builds: kept

```ogt
fn toast {
  doc "Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it."
  param world World "The world whose UI shows it."
  param text String "The message: a few words."
  param desc ToastDesc "Tone and time."
  returns u32 "Its id, for dismissToast."
}
```

<sub>`api.ogt` line 8545</sub>

<a id="fn-dismissToast"></a>

### `og.ui.dismissToast`

Start a toast's exit.

```ts
og.ui.dismissToast(world: World, toastId: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `toastId` | `number` | no | The id toast() answered. |

**Returns** `void`.

Binding symbol `og_ui_dismiss_toast` · publish builds: kept

```ogt
fn dismissToast {
  doc "Start a toast's exit."
  param world World "The world."
  param toastId u32 "The id toast() answered."
  returns Void
}
```

<sub>`api.ogt` line 8553</sub>

<a id="fn-navigate"></a>

### `og.ui.navigate`

Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command.

```ts
og.ui.navigate(world: World, intent: NavIntent): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `intent` | `NavIntent` | no | The intent. |

**Returns** `void`.

Binding symbol `og_ui_navigate` · publish builds: kept

```ogt
fn navigate {
  doc "Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command."
  param world World "The world."
  param intent NavIntent "The intent."
  returns Void
}
```

<sub>`api.ogt` line 8560</sub>

<a id="fn-back"></a>

### `og.ui.back`

Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it.

```ts
og.ui.back(): boolean
```

**Returns** `boolean`.

Binding symbol `og_ui_back` · publish builds: kept

```ogt
fn back {
  doc "Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it."
  returns bool
}
```

<sub>`api.ogt` line 8567</sub>

<a id="fn-setPlayers"></a>

### `og.ui.setPlayers`

Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it.

```ts
og.ui.setPlayers(count: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `count` | `number` | no | Players, 1 to 4 (clamped). |

**Returns** `void`.

Binding symbol `og_ui_set_players` · publish builds: kept

```ogt
fn setPlayers {
  doc "Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it."
  param count u32 "Players, 1 to 4 (clamped)."
  returns Void
}
```

<sub>`api.ogt` line 8572</sub>

<a id="fn-focus"></a>

### `og.ui.focus`

The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none.

```ts
og.ui.focus(pad: number): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `pad` | `number` | no | The pad, 0 to 3; past 3 answers empty. |

**Returns** `string`.

Binding symbol `og_ui_focus` · publish builds: kept

```ogt
fn focus {
  doc "The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none."
  param pad u32 "The pad, 0 to 3; past 3 answers empty."
  returns String
}
```

<sub>`api.ogt` line 8578</sub>

<a id="fn-a11y"></a>

### `og.ui.a11y`

The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements.

```ts
og.ui.a11y(): A11yTree
```

**Returns** `A11yTree`.

Binding symbol `og_ui_a11y` · publish builds: kept

```ogt
fn a11y {
  doc "The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements."
  returns A11yTree
}
```

<sub>`api.ogt` line 8613</sub>

<a id="fn-setNavigation"></a>

### `og.ui.setNavigation`

Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick.

```ts
og.ui.setNavigation(on: boolean): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `on` | `boolean` | no | On or off. |

**Returns** `void`.

Binding symbol `og_ui_set_navigation` · publish builds: kept

```ogt
fn setNavigation {
  doc "Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick."
  param on bool "On or off."
  returns Void
}
```

<sub>`api.ogt` line 8618</sub>

<a id="fn-memory"></a>

### `og.ui.memory`

A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped.

```ts
og.ui.memory(id: string, initial: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The id, unique among its siblings' memories. |
| `initial` | `number` | no | Its value the first time. |

**Returns** `number`.

Binding symbol `og_ui_memory` · publish builds: kept

```ogt
fn memory {
  doc "A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped."
  param id String "The id, unique among its siblings' memories."
  param initial f64 "Its value the first time."
  returns f64
}
```

<sub>`api.ogt` line 8624</sub>

<a id="fn-remember"></a>

### `og.ui.remember`

Set a number memory (see memory).

```ts
og.ui.remember(id: string, value: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The id. |
| `value` | `number` | no | The value. |

**Returns** `void`.

Binding symbol `og_ui_remember` · publish builds: kept

```ogt
fn remember {
  doc "Set a number memory (see memory)."
  param id String "The id."
  param value f64 "The value."
  returns Void
}
```

<sub>`api.ogt` line 8631</sub>

<a id="fn-tick"></a>

### `og.ui.tick`

Frames the UI has built: its clock, for animations by the tick.

```ts
og.ui.tick(): number
```

**Returns** `number`.

Binding symbol `og_ui_tick` · publish builds: kept

```ogt
fn tick {
  doc "Frames the UI has built: its clock, for animations by the tick."
  returns u32
}
```

<sub>`api.ogt` line 8638</sub>

<a id="fn-counter"></a>

### `og.ui.counter`

A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a skin with motion.pop bumps it when it lands. Scores, currency, rewards.

```ts
og.ui.counter(id: string, value: number, desc?: CounterDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `value` | `number` | no | The value to count to. |
| `desc` | `CounterDesc` | yes | Look, duration, start and format. |

**Returns** `number`: The number shown this tick.

Binding symbol `og_ui_counter` · publish builds: kept

```ogt
fn counter {
  doc "A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a skin with motion.pop bumps it when it lands. Scores, currency, rewards."
  param id String "Its id."
  param value f64 "The value to count to."
  param desc CounterDesc "Look, duration, start and format."
  returns f64 "The number shown this tick."
}
```

<sub>`api.ogt` line 8661</sub>

<a id="fn-burst"></a>

### `og.ui.burst`

Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion.

```ts
og.ui.burst(desc?: BurstDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `BurstDesc` | yes | Count, ink, reach and timing. |

**Returns** `void`.

Binding symbol `og_ui_burst` · publish builds: kept

```ogt
fn burst {
  doc "Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion."
  param desc BurstDesc "Count, ink, reach and timing."
  returns Void
}
```

<sub>`api.ogt` line 8688</sub>

<a id="fn-shake"></a>

### `og.ui.shake`

Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the skin's motion.shake is set.

```ts
og.ui.shake(desc?: ShakeDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ShakeDesc` | yes | Amount, duration and sound. |

**Returns** `void`.

Binding symbol `og_ui_shake` · publish builds: kept

```ogt
fn shake {
  doc "Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the skin's motion.shake is set."
  param desc ShakeDesc "Amount, duration and sound."
  returns Void
}
```

<sub>`api.ogt` line 8708</sub>

<a id="fn-screenTransition"></a>

### `og.ui.screenTransition`

A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade.

```ts
og.ui.screenTransition(id: string, screen: string, desc?: ScreenTransitionDesc): ScreenTransitionResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `screen` | `string` | no | The screen the game wants now. |
| `desc` | `ScreenTransitionDesc` | yes | Kind, duration, colour or texture, side. |

**Returns** `ScreenTransitionResult`.

Binding symbol `og_ui_screen_transition` · publish builds: kept

```ogt
fn screenTransition {
  doc "A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade."
  param id String "Its id."
  param screen String "The screen the game wants now."
  param desc ScreenTransitionDesc "Kind, duration, colour or texture, side."
  returns ScreenTransitionResult
}
```

<sub>`api.ogt` line 8754</sub>

<a id="fn-previewViewport"></a>

### `og.ui.previewViewport`

Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`).

```ts
og.ui.previewViewport(world: World, viewport: Viewport | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `viewport` | `Viewport \| null` | no | The surface to lay out for; null returns to the host's. |

**Returns** `void`.

Binding symbol `og_ui_preview_viewport` · publish builds: kept

```ogt
fn previewViewport {
  doc "Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`)."
  param world World "The world."
  param viewport Option.Viewport "The surface to lay out for; null returns to the host's."
  returns Void
}
```

<sub>`api.ogt` line 8762</sub>

<a id="fn-mainMenu"></a>

### `og.ui.mainMenu`

The title screen template. Returns the item chosen this tick.

```ts
og.ui.mainMenu(id: string, desc?: MainMenuDesc): MenuResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MainMenuDesc` | yes | Title, items and footer. |

**Returns** `MenuResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn mainMenu {
  doc "The title screen template. Returns the item chosen this tick."
  host false
  param id String "Its id."
  param desc MainMenuDesc "Title, items and footer."
  returns MenuResult
}
```

<sub>`api.ogt` line 9290</sub>

<a id="fn-pauseMenu"></a>

### `og.ui.pauseMenu`

The pause screen template: a dialog with Resume first; Back answers `resume`.

```ts
og.ui.pauseMenu(id: string, desc?: PauseMenuDesc): MenuResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PauseMenuDesc` | yes | Open and items. |

**Returns** `MenuResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn pauseMenu {
  doc "The pause screen template: a dialog with Resume first; Back answers `resume`."
  host false
  param id String "Its id."
  param desc PauseMenuDesc "Open and items."
  returns MenuResult
}
```

<sub>`api.ogt` line 9298</sub>

<a id="fn-settingsScreen"></a>

### `og.ui.settingsScreen`

The settings template: Audio (volumes, mute), Graphics (quality, frame rate, reduced motion, contrast), Controls (sensitivity, invert, vibration, bindings) and Language. Pass the settings in every tick and keep what it answers.

```ts
og.ui.settingsScreen(id: string, desc?: SettingsDesc): SettingsResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SettingsDesc` | yes | Open, the settings, languages and hooks. |

**Returns** `SettingsResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn settingsScreen {
  doc "The settings template: Audio (volumes, mute), Graphics (quality, frame rate, reduced motion, contrast), Controls (sensitivity, invert, vibration, bindings) and Language. Pass the settings in every tick and keep what it answers."
  host false
  param id String "Its id."
  param desc SettingsDesc "Open, the settings, languages and hooks."
  returns SettingsResult
}
```

<sub>`api.ogt` line 9306</sub>

<a id="fn-platformerHud"></a>

### `og.ui.platformerHud`

The platformer HUD template.

```ts
og.ui.platformerHud(id: string, desc?: PlatformerHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PlatformerHudDesc` | yes | Lives, coins, score, level and timer. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn platformerHud {
  doc "The platformer HUD template."
  host false
  param id String "Its id."
  param desc PlatformerHudDesc "Lives, coins, score, level and timer."
  returns HudResult
}
```

<sub>`api.ogt` line 9314</sub>

<a id="fn-shooterHud"></a>

### `og.ui.shooterHud`

The shooter HUD template.

```ts
og.ui.shooterHud(id: string, desc?: ShooterHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ShooterHudDesc` | yes | Health, armor, ammo, abilities and map. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn shooterHud {
  doc "The shooter HUD template."
  host false
  param id String "Its id."
  param desc ShooterHudDesc "Health, armor, ammo, abilities and map."
  returns HudResult
}
```

<sub>`api.ogt` line 9322</sub>

<a id="fn-puzzleHud"></a>

### `og.ui.puzzleHud`

The puzzle HUD template.

```ts
og.ui.puzzleHud(id: string, desc?: PuzzleHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `PuzzleHudDesc` | yes | Level, score, target, stars, moves and timer. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn puzzleHud {
  doc "The puzzle HUD template."
  host false
  param id String "Its id."
  param desc PuzzleHudDesc "Level, score, target, stars, moves and timer."
  returns HudResult
}
```

<sub>`api.ogt` line 9330</sub>

<a id="fn-idleHud"></a>

### `og.ui.idleHud`

The idle and incremental HUD template: answers `tap` and `buy:<id>` actions.

```ts
og.ui.idleHud(id: string, desc?: IdleHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `IdleHudDesc` | yes | Currencies and generators. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn idleHud {
  doc "The idle and incremental HUD template: answers `tap` and `buy:<id>` actions."
  host false
  param id String "Its id."
  param desc IdleHudDesc "Currencies and generators."
  returns HudResult
}
```

<sub>`api.ogt` line 9338</sub>

<a id="fn-racingHud"></a>

### `og.ui.racingHud`

The racing HUD template.

```ts
og.ui.racingHud(id: string, desc?: RacingHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `RacingHudDesc` | yes | Speed, gear, laps, place, times and boost. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn racingHud {
  doc "The racing HUD template."
  host false
  param id String "Its id."
  param desc RacingHudDesc "Speed, gear, laps, place, times and boost."
  returns HudResult
}
```

<sub>`api.ogt` line 9346</sub>

<a id="fn-cutscene"></a>

### `og.ui.cutscene`

The dialogue and cutscene template: letterbox, the dialogue box, Skip.

```ts
og.ui.cutscene(id: string, desc?: CutsceneDesc): CutsceneResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `CutsceneDesc` | yes | Lines and the current index. |

**Returns** `CutsceneResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn cutscene {
  doc "The dialogue and cutscene template: letterbox, the dialogue box, Skip."
  host false
  param id String "Its id."
  param desc CutsceneDesc "Lines and the current index."
  returns CutsceneResult
}
```

<sub>`api.ogt` line 9354</sub>

<a id="fn-shopScreen"></a>

### `og.ui.shopScreen`

The shop template.

```ts
og.ui.shopScreen(id: string, desc?: ShopDesc): ShopResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ShopDesc` | yes | Balance, items and categories. |

**Returns** `ShopResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn shopScreen {
  doc "The shop template."
  host false
  param id String "Its id."
  param desc ShopDesc "Balance, items and categories."
  returns ShopResult
}
```

<sub>`api.ogt` line 9362</sub>

<a id="fn-inventoryScreen"></a>

### `og.ui.inventoryScreen`

The inventory template: a grid of slots with drag and drop.

```ts
og.ui.inventoryScreen(id: string, desc?: InventoryDesc): InventoryResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `InventoryDesc` | yes | Slots and items. |

**Returns** `InventoryResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn inventoryScreen {
  doc "The inventory template: a grid of slots with drag and drop."
  host false
  param id String "Its id."
  param desc InventoryDesc "Slots and items."
  returns InventoryResult
}
```

<sub>`api.ogt` line 9370</sub>

<a id="fn-levelSelect"></a>

### `og.ui.levelSelect`

The level select template.

```ts
og.ui.levelSelect(id: string, desc?: LevelSelectDesc): LevelSelectResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `LevelSelectDesc` | yes | The levels. |

**Returns** `LevelSelectResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn levelSelect {
  doc "The level select template."
  host false
  param id String "Its id."
  param desc LevelSelectDesc "The levels."
  returns LevelSelectResult
}
```

<sub>`api.ogt` line 9378</sub>

<a id="fn-resultsScreen"></a>

### `og.ui.resultsScreen`

The results template: stars, the score counting up, the best, stats, actions and Share.

```ts
og.ui.resultsScreen(id: string, desc?: ResultsDesc): MenuResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ResultsDesc` | yes | Score, stars, stats and actions. |

**Returns** `MenuResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn resultsScreen {
  doc "The results template: stars, the score counting up, the best, stats, actions and Share."
  host false
  param id String "Its id."
  param desc ResultsDesc "Score, stars, stats and actions."
  returns MenuResult
}
```

<sub>`api.ogt` line 9386</sub>

<a id="fn-leaderboardScreen"></a>

### `og.ui.leaderboardScreen`

The leaderboard template.

```ts
og.ui.leaderboardScreen(id: string, desc?: LeaderboardDesc): LeaderboardResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `LeaderboardDesc` | yes | Scopes, rows and states. |

**Returns** `LeaderboardResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn leaderboardScreen {
  doc "The leaderboard template."
  host false
  param id String "Its id."
  param desc LeaderboardDesc "Scopes, rows and states."
  returns LeaderboardResult
}
```

<sub>`api.ogt` line 9394</sub>

<a id="fn-tutorialOverlay"></a>

### `og.ui.tutorialOverlay`

The onboarding template: a spotlight and a callout per step.

```ts
og.ui.tutorialOverlay(id: string, desc?: TutorialDesc): TutorialResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `TutorialDesc` | yes | Steps and the current index. |

**Returns** `TutorialResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn tutorialOverlay {
  doc "The onboarding template: a spotlight and a callout per step."
  host false
  param id String "Its id."
  param desc TutorialDesc "Steps and the current index."
  returns TutorialResult
}
```

<sub>`api.ogt` line 9402</sub>

<a id="fn-loadingScreen"></a>

### `og.ui.loadingScreen`

The loading template.

```ts
og.ui.loadingScreen(id: string, desc?: LoadingDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `LoadingDesc` | yes | Title, progress and tips. |

**Returns** `void`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn loadingScreen {
  doc "The loading template."
  host false
  param id String "Its id."
  param desc LoadingDesc "Title, progress and tips."
  returns Void
}
```

<sub>`api.ogt` line 9410</sub>

<a id="fn-confirmDialog"></a>

### `og.ui.confirmDialog`

The confirm dialog template.

```ts
og.ui.confirmDialog(id: string, desc?: ConfirmDesc): ConfirmResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ConfirmDesc` | yes | Question, consequence and labels. |

**Returns** `ConfirmResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn confirmDialog {
  doc "The confirm dialog template."
  host false
  param id String "Its id."
  param desc ConfirmDesc "Question, consequence and labels."
  returns ConfirmResult
}
```

<sub>`api.ogt` line 9418</sub>

<a id="fn-alertDialog"></a>

### `og.ui.alertDialog`

The alert template. Returns true the tick it is dismissed.

```ts
og.ui.alertDialog(id: string, desc?: AlertDesc): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `AlertDesc` | yes | Title, message and label. |

**Returns** `boolean`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn alertDialog {
  doc "The alert template. Returns true the tick it is dismissed."
  host false
  param id String "Its id."
  param desc AlertDesc "Title, message and label."
  returns bool
}
```

<sub>`api.ogt` line 9426</sub>

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

<sub>`api.ogt` line 7105</sub>

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

<sub>`api.ogt` line 7111</sub>

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

<sub>`api.ogt` line 7119</sub>

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

<sub>`api.ogt` line 7127</sub>

<a id="type-FontRole"></a>

### `FontRole` (enum)

Which configured font text uses; display and mono fall back to body, rounded to display, then body.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | The body font. |
| `"display"` | The display font. |
| `"mono"` | The monospaced font. |
| `"rounded"` | The rounded font, for levels and scores (the design language's font-rounded). |

```ogt
enum FontRole {
  doc "Which configured font text uses; display and mono fall back to body, rounded to display, then body."
  variant body "The body font."
  variant display "The display font."
  variant mono "The monospaced font."
  variant rounded "The rounded font, for levels and scores (the design language's font-rounded)."
}
```

<sub>`api.ogt` line 7134</sub>

<a id="type-ButtonVariant"></a>

### `ButtonVariant` (enum)

Button looks, as the design language's buttons.

A string: one of the variants below. The zero value is `"secondary"`.

| Variant | Doc |
|---|---|
| `"secondary"` | A neutral fill (the default). |
| `"primary"` | The amber action: one per screen. |
| `"quiet"` | Text until hovered. |
| `"danger"` | A destructive action: danger ink on the danger tint. |

```ogt
enum ButtonVariant {
  doc "Button looks, as the design language's buttons."
  variant secondary "A neutral fill (the default)."
  variant primary "The amber action: one per screen."
  variant quiet "Text until hovered."
  variant danger "A destructive action: danger ink on the danger tint."
}
```

<sub>`api.ogt` line 7142</sub>

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

<sub>`api.ogt` line 7150</sub>

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

<sub>`api.ogt` line 7157</sub>

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

<sub>`api.ogt` line 7163</sub>

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

<sub>`api.ogt` line 7171</sub>

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

<sub>`api.ogt` line 7220</sub>

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

<sub>`api.ogt` line 7245</sub>

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

<sub>`api.ogt` line 7257</sub>

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

<sub>`api.ogt` line 7264</sub>

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

<sub>`api.ogt` line 7271</sub>

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

<sub>`api.ogt` line 7280</sub>

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

<sub>`api.ogt` line 7296</sub>

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
| `scrollX` | `boolean` | `false` | — | — | — | Scroll its content horizontally and clip it (with scroll: both ways). |
| `interactive` | `boolean` | `false` | — | — | — | Report a Response from close(), as a button does. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `frame` | `SkinPart \| null` | `null` | — | — | — | The skin frame its background takes (a HUD capsule, a card); null is a plain fill. Without a skin the fill is drawn either way. |

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
  field scrollX bool "Scroll its content horizontally and clip it (with scroll: both ways)."
  field interactive bool "Report a Response from close(), as a button does."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field frame Option.SkinPart "The skin frame its background takes (a HUD capsule, a card); null is a plain fill. Without a skin the fill is drawn either way."
}
```

<sub>`api.ogt` line 7308</sub>

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
| `heading` | `boolean` | `false` | — | — | — | A heading: a skin with a flourish frames it. |
| `ellipsis` | `boolean` | `false` | — | — | — | Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text. |
| `maxLines` | `number` | `0` | — | — | — | The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out. |

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
  field heading bool "A heading: a skin with a flourish frames it."
  field ellipsis bool "Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text."
  field maxLines u32 "The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out."
}
```

<sub>`api.ogt` line 7331</sub>

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

<sub>`api.ogt` line 7345</sub>

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

<sub>`api.ogt` line 7354</sub>

<a id="type-ScrollAxes"></a>

### `ScrollAxes` (enum)

Which ways a scroll container scrolls.

A string: one of the variants below. The zero value is `"vertical"`.

| Variant | Doc |
|---|---|
| `"vertical"` | Up and down (the default). |
| `"horizontal"` | Left and right. |
| `"both"` | Both ways. |

```ogt
enum ScrollAxes {
  doc "Which ways a scroll container scrolls."
  variant vertical "Up and down (the default)."
  variant horizontal "Left and right."
  variant both "Both ways."
}
```

<sub>`api.ogt` line 7367</sub>

<a id="type-ScrollDesc"></a>

### `ScrollDesc` (desc struct)

A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `axes` | `ScrollAxes` | `"vertical"` | — | — | — | Which ways it scrolls. |

```ogt
struct ScrollDesc {
  doc "A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it."
  desc true
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Insets "Space inside."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field axes ScrollAxes "Which ways it scrolls."
}
```

<sub>`api.ogt` line 7374</sub>

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

<sub>`api.ogt` line 7386</sub>

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

<sub>`api.ogt` line 7396</sub>

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

<sub>`api.ogt` line 7402</sub>

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

<sub>`api.ogt` line 7413</sub>

<a id="type-TextAreaDesc"></a>

### `TextAreaDesc` (desc struct)

A multi-line text area.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown, dimmed, while the area is empty. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `rows` | `number` | `0` | — | — | — | Lines shown at once; 0 is 4. More scroll. |
| `maxChars` | `number` | `0` | — | — | — | The most characters it takes, line breaks included; 0 is no limit. |

```ogt
struct TextAreaDesc {
  doc "A multi-line text area."
  desc true
  field placeholder String "Shown, dimmed, while the area is empty."
  field width Option.Size "Its width; null grows."
  field rows u32 "Lines shown at once; 0 is 4. More scroll."
  field maxChars u32 "The most characters it takes, line breaks included; 0 is no limit."
}
```

<sub>`api.ogt` line 7421</sub>

<a id="type-Response"></a>

### `Response` (struct)

What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hovered` | `boolean` | `false` | — | — | — | The pointer is over it and no other element holds the press. |
| `pressed` | `boolean` | `false` | — | — | — | The press that started on it is still held. |
| `clicked` | `boolean` | `false` | — | — | — | Pressed and released inside it. |
| `changed` | `boolean` | `false` | — | — | — | Its value changed (slider, toggle, text field). |
| `focused` | `boolean` | `false` | — | — | — | It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on. |
| `submitted` | `boolean` | `false` | — | — | — | Enter was pressed in it (text field). |
| `dropped` | `number \| null` | `null` | — | — | — | A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop). |
| `pad` | `number \| null` | `null` | — | — | — | The gamepad (0 to 3) whose A pressed it this tick; null when the pointer or the keyboard did. Pads 1 to 3 press only while og.ui.setPlayers allows them (local multiplayer). |
| `focusedBy` | `number` | `0` | — | — | — | Bit k: pad k's focus is on it (bit 0 is the shared focus of the pointer, the keyboard and pad 0; bits 1 to 3 the other players' pads). |

```ogt
struct Response {
  doc "What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick."
  field hovered bool "The pointer is over it and no other element holds the press."
  field pressed bool "The press that started on it is still held."
  field clicked bool "Pressed and released inside it."
  field changed bool "Its value changed (slider, toggle, text field)."
  field focused bool "It has focus: a text field being edited, or the control keyboard and gamepad navigation landed on."
  field submitted bool "Enter was pressed in it (text field)."
  field dropped Option.u32 "A list row or grid cell a press started on was released over this one: that row's or cell's index (drag and drop)."
  field pad Option.u32 "The gamepad (0 to 3) whose A pressed it this tick; null when the pointer or the keyboard did. Pads 1 to 3 press only while og.ui.setPlayers allows them (local multiplayer)."
  field focusedBy u32 "Bit k: pad k's focus is on it (bit 0 is the shared focus of the pointer, the keyboard and pad 0; bits 1 to 3 the other players' pads)."
}
```

<sub>`api.ogt` line 7430</sub>

<a id="type-SliderResult"></a>

### `SliderResult` (struct)

A slider's response and its value after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it. |
| `value` | `number` | `0` | — | — | — | The value, to keep in game state and pass back next tick. |

```ogt
struct SliderResult {
  doc "A slider's response and its value after this tick's input."
  field response Response "What happened to it."
  field value f32 "The value, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 7443</sub>

<a id="type-ToggleResult"></a>

### `ToggleResult` (struct)

A toggle's response and its state after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it. |
| `on` | `boolean` | `false` | — | — | — | The state, to keep in game state and pass back next tick. |

```ogt
struct ToggleResult {
  doc "A toggle's response and its state after this tick's input."
  field response Response "What happened to it."
  field on bool "The state, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 7449</sub>

<a id="type-FieldResult"></a>

### `FieldResult` (struct)

A text field's response and its text after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it. |
| `value` | `string` | `""` | — | — | — | The text, to keep in game state and pass back next tick. |

```ogt
struct FieldResult {
  doc "A text field's response and its text after this tick's input."
  field response Response "What happened to it."
  field value String "The text, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 7455</sub>

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
| `breakpoint` | `Breakpoint` | `"desktop"` | — | — | — | Its size class: phone portrait or landscape, tablet, desktop. |

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
  field breakpoint Breakpoint "Its size class: phone portrait or landscape, tablet, desktop."
}
```

<sub>`api.ogt` line 7461</sub>

<a id="type-UiDesc"></a>

### `UiDesc` (desc struct)

How the UI looks: the scheme, the density, its fonts, the contrast, the game's accent, motion and the icon atlas.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette. |
| `density` | `DensityMode` | `"auto"` | — | — | — | The control sizes. |
| `font` | `Font \| null` | `null` | — | — | — | The body font, from og.load.font; null measures text at half an em per character and draws none. |
| `displayFont` | `Font \| null` | `null` | — | — | — | The display font; null uses the body font. |
| `monoFont` | `Font \| null` | `null` | — | — | — | The monospaced font; null uses the body font. |
| `roundedFont` | `Font \| null` | `null` | — | — | — | The rounded font (levels and scores); null uses the display font, then the body font. |
| `fallbackFonts` | `readonly Font[]` | `[]` | — | — | — | Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | Standard or high contrast. |
| `accentHue` | `HueName \| null` | `null` | — | — | — | A token hue as the game's accent (its hover, active, ink and focus tones derived as the tokens derive amber's); null keeps amber. |
| `accentColor` | `Color \| null` | `null` | — | — | — | An sRGB colour as the game's accent, when no hue is given. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade; sheens stop (prefers-reduced-motion). |
| `icons` | `IconAtlasDesc \| null` | `null` | — | — | — | The icon atlas og.ui.icon draws from; null takes the skin's. |
| `toastAnchor` | `Anchor \| null` | `null` | — | — | — | Where toasts stack; null is the bottom centre. |
| `skin` | `Skin \| null` | `null` | — | — | — | A skin from og.load.skin: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the token look (fonts, icons and an accent given here still win); null is the platform tokens. |
| `sounds` | `UiSounds \| null` | `null` | — | — | — | Sounds for UI events (they win over the skin's). |

```ogt
struct UiDesc {
  doc "How the UI looks: the scheme, the density, its fonts, the contrast, the game's accent, motion and the icon atlas."
  desc true
  field scheme Scheme "The palette."
  field density DensityMode "The control sizes."
  field font Option.Font "The body font, from og.load.font; null measures text at half an em per character and draws none."
  field displayFont Option.Font "The display font; null uses the body font."
  field monoFont Option.Font "The monospaced font; null uses the body font."
  field roundedFont Option.Font "The rounded font (levels and scores); null uses the display font, then the body font."
  field fallbackFonts List.Font "Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole."
  field contrast Contrast "Standard or high contrast."
  field accentHue Option.HueName "A token hue as the game's accent (its hover, active, ink and focus tones derived as the tokens derive amber's); null keeps amber."
  field accentColor Option.Color "An sRGB colour as the game's accent, when no hue is given."
  field reducedMotion bool "Transitions only fade; sheens stop (prefers-reduced-motion)."
  field icons Option.IconAtlasDesc "The icon atlas og.ui.icon draws from; null takes the skin's."
  field toastAnchor Option.Anchor "Where toasts stack; null is the bottom centre."
  field skin Option.Skin "A skin from og.load.skin: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the token look (fonts, icons and an accent given here still win); null is the platform tokens."
  field sounds Option.UiSounds "Sounds for UI events (they win over the skin's)."
}
```

<sub>`api.ogt` line 7477</sub>

<a id="type-UiSounds"></a>

### `UiSounds` (desc struct)

Sounds the UI plays through og-audio (the ui bus) on its events; null is silent.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `hover` | `Sound \| null` | `null` | — | — | — | The pointer reaches a control, or keyboard or gamepad focus moves to one. |
| `press` | `Sound \| null` | `null` | — | — | — | A control is pressed: by the pointer, Enter, Space or the gamepad's A. |
| `open` | `Sound \| null` | `null` | — | — | — | A dialog opens. |
| `close` | `Sound \| null` | `null` | — | — | — | A dialog closes. |
| `success` | `Sound \| null` | `null` | — | — | — | A success toast. |
| `error` | `Sound \| null` | `null` | — | — | — | A refused press or a danger toast. |

```ogt
struct UiSounds {
  doc "Sounds the UI plays through og-audio (the ui bus) on its events; null is silent."
  desc true
  field hover Option.Sound "The pointer reaches a control, or keyboard or gamepad focus moves to one."
  field press Option.Sound "A control is pressed: by the pointer, Enter, Space or the gamepad's A."
  field open Option.Sound "A dialog opens."
  field close Option.Sound "A dialog closes."
  field success Option.Sound "A success toast."
  field error Option.Sound "A refused press or a danger toast."
}
```

<sub>`api.ogt` line 7497</sub>

<a id="type-SkinPart"></a>

### `SkinPart` (enum)

An element class a skin dresses with a nine-slice frame.

A string: one of the variants below. The zero value is `"panel"`.

| Variant | Doc |
|---|---|
| `"panel"` | Dialogs and screen panels. |
| `"card"` | Cards. |
| `"button"` | Buttons (with hover, pressed and disabled states). |
| `"field"` | Text fields. |
| `"slot"` | Inventory and grid cells. |
| `"tooltip"` | Tooltips. |
| `"capsule"` | HUD capsules and toasts. |
| `"track"` | A progress bar's track. |

```ogt
enum SkinPart {
  doc "An element class a skin dresses with a nine-slice frame."
  variant panel "Dialogs and screen panels."
  variant card "Cards."
  variant button "Buttons (with hover, pressed and disabled states)."
  variant field "Text fields."
  variant slot "Inventory and grid cells."
  variant tooltip "Tooltips."
  variant capsule "HUD capsules and toasts."
  variant track "A progress bar's track."
}
```

<sub>`api.ogt` line 7508</sub>

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

<sub>`api.ogt` line 7520</sub>

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

<sub>`api.ogt` line 7534</sub>

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
| `skin` | `string` | `""` | — | — | — | The configured skin's name; empty for the platform tokens. |

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
  field skin String "The configured skin's name; empty for the platform tokens."
}
```

<sub>`api.ogt` line 7559</sub>

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

<sub>`api.ogt` line 7571</sub>

<a id="type-RichTextDesc"></a>

### `RichTextDesc` (desc struct)

How a rich text paragraph draws: its defaults, which its markup's spans override.

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
struct RichTextDesc {
  doc "How a rich text paragraph draws: its defaults, which its markup's spans override."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is fg."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
}
```

<sub>`api.ogt` line 7656</sub>

<a id="type-RichTextResult"></a>

### `RichTextResult` (struct)

What a rich text's links did this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | Its links together: hovered while one is, clicked when one was. |
| `link` | `string \| null` | `null` | — | — | — | The link clicked this tick: its `[link=…]` id. |
| `hovered` | `string \| null` | `null` | — | — | — | The link under the pointer. |

```ogt
struct RichTextResult {
  doc "What a rich text's links did this tick."
  field response Response "Its links together: hovered while one is, clicked when one was."
  field link Option.String "The link clicked this tick: its `[link=…]` id."
  field hovered Option.String "The link under the pointer."
}
```

<sub>`api.ogt` line 7667</sub>

<a id="type-Anchor"></a>

### `Anchor` (enum)

A point of the safe area an anchored group sits at.

A string: one of the variants below. The zero value is `"topLeft"`.

| Variant | Doc |
|---|---|
| `"topLeft"` | The top left corner. |
| `"top"` | The middle of the top edge. |
| `"topRight"` | The top right corner. |
| `"left"` | The middle of the left edge. |
| `"center"` | The centre. |
| `"right"` | The middle of the right edge. |
| `"bottomLeft"` | The bottom left corner. |
| `"bottom"` | The middle of the bottom edge. |
| `"bottomRight"` | The bottom right corner. |

```ogt
enum Anchor {
  doc "A point of the safe area an anchored group sits at."
  variant topLeft "The top left corner."
  variant top "The middle of the top edge."
  variant topRight "The top right corner."
  variant left "The middle of the left edge."
  variant center "The centre."
  variant right "The middle of the right edge."
  variant bottomLeft "The bottom left corner."
  variant bottom "The middle of the bottom edge."
  variant bottomRight "The bottom right corner."
}
```

<sub>`api.ogt` line 7761</sub>

<a id="type-TransitionKind"></a>

### `TransitionKind` (enum)

How a transition moves its content in and out. Under reduced motion every kind only fades.

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | Opacity only. |
| `"slideUp"` | Rises into place from below. |
| `"slideDown"` | Drops into place from above. |
| `"slideLeft"` | Enters from the right. |
| `"slideRight"` | Enters from the left. |
| `"scale"` | Grows from slightly smaller around its centre. |
| `"pop"` | Springs up from smaller around its centre, overshoots and settles. |

```ogt
enum TransitionKind {
  doc "How a transition moves its content in and out. Under reduced motion every kind only fades."
  variant fade "Opacity only."
  variant slideUp "Rises into place from below."
  variant slideDown "Drops into place from above."
  variant slideLeft "Enters from the right."
  variant slideRight "Enters from the left."
  variant scale "Grows from slightly smaller around its centre."
  variant pop "Springs up from smaller around its centre, overshoots and settles."
}
```

<sub>`api.ogt` line 7774</sub>

<a id="type-Tone"></a>

### `Tone` (enum)

A status tone, as the design language's badges and toasts.

A string: one of the variants below. The zero value is `"neutral"`.

| Variant | Doc |
|---|---|
| `"neutral"` | Neutral: fill and muted ink. |
| `"success"` | Success. |
| `"warn"` | Warning (orange). |
| `"danger"` | Danger. |
| `"info"` | Information. |
| `"accent"` | Amber identity: the soft accent and amber ink. |
| `"inverse"` | Neutral emphasis: fg fill, inverse ink. |

```ogt
enum Tone {
  doc "A status tone, as the design language's badges and toasts."
  variant neutral "Neutral: fill and muted ink."
  variant success "Success."
  variant warn "Warning (orange)."
  variant danger "Danger."
  variant info "Information."
  variant accent "Amber identity: the soft accent and amber ink."
  variant inverse "Neutral emphasis: fg fill, inverse ink."
}
```

<sub>`api.ogt` line 7785</sub>

<a id="type-MeterKind"></a>

### `MeterKind` (enum)

What a meter measures, which sets its colour.

A string: one of the variants below. The zero value is `"health"`.

| Variant | Doc |
|---|---|
| `"health"` | Green, warning orange under 30 %, danger red under 15 %. |
| `"mana"` | The blue hue. |
| `"xp"` | The purple hue. |
| `"accent"` | The accent. |

```ogt
enum MeterKind {
  doc "What a meter measures, which sets its colour."
  variant health "Green, warning orange under 30 %, danger red under 15 %."
  variant mana "The blue hue."
  variant xp "The purple hue."
  variant accent "The accent."
}
```

<sub>`api.ogt` line 7796</sub>

<a id="type-Breakpoint"></a>

### `Breakpoint` (enum)

The play surface's size class.

A string: one of the variants below. The zero value is `"desktop"`.

| Variant | Doc |
|---|---|
| `"desktop"` | 1024 CSS px wide or more, the shorter side 600 or more. |
| `"tablet"` | The shorter side 600 or more, under 1024 wide. |
| `"phoneLandscape"` | The shorter side under 600, wider than tall. |
| `"phonePortrait"` | The shorter side under 600, taller than wide. |

```ogt
enum Breakpoint {
  doc "The play surface's size class."
  variant desktop "1024 CSS px wide or more, the shorter side 600 or more."
  variant tablet "The shorter side 600 or more, under 1024 wide."
  variant phoneLandscape "The shorter side under 600, wider than tall."
  variant phonePortrait "The shorter side under 600, taller than wide."
}
```

<sub>`api.ogt` line 7804</sub>

<a id="type-Contrast"></a>

### `Contrast` (enum)

The theme variant: high contrast remaps muted inks, fills and borders to stronger tokens and outlines every control (no new colour).

A string: one of the variants below. The zero value is `"standard"`.

| Variant | Doc |
|---|---|
| `"standard"` | The token palette as it is. |
| `"high"` | Stronger inks, fills and edges. |

```ogt
enum Contrast {
  doc "The theme variant: high contrast remaps muted inks, fills and borders to stronger tokens and outlines every control (no new colour)."
  variant standard "The token palette as it is."
  variant high "Stronger inks, fills and edges."
}
```

<sub>`api.ogt` line 7812</sub>

<a id="type-HueName"></a>

### `HueName` (enum)

One of the twelve category hues of the tokens, for a game's accent.

A string: one of the variants below. The zero value is `"red"`.

| Variant | Doc |
|---|---|
| `"red"` | hue-red. |
| `"orange"` | hue-orange. |
| `"yellow"` | hue-yellow. |
| `"green"` | hue-green. |
| `"mint"` | hue-mint. |
| `"teal"` | hue-teal. |
| `"cyan"` | hue-cyan. |
| `"blue"` | hue-blue. |
| `"indigo"` | hue-indigo. |
| `"purple"` | hue-purple. |
| `"pink"` | hue-pink. |
| `"brown"` | hue-brown. |

```ogt
enum HueName {
  doc "One of the twelve category hues of the tokens, for a game's accent."
  variant red "hue-red."
  variant orange "hue-orange."
  variant yellow "hue-yellow."
  variant green "hue-green."
  variant mint "hue-mint."
  variant teal "hue-teal."
  variant cyan "hue-cyan."
  variant blue "hue-blue."
  variant indigo "hue-indigo."
  variant purple "hue-purple."
  variant pink "hue-pink."
  variant brown "hue-brown."
}
```

<sub>`api.ogt` line 7818</sub>

<a id="type-NavIntent"></a>

### `NavIntent` (enum)

A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give.

A string: one of the variants below. The zero value is `"up"`.

| Variant | Doc |
|---|---|
| `"up"` | Up. |
| `"down"` | Down. |
| `"left"` | Left. |
| `"right"` | Right. |
| `"next"` | The next control in reading order (Tab). |
| `"prev"` | The previous one (Shift+Tab). |
| `"activate"` | Press the focused control (Enter, Space, A). |
| `"back"` | Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B). |

```ogt
enum NavIntent {
  doc "A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give."
  variant up "Up."
  variant down "Down."
  variant left "Left."
  variant right "Right."
  variant next "The next control in reading order (Tab)."
  variant prev "The previous one (Shift+Tab)."
  variant activate "Press the focused control (Enter, Space, A)."
  variant back "Leave: the topmost dialog or menu closes, else og.ui.back() answers true (Escape, B)."
}
```

<sub>`api.ogt` line 7834</sub>

<a id="type-UiRect"></a>

### `UiRect` (struct)

A rectangle of the play surface.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its left edge. |
| `y` | `number` | `0` | — | `cssPx` | — | Its top edge. |
| `width` | `number` | `0` | — | `cssPx` | — | Its width. |
| `height` | `number` | `0` | — | `cssPx` | — | Its height. |

```ogt
struct UiRect {
  doc "A rectangle of the play surface."
  field x f32 "Its left edge." {
    unit cssPx
  }
  field y f32 "Its top edge." {
    unit cssPx
  }
  field width f32 "Its width." {
    unit cssPx
  }
  field height f32 "Its height." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 7846</sub>

<a id="type-IconAtlasDesc"></a>

### `IconAtlasDesc` (desc struct)

The icon atlas: a texture of equal cells in a grid, named in reading order.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The atlas texture, from og.load.texture. |
| `columns` | `number` | `1` | ≥ 1 | — | — | Cells across. |
| `rows` | `number` | `1` | ≥ 1 | — | — | Cells down. |
| `names` | `readonly string[]` | `[]` | — | — | — | Each cell's name, in reading order: og.ui.icon draws a cell by its name. |

```ogt
struct IconAtlasDesc {
  doc "The icon atlas: a texture of equal cells in a grid, named in reading order."
  desc true
  field texture Option.Texture "The atlas texture, from og.load.texture."
  field columns u32 "Cells across." {
    default 1
    min 1
  }
  field rows u32 "Cells down." {
    default 1
    min 1
  }
  field names List.String "Each cell's name, in reading order: og.ui.icon draws a cell by its name."
}
```

<sub>`api.ogt` line 7862</sub>

<a id="type-AnchorDesc"></a>

### `AnchorDesc` (desc struct)

An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `anchor` | `Anchor` | `"topLeft"` | — | — | — | Where it sits. |
| `margin` | `number \| null` | `null` | — | — | — | The inset from the safe area's edges; null is the density's gutter. |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Moves it right after placing. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Moves it down after placing. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the content. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits the content. |
| `over` | `boolean` | `false` | — | — | — | Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them. |
| `frame` | `SkinPart \| null` | `null` | — | — | — | The skin frame its background takes; null is the panel's. |

```ogt
struct AnchorDesc {
  doc "An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls)."
  desc true
  field anchor Anchor "Where it sits."
  field margin Option.f32 "The inset from the safe area's edges; null is the density's gutter."
  field offsetX f32 "Moves it right after placing." {
    unit cssPx
  }
  field offsetY f32 "Moves it down after placing." {
    unit cssPx
  }
  field direction Direction "Which way it lays out its children."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field padding Insets "Space inside its edges."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null draws none."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field over bool "Draw over dialogs and spotlights (a tutorial callout, a dragged item); the default draws under them."
  field frame Option.SkinPart "The skin frame its background takes; null is the panel's."
}
```

<sub>`api.ogt` line 7877</sub>

<a id="type-TransitionDesc"></a>

### `TransitionDesc` (desc struct)

How a transition moves, timed in ticks from the motion tokens.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `TransitionKind` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long entering and leaving take; 0 is motion.duration.base (240 ms). |
| `distance` | `number` | `0` | ≥ 0 | `cssPx` | — | How far slides travel; 0 is space.xl. |
| `instant` | `boolean` | `false` | — | — | — | Shown at once the first time it is built (no entrance). |

```ogt
struct TransitionDesc {
  doc "How a transition moves, timed in ticks from the motion tokens."
  desc true
  field kind TransitionKind "The movement."
  field durationMs f32 "How long entering and leaving take; 0 is motion.duration.base (240 ms)." {
    unit ms
    min 0
  }
  field distance f32 "How far slides travel; 0 is space.xl." {
    unit cssPx
    min 0
  }
  field instant bool "Shown at once the first time it is built (no entrance)."
}
```

<sub>`api.ogt` line 7905</sub>

<a id="type-Presence"></a>

### `Presence` (struct)

Where a transition or dialog is this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | Build its content: shown, entering or leaving. |
| `progress` | `number` | `0` | — | `ratio` | — | 0 gone, 1 settled. |
| `entering` | `boolean` | `false` | — | — | — | Moving in. |
| `leaving` | `boolean` | `false` | — | — | — | Moving out. |

```ogt
struct Presence {
  doc "Where a transition or dialog is this tick."
  field visible bool "Build its content: shown, entering or leaving."
  field progress f32 "0 gone, 1 settled." {
    unit ratio
  }
  field entering bool "Moving in."
  field leaving bool "Moving out."
}
```

<sub>`api.ogt` line 7920</sub>

<a id="type-ModalDesc"></a>

### `ModalDesc` (desc struct)

A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | A title over the content; empty is none. |
| `width` | `Size \| null` | `null` | — | — | — | The panel's width; null is 420 CSS px, at most the surface less its gutters. |
| `locked` | `boolean` | `false` | — | — | — | Back and the backdrop do not dismiss it (a choice must be made). |
| `ignoreBackdrop` | `boolean` | `false` | — | — | — | A press on the backdrop does not dismiss it. |
| `transition` | `TransitionKind \| null` | `null` | — | — | — | How it enters and leaves; null is scale. |
| `backdrop` | `Paint \| null` | `null` | — | — | — | The backdrop's fill; null is scrim. |
| `bare` | `boolean` | `false` | — | — | — | No panel: the content sits centred on the backdrop (full-screen menus, loading). |

```ogt
struct ModalDesc {
  doc "A dialog: a scrim over everything and a centred panel. Focus moves in and stays while it is the topmost; Back and the backdrop dismiss it unless locked."
  desc true
  field title String "A title over the content; empty is none."
  field width Option.Size "The panel's width; null is 420 CSS px, at most the surface less its gutters."
  field locked bool "Back and the backdrop do not dismiss it (a choice must be made)."
  field ignoreBackdrop bool "A press on the backdrop does not dismiss it."
  field transition Option.TransitionKind "How it enters and leaves; null is scale."
  field backdrop Option.Paint "The backdrop's fill; null is scrim."
  field bare bool "No panel: the content sits centred on the backdrop (full-screen menus, loading)."
}
```

<sub>`api.ogt` line 7930</sub>

<a id="type-ModalResult"></a>

### `ModalResult` (struct)

A dialog's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `visible` | `boolean` | `false` | — | — | — | Build its content (open, or leaving). |
| `dismissed` | `boolean` | `false` | — | — | — | Back or a press on the backdrop asked to close it: set your open flag false. |

```ogt
struct ModalResult {
  doc "A dialog's answer this tick."
  field visible bool "Build its content (open, or leaving)."
  field dismissed bool "Back or a press on the backdrop asked to close it: set your open flag false."
}
```

<sub>`api.ogt` line 7942</sub>

<a id="type-SpotlightDesc"></a>

### `SpotlightDesc` (desc struct)

A spotlight: the surface dimmed except a hole around a target.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `padding` | `number` | `0` | ≥ 0 | `cssPx` | — | Room around the target; 0 is space.sm. |

```ogt
struct SpotlightDesc {
  doc "A spotlight: the surface dimmed except a hole around a target."
  desc true
  field padding f32 "Room around the target; 0 is space.sm." {
    unit cssPx
    min 0
  }
}
```

<sub>`api.ogt` line 7948</sub>

<a id="type-ListDesc"></a>

### `ListDesc` (desc struct)

A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many rows there are. |
| `rowHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | Every row's height; 0 is the density's row height. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between rows. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height (it scrolls inside); null grows. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `selected` | `number \| null` | `null` | — | — | — | The selected row (fill-strong), for og.ui.list. |

```ogt
struct ListDesc {
  doc "A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row."
  desc true
  field count u32 "How many rows there are."
  field rowHeight f32 "Every row's height; 0 is the density's row height." {
    unit cssPx
    min 0
  }
  field gap f32 "Space between rows." {
    unit cssPx
  }
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height (it scrolls inside); null grows."
  field padding Insets "Space inside."
  field selected Option.u32 "The selected row (fill-strong), for og.ui.list."
}
```

<sub>`api.ogt` line 7957</sub>

<a id="type-ListRange"></a>

### `ListRange` (struct)

The rows (or cells) to build this tick: first up to, not including, last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `first` | `number` | `0` | — | — | — | The first index. |
| `last` | `number` | `0` | — | — | — | One past the last index. |

```ogt
struct ListRange {
  doc "The rows (or cells) to build this tick: first up to, not including, last."
  field first u32 "The first index."
  field last u32 "One past the last index."
}
```

<sub>`api.ogt` line 7974</sub>

<a id="type-ListResult"></a>

### `ListResult` (struct)

What happened in a list or grid this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `clicked` | `number \| null` | `null` | — | — | — | The row or cell pressed (or activated) this tick. |
| `dropFrom` | `number \| null` | `null` | — | — | — | A drag and drop ended: the index it started on. |
| `dropTo` | `number \| null` | `null` | — | — | — | And the index it was released over. |

```ogt
struct ListResult {
  doc "What happened in a list or grid this tick."
  field clicked Option.u32 "The row or cell pressed (or activated) this tick."
  field dropFrom Option.u32 "A drag and drop ended: the index it started on."
  field dropTo Option.u32 "And the index it was released over."
}
```

<sub>`api.ogt` line 7980</sub>

<a id="type-GridDesc"></a>

### `GridDesc` (desc struct)

A virtualised grid of equal cells (inventories, level select), scrolling vertically.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many cells there are. |
| `cellWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's width; 0 is 64 (never under the 44 px touch target). |
| `cellHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's height; 0 is its width. |
| `columns` | `number` | `0` | — | — | — | Cells across; 0 is as many as fit. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | Space between cells; 0 is space.sm. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits (exactly the columns when columns is set). |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null grows. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside. |
| `selected` | `number \| null` | `null` | — | — | — | The selected cell, for og.ui.grid. |

```ogt
struct GridDesc {
  doc "A virtualised grid of equal cells (inventories, level select), scrolling vertically."
  desc true
  field count u32 "How many cells there are."
  field cellWidth f32 "A cell's width; 0 is 64 (never under the 44 px touch target)." {
    unit cssPx
    min 0
  }
  field cellHeight f32 "A cell's height; 0 is its width." {
    unit cssPx
    min 0
  }
  field columns u32 "Cells across; 0 is as many as fit."
  field gap f32 "Space between cells; 0 is space.sm." {
    unit cssPx
    min 0
  }
  field width Option.Size "Its width; null fits (exactly the columns when columns is set)."
  field height Option.Size "Its height; null grows."
  field padding Insets "Space inside."
  field selected Option.u32 "The selected cell, for og.ui.grid."
}
```

<sub>`api.ogt` line 7987</sub>

<a id="type-TabsDesc"></a>

### `TabsDesc` (desc struct)

A segmented control of tabs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fill` | `boolean` | `false` | — | — | — | Equal segments across the width. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits (grows with fill). |

```ogt
struct TabsDesc {
  doc "A segmented control of tabs."
  desc true
  field fill bool "Equal segments across the width."
  field width Option.Size "Its width; null fits (grows with fill)."
}
```

<sub>`api.ogt` line 8010</sub>

<a id="type-ChoiceResult"></a>

### `ChoiceResult` (struct)

A choice control's response and the chosen index after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it; changed when the choice moved. |
| `selected` | `number` | `0` | — | — | — | The chosen index, to keep in game state and pass back next tick. |

```ogt
struct ChoiceResult {
  doc "A choice control's response and the chosen index after this tick's input."
  field response Response "What happened to it; changed when the choice moved."
  field selected u32 "The chosen index, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 8017</sub>

<a id="type-SelectDesc"></a>

### `SelectDesc` (desc struct)

A select: a trigger that opens a menu of options.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown while nothing is chosen. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows from 160. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |

```ogt
struct SelectDesc {
  doc "A select: a trigger that opens a menu of options."
  desc true
  field placeholder String "Shown while nothing is chosen."
  field width Option.Size "Its width; null grows from 160."
  field disabled bool "Neutral and inert."
}
```

<sub>`api.ogt` line 8023</sub>

<a id="type-StepperDesc"></a>

### `StepperDesc` (desc struct)

A number stepper's range, step and digits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | — | — | The smallest value; min equal to max (the zero value) is unbounded. |
| `max` | `number` | `0` | — | — | — | The largest value. |
| `step` | `number` | `0` | ≥ 0 | — | — | How much a press changes it; 0 is 1. |
| `decimals` | `number` | `0` | ≤ 6 | — | — | Digits after the point. |

```ogt
struct StepperDesc {
  doc "A number stepper's range, step and digits."
  desc true
  field min f32 "The smallest value; min equal to max (the zero value) is unbounded."
  field max f32 "The largest value."
  field step f32 "How much a press changes it; 0 is 1." {
    min 0
  }
  field decimals u32 "Digits after the point." {
    max 6
  }
}
```

<sub>`api.ogt` line 8031</sub>

<a id="type-StepperResult"></a>

### `StepperResult` (struct)

A stepper's response and value after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it. |
| `value` | `number` | `0` | — | — | — | The value, to keep in game state and pass back next tick. |

```ogt
struct StepperResult {
  doc "A stepper's response and value after this tick's input."
  field response Response "What happened to it."
  field value f32 "The value, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 8044</sub>

<a id="type-RadioDesc"></a>

### `RadioDesc` (desc struct)

How a radio group lays out.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `horizontal` | `boolean` | `false` | — | — | — | Options in a row (the default is a column). |

```ogt
struct RadioDesc {
  doc "How a radio group lays out."
  desc true
  field horizontal bool "Options in a row (the default is a column)."
}
```

<sub>`api.ogt` line 8050</sub>

<a id="type-CheckListResult"></a>

### `CheckListResult` (struct)

A checkbox list's response and values after this tick's input.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `response` | `Response` | `{ hovered: false, pressed: false, clicked: false, changed: false, focused: false, submitted: false, dropped: null, pad: null, focusedBy: 0 }` | — | — | — | What happened to it; changed when any box flipped. |
| `values` | `readonly boolean[]` | `[]` | — | — | — | The values, to keep in game state and pass back next tick. |

```ogt
struct CheckListResult {
  doc "A checkbox list's response and values after this tick's input."
  field response Response "What happened to it; changed when any box flipped."
  field values List.bool "The values, to keep in game state and pass back next tick."
}
```

<sub>`api.ogt` line 8056</sub>

<a id="type-ToastDesc"></a>

### `ToastDesc` (desc struct)

A toast's tone and time.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tone` | `Tone` | `"neutral"` | — | — | — | Its tone: a dot of the tone's colour before the text. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it stays; 0 is 3200 ms (6000 for danger). |

```ogt
struct ToastDesc {
  doc "A toast's tone and time."
  desc true
  field tone Tone "Its tone: a dot of the tone's colour before the text."
  field durationMs f32 "How long it stays; 0 is 3200 ms (6000 for danger)." {
    unit ms
    min 0
  }
}
```

<sub>`api.ogt` line 8062</sub>

<a id="type-ChipDesc"></a>

### `ChipDesc` (desc struct)

A chip's state.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `boolean` | `false` | — | — | — | Selected: inverted to fg. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |

```ogt
struct ChipDesc {
  doc "A chip's state."
  desc true
  field selected bool "Selected: inverted to fg."
  field disabled bool "Neutral and inert."
}
```

<sub>`api.ogt` line 8072</sub>

<a id="type-MeterDesc"></a>

### `MeterDesc` (desc struct)

A meter: a capsule track with a fill that animates its changes by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MeterKind` | `"health"` | — | — | — | What it measures, which sets its colour. |
| `color` | `Paint \| null` | `null` | — | — | — | A colour of your own instead of the kind's. |
| `label` | `string` | `""` | — | — | — | Text inside the bar (such as `75 / 100`); empty is none. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | Its height; 0 is space.md, or the caption's line with a label. |
| `segments` | `number` | `0` | — | — | — | Notches dividing it into this many parts; 0 or 1 is none. |

```ogt
struct MeterDesc {
  doc "A meter: a capsule track with a fill that animates its changes by the tick."
  desc true
  field kind MeterKind "What it measures, which sets its colour."
  field color Option.Paint "A colour of your own instead of the kind's."
  field label String "Text inside the bar (such as `75 / 100`); empty is none."
  field width Option.Size "Its width; null grows."
  field height f32 "Its height; 0 is space.md, or the caption's line with a label." {
    unit cssPx
    min 0
  }
  field segments u32 "Notches dividing it into this many parts; 0 or 1 is none."
}
```

<sub>`api.ogt` line 8079</sub>

<a id="type-CooldownDesc"></a>

### `CooldownDesc` (desc struct)

A cooldown ring's size and centre.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 56 (never under 44). |
| `label` | `string` | `""` | — | — | — | Text in the middle (a key, seconds left); empty is none. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas in the middle, by name; empty is none. |

```ogt
struct CooldownDesc {
  doc "A cooldown ring's size and centre."
  desc true
  field size f32 "Its diameter; 0 is 56 (never under 44)." {
    unit cssPx
    min 0
  }
  field label String "Text in the middle (a key, seconds left); empty is none."
  field icon String "An icon of the atlas in the middle, by name; empty is none."
}
```

<sub>`api.ogt` line 8093</sub>

<a id="type-IconDesc"></a>

### `IconDesc` (desc struct)

An icon's size and ink.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its size; 0 is 24. |
| `tint` | `Paint \| null` | `null` | — | — | — | Its ink; null is fg. |

```ogt
struct IconDesc {
  doc "An icon's size and ink."
  desc true
  field size f32 "Its size; 0 is 24." {
    unit cssPx
    min 0
  }
  field tint Option.Paint "Its ink; null is fg."
}
```

<sub>`api.ogt` line 8104</sub>

<a id="type-MapMarker"></a>

### `MapMarker` (desc struct)

A marker on a mini-map.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `ratio` | — | Across the map, 0 to 1. |
| `y` | `number` | `0` | — | `ratio` | — | Down the map, 0 to 1. |
| `tone` | `Tone` | `"neutral"` | — | — | — | Its colour; neutral is the stage ink. |
| `player` | `boolean` | `false` | — | — | — | The player: the accent, larger, ringed. |

```ogt
struct MapMarker {
  doc "A marker on a mini-map."
  desc true
  field x f32 "Across the map, 0 to 1." {
    unit ratio
  }
  field y f32 "Down the map, 0 to 1." {
    unit ratio
  }
  field tone Tone "Its colour; neutral is the stage ink."
  field player bool "The player: the accent, larger, ringed."
}
```

<sub>`api.ogt` line 8114</sub>

<a id="type-MinimapDesc"></a>

### `MinimapDesc` (desc struct)

A mini-map frame.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The map; null is the stage colour. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region of the map shown, [u0, v0, u1, v1]; null is the whole texture. |
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 160. |
| `markers` | `readonly MapMarker[]` | `[]` | — | — | — | Markers over the map. |
| `label` | `string` | `""` | — | — | — | A zone name under it; empty is none. |

```ogt
struct MinimapDesc {
  doc "A mini-map frame."
  desc true
  field texture Option.Texture "The map; null is the stage colour."
  field uv Option.Array.f32.4 "The region of the map shown, [u0, v0, u1, v1]; null is the whole texture."
  field size f32 "Its side; 0 is 160." {
    unit cssPx
    min 0
  }
  field markers List.MapMarker "Markers over the map."
  field label String "A zone name under it; empty is none."
}
```

<sub>`api.ogt` line 8127</sub>

<a id="type-DialogueDesc"></a>

### `DialogueDesc` (desc struct)

A dialogue box: a portrait slot, the speaker and the line typed out by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speaker` | `string` | `""` | — | — | — | Who speaks (amber ink). |
| `text` | `string` | `""` | — | — | — | The line. A new text starts typing again. |
| `portrait` | `Texture \| null` | `null` | — | — | — | A portrait texture; null is the speaker's initial. |
| `portraitUv` | `[number, number, number, number] \| null` | `null` | — | — | — | The portrait's region of the texture; null is the whole texture. |
| `portraitIcon` | `string` | `""` | — | — | — | A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial. |
| `charsPerSecond` | `number` | `0` | ≥ 0 | — | — | Typing speed; 0 is 40. |
| `portraitRight` | `boolean` | `false` | — | — | — | The portrait on the right. |

```ogt
struct DialogueDesc {
  doc "A dialogue box: a portrait slot, the speaker and the line typed out by the tick."
  desc true
  field speaker String "Who speaks (amber ink)."
  field text String "The line. A new text starts typing again."
  field portrait Option.Texture "A portrait texture; null is the speaker's initial."
  field portraitUv Option.Array.f32.4 "The portrait's region of the texture; null is the whole texture."
  field portraitIcon String "A portrait from the icon atlas by name, when no texture is given; empty is the speaker's initial."
  field charsPerSecond f32 "Typing speed; 0 is 40." {
    min 0
  }
  field portraitRight bool "The portrait on the right."
}
```

<sub>`api.ogt` line 8140</sub>

<a id="type-DialogueResult"></a>

### `DialogueResult` (struct)

A dialogue box's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shown` | `number` | `0` | — | — | — | Characters shown. |
| `done` | `boolean` | `false` | — | — | — | The whole line is shown. |
| `advanced` | `boolean` | `false` | — | — | — | Pressed (or activated) while done: show the next line. |

```ogt
struct DialogueResult {
  doc "A dialogue box's answer this tick."
  field shown u32 "Characters shown."
  field done bool "The whole line is shown."
  field advanced bool "Pressed (or activated) while done: show the next line."
}
```

<sub>`api.ogt` line 8154</sub>

<a id="type-JoystickDesc"></a>

### `JoystickDesc` (desc struct)

A virtual joystick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | The base's diameter; 0 is 128 (never under three touch targets). |
| `deadZone` | `number` | `0` | 0 … 1 | `ratio` | — | The ignored radius; 0 is 0.15. |

```ogt
struct JoystickDesc {
  doc "A virtual joystick."
  desc true
  field size f32 "The base's diameter; 0 is 128 (never under three touch targets)." {
    unit cssPx
    min 0
  }
  field deadZone f32 "The ignored radius; 0 is 0.15." {
    unit ratio
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 8161</sub>

<a id="type-JoystickResult"></a>

### `JoystickResult` (struct)

A joystick's vector this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | — | — | -1 left to 1 right; 0 inside the dead zone. |
| `y` | `number` | `0` | — | — | — | -1 up to 1 down; 0 inside the dead zone. |
| `active` | `boolean` | `false` | — | — | — | A press that started on it is held. |

```ogt
struct JoystickResult {
  doc "A joystick's vector this tick."
  field x f32 "-1 left to 1 right; 0 inside the dead zone."
  field y f32 "-1 up to 1 down; 0 inside the dead zone."
  field active bool "A press that started on it is held."
}
```

<sub>`api.ogt` line 8175</sub>

<a id="type-DpadDesc"></a>

### `DpadDesc` (desc struct)

A d-pad.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | The pad's size; 0 is 132 (each arm at least 44). |

```ogt
struct DpadDesc {
  doc "A d-pad."
  desc true
  field size f32 "The pad's size; 0 is 132 (each arm at least 44)." {
    unit cssPx
    min 0
  }
}
```

<sub>`api.ogt` line 8182</sub>

<a id="type-DpadResult"></a>

### `DpadResult` (struct)

Which arms of a d-pad are held this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `up` | `boolean` | `false` | — | — | — | Up is held. |
| `down` | `boolean` | `false` | — | — | — | Down is held. |
| `left` | `boolean` | `false` | — | — | — | Left is held. |
| `right` | `boolean` | `false` | — | — | — | Right is held. |

```ogt
struct DpadResult {
  doc "Which arms of a d-pad are held this tick."
  field up bool "Up is held."
  field down bool "Down is held."
  field left bool "Left is held."
  field right bool "Right is held."
}
```

<sub>`api.ogt` line 8191</sub>

<a id="type-ActionButtonDesc"></a>

### `ActionButtonDesc` (desc struct)

A round touch action button (jump, fire).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its diameter; 0 is 64 (never under 44). |
| `primary` | `boolean` | `false` | — | — | — | The amber button (one per screen). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas by name, instead of the label; empty is the label. |

```ogt
struct ActionButtonDesc {
  doc "A round touch action button (jump, fire)."
  desc true
  field size f32 "Its diameter; 0 is 64 (never under 44)." {
    unit cssPx
    min 0
  }
  field primary bool "The amber button (one per screen)."
  field icon String "An icon of the atlas by name, instead of the label; empty is the label."
}
```

<sub>`api.ogt` line 8199</sub>

<a id="type-SkeletonDesc"></a>

### `SkeletonDesc` (desc struct)

A skeleton: the neutral fill standing in for content that is loading.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows. |
| `height` | `number` | `0` | ≥ 0 | `cssPx` | — | A line's height; 0 is the body size. |
| `lines` | `number` | `0` | — | — | — | Stacked lines; 0 or 1 is one, the last of several is shorter. |

```ogt
struct SkeletonDesc {
  doc "A skeleton: the neutral fill standing in for content that is loading."
  desc true
  field width Option.Size "Its width; null grows."
  field height f32 "A line's height; 0 is the body size." {
    unit cssPx
    min 0
  }
  field lines u32 "Stacked lines; 0 or 1 is one, the last of several is shorter."
}
```

<sub>`api.ogt` line 8210</sub>

<a id="type-UiItemBuild"></a>

### `UiItemBuild` (callback)

Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it.

Game code supplies this function.

```ts
type UiItemBuild = (index: number) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |

```ogt
callback UiItemBuild {
  doc "Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it."
  param index u32 "Its index."
}
```

<sub>`api.ogt` line 8221</sub>

<a id="type-A11yNode"></a>

### `A11yNode` (struct)

One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its path, as touch_move_to and og.ui.focus name it. |
| `parent` | `string` | `""` | — | — | — | The path of the nearest node above it in the tree; empty at the top. |
| `role` | `string` | `""` | — | — | — | Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group. |
| `name` | `string` | `""` | — | — | — | Its accessible name: what a screen reader says it is. |
| `value` | `string` | `""` | — | — | — | Its value as text (a slider's number, a progress's percent, a field's text); empty for none. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states: disabled, focused, focusable, checked, selected, expanded, pressed, busy, offscreen (a scroll container cuts it out of view). |
| `x` | `number` | `0` | — | `cssPx` | — | Left edge of the whole element. |
| `y` | `number` | `0` | — | `cssPx` | — | Top edge of the whole element. |
| `width` | `number` | `0` | — | `cssPx` | — | Width. |
| `height` | `number` | `0` | — | `cssPx` | — | Height. |

```ogt
struct A11yNode {
  doc "One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out)."
  field path String "Its path, as touch_move_to and og.ui.focus name it."
  field parent String "The path of the nearest node above it in the tree; empty at the top."
  field role String "Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group."
  field name String "Its accessible name: what a screen reader says it is."
  field value String "Its value as text (a slider's number, a progress's percent, a field's text); empty for none."
  field states List.String "Its states: disabled, focused, focusable, checked, selected, expanded, pressed, busy, offscreen (a scroll container cuts it out of view)."
  field x f32 "Left edge of the whole element." {
    unit cssPx
  }
  field y f32 "Top edge of the whole element." {
    unit cssPx
  }
  field width f32 "Width." {
    unit cssPx
  }
  field height f32 "Height." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 8584</sub>

<a id="type-A11yTree"></a>

### `A11yTree` (struct)

The last frame's accessibility tree, built from the same nodes the UI draws and takes input on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `nodes` | `readonly A11yNode[]` | `[]` | — | — | — | The nodes, in tree order: the page, then each layer over it (a dialog after the page). |
| `focusOrder` | `readonly string[]` | `[]` | — | — | — | The paths keyboard Tab and the pads visit, in order: the nodes marked focusable, within the topmost dialog while one is open. |
| `focused` | `string` | `""` | — | — | — | The path keyboard and pad 0's focus is on; empty for none. |

```ogt
struct A11yTree {
  doc "The last frame's accessibility tree, built from the same nodes the UI draws and takes input on."
  field nodes List.A11yNode "The nodes, in tree order: the page, then each layer over it (a dialog after the page)."
  field focusOrder List.String "The paths keyboard Tab and the pads visit, in order: the nodes marked focusable, within the topmost dialog while one is open."
  field focused String "The path keyboard and pad 0's focus is on; empty for none."
}
```

<sub>`api.ogt` line 8606</sub>

<a id="type-CounterDesc"></a>

### `CounterDesc` (desc struct)

A number that counts to its value.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font: mono (the numeric role) for counts in a row, display for a big score. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is fg. |
| `align` | `TextAlign` | `"start"` | — | — | — | How it aligns in its width. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a change takes to count; 0 is 900 ms. |
| `from` | `number \| null` | `null` | — | — | — | Where the first count starts; null starts at the value (no count). |
| `decimals` | `number` | `0` | — | — | — | Digits after the point. |
| `prefix` | `string` | `""` | — | — | — | Text before the number. |
| `suffix` | `string` | `""` | — | — | — | Text after the number. |

```ogt
struct CounterDesc {
  doc "A number that counts to its value."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field font FontRole "Which font: mono (the numeric role) for counts in a row, display for a big score."
  field color Option.Paint "The ink; null is fg."
  field align TextAlign "How it aligns in its width."
  field width Option.Size "Its width; null fits the text."
  field durationMs f32 "How long a change takes to count; 0 is 900 ms." {
    unit ms
    min 0
  }
  field from Option.f64 "Where the first count starts; null starts at the value (no count)."
  field decimals u32 "Digits after the point."
  field prefix String "Text before the number."
  field suffix String "Text after the number."
}
```

<sub>`api.ogt` line 8643</sub>

<a id="type-BurstDesc"></a>

### `BurstDesc` (desc struct)

Rays and sparks thrown from an element.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | Rays; 0 is 8. |
| `color` | `Paint \| null` | `null` | — | — | — | Their ink; null is the accent. |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | How far they fly from the element's centre; 0 is 1.4 times its size. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long they fly; 0 is motion.duration.slow plus base. |
| `delayMs` | `number` | `0` | ≥ 0 | `ms` | — | Wait this long after the element first appears. |

```ogt
struct BurstDesc {
  doc "Rays and sparks thrown from an element."
  desc true
  field count u32 "Rays; 0 is 8."
  field color Option.Paint "Their ink; null is the accent."
  field radius f32 "How far they fly from the element's centre; 0 is 1.4 times its size." {
    unit cssPx
    min 0
  }
  field durationMs f32 "How long they fly; 0 is motion.duration.slow plus base." {
    unit ms
    min 0
  }
  field delayMs f32 "Wait this long after the element first appears." {
    unit ms
    min 0
  }
}
```

<sub>`api.ogt` line 8669</sub>

<a id="type-ShakeDesc"></a>

### `ShakeDesc` (desc struct)

A shake of an element: three sideways swings decaying to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `amount` | `number` | `0` | ≥ 0 | `cssPx` | — | The first swing; 0 is the skin's motion.shake, else 6. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it takes to come to rest; 0 is motion.duration.slow. |
| `sound` | `boolean` | `false` | — | — | — | Also play the skin's error sound, as a refused press does. |

```ogt
struct ShakeDesc {
  doc "A shake of an element: three sideways swings decaying to rest."
  desc true
  field amount f32 "The first swing; 0 is the skin's motion.shake, else 6." {
    unit cssPx
    min 0
  }
  field durationMs f32 "How long it takes to come to rest; 0 is motion.duration.slow." {
    unit ms
    min 0
  }
  field sound bool "Also play the skin's error sound, as a refused press does."
}
```

<sub>`api.ogt` line 8694</sub>

<a id="type-ScreenWipe"></a>

### `ScreenWipe` (enum)

How a screen transition covers the old screen and reveals the new one.

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | The cover fades in and out. |
| `"wipe"` | An edge sweeps across. |
| `"iris"` | A circle closes on the centre and opens again. |
| `"dissolve"` | Cells appear in a fixed scattered order. |
| `"slide"` | A sheet slides over and away. |

```ogt
enum ScreenWipe {
  doc "How a screen transition covers the old screen and reveals the new one."
  variant fade "The cover fades in and out."
  variant wipe "An edge sweeps across."
  variant iris "A circle closes on the centre and opens again."
  variant dissolve "Cells appear in a fixed scattered order."
  variant slide "A sheet slides over and away."
}
```

<sub>`api.ogt` line 8714</sub>

<a id="type-WipeSide"></a>

### `WipeSide` (enum)

Where a wipe or a slide comes from.

A string: one of the variants below. The zero value is `"left"`.

| Variant | Doc |
|---|---|
| `"left"` | From the left. |
| `"right"` | From the right. |
| `"top"` | From the top. |
| `"bottom"` | From the bottom. |

```ogt
enum WipeSide {
  doc "Where a wipe or a slide comes from."
  variant left "From the left."
  variant right "From the right."
  variant top "From the top."
  variant bottom "From the bottom."
}
```

<sub>`api.ogt` line 8723</sub>

<a id="type-ScreenTransitionDesc"></a>

### `ScreenTransitionDesc` (desc struct)

How a screen transition looks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ScreenWipe` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | Cover and reveal together; 0 is twice motion.duration.slow. |
| `color` | `Paint \| null` | `null` | — | — | — | The cover's colour (the texture's tint); null is bg. |
| `texture` | `Texture \| null` | `null` | — | — | — | A texture stretched over the surface as the cover; null is the flat colour. |
| `from` | `WipeSide` | `"left"` | — | — | — | Where a wipe or a slide comes from. |

```ogt
struct ScreenTransitionDesc {
  doc "How a screen transition looks."
  desc true
  field kind ScreenWipe "The movement."
  field durationMs f32 "Cover and reveal together; 0 is twice motion.duration.slow." {
    unit ms
    min 0
  }
  field color Option.Paint "The cover's colour (the texture's tint); null is bg."
  field texture Option.Texture "A texture stretched over the surface as the cover; null is the flat colour."
  field from WipeSide "Where a wipe or a slide comes from."
}
```

<sub>`api.ogt` line 8731</sub>

<a id="type-ScreenTransitionResult"></a>

### `ScreenTransitionResult` (struct)

Where a screen transition is this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shown` | `string` | `""` | — | — | — | The screen to build this tick: the old one until the cover closes, then the new one. |
| `covering` | `boolean` | `false` | — | — | — | The cover is closing over the old screen. |
| `revealing` | `boolean` | `false` | — | — | — | The cover is opening on the new screen. |
| `cover` | `number` | `0` | — | `ratio` | — | How much of the surface is covered, 0 to 1. |

```ogt
struct ScreenTransitionResult {
  doc "Where a screen transition is this tick."
  field shown String "The screen to build this tick: the old one until the cover closes, then the new one."
  field covering bool "The cover is closing over the old screen."
  field revealing bool "The cover is opening on the new screen."
  field cover f32 "How much of the surface is covered, 0 to 1." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 8744</sub>

<a id="type-GraphicsQuality"></a>

### `GraphicsQuality` (enum)

A graphics quality preset.

A string: one of the variants below. The zero value is `"medium"`.

| Variant | Doc |
|---|---|
| `"medium"` | Medium. |
| `"low"` | Low: the fewest effects. |
| `"high"` | High: every effect. |

```ogt
enum GraphicsQuality {
  doc "A graphics quality preset."
  variant medium "Medium."
  variant low "Low: the fewest effects."
  variant high "High: every effect."
}
```

<sub>`api.ogt` line 8771</sub>

<a id="type-UiAction"></a>

### `UiAction` (callback)

A template's action, by its id (a menu item, `buy:<item>`, `share`).

Game code supplies this function.

```ts
type UiAction = (action: string) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `action` | `string` | no | The action's id. |

```ogt
callback UiAction {
  doc "A template's action, by its id (a menu item, `buy:<item>`, `share`)."
  param action String "The action's id."
}
```

<sub>`api.ogt` line 8778</sub>

<a id="type-SettingsChanged"></a>

### `SettingsChanged` (callback)

The settings after a change: persist them (cloud save, local storage through the shell) and apply them.

Game code supplies this function.

```ts
type SettingsChanged = (settings: GameSettings) => void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `settings` | `GameSettings` | yes | The new settings. |

```ogt
callback SettingsChanged {
  doc "The settings after a change: persist them (cloud save, local storage through the shell) and apply them."
  param settings GameSettings "The new settings."
}
```

<sub>`api.ogt` line 8783</sub>

<a id="type-MenuItem"></a>

### `MenuItem` (desc struct)

An item of a menu: an id the game acts on and the label a player reads.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The id answered when it is chosen; also its probe id. |
| `label` | `string` | `""` | — | — | — | What it says. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `primary` | `boolean` | `false` | — | — | — | The amber action (one per screen). |

```ogt
struct MenuItem {
  doc "An item of a menu: an id the game acts on and the label a player reads."
  desc true
  field id String "The id answered when it is chosen; also its probe id."
  field label String "What it says."
  field disabled bool "Shown but inert."
  field primary bool "The amber action (one per screen)."
}
```

<sub>`api.ogt` line 8788</sub>

<a id="type-MenuResult"></a>

### `MenuResult` (struct)

What a menu-like template answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `string \| null` | `null` | — | — | — | The id of the item chosen this tick. |

```ogt
struct MenuResult {
  doc "What a menu-like template answered this tick."
  field selected Option.String "The id of the item chosen this tick."
}
```

<sub>`api.ogt` line 8797</sub>

<a id="type-MainMenuDesc"></a>

### `MainMenuDesc` (desc struct)

A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `""` | — | — | — | The game's title, in display type. |
| `subtitle` | `string` | `""` | — | — | — | A line under it; empty is none. |
| `items` | `readonly MenuItem[]` | `[]` | — | — | — | The items, such as Play, Continue, Settings, Quit. |
| `footer` | `string` | `""` | — | — | — | A line at the bottom (a version, credits); empty is none. |
| `onSelect` | `UiAction \| null` | `null` | — | — | — | Called with the chosen item's id. |

```ogt
struct MainMenuDesc {
  doc "A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone."
  desc true
  field title String "The game's title, in display type."
  field subtitle String "A line under it; empty is none."
  field items List.MenuItem "The items, such as Play, Continue, Settings, Quit."
  field footer String "A line at the bottom (a version, credits); empty is none."
  field onSelect Option.UiAction "Called with the chosen item's id."
}
```

<sub>`api.ogt` line 8802</sub>

<a id="type-PauseMenuDesc"></a>

### `PauseMenuDesc` (desc struct)

A pause screen: a dialog over the frozen game with Resume first. Back resumes.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether the game is paused. |
| `title` | `string` | `"Paused"` | — | — | — | Its title. |
| `items` | `readonly MenuItem[]` | `[]` | — | — | — | The items; empty is Resume, Settings, Quit to menu (`resume`, `settings`, `quit`). |
| `onSelect` | `UiAction \| null` | `null` | — | — | — | Called with the chosen item's id; Back answers `resume`. |

```ogt
struct PauseMenuDesc {
  doc "A pause screen: a dialog over the frozen game with Resume first. Back resumes."
  desc true
  field open bool "Whether the game is paused."
  field title String "Its title." {
    default "Paused"
  }
  field items List.MenuItem "The items; empty is Resume, Settings, Quit to menu (`resume`, `settings`, `quit`)."
  field onSelect Option.UiAction "Called with the chosen item's id; Back answers `resume`."
}
```

<sub>`api.ogt` line 8812</sub>

<a id="type-KeyBinding"></a>

### `KeyBinding` (desc struct)

A control the settings screen lists: an action and the key bound to it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action's id. |
| `label` | `string` | `""` | — | — | — | What a player reads. |
| `key` | `string` | `""` | — | — | — | The bound key, as shown (`Space`, `A`, `←`). |

```ogt
struct KeyBinding {
  doc "A control the settings screen lists: an action and the key bound to it."
  desc true
  field action String "The action's id."
  field label String "What a player reads."
  field key String "The bound key, as shown (`Space`, `A`, `←`)."
}
```

<sub>`api.ogt` line 8823</sub>

<a id="type-LanguageOption"></a>

### `LanguageOption` (desc struct)

A language the settings screen offers, in its own name.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `code` | `string` | `""` | — | — | — | Its code (`en`, `zh-CN`). |
| `label` | `string` | `""` | — | — | — | Its name in itself (`English`, `简体中文`). |

```ogt
struct LanguageOption {
  doc "A language the settings screen offers, in its own name."
  desc true
  field code String "Its code (`en`, `zh-CN`)."
  field label String "Its name in itself (`English`, `简体中文`)."
}
```

<sub>`api.ogt` line 8831</sub>

<a id="type-GameSettings"></a>

### `GameSettings` (desc struct)

The settings most games share. The game keeps them (a resource, a save) and passes them in every tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `masterVolume` | `number` | `1` | 0 … 1 | `ratio` | — | The master volume. |
| `musicVolume` | `number` | `0.8` | 0 … 1 | `ratio` | — | The music bus. |
| `sfxVolume` | `number` | `1` | 0 … 1 | `ratio` | — | The effects bus. |
| `muted` | `boolean` | `false` | — | — | — | Everything silent. |
| `quality` | `GraphicsQuality` | `"medium"` | — | — | — | The graphics preset. |
| `showFps` | `boolean` | `false` | — | — | — | Show the frame rate. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Keep state changes, drop movement (the UI's transitions only fade). |
| `highContrast` | `boolean` | `false` | — | — | — | The high-contrast theme. |
| `vibration` | `boolean` | `true` | — | — | — | Vibrate on hits (where the device can). |
| `invertY` | `boolean` | `false` | — | — | — | Invert the vertical look axis. |
| `sensitivity` | `number` | `1` | 0.1 … 3 | — | — | Look sensitivity. |
| `language` | `string` | `"en"` | — | — | — | The language code. |
| `bindings` | `readonly KeyBinding[]` | `[]` | — | — | — | The controls, as shown on the Controls tab. |

```ogt
struct GameSettings {
  doc "The settings most games share. The game keeps them (a resource, a save) and passes them in every tick."
  desc true
  field masterVolume f32 "The master volume." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field musicVolume f32 "The music bus." {
    default 0.8
    min 0
    max 1
    unit ratio
  }
  field sfxVolume f32 "The effects bus." {
    default 1
    min 0
    max 1
    unit ratio
  }
  field muted bool "Everything silent."
  field quality GraphicsQuality "The graphics preset."
  field showFps bool "Show the frame rate."
  field reducedMotion bool "Keep state changes, drop movement (the UI's transitions only fade)."
  field highContrast bool "The high-contrast theme."
  field vibration bool "Vibrate on hits (where the device can)." {
    default true
  }
  field invertY bool "Invert the vertical look axis."
  field sensitivity f32 "Look sensitivity." {
    default 1
    min 0.1
    max 3
  }
  field language String "The language code." {
    default "en"
  }
  field bindings List.KeyBinding "The controls, as shown on the Controls tab."
}
```

<sub>`api.ogt` line 8838</sub>

<a id="type-SettingsDesc"></a>

### `SettingsDesc` (desc struct)

A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is shown. |
| `settings` | `GameSettings` | `{}` | — | — | — | The settings from game state. |
| `languages` | `readonly LanguageOption[]` | `[]` | — | — | — | The languages offered; empty hides the Language tab. |
| `onChange` | `SettingsChanged \| null` | `null` | — | — | — | Called on every change (apply it at once: volume, contrast). |
| `onSave` | `SettingsChanged \| null` | `null` | — | — | — | Called when the screen closes after changes (persist here). |

```ogt
struct SettingsDesc {
  doc "A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks."
  desc true
  field open bool "Whether it is shown."
  field settings GameSettings "The settings from game state."
  field languages List.LanguageOption "The languages offered; empty hides the Language tab."
  field onChange Option.SettingsChanged "Called on every change (apply it at once: volume, contrast)."
  field onSave Option.SettingsChanged "Called when the screen closes after changes (persist here)."
}
```

<sub>`api.ogt` line 8879</sub>

<a id="type-SettingsResult"></a>

### `SettingsResult` (struct)

The settings screen's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `settings` | `GameSettings` | `{}` | — | — | — | The settings after this tick's input: keep them. |
| `changed` | `boolean` | `false` | — | — | — | Something changed this tick. |
| `closed` | `boolean` | `false` | — | — | — | The player closed the screen (Back, Done): set open false. |

```ogt
struct SettingsResult {
  doc "The settings screen's answer this tick."
  field settings GameSettings "The settings after this tick's input: keep them."
  field changed bool "Something changed this tick."
  field closed bool "The player closed the screen (Back, Done): set open false."
}
```

<sub>`api.ogt` line 8889</sub>

<a id="type-Ability"></a>

### `Ability` (desc struct)

An ability on a HUD: a cooldown ring with its key.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id (and probe id). |
| `key` | `string` | `""` | — | — | — | The key or button shown in the middle. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas instead of the key; empty is the key. |
| `remaining` | `number` | `0` | — | `ratio` | — | How much of its cooldown is left: 1 just used, 0 ready. |

```ogt
struct Ability {
  doc "An ability on a HUD: a cooldown ring with its key."
  desc true
  field id String "Its id (and probe id)."
  field key String "The key or button shown in the middle."
  field icon String "An icon of the atlas instead of the key; empty is the key."
  field remaining f32 "How much of its cooldown is left: 1 just used, 0 ready." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 8896</sub>

<a id="type-PlatformerHudDesc"></a>

### `PlatformerHudDesc` (desc struct)

A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lives` | `number` | `0` | — | — | — | Lives left. |
| `maxLives` | `number` | `3` | — | — | — | Lives at most (hearts drawn). |
| `coins` | `number` | `0` | — | — | — | Coins collected. |
| `score` | `number` | `0` | — | — | — | The score. |
| `level` | `string` | `""` | — | — | — | The level's name; empty is none. |
| `timeLeft` | `number \| null` | `null` | — | — | — | Seconds left; null hides the timer. |
| `health` | `number \| null` | `null` | — | — | — | A health bar instead of hearts, 0 to 1; null shows hearts. |
| `touch` | `boolean` | `false` | — | — | — | Show touch controls (a d-pad and Jump) on phones and tablets. |

```ogt
struct PlatformerHudDesc {
  doc "A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button."
  desc true
  field lives u32 "Lives left."
  field maxLives u32 "Lives at most (hearts drawn)." {
    default 3
  }
  field coins u32 "Coins collected."
  field score u32 "The score."
  field level String "The level's name; empty is none."
  field timeLeft Option.f32 "Seconds left; null hides the timer."
  field health Option.f32 "A health bar instead of hearts, 0 to 1; null shows hearts."
  field touch bool "Show touch controls (a d-pad and Jump) on phones and tablets."
}
```

<sub>`api.ogt` line 8907</sub>

<a id="type-HudResult"></a>

### `HudResult` (struct)

What a HUD answered this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `paused` | `boolean` | `false` | — | — | — | The pause button was pressed (or Back). |
| `action` | `string \| null` | `null` | — | — | — | A HUD action pressed this tick (an ability's id, `jump`, a generator's `buy:<id>`, `tap`). |
| `moveX` | `number` | `0` | — | — | — | The touch controls' horizontal input, -1 to 1. |
| `moveY` | `number` | `0` | — | — | — | The touch controls' vertical input, -1 (up) to 1. |
| `held` | `readonly string[]` | `[]` | — | — | — | The touch action buttons held this tick. |

```ogt
struct HudResult {
  doc "What a HUD answered this tick."
  field paused bool "The pause button was pressed (or Back)."
  field action Option.String "A HUD action pressed this tick (an ability's id, `jump`, a generator's `buy:<id>`, `tap`)."
  field moveX f32 "The touch controls' horizontal input, -1 to 1."
  field moveY f32 "The touch controls' vertical input, -1 (up) to 1."
  field held List.String "The touch action buttons held this tick."
}
```

<sub>`api.ogt` line 8922</sub>

<a id="type-ShooterHudDesc"></a>

### `ShooterHudDesc` (desc struct)

A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `health` | `number` | `0` | — | — | — | Health. |
| `maxHealth` | `number` | `100` | — | — | — | Full health. |
| `armor` | `number` | `0` | — | — | — | Armor; 0 with maxArmor 0 hides it. |
| `maxArmor` | `number` | `0` | — | — | — | Full armor. |
| `ammo` | `number` | `0` | — | — | — | Rounds in the magazine. |
| `magazine` | `number` | `0` | — | — | — | Rounds a magazine holds. |
| `reserve` | `number` | `0` | — | — | — | Rounds in reserve. |
| `weapon` | `string` | `""` | — | — | — | The weapon's name. |
| `kills` | `number` | `0` | — | — | — | Kills. |
| `abilities` | `readonly Ability[]` | `[]` | — | — | — | Abilities with cooldowns. |
| `crosshair` | `boolean` | `true` | — | — | — | Draw a crosshair at the centre. |
| `map` | `MinimapDesc \| null` | `null` | — | — | — | A mini-map top left; null is none. |
| `touch` | `boolean` | `false` | — | — | — | Show touch controls (a stick and Fire) on phones and tablets. |

```ogt
struct ShooterHudDesc {
  doc "A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map."
  desc true
  field health f32 "Health."
  field maxHealth f32 "Full health." {
    default 100
  }
  field armor f32 "Armor; 0 with maxArmor 0 hides it."
  field maxArmor f32 "Full armor."
  field ammo u32 "Rounds in the magazine."
  field magazine u32 "Rounds a magazine holds."
  field reserve u32 "Rounds in reserve."
  field weapon String "The weapon's name."
  field kills u32 "Kills."
  field abilities List.Ability "Abilities with cooldowns."
  field crosshair bool "Draw a crosshair at the centre." {
    default true
  }
  field map Option.MinimapDesc "A mini-map top left; null is none."
  field touch bool "Show touch controls (a stick and Fire) on phones and tablets."
}
```

<sub>`api.ogt` line 8931</sub>

<a id="type-PuzzleHudDesc"></a>

### `PuzzleHudDesc` (desc struct)

A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `level` | `string` | `""` | — | — | — | The level's name. |
| `score` | `number` | `0` | — | — | — | The score. |
| `target` | `number` | `1000` | — | — | — | The score that wins. |
| `stars` | `readonly number[]` | `[]` | — | — | — | The scores for one, two and three stars; empty is a third, two thirds and all of the target. |
| `moves` | `number \| null` | `null` | — | — | — | Moves left; null hides it. |
| `timeLeft` | `number \| null` | `null` | — | — | — | Seconds left; null hides it. |

```ogt
struct PuzzleHudDesc {
  doc "A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer."
  desc true
  field level String "The level's name."
  field score u32 "The score."
  field target u32 "The score that wins." {
    default 1000
  }
  field stars List.u32 "The scores for one, two and three stars; empty is a third, two thirds and all of the target."
  field moves Option.u32 "Moves left; null hides it."
  field timeLeft Option.f32 "Seconds left; null hides it."
}
```

<sub>`api.ogt` line 8953</sub>

<a id="type-Currency"></a>

### `Currency` (desc struct)

A currency an idle game counts.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `amount` | `number` | `0` | — | — | — | How much there is. |
| `perSecond` | `number` | `0` | — | — | — | How much it earns per second. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas; empty is none. |

```ogt
struct Currency {
  doc "A currency an idle game counts."
  desc true
  field id String "Its id."
  field name String "Its name."
  field amount f64 "How much there is."
  field perSecond f64 "How much it earns per second."
  field icon String "An icon of the atlas; empty is none."
}
```

<sub>`api.ogt` line 8966</sub>

<a id="type-Generator"></a>

### `Generator` (desc struct)

A generator an idle game lists: level, cost, progress.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id: buying answers `buy:<id>`. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `level` | `number` | `0` | — | — | — | Its level. |
| `cost` | `number` | `0` | — | — | — | The next level's price. |
| `affordable` | `boolean` | `false` | — | — | — | The player can pay for it. |
| `progress` | `number` | `0` | — | `ratio` | — | Its cycle, 0 to 1. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas; empty is none. |

```ogt
struct Generator {
  doc "A generator an idle game lists: level, cost, progress."
  desc true
  field id String "Its id: buying answers `buy:<id>`."
  field name String "Its name."
  field level u32 "Its level."
  field cost f64 "The next level's price."
  field affordable bool "The player can pay for it."
  field progress f32 "Its cycle, 0 to 1." {
    unit ratio
  }
  field icon String "An icon of the atlas; empty is none."
}
```

<sub>`api.ogt` line 8976</sub>

<a id="type-IdleHudDesc"></a>

### `IdleHudDesc` (desc struct)

An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `currencies` | `readonly Currency[]` | `[]` | — | — | — | The currencies, the first the main one. |
| `generators` | `readonly Generator[]` | `[]` | — | — | — | The generators. |
| `tapLabel` | `string` | `"Tap"` | — | — | — | The tap target's label; empty hides it. |

```ogt
struct IdleHudDesc {
  doc "An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list."
  desc true
  field currencies List.Currency "The currencies, the first the main one."
  field generators List.Generator "The generators."
  field tapLabel String "The tap target's label; empty hides it." {
    default "Tap"
  }
}
```

<sub>`api.ogt` line 8990</sub>

<a id="type-RacingHudDesc"></a>

### `RacingHudDesc` (desc struct)

A racing HUD: position and lap top left, the times top right, the speedometer and gear bottom right, the boost meter, an optional mini-map.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speed` | `number` | `0` | — | — | — | The speed, in `unit`. |
| `maxSpeed` | `number` | `300` | — | — | — | The dial's top. |
| `unit` | `string` | `"km/h"` | — | — | — | The speed's unit. |
| `gear` | `string` | `""` | — | — | — | The gear (`3`, `R`, `N`). |
| `lap` | `number` | `1` | — | — | — | The lap. |
| `laps` | `number` | `3` | — | — | — | Laps in the race. |
| `position` | `number` | `1` | — | — | — | The player's place. |
| `racers` | `number` | `8` | — | — | — | Cars in the race. |
| `time` | `number` | `0` | — | — | — | The race time in seconds. |
| `best` | `number \| null` | `null` | — | — | — | The best lap in seconds; null hides it. |
| `boost` | `number` | `0` | — | `ratio` | — | The boost meter. |
| `map` | `MinimapDesc \| null` | `null` | — | — | — | A mini-map; null is none. |
| `touch` | `boolean` | `false` | — | — | — | Show touch controls (steering arrows, Brake, Boost) on phones and tablets. |

```ogt
struct RacingHudDesc {
  doc "A racing HUD: position and lap top left, the times top right, the speedometer and gear bottom right, the boost meter, an optional mini-map."
  desc true
  field speed f32 "The speed, in `unit`."
  field maxSpeed f32 "The dial's top." {
    default 300
  }
  field unit String "The speed's unit." {
    default "km/h"
  }
  field gear String "The gear (`3`, `R`, `N`)."
  field lap u32 "The lap." {
    default 1
  }
  field laps u32 "Laps in the race." {
    default 3
  }
  field position u32 "The player's place." {
    default 1
  }
  field racers u32 "Cars in the race." {
    default 8
  }
  field time f32 "The race time in seconds."
  field best Option.f32 "The best lap in seconds; null hides it."
  field boost f32 "The boost meter." {
    unit ratio
  }
  field map Option.MinimapDesc "A mini-map; null is none."
  field touch bool "Show touch controls (steering arrows, Brake, Boost) on phones and tablets."
}
```

<sub>`api.ogt` line 9000</sub>

<a id="type-DialogueLine"></a>

### `DialogueLine` (desc struct)

One line of a cutscene.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speaker` | `string` | `""` | — | — | — | Who speaks. |
| `text` | `string` | `""` | — | — | — | What they say. |
| `portrait` | `string` | `""` | — | — | — | An icon of the atlas as the portrait; empty is the speaker's initial. |

```ogt
struct DialogueLine {
  doc "One line of a cutscene."
  desc true
  field speaker String "Who speaks."
  field text String "What they say."
  field portrait String "An icon of the atlas as the portrait; empty is the speaker's initial."
}
```

<sub>`api.ogt` line 9032</sub>

<a id="type-CutsceneDesc"></a>

### `CutsceneDesc` (desc struct)

A dialogue scene: letterbox bars, the dialogue box at the bottom, Skip at the top right.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it plays. |
| `lines` | `readonly DialogueLine[]` | `[]` | — | — | — | The lines. |
| `index` | `number` | `0` | — | — | — | The line shown, from game state. |
| `letterbox` | `boolean` | `true` | — | — | — | Cinematic bars. |
| `skippable` | `boolean` | `true` | — | — | — | Show Skip. |

```ogt
struct CutsceneDesc {
  doc "A dialogue scene: letterbox bars, the dialogue box at the bottom, Skip at the top right."
  desc true
  field open bool "Whether it plays."
  field lines List.DialogueLine "The lines."
  field index u32 "The line shown, from game state."
  field letterbox bool "Cinematic bars." {
    default true
  }
  field skippable bool "Show Skip." {
    default true
  }
}
```

<sub>`api.ogt` line 9040</sub>

<a id="type-CutsceneResult"></a>

### `CutsceneResult` (struct)

A cutscene's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | The line to show next tick: keep it. |
| `finished` | `boolean` | `false` | — | — | — | The last line was advanced past. |
| `skipped` | `boolean` | `false` | — | — | — | Skip was pressed (or Back). |

```ogt
struct CutsceneResult {
  doc "A cutscene's answer this tick."
  field index u32 "The line to show next tick: keep it."
  field finished bool "The last line was advanced past."
  field skipped bool "Skip was pressed (or Back)."
}
```

<sub>`api.ogt` line 9054</sub>

<a id="type-ShopItem"></a>

### `ShopItem` (desc struct)

An item a shop sells.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | Its id: buying answers it. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `price` | `number` | `0` | — | — | — | Its price in the shop's currency. |
| `description` | `string` | `""` | — | — | — | A line about it. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas; empty is none. |
| `category` | `string` | `""` | — | — | — | Its category tab; empty is the first. |
| `owned` | `boolean` | `false` | — | — | — | Already owned (one-off items): shown, not for sale. |

```ogt
struct ShopItem {
  doc "An item a shop sells."
  desc true
  field id String "Its id: buying answers it."
  field name String "Its name."
  field price f64 "Its price in the shop's currency."
  field description String "A line about it."
  field icon String "An icon of the atlas; empty is none."
  field category String "Its category tab; empty is the first."
  field owned bool "Already owned (one-off items): shown, not for sale."
}
```

<sub>`api.ogt` line 9061</sub>

<a id="type-ShopDesc"></a>

### `ShopDesc` (desc struct)

A shop: the balance, category tabs, a grid of items with prices, a detail panel with Buy (disabled when the balance is short).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Shop"` | — | — | — | Its title. |
| `currency` | `string` | `"coins"` | — | — | — | The currency's name (`coins`). |
| `currencyIcon` | `string` | `""` | — | — | — | The currency's icon; empty is none. |
| `balance` | `number` | `0` | — | — | — | What the player has. |
| `items` | `readonly ShopItem[]` | `[]` | — | — | — | What is for sale. |
| `categories` | `readonly string[]` | `[]` | — | — | — | Category tabs; empty is one list. |
| `onBuy` | `UiAction \| null` | `null` | — | — | — | Called with the bought item's id. |

```ogt
struct ShopDesc {
  doc "A shop: the balance, category tabs, a grid of items with prices, a detail panel with Buy (disabled when the balance is short)."
  desc true
  field title String "Its title." {
    default "Shop"
  }
  field currency String "The currency's name (`coins`)." {
    default "coins"
  }
  field currencyIcon String "The currency's icon; empty is none."
  field balance f64 "What the player has."
  field items List.ShopItem "What is for sale."
  field categories List.String "Category tabs; empty is one list."
  field onBuy Option.UiAction "Called with the bought item's id."
}
```

<sub>`api.ogt` line 9073</sub>

<a id="type-ShopResult"></a>

### `ShopResult` (struct)

A shop's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bought` | `string \| null` | `null` | — | — | — | The item bought this tick: take the price and grant it. |
| `refused` | `string \| null` | `null` | — | — | — | The item whose Buy was pressed this tick while the balance was short: the button shook with the error sound (og.ui.shake) and nothing was bought. |
| `closed` | `boolean` | `false` | — | — | — | Close was pressed (or Back). |

```ogt
struct ShopResult {
  doc "A shop's answer this tick."
  field bought Option.String "The item bought this tick: take the price and grant it."
  field refused Option.String "The item whose Buy was pressed this tick while the balance was short: the button shook with the error sound (og.ui.shake) and nothing was bought."
  field closed bool "Close was pressed (or Back)."
}
```

<sub>`api.ogt` line 9089</sub>

<a id="type-InventoryItem"></a>

### `InventoryItem` (desc struct)

What an inventory slot holds.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `slot` | `number` | `0` | — | — | — | The slot index. |
| `id` | `string` | `""` | — | — | — | The item's id. |
| `name` | `string` | `""` | — | — | — | Its name. |
| `count` | `number` | `0` | — | — | — | How many (stacks); 0 or 1 hides the count. |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas; empty is its initial. |
| `description` | `string` | `""` | — | — | — | A line about it. |

```ogt
struct InventoryItem {
  doc "What an inventory slot holds."
  desc true
  field slot u32 "The slot index."
  field id String "The item's id."
  field name String "Its name."
  field count u32 "How many (stacks); 0 or 1 hides the count."
  field icon String "An icon of the atlas; empty is its initial."
  field description String "A line about it."
}
```

<sub>`api.ogt` line 9096</sub>

<a id="type-InventoryDesc"></a>

### `InventoryDesc` (desc struct)

An inventory: a grid of slots, items dragged between them, a detail panel with Use.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Inventory"` | — | — | — | Its title. |
| `slots` | `number` | `24` | — | — | — | How many slots. |
| `columns` | `number` | `0` | — | — | — | Slots across; 0 fits the width. |
| `items` | `readonly InventoryItem[]` | `[]` | — | — | — | The items, by slot. |
| `useLabel` | `string` | `"Use"` | — | — | — | The detail panel's action; empty hides it. |

```ogt
struct InventoryDesc {
  doc "An inventory: a grid of slots, items dragged between them, a detail panel with Use."
  desc true
  field title String "Its title." {
    default "Inventory"
  }
  field slots u32 "How many slots." {
    default 24
  }
  field columns u32 "Slots across; 0 fits the width."
  field items List.InventoryItem "The items, by slot."
  field useLabel String "The detail panel's action; empty hides it." {
    default "Use"
  }
}
```

<sub>`api.ogt` line 9107</sub>

<a id="type-SlotMove"></a>

### `SlotMove` (struct)

An item dragged from one slot to another.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `number` | `0` | — | — | — | The slot it left. |
| `to` | `number` | `0` | — | — | — | The slot it was dropped on (swap when that slot holds an item). |

```ogt
struct SlotMove {
  doc "An item dragged from one slot to another."
  field from u32 "The slot it left."
  field to u32 "The slot it was dropped on (swap when that slot holds an item)."
}
```

<sub>`api.ogt` line 9123</sub>

<a id="type-InventoryResult"></a>

### `InventoryResult` (struct)

An inventory's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `number \| null` | `null` | — | — | — | The selected slot. |
| `moved` | `SlotMove \| null` | `null` | — | — | — | A drag and drop this tick: move (or swap) the items. |
| `used` | `string \| null` | `null` | — | — | — | The id of the item Use was pressed for. |
| `closed` | `boolean` | `false` | — | — | — | Close was pressed (or Back). |

```ogt
struct InventoryResult {
  doc "An inventory's answer this tick."
  field selected Option.u32 "The selected slot."
  field moved Option.SlotMove "A drag and drop this tick: move (or swap) the items."
  field used Option.String "The id of the item Use was pressed for."
  field closed bool "Close was pressed (or Back)."
}
```

<sub>`api.ogt` line 9129</sub>

<a id="type-LevelInfo"></a>

### `LevelInfo` (desc struct)

A level in a level select.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name or number. |
| `stars` | `number` | `0` | — | — | — | Stars earned, 0 to 3. |
| `locked` | `boolean` | `false` | — | — | — | Not yet reachable: a lock, inert. |
| `best` | `string` | `""` | — | — | — | The best result as shown; empty is none. |

```ogt
struct LevelInfo {
  doc "A level in a level select."
  desc true
  field name String "Its name or number."
  field stars u32 "Stars earned, 0 to 3."
  field locked bool "Not yet reachable: a lock, inert."
  field best String "The best result as shown; empty is none."
}
```

<sub>`api.ogt` line 9137</sub>

<a id="type-LevelSelectDesc"></a>

### `LevelSelectDesc` (desc struct)

A level select: a grid of level cards with stars and locks, the total stars.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Levels"` | — | — | — | Its title. |
| `levels` | `readonly LevelInfo[]` | `[]` | — | — | — | The levels. |
| `columns` | `number` | `0` | — | — | — | Cards across; 0 fits the width. |

```ogt
struct LevelSelectDesc {
  doc "A level select: a grid of level cards with stars and locks, the total stars."
  desc true
  field title String "Its title." {
    default "Levels"
  }
  field levels List.LevelInfo "The levels."
  field columns u32 "Cards across; 0 fits the width."
}
```

<sub>`api.ogt` line 9146</sub>

<a id="type-LevelSelectResult"></a>

### `LevelSelectResult` (struct)

A level select's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `number \| null` | `null` | — | — | — | The level picked this tick. |
| `closed` | `boolean` | `false` | — | — | — | Back was pressed. |

```ogt
struct LevelSelectResult {
  doc "A level select's answer this tick."
  field chosen Option.u32 "The level picked this tick."
  field closed bool "Back was pressed."
}
```

<sub>`api.ogt` line 9156</sub>

<a id="type-StatRow"></a>

### `StatRow` (desc struct)

A labelled value on a results screen.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | The label. |
| `value` | `string` | `""` | — | — | — | The value, as shown. |

```ogt
struct StatRow {
  doc "A labelled value on a results screen."
  desc true
  field label String "The label."
  field value String "The value, as shown."
}
```

<sub>`api.ogt` line 9162</sub>

<a id="type-ResultsDesc"></a>

### `ResultsDesc` (desc struct)

A results screen: the title, stars that land one by one, the score counting up, the best (a badge for a new best), stats, actions and a share hook.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is shown. |
| `title` | `string` | `"Level complete"` | — | — | — | Its title. |
| `score` | `number` | `0` | — | — | — | The score. |
| `best` | `number` | `0` | — | — | — | The best score. |
| `newBest` | `boolean` | `false` | — | — | — | This run set the best. |
| `stars` | `number` | `0` | — | — | — | Stars earned. |
| `maxStars` | `number` | `3` | — | — | — | Stars possible; 0 hides them. |
| `stats` | `readonly StatRow[]` | `[]` | — | — | — | Stats under the score. |
| `actions` | `readonly MenuItem[]` | `[]` | — | — | — | The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`). |
| `shareText` | `string` | `""` | — | — | — | Text a share would post; empty hides Share. |
| `onShare` | `UiAction \| null` | `null` | — | — | — | Called with the share text when Share is pressed (the shell posts it). |

```ogt
struct ResultsDesc {
  doc "A results screen: the title, stars that land one by one, the score counting up, the best (a badge for a new best), stats, actions and a share hook."
  desc true
  field open bool "Whether it is shown."
  field title String "Its title." {
    default "Level complete"
  }
  field score f64 "The score."
  field best f64 "The best score."
  field newBest bool "This run set the best."
  field stars u32 "Stars earned."
  field maxStars u32 "Stars possible; 0 hides them." {
    default 3
  }
  field stats List.StatRow "Stats under the score."
  field actions List.MenuItem "The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`)."
  field shareText String "Text a share would post; empty hides Share."
  field onShare Option.UiAction "Called with the share text when Share is pressed (the shell posts it)."
}
```

<sub>`api.ogt` line 9169</sub>

<a id="type-LeaderboardEntry"></a>

### `LeaderboardEntry` (desc struct)

A row of a leaderboard.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rank` | `number` | `0` | — | — | — | The rank. |
| `name` | `string` | `""` | — | — | — | The player's name. |
| `score` | `string` | `""` | — | — | — | The score, as shown. |
| `player` | `boolean` | `false` | — | — | — | This is the player: highlighted. |

```ogt
struct LeaderboardEntry {
  doc "A row of a leaderboard."
  desc true
  field rank u32 "The rank."
  field name String "The player's name."
  field score String "The score, as shown."
  field player bool "This is the player: highlighted."
}
```

<sub>`api.ogt` line 9189</sub>

<a id="type-LeaderboardDesc"></a>

### `LeaderboardDesc` (desc struct)

A leaderboard: scope tabs, a virtualised list of ranks (medal colours for 1 to 3, the player highlighted), and honest loading, error and empty states.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Leaderboard"` | — | — | — | Its title. |
| `scopes` | `readonly string[]` | `[]` | — | — | — | Scope tabs (Today, This week, All time); empty is none. |
| `entries` | `readonly LeaderboardEntry[]` | `[]` | — | — | — | The rows. |
| `loading` | `boolean` | `false` | — | — | — | The rows are loading: skeletons. |
| `error` | `string` | `""` | — | — | — | Loading failed: this message and Retry. |
| `empty` | `string` | `"No scores yet"` | — | — | — | Shown when there are no rows. |

```ogt
struct LeaderboardDesc {
  doc "A leaderboard: scope tabs, a virtualised list of ranks (medal colours for 1 to 3, the player highlighted), and honest loading, error and empty states."
  desc true
  field title String "Its title." {
    default "Leaderboard"
  }
  field scopes List.String "Scope tabs (Today, This week, All time); empty is none."
  field entries List.LeaderboardEntry "The rows."
  field loading bool "The rows are loading: skeletons."
  field error String "Loading failed: this message and Retry."
  field empty String "Shown when there are no rows." {
    default "No scores yet"
  }
}
```

<sub>`api.ogt` line 9198</sub>

<a id="type-LeaderboardResult"></a>

### `LeaderboardResult` (struct)

A leaderboard's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scope` | `number` | `0` | — | — | — | The scope tab chosen: keep it and load its rows. |
| `changed` | `boolean` | `false` | — | — | — | The scope changed this tick. |
| `retry` | `boolean` | `false` | — | — | — | Retry was pressed. |
| `closed` | `boolean` | `false` | — | — | — | Close was pressed (or Back). |

```ogt
struct LeaderboardResult {
  doc "A leaderboard's answer this tick."
  field scope u32 "The scope tab chosen: keep it and load its rows."
  field changed bool "The scope changed this tick."
  field retry bool "Retry was pressed."
  field closed bool "Close was pressed (or Back)."
}
```

<sub>`api.ogt` line 9213</sub>

<a id="type-TutorialStep"></a>

### `TutorialStep` (desc struct)

A step of a tutorial: the element to point at, a title and a line.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `string` | `""` | — | — | — | The element's id or path (as touch_move_to takes); empty points at nothing. |
| `title` | `string` | `""` | — | — | — | The step's title. |
| `text` | `string` | `""` | — | — | — | What to do. |

```ogt
struct TutorialStep {
  doc "A step of a tutorial: the element to point at, a title and a line."
  desc true
  field target String "The element's id or path (as touch_move_to takes); empty points at nothing."
  field title String "The step's title."
  field text String "What to do."
}
```

<sub>`api.ogt` line 9221</sub>

<a id="type-TutorialDesc"></a>

### `TutorialDesc` (desc struct)

An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is shown. |
| `steps` | `readonly TutorialStep[]` | `[]` | — | — | — | The steps. |
| `index` | `number` | `0` | — | — | — | The step shown, from game state. |

```ogt
struct TutorialDesc {
  doc "An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count."
  desc true
  field open bool "Whether it is shown."
  field steps List.TutorialStep "The steps."
  field index u32 "The step shown, from game state."
}
```

<sub>`api.ogt` line 9229</sub>

<a id="type-TutorialResult"></a>

### `TutorialResult` (struct)

A tutorial's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `index` | `number` | `0` | — | — | — | The step to show next tick: keep it. |
| `finished` | `boolean` | `false` | — | — | — | The last step's Done was pressed. |
| `skipped` | `boolean` | `false` | — | — | — | Skip was pressed (or Back). |

```ogt
struct TutorialResult {
  doc "A tutorial's answer this tick."
  field index u32 "The step to show next tick: keep it."
  field finished bool "The last step's Done was pressed."
  field skipped bool "Skip was pressed (or Back)."
}
```

<sub>`api.ogt` line 9237</sub>

<a id="type-LoadingDesc"></a>

### `LoadingDesc` (desc struct)

A loading screen: the title, a real progress bar (or a spinner while unknown) and tips that change by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Loading"` | — | — | — | What is loading. |
| `progress` | `number \| null` | `null` | — | — | — | How far, 0 to 1; null is unknown (a spinner, never a fake bar). |
| `tips` | `readonly string[]` | `[]` | — | — | — | Tips shown in turn. |
| `tipSeconds` | `number` | `4` | ≥ 1 | — | — | Seconds per tip. |

```ogt
struct LoadingDesc {
  doc "A loading screen: the title, a real progress bar (or a spinner while unknown) and tips that change by the tick."
  desc true
  field title String "What is loading." {
    default "Loading"
  }
  field progress Option.f32 "How far, 0 to 1; null is unknown (a spinner, never a fake bar)."
  field tips List.String "Tips shown in turn."
  field tipSeconds f32 "Seconds per tip." {
    default 4
    min 1
  }
}
```

<sub>`api.ogt` line 9244</sub>

<a id="type-ConfirmDesc"></a>

### `ConfirmDesc` (desc struct)

A confirm dialog: a title, a consequence sentence, Cancel and the action.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is open. |
| `title` | `string` | `""` | — | — | — | The question. |
| `message` | `string` | `""` | — | — | — | What will happen, as rich text markup (og.ui.richText: [b], [color=…], [icon=…]…), so a translation carries its own emphasis. |
| `confirmLabel` | `string` | `"Confirm"` | — | — | — | The action's label. |
| `cancelLabel` | `string` | `"Cancel"` | — | — | — | Cancel's label. |
| `danger` | `boolean` | `false` | — | — | — | The action destroys something: the danger button. |

```ogt
struct ConfirmDesc {
  doc "A confirm dialog: a title, a consequence sentence, Cancel and the action."
  desc true
  field open bool "Whether it is open."
  field title String "The question."
  field message String "What will happen, as rich text markup (og.ui.richText: [b], [color=…], [icon=…]…), so a translation carries its own emphasis."
  field confirmLabel String "The action's label." {
    default "Confirm"
  }
  field cancelLabel String "Cancel's label." {
    default "Cancel"
  }
  field danger bool "The action destroys something: the danger button."
}
```

<sub>`api.ogt` line 9258</sub>

<a id="type-ConfirmResult"></a>

### `ConfirmResult` (struct)

A confirm dialog's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `confirmed` | `boolean` | `false` | — | — | — | The action was chosen. |
| `cancelled` | `boolean` | `false` | — | — | — | Cancel, Back or the backdrop. |

```ogt
struct ConfirmResult {
  doc "A confirm dialog's answer this tick."
  field confirmed bool "The action was chosen."
  field cancelled bool "Cancel, Back or the backdrop."
}
```

<sub>`api.ogt` line 9273</sub>

<a id="type-AlertDesc"></a>

### `AlertDesc` (desc struct)

An alert: a title, a message and one button.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is open. |
| `title` | `string` | `""` | — | — | — | The title. |
| `message` | `string` | `""` | — | — | — | The message, as rich text markup (og.ui.richText). |
| `okLabel` | `string` | `"OK"` | — | — | — | The button's label. |

```ogt
struct AlertDesc {
  doc "An alert: a title, a message and one button."
  desc true
  field open bool "Whether it is open."
  field title String "The title."
  field message String "The message, as rich text markup (og.ui.richText)."
  field okLabel String "The button's label." {
    default "OK"
  }
}
```

<sub>`api.ogt` line 9279</sub>
