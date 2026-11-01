<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui` module

The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the placeholder prototype style until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.

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

<sub>`api.ogt` line 11438</sub>

## Contents

| Name | Kind | Summary |
|---|---|---|
| [`Direction`](#type-Direction) | enum | Which way a container lays out its children. |
| [`Align`](#type-Align) | enum | Where children sit along the main axis. |
| [`Cross`](#type-Cross) | enum | Where children sit across the main axis. |
| [`TextAlign`](#type-TextAlign) | enum | How the lines of a label align. |
| [`FontRole`](#type-FontRole) | enum | Which configured font text uses; display and mono fall back to body, rounded to display, then body. |
| [`ButtonVariant`](#type-ButtonVariant) | enum | Button roles: how the style draws each. |
| [`ControlSize`](#type-ControlSize) | enum | Control heights from the density (h-control-sm, -md, -lg). |
| [`Scheme`](#type-Scheme) | enum | The palette: games run on a stage, so dark is the default. |
| [`DensityMode`](#type-DensityMode) | enum | Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise. |
| [`ThemeColor`](#type-ThemeColor) | enum | A colour role of the UI style in effect (the game's style, or the prototype style before it picks one). |
| [`TypeStepName`](#type-TypeStepName) | enum | A step of the style's type ramp. |
| [`SizeRange`](#type-SizeRange) | struct | The bounds of a fit or grow size. |
| [`FixedSize`](#type-FixedSize) | struct | A fixed size. |
| [`PercentSize`](#type-PercentSize) | struct | A size relative to the parent. |
| [`Size`](#type-Size) | union | A size along one axis. Absent (null) is fit: the content's size. |
| [`Insets`](#type-Insets) | struct | Padding or margins on four sides. |
| [`Paint`](#type-Paint) | desc struct | A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. Neither is white. |
| [`ElementDesc`](#type-ElementDesc) | desc struct | A container: its layout and its look. |
| [`LabelDesc`](#type-LabelDesc) | desc struct | How a label draws its text. |
| [`ButtonDesc`](#type-ButtonDesc) | desc struct | A button's look. |
| [`PanelDesc`](#type-PanelDesc) | desc struct | A card: panel, the panel radius, padded and spaced by the style. |
| [`ScrollAxes`](#type-ScrollAxes) | enum | Which ways a scroll container scrolls. |
| [`ScrollDesc`](#type-ScrollDesc) | desc struct | A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); it clips its content; a scrollbar on each scrolling axis shows the position, and a press on one grabs and drags it. Nested containers hand off: what the innermost under the pointer cannot take (it reached its end, or does not scroll that way) goes to the one around it. |
| [`ImageDesc`](#type-ImageDesc) | desc struct | A texture region, or a solid rectangle without a texture. |
| [`ZoomViewDesc`](#type-ZoomViewDesc) | desc struct | A zoomable picture (a map, an artwork): what it shows and its zoom limits. |
| [`ZoomResult`](#type-ZoomResult) | struct | Where a zoom view is. |
| [`ProgressDesc`](#type-ProgressDesc) | desc struct | A progress bar. |
| [`SliderDesc`](#type-SliderDesc) | desc struct | A slider's range and width. |
| [`FieldDesc`](#type-FieldDesc) | desc struct | A one-line text field. |
| [`TextAreaDesc`](#type-TextAreaDesc) | desc struct | A multi-line text area. |
| [`Response`](#type-Response) | struct | What happened to an element this tick. Edges (clicked, changed, submitted) are true for exactly one tick. |
| [`SliderResult`](#type-SliderResult) | struct | A slider's response and its value after this tick's input. |
| [`ToggleResult`](#type-ToggleResult) | struct | A toggle's response and its state after this tick's input. |
| [`FieldResult`](#type-FieldResult) | struct | A text field's response and its text after this tick's input. |
| [`Viewport`](#type-Viewport) | struct | The play surface the UI lays out in. |
| [`UiDesc`](#type-UiDesc) | desc struct | How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it. |
| [`UiSounds`](#type-UiSounds) | desc struct | Sounds the UI plays through og-audio (the ui bus) on its events; null is silent. |
| [`StylePart`](#type-StylePart) | enum | An element class a style dresses with a nine-slice frame. |
| [`TypeStep`](#type-TypeStep) | struct | One step of the type ramp. |
| [`ControlSizes`](#type-ControlSizes) | struct | The control, row and header heights and the gutter of the density in effect. |
| [`UiTheme`](#type-UiTheme) | struct | The style values the UI draws with, read-only. |
| [`UiBuild`](#type-UiBuild) | callback | Builds a container's children: ui calls made inside it nest under the container. |
| [`og.ui.configure`](#fn-configure) | function | Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the prototype style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text. |
| [`og.ui.viewport`](#fn-viewport) | function | This tick's play surface: the render target's size in CSS pixels, its DPR and safe area, as the host reports them. |
| [`og.ui.theme`](#fn-theme) | function | The style values the UI draws with. |
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
| [`og.ui.richText`](#fn-richText) | function | A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning. |
| [`og.ui.label`](#fn-label) | function | MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. The probe reads its text. |
| [`og.ui.button`](#fn-button) | function | A button with a label; `clicked` is a press and a release inside it. The probe reads its label. |
| [`og.ui.image`](#fn-image) | function | A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other. |
| [`og.ui.zoomView`](#fn-zoomView) | function | A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value. |
| [`og.ui.progress`](#fn-progress) | function | A track filled to a fraction. |
| [`og.ui.slider`](#fn-slider) | function | A track with a fill and a thumb: a press anywhere on it sets the value and dragging follows. Pass back the value it answers next tick. |
| [`og.ui.toggle`](#fn-toggle) | function | A switch with a label; a click flips it. Pass back the state it answers next tick. The probe reads its label. |
| [`og.ui.textField`](#fn-textField) | function | One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty. |
| [`og.ui.textArea`](#fn-textArea) | function | Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty. |
| [`og.ui.tooltip`](#fn-tooltip) | function | A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. The probe reads its text. |
| [`og.ui.idi`](#fn-idi) | function | The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`. |
| [`Anchor`](#type-Anchor) | enum | A point of the safe area an anchored group sits at. |
| [`TransitionKind`](#type-TransitionKind) | enum | How a transition moves its content in and out. Under reduced motion every kind only fades. |
| [`Tone`](#type-Tone) | enum | A status tone of badges, toasts and map markers. |
| [`MeterKind`](#type-MeterKind) | enum | What a meter measures, which sets its colour. |
| [`Breakpoint`](#type-Breakpoint) | enum | The play surface's size class. |
| [`Contrast`](#type-Contrast) | enum | The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour). |
| [`NavIntent`](#type-NavIntent) | enum | A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give. |
| [`UiRect`](#type-UiRect) | struct | A rectangle of the play surface. |
| [`IconAtlasDesc`](#type-IconAtlasDesc) | desc struct | The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font). |
| [`AnchorDesc`](#type-AnchorDesc) | desc struct | An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls). |
| [`TransitionDesc`](#type-TransitionDesc) | desc struct | How a transition moves, timed in ticks from the style's motion. |
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
| [`RadioDesc`](#type-RadioDesc) | desc struct | How a radio group lays out, and its name. |
| [`CheckListResult`](#type-CheckListResult) | struct | A checkbox list's response and values after this tick's input. |
| [`ToastDesc`](#type-ToastDesc) | desc struct | A toast's tone and time. |
| [`ChipDesc`](#type-ChipDesc) | desc struct | A chip's state. |
| [`MeterDesc`](#type-MeterDesc) | desc struct | A meter: a track with a fill that animates its changes by the tick. |
| [`CooldownDesc`](#type-CooldownDesc) | desc struct | A cooldown ring's size and centre. |
| [`IconDesc`](#type-IconDesc) | desc struct | An icon's size, ink and name. |
| [`MapMarker`](#type-MapMarker) | desc struct | A marker on a mini-map. |
| [`MinimapDesc`](#type-MinimapDesc) | desc struct | A mini-map frame. |
| [`DialogueDesc`](#type-DialogueDesc) | desc struct | A dialogue box: a portrait slot, the speaker and the line typed out by the tick. |
| [`DialogueResult`](#type-DialogueResult) | struct | A dialogue box's answer this tick. |
| [`JoystickDesc`](#type-JoystickDesc) | desc struct | A virtual joystick. |
| [`JoystickResult`](#type-JoystickResult) | struct | A joystick's vector this tick. |
| [`DpadDesc`](#type-DpadDesc) | desc struct | A d-pad. |
| [`DpadResult`](#type-DpadResult) | struct | Which arms of a d-pad are held this tick. |
| [`ActionButtonDesc`](#type-ActionButtonDesc) | desc struct | A square touch action button (jump, fire), cornered by the style's control radius. |
| [`SkeletonDesc`](#type-SkeletonDesc) | desc struct | A skeleton: the neutral fill standing in for content that is loading. |
| [`UiItemBuild`](#type-UiItemBuild) | callback | Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it. |
| [`og.ui.breakpoint`](#fn-breakpoint) | function | The size class of the play surface of the frame being built (desktop outside a frame). |
| [`og.ui.frameViewport`](#fn-frameViewport) | function | The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI. |
| [`og.ui.hasIcon`](#fn-hasIcon) | function | Whether the icon atlas has an icon of this name (templates draw a fallback otherwise). |
| [`og.ui.seconds`](#fn-seconds) | function | The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock). |
| [`og.ui.frameTheme`](#fn-frameTheme) | function | The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates). |
| [`og.ui.openAnchor`](#fn-openAnchor) | function | Open an anchored group, closed by close(): og.ui.anchor without the callback. |
| [`og.ui.anchor`](#fn-anchor) | function | An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls. |
| [`og.ui.openTransition`](#fn-openTransition) | function | Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. Its content takes no input while it moves. |
| [`og.ui.transition`](#fn-transition) | function | Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible. |
| [`og.ui.openModal`](#fn-openModal) | function | Open a dialog, closed by closeModal(); build its content only while the answer is visible. The probe sees the backdrop as `<id>` and the panel as `<id>/Panel`. |
| [`og.ui.closeModal`](#fn-closeModal) | function | Close a dialog opened with openModal (its panel and backdrop). |
| [`og.ui.modal`](#fn-modal) | function | A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. Set your open flag false on `dismissed`. |
| [`og.ui.spotlight`](#fn-spotlight) | function | A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`. |
| [`og.ui.bounds`](#fn-bounds) | function | Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it. |
| [`og.ui.openList`](#fn-openList) | function | Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList(). |
| [`og.ui.openGrid`](#fn-openGrid) | function | Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList(). |
| [`og.ui.listItem`](#fn-listItem) | function | Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`. |
| [`og.ui.gridItem`](#fn-gridItem) | function | Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's. |
| [`og.ui.closeList`](#fn-closeList) | function | Close a list or grid opened with openList or openGrid. |
| [`og.ui.dragging`](#fn-dragging) | function | The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer. |
| [`og.ui.list`](#fn-list) | function | A virtualised list built by `build(index)` for the rows in view. |
| [`og.ui.grid`](#fn-grid) | function | A virtualised grid built by `build(index)` for the cells in view. |
| [`og.ui.tabs`](#fn-tabs) | function | A segmented control: a neutral track, the chosen tab on the thumb. Tabs are `Tab#i` and the probe reads their labels. |
| [`og.ui.select`](#fn-select) | function | A select: a field-like trigger with the chosen label and a chevron; a press or Activate opens its menu `<id>.menu` (options `Option#i`) under it, a focus scope; choosing closes it with changed. Back or a press outside closes it. |
| [`og.ui.stepper`](#fn-stepper) | function | A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too. |
| [`og.ui.radio`](#fn-radio) | function | A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label. |
| [`og.ui.checkbox`](#fn-checkbox) | function | A checkbox with a label; a press flips it. |
| [`og.ui.checkboxList`](#fn-checkboxList) | function | Checkboxes in a column (`Option#i`). |
| [`og.ui.chip`](#fn-chip) | function | A chip: a control-filled tag; selected inverts to ink. For filters and tags. |
| [`og.ui.badge`](#fn-badge) | function | A badge: a small status tag in its tone. |
| [`og.ui.divider`](#fn-divider) | function | A hairline in the border colour across its parent. |
| [`og.ui.spacer`](#fn-spacer) | function | Empty space. |
| [`og.ui.keyValue`](#fn-keyValue) | function | A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts. |
| [`og.ui.skeleton`](#fn-skeleton) | function | A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion). |
| [`og.ui.spinner`](#fn-spinner) | function | A loading spinner: eight dots around a circle, the bright one turning by the tick. |
| [`og.ui.icon`](#fn-icon) | function | An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name. |
| [`og.ui.meter`](#fn-meter) | function | A health, mana or xp bar. A drop shows at once and leaves a pale trail that follows after a beat; a rise grows into place with a ghost of the target; both by the tick. |
| [`og.ui.cooldown`](#fn-cooldown) | function | A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not). |
| [`og.ui.minimap`](#fn-minimap) | function | A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label. |
| [`og.ui.dialogue`](#fn-dialogue) | function | A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown. |
| [`og.ui.joystick`](#fn-joystick) | function | A virtual joystick: a base and a knob that follows a press that started on it. It keeps the finger that grabbed it (any of og.Pointers), so another finger can hold an action button at the same time. |
| [`og.ui.dpad`](#fn-dpad) | function | A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px. |
| [`og.ui.actionButton`](#fn-actionButton) | function | A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px. |
| [`og.ui.toast`](#fn-toast) | function | Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. Callable anywhere (systems, commands, start); it is world state, so snapshots carry it. |
| [`og.ui.dismissToast`](#fn-dismissToast) | function | Start a toast's exit. |
| [`og.ui.navigate`](#fn-navigate) | function | Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control. |
| [`og.ui.back`](#fn-back) | function | Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through. |
| [`og.ui.setPlayers`](#fn-setPlayers) | function | Local multiplayer: how many players this screen takes (1 to 4; 1 by default). Pads 1 to count - 1 each move a focus of their own among the same controls, shown as a ring in the player's colour (info, success, warn), and their A presses the control under it (Response.pad says which pad); pad 0 shares the one focus with the keyboard and the pointer, and Back and adjusting a slider stay pad 0's. Declare it every tick the screen shows; it takes effect from the next tick, and a dialog's focus scope holds every pad's focus inside it. |
| [`og.ui.focus`](#fn-focus) | function | The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none. |
| [`A11yNode`](#type-A11yNode) | struct | One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out). |
| [`A11yTree`](#type-A11yTree) | struct | The last frame's accessibility tree, built from the same nodes the UI draws and takes input on. |
| [`og.ui.a11y`](#fn-a11y) | function | The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. A shell mirrors it into the page for screen readers; a playtest reads the same fields on dump_ui's elements. |
| [`og.ui.feedback`](#fn-feedback) | function | Play a UI event's feedback from the style (docs/ui.md §7.8): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at the player's strength (setHaptics). Widgets play their own events (press, focus, select, confirm, back, open, close, tab, toggleOn, toggleOff, sliderTick); game code plays the rest: scrollTick, typewriter, counterTick, notification, warning, error, success, reward and the reveals revealCommon to revealMythic. The choices come from the UI state, so a replay plays the same. An unknown event or one the style leaves silent plays nothing. |
| [`og.ui.setHaptics`](#fn-setHaptics) | function | The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. The web shell rumbles the gamepad and vibrates a phone; the native host rumbles its pad. |
| [`og.ui.setNavigation`](#fn-setNavigation) | function | Whether keyboard and gamepad navigation moves focus on this screen (on by default). A gameplay HUD whose keys move the player turns it off; an open dialog or menu turns it back on while it is open. Takes effect from the next tick. |
| [`og.ui.memory`](#fn-memory) | function | A number the UI keeps for an id under the open element (a template's tab, a count-up's start tick), in snapshots like the rest of the UI state. Read it every tick it is used: an unused memory is dropped. |
| [`og.ui.remember`](#fn-remember) | function | Set a number memory (see memory). |
| [`og.ui.tick`](#fn-tick) | function | Frames the UI has built: its clock, for animations by the tick. |
| [`CounterDesc`](#type-CounterDesc) | desc struct | A number that counts to its value. |
| [`og.ui.counter`](#fn-counter) | function | A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards. |
| [`BurstDesc`](#type-BurstDesc) | desc struct | Rays and sparks thrown from an element. |
| [`og.ui.burst`](#fn-burst) | function | Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. Off under reduced motion. |
| [`ShakeDesc`](#type-ShakeDesc) | desc struct | A shake of an element: three sideways swings decaying to rest. |
| [`og.ui.shake`](#fn-shake) | function | Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set. |
| [`ScreenWipe`](#type-ScreenWipe) | enum | How a screen transition covers the old screen and reveals the new one. |
| [`WipeSide`](#type-WipeSide) | enum | Where a wipe or a slide comes from. |
| [`ScreenTransitionDesc`](#type-ScreenTransitionDesc) | desc struct | How a screen transition looks. |
| [`ScreenTransitionResult`](#type-ScreenTransitionResult) | struct | Where a screen transition is this tick. |
| [`og.ui.screenTransition`](#fn-screenTransition) | function | A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). Pass the screen the game wants every tick and build the one it answers: when the wanted screen changes the cover closes over the old screen for half the duration, the answer switches, and the cover opens on the new one. Deterministic by the tick; snapshots carry it; under reduced motion every kind is a fade. |
| [`og.ui.previewViewport`](#fn-previewViewport) | function | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. Render at width x dpr by height x dpr (`og run --size`). |
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
| [`PlatformerHudDesc`](#type-PlatformerHudDesc) | desc struct | A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button. What a game leaves out it does not draw: no hearts with maxLives 0 (and no health), no coins or score capsule without coins or score. |
| [`HudResult`](#type-HudResult) | struct | What a HUD answered this tick. |
| [`ShooterHudDesc`](#type-ShooterHudDesc) | desc struct | A shooter HUD: health and armor bottom left, ammo bottom right, abilities as cooldown rings, kills top right, a crosshair, an optional mini-map. |
| [`PuzzleHudDesc`](#type-PuzzleHudDesc) | desc struct | A puzzle HUD: level and moves, the score towards a target with three star marks, an optional timer. |
| [`Currency`](#type-Currency) | desc struct | A currency an idle game counts. |
| [`Generator`](#type-Generator) | desc struct | A generator an idle game lists: level, cost, progress. |
| [`IdleHudDesc`](#type-IdleHudDesc) | desc struct | An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list. |
| [`RaceSplit`](#type-RaceSplit) | desc struct | One split of a racing HUD: a sector's time and its delta against the reference (the record's split). |
| [`RacingHudDesc`](#type-RacingHudDesc) | desc struct | A racing HUD: position and lap top left, the times top right (the race time, a live delta, the best, the splits), the speedometer and gear bottom right, the boost meter, an optional mini-map. A time trial sets racers 0 (no place) and boostMeter false (no meter), and shows its sectors as splits. |
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
| [`LevelSelectDesc`](#type-LevelSelectDesc) | desc struct | A level select: a grid of level cards with stars and locks, the total stars, and a Back button. Back closes it from every input: the button, a press on the scrim around it, Escape and the pad's B. |
| [`LevelSelectResult`](#type-LevelSelectResult) | struct | A level select's answer this tick. |
| [`StatRow`](#type-StatRow) | desc struct | A labelled value on a results screen. |
| [`ScoreFormat`](#type-ScoreFormat) | enum | How a results screen shows its score and best. |
| [`ResultsDesc`](#type-ResultsDesc) | desc struct | A results screen: the title, stars that land one by one, the score counting up (points, or a time with `format` time) under its label, the best (a badge for a new best), stats, actions and a share hook. |
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
| [`og.ui.levelSelect`](#fn-levelSelect) | function | The level select template: a card picked answers `chosen`; its Back button, a press on the scrim around it, Escape and the pad's B answer `closed`. |
| [`og.ui.resultsScreen`](#fn-resultsScreen) | function | The results template: stars, the score (points or a time) counting up under its label, the best, stats, actions and Share. |
| [`og.ui.leaderboardScreen`](#fn-leaderboardScreen) | function | The leaderboard template. |
| [`og.ui.tutorialOverlay`](#fn-tutorialOverlay) | function | The onboarding template: a spotlight and a callout per step. |
| [`og.ui.loadingScreen`](#fn-loadingScreen) | function | The loading template. |
| [`og.ui.confirmDialog`](#fn-confirmDialog) | function | The confirm dialog template. |
| [`og.ui.alertDialog`](#fn-alertDialog) | function | The alert template. Returns true the tick it is dismissed. |
| [`UiProp`](#type-UiProp) | enum | An animatable value of an element (docs/ui.md §7.1): a state, a timeline key, a presence or og.ui.animate moves it. Colours are sRGB components 0 to 1; rotation and skew are radians. |
| [`LayoutMode`](#type-LayoutMode) | enum | How a container lays out its children. |
| [`Position`](#type-Position) | enum | How an element is placed. |
| [`GridFlow`](#type-GridFlow) | enum | Which way auto-placed items fill a grid. |
| [`DragAxis`](#type-DragAxis) | enum | Which way a drag moves its element. |
| [`EaseName`](#type-EaseName) | enum | A named easing curve. |
| [`MotionToken`](#type-MotionToken) | enum | A named motion of the style (docs/ui.md §7.7): components use these, so a style's personality changes every motion at once. |
| [`ReducedMotion`](#type-ReducedMotion) | enum | What an animation does under reduced motion (docs/ui.md §7.7). |
| [`Orientation`](#type-Orientation) | enum | A screen orientation. |
| [`Repeat`](#type-Repeat) | enum | How a play repeats. |
| [`UiTransform`](#type-UiTransform) | desc struct | A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. Hit tests, the probe's rectangles and the accessibility tree's bounds follow it; layout does not. |
| [`Edges`](#type-Edges) | desc struct | Insets of an absolute or anchored element from its parent's edges; an absent one is not set. start and end follow the writing direction: a right-to-left layout mirrors them. |
| [`TrackPx`](#type-TrackPx) | struct | A fixed track. |
| [`TrackFr`](#type-TrackFr) | struct | A share of the space the other tracks leave. |
| [`TrackRange`](#type-TrackRange) | struct | A track sized by its content, within bounds. |
| [`TrackPercent`](#type-TrackPercent) | struct | A fraction of the grid's inner size. |
| [`TrackAuto`](#type-TrackAuto) | struct | Its largest single-cell item. |
| [`Track`](#type-Track) | union | A grid track's size. |
| [`GridSpec`](#type-GridSpec) | desc struct | A grid's tracks and areas. |
| [`GridPlace`](#type-GridPlace) | desc struct | Where an item sits in its parent grid; the zero value is auto-placed in one cell. |
| [`PropValue`](#type-PropValue) | struct | A value for an animatable prop. |
| [`UiQuery`](#type-UiQuery) | desc struct | When a variant applies: every condition given must hold. |
| [`UiPatch`](#type-UiPatch) | desc struct | What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged. |
| [`UiVariant`](#type-UiVariant) | struct | A prop override under a condition (docs/ui.md §6.2: breakpoint variants and container queries). Later variants win. |
| [`SpringSpec`](#type-SpringSpec) | desc struct | A spring by how it feels: about how long it takes to settle and how much it overshoots. Retargeting it keeps its value and velocity, so an interrupted motion never jumps. |
| [`TweenSpec`](#type-TweenSpec) | desc struct | A duration on an ease: a retarget starts a new ease from where it is. |
| [`TokenRef`](#type-TokenRef) | struct | One of the style's motion tokens. |
| [`InstantSpec`](#type-InstantSpec) | struct | At once. |
| [`Transition`](#type-Transition) | union | How a value moves to a new target. |
| [`DismissSpec`](#type-DismissSpec) | desc struct | Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off. |
| [`SnapPoint`](#type-SnapPoint) | struct | An offset a released drag can settle on. |
| [`DragSpec`](#type-DragSpec) | desc struct | The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity. |
| [`PinchSpec`](#type-PinchSpec) | desc struct | Two touches on the element scale (and turn) it. |
| [`DropSpec`](#type-DropSpec) | desc struct | The element takes drops of dragged elements. |
| [`PresenceSpec`](#type-PresenceSpec) | desc struct | How an element arrives or leaves (docs/ui.md §7.5): the values it enters from or exits to, and how it moves between them and its own. |
| [`VisualState`](#type-VisualState) | struct | One state of a group: the values it sets, and timelines it starts. |
| [`StateTransition`](#type-StateTransition) | struct | The transition between two states of a group (`*` matches any state). The exact pair wins, then `* -> to`, `from -> *`, `* -> *`. |
| [`StateGroup`](#type-StateGroup) | struct | A visual state group (docs/ui.md §7.3): one of its states is current, and moving between them animates every value that changes. A group named interaction is derived from input: normal, hover, pressed, focused and disabled (a state the group lacks falls back: pressed to hover to normal, focused to hover). Other groups take their state from ElementProps.state, a timeline's state event or og.ui.forceState; the first state is the default. |
| [`StateValue`](#type-StateValue) | struct | The current state of a group. |
| [`ElementProps`](#type-ElementProps) | desc struct | Everything an element carries beyond its box (docs/ui.md §6.1): set with og.ui.props right after the element. The zero value is an element in the flow, unmoved and opaque, with no motion. |
| [`EaseKey`](#type-EaseKey) | struct | A named ease into a key. |
| [`HoldKey`](#type-HoldKey) | struct | Holds the previous value, then jumps at the key. |
| [`LinearKey`](#type-LinearKey) | struct | Constant speed into the key. |
| [`BezierKey`](#type-BezierKey) | struct | A cubic Bézier into the key. |
| [`KeyEase`](#type-KeyEase) | union | How the segment into a key moves. |
| [`TimelineKey`](#type-TimelineKey) | struct | A keyframe: a value at a time. |
| [`TimelineTrack`](#type-TimelineTrack) | struct | Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). Before its first key it holds that key's value, after its last the last. |
| [`SoundEvent`](#type-SoundEvent) | struct | A UI sound of the style. |
| [`HapticEvent`](#type-HapticEvent) | struct | A haptic pattern for the host. |
| [`BurstEvent`](#type-BurstEvent) | struct | Rays and sparks from the target. |
| [`StateEvent`](#type-StateEvent) | struct | Sets a state group of the target. |
| [`CallbackEvent`](#type-CallbackEvent) | struct | A callback to game code: it arrives in og.ui.motionEvents. |
| [`TimelineAction`](#type-TimelineAction) | union | What an event does when a play reaches it. |
| [`TimelineEvent`](#type-TimelineEvent) | struct | An event at a time, on the play's element or a descendant. |
| [`TimelineLabel`](#type-TimelineLabel) | struct | A named time, for seeking (PlayDesc.fromLabel). |
| [`UiTimeline`](#type-UiTimeline) | desc struct | Keyframed tracks and events (docs/ui.md §7.4). |
| [`ChoreoPlay`](#type-ChoreoPlay) | struct | A timeline on the play's element or a descendant. |
| [`ChoreoSteps`](#type-ChoreoSteps) | struct | Steps composed. |
| [`ChoreoStagger`](#type-ChoreoStagger) | struct | One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs. |
| [`ChoreoCascade`](#type-ChoreoCascade) | struct | Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher. |
| [`Choreo`](#type-Choreo) | union | Timelines composed into one play (docs/ui.md §7.4). |
| [`PlayDesc`](#type-PlayDesc) | desc struct | How a play runs. |
| [`PlayInfo`](#type-PlayInfo) | struct | Where a play is. |
| [`GestureInfo`](#type-GestureInfo) | struct | What a drag or pinch on an element is doing (last tick's answer, as responses are). |
| [`UiMotionEvent`](#type-UiMotionEvent) | struct | What happened in the UI's motion this tick. |
| [`UiOverlap`](#type-UiOverlap) | struct | Two elements overlapping where neither declared it (docs/ui.md §6.2). |
| [`UiLayoutStats`](#type-UiLayoutStats) | struct | What the last frame rebuilt and laid out (docs/ui.md §6.3). |
| [`og.ui.props`](#fn-props) | function | Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (docs/ui.md §6, §7). Call it right after the element. |
| [`og.ui.defineTimeline`](#fn-defineTimeline) | function | Define a timeline by name: configuration, like the theme, defined once at start. Plays and state groups name it. |
| [`og.ui.play`](#fn-play) | function | Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). Returns the play's id (0: no element, or a choreo of no known kind). |
| [`og.ui.playTimeline`](#fn-playTimeline) | function | Play one timeline (by name) on an element: og.ui.play with a single step. |
| [`og.ui.seek`](#fn-seek) | function | Move a play to a time; it fires no events on the way. Returns false for an unknown play. |
| [`og.ui.pause`](#fn-pause) | function | Hold a play where it is. |
| [`og.ui.resume`](#fn-resume) | function | Let a held play go on. |
| [`og.ui.reverse`](#fn-reverse) | function | Turn a play round where it is: an interrupted opening closes from where it got to. |
| [`og.ui.stop`](#fn-stop) | function | Stop a play: its elements go back to their own values (on their motion, if they have one). |
| [`og.ui.playInfo`](#fn-playInfo) | function | Where a play is; null once it is gone. |
| [`og.ui.forceState`](#fn-forceState) | function | Force a state group of an element (the probe and agents render and check each state this way); null lets it go. |
| [`og.ui.visualState`](#fn-visualState) | function | The state a group of an element is in (last tick); empty for none. |
| [`og.ui.animate`](#fn-animate) | function | Any number on a spring or an ease (docs/ui.md §7.1: every value is animatable): call it every tick with the target and it answers this tick's value. A retarget keeps its value and velocity; under reduced motion it jumps. |
| [`og.ui.gesture`](#fn-gesture) | function | What the drag or pinch on an element is doing. |
| [`og.ui.motionEvents`](#fn-motionEvents) | function | The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps. |
| [`og.ui.overlaps`](#fn-overlaps) | function | The last frame's undeclared overlaps between elements (docs/ui.md §6.2), with both paths. |
| [`og.ui.layoutStats`](#fn-layoutStats) | function | What the last frame built, rebuilt, laid out and kept. |
| [`VectorArt`](#type-VectorArt) | handle | A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill. |
| [`UiMaterial`](#type-UiMaterial) | handle | A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through. |
| [`VectorArt()`](#column-VectorArt) | column kind | A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill. |
| [`UiMaterial()`](#column-UiMaterial) | column kind | A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint. |
| [`ShapeType`](#type-ShapeType) | enum | An element's outline (DrawShape.kind). |
| [`CornerShape`](#type-CornerShape) | enum | How a rect's corners turn. |
| [`LineCap`](#type-LineCap) | enum | The end of an open line, an arc or a dash. |
| [`LineJoin`](#type-LineJoin) | enum | Where a line's segments meet. |
| [`DrawShape`](#type-DrawShape) | desc struct | An element's outline; its fills, strokes, effects and clip take it. |
| [`GradientType`](#type-GradientType) | enum | How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right). |
| [`GradientSpread`](#type-GradientSpread) | enum | What a gradient does past its ends. |
| [`GradientStop`](#type-GradientStop) | desc struct | A gradient's colour at an offset. |
| [`Gradient`](#type-Gradient) | desc struct | A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands. |
| [`ImageFit`](#type-ImageFit) | enum | How an image fills the element's box. |
| [`SliceEdges`](#type-SliceEdges) | enum | How a nine-slice's edges and centre fill their length. |
| [`SolidFill`](#type-SolidFill) | desc struct | A colour. |
| [`ImageFill`](#type-ImageFill) | desc struct | A texture region with a fit. |
| [`VectorFill`](#type-VectorFill) | desc struct | A cooked vector contained and centred in the box. |
| [`Fill`](#type-Fill) | union | One layer of an element's fill, or a stroke's paint. |
| [`StrokeAlign`](#type-StrokeAlign) | enum | Which side of the outline a stroke lies on. |
| [`Stroke`](#type-Stroke) | desc struct | A stroke along the element's outline with a paint of its own (a gradient border is one stroke). |
| [`ShadowEffect`](#type-ShadowEffect) | desc struct | A shadow: a drop shadow under the element, or an inner shadow cast inwards. |
| [`GlowEffect`](#type-GlowEffect) | desc struct | A glow: around the element, or along its inside. |
| [`BlurEffect`](#type-BlurEffect) | desc struct | The element and its subtree blurred (drawn offscreen). |
| [`BackdropEffect`](#type-BackdropEffect) | desc struct | What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass. |
| [`Effect`](#type-Effect) | union | An effect of the element. |
| [`ClipMode`](#type-ClipMode) | enum | How an element cuts its children. |
| [`MaskDesc`](#type-MaskDesc) | desc struct | An alpha mask over an element's subtree: a gradient's alpha, else an image's. |
| [`MaterialName`](#type-MaterialName) | enum | What a layer composites through: a built-in UI material, or a custom one. params is read per material. |
| [`LayerDesc`](#type-LayerDesc) | desc struct | The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel). |
| [`TextShadowDesc`](#type-TextShadowDesc) | desc struct | A shadow under an element's text. |
| [`TextOutlineDesc`](#type-TextOutlineDesc) | desc struct | An outline around an element's glyphs. |
| [`TextEffectsDesc`](#type-TextEffectsDesc) | desc struct | Effects of the text an element holds (its labels and rich text). |
| [`DrawDesc`](#type-DrawDesc) | desc struct | What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen through a material. Colours given as theme tokens follow the style; every value is data an agent can read and set. |
| [`BlendMode`](#type-BlendMode) | enum | How a paint mixes with what is under it. |
| [`og.ui.setDraw`](#fn-setDraw) | function | Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glows, glass, clipping to its shape, masks, blend modes and materials (docs/ui.md §5). Call it right after the element's open or control; immediate mode, so every tick it is drawn. |
| [`WorldTarget`](#type-WorldTarget) | desc struct | What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point. |
| [`AnchorScaleMode`](#type-AnchorScaleMode) | enum | How an anchored element's size follows its distance from the camera. |
| [`WorldOcclusion`](#type-WorldOcclusion) | enum | What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU. |
| [`Declutter`](#type-Declutter) | enum | How an anchored element that overlaps one of higher priority (or as high and nearer) resolves. |
| [`EdgeClampDesc`](#type-EdgeClampDesc) | desc struct | Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there. |
| [`WorldAnchorDesc`](#type-WorldAnchorDesc) | desc struct | An element anchored in the world (docs/ui.md §14): laid out at its own size and placed each tick on its target's point projected with the tick's camera. A nameplate, an interaction prompt, a speech bubble, a waypoint. |
| [`AnchorInfo`](#type-AnchorInfo) | struct | Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it. |
| [`og.ui.openWorldAnchor`](#fn-openWorldAnchor) | function | Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, decluttered and clamped as desc says. Drawn under the screen's own elements. og.ui.setDraw after it draws its box. For thousands of tags of one kind (overhead bars on an army) use mass elements instead. |
| [`WorldPanelDesc`](#type-WorldPanelDesc) | desc struct | A world panel (docs/ui.md §14): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad. |
| [`PanelInfo`](#type-PanelInfo) | struct | What the pointer does on a world panel this tick. |
| [`og.ui.openWorldPanel`](#fn-openWorldPanel) | function | Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. One render target per panel. |
| [`MassKind`](#type-MassKind) | enum | What a mass element template draws. |
| [`MassEase`](#type-MassEase) | enum | How a mass element's timeline moves into a key. |
| [`MassSpace`](#type-MassSpace) | enum | Where a mass element's position is. |
| [`ParticleShape`](#type-ParticleShape) | enum | A particle's shape. |
| [`MassKey`](#type-MassKey) | desc struct | A key of a mass element's timeline, evaluated in the shader. |
| [`MassTemplateDesc`](#type-MassTemplateDesc) | desc struct | A mass element template (docs/ui.md §14): declared once, then instanced thousands of times a frame with a few values each. No instance is laid out; its motion is the timeline, evaluated on the GPU. |
| [`MassInstance`](#type-MassInstance) | desc struct | One mass element: a few values; its template does the rest. |
| [`og.ui.defineMass`](#fn-defineMass) | function | Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle. |
| [`og.ui.spawnMass`](#fn-spawnMass) | function | Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). They live in the UI's state, so a snapshot or a replay carries them. |
| [`og.ui.pushMass`](#fn-pushMass) | function | Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values). |
| [`og.ui.massCount`](#fn-massCount) | function | The live spawned instances of a template. |
| [`EmitFrom`](#type-EmitFrom) | enum | Where on its element a particle starts. |
| [`ParticlesDesc`](#type-ParticlesDesc) | desc struct | UI particles from an element (docs/ui.md §5.5): confetti, sparks, coin streams, rarity bursts. Drawn over it on its layer, clipped by its clip, seeded so a replay throws the same. Nothing under reduced motion. |
| [`og.ui.particles`](#fn-particles) | function | Emit particles from the element built just before (a control, or a container just opened). Call every tick it emits; a burst throws on the ticks desc.fire is set. og.ui.burst is a preset of it. |
| [`MetaKind`](#type-MetaKind) | enum | What a meta layer draws over the scene. |
| [`MetaDesc`](#type-MetaDesc) | desc struct | A meta layer (docs/ui.md §14): full screen, over the scene and under the screen's elements, never taking input. |
| [`og.ui.meta`](#fn-meta) | function | A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame. |
| [`SceneViewportDesc`](#type-SceneViewportDesc) | desc struct | The viewport element (docs/ui.md §5.5): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). Draw it with an og.CameraView into a render target on its own layers, with its own lights; this element shows the target and passes input in. |
| [`SceneViewportInfo`](#type-SceneViewportInfo) | struct | What the pointer does in a viewport this tick. |
| [`og.ui.sceneViewport`](#fn-sceneViewport) | function | A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate. |
| [`UiDocEvent`](#type-UiDocEvent) | struct | Something a document did (docs/ui.md §10.2): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back is `emit`. |
| [`UiDocResponse`](#type-UiDocResponse) | struct | What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes. |
| [`UiLayer`](#type-UiLayer) | struct | A layer of the stack (docs/ui.md §8.1), as its screen declares it. |
| [`UiActionTrigger`](#type-UiActionTrigger) | enum | When a UI action fires. |
| [`UiActionDesc`](#type-UiActionDesc) | struct | A named UI action (docs/ui.md §8.2) bound to og-input's controls: a component asks for an action, not a key. |
| [`UiDevice`](#type-UiDevice) | enum | The device a glyph set is for: the one the player used last. |
| [`UiGlyph`](#type-UiGlyph) | struct | An action's control on the device used last. |
| [`UiActionHint`](#type-UiActionHint) | struct | One entry of the action bar: an action the top layer accepts, its hint and glyph. |
| [`og.ui.screen`](#fn-screen) | function | Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots after this call (og.ui.openSlot). Its bindings read the viewmodel every tick, a Capitalised root (`Wallet.gold`) the resource as of the last tick; two-way controls write into it (each write is a `set` event). Layers code does not mount are built at the frame's end with the viewmodel they have. |
| [`og.ui.component`](#fn-component) | function | Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props. |
| [`og.ui.openSlot`](#fn-openSlot) | function | Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. False when no such slot was built this tick. |
| [`og.ui.push`](#fn-push) | function | Push a screen onto the layer stack (docs/ui.md §8.1); the layer under it remembers its focus. Callable anywhere (systems, commands, start); the stack is world state, so snapshots carry it. |
| [`og.ui.pop`](#fn-pop) | function | Pop the top layer; the one under it takes its remembered focus. Answers the popped screen's name (empty: the stack was empty). |
| [`og.ui.replace`](#fn-replace) | function | Replace the top layer with a screen. |
| [`og.ui.clear`](#fn-clear) | function | Empty the layer stack. |
| [`og.ui.layers`](#fn-layers) | function | The layer stack, bottom first. |
| [`og.ui.screenVm`](#fn-screenVm) | function | A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack. |
| [`og.ui.docEvents`](#fn-docEvents) | function | Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own. |
| [`og.ui.gameInput`](#fn-gameInput) | function | Gameplay takes input this tick: no layer whose input is `menu` is on the stack. Input goes to the top layer that takes it, once per tick. |
| [`og.ui.paused`](#fn-paused) | function | A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had. |
| [`og.ui.defineAction`](#fn-defineAction) | function | Define (or redefine) a UI action. confirm (Enter, Space, A), back (Escape, B), tabNext (E, ], RB), tabPrev (Q, [, LB) and details (X, Y) are defined by default. |
| [`og.ui.action`](#fn-action) | function | The action fired this tick (a document's `on action` fires on the top layer that takes input). |
| [`og.ui.actionHeld`](#fn-actionHeld) | function | A hold action is held past its hold, or its toggle is on. |
| [`og.ui.glyph`](#fn-glyph) | function | An action's glyph on the device used last: its first binding there. Null for an unknown action or one with no binding on that device. |
| [`og.ui.device`](#fn-device) | function | The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer. |
| [`og.ui.setDevice`](#fn-setDevice) | function | Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on. |
| [`og.ui.actionBar`](#fn-actionBar) | function | What an action bar component reads: the actions the top layer's document lists (`actions { confirm "Buy" }`), each with its hint and its glyph on the device used last. |
| [`og.ui.documentSchema`](#fn-documentSchema) | function | The reflected schema documents are validated against (docs/ui.md §10): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back. |
| [`og.ui.setProp`](#fn-setProp) | function | Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. The value is written as the document's line would be (`token primary`, `grow`, `bind gold compact`). |
| [`og.ui.saveEdits`](#fn-saveEdits) | function | Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes; the documents take the new text at once. Answers the files written. |

## Functions

<a id="fn-configure"></a>

### `og.ui.configure`

Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the prototype style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text.

```ts
og.ui.configure(world: World, desc?: UiDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world whose UI is configured. |
| `desc` | `UiDesc` | yes | The fields to change; the others keep their values. |

**Returns** `Result<void>`: Refused for a dead font or style, or a desc that is not an object.

Binding symbol `og_ui_configure` · publish builds: kept

```ogt
fn configure {
  doc "Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. The desc is a partial: a field left out keeps the last UiDesc's value, the one an earlier call or a look's style (og.render.applyLook) gave it, so a settings screen that configures sounds or reducedMotion keeps the style and the fonts; null clears a field to its default (`style: null` goes back to the prototype style). A UI left with no font at all (no font, no style with fonts, no fallback or locale font) warns: it would draw no text."
  param world World "The world whose UI is configured."
  param desc UiDesc "The fields to change; the others keep their values."
  returns Result.Void "Refused for a dead font or style, or a desc that is not an object."
}
```

<sub>`api.ogt` line 11958</sub>

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

<sub>`api.ogt` line 11965</sub>

<a id="fn-theme"></a>

### `og.ui.theme`

The style values the UI draws with.

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
  doc "The style values the UI draws with."
  param world World "The world."
  returns UiTheme
}
```

<sub>`api.ogt` line 11971</sub>

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

<sub>`api.ogt` line 11977</sub>

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

<sub>`api.ogt` line 11984</sub>

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

<sub>`api.ogt` line 11991</sub>

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

<sub>`api.ogt` line 11998</sub>

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

<sub>`api.ogt` line 12003</sub>

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

<sub>`api.ogt` line 12012</sub>

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

<sub>`api.ogt` line 12021</sub>

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

<sub>`api.ogt` line 12030</sub>

<a id="fn-richText"></a>

### `og.ui.richText`

A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning.

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
  doc "A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a second copy one device pixel right), `[color=danger]` or `[color=#ff8800]` (a ThemeColor follows the style), `[size=xl]` (a TypeStepName), `[font=mono]`, `[icon=coin]` (a cell of the icon atlas, one em), `[link=shop]Open the shop[/link]` (accent ink, underlined, focusable by keys and pads; a child element `<id>/shop` of kind link for the probe) and per-glyph motion `[wave]`, `[shake]`, `[pulse]` (by the tick; still under reduced motion). `[[` is a literal bracket. Lines break between the spans' words (a word restyled mid-way stays whole), each as tall as its tallest span on one baseline. Markup it cannot read shows as text and logs a warning."
  param id String "Its id."
  param markup String "The markup."
  param desc RichTextDesc "Defaults for its spans: type step, ink, alignment, wrapping, width and font."
  returns RichTextResult
}
```

<sub>`api.ogt` line 12057</sub>

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

<sub>`api.ogt` line 12065</sub>

<a id="fn-button"></a>

### `og.ui.button`

A button with a label; `clicked` is a press and a release inside it. The probe reads its label.

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
  doc "A button with a label; `clicked` is a press and a release inside it. The probe reads its label."
  param id String "Its id."
  param text String "The label."
  param desc ButtonDesc "Variant, size, disabled and width."
  returns Response
}
```

<sub>`api.ogt` line 12073</sub>

<a id="fn-image"></a>

### `og.ui.image`

A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other.

```ts
og.ui.image(id: string, desc?: ImageDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ImageDesc` | yes | Texture, region, tint, size and aspect ratio. |

**Returns** `void`.

Binding symbol `og_ui_image` · publish builds: kept

```ogt
fn image {
  doc "A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other."
  param id String "Its id."
  param desc ImageDesc "Texture, region, tint, size and aspect ratio."
  returns Void
}
```

<sub>`api.ogt` line 12081</sub>

<a id="fn-zoomView"></a>

### `og.ui.zoomView`

A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value.

```ts
og.ui.zoomView(id: string, desc?: ZoomViewDesc): ZoomResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `ZoomViewDesc` | yes | The picture, its size and zoom limits. |

**Returns** `ZoomResult`.

Binding symbol `og_ui_zoom_view` · publish builds: kept

```ogt
fn zoomView {
  doc "A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it, and then + (= and the keypad's) and - zoom it about its centre, 0 fits it again and the arrows pan it a tenth of its size. It stays on its picture and within its zoom limits; its zoom and centre are UI state (snapshots carry them). Two fingers over a scroll container scroll it the same way, and fling it as they lift. The probe reads its zoom as its value."
  param id String "Its id."
  param desc ZoomViewDesc "The picture, its size and zoom limits."
  returns ZoomResult
}
```

<sub>`api.ogt` line 12088</sub>

<a id="fn-progress"></a>

### `og.ui.progress`

A track filled to a fraction.

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
  doc "A track filled to a fraction."
  param id String "Its id."
  param value f32 "How full it is, from 0 to 1."
  param desc ProgressDesc "Its width."
  returns Void
}
```

<sub>`api.ogt` line 12095</sub>

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

<sub>`api.ogt` line 12103</sub>

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

<sub>`api.ogt` line 12111</sub>

<a id="fn-textField"></a>

### `og.ui.textField`

One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

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
  doc "One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text (og.input's TextInput events: the page's typing, the probe's type: and commit:; until a world has had one, a printable key pressed types its US-layout character, and after it keys only edit) replaces the selection at the caret; Backspace, Delete, arrows, Home and End edit, Shift with them selects, Alt or Ctrl with them moves or deletes by words (Ctrl with Home and End: the text's ends); Ctrl or Cmd with A selects all, with C copies and with X cuts (the page's clipboard, through the shell), with Z undoes and with Shift+Z or Y redoes (a history of 100 steps per field, typing undone a word at a time; a text the game sets starts it again); an input method's composition shows underlined at the caret until it commits; Enter submits and leaves, Escape leaves. The text scrolls to keep the caret in view and the caret blinks by the tick (steady under reduced motion). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc FieldDesc "Placeholder, width and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 12119</sub>

<a id="fn-textArea"></a>

### `og.ui.textArea`

Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty.

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
  doc "Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, the text wrapped between words at the area's width and scrolled to keep the caret's line in view (a drag above or below the area scrolls it, faster the further out). Pass back the text it answers next tick. The probe reads its text, or the placeholder while empty."
  param id String "Its id."
  param value String "The text from game state."
  param desc TextAreaDesc "Placeholder, width, rows and length limit."
  returns FieldResult
}
```

<sub>`api.ogt` line 12127</sub>

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

<sub>`api.ogt` line 12135</sub>

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

<sub>`api.ogt` line 12141</sub>

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

<sub>`api.ogt` line 12606</sub>

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

<sub>`api.ogt` line 12611</sub>

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

<sub>`api.ogt` line 12616</sub>

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

<sub>`api.ogt` line 12622</sub>

<a id="fn-frameTheme"></a>

### `og.ui.frameTheme`

The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates).

```ts
og.ui.frameTheme(): UiTheme
```

**Returns** `UiTheme`.

Binding symbol `og_ui_frame_theme` · publish builds: kept

```ogt
fn frameTheme {
  doc "The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the templates)."
  returns UiTheme
}
```

<sub>`api.ogt` line 12627</sub>

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

<sub>`api.ogt` line 12632</sub>

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

<sub>`api.ogt` line 12639</sub>

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

<sub>`api.ogt` line 12648</sub>

<a id="fn-transition"></a>

### `og.ui.transition`

Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible.

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
  doc "Content that enters and leaves: fade, slide or scale by the tick from the style's motion. `build` runs while it is visible."
  host false
  param id String "Its id."
  param show bool "Whether it should be shown."
  param desc TransitionDesc "How it moves."
  param build UiBuild "Makes its content."
  returns Presence
}
```

<sub>`api.ogt` line 12656</sub>

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

<sub>`api.ogt` line 12666</sub>

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

<sub>`api.ogt` line 12674</sub>

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

<sub>`api.ogt` line 12679</sub>

<a id="fn-spotlight"></a>

### `og.ui.spotlight`

A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`.

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
  doc "A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. Presses on the dimmed parts stop there (clicked); presses in the hole reach what is under it. Shades are `<id>/Top`, `Left`, `Right`, `Bottom`; the hole is `<id>/Hole`."
  param id String "Its id."
  param hole Option.UiRect "The rectangle to leave lit, as og.ui.bounds answers; null dims everything."
  param desc SpotlightDesc "Room around the hole."
  returns Response
}
```

<sub>`api.ogt` line 12689</sub>

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

<sub>`api.ogt` line 12697</sub>

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

<sub>`api.ogt` line 12703</sub>

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

<sub>`api.ogt` line 12710</sub>

<a id="fn-listItem"></a>

### `og.ui.listItem`

Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`.

```ts
og.ui.listItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (controlStrong). |

**Returns** `Response`.

Binding symbol `og_ui_list_item` · publish builds: kept

```ogt
fn listItem {
  doc "Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). A press that started on one row and is released on this one answers `dropped`."
  param index u32 "Its index."
  param selected bool "Drawn selected (controlStrong)."
  returns Response
}
```

<sub>`api.ogt` line 12717</sub>

<a id="fn-gridItem"></a>

### `og.ui.gridItem`

Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's.

```ts
og.ui.gridItem(index: number, selected: boolean): Response
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `index` | `number` | no | Its index. |
| `selected` | `boolean` | no | Drawn selected (controlStrong with a primary bar). |

**Returns** `Response`.

Binding symbol `og_ui_grid_item` · publish builds: kept

```ogt
fn gridItem {
  doc "Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). Drops answer `dropped` as listItem's."
  param index u32 "Its index."
  param selected bool "Drawn selected (controlStrong with a primary bar)."
  returns Response
}
```

<sub>`api.ogt` line 12724</sub>

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

<sub>`api.ogt` line 12731</sub>

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

<sub>`api.ogt` line 12736</sub>

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

<sub>`api.ogt` line 12742</sub>

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

<sub>`api.ogt` line 12751</sub>

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

<sub>`api.ogt` line 12760</sub>

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

<sub>`api.ogt` line 12769</sub>

<a id="fn-stepper"></a>

### `og.ui.stepper`

A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too.

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
  doc "A number stepper: `Dec`, `Value`, `Inc`. The buttons step within the range; Left and Right step it while a button is focused; a screen reader's increment and decrement on the stepper itself (its spinbutton, which the web shell mirrors as an adjustable range) step it too."
  param id String "Its id."
  param value f32 "The value from game state."
  param desc StepperDesc "Range, step and digits."
  returns StepperResult
}
```

<sub>`api.ogt` line 12778</sub>

<a id="fn-radio"></a>

### `og.ui.radio`

A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label.

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
  doc "A radio group of `Option#i` rows: a ring, filled primary when chosen, and a label."
  param id String "Its id."
  param options List.String "The option labels."
  param selected u32 "The chosen index from game state."
  param desc RadioDesc "Row or column."
  returns ChoiceResult
}
```

<sub>`api.ogt` line 12786</sub>

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

<sub>`api.ogt` line 12795</sub>

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

<sub>`api.ogt` line 12803</sub>

<a id="fn-chip"></a>

### `og.ui.chip`

A chip: a control-filled tag; selected inverts to ink. For filters and tags.

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
  doc "A chip: a control-filled tag; selected inverts to ink. For filters and tags."
  param id String "Its id."
  param text String "The label."
  param desc ChipDesc "Selected and disabled."
  returns Response
}
```

<sub>`api.ogt` line 12811</sub>

<a id="fn-badge"></a>

### `og.ui.badge`

A badge: a small status tag in its tone.

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
  doc "A badge: a small status tag in its tone."
  param id String "Its id."
  param text String "The label (a count, a word)."
  param tone Tone "Its tone."
  returns Void
}
```

<sub>`api.ogt` line 12819</sub>

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

<sub>`api.ogt` line 12827</sub>

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

<sub>`api.ogt` line 12834</sub>

<a id="fn-keyValue"></a>

### `og.ui.keyValue`

A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts.

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
  doc "A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts."
  param id String "Its id."
  param key String "The key."
  param value String "The value."
  param mono bool "The value in the monospaced font (prices, ids)."
  returns Void
}
```

<sub>`api.ogt` line 12841</sub>

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

<sub>`api.ogt` line 12850</sub>

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

<sub>`api.ogt` line 12857</sub>

<a id="fn-icon"></a>

### `og.ui.icon`

An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name.

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
  doc "An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. Rich text takes the same icons inline with `[icon=name]`. The probe reads the name."
  param id String "Its id."
  param name String "The icon's name."
  param desc IconDesc "Size and ink."
  returns Void
}
```

<sub>`api.ogt` line 12864</sub>

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

<sub>`api.ogt` line 12872</sub>

<a id="fn-cooldown"></a>

### `og.ui.cooldown`

A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not).

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
  doc "A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. A press reports clicked (ready or not)."
  param id String "Its id."
  param remaining f32 "How much of the cooldown is left, 0 to 1: 1 just used, 0 ready."
  param desc CooldownDesc "Size, label and icon."
  returns Response
}
```

<sub>`api.ogt` line 12881</sub>

<a id="fn-minimap"></a>

### `og.ui.minimap`

A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label.

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
  doc "A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label."
  param id String "Its id."
  param desc MinimapDesc "Map, markers and label."
  returns Void
}
```

<sub>`api.ogt` line 12889</sub>

<a id="fn-dialogue"></a>

### `og.ui.dialogue`

A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown.

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
  doc "A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. A press or Activate while typing shows the whole line; once shown it answers advanced. The line is `<id>/Body/Line` and the probe reads what is shown."
  param id String "Its id."
  param desc DialogueDesc "Speaker, text, portrait and speed."
  returns DialogueResult
}
```

<sub>`api.ogt` line 12896</sub>

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

<sub>`api.ogt` line 12903</sub>

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

<sub>`api.ogt` line 12910</sub>

<a id="fn-actionButton"></a>

### `og.ui.actionButton`

A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px.

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
  doc "A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px."
  param id String "Its id."
  param text String "Its label (and what the probe reads)."
  param desc ActionButtonDesc "Size, primary and icon."
  returns Response
}
```

<sub>`api.ogt` line 12917</sub>

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

<sub>`api.ogt` line 12925</sub>

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

<sub>`api.ogt` line 12933</sub>

<a id="fn-navigate"></a>

### `og.ui.navigate`

Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control.

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
  doc "Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. A move queued while the focus is on nothing puts the focus on the first control, whatever its direction, and no further: moves asked again before the focus lands (each tick until og.ui.focus answers a path) land on that same first control."
  param world World "The world."
  param intent NavIntent "The intent."
  returns Void
}
```

<sub>`api.ogt` line 12940</sub>

<a id="fn-back"></a>

### `og.ui.back`

Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through.

```ts
og.ui.back(): boolean
```

**Returns** `boolean`.

Binding symbol `og_ui_back` · publish builds: kept

```ogt
fn back {
  doc "Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. A pause screen closes on it. While a dialog or menu that Back closes is open (the topmost of the last frame) it answers false and leaves Back to it, even when asked before the dialog is built this tick: a HUD drawn under pause never takes the Back that closes pause. A locked dialog lets Back through."
  returns bool
}
```

<sub>`api.ogt` line 12947</sub>

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

<sub>`api.ogt` line 12952</sub>

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

<sub>`api.ogt` line 12958</sub>

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

<sub>`api.ogt` line 12997</sub>

<a id="fn-feedback"></a>

### `og.ui.feedback`

Play a UI event's feedback from the style (docs/ui.md §7.8): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at the player's strength (setHaptics). Widgets play their own events (press, focus, select, confirm, back, open, close, tab, toggleOn, toggleOff, sliderTick); game code plays the rest: scrollTick, typewriter, counterTick, notification, warning, error, success, reward and the reveals revealCommon to revealMythic. The choices come from the UI state, so a replay plays the same. An unknown event or one the style leaves silent plays nothing.

```ts
og.ui.feedback(event: string, rise: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `event` | `string` | no | The event's name. |
| `rise` | `number` | no | A rising cascade's step, 0 to 1, walking the set's pitch range up (a counter's ticks); below 0 picks the pitch at random. |

**Returns** `void`.

Binding symbol `og_ui_feedback` · publish builds: kept

```ogt
fn feedback {
  doc "Play a UI event's feedback from the style (docs/ui.md §7.8): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at the player's strength (setHaptics). Widgets play their own events (press, focus, select, confirm, back, open, close, tab, toggleOn, toggleOff, sliderTick); game code plays the rest: scrollTick, typewriter, counterTick, notification, warning, error, success, reward and the reveals revealCommon to revealMythic. The choices come from the UI state, so a replay plays the same. An unknown event or one the style leaves silent plays nothing."
  param event String "The event's name."
  param rise f32 "A rising cascade's step, 0 to 1, walking the set's pitch range up (a counter's ticks); below 0 picks the pitch at random."
  returns Void
}
```

<sub>`api.ogt` line 13002</sub>

<a id="fn-setHaptics"></a>

### `og.ui.setHaptics`

The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. The web shell rumbles the gamepad and vibrates a phone; the native host rumbles its pad.

```ts
og.ui.setHaptics(intensity: number): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `intensity` | `number` | no | 0 to 1 (clamped). |

**Returns** `void`.

Binding symbol `og_ui_set_haptics` · publish builds: kept

```ogt
fn setHaptics {
  doc "The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. The web shell rumbles the gamepad and vibrates a phone; the native host rumbles its pad."
  param intensity f32 "0 to 1 (clamped)."
  returns Void
}
```

<sub>`api.ogt` line 13009</sub>

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

<sub>`api.ogt` line 13015</sub>

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

<sub>`api.ogt` line 13021</sub>

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

<sub>`api.ogt` line 13028</sub>

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

<sub>`api.ogt` line 13035</sub>

<a id="fn-counter"></a>

### `og.ui.counter`

A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards.

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
  doc "A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. Scores, currency, rewards."
  param id String "Its id."
  param value f64 "The value to count to."
  param desc CounterDesc "Look, duration, start and format."
  returns f64 "The number shown this tick."
}
```

<sub>`api.ogt` line 13058</sub>

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

<sub>`api.ogt` line 13085</sub>

<a id="fn-shake"></a>

### `og.ui.shake`

Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set.

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
  doc "Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snapshots carry it. Still under reduced motion. A refused press on a disabled button already shakes when the style's motion.shake is set."
  param desc ShakeDesc "Amount, duration and sound."
  returns Void
}
```

<sub>`api.ogt` line 13105</sub>

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

<sub>`api.ogt` line 13151</sub>

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

<sub>`api.ogt` line 13159</sub>

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

<sub>`api.ogt` line 13880</sub>

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

<sub>`api.ogt` line 13888</sub>

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

<sub>`api.ogt` line 13896</sub>

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

<sub>`api.ogt` line 13904</sub>

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

<sub>`api.ogt` line 13912</sub>

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

<sub>`api.ogt` line 13920</sub>

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

<sub>`api.ogt` line 13928</sub>

<a id="fn-racingHud"></a>

### `og.ui.racingHud`

The racing HUD template.

```ts
og.ui.racingHud(id: string, desc?: RacingHudDesc): HudResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `RacingHudDesc` | yes | Speed, gear, laps, place, times, splits, boost and its words. |

**Returns** `HudResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn racingHud {
  doc "The racing HUD template."
  host false
  param id String "Its id."
  param desc RacingHudDesc "Speed, gear, laps, place, times, splits, boost and its words."
  returns HudResult
}
```

<sub>`api.ogt` line 13936</sub>

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

<sub>`api.ogt` line 13944</sub>

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

<sub>`api.ogt` line 13952</sub>

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

<sub>`api.ogt` line 13960</sub>

<a id="fn-levelSelect"></a>

### `og.ui.levelSelect`

The level select template: a card picked answers `chosen`; its Back button, a press on the scrim around it, Escape and the pad's B answer `closed`.

```ts
og.ui.levelSelect(id: string, desc?: LevelSelectDesc): LevelSelectResult
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `LevelSelectDesc` | yes | The levels and the screen's words. |

**Returns** `LevelSelectResult`.

Implemented by the TypeScript runtime over the members of this module: no binding symbol.

```ogt
fn levelSelect {
  doc "The level select template: a card picked answers `chosen`; its Back button, a press on the scrim around it, Escape and the pad's B answer `closed`."
  host false
  param id String "Its id."
  param desc LevelSelectDesc "The levels and the screen's words."
  returns LevelSelectResult
}
```

<sub>`api.ogt` line 13968</sub>

<a id="fn-resultsScreen"></a>

### `og.ui.resultsScreen`

The results template: stars, the score (points or a time) counting up under its label, the best, stats, actions and Share.

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
  doc "The results template: stars, the score (points or a time) counting up under its label, the best, stats, actions and Share."
  host false
  param id String "Its id."
  param desc ResultsDesc "Score, stars, stats and actions."
  returns MenuResult
}
```

<sub>`api.ogt` line 13976</sub>

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

<sub>`api.ogt` line 13984</sub>

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

<sub>`api.ogt` line 13992</sub>

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

<sub>`api.ogt` line 14000</sub>

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

<sub>`api.ogt` line 14008</sub>

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

<sub>`api.ogt` line 14016</sub>

<a id="fn-props"></a>

### `og.ui.props`

Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (docs/ui.md §6, §7). Call it right after the element.

```ts
og.ui.props(desc?: ElementProps): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ElementProps` | yes | Its props. |

**Returns** `boolean`: False when no element was built before it.

Binding symbol `og_ui_props` · publish builds: kept

```ogt
fn props {
  doc "Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (docs/ui.md §6, §7). Call it right after the element."
  param desc ElementProps "Its props."
  returns bool "False when no element was built before it."
}
```

<sub>`api.ogt` line 14685</sub>

<a id="fn-defineTimeline"></a>

### `og.ui.defineTimeline`

Define a timeline by name: configuration, like the theme, defined once at start. Plays and state groups name it.

```ts
og.ui.defineTimeline(name: string, timeline?: UiTimeline): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name. |
| `timeline` | `UiTimeline` | yes | Its tracks, events and labels. |

**Returns** `void`.

Binding symbol `og_ui_define_timeline` · publish builds: kept

```ogt
fn defineTimeline {
  doc "Define a timeline by name: configuration, like the theme, defined once at start. Plays and state groups name it."
  param name String "Its name."
  param timeline UiTimeline "Its tracks, events and labels."
  returns Void
}
```

<sub>`api.ogt` line 14691</sub>

<a id="fn-play"></a>

### `og.ui.play`

Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). Returns the play's id (0: no element, or a choreo of no known kind).

```ts
og.ui.play(path: string, choreo: Choreo, desc?: PlayDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path; empty is the element built just before. |
| `choreo` | `Choreo` | no | What plays. |
| `desc` | `PlayDesc` | yes | How it runs. |

**Returns** `number`.

Binding symbol `og_ui_play` · publish builds: kept

```ogt
fn play {
  doc "Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). Returns the play's id (0: no element, or a choreo of no known kind)."
  param path String "The element's full path; empty is the element built just before."
  param choreo Choreo "What plays."
  param desc PlayDesc "How it runs."
  returns u32
}
```

<sub>`api.ogt` line 14698</sub>

<a id="fn-playTimeline"></a>

### `og.ui.playTimeline`

Play one timeline (by name) on an element: og.ui.play with a single step.

```ts
og.ui.playTimeline(path: string, timeline: string, desc?: PlayDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path; empty is the element built just before. |
| `timeline` | `string` | no | The timeline. |
| `desc` | `PlayDesc` | yes | How it runs. |

**Returns** `number`.

Binding symbol `og_ui_play_timeline` · publish builds: kept

```ogt
fn playTimeline {
  doc "Play one timeline (by name) on an element: og.ui.play with a single step."
  param path String "The element's full path; empty is the element built just before."
  param timeline String "The timeline."
  param desc PlayDesc "How it runs."
  returns u32
}
```

<sub>`api.ogt` line 14706</sub>

<a id="fn-seek"></a>

### `og.ui.seek`

Move a play to a time; it fires no events on the way. Returns false for an unknown play.

```ts
og.ui.seek(play: number, ms: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |
| `ms` | `number` | no | The time, ms. |

**Returns** `boolean`.

Binding symbol `og_ui_seek` · publish builds: kept

```ogt
fn seek {
  doc "Move a play to a time; it fires no events on the way. Returns false for an unknown play."
  param play u32 "The play."
  param ms f32 "The time, ms."
  returns bool
}
```

<sub>`api.ogt` line 14714</sub>

<a id="fn-pause"></a>

### `og.ui.pause`

Hold a play where it is.

```ts
og.ui.pause(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_pause` · publish builds: kept

```ogt
fn pause {
  doc "Hold a play where it is."
  param play u32 "The play."
  returns bool
}
```

<sub>`api.ogt` line 14721</sub>

<a id="fn-resume"></a>

### `og.ui.resume`

Let a held play go on.

```ts
og.ui.resume(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_resume` · publish builds: kept

```ogt
fn resume {
  doc "Let a held play go on."
  param play u32 "The play."
  returns bool
}
```

<sub>`api.ogt` line 14727</sub>

<a id="fn-reverse"></a>

### `og.ui.reverse`

Turn a play round where it is: an interrupted opening closes from where it got to.

```ts
og.ui.reverse(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_reverse` · publish builds: kept

```ogt
fn reverse {
  doc "Turn a play round where it is: an interrupted opening closes from where it got to."
  param play u32 "The play."
  returns bool
}
```

<sub>`api.ogt` line 14733</sub>

<a id="fn-stop"></a>

### `og.ui.stop`

Stop a play: its elements go back to their own values (on their motion, if they have one).

```ts
og.ui.stop(play: number): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `boolean`.

Binding symbol `og_ui_stop` · publish builds: kept

```ogt
fn stop {
  doc "Stop a play: its elements go back to their own values (on their motion, if they have one)."
  param play u32 "The play."
  returns bool
}
```

<sub>`api.ogt` line 14739</sub>

<a id="fn-playInfo"></a>

### `og.ui.playInfo`

Where a play is; null once it is gone.

```ts
og.ui.playInfo(play: number): PlayInfo | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `play` | `number` | no | The play. |

**Returns** `PlayInfo | null`.

Binding symbol `og_ui_play_info` · publish builds: kept

```ogt
fn playInfo {
  doc "Where a play is; null once it is gone."
  param play u32 "The play."
  returns Option.PlayInfo
}
```

<sub>`api.ogt` line 14745</sub>

<a id="fn-forceState"></a>

### `og.ui.forceState`

Force a state group of an element (the probe and agents render and check each state this way); null lets it go.

```ts
og.ui.forceState(path: string, group: string, state: string | null): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |
| `group` | `string` | no | The group. |
| `state` | `string \| null` | no | The state; null releases it. |

**Returns** `void`.

Binding symbol `og_ui_force_state` · publish builds: kept

```ogt
fn forceState {
  doc "Force a state group of an element (the probe and agents render and check each state this way); null lets it go."
  param path String "The element's full path."
  param group String "The group."
  param state Option.String "The state; null releases it."
  returns Void
}
```

<sub>`api.ogt` line 14751</sub>

<a id="fn-visualState"></a>

### `og.ui.visualState`

The state a group of an element is in (last tick); empty for none.

```ts
og.ui.visualState(path: string, group: string): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |
| `group` | `string` | no | The group. |

**Returns** `string`.

Binding symbol `og_ui_visual_state` · publish builds: kept

```ogt
fn visualState {
  doc "The state a group of an element is in (last tick); empty for none."
  param path String "The element's full path."
  param group String "The group."
  returns String
}
```

<sub>`api.ogt` line 14759</sub>

<a id="fn-animate"></a>

### `og.ui.animate`

Any number on a spring or an ease (docs/ui.md §7.1: every value is animatable): call it every tick with the target and it answers this tick's value. A retarget keeps its value and velocity; under reduced motion it jumps.

```ts
og.ui.animate(id: string, target: number, transition: Transition): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id under the open element. |
| `target` | `number` | no | Where it is going. |
| `transition` | `Transition` | no | How it moves. |

**Returns** `number`.

Binding symbol `og_ui_animate` · publish builds: kept

```ogt
fn animate {
  doc "Any number on a spring or an ease (docs/ui.md §7.1: every value is animatable): call it every tick with the target and it answers this tick's value. A retarget keeps its value and velocity; under reduced motion it jumps."
  param id String "Its id under the open element."
  param target f32 "Where it is going."
  param transition Transition "How it moves."
  returns f32
}
```

<sub>`api.ogt` line 14766</sub>

<a id="fn-gesture"></a>

### `og.ui.gesture`

What the drag or pinch on an element is doing.

```ts
og.ui.gesture(path: string): GestureInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | The element's full path. |

**Returns** `GestureInfo`.

Binding symbol `og_ui_gesture` · publish builds: kept

```ogt
fn gesture {
  doc "What the drag or pinch on an element is doing."
  param path String "The element's full path."
  returns GestureInfo
}
```

<sub>`api.ogt` line 14774</sub>

<a id="fn-motionEvents"></a>

### `og.ui.motionEvents`

The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps.

```ts
og.ui.motionEvents(): readonly UiMotionEvent[]
```

**Returns** `readonly UiMotionEvent[]`.

Binding symbol `og_ui_motion_events` · publish builds: kept

```ogt
fn motionEvents {
  doc "The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps."
  returns List.UiMotionEvent
}
```

<sub>`api.ogt` line 14780</sub>

<a id="fn-overlaps"></a>

### `og.ui.overlaps`

The last frame's undeclared overlaps between elements (docs/ui.md §6.2), with both paths.

```ts
og.ui.overlaps(): readonly UiOverlap[]
```

**Returns** `readonly UiOverlap[]`.

Binding symbol `og_ui_overlaps` · publish builds: kept

```ogt
fn overlaps {
  doc "The last frame's undeclared overlaps between elements (docs/ui.md §6.2), with both paths."
  returns List.UiOverlap
}
```

<sub>`api.ogt` line 14785</sub>

<a id="fn-layoutStats"></a>

### `og.ui.layoutStats`

What the last frame built, rebuilt, laid out and kept.

```ts
og.ui.layoutStats(): UiLayoutStats
```

**Returns** `UiLayoutStats`.

Binding symbol `og_ui_layout_stats` · publish builds: kept

```ogt
fn layoutStats {
  doc "What the last frame built, rebuilt, laid out and kept."
  returns UiLayoutStats
}
```

<sub>`api.ogt` line 14790</sub>

<a id="fn-setDraw"></a>

### `og.ui.setDraw`

Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glows, glass, clipping to its shape, masks, blend modes and materials (docs/ui.md §5). Call it right after the element's open or control; immediate mode, so every tick it is drawn.

```ts
og.ui.setDraw(desc?: DrawDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `DrawDesc` | yes | What it draws with. |

**Returns** `void`.

Binding symbol `og_ui_set_draw` · publish builds: kept

```ogt
fn setDraw {
  doc "Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glows, glass, clipping to its shape, masks, blend modes and materials (docs/ui.md §5). Call it right after the element's open or control; immediate mode, so every tick it is drawn."
  param desc DrawDesc "What it draws with."
  returns Void
}
```

<sub>`api.ogt` line 15218</sub>

<a id="fn-openWorldAnchor"></a>

### `og.ui.openWorldAnchor`

Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, decluttered and clamped as desc says. Drawn under the screen's own elements. og.ui.setDraw after it draws its box. For thousands of tags of one kind (overhead bars on an army) use mass elements instead.

```ts
og.ui.openWorldAnchor(id: string, desc?: WorldAnchorDesc): AnchorInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `WorldAnchorDesc` | yes | What it sits on and how it follows the camera. |

**Returns** `AnchorInfo`.

Binding symbol `og_ui_open_world_anchor` · publish builds: kept

```ogt
fn openWorldAnchor {
  doc "Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, decluttered and clamped as desc says. Drawn under the screen's own elements. og.ui.setDraw after it draws its box. For thousands of tags of one kind (overhead bars on an army) use mass elements instead."
  param id String "Its id; the probe path is the id alone."
  param desc WorldAnchorDesc "What it sits on and how it follows the camera."
  returns AnchorInfo
}
```

<sub>`api.ogt` line 15348</sub>

<a id="fn-openWorldPanel"></a>

### `og.ui.openWorldPanel`

Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. One render target per panel.

```ts
og.ui.openWorldPanel(id: string, desc?: WorldPanelDesc): PanelInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id; the probe path is the id alone. |
| `desc` | `WorldPanelDesc` | yes | Its target, size and quad. |

**Returns** `PanelInfo`.

Binding symbol `og_ui_open_world_panel` · publish builds: kept

```ogt
fn openWorldPanel {
  doc "Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. One render target per panel."
  param id String "Its id; the probe path is the id alone."
  param desc WorldPanelDesc "Its target, size and quad."
  returns PanelInfo
}
```

<sub>`api.ogt` line 15403</sub>

<a id="fn-defineMass"></a>

### `og.ui.defineMass`

Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle.

```ts
og.ui.defineMass(name: string, desc?: MassTemplateDesc): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | Its name. |
| `desc` | `MassTemplateDesc` | yes | What it draws and how it moves. |

**Returns** `number`.

Binding symbol `og_ui_define_mass` · publish builds: kept

```ogt
fn defineMass {
  doc "Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle."
  param name String "Its name."
  param desc MassTemplateDesc "What it draws and how it moves."
  returns u32
}
```

<sub>`api.ogt` line 15548</sub>

<a id="fn-spawnMass"></a>

### `og.ui.spawnMass`

Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). They live in the UI's state, so a snapshot or a replay carries them.

```ts
og.ui.spawnMass(template: number, instances: readonly MassInstance[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |
| `instances` | `readonly MassInstance[]` | no | The instances. |

**Returns** `void`.

Binding symbol `og_ui_spawn_mass` · publish builds: kept

```ogt
fn spawnMass {
  doc "Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). They live in the UI's state, so a snapshot or a replay carries them."
  param template u32 "From defineMass."
  param instances List.MassInstance "The instances."
  returns Void
}
```

<sub>`api.ogt` line 15555</sub>

<a id="fn-pushMass"></a>

### `og.ui.pushMass`

Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values).

```ts
og.ui.pushMass(template: number, instances: readonly MassInstance[]): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |
| `instances` | `readonly MassInstance[]` | no | The instances. |

**Returns** `void`.

Binding symbol `og_ui_push_mass` · publish builds: kept

```ogt
fn pushMass {
  doc "Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values)."
  param template u32 "From defineMass."
  param instances List.MassInstance "The instances."
  returns Void
}
```

<sub>`api.ogt` line 15562</sub>

<a id="fn-massCount"></a>

### `og.ui.massCount`

The live spawned instances of a template.

```ts
og.ui.massCount(template: number): number
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `template` | `number` | no | From defineMass. |

**Returns** `number`.

Binding symbol `og_ui_mass_count` · publish builds: kept

```ogt
fn massCount {
  doc "The live spawned instances of a template."
  param template u32 "From defineMass."
  returns u32
}
```

<sub>`api.ogt` line 15569</sub>

<a id="fn-particles"></a>

### `og.ui.particles`

Emit particles from the element built just before (a control, or a container just opened). Call every tick it emits; a burst throws on the ticks desc.fire is set. og.ui.burst is a preset of it.

```ts
og.ui.particles(desc?: ParticlesDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `desc` | `ParticlesDesc` | yes | What it throws. |

**Returns** `void`.

Binding symbol `og_ui_particles` · publish builds: kept

```ogt
fn particles {
  doc "Emit particles from the element built just before (a control, or a container just opened). Call every tick it emits; a burst throws on the ticks desc.fire is set. og.ui.burst is a preset of it."
  param desc ParticlesDesc "What it throws."
  returns Void
}
```

<sub>`api.ogt` line 15642</sub>

<a id="fn-meta"></a>

### `og.ui.meta`

A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame.

```ts
og.ui.meta(id: string, desc?: MetaDesc): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `MetaDesc` | yes | What it draws. |

**Returns** `void`.

Binding symbol `og_ui_meta` · publish builds: kept

```ogt
fn meta {
  doc "A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame."
  param id String "Its id."
  param desc MetaDesc "What it draws."
  returns Void
}
```

<sub>`api.ogt` line 15692</sub>

<a id="fn-sceneViewport"></a>

### `og.ui.sceneViewport`

A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate.

```ts
og.ui.sceneViewport(id: string, desc?: SceneViewportDesc): SceneViewportInfo
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | Its id. |
| `desc` | `SceneViewportDesc` | yes | What it shows and its camera. |

**Returns** `SceneViewportInfo`.

Binding symbol `og_ui_scene_viewport` · publish builds: kept

```ogt
fn sceneViewport {
  doc "A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate."
  param id String "Its id."
  param desc SceneViewportDesc "What it shows and its camera."
  returns SceneViewportInfo
}
```

<sub>`api.ogt` line 15743</sub>

<a id="fn-screen"></a>

### `og.ui.screen`

Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots after this call (og.ui.openSlot). Its bindings read the viewmodel every tick, a Capitalised root (`Wallet.gold`) the resource as of the last tick; two-way controls write into it (each write is a `set` event). Layers code does not mount are built at the frame's end with the viewmodel they have.

```ts
og.ui.screen(name: string, vm?: unknown): UiDocResponse
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The screen's name (its `name` line). |
| `vm` | `unknown` | yes | Its viewmodel: an object (null keeps the one it has). |

**Returns** `UiDocResponse`.

Binding symbol `og_ui_screen` · publish builds: kept

```ogt
fn screen {
  doc "Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots after this call (og.ui.openSlot). Its bindings read the viewmodel every tick, a Capitalised root (`Wallet.gold`) the resource as of the last tick; two-way controls write into it (each write is a `set` event). Layers code does not mount are built at the frame's end with the viewmodel they have."
  param name String "The screen's name (its `name` line)."
  param vm Json "Its viewmodel: an object (null keeps the one it has)." {
    optional true
  }
  returns UiDocResponse
}
```

<sub>`api.ogt` line 15833</sub>

<a id="fn-component"></a>

### `og.ui.component`

Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props.

```ts
og.ui.component(id: string, name: string, props?: unknown): UiDocResponse
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `id` | `string` | no | The instance's root id. |
| `name` | `string` | no | The component's name. |
| `props` | `unknown` | yes | Its props: an object. |

**Returns** `UiDocResponse`.

Binding symbol `og_ui_component` · publish builds: kept

```ogt
fn component {
  doc "Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props."
  param id String "The instance's root id."
  param name String "The component's name."
  param props Json "Its props: an object." {
    optional true
  }
  returns UiDocResponse
}
```

<sub>`api.ogt` line 15842</sub>

<a id="fn-openSlot"></a>

### `og.ui.openSlot`

Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. False when no such slot was built this tick.

```ts
og.ui.openSlot(path: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `path` | `string` | no | Its full path, or `<screen>/<slot id>`. |

**Returns** `boolean`.

Binding symbol `og_ui_open_slot` · publish builds: kept

```ogt
fn openSlot {
  doc "Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. False when no such slot was built this tick."
  param path String "Its full path, or `<screen>/<slot id>`."
  returns bool
}
```

<sub>`api.ogt` line 15852</sub>

<a id="fn-push"></a>

### `og.ui.push`

Push a screen onto the layer stack (docs/ui.md §8.1); the layer under it remembers its focus. Callable anywhere (systems, commands, start); the stack is world state, so snapshots carry it.

```ts
og.ui.push(world: World, screen: string, vm?: unknown): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |
| `vm` | `unknown` | yes | Its viewmodel (null: its document's first fixture, else an empty object). |

**Returns** `Result<number>`: The layer's serial. Refused for a name no screen document has.

Binding symbol `og_ui_push` · publish builds: kept

```ogt
fn push {
  doc "Push a screen onto the layer stack (docs/ui.md §8.1); the layer under it remembers its focus. Callable anywhere (systems, commands, start); the stack is world state, so snapshots carry it."
  param world World "The world."
  param screen String "The screen's name."
  param vm Json "Its viewmodel (null: its document's first fixture, else an empty object)." {
    optional true
  }
  returns Result.u32 "The layer's serial. Refused for a name no screen document has."
}
```

<sub>`api.ogt` line 15858</sub>

<a id="fn-pop"></a>

### `og.ui.pop`

Pop the top layer; the one under it takes its remembered focus. Answers the popped screen's name (empty: the stack was empty).

```ts
og.ui.pop(world: World): string
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `string`.

Binding symbol `og_ui_pop` · publish builds: kept

```ogt
fn pop {
  doc "Pop the top layer; the one under it takes its remembered focus. Answers the popped screen's name (empty: the stack was empty)."
  param world World "The world."
  returns String
}
```

<sub>`api.ogt` line 15868</sub>

<a id="fn-replace"></a>

### `og.ui.replace`

Replace the top layer with a screen.

```ts
og.ui.replace(world: World, screen: string, vm?: unknown): Result<number>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |
| `vm` | `unknown` | yes | Its viewmodel. |

**Returns** `Result<number>`: The new layer's serial.

Binding symbol `og_ui_replace` · publish builds: kept

```ogt
fn replace {
  doc "Replace the top layer with a screen."
  param world World "The world."
  param screen String "The screen's name."
  param vm Json "Its viewmodel." {
    optional true
  }
  returns Result.u32 "The new layer's serial."
}
```

<sub>`api.ogt` line 15874</sub>

<a id="fn-clear"></a>

### `og.ui.clear`

Empty the layer stack.

```ts
og.ui.clear(world: World): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `void`.

Binding symbol `og_ui_clear` · publish builds: kept

```ogt
fn clear {
  doc "Empty the layer stack."
  param world World "The world."
  returns Void
}
```

<sub>`api.ogt` line 15884</sub>

<a id="fn-layers"></a>

### `og.ui.layers`

The layer stack, bottom first.

```ts
og.ui.layers(world: World): readonly UiLayer[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiLayer[]`.

Binding symbol `og_ui_layers` · publish builds: kept

```ogt
fn layers {
  doc "The layer stack, bottom first."
  param world World "The world."
  returns List.UiLayer
}
```

<sub>`api.ogt` line 15890</sub>

<a id="fn-screenVm"></a>

### `og.ui.screenVm`

A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack.

```ts
og.ui.screenVm(world: World, screen: string): unknown
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `screen` | `string` | no | The screen's name. |

**Returns** `unknown`.

Binding symbol `og_ui_screen_vm` · publish builds: kept

```ogt
fn screenVm {
  doc "A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack."
  param world World "The world."
  param screen String "The screen's name."
  returns Json
}
```

<sub>`api.ogt` line 15896</sub>

<a id="fn-docEvents"></a>

### `og.ui.docEvents`

Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own.

```ts
og.ui.docEvents(world: World): readonly UiDocEvent[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiDocEvent[]`.

Binding symbol `og_ui_doc_events` · publish builds: kept

```ogt
fn docEvents {
  doc "Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own."
  param world World "The world."
  returns List.UiDocEvent
}
```

<sub>`api.ogt` line 15903</sub>

<a id="fn-gameInput"></a>

### `og.ui.gameInput`

Gameplay takes input this tick: no layer whose input is `menu` is on the stack. Input goes to the top layer that takes it, once per tick.

```ts
og.ui.gameInput(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_game_input` · publish builds: kept

```ogt
fn gameInput {
  doc "Gameplay takes input this tick: no layer whose input is `menu` is on the stack. Input goes to the top layer that takes it, once per tick."
  param world World "The world."
  returns bool
}
```

<sub>`api.ogt` line 15909</sub>

<a id="fn-paused"></a>

### `og.ui.paused`

A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had.

```ts
og.ui.paused(world: World): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `boolean`.

Binding symbol `og_ui_paused` · publish builds: kept

```ogt
fn paused {
  doc "A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had."
  param world World "The world."
  returns bool
}
```

<sub>`api.ogt` line 15915</sub>

<a id="fn-defineAction"></a>

### `og.ui.defineAction`

Define (or redefine) a UI action. confirm (Enter, Space, A), back (Escape, B), tabNext (E, ], RB), tabPrev (Q, [, LB) and details (X, Y) are defined by default.

```ts
og.ui.defineAction(world: World, desc: UiActionDesc): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `desc` | `UiActionDesc` | no | The action. |

**Returns** `Result<void>`.

Binding symbol `og_ui_define_action` · publish builds: kept

```ogt
fn defineAction {
  doc "Define (or redefine) a UI action. confirm (Enter, Space, A), back (Escape, B), tabNext (E, ], RB), tabPrev (Q, [, LB) and details (X, Y) are defined by default."
  param world World "The world."
  param desc UiActionDesc "The action."
  returns Result.Void
}
```

<sub>`api.ogt` line 15921</sub>

<a id="fn-action"></a>

### `og.ui.action`

The action fired this tick (a document's `on action` fires on the top layer that takes input).

```ts
og.ui.action(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_ui_action` · publish builds: kept

```ogt
fn action {
  doc "The action fired this tick (a document's `on action` fires on the top layer that takes input)."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 15928</sub>

<a id="fn-actionHeld"></a>

### `og.ui.actionHeld`

A hold action is held past its hold, or its toggle is on.

```ts
og.ui.actionHeld(name: string): boolean
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `name` | `string` | no | The action. |

**Returns** `boolean`.

Binding symbol `og_ui_action_held` · publish builds: kept

```ogt
fn actionHeld {
  doc "A hold action is held past its hold, or its toggle is on."
  param name String "The action."
  returns bool
}
```

<sub>`api.ogt` line 15934</sub>

<a id="fn-glyph"></a>

### `og.ui.glyph`

An action's glyph on the device used last: its first binding there. Null for an unknown action or one with no binding on that device.

```ts
og.ui.glyph(world: World, action: string): UiGlyph | null
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `action` | `string` | no | The action. |

**Returns** `UiGlyph | null`.

Binding symbol `og_ui_glyph` · publish builds: kept

```ogt
fn glyph {
  doc "An action's glyph on the device used last: its first binding there. Null for an unknown action or one with no binding on that device."
  param world World "The world."
  param action String "The action."
  returns Option.UiGlyph
}
```

<sub>`api.ogt` line 15940</sub>

<a id="fn-device"></a>

### `og.ui.device`

The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer.

```ts
og.ui.device(world: World): UiDevice
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `UiDevice`.

Binding symbol `og_ui_device` · publish builds: kept

```ogt
fn device {
  doc "The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer."
  param world World "The world."
  returns UiDevice
}
```

<sub>`api.ogt` line 15947</sub>

<a id="fn-setDevice"></a>

### `og.ui.setDevice`

Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on.

```ts
og.ui.setDevice(world: World, device: UiDevice): void
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `device` | `UiDevice` | no | The device. |

**Returns** `void`.

Binding symbol `og_ui_set_device` · publish builds: kept

```ogt
fn setDevice {
  doc "Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on."
  param world World "The world."
  param device UiDevice "The device."
  returns Void
}
```

<sub>`api.ogt` line 15953</sub>

<a id="fn-actionBar"></a>

### `og.ui.actionBar`

What an action bar component reads: the actions the top layer's document lists (`actions { confirm "Buy" }`), each with its hint and its glyph on the device used last.

```ts
og.ui.actionBar(world: World): readonly UiActionHint[]
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `readonly UiActionHint[]`.

Binding symbol `og_ui_action_bar` · publish builds: kept

```ogt
fn actionBar {
  doc "What an action bar component reads: the actions the top layer's document lists (`actions { confirm \"Buy\" }`), each with its hint and its glyph on the device used last."
  param world World "The world."
  returns List.UiActionHint
}
```

<sub>`api.ogt` line 15960</sub>

<a id="fn-documentSchema"></a>

### `og.ui.documentSchema`

The reflected schema documents are validated against (docs/ui.md §10): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back.

```ts
og.ui.documentSchema(): unknown
```

**Returns** `unknown`.

Binding symbol `og_ui_document_schema` · publish builds: kept

```ogt
fn documentSchema {
  doc "The reflected schema documents are validated against (docs/ui.md §10): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back."
  returns Json
}
```

<sub>`api.ogt` line 15966</sub>

<a id="fn-setProp"></a>

### `og.ui.setProp`

Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. The value is written as the document's line would be (`token primary`, `grow`, `bind gold compact`).

```ts
og.ui.setProp(world: World, path: string, prop: string, value: string): Result<void>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |
| `path` | `string` | no | The element's full path. |
| `prop` | `string` | no | The prop (og.ui.documentSchema). |
| `value` | `string` | no | Its value as a document line writes it. |

**Returns** `Result<void>`: Refused for an unknown prop or a value it does not take.

Binding symbol `og_ui_set_prop` · publish builds: kept

```ogt
fn setProp {
  doc "Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. The value is written as the document's line would be (`token primary`, `grow`, `bind gold compact`)."
  param world World "The world."
  param path String "The element's full path."
  param prop String "The prop (og.ui.documentSchema)."
  param value String "Its value as a document line writes it."
  returns Result.Void "Refused for an unknown prop or a value it does not take."
}
```

<sub>`api.ogt` line 15971</sub>

<a id="fn-saveEdits"></a>

### `og.ui.saveEdits`

Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes; the documents take the new text at once. Answers the files written.

```ts
og.ui.saveEdits(world: World): Result<readonly string[]>
```

| Parameter | Type | Optional | Doc |
|---|---|---|---|
| `world` | `World` | no | The world. |

**Returns** `Result<readonly string[]>`: Refused on a host with no project directory, or a write that fails.

Binding symbol `og_ui_save_edits` · publish builds: kept

```ogt
fn saveEdits {
  doc "Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes; the documents take the new text at once. Answers the files written."
  param world World "The world."
  returns Result.List.String "Refused on a host with no project directory, or a write that fails."
}
```

<sub>`api.ogt` line 15980</sub>

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

<sub>`api.ogt` line 11444</sub>

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

<sub>`api.ogt` line 11450</sub>

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
| `"baseline"` | In a row, children share their first text baseline (rows of mixed type sizes); a column treats it as start. |

```ogt
enum Cross {
  doc "Where children sit across the main axis."
  variant start "At the start."
  variant center "Centred."
  variant end "At the end."
  variant stretch "Stretched to the container's inner size."
  variant baseline "In a row, children share their first text baseline (rows of mixed type sizes); a column treats it as start."
}
```

<sub>`api.ogt` line 11458</sub>

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

<sub>`api.ogt` line 11467</sub>

<a id="type-FontRole"></a>

### `FontRole` (enum)

Which configured font text uses; display and mono fall back to body, rounded to display, then body.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | The body font. |
| `"display"` | The display font. |
| `"mono"` | The monospaced font. |
| `"rounded"` | The rounded font, for levels and scores. |

```ogt
enum FontRole {
  doc "Which configured font text uses; display and mono fall back to body, rounded to display, then body."
  variant body "The body font."
  variant display "The display font."
  variant mono "The monospaced font."
  variant rounded "The rounded font, for levels and scores."
}
```

<sub>`api.ogt` line 11474</sub>

<a id="type-ButtonVariant"></a>

### `ButtonVariant` (enum)

Button roles: how the style draws each.

A string: one of the variants below. The zero value is `"secondary"`.

| Variant | Doc |
|---|---|
| `"secondary"` | A neutral fill (the default). |
| `"primary"` | The primary action: one per screen. |
| `"quiet"` | Text until hovered. |
| `"danger"` | A destructive action: danger ink on the danger tint. |

```ogt
enum ButtonVariant {
  doc "Button roles: how the style draws each."
  variant secondary "A neutral fill (the default)."
  variant primary "The primary action: one per screen."
  variant quiet "Text until hovered."
  variant danger "A destructive action: danger ink on the danger tint."
}
```

<sub>`api.ogt` line 11482</sub>

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

<sub>`api.ogt` line 11490</sub>

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

<sub>`api.ogt` line 11497</sub>

<a id="type-DensityMode"></a>

### `DensityMode` (enum)

Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise.

A string: one of the variants below. The zero value is `"auto"`.

| Variant | Doc |
|---|---|
| `"auto"` | Touch sizes on a coarse pointer or a narrow surface, mouse sizes otherwise. |
| `"mouse"` | A mouse and keyboard: the tightest controls. |
| `"pad"` | A gamepad across a room: larger controls, rows and gutters that read from a couch. |
| `"touch"` | A finger: every control at least 44 CSS px. |

```ogt
enum DensityMode {
  doc "Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise."
  variant auto "Touch sizes on a coarse pointer or a narrow surface, mouse sizes otherwise."
  variant mouse "A mouse and keyboard: the tightest controls."
  variant pad "A gamepad across a room: larger controls, rows and gutters that read from a couch."
  variant touch "A finger: every control at least 44 CSS px."
}
```

<sub>`api.ogt` line 11503</sub>

<a id="type-ThemeColor"></a>

### `ThemeColor` (enum)

A colour role of the UI style in effect (the game's style, or the prototype style before it picks one).

A string: one of the variants below. The zero value is `"backdrop"`.

| Variant | Doc |
|---|---|
| `"backdrop"` | The screen behind the UI where nothing else is. |
| `"well"` | A recessed ground inside a panel: wells, map frames, empty slots. |
| `"panel"` | Panels and cards. |
| `"panelHover"` | A panel under the pointer. |
| `"panelRaised"` | Dialogs, menus and tooltips. |
| `"line"` | Hairlines and frames. |
| `"lineStrong"` | Stronger frames: high contrast, map borders, scrollbars. |
| `"ink"` | The main ink. |
| `"inkMuted"` | Secondary ink: captions, keys, inactive tabs. |
| `"inkFaint"` | The faintest ink that still reads: placeholders, hints. |
| `"inkInverse"` | Ink on a ground of ink (a selected chip). |
| `"control"` | Neutral control fills: secondary buttons, tracks, fields. |
| `"controlStrong"` | A neutral control hovered or pressed; selection. |
| `"primary"` | The one directed action: primary buttons, a slider's fill, a switch that is on, the caret. |
| `"primaryHover"` | primary hovered. |
| `"primaryActive"` | primary pressed. |
| `"onPrimary"` | Ink on primary. |
| `"primarySoft"` | A translucent wash of primary. |
| `"highlight"` | Emphasised ink: a speaker's name, a price, a link, the player's map dot. |
| `"focus"` | The focus ring of key and pad navigation. |
| `"knob"` | Slider and switch knobs. |
| `"hud"` | HUD capsules over the game (scores, timers, toasts). |
| `"scrim"` | Behind a dialog. |
| `"shadow"` | Drop shadows. |
| `"sheen"` | A skeleton's passing sheen. |
| `"success"` | Success. |
| `"warn"` | Warnings. |
| `"danger"` | Danger and errors. |
| `"info"` | Information. |
| `"health"` | A health meter. |
| `"mana"` | A mana meter. |
| `"xp"` | An experience meter. |
| `"gold"` | Currency: coins, gold, prices that can be paid. |

```ogt
enum ThemeColor {
  doc "A colour role of the UI style in effect (the game's style, or the prototype style before it picks one)."
  variant backdrop "The screen behind the UI where nothing else is."
  variant well "A recessed ground inside a panel: wells, map frames, empty slots."
  variant panel "Panels and cards."
  variant panelHover "A panel under the pointer."
  variant panelRaised "Dialogs, menus and tooltips."
  variant line "Hairlines and frames."
  variant lineStrong "Stronger frames: high contrast, map borders, scrollbars."
  variant ink "The main ink."
  variant inkMuted "Secondary ink: captions, keys, inactive tabs."
  variant inkFaint "The faintest ink that still reads: placeholders, hints."
  variant inkInverse "Ink on a ground of ink (a selected chip)."
  variant control "Neutral control fills: secondary buttons, tracks, fields."
  variant controlStrong "A neutral control hovered or pressed; selection."
  variant primary "The one directed action: primary buttons, a slider's fill, a switch that is on, the caret."
  variant primaryHover "primary hovered."
  variant primaryActive "primary pressed."
  variant onPrimary "Ink on primary."
  variant primarySoft "A translucent wash of primary."
  variant highlight "Emphasised ink: a speaker's name, a price, a link, the player's map dot."
  variant focus "The focus ring of key and pad navigation."
  variant knob "Slider and switch knobs."
  variant hud "HUD capsules over the game (scores, timers, toasts)."
  variant scrim "Behind a dialog."
  variant shadow "Drop shadows."
  variant sheen "A skeleton's passing sheen."
  variant success "Success."
  variant warn "Warnings."
  variant danger "Danger and errors."
  variant info "Information."
  variant health "A health meter."
  variant mana "A mana meter."
  variant xp "An experience meter."
  variant gold "Currency: coins, gold, prices that can be paid."
}
```

<sub>`api.ogt` line 11511</sub>

<a id="type-TypeStepName"></a>

### `TypeStepName` (enum)

A step of the style's type ramp.

A string: one of the variants below. The zero value is `"body"`.

| Variant | Doc |
|---|---|
| `"body"` | The default step. |
| `"xs"` | A step of the ramp. |
| `"meta"` | A step of the ramp. |
| `"caption"` | A step of the ramp. |
| `"sm"` | A step of the ramp. |
| `"reading"` | A step of the ramp. |
| `"base"` | A step of the ramp. |
| `"lg"` | A step of the ramp. |
| `"xl"` | A step of the ramp. |
| `"xl2"` | A step of the ramp. |
| `"xl3"` | A step of the ramp. |
| `"xl4"` | A step of the ramp. |
| `"xl5"` | A step of the ramp. |
| `"xl6"` | A step of the ramp. |
| `"xl7"` | A step of the ramp. |
| `"xl8"` | A step of the ramp. |
| `"displaySm"` | A step of the ramp. |
| `"displayMd"` | A step of the ramp. |
| `"displayLg"` | A step of the ramp. |
| `"displayXl"` | A step of the ramp. |
| `"displayXl2"` | A step of the ramp. |

```ogt
enum TypeStepName {
  doc "A step of the style's type ramp."
  variant body "The default step."
  variant xs "A step of the ramp."
  variant meta "A step of the ramp."
  variant caption "A step of the ramp."
  variant sm "A step of the ramp."
  variant reading "A step of the ramp."
  variant base "A step of the ramp."
  variant lg "A step of the ramp."
  variant xl "A step of the ramp."
  variant xl2 "A step of the ramp."
  variant xl3 "A step of the ramp."
  variant xl4 "A step of the ramp."
  variant xl5 "A step of the ramp."
  variant xl6 "A step of the ramp."
  variant xl7 "A step of the ramp."
  variant xl8 "A step of the ramp."
  variant displaySm "A step of the ramp."
  variant displayMd "A step of the ramp."
  variant displayLg "A step of the ramp."
  variant displayXl "A step of the ramp."
  variant displayXl2 "A step of the ramp."
}
```

<sub>`api.ogt` line 11548</sub>

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

<sub>`api.ogt` line 11573</sub>

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

<sub>`api.ogt` line 11585</sub>

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

<sub>`api.ogt` line 11592</sub>

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

<sub>`api.ogt` line 11599</sub>

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

<sub>`api.ogt` line 11608</sub>

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

<sub>`api.ogt` line 11624</sub>

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
| `frame` | `StylePart \| null` | `null` | — | — | — | The style frame its background takes (a HUD element, a card); null is a plain fill. Without a style the fill is drawn either way. |
| `part` | `string \| null` | `null` | — | — | — | The part its style dresses it as, a selector with its variant, state and attributes (`ShopCard.frame[rarity=epic][selected]`): the style's most specific matching `dress` blocks draw it, and a part the style does not dress draws from the tokens (docs/ui.md §12.3). Overrides frame; null keeps frame's. |
| `partClass` | `string \| null` | `null` | — | — | — | The element class whose shape language and elevation an undressed part takes: panel, dialog, card, tooltip, button, field, chip, badge, tab, slot, bar, hud, avatar; null is card. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none): with the height left to fit, it is as tall as its laid-out width over this; with a fixed height and the width left to fit, as wide as the height times it. |

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
  field frame Option.StylePart "The style frame its background takes (a HUD element, a card); null is a plain fill. Without a style the fill is drawn either way."
  field part Option.String "The part its style dresses it as, a selector with its variant, state and attributes (`ShopCard.frame[rarity=epic][selected]`): the style's most specific matching `dress` blocks draw it, and a part the style does not dress draws from the tokens (docs/ui.md §12.3). Overrides frame; null keeps frame's."
  field partClass Option.String "The element class whose shape language and elevation an undressed part takes: panel, dialog, card, tooltip, button, field, chip, badge, tab, slot, bar, hud, avatar; null is card."
  field aspect f32 "Width over height (0: none): with the height left to fit, it is as tall as its laid-out width over this; with a fixed height and the width left to fit, as wide as the height times it." {
    min 0
  }
}
```

<sub>`api.ogt` line 11636</sub>

<a id="type-LabelDesc"></a>

### `LabelDesc` (desc struct)

How a label draws its text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `align` | `TextAlign` | `"start"` | — | — | — | How the lines align. |
| `nowrap` | `boolean` | `false` | — | — | — | Keep to one line (breaking only at `\n`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font. |
| `heading` | `boolean` | `false` | — | — | — | A heading: a style with a flourish frames it. |
| `ellipsis` | `boolean` | `false` | — | — | — | Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text. |
| `maxLines` | `number` | `0` | — | — | — | The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out. |

```ogt
struct LabelDesc {
  doc "How a label draws its text."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is ink."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
  field heading bool "A heading: a style with a flourish frames it."
  field ellipsis bool "Cut with an ellipsis where the text does not fit its width: on its one line (with nowrap; the label can then shrink in a row) or on its last line (maxLines). The probe still reads the whole text."
  field maxLines u32 "The most lines a wrapping label shows; 0 is no limit. With ellipsis the last ends in an ellipsis when text is left out."
}
```

<sub>`api.ogt` line 11664</sub>

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
| `autofocus` | `boolean` | `false` | — | — | — | Take the focus when it appears (it was not on screen the tick before) and no dialog is open: what opens takes the focus (og-ui §4), so a menu's first item answers Enter and the pad's A from its first tick. og.ui.mainMenu's first item has it. |

```ogt
struct ButtonDesc {
  doc "A button's look."
  desc true
  field variant ButtonVariant "The look."
  field size ControlSize "The control height."
  field disabled bool "Neutral and inert."
  field width Option.Size "Its width; null fits the label."
  field autofocus bool "Take the focus when it appears (it was not on screen the tick before) and no dialog is open: what opens takes the focus (og-ui §4), so a menu's first item answers Enter and the pad's A from its first tick. og.ui.mainMenu's first item has it."
}
```

<sub>`api.ogt` line 11678</sub>

<a id="type-PanelDesc"></a>

### `PanelDesc` (desc struct)

A card: panel, the panel radius, padded and spaced by the style.

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
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null is panel. |

```ogt
struct PanelDesc {
  doc "A card: panel, the panel radius, padded and spaced by the style."
  desc true
  field row bool "Lay children left to right (the default is top to bottom)."
  field width Option.Size "Its width; null fits the content."
  field height Option.Size "Its height; null fits the content."
  field padding Option.Insets "Space inside; null is space.lg on every side."
  field gap Option.f32 "Space between children; null is space.md."
  field align Align "Where children sit along the main axis."
  field cross Cross "Where children sit across it."
  field background Option.Paint "Its fill; null is panel."
}
```

<sub>`api.ogt` line 11688</sub>

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

<sub>`api.ogt` line 11701</sub>

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

<sub>`api.ogt` line 11708</sub>

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
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none): with the height left to fit, the image is as tall as its laid-out width over this (a growing column keeps a picture's proportions); with a fixed height and the width left to fit, as wide as the height times it. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it (and the probe's text): a picture that means something (a portrait, a star earned) names it in the game's words; empty is decoration. |

```ogt
struct ImageDesc {
  doc "A texture region, or a solid rectangle without a texture."
  desc true
  field texture Option.Texture "The texture; null draws a solid rectangle of the tint."
  field uv Option.Array.f32.4 "The region drawn, [u0, v0, u1, v1] with v down; null is the whole texture."
  field tint Option.Paint "Multiplies the texture; null is white."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
  field aspect f32 "Width over height (0: none): with the height left to fit, the image is as tall as its laid-out width over this (a growing column keeps a picture's proportions); with a fixed height and the width left to fit, as wide as the height times it." {
    min 0
  }
  field label String "What a screen reader calls it (and the probe's text): a picture that means something (a portrait, a star earned) names it in the game's words; empty is decoration."
}
```

<sub>`api.ogt` line 11720</sub>

<a id="type-ZoomViewDesc"></a>

### `ZoomViewDesc` (desc struct)

A zoomable picture (a map, an artwork): what it shows and its zoom limits.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The picture; null draws a solid rectangle of the tint. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The picture's region, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies the texture; null is white. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null fits. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height (0: none), as ImageDesc.aspect. |
| `minZoom` | `number` | `0` | ≥ 0 | — | — | The least zoom; 0 is 1 (the whole picture). |
| `maxZoom` | `number` | `0` | ≥ 0 | — | — | The most zoom; 0 is 4. |

```ogt
struct ZoomViewDesc {
  doc "A zoomable picture (a map, an artwork): what it shows and its zoom limits."
  desc true
  field texture Option.Texture "The picture; null draws a solid rectangle of the tint."
  field uv Option.Array.f32.4 "The picture's region, [u0, v0, u1, v1] with v down; null is the whole texture."
  field tint Option.Paint "Multiplies the texture; null is white."
  field width Option.Size "Its width; null fits."
  field height Option.Size "Its height; null fits."
  field aspect f32 "Width over height (0: none), as ImageDesc.aspect." {
    min 0
  }
  field minZoom f32 "The least zoom; 0 is 1 (the whole picture)." {
    min 0
  }
  field maxZoom f32 "The most zoom; 0 is 4." {
    min 0
  }
}
```

<sub>`api.ogt` line 11734</sub>

<a id="type-ZoomResult"></a>

### `ZoomResult` (struct)

Where a zoom view is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `zoom` | `number` | `0` | — | — | — | Its zoom: 1 shows the whole picture, 2 half of it each way. |
| `centerX` | `number` | `0` | — | — | — | The middle of what it shows, as a fraction of the picture's width (0.5 at rest). |
| `centerY` | `number` | `0` | — | — | — | The middle of what it shows, as a fraction of the picture's height (0.5 at rest). |

```ogt
struct ZoomResult {
  doc "Where a zoom view is."
  field zoom f32 "Its zoom: 1 shows the whole picture, 2 half of it each way."
  field centerX f32 "The middle of what it shows, as a fraction of the picture's width (0.5 at rest)."
  field centerY f32 "The middle of what it shows, as a fraction of the picture's height (0.5 at rest)."
}
```

<sub>`api.ogt` line 11753</sub>

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

<sub>`api.ogt` line 11760</sub>

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

<sub>`api.ogt` line 11766</sub>

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

<sub>`api.ogt` line 11777</sub>

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

<sub>`api.ogt` line 11785</sub>

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

<sub>`api.ogt` line 11794</sub>

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

<sub>`api.ogt` line 11807</sub>

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

<sub>`api.ogt` line 11813</sub>

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

<sub>`api.ogt` line 11819</sub>

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
| `reducedMotion` | `boolean` | `false` | — | — | optional | The page prefers reduced motion (prefers-reduced-motion: reduce, from the web shell): the UI's theme only fades, whatever UiDesc.reducedMotion says. Omitted is false. |
| `highContrast` | `boolean` | `false` | — | — | optional | The page prefers more contrast (prefers-contrast: more, from the web shell): the UI's theme is the high-contrast one, whatever UiDesc.contrast says. Omitted is false. |
| `reducedTransparency` | `boolean` | `false` | — | — | optional | The page prefers less transparency (prefers-reduced-transparency: reduce, from the web shell): glass parts are opaque surfaces and the scrim and translucent fills flat, whatever UiDesc.reducedTransparency says (UiTheme.reducedTransparency). Omitted is false. |
| `macKeys` | `boolean` | `false` | — | — | optional | Text controls take macOS's editing keys: Command with A, C, X and Z, Command+←/→ to the line's ends (↑/↓ the text's), Option+←/→ and Option+Backspace by words, Control+A and E to the line's ends. Off: Control takes the shortcuts and moves by words (Alt too), Home and End go to the line's ends. The web shell sets it from the browser's platform (Apple's); a native host sets it on macOS, so a script that holds Ctrl or Meta edits as a person on that OS would (og.ui.previewViewport pins either). Omitted is false. |
| `covered` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | optional | Where the host's own control covers the surface: on a published page the web shell's round Back to the Portal, top left (about 12, 12 to 52, 52 CSS px; its name shows beside it for the first seconds of play). Anchored groups keep out of it, moved along the edge they sit on (a top-left group to its right), so the HUD templates never draw under it; a game drawing its own corner reads it from og.ui.frameViewport. Empty (omitted) is none: native hosts and pages without Back. |
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
}
```

<sub>`api.ogt` line 11825</sub>

<a id="type-UiDesc"></a>

### `UiDesc` (desc struct)

How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette. |
| `density` | `DensityMode` | `"auto"` | — | — | — | The control sizes. |
| `font` | `Font \| null` | `null` | — | — | — | The body font, from og.load.font; null measures text at half an em per character and draws none. |
| `displayFont` | `Font \| null` | `null` | — | — | — | The display font; null uses the body font. |
| `monoFont` | `Font \| null` | `null` | — | — | — | The monospaced font; null uses the body font. |
| `roundedFont` | `Font \| null` | `null` | — | — | — | The rounded font (levels and scores); null uses the display font, then the body font. |
| `fallbackFonts` | `readonly Font[]` | `[]` | — | — | — | Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. A text mixing scripts takes each run from the first font that has it (the role's for its Latin, a fallback for its CJK or Arabic), one label in several fonts. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast). |
| `primaryColor` | `Color \| null` | `null` | — | — | — | An sRGB colour as the game's primary (its hover, pressed and soft tones derived from it, the ink on it whichever of ink and inkInverse reads better); null keeps the style's. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion). |
| `reducedTransparency` | `boolean` | `false` | — | — | — | Nothing is see-through: glass parts (a style's glass line) are opaque surfaces (no blur of the frame under them, their frame whole), the scrim is the opaque colour it makes over backdrop and the translucent fills (control, controlStrong, primarySoft, hud) the opaque colours they make over panel. The page's prefers-reduced-transparency turns it on too (Viewport.reducedTransparency); UiTheme.reducedTransparency says it is on. |
| `icons` | `IconAtlasDesc \| null` | `null` | — | — | — | The icon atlas og.ui.icon draws from; null takes the style's. |
| `toastAnchor` | `Anchor \| null` | `null` | — | — | — | Where toasts stack; null is the bottom centre. |
| `style` | `Style \| null` | `null` | — | — | — | A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the prototype style (fonts, icons and a primary colour given here still win); null is the prototype style, the engine's placeholder. |
| `sounds` | `UiSounds \| null` | `null` | — | — | — | Sounds for UI events (they win over the style's). |

```ogt
struct UiDesc {
  doc "How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. og.ui.configure takes it in part: a field left out keeps the value the last call (or a look's style) gave it, null clears it."
  desc true
  field scheme Scheme "The palette."
  field density DensityMode "The control sizes."
  field font Option.Font "The body font, from og.load.font; null measures text at half an em per character and draws none."
  field displayFont Option.Font "The display font; null uses the body font."
  field monoFont Option.Font "The monospaced font; null uses the body font."
  field roundedFont Option.Font "The rounded font (levels and scores); null uses the display font, then the body font."
  field fallbackFonts List.Font "Fonts for other scripts (Hebrew, Devanagari, a CJK face with `font { runtime true }`), tried in order after the locale's (og.i18n) for a text the role's font cannot draw whole. A text mixing scripts takes each run from the first font that has it (the role's for its Latin, a fallback for its CJK or Arabic), one label in several fonts."
  field contrast Contrast "Standard or high contrast. The page's prefers-contrast: more takes high contrast too (Viewport.highContrast)."
  field primaryColor Option.Color "An sRGB colour as the game's primary (its hover, pressed and soft tones derived from it, the ink on it whichever of ink and inkInverse reads better); null keeps the style's."
  field reducedMotion bool "Transitions only fade; sheens stop. The page's prefers-reduced-motion turns it on too (Viewport.reducedMotion)."
  field reducedTransparency bool "Nothing is see-through: glass parts (a style's glass line) are opaque surfaces (no blur of the frame under them, their frame whole), the scrim is the opaque colour it makes over backdrop and the translucent fills (control, controlStrong, primarySoft, hud) the opaque colours they make over panel. The page's prefers-reduced-transparency turns it on too (Viewport.reducedTransparency); UiTheme.reducedTransparency says it is on."
  field icons Option.IconAtlasDesc "The icon atlas og.ui.icon draws from; null takes the style's."
  field toastAnchor Option.Anchor "Where toasts stack; null is the bottom centre."
  field style Option.Style "A UI style from og.load.style: its palette, radii, type ramp, fonts, icons, frames, ornaments, motion and sounds replace the prototype style (fonts, icons and a primary colour given here still win); null is the prototype style, the engine's placeholder."
  field sounds Option.UiSounds "Sounds for UI events (they win over the style's)."
}
```

<sub>`api.ogt` line 11856</sub>

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

<sub>`api.ogt` line 11876</sub>

<a id="type-StylePart"></a>

### `StylePart` (enum)

An element class a style dresses with a nine-slice frame.

A string: one of the variants below. The zero value is `"panel"`.

| Variant | Doc |
|---|---|
| `"panel"` | Dialogs and screen panels. |
| `"card"` | Cards. |
| `"button"` | Buttons (with hover, pressed and disabled states). |
| `"field"` | Text fields. |
| `"slot"` | Inventory and grid cells. |
| `"tooltip"` | Tooltips. |
| `"hud"` | HUD elements over the game (counters, timers) and toasts. |
| `"track"` | A progress bar's track. |

```ogt
enum StylePart {
  doc "An element class a style dresses with a nine-slice frame."
  variant panel "Dialogs and screen panels."
  variant card "Cards."
  variant button "Buttons (with hover, pressed and disabled states)."
  variant field "Text fields."
  variant slot "Inventory and grid cells."
  variant tooltip "Tooltips."
  variant hud "HUD elements over the game (counters, timers) and toasts."
  variant track "A progress bar's track."
}
```

<sub>`api.ogt` line 11887</sub>

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

<sub>`api.ogt` line 11899</sub>

<a id="type-ControlSizes"></a>

### `ControlSizes` (struct)

The control, row and header heights and the gutter of the density in effect.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `controlSm` | `number` | `0` | — | `cssPx` | — | A small control's height. |
| `controlMd` | `number` | `0` | — | `cssPx` | — | A control's height. |
| `controlLg` | `number` | `0` | — | `cssPx` | — | A large control's height. |
| `gutter` | `number` | `0` | — | `cssPx` | — | The gutter. |
| `header` | `number` | `0` | — | `cssPx` | — | The header height. |
| `row` | `number` | `0` | — | `cssPx` | — | The row height. |
| `section` | `number` | `0` | — | `cssPx` | — | The section spacing. |

```ogt
struct ControlSizes {
  doc "The control, row and header heights and the gutter of the density in effect."
  field controlSm f32 "A small control's height." {
    unit cssPx
  }
  field controlMd f32 "A control's height." {
    unit cssPx
  }
  field controlLg f32 "A large control's height." {
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

<sub>`api.ogt` line 11913</sub>

<a id="type-UiTheme"></a>

### `UiTheme` (struct)

The style values the UI draws with, read-only.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `scheme` | `Scheme` | `"dark"` | — | — | — | The palette in use. |
| `palette` | `Readonly<Record<string, string>>` | `{}` | — | — | — | Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored). |
| `space` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels. |
| `radius` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The corner radii by name (small, control, bar, panel), in CSS pixels. |
| `typeRamp` | `Readonly<Record<string, TypeStep>>` | `{}` | — | — | — | Each TypeStepName's step. |
| `sizes` | `ControlSizes` | `{ controlSm: 0, controlMd: 0, controlLg: 0, gutter: 0, header: 0, row: 0, section: 0 }` | — | — | — | The control sizes of the density in effect. |
| `durationsMs` | `Readonly<Record<string, number>>` | `{}` | — | — | — | The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds. |
| `style` | `string` | `""` | — | — | — | The configured style's name; empty for the prototype style. |
| `contrast` | `Contrast` | `"standard"` | — | — | — | The variant in effect: high when the game asked for it or the page prefers more contrast. |
| `reducedMotion` | `boolean` | `false` | — | — | — | Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement. |
| `reducedTransparency` | `boolean` | `false` | — | — | — | Nothing is see-through: the scrim and the translucent fills of `palette` are already the opaque colours they make over panel (the scrim over backdrop) and glass is an opaque surface; the game asked for it or the page prefers reduced transparency. A game's own translucent overlays read it to draw opaque too. |
| `tokens` | `unknown` | `null` | — | — | — | The style's tokens (docs/ui.md §12.2), what a game's own drawing reads to match its UI: `typography` (each role, display title heading body label caption numeric, at each breakpoint: size, lineHeight, trackingEm, weight, font), the `space`, `size`, `radius` and `stroke` scales by step, `elevation` (each level's shadows: dx, dy, blur, spread, color), `shape` (each element class's corner and radius: the shape language), `springs` (ui, enter, exit, emphasis, playful: durationMs, bounce), `staggerMs`, `overshoot`, the `rarity`, `team` and `series` colours, and `feedback` (each UI event's set: how many sounds, the volume and pitch ranges, the haptic's [startMs, ms, low, high] pulses). |

```ogt
struct UiTheme {
  doc "The style values the UI draws with, read-only."
  field scheme Scheme "The palette in use."
  field palette Map.String "Each ThemeColor as `#rrggbb` or `#rrggbbaa` (sRGB as authored)."
  field space Map.f32 "The spacing scale by name (xxs, xs, sm, md, lg, xl, xxl, xl3), in CSS pixels."
  field radius Map.f32 "The corner radii by name (small, control, bar, panel), in CSS pixels."
  field typeRamp Map.TypeStep "Each TypeStepName's step."
  field sizes ControlSizes "The control sizes of the density in effect."
  field durationsMs Map.f32 "The motion durations by name (instant, fast, base, slow, spring, bounce), in milliseconds."
  field style String "The configured style's name; empty for the prototype style."
  field contrast Contrast "The variant in effect: high when the game asked for it or the page prefers more contrast."
  field reducedMotion bool "Transitions only fade and sheens stop: the game asked for it or the page prefers reduced motion. A game's own animations read it to keep their state changes and drop their movement."
  field reducedTransparency bool "Nothing is see-through: the scrim and the translucent fills of `palette` are already the opaque colours they make over panel (the scrim over backdrop) and glass is an opaque surface; the game asked for it or the page prefers reduced transparency. A game's own translucent overlays read it to draw opaque too."
  field tokens Json "The style's tokens (docs/ui.md §12.2), what a game's own drawing reads to match its UI: `typography` (each role, display title heading body label caption numeric, at each breakpoint: size, lineHeight, trackingEm, weight, font), the `space`, `size`, `radius` and `stroke` scales by step, `elevation` (each level's shadows: dx, dy, blur, spread, color), `shape` (each element class's corner and radius: the shape language), `springs` (ui, enter, exit, emphasis, playful: durationMs, bounce), `staggerMs`, `overshoot`, the `rarity`, `team` and `series` colours, and `feedback` (each UI event's set: how many sounds, the volume and pitch ranges, the haptic's [startMs, ms, low, high] pulses)."
}
```

<sub>`api.ogt` line 11938</sub>

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

<sub>`api.ogt` line 11954</sub>

<a id="type-RichTextDesc"></a>

### `RichTextDesc` (desc struct)

How a rich text paragraph draws: its defaults, which its markup's spans override.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
| `align` | `TextAlign` | `"start"` | — | — | — | How the lines align. |
| `nowrap` | `boolean` | `false` | — | — | — | Keep to one line (breaking only at `\n`). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits the text. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font. |

```ogt
struct RichTextDesc {
  doc "How a rich text paragraph draws: its defaults, which its markup's spans override."
  desc true
  field step Option.TypeStepName "The type step; null is body."
  field color Option.Paint "The ink; null is ink."
  field align TextAlign "How the lines align."
  field nowrap bool "Keep to one line (breaking only at `\\n`)."
  field width Option.Size "Its width; null fits the text."
  field font FontRole "Which font."
}
```

<sub>`api.ogt` line 12039</sub>

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

<sub>`api.ogt` line 12050</sub>

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

<sub>`api.ogt` line 12151</sub>

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

<sub>`api.ogt` line 12164</sub>

<a id="type-Tone"></a>

### `Tone` (enum)

A status tone of badges, toasts and map markers.

A string: one of the variants below. The zero value is `"neutral"`.

| Variant | Doc |
|---|---|
| `"neutral"` | Neutral: control fill and muted ink. |
| `"success"` | Success. |
| `"warn"` | Warning (orange). |
| `"danger"` | Danger. |
| `"info"` | Information. |
| `"primary"` | The game's identity: the primary's soft wash and highlight ink. |
| `"inverse"` | Neutral emphasis: ink fill, inverse ink. |

```ogt
enum Tone {
  doc "A status tone of badges, toasts and map markers."
  variant neutral "Neutral: control fill and muted ink."
  variant success "Success."
  variant warn "Warning (orange)."
  variant danger "Danger."
  variant info "Information."
  variant primary "The game's identity: the primary's soft wash and highlight ink."
  variant inverse "Neutral emphasis: ink fill, inverse ink."
}
```

<sub>`api.ogt` line 12175</sub>

<a id="type-MeterKind"></a>

### `MeterKind` (enum)

What a meter measures, which sets its colour.

A string: one of the variants below. The zero value is `"health"`.

| Variant | Doc |
|---|---|
| `"health"` | health; warn under 30 %, danger under 15 %. |
| `"mana"` | mana. |
| `"xp"` | xp. |
| `"primary"` | primary. |

```ogt
enum MeterKind {
  doc "What a meter measures, which sets its colour."
  variant health "health; warn under 30 %, danger under 15 %."
  variant mana "mana."
  variant xp "xp."
  variant primary "primary."
}
```

<sub>`api.ogt` line 12186</sub>

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

<sub>`api.ogt` line 12194</sub>

<a id="type-Contrast"></a>

### `Contrast` (enum)

The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour).

A string: one of the variants below. The zero value is `"standard"`.

| Variant | Doc |
|---|---|
| `"standard"` | The style's palette as it is. |
| `"high"` | Stronger inks, fills and edges. |

```ogt
enum Contrast {
  doc "The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour)."
  variant standard "The style's palette as it is."
  variant high "Stronger inks, fills and edges."
}
```

<sub>`api.ogt` line 12202</sub>

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

<sub>`api.ogt` line 12208</sub>

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

<sub>`api.ogt` line 12220</sub>

<a id="type-IconAtlasDesc"></a>

### `IconAtlasDesc` (desc struct)

The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `font` | `Font \| null` | `null` | — | — | — | An icon set from og.load.font (an `.ogt` of `type icons` naming SVGs, such as the UI kit's iconoir set): its icons are distance-field glyphs, crisp at any size and tinted by their ink, named by the set; texture, columns, rows and names are then not used. |
| `texture` | `Texture \| null` | `null` | — | — | — | The atlas texture, from og.load.texture. |
| `columns` | `number` | `1` | ≥ 1 | — | — | Cells across. |
| `rows` | `number` | `1` | ≥ 1 | — | — | Cells down. |
| `names` | `readonly string[]` | `[]` | — | — | — | Each cell's name, in reading order: og.ui.icon draws a cell by its name. |

```ogt
struct IconAtlasDesc {
  doc "The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font)."
  desc true
  field font Option.Font "An icon set from og.load.font (an `.ogt` of `type icons` naming SVGs, such as the UI kit's iconoir set): its icons are distance-field glyphs, crisp at any size and tinted by their ink, named by the set; texture, columns, rows and names are then not used."
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

<sub>`api.ogt` line 12236</sub>

<a id="type-AnchorDesc"></a>

### `AnchorDesc` (desc struct)

An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `anchor` | `Anchor` | `"topLeft"` | — | — | — | Where it sits. |
| `margin` | `number \| null` | `null` | — | — | — | The inset from the safe area's edges, the same on every edge: a margin wide enough to keep a top-left HUD over a centred portrait play field also pushes it that far down (use offsetX for the sideways shift, or a width); null is the density's gutter. |
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
| `frame` | `StylePart \| null` | `null` | — | — | — | The style frame its background takes; null is the panel's. |
| `instant` | `boolean` | `false` | — | — | — | Shown settled the first time it is built: no style entrance or stagger, its controls answering at once. For the screen a game opens on while it is that: the web draws it behind the play gate, where an entrance would stay part-way until the tap. |

```ogt
struct AnchorDesc {
  doc "An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls)."
  desc true
  field anchor Anchor "Where it sits."
  field margin Option.f32 "The inset from the safe area's edges, the same on every edge: a margin wide enough to keep a top-left HUD over a centred portrait play field also pushes it that far down (use offsetX for the sideways shift, or a width); null is the density's gutter."
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
  field frame Option.StylePart "The style frame its background takes; null is the panel's."
  field instant bool "Shown settled the first time it is built: no style entrance or stagger, its controls answering at once. For the screen a game opens on while it is that: the web draws it behind the play gate, where an entrance would stay part-way until the tap."
}
```

<sub>`api.ogt` line 12252</sub>

<a id="type-TransitionDesc"></a>

### `TransitionDesc` (desc struct)

How a transition moves, timed in ticks from the style's motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `TransitionKind` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long entering and leaving take; 0 is motion.duration.base (240 ms). |
| `distance` | `number` | `0` | ≥ 0 | `cssPx` | — | How far slides travel; 0 is space.xl. |
| `instant` | `boolean` | `false` | — | — | — | Shown at once the first time it is built (no entrance). |

```ogt
struct TransitionDesc {
  doc "How a transition moves, timed in ticks from the style's motion."
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

<sub>`api.ogt` line 12281</sub>

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

<sub>`api.ogt` line 12296</sub>

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

<sub>`api.ogt` line 12306</sub>

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

<sub>`api.ogt` line 12318</sub>

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

<sub>`api.ogt` line 12324</sub>

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
| `selected` | `number \| null` | `null` | — | — | — | The selected row (controlStrong), for og.ui.list. |

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
  field selected Option.u32 "The selected row (controlStrong), for og.ui.list."
}
```

<sub>`api.ogt` line 12333</sub>

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

<sub>`api.ogt` line 12350</sub>

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

<sub>`api.ogt` line 12356</sub>

<a id="type-GridDesc"></a>

### `GridDesc` (desc struct)

A virtualised grid of equal cells (inventories, level select), scrolling vertically.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many cells there are. |
| `cellWidth` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's width; 0 is 64 (never under the 44 px touch target). With columns set and a width other than fit, the cells share the grid's width instead, as wide as it allows (never under the touch target), and keep their height. |
| `cellHeight` | `number` | `0` | ≥ 0 | `cssPx` | — | A cell's height; 0 is its width. |
| `columns` | `number` | `0` | — | — | — | Cells across, exactly, whatever the width: a fit grid is that many cells of cellWidth, any other the cells share; 0 is as many cells of cellWidth as fit. |
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
  field cellWidth f32 "A cell's width; 0 is 64 (never under the 44 px touch target). With columns set and a width other than fit, the cells share the grid's width instead, as wide as it allows (never under the touch target), and keep their height." {
    unit cssPx
    min 0
  }
  field cellHeight f32 "A cell's height; 0 is its width." {
    unit cssPx
    min 0
  }
  field columns u32 "Cells across, exactly, whatever the width: a fit grid is that many cells of cellWidth, any other the cells share; 0 is as many cells of cellWidth as fit."
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

<sub>`api.ogt` line 12363</sub>

<a id="type-TabsDesc"></a>

### `TabsDesc` (desc struct)

A segmented control of tabs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fill` | `boolean` | `false` | — | — | — | Equal segments across the width. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null fits (grows with fill). |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the row of tabs (its accessible name), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct TabsDesc {
  doc "A segmented control of tabs."
  desc true
  field fill bool "Equal segments across the width."
  field width Option.Size "Its width; null fits (grows with fill)."
  field label String "What a screen reader calls the row of tabs (its accessible name), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 12386</sub>

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

<sub>`api.ogt` line 12394</sub>

<a id="type-SelectDesc"></a>

### `SelectDesc` (desc struct)

A select: a trigger that opens a menu of options.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `placeholder` | `string` | `""` | — | — | — | Shown while nothing is chosen. |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null grows from 160. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |
| `label` | `string` | `""` | — | — | — | What a screen reader calls it (its accessible name; the chosen option is its value), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct SelectDesc {
  doc "A select: a trigger that opens a menu of options."
  desc true
  field placeholder String "Shown while nothing is chosen."
  field width Option.Size "Its width; null grows from 160."
  field disabled bool "Neutral and inert."
  field label String "What a screen reader calls it (its accessible name; the chosen option is its value), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 12400</sub>

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

<sub>`api.ogt` line 12409</sub>

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

<sub>`api.ogt` line 12422</sub>

<a id="type-RadioDesc"></a>

### `RadioDesc` (desc struct)

How a radio group lays out, and its name.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `horizontal` | `boolean` | `false` | — | — | — | Options in a row (the default is a column). |
| `label` | `string` | `""` | — | — | — | What a screen reader calls the group (its accessible name), never drawn; empty is its id. A localised game passes its string table's. |

```ogt
struct RadioDesc {
  doc "How a radio group lays out, and its name."
  desc true
  field horizontal bool "Options in a row (the default is a column)."
  field label String "What a screen reader calls the group (its accessible name), never drawn; empty is its id. A localised game passes its string table's."
}
```

<sub>`api.ogt` line 12428</sub>

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

<sub>`api.ogt` line 12435</sub>

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

<sub>`api.ogt` line 12441</sub>

<a id="type-ChipDesc"></a>

### `ChipDesc` (desc struct)

A chip's state.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `selected` | `boolean` | `false` | — | — | — | Selected: inverted to ink. |
| `disabled` | `boolean` | `false` | — | — | — | Neutral and inert. |

```ogt
struct ChipDesc {
  doc "A chip's state."
  desc true
  field selected bool "Selected: inverted to ink."
  field disabled bool "Neutral and inert."
}
```

<sub>`api.ogt` line 12451</sub>

<a id="type-MeterDesc"></a>

### `MeterDesc` (desc struct)

A meter: a track with a fill that animates its changes by the tick.

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
  doc "A meter: a track with a fill that animates its changes by the tick."
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

<sub>`api.ogt` line 12458</sub>

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

<sub>`api.ogt` line 12472</sub>

<a id="type-IconDesc"></a>

### `IconDesc` (desc struct)

An icon's size, ink and name.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its size; 0 is 24. |
| `tint` | `Paint \| null` | `null` | — | — | — | Its ink; null is ink. |
| `label` | `string \| null` | `null` | — | — | — | What a screen reader calls it (and the probe's text): null is the icon's name in the atlas (an English word: `star`); empty is decoration, named by nothing, so a card or button it sits on is named by its other text. A localised game names a meaningful icon from its string table. |

```ogt
struct IconDesc {
  doc "An icon's size, ink and name."
  desc true
  field size f32 "Its size; 0 is 24." {
    unit cssPx
    min 0
  }
  field tint Option.Paint "Its ink; null is ink."
  field label Option.String "What a screen reader calls it (and the probe's text): null is the icon's name in the atlas (an English word: `star`); empty is decoration, named by nothing, so a card or button it sits on is named by its other text. A localised game names a meaningful icon from its string table."
}
```

<sub>`api.ogt` line 12483</sub>

<a id="type-MapMarker"></a>

### `MapMarker` (desc struct)

A marker on a mini-map.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `ratio` | — | Across the map, 0 to 1. |
| `y` | `number` | `0` | — | `ratio` | — | Down the map, 0 to 1. |
| `tone` | `Tone` | `"neutral"` | — | — | — | Its colour; neutral is the stage ink. |
| `player` | `boolean` | `false` | — | — | — | The player: the primary, larger, ringed. |

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
  field player bool "The player: the primary, larger, ringed."
}
```

<sub>`api.ogt` line 12494</sub>

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

<sub>`api.ogt` line 12507</sub>

<a id="type-DialogueDesc"></a>

### `DialogueDesc` (desc struct)

A dialogue box: a portrait slot, the speaker and the line typed out by the tick.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `speaker` | `string` | `""` | — | — | — | Who speaks (primary ink). |
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
  field speaker String "Who speaks (primary ink)."
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

<sub>`api.ogt` line 12520</sub>

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

<sub>`api.ogt` line 12534</sub>

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

<sub>`api.ogt` line 12541</sub>

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

<sub>`api.ogt` line 12555</sub>

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

<sub>`api.ogt` line 12562</sub>

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

<sub>`api.ogt` line 12571</sub>

<a id="type-ActionButtonDesc"></a>

### `ActionButtonDesc` (desc struct)

A square touch action button (jump, fire), cornered by the style's control radius.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `size` | `number` | `0` | ≥ 0 | `cssPx` | — | Its side; 0 is 64 (never under 44). |
| `primary` | `boolean` | `false` | — | — | — | The primary button (one per screen). |
| `icon` | `string` | `""` | — | — | — | An icon of the atlas by name, instead of the label; empty is the label. |

```ogt
struct ActionButtonDesc {
  doc "A square touch action button (jump, fire), cornered by the style's control radius."
  desc true
  field size f32 "Its side; 0 is 64 (never under 44)." {
    unit cssPx
    min 0
  }
  field primary bool "The primary button (one per screen)."
  field icon String "An icon of the atlas by name, instead of the label; empty is the label."
}
```

<sub>`api.ogt` line 12579</sub>

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

<sub>`api.ogt` line 12590</sub>

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

<sub>`api.ogt` line 12601</sub>

<a id="type-A11yNode"></a>

### `A11yNode` (struct)

One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its path, as touch_move_to and og.ui.focus name it. |
| `parent` | `string` | `""` | — | — | — | The path of the nearest node above it in the tree; empty at the top. |
| `role` | `string` | `""` | — | — | — | Its role (probe.UiElement.role): button, link, text, heading, image, checkbox, radio, switch, slider, spinbutton, textbox, combobox, listbox, option, tab, tablist, radiogroup, progressbar, meter, timer, status, dialog, tooltip, separator, group. |
| `name` | `string` | `""` | — | — | — | Its accessible name: what a screen reader says it is. A text field or area is named by the label before it in its row, else by its placeholder, else by its id; never by the text in it (that is its value). |
| `description` | `string` | `""` | — | — | — | Read after the name (aria-description): a text field's placeholder when a label names it; empty for none. |
| `value` | `string` | `""` | — | — | — | Its value as text (a slider's number, a progress's percent, a field's text); empty for none. |
| `states` | `readonly string[]` | `[]` | — | — | — | Its states: disabled, focused, focusable, checked, selected, expanded, pressed, busy, offscreen (a scroll container cuts it out of view). |
| `valueMin` | `number \| null` | `null` | — | — | — | A range's smallest value (aria-valuemin): a slider's min, a bounded stepper's, 0 for a progress (in percent) and a meter; null for a stepper without bounds and a control that is no range. |
| `valueMax` | `number \| null` | `null` | — | — | — | A range's largest value (aria-valuemax): a slider's max, a bounded stepper's, 100 for a progress, a meter's max; null as valueMin is. |
| `valueNow` | `number \| null` | `null` | — | — | — | A range's value as a number (aria-valuenow; value says it as people read it): a slider's, a stepper's, a meter's, a progress's percent; null for a control that is no range. |
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
  field name String "Its accessible name: what a screen reader says it is. A text field or area is named by the label before it in its row, else by its placeholder, else by its id; never by the text in it (that is its value)."
  field description String "Read after the name (aria-description): a text field's placeholder when a label names it; empty for none."
  field value String "Its value as text (a slider's number, a progress's percent, a field's text); empty for none."
  field states List.String "Its states: disabled, focused, focusable, checked, selected, expanded, pressed, busy, offscreen (a scroll container cuts it out of view)."
  field valueMin Option.f32 "A range's smallest value (aria-valuemin): a slider's min, a bounded stepper's, 0 for a progress (in percent) and a meter; null for a stepper without bounds and a control that is no range."
  field valueMax Option.f32 "A range's largest value (aria-valuemax): a slider's max, a bounded stepper's, 100 for a progress, a meter's max; null as valueMin is."
  field valueNow Option.f32 "A range's value as a number (aria-valuenow; value says it as people read it): a slider's, a stepper's, a meter's, a progress's percent; null for a control that is no range."
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

<sub>`api.ogt` line 12964</sub>

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

<sub>`api.ogt` line 12990</sub>

<a id="type-CounterDesc"></a>

### `CounterDesc` (desc struct)

A number that counts to its value.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `step` | `TypeStepName \| null` | `null` | — | — | — | The type step; null is body. |
| `font` | `FontRole` | `"body"` | — | — | — | Which font: mono (the numeric role) for counts in a row, display for a big score. |
| `color` | `Paint \| null` | `null` | — | — | — | The ink; null is ink. |
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
  field color Option.Paint "The ink; null is ink."
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

<sub>`api.ogt` line 13040</sub>

<a id="type-BurstDesc"></a>

### `BurstDesc` (desc struct)

Rays and sparks thrown from an element.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | Rays; 0 is 8. |
| `color` | `Paint \| null` | `null` | — | — | — | Their ink; null is the primary. |
| `radius` | `number` | `0` | ≥ 0 | `cssPx` | — | How far they fly from the element's centre; 0 is 1.4 times its size. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long they fly; 0 is motion.duration.slow plus base. |
| `delayMs` | `number` | `0` | ≥ 0 | `ms` | — | Wait this long after the element first appears. |

```ogt
struct BurstDesc {
  doc "Rays and sparks thrown from an element."
  desc true
  field count u32 "Rays; 0 is 8."
  field color Option.Paint "Their ink; null is the primary."
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

<sub>`api.ogt` line 13066</sub>

<a id="type-ShakeDesc"></a>

### `ShakeDesc` (desc struct)

A shake of an element: three sideways swings decaying to rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `amount` | `number` | `0` | ≥ 0 | `cssPx` | — | The first swing; 0 is the style's motion.shake, else 6. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | How long it takes to come to rest; 0 is motion.duration.slow. |
| `sound` | `boolean` | `false` | — | — | — | Also play the style's error sound, as a refused press does. |

```ogt
struct ShakeDesc {
  doc "A shake of an element: three sideways swings decaying to rest."
  desc true
  field amount f32 "The first swing; 0 is the style's motion.shake, else 6." {
    unit cssPx
    min 0
  }
  field durationMs f32 "How long it takes to come to rest; 0 is motion.duration.slow." {
    unit ms
    min 0
  }
  field sound bool "Also play the style's error sound, as a refused press does."
}
```

<sub>`api.ogt` line 13091</sub>

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

<sub>`api.ogt` line 13111</sub>

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

<sub>`api.ogt` line 13120</sub>

<a id="type-ScreenTransitionDesc"></a>

### `ScreenTransitionDesc` (desc struct)

How a screen transition looks.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ScreenWipe` | `"fade"` | — | — | — | The movement. |
| `durationMs` | `number` | `0` | ≥ 0 | `ms` | — | Cover and reveal together; 0 is twice motion.duration.slow. |
| `color` | `Paint \| null` | `null` | — | — | — | The cover's colour (the texture's tint); null is backdrop. |
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
  field color Option.Paint "The cover's colour (the texture's tint); null is backdrop."
  field texture Option.Texture "A texture stretched over the surface as the cover; null is the flat colour."
  field from WipeSide "Where a wipe or a slide comes from."
}
```

<sub>`api.ogt` line 13128</sub>

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

<sub>`api.ogt` line 13141</sub>

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

<sub>`api.ogt` line 13168</sub>

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

<sub>`api.ogt` line 13173</sub>

<a id="type-MenuItem"></a>

### `MenuItem` (desc struct)

An item of a menu: an id the game acts on and the label a player reads.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `id` | `string` | `""` | — | — | — | The id answered when it is chosen; also its probe id. |
| `label` | `string` | `""` | — | — | — | What it says. |
| `disabled` | `boolean` | `false` | — | — | — | Shown but inert. |
| `primary` | `boolean` | `false` | — | — | — | The primary action (one per screen). |

```ogt
struct MenuItem {
  doc "An item of a menu: an id the game acts on and the label a player reads."
  desc true
  field id String "The id answered when it is chosen; also its probe id."
  field label String "What it says."
  field disabled bool "Shown but inert."
  field primary bool "The primary action (one per screen)."
}
```

<sub>`api.ogt` line 13178</sub>

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

<sub>`api.ogt` line 13187</sub>

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
| `instant` | `boolean` | `false` | — | — | — | Shown settled the first time it is built (AnchorDesc.instant): pass it while the title is the screen the game opens on, so the frame behind the web's play gate is the whole title; a title the player comes back to enters with the style. |

```ogt
struct MainMenuDesc {
  doc "A title screen: the title, a subtitle, a column of items (the first primary unless one says so) and a footer. Centred on desktop and tablet, bottom-weighted on a phone."
  desc true
  field title String "The game's title, in display type."
  field subtitle String "A line under it; empty is none."
  field items List.MenuItem "The items, such as Play, Continue, Settings, Quit."
  field footer String "A line at the bottom (a version, credits); empty is none."
  field onSelect Option.UiAction "Called with the chosen item's id."
  field instant bool "Shown settled the first time it is built (AnchorDesc.instant): pass it while the title is the screen the game opens on, so the frame behind the web's play gate is the whole title; a title the player comes back to enters with the style."
}
```

<sub>`api.ogt` line 13192</sub>

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

<sub>`api.ogt` line 13203</sub>

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

<sub>`api.ogt` line 13214</sub>

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

<sub>`api.ogt` line 13222</sub>

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

<sub>`api.ogt` line 13229</sub>

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
| `world` | `World \| null` | `null` | — | — | — | The world the graphics preset is drawn in: when the player picks another preset the screen calls og.render.applyQuality(world, preset) before onChange. Null leaves applying it to the game. Inside a system, the system declares RenderSettings among its writes. |
| `title` | `string` | `"Settings"` | — | — | — | The dialog's title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `audioLabel` | `string` | `"Audio"` | — | — | — | The Audio tab. |
| `graphicsLabel` | `string` | `"Graphics"` | — | — | — | The Graphics tab. |
| `controlsLabel` | `string` | `"Controls"` | — | — | — | The Controls tab. |
| `languageLabel` | `string` | `"Language"` | — | — | — | The Language tab. |
| `masterLabel` | `string` | `"Master"` | — | — | — | The master volume's slider. |
| `musicLabel` | `string` | `"Music"` | — | — | — | The music volume's slider. |
| `effectsLabel` | `string` | `"Effects"` | — | — | — | The effects volume's slider. |
| `muteLabel` | `string` | `"Mute everything"` | — | — | — | The mute toggle. |
| `qualityLabel` | `string` | `"Quality"` | — | — | — | The graphics preset's heading, also the presets' group's accessible name. |
| `lowLabel` | `string` | `"Low"` | — | — | — | The low preset. |
| `mediumLabel` | `string` | `"Medium"` | — | — | — | The medium preset. |
| `highLabel` | `string` | `"High"` | — | — | — | The high preset. |
| `fpsLabel` | `string` | `"Show frame rate"` | — | — | — | The frame-rate toggle. |
| `motionLabel` | `string` | `"Reduce motion"` | — | — | — | The reduced-motion toggle. |
| `contrastLabel` | `string` | `"High contrast"` | — | — | — | The high-contrast toggle. |
| `sensitivityLabel` | `string` | `"Sensitivity"` | — | — | — | The look sensitivity's slider. |
| `invertLabel` | `string` | `"Invert vertical look"` | — | — | — | The invert-Y checkbox. |
| `vibrationLabel` | `string` | `"Vibration"` | — | — | — | The vibration checkbox. |
| `doneLabel` | `string` | `"Done"` | — | — | — | The button that closes the screen. |
| `sectionLabel` | `string` | `"Section"` | — | — | — | What a screen reader calls the row of tabs (the select on a phone): the group's accessible name, never drawn. |
| `languagesLabel` | `string` | `"Languages"` | — | — | — | What a screen reader calls the list of languages: the group's accessible name, never drawn. |

```ogt
struct SettingsDesc {
  doc "A settings screen: Audio, Graphics, Controls and Language tabs (a select on a phone) over the game's settings, with persistence hooks."
  desc true
  field open bool "Whether it is shown."
  field settings GameSettings "The settings from game state."
  field languages List.LanguageOption "The languages offered; empty hides the Language tab."
  field onChange Option.SettingsChanged "Called on every change (apply it at once: volume, contrast)."
  field onSave Option.SettingsChanged "Called when the screen closes after changes (persist here)."
  field world Option.World "The world the graphics preset is drawn in: when the player picks another preset the screen calls og.render.applyQuality(world, preset) before onChange. Null leaves applying it to the game. Inside a system, the system declares RenderSettings among its writes."
  field title String "The dialog's title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "Settings"
  }
  field audioLabel String "The Audio tab." {
    default "Audio"
  }
  field graphicsLabel String "The Graphics tab." {
    default "Graphics"
  }
  field controlsLabel String "The Controls tab." {
    default "Controls"
  }
  field languageLabel String "The Language tab." {
    default "Language"
  }
  field masterLabel String "The master volume's slider." {
    default "Master"
  }
  field musicLabel String "The music volume's slider." {
    default "Music"
  }
  field effectsLabel String "The effects volume's slider." {
    default "Effects"
  }
  field muteLabel String "The mute toggle." {
    default "Mute everything"
  }
  field qualityLabel String "The graphics preset's heading, also the presets' group's accessible name." {
    default "Quality"
  }
  field lowLabel String "The low preset." {
    default "Low"
  }
  field mediumLabel String "The medium preset." {
    default "Medium"
  }
  field highLabel String "The high preset." {
    default "High"
  }
  field fpsLabel String "The frame-rate toggle." {
    default "Show frame rate"
  }
  field motionLabel String "The reduced-motion toggle." {
    default "Reduce motion"
  }
  field contrastLabel String "The high-contrast toggle." {
    default "High contrast"
  }
  field sensitivityLabel String "The look sensitivity's slider." {
    default "Sensitivity"
  }
  field invertLabel String "The invert-Y checkbox." {
    default "Invert vertical look"
  }
  field vibrationLabel String "The vibration checkbox." {
    default "Vibration"
  }
  field doneLabel String "The button that closes the screen." {
    default "Done"
  }
  field sectionLabel String "What a screen reader calls the row of tabs (the select on a phone): the group's accessible name, never drawn." {
    default "Section"
  }
  field languagesLabel String "What a screen reader calls the list of languages: the group's accessible name, never drawn." {
    default "Languages"
  }
}
```

<sub>`api.ogt` line 13270</sub>

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

<sub>`api.ogt` line 13347</sub>

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

<sub>`api.ogt` line 13354</sub>

<a id="type-PlatformerHudDesc"></a>

### `PlatformerHudDesc` (desc struct)

A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button. What a game leaves out it does not draw: no hearts with maxLives 0 (and no health), no coins or score capsule without coins or score.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `lives` | `number` | `0` | — | — | — | Lives left. |
| `maxLives` | `number` | `3` | — | — | — | Lives at most (hearts drawn); 0 draws no hearts and no capsule for them (a game without lives). |
| `coins` | `number \| null` | `null` | — | — | — | Coins collected; null or left out draws no coins capsule. |
| `score` | `number \| null` | `null` | — | — | — | The score; null or left out draws no score capsule. |
| `level` | `string` | `""` | — | — | — | The level's name; empty is none. |
| `timeLeft` | `number \| null` | `null` | — | — | — | Seconds left; null hides the timer. |
| `health` | `number \| null` | `null` | — | — | — | A health bar instead of hearts, 0 to 1; null shows hearts. |
| `touch` | `boolean` | `false` | — | — | — | Show touch controls (a d-pad and Jump) on phones and tablets. |
| `jumpLabel` | `string` | `"Jump"` | — | — | — | The touch Jump button. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `pauseLabel` | `string` | `"Pause"` | — | — | — | The pause button's name (what a screen reader says; the button shows its icon). |

```ogt
struct PlatformerHudDesc {
  doc "A platformer HUD: lives as hearts and a level name top left, coins and score top right, a timer, a pause button. What a game leaves out it does not draw: no hearts with maxLives 0 (and no health), no coins or score capsule without coins or score."
  desc true
  field lives u32 "Lives left."
  field maxLives u32 "Lives at most (hearts drawn); 0 draws no hearts and no capsule for them (a game without lives)." {
    default 3
  }
  field coins Option.u32 "Coins collected; null or left out draws no coins capsule."
  field score Option.u32 "The score; null or left out draws no score capsule."
  field level String "The level's name; empty is none."
  field timeLeft Option.f32 "Seconds left; null hides the timer."
  field health Option.f32 "A health bar instead of hearts, 0 to 1; null shows hearts."
  field touch bool "Show touch controls (a d-pad and Jump) on phones and tablets."
  field jumpLabel String "The touch Jump button. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "Jump"
  }
  field pauseLabel String "The pause button's name (what a screen reader says; the button shows its icon)." {
    default "Pause"
  }
}
```

<sub>`api.ogt` line 13365</sub>

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

<sub>`api.ogt` line 13386</sub>

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
| `killsLabel` | `string` | `"kills"` | — | — | — | The word after the kill count. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `fireLabel` | `string` | `"Fire"` | — | — | — | The touch Fire button. |
| `pauseLabel` | `string` | `"Pause"` | — | — | — | The pause button's name (what a screen reader says; the button shows its icon). |

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
  field killsLabel String "The word after the kill count. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "kills"
  }
  field fireLabel String "The touch Fire button." {
    default "Fire"
  }
  field pauseLabel String "The pause button's name (what a screen reader says; the button shows its icon)." {
    default "Pause"
  }
}
```

<sub>`api.ogt` line 13395</sub>

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
| `movesLabel` | `string` | `"moves left"` | — | — | — | The caption under the moves left. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `targetLabel` | `string` | `"Target {target}"` | — | — | — | The line beside the stars; `{target}` is replaced by the target, formatted (1,000). A localised game passes its string table's, with `{target}` in it. |
| `pauseLabel` | `string` | `"Pause"` | — | — | — | The pause button's name (what a screen reader says; the button shows its icon). |

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
  field movesLabel String "The caption under the moves left. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "moves left"
  }
  field targetLabel String "The line beside the stars; `{target}` is replaced by the target, formatted (1,000). A localised game passes its string table's, with `{target}` in it." {
    default "Target {target}"
  }
  field pauseLabel String "The pause button's name (what a screen reader says; the button shows its icon)." {
    default "Pause"
  }
}
```

<sub>`api.ogt` line 13426</sub>

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

<sub>`api.ogt` line 13448</sub>

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

<sub>`api.ogt` line 13458</sub>

<a id="type-IdleHudDesc"></a>

### `IdleHudDesc` (desc struct)

An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `currencies` | `readonly Currency[]` | `[]` | — | — | — | The currencies, the first the main one. |
| `generators` | `readonly Generator[]` | `[]` | — | — | — | The generators. |
| `tapLabel` | `string` | `"Tap"` | — | — | — | The tap target's label; empty hides it. |
| `levelLabel` | `string` | `"Lv {level}"` | — | — | — | A generator's level badge; `{level}` is replaced by its level. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t), with `{level}` in it. |
| `rateLabel` | `string` | `"+{rate}/s"` | — | — | — | A currency's rate; `{rate}` is replaced by its per-second amount, formatted (+12/s is `+{rate}/s`). A localised game passes its string table's, with `{rate}` in it. |
| `pauseLabel` | `string` | `"Pause"` | — | — | — | The pause button's name (what a screen reader says; the button shows its icon). |

```ogt
struct IdleHudDesc {
  doc "An idle or incremental HUD: currencies with their rates on top, a big tap target, the generators in a list."
  desc true
  field currencies List.Currency "The currencies, the first the main one."
  field generators List.Generator "The generators."
  field tapLabel String "The tap target's label; empty hides it." {
    default "Tap"
  }
  field levelLabel String "A generator's level badge; `{level}` is replaced by its level. Every word of the HUD is a field with its English default: a localised game passes its string table's (og.i18n.t), with `{level}` in it." {
    default "Lv {level}"
  }
  field rateLabel String "A currency's rate; `{rate}` is replaced by its per-second amount, formatted (+12/s is `+{rate}/s`). A localised game passes its string table's, with `{rate}` in it." {
    default "+{rate}/s"
  }
  field pauseLabel String "The pause button's name (what a screen reader says; the button shows its icon)." {
    default "Pause"
  }
}
```

<sub>`api.ogt` line 13472</sub>

<a id="type-RaceSplit"></a>

### `RaceSplit` (desc struct)

One split of a racing HUD: a sector's time and its delta against the reference (the record's split).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `label` | `string` | `""` | — | — | — | Its name (`S1`, a sector's), the game's own word. |
| `time` | `number \| null` | `null` | — | — | — | Its time in seconds, shown as m:ss.mmm; null is not yet run (a dash). |
| `delta` | `number \| null` | `null` | — | — | — | Against the reference in seconds, shown signed (`-0.412`, `+1.050`, with the ASCII minus every pack draws): negative is ahead, in success ink; positive behind, in danger ink; null shows none. |

```ogt
struct RaceSplit {
  doc "One split of a racing HUD: a sector's time and its delta against the reference (the record's split)."
  desc true
  field label String "Its name (`S1`, a sector's), the game's own word."
  field time Option.f32 "Its time in seconds, shown as m:ss.mmm; null is not yet run (a dash)."
  field delta Option.f32 "Against the reference in seconds, shown signed (`-0.412`, `+1.050`, with the ASCII minus every pack draws): negative is ahead, in success ink; positive behind, in danger ink; null shows none."
}
```

<sub>`api.ogt` line 13491</sub>

<a id="type-RacingHudDesc"></a>

### `RacingHudDesc` (desc struct)

A racing HUD: position and lap top left, the times top right (the race time, a live delta, the best, the splits), the speedometer and gear bottom right, the boost meter, an optional mini-map. A time trial sets racers 0 (no place) and boostMeter false (no meter), and shows its sectors as splits.

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
| `racers` | `number` | `8` | — | — | — | Cars in the race; 0 hides the place (a time trial). |
| `time` | `number` | `0` | — | — | — | The race time in seconds. |
| `delta` | `number \| null` | `null` | — | — | — | A live delta in seconds against the reference lap, under the time, signed like a split's: negative (ahead) in success ink, positive in danger ink; null hides it. |
| `best` | `number \| null` | `null` | — | — | — | The best lap in seconds; null hides it. |
| `splits` | `readonly RaceSplit[]` | `[]` | — | — | — | Splits under the times (a time trial's sectors), one row each: label, time, delta; empty is none. |
| `boost` | `number` | `0` | — | `ratio` | — | The boost meter, 0 to 1. |
| `boostMeter` | `boolean` | `true` | — | — | — | Draw the boost meter and the touch Boost pedal; a race without boost (a time trial) sets false. |
| `map` | `MinimapDesc \| null` | `null` | — | — | — | A mini-map; null is none. |
| `touch` | `boolean` | `false` | — | — | — | Show touch controls (steering arrows, Brake, Boost) on phones and tablets. |
| `lapLabel` | `string` | `"lap"` | — | — | — | The caption under the lap. Every word of the HUD is a field with its English default (unit too): a localised game passes its string table's (og.i18n.t). |
| `bestLabel` | `string` | `"Best {best}"` | — | — | — | The best lap's line; `{best}` is replaced by the best lap as m:ss.mmm. A localised game passes its string table's, with `{best}` in it. |
| `pauseLabel` | `string` | `"Pause"` | — | — | — | The pause button's name (what a screen reader says; the button shows its icon). |
| `boostLabel` | `string` | `"Boost"` | — | — | — | The boost meter's label and the touch Boost pedal. |
| `brakeLabel` | `string` | `"Brake"` | — | — | — | The touch Brake pedal. |
| `leftLabel` | `string` | `"Left"` | — | — | — | The touch steering arrow left. |
| `rightLabel` | `string` | `"Right"` | — | — | — | The touch steering arrow right. |

```ogt
struct RacingHudDesc {
  doc "A racing HUD: position and lap top left, the times top right (the race time, a live delta, the best, the splits), the speedometer and gear bottom right, the boost meter, an optional mini-map. A time trial sets racers 0 (no place) and boostMeter false (no meter), and shows its sectors as splits."
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
  field racers u32 "Cars in the race; 0 hides the place (a time trial)." {
    default 8
  }
  field time f32 "The race time in seconds."
  field delta Option.f32 "A live delta in seconds against the reference lap, under the time, signed like a split's: negative (ahead) in success ink, positive in danger ink; null hides it."
  field best Option.f32 "The best lap in seconds; null hides it."
  field splits List.RaceSplit "Splits under the times (a time trial's sectors), one row each: label, time, delta; empty is none."
  field boost f32 "The boost meter, 0 to 1." {
    unit ratio
  }
  field boostMeter bool "Draw the boost meter and the touch Boost pedal; a race without boost (a time trial) sets false." {
    default true
  }
  field map Option.MinimapDesc "A mini-map; null is none."
  field touch bool "Show touch controls (steering arrows, Brake, Boost) on phones and tablets."
  field lapLabel String "The caption under the lap. Every word of the HUD is a field with its English default (unit too): a localised game passes its string table's (og.i18n.t)." {
    default "lap"
  }
  field bestLabel String "The best lap's line; `{best}` is replaced by the best lap as m:ss.mmm. A localised game passes its string table's, with `{best}` in it." {
    default "Best {best}"
  }
  field pauseLabel String "The pause button's name (what a screen reader says; the button shows its icon)." {
    default "Pause"
  }
  field boostLabel String "The boost meter's label and the touch Boost pedal." {
    default "Boost"
  }
  field brakeLabel String "The touch Brake pedal." {
    default "Brake"
  }
  field leftLabel String "The touch steering arrow left." {
    default "Left"
  }
  field rightLabel String "The touch steering arrow right." {
    default "Right"
  }
}
```

<sub>`api.ogt` line 13499</sub>

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

<sub>`api.ogt` line 13557</sub>

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
| `skipLabel` | `string` | `"Skip"` | — | — | — | The Skip button. Every word of the cutscene is a field with its English default: a localised game passes its string table's (og.i18n.t). |

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
  field skipLabel String "The Skip button. Every word of the cutscene is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "Skip"
  }
}
```

<sub>`api.ogt` line 13565</sub>

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

<sub>`api.ogt` line 13582</sub>

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

<sub>`api.ogt` line 13589</sub>

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
| `priceLabel` | `string` | `"Price"` | — | — | — | The detail panel's price line. Every word of the shop is a field with its English default (title and currency too): a localised game passes its string table's (og.i18n.t). |
| `buyLabel` | `string` | `"Buy"` | — | — | — | The Buy button. |
| `shortLabel` | `string` | `"Not enough"` | — | — | — | The Buy button while the balance is short of the price. |
| `ownedLabel` | `string` | `"Owned"` | — | — | — | An owned item's badge and button. |
| `emptyLabel` | `string` | `"Nothing for sale here"` | — | — | — | The detail panel when the category has no items. |

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
  field priceLabel String "The detail panel's price line. Every word of the shop is a field with its English default (title and currency too): a localised game passes its string table's (og.i18n.t)." {
    default "Price"
  }
  field buyLabel String "The Buy button." {
    default "Buy"
  }
  field shortLabel String "The Buy button while the balance is short of the price." {
    default "Not enough"
  }
  field ownedLabel String "An owned item's badge and button." {
    default "Owned"
  }
  field emptyLabel String "The detail panel when the category has no items." {
    default "Nothing for sale here"
  }
}
```

<sub>`api.ogt` line 13601</sub>

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

<sub>`api.ogt` line 13632</sub>

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

<sub>`api.ogt` line 13639</sub>

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
| `useLabel` | `string` | `"Use"` | — | — | — | The detail panel's action; empty hides it. Every word of the inventory is a field with its English default (title too): a localised game passes its string table's (og.i18n.t). |
| `hintLabel` | `string` | `"Select an item. Drag it to another slot to move it."` | — | — | — | The detail panel while no slot is selected. |
| `countLabel` | `string` | `"Count"` | — | — | — | The detail panel's count line, beside a stacked item's count. |

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
  field useLabel String "The detail panel's action; empty hides it. Every word of the inventory is a field with its English default (title too): a localised game passes its string table's (og.i18n.t)." {
    default "Use"
  }
  field hintLabel String "The detail panel while no slot is selected." {
    default "Select an item. Drag it to another slot to move it."
  }
  field countLabel String "The detail panel's count line, beside a stacked item's count." {
    default "Count"
  }
}
```

<sub>`api.ogt` line 13650</sub>

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

<sub>`api.ogt` line 13672</sub>

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

<sub>`api.ogt` line 13678</sub>

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

<sub>`api.ogt` line 13686</sub>

<a id="type-LevelSelectDesc"></a>

### `LevelSelectDesc` (desc struct)

A level select: a grid of level cards with stars and locks, the total stars, and a Back button. Back closes it from every input: the button, a press on the scrim around it, Escape and the pad's B.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `title` | `string` | `"Levels"` | — | — | — | Its title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `levels` | `readonly LevelInfo[]` | `[]` | — | — | — | The levels. |
| `columns` | `number` | `0` | — | — | — | Cards across; 0 fits the width. |
| `backLabel` | `string` | `"Back"` | — | — | — | The button that closes it. |
| `starsLabel` | `string` | `"{stars} of {max} stars"` | — | — | — | What a screen reader says for a card's stars (and the total's): `{stars}` is replaced by the stars earned, `{max}` by the most there are. The stars themselves are drawn, never this text. |
| `lockedLabel` | `string` | `"{name}, locked"` | — | — | — | What a screen reader calls a locked card (it shows only a lock): `{name}` is replaced by the level's name or number. |

```ogt
struct LevelSelectDesc {
  doc "A level select: a grid of level cards with stars and locks, the total stars, and a Back button. Back closes it from every input: the button, a press on the scrim around it, Escape and the pad's B."
  desc true
  field title String "Its title. Every word of the screen is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "Levels"
  }
  field levels List.LevelInfo "The levels."
  field columns u32 "Cards across; 0 fits the width."
  field backLabel String "The button that closes it." {
    default "Back"
  }
  field starsLabel String "What a screen reader says for a card's stars (and the total's): `{stars}` is replaced by the stars earned, `{max}` by the most there are. The stars themselves are drawn, never this text." {
    default "{stars} of {max} stars"
  }
  field lockedLabel String "What a screen reader calls a locked card (it shows only a lock): `{name}` is replaced by the level's name or number." {
    default "{name}, locked"
  }
}
```

<sub>`api.ogt` line 13695</sub>

<a id="type-LevelSelectResult"></a>

### `LevelSelectResult` (struct)

A level select's answer this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `chosen` | `number \| null` | `null` | — | — | — | The level picked this tick. |
| `closed` | `boolean` | `false` | — | — | — | The player asked to close it: its Back button, a press on the scrim, Escape or the pad's B. |

```ogt
struct LevelSelectResult {
  doc "A level select's answer this tick."
  field chosen Option.u32 "The level picked this tick."
  field closed bool "The player asked to close it: its Back button, a press on the scrim, Escape or the pad's B."
}
```

<sub>`api.ogt` line 13714</sub>

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

<sub>`api.ogt` line 13720</sub>

<a id="type-ScoreFormat"></a>

### `ScoreFormat` (enum)

How a results screen shows its score and best.

A string: one of the variants below. The zero value is `"points"`.

| Variant | Doc |
|---|---|
| `"points"` | A whole number grouped with commas (12,840), counted up from 0; the best is the higher. |
| `"time"` | Seconds as m:ss.mmm (1:36.233), counted up from 0:00.000: a time trial's result. Lower is better: the best is the lower, and a best of 0 is none (no best line). |

```ogt
enum ScoreFormat {
  doc "How a results screen shows its score and best."
  variant points "A whole number grouped with commas (12,840), counted up from 0; the best is the higher."
  variant time "Seconds as m:ss.mmm (1:36.233), counted up from 0:00.000: a time trial's result. Lower is better: the best is the lower, and a best of 0 is none (no best line)."
}
```

<sub>`api.ogt` line 13727</sub>

<a id="type-ResultsDesc"></a>

### `ResultsDesc` (desc struct)

A results screen: the title, stars that land one by one, the score counting up (points, or a time with `format` time) under its label, the best (a badge for a new best), stats, actions and a share hook.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is shown. |
| `title` | `string` | `"Level complete"` | — | — | — | Its title. |
| `score` | `number` | `0` | — | — | — | The score: points, or seconds with `format` time. |
| `best` | `number` | `0` | — | — | — | The best score: points, or seconds with `format` time (0 is none). |
| `format` | `ScoreFormat` | `"points"` | — | — | — | How the score and best are shown: points (the default) or a time. |
| `scoreLabel` | `string` | `""` | — | — | — | A caption over the score (`Total time`, `Score`); empty is none. A localised game passes its string table's. |
| `newBest` | `boolean` | `false` | — | — | — | This run set the best. |
| `stars` | `number` | `0` | — | — | — | Stars earned. |
| `maxStars` | `number` | `3` | — | — | — | Stars possible; 0 hides them. |
| `stats` | `readonly StatRow[]` | `[]` | — | — | — | Stats under the score. |
| `actions` | `readonly MenuItem[]` | `[]` | — | — | — | The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`). |
| `shareText` | `string` | `""` | — | — | — | Text a share would post; empty hides Share. |
| `onShare` | `UiAction \| null` | `null` | — | — | — | Called with the share text when Share is pressed (the shell posts it). |
| `bestLabel` | `string` | `"Best {best}"` | — | — | — | The best score's line; `{best}` is replaced by the best score, formatted as the score is (m:ss.mmm with `format` time). A localised game passes its string table's (og.i18n.t), with `{best}` in it. |
| `newBestLabel` | `string` | `"New best"` | — | — | — | The badge a new best shows. |
| `shareLabel` | `string` | `"Share"` | — | — | — | The Share button's label. |

```ogt
struct ResultsDesc {
  doc "A results screen: the title, stars that land one by one, the score counting up (points, or a time with `format` time) under its label, the best (a badge for a new best), stats, actions and a share hook."
  desc true
  field open bool "Whether it is shown."
  field title String "Its title." {
    default "Level complete"
  }
  field score f64 "The score: points, or seconds with `format` time."
  field best f64 "The best score: points, or seconds with `format` time (0 is none)."
  field format ScoreFormat "How the score and best are shown: points (the default) or a time."
  field scoreLabel String "A caption over the score (`Total time`, `Score`); empty is none. A localised game passes its string table's."
  field newBest bool "This run set the best."
  field stars u32 "Stars earned."
  field maxStars u32 "Stars possible; 0 hides them." {
    default 3
  }
  field stats List.StatRow "Stats under the score."
  field actions List.MenuItem "The actions; empty is Retry, Next and Menu (`retry`, `next`, `menu`)."
  field shareText String "Text a share would post; empty hides Share."
  field onShare Option.UiAction "Called with the share text when Share is pressed (the shell posts it)."
  field bestLabel String "The best score's line; `{best}` is replaced by the best score, formatted as the score is (m:ss.mmm with `format` time). A localised game passes its string table's (og.i18n.t), with `{best}` in it." {
    default "Best {best}"
  }
  field newBestLabel String "The badge a new best shows." {
    default "New best"
  }
  field shareLabel String "The Share button's label." {
    default "Share"
  }
}
```

<sub>`api.ogt` line 13733</sub>

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

<sub>`api.ogt` line 13764</sub>

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
| `retryLabel` | `string` | `"Retry"` | — | — | — | The button under an error. Every word of the leaderboard is a field with its English default (title and empty too; error and scopes are the game's): a localised game passes its string table's (og.i18n.t). |

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
  field retryLabel String "The button under an error. Every word of the leaderboard is a field with its English default (title and empty too; error and scopes are the game's): a localised game passes its string table's (og.i18n.t)." {
    default "Retry"
  }
}
```

<sub>`api.ogt` line 13773</sub>

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

<sub>`api.ogt` line 13791</sub>

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

<sub>`api.ogt` line 13799</sub>

<a id="type-TutorialDesc"></a>

### `TutorialDesc` (desc struct)

An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `open` | `boolean` | `false` | — | — | — | Whether it is shown. |
| `steps` | `readonly TutorialStep[]` | `[]` | — | — | — | The steps. |
| `index` | `number` | `0` | — | — | — | The step shown, from game state. |
| `skipLabel` | `string` | `"Skip"` | — | — | — | The Skip button. Every word of the overlay is a field with its English default: a localised game passes its string table's (og.i18n.t). |
| `nextLabel` | `string` | `"Next"` | — | — | — | The button that shows the next step. |
| `doneLabel` | `string` | `"Done"` | — | — | — | That button on the last step. |
| `stepLabel` | `string` | `"{step} of {count}"` | — | — | — | The step count; `{step}` is replaced by the step's number from 1 and `{count}` by the number of steps. A localised game passes its string table's, with both in it. |

```ogt
struct TutorialDesc {
  doc "An onboarding overlay: a spotlight on each step's target, a callout beside it with Next and Skip, and the step count."
  desc true
  field open bool "Whether it is shown."
  field steps List.TutorialStep "The steps."
  field index u32 "The step shown, from game state."
  field skipLabel String "The Skip button. Every word of the overlay is a field with its English default: a localised game passes its string table's (og.i18n.t)." {
    default "Skip"
  }
  field nextLabel String "The button that shows the next step." {
    default "Next"
  }
  field doneLabel String "That button on the last step." {
    default "Done"
  }
  field stepLabel String "The step count; `{step}` is replaced by the step's number from 1 and `{count}` by the number of steps. A localised game passes its string table's, with both in it." {
    default "{step} of {count}"
  }
}
```

<sub>`api.ogt` line 13807</sub>

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

<sub>`api.ogt` line 13827</sub>

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

<sub>`api.ogt` line 13834</sub>

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

<sub>`api.ogt` line 13848</sub>

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

<sub>`api.ogt` line 13863</sub>

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

<sub>`api.ogt` line 13869</sub>

<a id="type-UiProp"></a>

### `UiProp` (enum)

An animatable value of an element (docs/ui.md §7.1): a state, a timeline key, a presence or og.ui.animate moves it. Colours are sRGB components 0 to 1; rotation and skew are radians.

A string: one of the variants below. The zero value is `"x"`.

| Variant | Doc |
|---|---|
| `"x"` | Its horizontal offset, CSS px (the transform's x). |
| `"y"` | Its vertical offset, CSS px. |
| `"scaleX"` | Its horizontal scale (1: unscaled). |
| `"scaleY"` | Its vertical scale. |
| `"scale"` | Both scales at once. |
| `"rotate"` | Its turn, radians, clockwise on screen. |
| `"skewX"` | Its horizontal skew, radians. |
| `"skewY"` | Its vertical skew, radians. |
| `"opacity"` | Its opacity, 0 to 1, multiplied down the tree. |
| `"radius"` | Its corner radius, CSS px. |
| `"backgroundR"` | Its fill's red. |
| `"backgroundG"` | Its fill's green. |
| `"backgroundB"` | Its fill's blue. |
| `"backgroundA"` | Its fill's alpha. |
| `"inkR"` | Its text's red. |
| `"inkG"` | Its text's green. |
| `"inkB"` | Its text's blue. |
| `"inkA"` | Its text's alpha. |
| `"z"` | Its z among its siblings (stepped, never tweened). |
| `"inert"` | 1: it and its subtree take no input (stepped): a results screen's buttons wait for the sequence. |

```ogt
enum UiProp {
  doc "An animatable value of an element (docs/ui.md §7.1): a state, a timeline key, a presence or og.ui.animate moves it. Colours are sRGB components 0 to 1; rotation and skew are radians."
  variant x "Its horizontal offset, CSS px (the transform's x)."
  variant y "Its vertical offset, CSS px."
  variant scaleX "Its horizontal scale (1: unscaled)."
  variant scaleY "Its vertical scale."
  variant scale "Both scales at once."
  variant rotate "Its turn, radians, clockwise on screen."
  variant skewX "Its horizontal skew, radians."
  variant skewY "Its vertical skew, radians."
  variant opacity "Its opacity, 0 to 1, multiplied down the tree."
  variant radius "Its corner radius, CSS px."
  variant backgroundR "Its fill's red."
  variant backgroundG "Its fill's green."
  variant backgroundB "Its fill's blue."
  variant backgroundA "Its fill's alpha."
  variant inkR "Its text's red."
  variant inkG "Its text's green."
  variant inkB "Its text's blue."
  variant inkA "Its text's alpha."
  variant z "Its z among its siblings (stepped, never tweened)."
  variant inert "1: it and its subtree take no input (stepped): a results screen's buttons wait for the sequence."
}
```

<sub>`api.ogt` line 14024</sub>

<a id="type-LayoutMode"></a>

### `LayoutMode` (enum)

How a container lays out its children.

A string: one of the variants below. The zero value is `"flex"`.

| Variant | Doc |
|---|---|
| `"flex"` | The row and column walks of its direction (the default). |
| `"stack"` | Children overlaid in the same rectangle, each placed by its insets, else by the container's align (across) and cross (down): a card's art, frame, cost gem and name. |
| `"grid"` | Tracks and cells (ElementProps.grid). |

```ogt
enum LayoutMode {
  doc "How a container lays out its children."
  variant flex "The row and column walks of its direction (the default)."
  variant stack "Children overlaid in the same rectangle, each placed by its insets, else by the container's align (across) and cross (down): a card's art, frame, cost gem and name."
  variant grid "Tracks and cells (ElementProps.grid)."
}
```

<sub>`api.ogt` line 14048</sub>

<a id="type-Position"></a>

### `Position` (enum)

How an element is placed.

A string: one of the variants below. The zero value is `"flow"`.

| Variant | Doc |
|---|---|
| `"flow"` | In its parent's flow (the default). |
| `"absolute"` | Out of the flow, placed by its insets inside its parent's padding box. |
| `"anchored"` | Out of the flow, its anchorSelf point on the anchorAt point of the element at anchorTo, moved by its start and top insets. |

```ogt
enum Position {
  doc "How an element is placed."
  variant flow "In its parent's flow (the default)."
  variant absolute "Out of the flow, placed by its insets inside its parent's padding box."
  variant anchored "Out of the flow, its anchorSelf point on the anchorAt point of the element at anchorTo, moved by its start and top insets."
}
```

<sub>`api.ogt` line 14055</sub>

<a id="type-GridFlow"></a>

### `GridFlow` (enum)

Which way auto-placed items fill a grid.

A string: one of the variants below. The zero value is `"row"`.

| Variant | Doc |
|---|---|
| `"row"` | Along rows, adding rows (the default). |
| `"column"` | Down columns, adding columns. |

```ogt
enum GridFlow {
  doc "Which way auto-placed items fill a grid."
  variant row "Along rows, adding rows (the default)."
  variant column "Down columns, adding columns."
}
```

<sub>`api.ogt` line 14062</sub>

<a id="type-DragAxis"></a>

### `DragAxis` (enum)

Which way a drag moves its element.

A string: one of the variants below. The zero value is `"both"`.

| Variant | Doc |
|---|---|
| `"both"` | Both ways. |
| `"x"` | Across only. |
| `"y"` | Down only. |
| `"lock"` | Either, locked to the first one the drag moves along. |

```ogt
enum DragAxis {
  doc "Which way a drag moves its element."
  variant both "Both ways."
  variant x "Across only."
  variant y "Down only."
  variant lock "Either, locked to the first one the drag moves along."
}
```

<sub>`api.ogt` line 14068</sub>

<a id="type-EaseName"></a>

### `EaseName` (enum)

A named easing curve.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Constant speed. |
| `"out"` | How things arrive: fast, then settling (the default). |
| `"exit"` | How things leave: slow, then away. |
| `"inOut"` | Slow at both ends. |

```ogt
enum EaseName {
  doc "A named easing curve."
  variant linear "Constant speed."
  variant out "How things arrive: fast, then settling (the default)."
  variant exit "How things leave: slow, then away."
  variant inOut "Slow at both ends."
}
```

<sub>`api.ogt` line 14076</sub>

<a id="type-MotionToken"></a>

### `MotionToken` (enum)

A named motion of the style (docs/ui.md §7.7): components use these, so a style's personality changes every motion at once.

A string: one of the variants below. The zero value is `"ui"`.

| Variant | Doc |
|---|---|
| `"ui"` | The control spring: hover, press, toggles. |
| `"panel"` | Panels, sheets and drawers. |
| `"emphasis"` | Rewards, results and level-ups. |
| `"playful"` | Bouncy toys: chips, stickers, mascots. |
| `"enter"` | The entrance ease and duration. |
| `"exit"` | The exit ease and duration. |

```ogt
enum MotionToken {
  doc "A named motion of the style (docs/ui.md §7.7): components use these, so a style's personality changes every motion at once."
  variant ui "The control spring: hover, press, toggles."
  variant panel "Panels, sheets and drawers."
  variant emphasis "Rewards, results and level-ups."
  variant playful "Bouncy toys: chips, stickers, mascots."
  variant enter "The entrance ease and duration."
  variant exit "The exit ease and duration."
}
```

<sub>`api.ogt` line 14084</sub>

<a id="type-ReducedMotion"></a>

### `ReducedMotion` (enum)

What an animation does under reduced motion (docs/ui.md §7.7).

A string: one of the variants below. The zero value is `"fade"`.

| Variant | Doc |
|---|---|
| `"fade"` | Movement and scale jump; opacity and colour fade quickly (the default). |
| `"none"` | Everything jumps. |
| `"keep"` | Plays as authored: motion that carries meaning, kept small. |

```ogt
enum ReducedMotion {
  doc "What an animation does under reduced motion (docs/ui.md §7.7)."
  variant fade "Movement and scale jump; opacity and colour fade quickly (the default)."
  variant none "Everything jumps."
  variant keep "Plays as authored: motion that carries meaning, kept small."
}
```

<sub>`api.ogt` line 14094</sub>

<a id="type-Orientation"></a>

### `Orientation` (enum)

A screen orientation.

A string: one of the variants below. The zero value is `"portrait"`.

| Variant | Doc |
|---|---|
| `"portrait"` | Taller than wide. |
| `"landscape"` | Wider than tall. |

```ogt
enum Orientation {
  doc "A screen orientation."
  variant portrait "Taller than wide."
  variant landscape "Wider than tall."
}
```

<sub>`api.ogt` line 14101</sub>

<a id="type-Repeat"></a>

### `Repeat` (enum)

How a play repeats.

A string: one of the variants below. The zero value is `"once"`.

| Variant | Doc |
|---|---|
| `"once"` | Once (the default). |
| `"loop"` | Over and over. |
| `"pingPong"` | Forward then back, over and over. |

```ogt
enum Repeat {
  doc "How a play repeats."
  variant once "Once (the default)."
  variant loop "Over and over."
  variant pingPong "Forward then back, over and over."
}
```

<sub>`api.ogt` line 14107</sub>

<a id="type-UiTransform"></a>

### `UiTransform` (desc struct)

A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. Hit tests, the probe's rectangles and the accessibility tree's bounds follow it; layout does not.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Moves it across. |
| `y` | `number` | `0` | — | `cssPx` | — | Moves it down. |
| `scaleX` | `number` | `1` | — | — | — | Its horizontal scale. |
| `scaleY` | `number` | `1` | — | — | — | Its vertical scale. |
| `rotate` | `number` | `0` | — | — | — | Its turn, radians, clockwise on screen. |
| `skewX` | `number` | `0` | — | — | — | Its horizontal skew, radians. |
| `skewY` | `number` | `0` | — | — | — | Its vertical skew, radians. |
| `pivotX` | `number` | `0.5` | — | `ratio` | — | The pivot across its box (0 its start edge, 1 its end). |
| `pivotY` | `number` | `0.5` | — | `ratio` | — | The pivot down its box. |

```ogt
struct UiTransform {
  doc "A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. Hit tests, the probe's rectangles and the accessibility tree's bounds follow it; layout does not."
  desc true
  field x f32 "Moves it across." {
    unit cssPx
  }
  field y f32 "Moves it down." {
    unit cssPx
  }
  field scaleX f32 "Its horizontal scale." {
    default 1
  }
  field scaleY f32 "Its vertical scale." {
    default 1
  }
  field rotate f32 "Its turn, radians, clockwise on screen."
  field skewX f32 "Its horizontal skew, radians."
  field skewY f32 "Its vertical skew, radians."
  field pivotX f32 "The pivot across its box (0 its start edge, 1 its end)." {
    default 0.5
    unit ratio
  }
  field pivotY f32 "The pivot down its box." {
    default 0.5
    unit ratio
  }
}
```

<sub>`api.ogt` line 14114</sub>

<a id="type-Edges"></a>

### `Edges` (desc struct)

Insets of an absolute or anchored element from its parent's edges; an absent one is not set. start and end follow the writing direction: a right-to-left layout mirrors them.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `start` | `number \| null` | `null` | — | `cssPx` | — | From the start edge. |
| `top` | `number \| null` | `null` | — | `cssPx` | — | From the top. |
| `end` | `number \| null` | `null` | — | `cssPx` | — | From the end edge. |
| `bottom` | `number \| null` | `null` | — | `cssPx` | — | From the bottom. |
| `centerX` | `boolean` | `false` | — | — | — | Centred across the parent (start, if given, moves it). |
| `centerY` | `boolean` | `false` | — | — | — | Centred down the parent (top, if given, moves it). |

```ogt
struct Edges {
  doc "Insets of an absolute or anchored element from its parent's edges; an absent one is not set. start and end follow the writing direction: a right-to-left layout mirrors them."
  desc true
  field start Option.f32 "From the start edge." {
    unit cssPx
  }
  field top Option.f32 "From the top." {
    unit cssPx
  }
  field end Option.f32 "From the end edge." {
    unit cssPx
  }
  field bottom Option.f32 "From the bottom." {
    unit cssPx
  }
  field centerX bool "Centred across the parent (start, if given, moves it)."
  field centerY bool "Centred down the parent (top, if given, moves it)."
}
```

<sub>`api.ogt` line 14142</sub>

<a id="type-TrackPx"></a>

### `TrackPx` (struct)

A fixed track.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `px` | `number` | `0` | — | `cssPx` | — | Its size. |

```ogt
struct TrackPx {
  doc "A fixed track."
  field px f32 "Its size." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 14161</sub>

<a id="type-TrackFr"></a>

### `TrackFr` (struct)

A share of the space the other tracks leave.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fr` | `number` | `1` | — | — | — | Its share. |

```ogt
struct TrackFr {
  doc "A share of the space the other tracks leave."
  field fr f32 "Its share." {
    default 1
  }
}
```

<sub>`api.ogt` line 14168</sub>

<a id="type-TrackRange"></a>

### `TrackRange` (struct)

A track sized by its content, within bounds.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `min` | `number` | `0` | — | `cssPx` | — | The smallest it gets. |
| `max` | `number` | `0` | — | `cssPx` | — | The largest it gets. |

```ogt
struct TrackRange {
  doc "A track sized by its content, within bounds."
  field min f32 "The smallest it gets." {
    unit cssPx
  }
  field max f32 "The largest it gets." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 14175</sub>

<a id="type-TrackPercent"></a>

### `TrackPercent` (struct)

A fraction of the grid's inner size.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `fraction` | `number` | `0` | — | `ratio` | — | The fraction. |

```ogt
struct TrackPercent {
  doc "A fraction of the grid's inner size."
  field fraction f32 "The fraction." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 14185</sub>

<a id="type-TrackAuto"></a>

### `TrackAuto` (struct)

Its largest single-cell item.

```ogt
struct TrackAuto {
  doc "Its largest single-cell item."
}
```

<sub>`api.ogt` line 14192</sub>

<a id="type-Track"></a>

### `Track` (union)

A grid track's size.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "px"` | [`TrackPx`](#type-TrackPx) | A fixed size. |
| `kind: "fr"` | [`TrackFr`](#type-TrackFr) | A share of the free space. |
| `kind: "auto"` | [`TrackAuto`](#type-TrackAuto) | Its largest single-cell item. |
| `kind: "minMax"` | [`TrackRange`](#type-TrackRange) | Its content between min and max. |
| `kind: "percent"` | [`TrackPercent`](#type-TrackPercent) | A fraction of the grid. |

```ogt
union Track {
  doc "A grid track's size."
  tag kind
  case px TrackPx "A fixed size."
  case fr TrackFr "A share of the free space."
  case auto TrackAuto "Its largest single-cell item."
  case minMax TrackRange "Its content between min and max."
  case percent TrackPercent "A fraction of the grid."
}
```

<sub>`api.ogt` line 14196</sub>

<a id="type-GridSpec"></a>

### `GridSpec` (desc struct)

A grid's tracks and areas.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `columns` | `readonly Track[]` | `[]` | — | — | — | The column tracks. |
| `rows` | `readonly Track[]` | `[]` | — | — | — | The row tracks; items past them add autoTrack rows. |
| `autoTrack` | `Track \| null` | `null` | — | — | — | The track added for items past the declared ones; null is auto. |
| `columnGap` | `number \| null` | `null` | — | `cssPx` | — | Space between columns; absent is the element's gap. |
| `rowGap` | `number \| null` | `null` | — | `cssPx` | — | Space between rows; absent is the element's gap. |
| `areas` | `readonly string[]` | `[]` | — | — | — | Named areas, one string per row of space-separated cell names, as CSS grid-template-areas (`.` is an unnamed cell). |
| `flow` | `GridFlow` | `"row"` | — | — | — | Which way auto-placed items fill it. |
| `dense` | `boolean` | `false` | — | — | — | Auto-placement back-fills holes left by spanning items. |

```ogt
struct GridSpec {
  doc "A grid's tracks and areas."
  desc true
  field columns List.Track "The column tracks."
  field rows List.Track "The row tracks; items past them add autoTrack rows."
  field autoTrack Option.Track "The track added for items past the declared ones; null is auto."
  field columnGap Option.f32 "Space between columns; absent is the element's gap." {
    unit cssPx
  }
  field rowGap Option.f32 "Space between rows; absent is the element's gap." {
    unit cssPx
  }
  field areas List.String "Named areas, one string per row of space-separated cell names, as CSS grid-template-areas (`.` is an unnamed cell)."
  field flow GridFlow "Which way auto-placed items fill it."
  field dense bool "Auto-placement back-fills holes left by spanning items."
}
```

<sub>`api.ogt` line 14206</sub>

<a id="type-GridPlace"></a>

### `GridPlace` (desc struct)

Where an item sits in its parent grid; the zero value is auto-placed in one cell.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `column` | `number` | `0` | — | — | — | Its first column, 1-based (0: auto). |
| `row` | `number` | `0` | — | — | — | Its first row, 1-based (0: auto). |
| `columnSpan` | `number` | `0` | — | — | — | Columns spanned (0: 1). |
| `rowSpan` | `number` | `0` | — | — | — | Rows spanned (0: 1). |
| `area` | `string` | `""` | — | — | — | A named area of the grid (wins over the numbers). |

```ogt
struct GridPlace {
  doc "Where an item sits in its parent grid; the zero value is auto-placed in one cell."
  desc true
  field column u32 "Its first column, 1-based (0: auto)."
  field row u32 "Its first row, 1-based (0: auto)."
  field columnSpan u32 "Columns spanned (0: 1)."
  field rowSpan u32 "Rows spanned (0: 1)."
  field area String "A named area of the grid (wins over the numbers)."
}
```

<sub>`api.ogt` line 14223</sub>

<a id="type-PropValue"></a>

### `PropValue` (struct)

A value for an animatable prop.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `prop` | `UiProp` | `"x"` | — | — | — | The prop. |
| `value` | `number` | `0` | — | — | — | Its value. |

```ogt
struct PropValue {
  doc "A value for an animatable prop."
  field prop UiProp "The prop."
  field value f32 "Its value."
}
```

<sub>`api.ogt` line 14233</sub>

<a id="type-UiQuery"></a>

### `UiQuery` (desc struct)

When a variant applies: every condition given must hold.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `breakpoints` | `readonly Breakpoint[]` | `[]` | — | — | — | The size classes it applies on (empty: any). |
| `orientation` | `Orientation \| null` | `null` | — | — | — | The orientation it applies in; null is either. |
| `minWidth` | `number` | `0` | — | `cssPx` | — | The narrowest width it applies at (0: unbounded): the viewport's, or with container the element's own laid-out width. |
| `maxWidth` | `number` | `0` | — | `cssPx` | — | The widest (0: unbounded). |
| `container` | `boolean` | `false` | — | — | — | A container query: the widths are the element's own (the same card in a four-column shop and a one-column phone list). |

```ogt
struct UiQuery {
  doc "When a variant applies: every condition given must hold."
  desc true
  field breakpoints List.Breakpoint "The size classes it applies on (empty: any)."
  field orientation Option.Orientation "The orientation it applies in; null is either."
  field minWidth f32 "The narrowest width it applies at (0: unbounded): the viewport's, or with container the element's own laid-out width." {
    unit cssPx
  }
  field maxWidth f32 "The widest (0: unbounded)." {
    unit cssPx
  }
  field container bool "A container query: the widths are the element's own (the same card in a four-column shop and a one-column phone list)."
}
```

<sub>`api.ogt` line 14239</sub>

<a id="type-UiPatch"></a>

### `UiPatch` (desc struct)

What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `direction` | `Direction \| null` | `null` | — | — | — | Its direction. |
| `width` | `Size \| null` | `null` | — | — | — | Its width. |
| `height` | `Size \| null` | `null` | — | — | — | Its height. |
| `padding` | `Insets \| null` | `null` | — | — | — | Its padding. |
| `gap` | `number \| null` | `null` | — | `cssPx` | — | Its gap. |
| `align` | `Align \| null` | `null` | — | — | — | Its main-axis alignment. |
| `cross` | `Cross \| null` | `null` | — | — | — | Its cross-axis alignment. |
| `wrap` | `boolean \| null` | `null` | — | — | — | Its wrapping. |
| `layout` | `LayoutMode \| null` | `null` | — | — | — | Its layout mode. |
| `columns` | `readonly Track[]` | `[]` | — | — | — | Its grid columns (empty: unchanged). |
| `hidden` | `boolean \| null` | `null` | — | — | — | Hide it and its subtree where the variant applies. |
| `values` | `readonly PropValue[]` | `[]` | — | — | — | Animatable values it sets. |

```ogt
struct UiPatch {
  doc "What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged."
  desc true
  field direction Option.Direction "Its direction."
  field width Option.Size "Its width."
  field height Option.Size "Its height."
  field padding Option.Insets "Its padding."
  field gap Option.f32 "Its gap." {
    unit cssPx
  }
  field align Option.Align "Its main-axis alignment."
  field cross Option.Cross "Its cross-axis alignment."
  field wrap Option.bool "Its wrapping."
  field layout Option.LayoutMode "Its layout mode."
  field columns List.Track "Its grid columns (empty: unchanged)."
  field hidden Option.bool "Hide it and its subtree where the variant applies."
  field values List.PropValue "Animatable values it sets."
}
```

<sub>`api.ogt` line 14253</sub>

<a id="type-UiVariant"></a>

### `UiVariant` (struct)

A prop override under a condition (docs/ui.md §6.2: breakpoint variants and container queries). Later variants win.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `when` | `UiQuery` | `{}` | — | — | — | When it applies. |
| `set` | `UiPatch` | `{}` | — | — | — | What it changes. |

```ogt
struct UiVariant {
  doc "A prop override under a condition (docs/ui.md §6.2: breakpoint variants and container queries). Later variants win."
  field when UiQuery "When it applies."
  field set UiPatch "What it changes."
}
```

<sub>`api.ogt` line 14272</sub>

<a id="type-SpringSpec"></a>

### `SpringSpec` (desc struct)

A spring by how it feels: about how long it takes to settle and how much it overshoots. Retargeting it keeps its value and velocity, so an interrupted motion never jumps.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `durationMs` | `number` | `350` | — | `ms` | — | About how long it takes to arrive, ms. |
| `bounce` | `number` | `0` | -0.95 … 1 | — | — | 0 arrives without overshoot; 0.3 overshoots a little; up to 1 rings (below 0 is sluggish). The style's overshoot caps it. |

```ogt
struct SpringSpec {
  doc "A spring by how it feels: about how long it takes to settle and how much it overshoots. Retargeting it keeps its value and velocity, so an interrupted motion never jumps."
  desc true
  field durationMs f32 "About how long it takes to arrive, ms." {
    default 350
    unit ms
  }
  field bounce f32 "0 arrives without overshoot; 0.3 overshoots a little; up to 1 rings (below 0 is sluggish). The style's overshoot caps it." {
    min -0.95
    max 1
  }
}
```

<sub>`api.ogt` line 14278</sub>

<a id="type-TweenSpec"></a>

### `TweenSpec` (desc struct)

A duration on an ease: a retarget starts a new ease from where it is.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `durationMs` | `number` | `200` | — | `ms` | — | Its length, ms. |
| `ease` | `EaseName` | `"linear"` | — | — | — | Its curve. |
| `bezier` | `readonly number[]` | `[]` | — | — | — | A cubic Bézier (x1, y1, x2, y2) in place of the named ease (empty: the named one). |

```ogt
struct TweenSpec {
  doc "A duration on an ease: a retarget starts a new ease from where it is."
  desc true
  field durationMs f32 "Its length, ms." {
    default 200
    unit ms
  }
  field ease EaseName "Its curve."
  field bezier List.f32 "A cubic Bézier (x1, y1, x2, y2) in place of the named ease (empty: the named one)."
}
```

<sub>`api.ogt` line 14291</sub>

<a id="type-TokenRef"></a>

### `TokenRef` (struct)

One of the style's motion tokens.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `token` | `MotionToken` | `"ui"` | — | — | — | The token. |

```ogt
struct TokenRef {
  doc "One of the style's motion tokens."
  field token MotionToken "The token."
}
```

<sub>`api.ogt` line 14302</sub>

<a id="type-InstantSpec"></a>

### `InstantSpec` (struct)

At once.

```ogt
struct InstantSpec {
  doc "At once."
}
```

<sub>`api.ogt` line 14307</sub>

<a id="type-Transition"></a>

### `Transition` (union)

How a value moves to a new target.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "instant"` | [`InstantSpec`](#type-InstantSpec) | At once. |
| `kind: "spring"` | [`SpringSpec`](#type-SpringSpec) | On a spring that keeps its velocity. |
| `kind: "tween"` | [`TweenSpec`](#type-TweenSpec) | Over a duration on an ease. |
| `kind: "token"` | [`TokenRef`](#type-TokenRef) | The style's token. |

```ogt
union Transition {
  doc "How a value moves to a new target."
  tag kind
  case instant InstantSpec "At once."
  case spring SpringSpec "On a spring that keeps its velocity."
  case tween TweenSpec "Over a duration on an ease."
  case token TokenRef "The style's token."
}
```

<sub>`api.ogt` line 14311</sub>

<a id="type-DismissSpec"></a>

### `DismissSpec` (desc struct)

Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `distance` | `number` | `0` | — | `cssPx` | — | 0: 40 % of its size along the axis. |
| `velocity` | `number` | `0` | — | — | — | CSS px per second (0: 800). |

```ogt
struct DismissSpec {
  doc "Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off."
  desc true
  field distance f32 "0: 40 % of its size along the axis." {
    unit cssPx
  }
  field velocity f32 "CSS px per second (0: 800)."
}
```

<sub>`api.ogt` line 14320</sub>

<a id="type-SnapPoint"></a>

### `SnapPoint` (struct)

An offset a released drag can settle on.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `x` | `number` | `0` | — | `cssPx` | — | Its x offset. |
| `y` | `number` | `0` | — | `cssPx` | — | Its y offset. |

```ogt
struct SnapPoint {
  doc "An offset a released drag can settle on."
  field x f32 "Its x offset." {
    unit cssPx
  }
  field y f32 "Its y offset." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 14329</sub>

<a id="type-DragSpec"></a>

### `DragSpec` (desc struct)

The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `axis` | `DragAxis` | `"both"` | — | — | — | Which way it moves. |
| `bounds` | `UiRect \| null` | `null` | — | — | — | Bounds of its offset (x to x + width, y to y + height); null is free. |
| `rubber` | `number` | `0` | — | — | — | Past the bounds it follows with resistance, iOS's rubber band with this constant (0 stops at the bounds; 0.55 is iOS's). |
| `snap` | `readonly SnapPoint[]` | `[]` | — | — | — | Offsets a release settles on: the nearest to where its momentum would carry it. |
| `momentum` | `boolean` | `false` | — | — | — | A release projects the offset by its velocity (0.3 s of it) before choosing a snap point: a flick. |
| `returnHome` | `boolean` | `false` | — | — | — | Without snap points, a release goes back to offset 0. |
| `dismiss` | `DismissSpec \| null` | `null` | — | — | — | Swipe to dismiss; null never throws it off. |
| `threshold` | `number` | `0` | — | `cssPx` | — | Pointer travel before a press becomes a drag (0: 4). |
| `spring` | `Transition \| null` | `null` | — | — | — | The release spring; null is the style's ui spring. |
| `tag` | `string` | `""` | — | — | — | What a drop target matches in DropSpec.accept. |

```ogt
struct DragSpec {
  doc "The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity."
  desc true
  field axis DragAxis "Which way it moves."
  field bounds Option.UiRect "Bounds of its offset (x to x + width, y to y + height); null is free."
  field rubber f32 "Past the bounds it follows with resistance, iOS's rubber band with this constant (0 stops at the bounds; 0.55 is iOS's)."
  field snap List.SnapPoint "Offsets a release settles on: the nearest to where its momentum would carry it."
  field momentum bool "A release projects the offset by its velocity (0.3 s of it) before choosing a snap point: a flick."
  field returnHome bool "Without snap points, a release goes back to offset 0."
  field dismiss Option.DismissSpec "Swipe to dismiss; null never throws it off."
  field threshold f32 "Pointer travel before a press becomes a drag (0: 4)." {
    unit cssPx
  }
  field spring Option.Transition "The release spring; null is the style's ui spring."
  field tag String "What a drop target matches in DropSpec.accept."
}
```

<sub>`api.ogt` line 14339</sub>

<a id="type-PinchSpec"></a>

### `PinchSpec` (desc struct)

Two touches on the element scale (and turn) it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `minScale` | `number` | `0` | — | — | — | 0: 0.5. |
| `maxScale` | `number` | `0` | — | — | — | 0: 4. |
| `rotate` | `boolean` | `false` | — | — | — | The touches turn it too. |

```ogt
struct PinchSpec {
  doc "Two touches on the element scale (and turn) it."
  desc true
  field minScale f32 "0: 0.5."
  field maxScale f32 "0: 4."
  field rotate bool "The touches turn it too."
}
```

<sub>`api.ogt` line 14356</sub>

<a id="type-DropSpec"></a>

### `DropSpec` (desc struct)

The element takes drops of dragged elements.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `accept` | `readonly string[]` | `[]` | — | — | — | The drag tags it takes (empty: any). |

```ogt
struct DropSpec {
  doc "The element takes drops of dragged elements."
  desc true
  field accept List.String "The drag tags it takes (empty: any)."
}
```

<sub>`api.ogt` line 14364</sub>

<a id="type-PresenceSpec"></a>

### `PresenceSpec` (desc struct)

How an element arrives or leaves (docs/ui.md §7.5): the values it enters from or exits to, and how it moves between them and its own.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `values` | `readonly PropValue[]` | `[]` | — | — | — | Where it starts (entering) or ends (leaving). |
| `kind` | `TransitionKind \| null` | `null` | — | — | — | A transition kind's values in place of values: what the kind enters from and exits to. |
| `distance` | `number` | `0` | — | `cssPx` | — | A slide kind's distance (0: 24). |
| `transition` | `Transition \| null` | `null` | — | — | — | How it moves; null is the style's enter or exit token. |
| `delayMs` | `number` | `0` | — | `ms` | — | Wait this long before moving (a hand-built stagger). |

```ogt
struct PresenceSpec {
  doc "How an element arrives or leaves (docs/ui.md §7.5): the values it enters from or exits to, and how it moves between them and its own."
  desc true
  field values List.PropValue "Where it starts (entering) or ends (leaving)."
  field kind Option.TransitionKind "A transition kind's values in place of values: what the kind enters from and exits to."
  field distance f32 "A slide kind's distance (0: 24)." {
    unit cssPx
  }
  field transition Option.Transition "How it moves; null is the style's enter or exit token."
  field delayMs f32 "Wait this long before moving (a hand-built stagger)." {
    unit ms
  }
}
```

<sub>`api.ogt` line 14370</sub>

<a id="type-VisualState"></a>

### `VisualState` (struct)

One state of a group: the values it sets, and timelines it starts.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `values` | `readonly PropValue[]` | `[]` | — | — | — | The values it sets. |
| `onEnter` | `string` | `""` | — | — | — | A timeline (og.ui.defineTimeline) played on the element when the state is entered (empty: none). |
| `onExit` | `string` | `""` | — | — | — | One played when it is left. |

```ogt
struct VisualState {
  doc "One state of a group: the values it sets, and timelines it starts."
  field name String "Its name."
  field values List.PropValue "The values it sets."
  field onEnter String "A timeline (og.ui.defineTimeline) played on the element when the state is entered (empty: none)."
  field onExit String "One played when it is left."
}
```

<sub>`api.ogt` line 14384</sub>

<a id="type-StateTransition"></a>

### `StateTransition` (struct)

The transition between two states of a group (`*` matches any state). The exact pair wins, then `* -> to`, `from -> *`, `* -> *`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `from` | `string` | `""` | — | — | — | The state left. |
| `to` | `string` | `""` | — | — | — | The state entered. |
| `transition` | `Transition` | — | — | — | — | How the values move. |

```ogt
struct StateTransition {
  doc "The transition between two states of a group (`*` matches any state). The exact pair wins, then `* -> to`, `from -> *`, `* -> *`."
  field from String "The state left."
  field to String "The state entered."
  field transition Transition "How the values move."
}
```

<sub>`api.ogt` line 14392</sub>

<a id="type-StateGroup"></a>

### `StateGroup` (struct)

A visual state group (docs/ui.md §7.3): one of its states is current, and moving between them animates every value that changes. A group named interaction is derived from input: normal, hover, pressed, focused and disabled (a state the group lacks falls back: pressed to hover to normal, focused to hover). Other groups take their state from ElementProps.state, a timeline's state event or og.ui.forceState; the first state is the default.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `states` | `readonly VisualState[]` | `[]` | — | — | — | Its states. |
| `transitions` | `readonly StateTransition[]` | `[]` | — | — | — | Its transitions; a pair without one moves on the element's motion, else the style's ui spring. |

```ogt
struct StateGroup {
  doc "A visual state group (docs/ui.md §7.3): one of its states is current, and moving between them animates every value that changes. A group named interaction is derived from input: normal, hover, pressed, focused and disabled (a state the group lacks falls back: pressed to hover to normal, focused to hover). Other groups take their state from ElementProps.state, a timeline's state event or og.ui.forceState; the first state is the default."
  field name String "Its name."
  field states List.VisualState "Its states."
  field transitions List.StateTransition "Its transitions; a pair without one moves on the element's motion, else the style's ui spring."
}
```

<sub>`api.ogt` line 14399</sub>

<a id="type-StateValue"></a>

### `StateValue` (struct)

The current state of a group.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group. |
| `state` | `string` | `""` | — | — | — | Its state. |

```ogt
struct StateValue {
  doc "The current state of a group."
  field group String "The group."
  field state String "Its state."
}
```

<sub>`api.ogt` line 14406</sub>

<a id="type-ElementProps"></a>

### `ElementProps` (desc struct)

Everything an element carries beyond its box (docs/ui.md §6.1): set with og.ui.props right after the element. The zero value is an element in the flow, unmoved and opaque, with no motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `transform` | `UiTransform` | `{}` | — | — | — | Its transform after layout. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | Its opacity, multiplied down the tree. |
| `z` | `number` | `0` | — | — | — | Its drawing and hit-testing order among its siblings: higher draws over lower without leaving the flow. |
| `position` | `Position` | `"flow"` | — | — | — | How it is placed. |
| `insets` | `Edges` | `{}` | — | — | — | Its insets, for an absolute or anchored element. |
| `anchorTo` | `string` | `""` | — | — | — | The path of the element an anchored element is placed against. |
| `anchorAt` | `Anchor` | `"topLeft"` | — | — | — | The target's point it sits on. |
| `anchorSelf` | `Anchor` | `"topLeft"` | — | — | — | Its own point that sits there. |
| `hitSlop` | `number` | `0` | — | `cssPx` | — | Extra pointer reach around its box. |
| `layout` | `LayoutMode` | `"flex"` | — | — | — | How it lays out its children. |
| `grid` | `GridSpec` | `{}` | — | — | — | Its tracks and areas, for a grid. |
| `place` | `GridPlace` | `{}` | — | — | — | Its place in a grid parent. |
| `container` | `boolean` | `false` | — | — | — | Its container queries read its own width. |
| `variants` | `readonly UiVariant[]` | `[]` | — | — | — | Prop overrides by breakpoint, orientation and width; later ones win. |
| `overlaps` | `readonly string[]` | `[]` | — | — | — | Paths of elements it overlaps on purpose (an ancestor path covers its subtree). Undeclared overlaps are reported (og.ui.overlaps). |
| `match` | `string` | `""` | — | — | — | A shared-element key: the element with the same key on the screen being left morphs into this one. |
| `drag` | `DragSpec \| null` | `null` | — | — | — | It moves with a drag. |
| `pinch` | `PinchSpec \| null` | `null` | — | — | — | Two touches scale and turn it. |
| `drop` | `DropSpec \| null` | `null` | — | — | — | It takes drops. |
| `motion` | `Transition \| null` | `null` | — | — | — | How its animatable values move when they change; null is at once. |
| `layoutMotion` | `Transition \| null` | `null` | — | — | — | Moves from its last laid-out rectangle to its new one (FLIP); null jumps. |
| `enter` | `PresenceSpec \| null` | `null` | — | — | — | How it arrives. |
| `exit` | `PresenceSpec \| null` | `null` | — | — | — | How it leaves: kept, inert and out of hit tests and the accessibility tree, while it plays after the element leaves the tree. |
| `states` | `readonly StateGroup[]` | `[]` | — | — | — | Its visual state groups. |
| `state` | `readonly StateValue[]` | `[]` | — | — | — | The current state of its groups that are not derived from input. |
| `reduced` | `ReducedMotion \| null` | `null` | — | — | — | Its reduced-motion fallback; null is the style's. |
| `inert` | `boolean` | `false` | — | — | — | It and its subtree take no input. |

```ogt
struct ElementProps {
  doc "Everything an element carries beyond its box (docs/ui.md §6.1): set with og.ui.props right after the element. The zero value is an element in the flow, unmoved and opaque, with no motion."
  desc true
  field transform UiTransform "Its transform after layout."
  field opacity f32 "Its opacity, multiplied down the tree." {
    default 1
    min 0
    max 1
  }
  field z i32 "Its drawing and hit-testing order among its siblings: higher draws over lower without leaving the flow."
  field position Position "How it is placed."
  field insets Edges "Its insets, for an absolute or anchored element."
  field anchorTo String "The path of the element an anchored element is placed against."
  field anchorAt Anchor "The target's point it sits on."
  field anchorSelf Anchor "Its own point that sits there."
  field hitSlop f32 "Extra pointer reach around its box." {
    unit cssPx
  }
  field layout LayoutMode "How it lays out its children."
  field grid GridSpec "Its tracks and areas, for a grid."
  field place GridPlace "Its place in a grid parent."
  field container bool "Its container queries read its own width."
  field variants List.UiVariant "Prop overrides by breakpoint, orientation and width; later ones win."
  field overlaps List.String "Paths of elements it overlaps on purpose (an ancestor path covers its subtree). Undeclared overlaps are reported (og.ui.overlaps)."
  field match String "A shared-element key: the element with the same key on the screen being left morphs into this one."
  field drag Option.DragSpec "It moves with a drag."
  field pinch Option.PinchSpec "Two touches scale and turn it."
  field drop Option.DropSpec "It takes drops."
  field motion Option.Transition "How its animatable values move when they change; null is at once."
  field layoutMotion Option.Transition "Moves from its last laid-out rectangle to its new one (FLIP); null jumps."
  field enter Option.PresenceSpec "How it arrives."
  field exit Option.PresenceSpec "How it leaves: kept, inert and out of hit tests and the accessibility tree, while it plays after the element leaves the tree."
  field states List.StateGroup "Its visual state groups."
  field state List.StateValue "The current state of its groups that are not derived from input."
  field reduced Option.ReducedMotion "Its reduced-motion fallback; null is the style's."
  field inert bool "It and its subtree take no input."
}
```

<sub>`api.ogt` line 14412</sub>

<a id="type-EaseKey"></a>

### `EaseKey` (struct)

A named ease into a key.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `ease` | `EaseName` | `"linear"` | — | — | — | The curve. |

```ogt
struct EaseKey {
  doc "A named ease into a key."
  field ease EaseName "The curve."
}
```

<sub>`api.ogt` line 14450</sub>

<a id="type-HoldKey"></a>

### `HoldKey` (struct)

Holds the previous value, then jumps at the key.

```ogt
struct HoldKey {
  doc "Holds the previous value, then jumps at the key."
}
```

<sub>`api.ogt` line 14455</sub>

<a id="type-LinearKey"></a>

### `LinearKey` (struct)

Constant speed into the key.

```ogt
struct LinearKey {
  doc "Constant speed into the key."
}
```

<sub>`api.ogt` line 14459</sub>

<a id="type-BezierKey"></a>

### `BezierKey` (struct)

A cubic Bézier into the key.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `bezier` | `readonly number[]` | `[]` | — | — | — | x1, y1, x2, y2. |

```ogt
struct BezierKey {
  doc "A cubic Bézier into the key."
  field bezier List.f32 "x1, y1, x2, y2."
}
```

<sub>`api.ogt` line 14463</sub>

<a id="type-KeyEase"></a>

### `KeyEase` (union)

How the segment into a key moves.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "linear"` | [`LinearKey`](#type-LinearKey) | Constant speed. |
| `kind: "ease"` | [`EaseKey`](#type-EaseKey) | A named ease. |
| `kind: "bezier"` | [`BezierKey`](#type-BezierKey) | A cubic Bézier. |
| `kind: "spring"` | [`SpringSpec`](#type-SpringSpec) | On a spring from the previous key's value (released at it). |
| `kind: "hold"` | [`HoldKey`](#type-HoldKey) | Holds, then jumps. |

```ogt
union KeyEase {
  doc "How the segment into a key moves."
  tag kind
  case linear LinearKey "Constant speed."
  case ease EaseKey "A named ease."
  case bezier BezierKey "A cubic Bézier."
  case spring SpringSpec "On a spring from the previous key's value (released at it)."
  case hold HoldKey "Holds, then jumps."
}
```

<sub>`api.ogt` line 14468</sub>

<a id="type-TimelineKey"></a>

### `TimelineKey` (struct)

A keyframe: a value at a time.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |
| `value` | `number` | `0` | — | — | — | Its value. |
| `ease` | `KeyEase \| null` | `null` | — | — | — | How the segment into it moves; null is the out ease. |

```ogt
struct TimelineKey {
  doc "A keyframe: a value at a time."
  field atMs f32 "Its time." {
    unit ms
  }
  field value f32 "Its value."
  field ease Option.KeyEase "How the segment into it moves; null is the out ease."
}
```

<sub>`api.ogt` line 14478</sub>

<a id="type-TimelineTrack"></a>

### `TimelineTrack` (struct)

Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). Before its first key it holds that key's value, after its last the last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `prop` | `UiProp` | `"x"` | — | — | — | The prop. |
| `keys` | `readonly TimelineKey[]` | `[]` | — | — | — | In time order. |

```ogt
struct TimelineTrack {
  doc "Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). Before its first key it holds that key's value, after its last the last."
  field target String "A path below the play's element (empty: the element itself)."
  field prop UiProp "The prop."
  field keys List.TimelineKey "In time order."
}
```

<sub>`api.ogt` line 14487</sub>

<a id="type-SoundEvent"></a>

### `SoundEvent` (struct)

A UI sound of the style.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | hover, press, open, close, success or error. |
| `pitch` | `number` | `1` | — | — | — | Its pitch (1: as recorded); a cascade step raises it 6 %. |

```ogt
struct SoundEvent {
  doc "A UI sound of the style."
  field name String "hover, press, open, close, success or error."
  field pitch f32 "Its pitch (1: as recorded); a cascade step raises it 6 %." {
    default 1
  }
}
```

<sub>`api.ogt` line 14494</sub>

<a id="type-HapticEvent"></a>

### `HapticEvent` (struct)

A haptic pattern for the host.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | light, medium, heavy, success, warning or error. |

```ogt
struct HapticEvent {
  doc "A haptic pattern for the host."
  field name String "light, medium, heavy, success, warning or error."
}
```

<sub>`api.ogt` line 14502</sub>

<a id="type-BurstEvent"></a>

### `BurstEvent` (struct)

Rays and sparks from the target.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `count` | `number` | `0` | — | — | — | How many (0: 12). |
| `radius` | `number` | `0` | — | `cssPx` | — | How far they reach (0: the style's). |
| `color` | `Color \| null` | `null` | — | — | — | Their colour; null is the style's highlight. |

```ogt
struct BurstEvent {
  doc "Rays and sparks from the target."
  field count u32 "How many (0: 12)."
  field radius f32 "How far they reach (0: the style's)." {
    unit cssPx
  }
  field color Option.Color "Their colour; null is the style's highlight."
}
```

<sub>`api.ogt` line 14507</sub>

<a id="type-StateEvent"></a>

### `StateEvent` (struct)

Sets a state group of the target.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `group` | `string` | `""` | — | — | — | The group. |
| `state` | `string` | `""` | — | — | — | Its new state. |

```ogt
struct StateEvent {
  doc "Sets a state group of the target."
  field group String "The group."
  field state String "Its new state."
}
```

<sub>`api.ogt` line 14516</sub>

<a id="type-CallbackEvent"></a>

### `CallbackEvent` (struct)

A callback to game code: it arrives in og.ui.motionEvents.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |

```ogt
struct CallbackEvent {
  doc "A callback to game code: it arrives in og.ui.motionEvents."
  field name String "Its name."
}
```

<sub>`api.ogt` line 14522</sub>

<a id="type-TimelineAction"></a>

### `TimelineAction` (union)

What an event does when a play reaches it.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "sound"` | [`SoundEvent`](#type-SoundEvent) | A UI sound. |
| `kind: "haptic"` | [`HapticEvent`](#type-HapticEvent) | A haptic. |
| `kind: "burst"` | [`BurstEvent`](#type-BurstEvent) | A burst. |
| `kind: "state"` | [`StateEvent`](#type-StateEvent) | A state change. |
| `kind: "callback"` | [`CallbackEvent`](#type-CallbackEvent) | A callback. |

```ogt
union TimelineAction {
  doc "What an event does when a play reaches it."
  tag kind
  case sound SoundEvent "A UI sound."
  case haptic HapticEvent "A haptic."
  case burst BurstEvent "A burst."
  case state StateEvent "A state change."
  case callback CallbackEvent "A callback."
}
```

<sub>`api.ogt` line 14527</sub>

<a id="type-TimelineEvent"></a>

### `TimelineEvent` (struct)

An event at a time, on the play's element or a descendant.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `action` | `TimelineAction` | — | — | — | — | What it does. |

```ogt
struct TimelineEvent {
  doc "An event at a time, on the play's element or a descendant."
  field atMs f32 "Its time." {
    unit ms
  }
  field target String "A path below the play's element (empty: the element itself)."
  field action TimelineAction "What it does."
}
```

<sub>`api.ogt` line 14537</sub>

<a id="type-TimelineLabel"></a>

### `TimelineLabel` (struct)

A named time, for seeking (PlayDesc.fromLabel).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name. |
| `atMs` | `number` | `0` | — | `ms` | — | Its time. |

```ogt
struct TimelineLabel {
  doc "A named time, for seeking (PlayDesc.fromLabel)."
  field name String "Its name."
  field atMs f32 "Its time." {
    unit ms
  }
}
```

<sub>`api.ogt` line 14546</sub>

<a id="type-UiTimeline"></a>

### `UiTimeline` (desc struct)

Keyframed tracks and events (docs/ui.md §7.4).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `tracks` | `readonly TimelineTrack[]` | `[]` | — | — | — | Its tracks. |
| `events` | `readonly TimelineEvent[]` | `[]` | — | — | — | Its events. |
| `labels` | `readonly TimelineLabel[]` | `[]` | — | — | — | Its named times. |
| `durationMs` | `number` | `0` | — | `ms` | — | Its length (0: its last key or event). |

```ogt
struct UiTimeline {
  doc "Keyframed tracks and events (docs/ui.md §7.4)."
  desc true
  field tracks List.TimelineTrack "Its tracks."
  field events List.TimelineEvent "Its events."
  field labels List.TimelineLabel "Its named times."
  field durationMs f32 "Its length (0: its last key or event)." {
    unit ms
  }
}
```

<sub>`api.ogt` line 14554</sub>

<a id="type-ChoreoPlay"></a>

### `ChoreoPlay` (struct)

A timeline on the play's element or a descendant.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `timeline` | `string` | `""` | — | — | — | The timeline, by name. |
| `target` | `string` | `""` | — | — | — | A path below the play's element (empty: the element itself). |
| `delayMs` | `number` | `0` | — | `ms` | — | Wait this long first. |

```ogt
struct ChoreoPlay {
  doc "A timeline on the play's element or a descendant."
  field timeline String "The timeline, by name."
  field target String "A path below the play's element (empty: the element itself)."
  field delayMs f32 "Wait this long first." {
    unit ms
  }
}
```

<sub>`api.ogt` line 14565</sub>

<a id="type-ChoreoSteps"></a>

### `ChoreoSteps` (struct)

Steps composed.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `steps` | `readonly Choreo[]` | `[]` | — | — | — | The steps. |

```ogt
struct ChoreoSteps {
  doc "Steps composed."
  field steps List.Choreo "The steps."
}
```

<sub>`api.ogt` line 14574</sub>

<a id="type-ChoreoStagger"></a>

### `ChoreoStagger` (struct)

One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `timeline` | `string` | `""` | — | — | — | The timeline, by name. |
| `targets` | `readonly string[]` | `[]` | — | — | — | Paths below the play's element (empty: its children in order). |
| `stepMs` | `number` | `0` | — | `ms` | — | Between one target's start and the next's (0: the style's stagger). |
| `capMs` | `number` | `0` | — | `ms` | — | The most the whole stagger spreads over (0: the style's cap). |
| `from` | `string` | `"start"` | — | — | — | Where delays grow from: start, end, center (nearest the centre of their bounds first), or `x,y` (nearest that point first). |

```ogt
struct ChoreoStagger {
  doc "One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs."
  field timeline String "The timeline, by name."
  field targets List.String "Paths below the play's element (empty: its children in order)."
  field stepMs f32 "Between one target's start and the next's (0: the style's stagger)." {
    unit ms
  }
  field capMs f32 "The most the whole stagger spreads over (0: the style's cap)." {
    unit ms
  }
  field from String "Where delays grow from: start, end, center (nearest the centre of their bounds first), or `x,y` (nearest that point first)." {
    default "start"
  }
}
```

<sub>`api.ogt` line 14579</sub>

<a id="type-ChoreoCascade"></a>

### `ChoreoCascade` (struct)

Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `steps` | `readonly Choreo[]` | `[]` | — | — | — | The steps. |
| `on` | `string` | `""` | — | — | — | The callback that starts the next step. |

```ogt
struct ChoreoCascade {
  doc "Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher."
  field steps List.Choreo "The steps."
  field on String "The callback that starts the next step."
}
```

<sub>`api.ogt` line 14594</sub>

<a id="type-Choreo"></a>

### `Choreo` (union)

Timelines composed into one play (docs/ui.md §7.4).

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "play"` | [`ChoreoPlay`](#type-ChoreoPlay) | One timeline. |
| `kind: "sequence"` | [`ChoreoSteps`](#type-ChoreoSteps) | One after another. |
| `kind: "parallel"` | [`ChoreoSteps`](#type-ChoreoSteps) | All at once. |
| `kind: "stagger"` | [`ChoreoStagger`](#type-ChoreoStagger) | One timeline over many targets. |
| `kind: "cascade"` | [`ChoreoCascade`](#type-ChoreoCascade) | Steps chained on callbacks. |

```ogt
union Choreo {
  doc "Timelines composed into one play (docs/ui.md §7.4)."
  tag kind
  case play ChoreoPlay "One timeline."
  case sequence ChoreoSteps "One after another."
  case parallel ChoreoSteps "All at once."
  case stagger ChoreoStagger "One timeline over many targets."
  case cascade ChoreoCascade "Steps chained on callbacks."
}
```

<sub>`api.ogt` line 14600</sub>

<a id="type-PlayDesc"></a>

### `PlayDesc` (desc struct)

How a play runs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `reverse` | `boolean` | `false` | — | — | — | Plays backwards. |
| `repeat` | `Repeat` | `"once"` | — | — | — | How it repeats. |
| `speed` | `number` | `0` | — | — | — | Its rate (0: 1). |
| `fromMs` | `number` | `0` | — | `ms` | — | Start this far in. |
| `fromLabel` | `string` | `""` | — | — | — | Start at a label of its first timeline (wins over fromMs). |
| `hold` | `boolean` | `false` | — | — | — | Keep the last values once it ends, until stopped. |
| `decorative` | `boolean` | `false` | — | — | — | Under reduced motion it does not play at all. |

```ogt
struct PlayDesc {
  doc "How a play runs."
  desc true
  field reverse bool "Plays backwards."
  field repeat Repeat "How it repeats."
  field speed f32 "Its rate (0: 1)."
  field fromMs f32 "Start this far in." {
    unit ms
  }
  field fromLabel String "Start at a label of its first timeline (wins over fromMs)."
  field hold bool "Keep the last values once it ends, until stopped."
  field decorative bool "Under reduced motion it does not play at all."
}
```

<sub>`api.ogt` line 14610</sub>

<a id="type-PlayInfo"></a>

### `PlayInfo` (struct)

Where a play is.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `atMs` | `number` | `0` | — | `ms` | — | Its time (before a reverse's turn). |
| `lengthMs` | `number` | `0` | — | `ms` | — | Its length. |
| `paused` | `boolean` | `false` | — | — | — | Held. |
| `reverse` | `boolean` | `false` | — | — | — | Playing backwards. |
| `ended` | `boolean` | `false` | — | — | — | Finished (a held play keeps its values). |

```ogt
struct PlayInfo {
  doc "Where a play is."
  field atMs f32 "Its time (before a reverse's turn)." {
    unit ms
  }
  field lengthMs f32 "Its length." {
    unit ms
  }
  field paused bool "Held."
  field reverse bool "Playing backwards."
  field ended bool "Finished (a held play keeps its values)."
}
```

<sub>`api.ogt` line 14624</sub>

<a id="type-GestureInfo"></a>

### `GestureInfo` (struct)

What a drag or pinch on an element is doing (last tick's answer, as responses are).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `held` | `boolean` | `false` | — | — | — | A press is held on it. |
| `dragging` | `boolean` | `false` | — | — | — | It moves with the pointer (the press travelled past the threshold). |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Its offset from where it was laid out. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Its offset down. |
| `velocityX` | `number` | `0` | — | — | — | CSS px per second. |
| `velocityY` | `number` | `0` | — | — | — | CSS px per second. |
| `scale` | `number` | `0` | — | — | — | A pinch's scale (1: none). |
| `rotation` | `number` | `0` | — | — | — | A pinch's turn, radians. |
| `snapped` | `number` | `0` | — | — | — | The snap point the last release chose (-1: none). |
| `dismissed` | `boolean` | `false` | — | — | — | A swipe threw it off. |
| `over` | `string` | `""` | — | — | — | The drop target under the drag (empty: none). |
| `droppedOn` | `string` | `""` | — | — | — | The drop target it was dropped on this tick (empty: none). |
| `hovering` | `boolean` | `false` | — | — | — | For a drop target: a drag is over it. |
| `received` | `string` | `""` | — | — | — | For a drop target: the element dropped on it this tick (empty: none). |

```ogt
struct GestureInfo {
  doc "What a drag or pinch on an element is doing (last tick's answer, as responses are)."
  field held bool "A press is held on it."
  field dragging bool "It moves with the pointer (the press travelled past the threshold)."
  field offsetX f32 "Its offset from where it was laid out." {
    unit cssPx
  }
  field offsetY f32 "Its offset down." {
    unit cssPx
  }
  field velocityX f32 "CSS px per second."
  field velocityY f32 "CSS px per second."
  field scale f32 "A pinch's scale (1: none)."
  field rotation f32 "A pinch's turn, radians."
  field snapped i32 "The snap point the last release chose (-1: none)."
  field dismissed bool "A swipe threw it off."
  field over String "The drop target under the drag (empty: none)."
  field droppedOn String "The drop target it was dropped on this tick (empty: none)."
  field hovering bool "For a drop target: a drag is over it."
  field received String "For a drop target: the element dropped on it this tick (empty: none)."
}
```

<sub>`api.ogt` line 14637</sub>

<a id="type-UiMotionEvent"></a>

### `UiMotionEvent` (struct)

What happened in the UI's motion this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `string` | `""` | — | — | — | sound, haptic, burst, state, callback, end (a play finished), dismiss (a swipe threw an element off), drop (a drag was dropped on a target) or snap (a released drag chose a snap point). |
| `name` | `string` | `""` | — | — | — | The sound, haptic, callback or state's name; for drop the target's path; for snap the point's index. |
| `target` | `string` | `""` | — | — | — | The element's path. |
| `play` | `number` | `0` | — | — | — | The play (0: none). |
| `index` | `number` | `0` | — | — | — | The clip's step in a cascade or stagger. |
| `pitch` | `number` | `0` | — | — | — | A sound's pitch: its own times 1.06 per cascade step. |

```ogt
struct UiMotionEvent {
  doc "What happened in the UI's motion this tick."
  field kind String "sound, haptic, burst, state, callback, end (a play finished), dismiss (a swipe threw an element off), drop (a drag was dropped on a target) or snap (a released drag chose a snap point)."
  field name String "The sound, haptic, callback or state's name; for drop the target's path; for snap the point's index."
  field target String "The element's path."
  field play u32 "The play (0: none)."
  field index u32 "The clip's step in a cascade or stagger."
  field pitch f32 "A sound's pitch: its own times 1.06 per cascade step."
}
```

<sub>`api.ogt` line 14659</sub>

<a id="type-UiOverlap"></a>

### `UiOverlap` (struct)

Two elements overlapping where neither declared it (docs/ui.md §6.2).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | The element placed over the other (absolute, anchored or floating), or the later of two siblings in the flow. |
| `other` | `string` | `""` | — | — | — | The other element. |
| `rect` | `UiRect` | `{ x: 0, y: 0, width: 0, height: 0 }` | — | — | — | Where they overlap. |
| `inFlow` | `boolean` | `false` | — | — | — | Both are in their parent's flow: a layout bug, not a placement. |

```ogt
struct UiOverlap {
  doc "Two elements overlapping where neither declared it (docs/ui.md §6.2)."
  field path String "The element placed over the other (absolute, anchored or floating), or the later of two siblings in the flow."
  field other String "The other element."
  field rect UiRect "Where they overlap."
  field inFlow bool "Both are in their parent's flow: a layout bug, not a placement."
}
```

<sub>`api.ogt` line 14669</sub>

<a id="type-UiLayoutStats"></a>

### `UiLayoutStats` (struct)

What the last frame rebuilt and laid out (docs/ui.md §6.3).

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `elements` | `number` | `0` | — | — | — | Elements built. |
| `rebuilt` | `number` | `0` | — | — | — | Elements whose inputs changed since the tick before (or are new). |
| `laidOut` | `number` | `0` | — | — | — | Elements laid out; the rest kept their last layout. |
| `keptCommands` | `number` | `0` | — | — | — | Elements whose draw commands were kept from the tick before. |

```ogt
struct UiLayoutStats {
  doc "What the last frame rebuilt and laid out (docs/ui.md §6.3)."
  field elements u32 "Elements built."
  field rebuilt u32 "Elements whose inputs changed since the tick before (or are new)."
  field laidOut u32 "Elements laid out; the rest kept their last layout."
  field keptCommands u32 "Elements whose draw commands were kept from the tick before."
}
```

<sub>`api.ogt` line 14677</sub>

<a id="type-VectorArt"></a>

### `VectorArt` (handle)

A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle VectorArt "A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill."
```

<sub>`api.ogt` line 14797</sub>

<a id="type-UiMaterial"></a>

### `UiMaterial` (handle)

A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through.

A generational handle: a number the engine hands out. A dead handle is an error value, never a crash.

```ogt
handle UiMaterial "A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through."
```

<sub>`api.ogt` line 14798</sub>

<a id="type-ShapeType"></a>

### `ShapeType` (enum)

An element's outline (DrawShape.kind).

A string: one of the variants below. The zero value is `"rect"`.

| Variant | Doc |
|---|---|
| `"rect"` | Its rectangle, with DrawShape.radii (or its radius) and corner. |
| `"ellipse"` | The ellipse inscribed in its rectangle (a circle when square; a capsule is a rect with radii of half its height). |
| `"polygon"` | A regular polygon of sides vertices on the circle in its rectangle, the first at the top; inner under 1 makes a star. |
| `"arc"` | A ring segment on the circle in its rectangle: start, sweep and thickness (a ring at a full turn), with caps. |
| `"path"` | A line through points (CSS px from its top left), Béziers with curves; its fills draw the line. |

```ogt
enum ShapeType {
  doc "An element's outline (DrawShape.kind)."
  variant rect "Its rectangle, with DrawShape.radii (or its radius) and corner."
  variant ellipse "The ellipse inscribed in its rectangle (a circle when square; a capsule is a rect with radii of half its height)."
  variant polygon "A regular polygon of sides vertices on the circle in its rectangle, the first at the top; inner under 1 makes a star."
  variant arc "A ring segment on the circle in its rectangle: start, sweep and thickness (a ring at a full turn), with caps."
  variant path "A line through points (CSS px from its top left), Béziers with curves; its fills draw the line."
}
```

<sub>`api.ogt` line 14814</sub>

<a id="type-CornerShape"></a>

### `CornerShape` (enum)

How a rect's corners turn.

A string: one of the variants below. The zero value is `"round"`.

| Variant | Doc |
|---|---|
| `"round"` | Circular arcs. |
| `"squircle"` | Continuous corners: a superellipse of DrawShape.squircle. |
| `"bevel"` | A straight cut across, the radius along each edge. |
| `"notch"` | A concave quarter circle cut in (an inset notch). |

```ogt
enum CornerShape {
  doc "How a rect's corners turn."
  variant round "Circular arcs."
  variant squircle "Continuous corners: a superellipse of DrawShape.squircle."
  variant bevel "A straight cut across, the radius along each edge."
  variant notch "A concave quarter circle cut in (an inset notch)."
}
```

<sub>`api.ogt` line 14823</sub>

<a id="type-LineCap"></a>

### `LineCap` (enum)

The end of an open line, an arc or a dash.

A string: one of the variants below. The zero value is `"butt"`.

| Variant | Doc |
|---|---|
| `"butt"` | Cut square at the end. |
| `"round"` | A half disc past the end. |
| `"square"` | Cut square half the width past the end. |

```ogt
enum LineCap {
  doc "The end of an open line, an arc or a dash."
  variant butt "Cut square at the end."
  variant round "A half disc past the end."
  variant square "Cut square half the width past the end."
}
```

<sub>`api.ogt` line 14831</sub>

<a id="type-LineJoin"></a>

### `LineJoin` (enum)

Where a line's segments meet.

A string: one of the variants below. The zero value is `"miter"`.

| Variant | Doc |
|---|---|
| `"miter"` | Sharp, cut to a bevel past DrawShape.miterLimit half widths. |
| `"round"` | Rounded. |
| `"bevel"` | Cut across. |

```ogt
enum LineJoin {
  doc "Where a line's segments meet."
  variant miter "Sharp, cut to a bevel past DrawShape.miterLimit half widths."
  variant round "Rounded."
  variant bevel "Cut across."
}
```

<sub>`api.ogt` line 14838</sub>

<a id="type-DrawShape"></a>

### `DrawShape` (desc struct)

An element's outline; its fills, strokes, effects and clip take it.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `ShapeType` | `"rect"` | — | — | — | Which outline. |
| `radii` | `[number, number, number, number] \| null` | `null` | — | `cssPx` | — | A rect's corner radii, top left, top right, bottom right, bottom left; null is the element's radius at every corner. |
| `corner` | `CornerShape` | `"round"` | — | — | — | A rect's corner shape. |
| `squircle` | `number` | `4` | 2 … 12 | — | — | A squircle corner's exponent: 2 is round, 4 the iOS look. |
| `sides` | `number` | `6` | 3 … 64 | — | — | A polygon's vertices. |
| `inner` | `number` | `1` | ≥ 0 | `ratio` | — | A star's inner vertices as a fraction of its radius; 1 (or more) is a plain polygon. |
| `rounding` | `number` | `0` | — | `cssPx` | — | A polygon's corner rounding. |
| `rotation` | `number` | `0` | — | `rad` | — | A polygon's turn, clockwise. |
| `start` | `number` | `0` | — | `rad` | — | An arc's start, clockwise from 3 o'clock. |
| `sweep` | `number` | `6.2831855` | — | `rad` | — | An arc's sweep, clockwise (a full turn or more: a ring). |
| `thickness` | `number` | `4` | — | `cssPx` | — | An arc's thickness, inwards from the circle. |
| `cap` | `LineCap` | `"butt"` | — | — | — | An arc's or a path's ends. |
| `points` | `readonly Vector2[]` | `[]` | — | `cssPx` | — | A path's points, CSS px from the element's top left. |
| `curves` | `boolean` | `false` | — | — | — | A path's points after the first come in threes: a cubic Bézier's two controls and its end. |
| `closed` | `boolean` | `false` | — | — | — | A path returns to its first point. |
| `width` | `number` | `2` | ≥ 0 | `cssPx` | — | A path's line width. |
| `join` | `LineJoin` | `"miter"` | — | — | — | Where a path's segments meet. |

```ogt
struct DrawShape {
  doc "An element's outline; its fills, strokes, effects and clip take it."
  desc true
  field kind ShapeType "Which outline."
  field radii Option.Array.f32.4 "A rect's corner radii, top left, top right, bottom right, bottom left; null is the element's radius at every corner." {
    unit cssPx
  }
  field corner CornerShape "A rect's corner shape."
  field squircle f32 "A squircle corner's exponent: 2 is round, 4 the iOS look." {
    default 4
    min 2
    max 12
  }
  field sides u32 "A polygon's vertices." {
    default 6
    min 3
    max 64
  }
  field inner f32 "A star's inner vertices as a fraction of its radius; 1 (or more) is a plain polygon." {
    default 1
    min 0
    unit ratio
  }
  field rounding f32 "A polygon's corner rounding." {
    unit cssPx
  }
  field rotation f32 "A polygon's turn, clockwise." {
    unit rad
  }
  field start f32 "An arc's start, clockwise from 3 o'clock." {
    unit rad
  }
  field sweep f32 "An arc's sweep, clockwise (a full turn or more: a ring)." {
    default 6.2831855
    unit rad
  }
  field thickness f32 "An arc's thickness, inwards from the circle." {
    default 4
    unit cssPx
  }
  field cap LineCap "An arc's or a path's ends."
  field points List.Vector2 "A path's points, CSS px from the element's top left." {
    unit cssPx
  }
  field curves bool "A path's points after the first come in threes: a cubic Bézier's two controls and its end."
  field closed bool "A path returns to its first point."
  field width f32 "A path's line width." {
    default 2
    min 0
    unit cssPx
  }
  field join LineJoin "Where a path's segments meet."
}
```

<sub>`api.ogt` line 14845</sub>

<a id="type-GradientType"></a>

### `GradientType` (enum)

How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right).

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | From `from` to `to`. |
| `"radial"` | Centred on `from`, `to` its x and y radii. |
| `"conic"` | Around `from`, starting at the angle to.x (radians clockwise from 3 o'clock). |
| `"diamond"` | Centred on `from`, `to` its x and y radii, by \|x\|/rx + \|y\|/ry. |

```ogt
enum GradientType {
  doc "How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right)."
  variant linear "From `from` to `to`."
  variant radial "Centred on `from`, `to` its x and y radii."
  variant conic "Around `from`, starting at the angle to.x (radians clockwise from 3 o'clock)."
  variant diamond "Centred on `from`, `to` its x and y radii, by |x|/rx + |y|/ry."
}
```

<sub>`api.ogt` line 14899</sub>

<a id="type-GradientSpread"></a>

### `GradientSpread` (enum)

What a gradient does past its ends.

A string: one of the variants below. The zero value is `"pad"`.

| Variant | Doc |
|---|---|
| `"pad"` | Holds the end stops' colours. |
| `"repeat"` | Starts over. |
| `"reflect"` | Runs back and forth. |

```ogt
enum GradientSpread {
  doc "What a gradient does past its ends."
  variant pad "Holds the end stops' colours."
  variant repeat "Starts over."
  variant reflect "Runs back and forth."
}
```

<sub>`api.ogt` line 14907</sub>

<a id="type-GradientStop"></a>

### `GradientStop` (desc struct)

A gradient's colour at an offset.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `number` | `0` | 0 … 1 | `ratio` | — | Where along the gradient, 0 to 1. |
| `color` | `Paint` | `{}` | — | — | — | Its colour: a theme token, resolved when the frame is drawn, or a literal. |

```ogt
struct GradientStop {
  doc "A gradient's colour at an offset."
  desc true
  field offset f32 "Where along the gradient, 0 to 1." {
    min 0
    max 1
    unit ratio
  }
  field color Paint "Its colour: a theme token, resolved when the frame is drawn, or a literal."
}
```

<sub>`api.ogt` line 14914</sub>

<a id="type-Gradient"></a>

### `Gradient` (desc struct)

A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `type` | `GradientType` | `"linear"` | — | — | — | Its shape. |
| `from` | `Vector2` | `{ x: 0, y: 0 }` | — | `ratio` | — | Its start or centre, a fraction of the box. |
| `to` | `Vector2` | `{ x: 0, y: 1 }` | — | `ratio` | — | Its end, or its radii, a fraction of the box. |
| `stops` | `readonly GradientStop[]` | `[]` | — | — | — | Its stops in order (at most 8). |
| `spread` | `GradientSpread` | `"pad"` | — | — | — | Past its ends. |
| `noDither` | `boolean` | `false` | — | — | — | Draws it without the dither. |

```ogt
struct Gradient {
  doc "A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands."
  desc true
  field type GradientType "Its shape."
  field from Vector2 "Its start or centre, a fraction of the box." {
    unit ratio
  }
  field to Vector2 "Its end, or its radii, a fraction of the box." {
    default 0 1
    unit ratio
  }
  field stops List.GradientStop "Its stops in order (at most 8)."
  field spread GradientSpread "Past its ends."
  field noDither bool "Draws it without the dither."
}
```

<sub>`api.ogt` line 14925</sub>

<a id="type-ImageFit"></a>

### `ImageFit` (enum)

How an image fills the element's box.

A string: one of the variants below. The zero value is `"fill"`.

| Variant | Doc |
|---|---|
| `"fill"` | Stretched to the box. |
| `"cover"` | Scaled to cover the box, cropped around the focal point. |
| `"contain"` | Scaled to fit inside the box, placed by the focal point. |
| `"none"` | At its own size (one image pixel per CSS px), placed by the focal point. |
| `"nineSlice"` | Nine-slice: corners kept at slice, edges and centre filled by edges. |
| `"pattern"` | Tiled at tile CSS px, turned, offset and scrolling. |

```ogt
enum ImageFit {
  doc "How an image fills the element's box."
  variant fill "Stretched to the box."
  variant cover "Scaled to cover the box, cropped around the focal point."
  variant contain "Scaled to fit inside the box, placed by the focal point."
  variant none "At its own size (one image pixel per CSS px), placed by the focal point."
  variant nineSlice "Nine-slice: corners kept at slice, edges and centre filled by edges."
  variant pattern "Tiled at tile CSS px, turned, offset and scrolling."
}
```

<sub>`api.ogt` line 14941</sub>

<a id="type-SliceEdges"></a>

### `SliceEdges` (enum)

How a nine-slice's edges and centre fill their length.

A string: one of the variants below. The zero value is `"stretch"`.

| Variant | Doc |
|---|---|
| `"stretch"` | Stretched. |
| `"repeat"` | Tiled at the border's scale, cut at the end. |
| `"round"` | Tiled a whole number of times, stretched to fit. |

```ogt
enum SliceEdges {
  doc "How a nine-slice's edges and centre fill their length."
  variant stretch "Stretched."
  variant repeat "Tiled at the border's scale, cut at the end."
  variant round "Tiled a whole number of times, stretched to fit."
}
```

<sub>`api.ogt` line 14951</sub>

<a id="type-SolidFill"></a>

### `SolidFill` (desc struct)

A colour.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `color` | `Paint` | `{}` | — | — | — | The colour: a theme token, resolved when the frame is drawn, or a literal. |

```ogt
struct SolidFill {
  doc "A colour."
  desc true
  field color Paint "The colour: a theme token, resolved when the frame is drawn, or a literal."
}
```

<sub>`api.ogt` line 14958</sub>

<a id="type-ImageFill"></a>

### `ImageFill` (desc struct)

A texture region with a fit.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture` | `0` | — | — | — | The texture (og.load.texture). |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The region, [u0, v0, u1, v1] with v down; null is the whole texture. |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The region's size in image pixels (none, nineSlice and pattern read it); zero is the texture's. |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies it; null is white. |
| `fit` | `ImageFit` | `"fill"` | — | — | — | How it fills the box. |
| `focal` | `Vector2` | `{ x: 0.5, y: 0.5 }` | — | `ratio` | — | Where cover crops and contain or none place it, fractions of the free space. |
| `slice` | `[number, number, number, number]` | `[0, 0, 0, 0]` | — | — | — | A nine-slice's border in image pixels: left, top, right, bottom. |
| `border` | `[number, number, number, number] \| null` | `null` | — | — | — | A nine-slice's border as drawn, CSS px; null is slice. |
| `edges` | `SliceEdges` | `"stretch"` | — | — | — | A nine-slice's edges and centre. |
| `hollow` | `boolean` | `false` | — | — | — | A nine-slice leaves its centre empty. |
| `tile` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's tile, CSS px; zero is size. |
| `patternRotation` | `number` | `0` | — | `rad` | — | A pattern's turn. |
| `offset` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's offset. |
| `scroll` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | A pattern's scroll, CSS px a second of the UI clock. |

```ogt
struct ImageFill {
  doc "A texture region with a fit."
  desc true
  field texture Texture "The texture (og.load.texture)."
  field uv Option.Array.f32.4 "The region, [u0, v0, u1, v1] with v down; null is the whole texture."
  field size Vector2 "The region's size in image pixels (none, nineSlice and pattern read it); zero is the texture's."
  field tint Option.Paint "Multiplies it; null is white."
  field fit ImageFit "How it fills the box."
  field focal Vector2 "Where cover crops and contain or none place it, fractions of the free space." {
    default 0.5 0.5
    unit ratio
  }
  field slice Array.f32.4 "A nine-slice's border in image pixels: left, top, right, bottom."
  field border Option.Array.f32.4 "A nine-slice's border as drawn, CSS px; null is slice."
  field edges SliceEdges "A nine-slice's edges and centre."
  field hollow bool "A nine-slice leaves its centre empty."
  field tile Vector2 "A pattern's tile, CSS px; zero is size." {
    unit cssPx
  }
  field patternRotation f32 "A pattern's turn." {
    unit rad
  }
  field offset Vector2 "A pattern's offset." {
    unit cssPx
  }
  field scroll Vector2 "A pattern's scroll, CSS px a second of the UI clock." {
    unit cssPx
  }
}
```

<sub>`api.ogt` line 14964</sub>

<a id="type-VectorFill"></a>

### `VectorFill` (desc struct)

A cooked vector contained and centred in the box.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `vector` | `VectorArt` | `0` | — | — | — | The vector (og.load.vector). |
| `tint` | `Paint \| null` | `null` | — | — | — | Multiplies its paints; null is white. |

```ogt
struct VectorFill {
  doc "A cooked vector contained and centred in the box."
  desc true
  field vector VectorArt "The vector (og.load.vector)."
  field tint Option.Paint "Multiplies its paints; null is white."
}
```

<sub>`api.ogt` line 14994</sub>

<a id="type-Fill"></a>

### `Fill` (union)

One layer of an element's fill, or a stroke's paint.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "solid"` | [`SolidFill`](#type-SolidFill) | A colour. |
| `kind: "gradient"` | [`Gradient`](#type-Gradient) | A gradient. |
| `kind: "image"` | [`ImageFill`](#type-ImageFill) | An image with a fit, a nine-slice or a pattern. |
| `kind: "vector"` | [`VectorFill`](#type-VectorFill) | A cooked vector (as a stroke's paint, its tint). |

```ogt
union Fill {
  doc "One layer of an element's fill, or a stroke's paint."
  tag kind
  case solid SolidFill "A colour."
  case gradient Gradient "A gradient."
  case image ImageFill "An image with a fit, a nine-slice or a pattern."
  case vector VectorFill "A cooked vector (as a stroke's paint, its tint)."
}
```

<sub>`api.ogt` line 15001</sub>

<a id="type-StrokeAlign"></a>

### `StrokeAlign` (enum)

Which side of the outline a stroke lies on.

A string: one of the variants below. The zero value is `"center"`.

| Variant | Doc |
|---|---|
| `"center"` | Across the outline. |
| `"inside"` | Inside it. |
| `"outside"` | Outside it. |

```ogt
enum StrokeAlign {
  doc "Which side of the outline a stroke lies on."
  variant center "Across the outline."
  variant inside "Inside it."
  variant outside "Outside it."
}
```

<sub>`api.ogt` line 15010</sub>

<a id="type-Stroke"></a>

### `Stroke` (desc struct)

A stroke along the element's outline with a paint of its own (a gradient border is one stroke).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `1` | ≥ 0 | `cssPx` | — | Its width. |
| `align` | `StrokeAlign` | `"center"` | — | — | — | Which side of the outline. |
| `dash` | `number` | `0` | ≥ 0 | `cssPx` | — | A dash's length; 0 is solid. |
| `gap` | `number` | `0` | ≥ 0 | `cssPx` | — | The gap between dashes; 0 is the dash's length. |
| `dashOffset` | `number` | `0` | — | `cssPx` | — | Where the dashes start along the outline (move it by the tick for marching ants). |
| `dashCap` | `LineCap` | `"butt"` | — | — | — | The dashes' ends. |
| `paint` | `Fill \| null` | `null` | — | — | — | Its paint; null is the theme's line colour. |

```ogt
struct Stroke {
  doc "A stroke along the element's outline with a paint of its own (a gradient border is one stroke)."
  desc true
  field width f32 "Its width." {
    default 1
    min 0
    unit cssPx
  }
  field align StrokeAlign "Which side of the outline."
  field dash f32 "A dash's length; 0 is solid." {
    min 0
    unit cssPx
  }
  field gap f32 "The gap between dashes; 0 is the dash's length." {
    min 0
    unit cssPx
  }
  field dashOffset f32 "Where the dashes start along the outline (move it by the tick for marching ants)." {
    unit cssPx
  }
  field dashCap LineCap "The dashes' ends."
  field paint Option.Fill "Its paint; null is the theme's line colour."
}
```

<sub>`api.ogt` line 15017</sub>

<a id="type-ShadowEffect"></a>

### `ShadowEffect` (desc struct)

A shadow: a drop shadow under the element, or an inner shadow cast inwards.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 4 }` | — | `cssPx` | — | How far it falls. |
| `blur` | `number` | `12` | ≥ 0 | `cssPx` | — | Its softness (CSS blur: the Gaussian's deviation is half of it). |
| `spread` | `number` | `0` | — | `cssPx` | — | How far it grows (or with a negative value shrinks) past the shape. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |

```ogt
struct ShadowEffect {
  doc "A shadow: a drop shadow under the element, or an inner shadow cast inwards."
  desc true
  field offset Vector2 "How far it falls." {
    default 0 4
    unit cssPx
  }
  field blur f32 "Its softness (CSS blur: the Gaussian's deviation is half of it)." {
    default 12
    min 0
    unit cssPx
  }
  field spread f32 "How far it grows (or with a negative value shrinks) past the shape." {
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
}
```

<sub>`api.ogt` line 15041</sub>

<a id="type-GlowEffect"></a>

### `GlowEffect` (desc struct)

A glow: around the element, or along its inside.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `blur` | `number` | `12` | ≥ 0 | `cssPx` | — | Its softness. |
| `spread` | `number` | `0` | — | `cssPx` | — | How far it grows past the shape. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's primary. |

```ogt
struct GlowEffect {
  doc "A glow: around the element, or along its inside."
  desc true
  field blur f32 "Its softness." {
    default 12
    min 0
    unit cssPx
  }
  field spread f32 "How far it grows past the shape." {
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's primary."
}
```

<sub>`api.ogt` line 15059</sub>

<a id="type-BlurEffect"></a>

### `BlurEffect` (desc struct)

The element and its subtree blurred (drawn offscreen).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `radius` | `number` | `4` | ≥ 0 | `cssPx` | — | The blur. |

```ogt
struct BlurEffect {
  doc "The element and its subtree blurred (drawn offscreen)."
  desc true
  field radius f32 "The blur." {
    default 4
    min 0
    unit cssPx
  }
}
```

<sub>`api.ogt` line 15073</sub>

<a id="type-BackdropEffect"></a>

### `BackdropEffect` (desc struct)

What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `blur` | `number` | `16` | ≥ 0 | `cssPx` | — | The blur. |
| `saturation` | `number` | `1` | ≥ 0 | `ratio` | — | Multiplies the saturation. |
| `brightness` | `number` | `1` | ≥ 0 | `ratio` | — | Multiplies the brightness. |
| `tint` | `Paint \| null` | `null` | — | — | — | Over the blur, by its alpha; null is the theme's panel at 60 %. |

```ogt
struct BackdropEffect {
  doc "What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass."
  desc true
  field blur f32 "The blur." {
    default 16
    min 0
    unit cssPx
  }
  field saturation f32 "Multiplies the saturation." {
    default 1
    min 0
    unit ratio
  }
  field brightness f32 "Multiplies the brightness." {
    default 1
    min 0
    unit ratio
  }
  field tint Option.Paint "Over the blur, by its alpha; null is the theme's panel at 60 %."
}
```

<sub>`api.ogt` line 15083</sub>

<a id="type-Effect"></a>

### `Effect` (union)

An effect of the element.

Tagged by `kind`.

| Case | Payload | Doc |
|---|---|---|
| `kind: "dropShadow"` | [`ShadowEffect`](#type-ShadowEffect) | A drop shadow (several layer). |
| `kind: "innerShadow"` | [`ShadowEffect`](#type-ShadowEffect) | An inner shadow. |
| `kind: "outerGlow"` | [`GlowEffect`](#type-GlowEffect) | A glow around it. |
| `kind: "innerGlow"` | [`GlowEffect`](#type-GlowEffect) | A glow along its inside. |
| `kind: "layerBlur"` | [`BlurEffect`](#type-BlurEffect) | Its subtree blurred. |
| `kind: "backdrop"` | [`BackdropEffect`](#type-BackdropEffect) | Glass: what is under it blurred. |

```ogt
union Effect {
  doc "An effect of the element."
  tag kind
  case dropShadow ShadowEffect "A drop shadow (several layer)."
  case innerShadow ShadowEffect "An inner shadow."
  case outerGlow GlowEffect "A glow around it."
  case innerGlow GlowEffect "A glow along its inside."
  case layerBlur BlurEffect "Its subtree blurred."
  case backdrop BackdropEffect "Glass: what is under it blurred."
}
```

<sub>`api.ogt` line 15104</sub>

<a id="type-ClipMode"></a>

### `ClipMode` (enum)

How an element cuts its children.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Not at all (a scroll container still clips to its rectangle). |
| `"shape"` | To its shape, antialiased (a round avatar, a hex frame). |
| `"mask"` | By DrawDesc.mask's alpha (the subtree draws offscreen). |

```ogt
enum ClipMode {
  doc "How an element cuts its children."
  variant none "Not at all (a scroll container still clips to its rectangle)."
  variant shape "To its shape, antialiased (a round avatar, a hex frame)."
  variant mask "By DrawDesc.mask's alpha (the subtree draws offscreen)."
}
```

<sub>`api.ogt` line 15115</sub>

<a id="type-MaskDesc"></a>

### `MaskDesc` (desc struct)

An alpha mask over an element's subtree: a gradient's alpha, else an image's.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `gradient` | `Gradient \| null` | `null` | — | — | — | The gradient whose alpha masks (a list fading at its edge). |
| `texture` | `Texture \| null` | `null` | — | — | — | The image whose alpha masks, stretched over the element. |
| `uv` | `[number, number, number, number] \| null` | `null` | — | — | — | The image's region; null is the whole texture. |

```ogt
struct MaskDesc {
  doc "An alpha mask over an element's subtree: a gradient's alpha, else an image's."
  desc true
  field gradient Option.Gradient "The gradient whose alpha masks (a list fading at its edge)."
  field texture Option.Texture "The image whose alpha masks, stretched over the element."
  field uv Option.Array.f32.4 "The image's region; null is the whole texture."
}
```

<sub>`api.ogt` line 15122</sub>

<a id="type-MaterialName"></a>

### `MaterialName` (enum)

What a layer composites through: a built-in UI material, or a custom one. params is read per material.

A string: one of the variants below. The zero value is `"plain"`.

| Variant | Doc |
|---|---|
| `"plain"` | The layer as drawn. |
| `"dissolve"` | [threshold 0–1, edge width, noise scale (CSS px), _], [edge colour rgba]. |
| `"shine"` | [position 0–1 across the diagonal (negative sweeps by the clock at -position a second), width, angle (rad), intensity], [colour rgba]. |
| `"hologram"` | [scanline spacing (CSS px), flicker 0–1, chromatic split (CSS px), scanline strength], [tint rgba]. |
| `"glitch"` | [strength 0–1, slice height (CSS px), split (CSS px), rate (Hz)]. |
| `"ripple"` | [amplitude (CSS px), wavelength (CSS px), speed (CSS px/s), _], [centre x, y as fractions]. |
| `"pixelate"` | [cell size (CSS px)]. |
| `"colorAdjust"` | [hue (rad), saturation, brightness, grayscale 0–1]. |
| `"outline"` | [width (CSS px), _, _, _], [colour rgba]: an outline of the layer's alpha. |
| `"blur"` | [radius (CSS px)]. |
| `"custom"` | LayerDesc.custom, a `type ui-material`. |

```ogt
enum MaterialName {
  doc "What a layer composites through: a built-in UI material, or a custom one. params is read per material."
  variant plain "The layer as drawn."
  variant dissolve "[threshold 0–1, edge width, noise scale (CSS px), _], [edge colour rgba]."
  variant shine "[position 0–1 across the diagonal (negative sweeps by the clock at -position a second), width, angle (rad), intensity], [colour rgba]."
  variant hologram "[scanline spacing (CSS px), flicker 0–1, chromatic split (CSS px), scanline strength], [tint rgba]."
  variant glitch "[strength 0–1, slice height (CSS px), split (CSS px), rate (Hz)]."
  variant ripple "[amplitude (CSS px), wavelength (CSS px), speed (CSS px/s), _], [centre x, y as fractions]."
  variant pixelate "[cell size (CSS px)]."
  variant colorAdjust "[hue (rad), saturation, brightness, grayscale 0–1]."
  variant outline "[width (CSS px), _, _, _], [colour rgba]: an outline of the layer's alpha."
  variant blur "[radius (CSS px)]."
  variant custom "LayerDesc.custom, a `type ui-material`."
}
```

<sub>`api.ogt` line 15130</sub>

<a id="type-LayerDesc"></a>

### `LayerDesc` (desc struct)

The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `material` | `MaterialName` | `"plain"` | — | — | — | The material. |
| `custom` | `UiMaterial \| null` | `null` | — | — | — | The custom material, for material custom. |
| `params` | `readonly number[]` | `[]` | — | — | — | Its parameters: up to four vec4s, flattened (16 numbers); for a custom material, missing ones are its defaults. |

```ogt
struct LayerDesc {
  doc "The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel)."
  desc true
  field material MaterialName "The material."
  field custom Option.UiMaterial "The custom material, for material custom."
  field params List.f32 "Its parameters: up to four vec4s, flattened (16 numbers); for a custom material, missing ones are its defaults."
}
```

<sub>`api.ogt` line 15145</sub>

<a id="type-TextShadowDesc"></a>

### `TextShadowDesc` (desc struct)

A shadow under an element's text.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `offset` | `Vector2` | `{ x: 0, y: 2 }` | — | `cssPx` | — | How far it falls. |
| `blur` | `number` | `2` | ≥ 0 | `cssPx` | — | Its softness. |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |

```ogt
struct TextShadowDesc {
  doc "A shadow under an element's text."
  desc true
  field offset Vector2 "How far it falls." {
    default 0 2
    unit cssPx
  }
  field blur f32 "Its softness." {
    default 2
    min 0
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
}
```

<sub>`api.ogt` line 15153</sub>

<a id="type-TextOutlineDesc"></a>

### `TextOutlineDesc` (desc struct)

An outline around an element's glyphs.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `2` | ≥ 0 | `cssPx` | — | Its width (up to the font's distance range). |
| `color` | `Paint \| null` | `null` | — | — | — | Its colour; null is the theme's shadow. |
| `align` | `StrokeAlign` | `"center"` | — | — | — | Across the glyph's edge, inside it or outside it. |

```ogt
struct TextOutlineDesc {
  doc "An outline around an element's glyphs."
  desc true
  field width f32 "Its width (up to the font's distance range)." {
    default 2
    min 0
    unit cssPx
  }
  field color Option.Paint "Its colour; null is the theme's shadow."
  field align StrokeAlign "Across the glyph's edge, inside it or outside it."
}
```

<sub>`api.ogt` line 15168</sub>

<a id="type-TextEffectsDesc"></a>

### `TextEffectsDesc` (desc struct)

Effects of the text an element holds (its labels and rich text).

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shadows` | `readonly TextShadowDesc[]` | `[]` | — | — | — | Shadows, under everything, in order. |
| `outline` | `TextOutlineDesc \| null` | `null` | — | — | — | An outline. |
| `fill` | `Gradient \| null` | `null` | — | — | — | A gradient fill over each line's box instead of its colour. |

```ogt
struct TextEffectsDesc {
  doc "Effects of the text an element holds (its labels and rich text)."
  desc true
  field shadows List.TextShadowDesc "Shadows, under everything, in order."
  field outline Option.TextOutlineDesc "An outline."
  field fill Option.Gradient "A gradient fill over each line's box instead of its colour."
}
```

<sub>`api.ogt` line 15180</sub>

<a id="type-DrawDesc"></a>

### `DrawDesc` (desc struct)

What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen through a material. Colours given as theme tokens follow the style; every value is data an agent can read and set.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `shape` | `DrawShape \| null` | `null` | — | — | — | Its outline; null is its rectangle with its radius. |
| `fills` | `readonly Fill[]` | `[]` | — | — | — | Fills, bottom first. |
| `strokes` | `readonly Stroke[]` | `[]` | — | — | — | Strokes, over the fills and the children, bottom first. |
| `effects` | `readonly Effect[]` | `[]` | — | — | — | Shadows, glows, blurs and backdrops. |
| `opacity` | `number` | `1` | 0 … 1 | — | — | Multiplies its paints (with a layer, its composite). |
| `blend` | `BlendMode` | `"normal"` | — | — | — | How its paints (or its layer) mix with what is under them. |
| `clip` | `ClipMode` | `"none"` | — | — | — | How it cuts its children. |
| `mask` | `MaskDesc \| null` | `null` | — | — | — | The mask, for clip mask. |
| `layer` | `LayerDesc \| null` | `null` | — | — | — | Its subtree drawn offscreen and composited through a material. |
| `text` | `TextEffectsDesc \| null` | `null` | — | — | — | Effects of its text. |

```ogt
struct DrawDesc {
  doc "What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen through a material. Colours given as theme tokens follow the style; every value is data an agent can read and set."
  desc true
  field shape Option.DrawShape "Its outline; null is its rectangle with its radius."
  field fills List.Fill "Fills, bottom first."
  field strokes List.Stroke "Strokes, over the fills and the children, bottom first."
  field effects List.Effect "Shadows, glows, blurs and backdrops."
  field opacity f32 "Multiplies its paints (with a layer, its composite)." {
    default 1
    min 0
    max 1
  }
  field blend BlendMode "How its paints (or its layer) mix with what is under them."
  field clip ClipMode "How it cuts its children."
  field mask Option.MaskDesc "The mask, for clip mask."
  field layer Option.LayerDesc "Its subtree drawn offscreen and composited through a material."
  field text Option.TextEffectsDesc "Effects of its text."
}
```

<sub>`api.ogt` line 15188</sub>

<a id="type-BlendMode"></a>

### `BlendMode` (enum)

How a paint mixes with what is under it.

A string: one of the variants below. The zero value is `"normal"`.

| Variant | Doc |
|---|---|
| `"normal"` | Over it. |
| `"multiply"` | Multiplied: darkens. |
| `"screen"` | Screened: lightens. |
| `"add"` | Added: glows. |
| `"overlay"` | Multiply under half grey, screen over it. |
| `"softLight"` | A gentle overlay. |
| `"colorDodge"` | Brightens what is under it by the paint. |

```ogt
enum BlendMode {
  doc "How a paint mixes with what is under it."
  variant normal "Over it."
  variant multiply "Multiplied: darkens."
  variant screen "Screened: lightens."
  variant add "Added: glows."
  variant overlay "Multiply under half grey, screen over it."
  variant softLight "A gentle overlay."
  variant colorDodge "Brightens what is under it by the paint."
}
```

<sub>`api.ogt` line 15207</sub>

<a id="type-WorldTarget"></a>

### `WorldTarget` (desc struct)

What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `entity` | `Entity \| null` | `null` | — | — | — | The entity; null places it on point. |
| `point` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | The world point, when entity is null. |

```ogt
struct WorldTarget {
  doc "What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point."
  desc true
  field entity Option.Entity "The entity; null places it on point."
  field point Vector3 "The world point, when entity is null."
}
```

<sub>`api.ogt` line 15226</sub>

<a id="type-AnchorScaleMode"></a>

### `AnchorScaleMode` (enum)

How an anchored element's size follows its distance from the camera.

A string: one of the variants below. The zero value is `"constant"`.

| Variant | Doc |
|---|---|
| `"constant"` | Its laid-out size at every distance (a nameplate that stays readable). |
| `"distance"` | 1 at reference world units, larger nearer and smaller farther, between minScale and maxScale (a tag that feels in the world). |

```ogt
enum AnchorScaleMode {
  doc "How an anchored element's size follows its distance from the camera."
  variant constant "Its laid-out size at every distance (a nameplate that stays readable)."
  variant distance "1 at reference world units, larger nearer and smaller farther, between minScale and maxScale (a tag that feels in the world)."
}
```

<sub>`api.ogt` line 15233</sub>

<a id="type-WorldOcclusion"></a>

### `WorldOcclusion` (enum)

What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | Drawn over the scene. |
| `"fade"` | Drawn at occludedAlpha where the scene is in front. |
| `"hide"` | Not drawn where the scene is in front. |

```ogt
enum WorldOcclusion {
  doc "What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU."
  variant none "Drawn over the scene."
  variant fade "Drawn at occludedAlpha where the scene is in front."
  variant hide "Not drawn where the scene is in front."
}
```

<sub>`api.ogt` line 15239</sub>

<a id="type-Declutter"></a>

### `Declutter` (enum)

How an anchored element that overlaps one of higher priority (or as high and nearer) resolves.

A string: one of the variants below. The zero value is `"none"`.

| Variant | Doc |
|---|---|
| `"none"` | It overlaps. |
| `"shift"` | Moved the shortest way clear (else stacked above). |
| `"stack"` | Moved above what it overlaps, one over another (a crowd's nameplates). |
| `"hide"` | Faded out while it overlaps. |

```ogt
enum Declutter {
  doc "How an anchored element that overlaps one of higher priority (or as high and nearer) resolves."
  variant none "It overlaps."
  variant shift "Moved the shortest way clear (else stacked above)."
  variant stack "Moved above what it overlaps, one over another (a crowd's nameplates)."
  variant hide "Faded out while it overlaps."
}
```

<sub>`api.ogt` line 15246</sub>

<a id="type-EdgeClampDesc"></a>

### `EdgeClampDesc` (desc struct)

Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `margin` | `number` | `24` | ≥ 0 | `cssPx` | — | Inset from the safe area's edges. |
| `arrow` | `Paint \| null` | `null` | — | — | — | The arrow's ink; null draws none. |
| `arrowSize` | `number` | `14` | ≥ 0 | `cssPx` | — | The arrow's length. |

```ogt
struct EdgeClampDesc {
  doc "Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there."
  desc true
  field margin f32 "Inset from the safe area's edges." {
    default 24
    unit cssPx
    min 0
  }
  field arrow Option.Paint "The arrow's ink; null draws none."
  field arrowSize f32 "The arrow's length." {
    default 14
    unit cssPx
    min 0
  }
}
```

<sub>`api.ogt` line 15254</sub>

<a id="type-WorldAnchorDesc"></a>

### `WorldAnchorDesc` (desc struct)

An element anchored in the world (docs/ui.md §14): laid out at its own size and placed each tick on its target's point projected with the tick's camera. A nameplate, an interaction prompt, a speech bubble, a waypoint.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `WorldTarget` | `{}` | — | — | — | What it sits on. |
| `offset` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Added to the point in world units (above a head: 0, 2, 0). |
| `screenOffsetX` | `number` | `0` | — | `cssPx` | — | Moves it right on the screen. |
| `screenOffsetY` | `number` | `0` | — | `cssPx` | — | Moves it down on the screen. |
| `pivot` | `Anchor` | `"topLeft"` | — | — | — | Its point that sits on the projected point; the default is its bottom centre. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children; the default is a column. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `align` | `Align` | `"start"` | — | — | — | Where children sit along the main axis. |
| `cross` | `Cross` | `"start"` | — | — | — | Where children sit across it. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `scale` | `AnchorScaleMode` | `"constant"` | — | — | — | How its size follows its distance. |
| `reference` | `number` | `10` | ≥ 0 | `m` | — | For scale distance: the distance at which it is its laid-out size. |
| `minScale` | `number` | `0.25` | ≥ 0 | — | — | For scale distance: the smallest it gets. |
| `maxScale` | `number` | `2` | ≥ 0 | — | — | For scale distance: the largest it gets. |
| `occlusion` | `WorldOcclusion` | `"none"` | — | — | — | What the scene in front of it does. |
| `occludedAlpha` | `number` | `0.25` | 0 … 1 | — | — | Its opacity behind the scene, for occlusion fade. |
| `lod` | `readonly number[]` | `[]` | — | — | — | Level-of-detail distances, ascending: level k from lod[k - 1] on (AnchorInfo.lod; build less at a higher level). |
| `maxDistance` | `number` | `0` | ≥ 0 | `m` | — | Hidden past this distance, fading over its last tenth; 0 never. |
| `declutter` | `Declutter` | `"none"` | — | — | — | How it resolves against the anchors it overlaps. |
| `priority` | `number` | `0` | — | — | — | Higher keeps its place; equal priorities favour the nearer. |
| `clamp` | `EdgeClampDesc \| null` | `null` | — | — | — | Edge clamping; null lets it leave the screen. |
| `fadeTicks` | `number` | `0` | — | — | — | Ticks a fade (declutter, distance, appearing) takes; 0 is 6. |

```ogt
struct WorldAnchorDesc {
  doc "An element anchored in the world (docs/ui.md §14): laid out at its own size and placed each tick on its target's point projected with the tick's camera. A nameplate, an interaction prompt, a speech bubble, a waypoint."
  desc true
  field target WorldTarget "What it sits on."
  field offset Vector3 "Added to the point in world units (above a head: 0, 2, 0)."
  field screenOffsetX f32 "Moves it right on the screen." {
    unit cssPx
  }
  field screenOffsetY f32 "Moves it down on the screen." {
    unit cssPx
  }
  field pivot Anchor "Its point that sits on the projected point; the default is its bottom centre."
  field direction Direction "Which way it lays out its children; the default is a column."
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
  field scale AnchorScaleMode "How its size follows its distance."
  field reference f32 "For scale distance: the distance at which it is its laid-out size." {
    default 10
    unit m
    min 0
  }
  field minScale f32 "For scale distance: the smallest it gets." {
    default 0.25
    min 0
  }
  field maxScale f32 "For scale distance: the largest it gets." {
    default 2
    min 0
  }
  field occlusion WorldOcclusion "What the scene in front of it does."
  field occludedAlpha f32 "Its opacity behind the scene, for occlusion fade." {
    default 0.25
    min 0
    max 1
  }
  field lod List.f32 "Level-of-detail distances, ascending: level k from lod[k - 1] on (AnchorInfo.lod; build less at a higher level)."
  field maxDistance f32 "Hidden past this distance, fading over its last tenth; 0 never." {
    unit m
    min 0
  }
  field declutter Declutter "How it resolves against the anchors it overlaps."
  field priority i32 "Higher keeps its place; equal priorities favour the nearer."
  field clamp Option.EdgeClampDesc "Edge clamping; null lets it leave the screen."
  field fadeTicks u32 "Ticks a fade (declutter, distance, appearing) takes; 0 is 6."
}
```

<sub>`api.ogt` line 15270</sub>

<a id="type-AnchorInfo"></a>

### `AnchorInfo` (struct)

Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `inView` | `boolean` | `false` | — | — | — | Its point is in front of the camera and within maxDistance (or clamped to the edge). |
| `onScreen` | `boolean` | `false` | — | — | — | Its point projects inside the screen. |
| `clamped` | `boolean` | `false` | — | — | — | Held on the screen's edge. |
| `lod` | `number` | `0` | — | — | — | Its level of detail. |
| `distance` | `number` | `0` | — | `m` | — | From the camera. |
| `x` | `number` | `0` | — | `cssPx` | — | Where its pivot was drawn. |
| `y` | `number` | `0` | — | `cssPx` | — | Where its pivot was drawn. |
| `angle` | `number` | `0` | — | `rad` | — | Toward its point from the screen's centre, clockwise from +x (the edge arrow's). |
| `scale` | `number` | `0` | — | — | — | The scale it was drawn at. |
| `alpha` | `number` | `0` | — | `ratio` | — | Its eased opacity (declutter, distance). |

```ogt
struct AnchorInfo {
  doc "Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it."
  field inView bool "Its point is in front of the camera and within maxDistance (or clamped to the edge)."
  field onScreen bool "Its point projects inside the screen."
  field clamped bool "Held on the screen's edge."
  field lod u32 "Its level of detail."
  field distance f32 "From the camera." {
    unit m
  }
  field x f32 "Where its pivot was drawn." {
    unit cssPx
  }
  field y f32 "Where its pivot was drawn." {
    unit cssPx
  }
  field angle f32 "Toward its point from the screen's centre, clockwise from +x (the edge arrow's)." {
    unit rad
  }
  field scale f32 "The scale it was drawn at."
  field alpha f32 "Its eased opacity (declutter, distance)." {
    unit ratio
  }
}
```

<sub>`api.ogt` line 15324</sub>

<a id="type-WorldPanelDesc"></a>

### `WorldPanelDesc` (desc struct)

A world panel (docs/ui.md §14): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `target` | `Texture \| null` | `null` | — | — | — | The colour of an og.render.renderTarget made with format rgba16float (RenderTargetInfo.texture); give the mesh's material it as map (lit) or emissiveMap with colour black (unlit). It shows the frame after the UI drew it. Null: laid out and hit, drawn nowhere. |
| `width` | `number` | `480` | ≥ 1 | `cssPx` | — | Its layout width; the target's pixels spread over it. |
| `height` | `number` | `320` | ≥ 1 | `cssPx` | — | Its layout height. |
| `quad` | `WorldTarget \| null` | `null` | — | — | — | Where its quad is: centred on the target's pose, facing its +Z (an og.render.planeGeometry of quadWidth by quadHeight on that entity). Null: no ray reaches it. |
| `rotation` | `Quaternion` | `{ x: 0, y: 0, z: 0, w: 1 }` | — | — | — | The quad's rotation after its target's. |
| `quadWidth` | `number` | `1` | ≥ 0 | `m` | — | The quad's width along its +X. |
| `quadHeight` | `number` | `1` | ≥ 0 | `m` | — | The quad's height along its +Y. |
| `direction` | `Direction` | `"row"` | — | — | — | Which way it lays out its children; the default is a column. |
| `gap` | `number` | `0` | — | `cssPx` | — | Space between children. |
| `padding` | `Insets` | `{ left: 0, top: 0, right: 0, bottom: 0 }` | — | — | — | Space inside its edges. |
| `background` | `Paint \| null` | `null` | — | — | — | Its fill; null draws none (the target clears to transparent). |

```ogt
struct WorldPanelDesc {
  doc "A world panel (docs/ui.md §14): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad."
  desc true
  field target Option.Texture "The colour of an og.render.renderTarget made with format rgba16float (RenderTargetInfo.texture); give the mesh's material it as map (lit) or emissiveMap with colour black (unlit). It shows the frame after the UI drew it. Null: laid out and hit, drawn nowhere."
  field width f32 "Its layout width; the target's pixels spread over it." {
    default 480
    unit cssPx
    min 1
  }
  field height f32 "Its layout height." {
    default 320
    unit cssPx
    min 1
  }
  field quad Option.WorldTarget "Where its quad is: centred on the target's pose, facing its +Z (an og.render.planeGeometry of quadWidth by quadHeight on that entity). Null: no ray reaches it."
  field rotation Quaternion "The quad's rotation after its target's."
  field quadWidth f32 "The quad's width along its +X." {
    default 1
    unit m
    min 0
  }
  field quadHeight f32 "The quad's height along its +Y." {
    default 1
    unit m
    min 0
  }
  field direction Direction "Which way it lays out its children; the default is a column."
  field gap f32 "Space between children." {
    unit cssPx
  }
  field padding Insets "Space inside its edges."
  field background Option.Paint "Its fill; null draws none (the target clears to transparent)."
}
```

<sub>`api.ogt` line 15355</sub>

<a id="type-PanelInfo"></a>

### `PanelInfo` (struct)

What the pointer does on a world panel this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `pointed` | `boolean` | `false` | — | — | — | The ray from the pointer meets the panel's quad (and no screen element is under it): its controls take the press. |
| `x` | `number` | `0` | — | `cssPx` | — | Where on the panel, from its top left. |
| `y` | `number` | `0` | — | `cssPx` | — | Where on the panel, from its top left. |
| `distance` | `number` | `0` | — | `m` | — | Along the ray to the hit. |

```ogt
struct PanelInfo {
  doc "What the pointer does on a world panel this tick."
  field pointed bool "The ray from the pointer meets the panel's quad (and no screen element is under it): its controls take the press."
  field x f32 "Where on the panel, from its top left." {
    unit cssPx
  }
  field y f32 "Where on the panel, from its top left." {
    unit cssPx
  }
  field distance f32 "Along the ray to the hit." {
    unit m
  }
}
```

<sub>`api.ogt` line 15389</sub>

<a id="type-MassKind"></a>

### `MassKind` (enum)

What a mass element template draws.

A string: one of the variants below. The zero value is `"number"`.

| Variant | Doc |
|---|---|
| `"number"` | The instance's value as a number (a damage number, a score). |
| `"text"` | One of strings, by the instance's value (a nameplate, a loot label). |
| `"bar"` | A bar filled to the instance's value 0–1 with a ghost at its ghost (an overhead health bar). |
| `"icon"` | Frame value of frames on texture (a status icon). |
| `"particle"` | A particle of particleShape (confetti, sparks). |

```ogt
enum MassKind {
  doc "What a mass element template draws."
  variant number "The instance's value as a number (a damage number, a score)."
  variant text "One of strings, by the instance's value (a nameplate, a loot label)."
  variant bar "A bar filled to the instance's value 0–1 with a ghost at its ghost (an overhead health bar)."
  variant icon "Frame value of frames on texture (a status icon)."
  variant particle "A particle of particleShape (confetti, sparks)."
}
```

<sub>`api.ogt` line 15410</sub>

<a id="type-MassEase"></a>

### `MassEase` (enum)

How a mass element's timeline moves into a key.

A string: one of the variants below. The zero value is `"linear"`.

| Variant | Doc |
|---|---|
| `"linear"` | Evenly. |
| `"out"` | Fast then slow. |
| `"in"` | Slow then fast. |
| `"back"` | Out, past the key and back (a pop). |
| `"step"` | Holds the key before until this one. |

```ogt
enum MassEase {
  doc "How a mass element's timeline moves into a key."
  variant linear "Evenly."
  variant out "Fast then slow."
  variant in "Slow then fast."
  variant back "Out, past the key and back (a pop)."
  variant step "Holds the key before until this one."
}
```

<sub>`api.ogt` line 15419</sub>

<a id="type-MassSpace"></a>

### `MassSpace` (enum)

Where a mass element's position is.

A string: one of the variants below. The zero value is `"world"`.

| Variant | Doc |
|---|---|
| `"world"` | A world point, projected by the GPU each frame. |
| `"screen"` | CSS pixels of the screen. |

```ogt
enum MassSpace {
  doc "Where a mass element's position is."
  variant world "A world point, projected by the GPU each frame."
  variant screen "CSS pixels of the screen."
}
```

<sub>`api.ogt` line 15428</sub>

<a id="type-ParticleShape"></a>

### `ParticleShape` (enum)

A particle's shape.

A string: one of the variants below. The zero value is `"square"`.

| Variant | Doc |
|---|---|
| `"square"` | A rectangle of size (a long thin one is a streak). |
| `"circle"` | A disc. |
| `"diamond"` | A diamond. |

```ogt
enum ParticleShape {
  doc "A particle's shape."
  variant square "A rectangle of size (a long thin one is a streak)."
  variant circle "A disc."
  variant diamond "A diamond."
}
```

<sub>`api.ogt` line 15434</sub>

<a id="type-MassKey"></a>

### `MassKey` (desc struct)

A key of a mass element's timeline, evaluated in the shader.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `t` | `number` | `0` | 0 … 1 | `ratio` | — | Where in its life, 0–1. |
| `offsetX` | `number` | `0` | — | `cssPx` | — | Moves it right. |
| `offsetY` | `number` | `0` | — | `cssPx` | — | Moves it down (negative rises). |
| `scale` | `number` | `1` | ≥ 0 | — | — | Its scale. |
| `alpha` | `number` | `1` | 0 … 1 | — | — | Its opacity. |
| `ease` | `MassEase` | `"linear"` | — | — | — | How it moves into this key. |

```ogt
struct MassKey {
  doc "A key of a mass element's timeline, evaluated in the shader."
  desc true
  field t f32 "Where in its life, 0–1." {
    unit ratio
    min 0
    max 1
  }
  field offsetX f32 "Moves it right." {
    unit cssPx
  }
  field offsetY f32 "Moves it down (negative rises)." {
    unit cssPx
  }
  field scale f32 "Its scale." {
    default 1
    min 0
  }
  field alpha f32 "Its opacity." {
    default 1
    min 0
    max 1
  }
  field ease MassEase "How it moves into this key."
}
```

<sub>`api.ogt` line 15441</sub>

<a id="type-MassTemplateDesc"></a>

### `MassTemplateDesc` (desc struct)

A mass element template (docs/ui.md §14): declared once, then instanced thousands of times a frame with a few values each. No instance is laid out; its motion is the timeline, evaluated on the GPU.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MassKind` | `"number"` | — | — | — | What it draws. |
| `space` | `MassSpace` | `"world"` | — | — | — | Where instances' positions are. |
| `font` | `Font \| null` | `null` | — | — | — | Numbers and text: the font (one page); null is the UI's body font. |
| `texture` | `Texture \| null` | `null` | — | — | — | Icons: the texture its frames are in. |
| `strings` | `readonly string[]` | `[]` | — | — | — | Text: the strings an instance's value picks. |
| `frames` | `readonly number[]` | `[]` | — | — | — | Icons: uv rectangles, four numbers each (u0, v0, u1, v1). |
| `decimals` | `number` | `0` | — | — | — | Numbers: digits after the point. |
| `plus` | `boolean` | `false` | — | — | — | Numbers: a + before positive values. |
| `compact` | `boolean` | `false` | — | — | — | Numbers: 12.3k, 4.5M. |
| `barBack` | `Paint \| null` | `null` | — | — | — | Bars: the track's ink; null is near black at 85 %. |
| `barGhost` | `Paint \| null` | `null` | — | — | — | Bars: the ghost's ink; null is pale. |
| `barRadius` | `number` | `0` | ≥ 0 | `cssPx` | — | Bars: corner radius. |
| `particleShape` | `ParticleShape` | `"square"` | — | — | — | Particles: the shape. |
| `size` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | Text and numbers: x is the em size; bars, icons and particles: width and height. |
| `pivot` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The element's point on its position, fractions (0.5, 1: its bottom centre). |
| `lifetime` | `number` | `0` | — | — | — | Ticks an instance lives; 0 shows pushed instances for their tick only. |
| `keys` | `readonly MassKey[]` | `[]` | — | — | — | The timeline over its life (up to 8). |
| `drift` | `number` | `0` | — | `cssPx` | — | Each instance drifts sideways up to this far over its life, by its seed. |
| `gravity` | `number` | `0` | — | `cssPx` | — | Added downward over its life, this much at its end. |
| `drag` | `boolean` | `false` | — | — | — | Instances' velocity slows to rest over the life. |
| `reference` | `number` | `0` | ≥ 0 | `m` | — | World space: the distance at which it is its size, nearer larger and farther smaller; 0 keeps its screen size. |
| `minScale` | `number` | `0.25` | ≥ 0 | — | — | World space: the smallest distance scale. |
| `maxScale` | `number` | `4` | ≥ 0 | — | — | World space: the largest distance scale. |
| `occlusion` | `WorldOcclusion` | `"none"` | — | — | — | World space: what the scene in front of an instance does. |
| `occludedAlpha` | `number` | `0.25` | 0 … 1 | — | — | Its opacity behind the scene, for occlusion fade. |
| `outline` | `Paint \| null` | `null` | — | — | — | Text and numbers: an outline's ink; null draws none. |
| `outlineWidth` | `number` | `2` | ≥ 0 | `cssPx` | — | Its width. |

```ogt
struct MassTemplateDesc {
  doc "A mass element template (docs/ui.md §14): declared once, then instanced thousands of times a frame with a few values each. No instance is laid out; its motion is the timeline, evaluated on the GPU."
  desc true
  field kind MassKind "What it draws."
  field space MassSpace "Where instances' positions are."
  field font Option.Font "Numbers and text: the font (one page); null is the UI's body font."
  field texture Option.Texture "Icons: the texture its frames are in."
  field strings List.String "Text: the strings an instance's value picks."
  field frames List.f32 "Icons: uv rectangles, four numbers each (u0, v0, u1, v1)."
  field decimals u32 "Numbers: digits after the point."
  field plus bool "Numbers: a + before positive values."
  field compact bool "Numbers: 12.3k, 4.5M."
  field barBack Option.Paint "Bars: the track's ink; null is near black at 85 %."
  field barGhost Option.Paint "Bars: the ghost's ink; null is pale."
  field barRadius f32 "Bars: corner radius." {
    unit cssPx
    min 0
  }
  field particleShape ParticleShape "Particles: the shape."
  field size Vector2 "Text and numbers: x is the em size; bars, icons and particles: width and height." {
    unit cssPx
  }
  field pivot Vector2 "The element's point on its position, fractions (0.5, 1: its bottom centre)."
  field lifetime u32 "Ticks an instance lives; 0 shows pushed instances for their tick only."
  field keys List.MassKey "The timeline over its life (up to 8)."
  field drift f32 "Each instance drifts sideways up to this far over its life, by its seed." {
    unit cssPx
  }
  field gravity f32 "Added downward over its life, this much at its end." {
    unit cssPx
  }
  field drag bool "Instances' velocity slows to rest over the life."
  field reference f32 "World space: the distance at which it is its size, nearer larger and farther smaller; 0 keeps its screen size." {
    unit m
    min 0
  }
  field minScale f32 "World space: the smallest distance scale." {
    default 0.25
    min 0
  }
  field maxScale f32 "World space: the largest distance scale." {
    default 4
    min 0
  }
  field occlusion WorldOcclusion "World space: what the scene in front of an instance does."
  field occludedAlpha f32 "Its opacity behind the scene, for occlusion fade." {
    default 0.25
    min 0
    max 1
  }
  field outline Option.Paint "Text and numbers: an outline's ink; null draws none."
  field outlineWidth f32 "Its width." {
    default 2
    unit cssPx
    min 0
  }
}
```

<sub>`api.ogt` line 15467</sub>

<a id="type-MassInstance"></a>

### `MassInstance` (desc struct)

One mass element: a few values; its template does the rest.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `position` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | A world point, or CSS pixels (x, y) in screen space. |
| `value` | `number` | `0` | — | — | — | The number, the string or frame index, or the bar's fill 0–1. |
| `color` | `Paint \| null` | `null` | — | — | — | Its ink (text, icon tint, bar fill, particle); null is white. |
| `scale` | `number` | `1` | ≥ 0 | — | — | Multiplies its timeline's scale (a critical hit). |
| `seed` | `number` | `0` | — | — | — | Its drift's side. |
| `ghost` | `number` | `0` | — | `ratio` | — | Bars: the ghost's fill 0–1. |
| `velocity` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | Travelled over its life. |
| `spin` | `number` | `0` | — | — | — | Turns over its life. |
| `rotation` | `number` | `0` | — | `rad` | — | Its starting rotation, clockwise. |

```ogt
struct MassInstance {
  doc "One mass element: a few values; its template does the rest."
  desc true
  field position Vector3 "A world point, or CSS pixels (x, y) in screen space."
  field value f64 "The number, the string or frame index, or the bar's fill 0–1."
  field color Option.Paint "Its ink (text, icon tint, bar fill, particle); null is white."
  field scale f32 "Multiplies its timeline's scale (a critical hit)." {
    default 1
    min 0
  }
  field seed u32 "Its drift's side."
  field ghost f32 "Bars: the ghost's fill 0–1." {
    unit ratio
  }
  field velocity Vector2 "Travelled over its life." {
    unit cssPx
  }
  field spin f32 "Turns over its life."
  field rotation f32 "Its starting rotation, clockwise." {
    unit rad
  }
}
```

<sub>`api.ogt` line 15525</sub>

<a id="type-EmitFrom"></a>

### `EmitFrom` (enum)

Where on its element a particle starts.

A string: one of the variants below. The zero value is `"center"`.

| Variant | Doc |
|---|---|
| `"center"` | Its centre. |
| `"area"` | Anywhere inside its box. |
| `"edges"` | On its edges. |

```ogt
enum EmitFrom {
  doc "Where on its element a particle starts."
  variant center "Its centre."
  variant area "Anywhere inside its box."
  variant edges "On its edges."
}
```

<sub>`api.ogt` line 15575</sub>

<a id="type-ParticlesDesc"></a>

### `ParticlesDesc` (desc struct)

UI particles from an element (docs/ui.md §5.5): confetti, sparks, coin streams, rarity bursts. Drawn over it on its layer, clipped by its clip, seeded so a replay throws the same. Nothing under reduced motion.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `rate` | `number` | `0` | ≥ 0 | — | — | Particles a second while it is built. |
| `burst` | `number` | `0` | — | — | — | Particles at once on a tick fire is set. |
| `fire` | `boolean` | `false` | — | — | — | Throw the burst this tick. |
| `lifetimeMs` | `number` | `0` | ≥ 0 | `ms` | — | How long a particle lives; 0 is 900. |
| `shape` | `ParticleShape` | `"square"` | — | — | — | Its shape. |
| `size` | `Vector2` | `{ x: 8, y: 8 }` | — | `cssPx` | — | Its size. |
| `colors` | `readonly Paint[]` | `[]` | — | — | — | Picked by each particle's seed; empty is the accent. |
| `travelMin` | `number` | `40` | ≥ 0 | `cssPx` | — | The shortest way a particle flies over its life. |
| `travelMax` | `number` | `120` | ≥ 0 | `cssPx` | — | The longest. |
| `direction` | `number` | `-1.5707964` | — | `rad` | — | Its heading, clockwise from +x (-π/2 is up). |
| `spread` | `number` | `6.2831855` | ≥ 0 | `rad` | — | The spread about the heading (2π is every way). |
| `gravity` | `number` | `0` | — | `cssPx` | — | Added downward over its life, this much at its end. |
| `drag` | `boolean` | `true` | — | — | — | Its flight slows to rest. |
| `spin` | `number` | `0` | ≥ 0 | — | — | Turns over its life, up to this many either way. |
| `align` | `boolean` | `false` | — | — | — | Its rotation follows its heading (a streak flying outward). |
| `from` | `EmitFrom` | `"center"` | — | — | — | Where it starts. |
| `endScale` | `number` | `1` | ≥ 0 | — | — | Its scale at the end of its life. |
| `fade` | `boolean` | `true` | — | — | — | Fades out toward its end. |
| `clip` | `boolean` | `true` | — | — | — | Clipped by the element's clip (a scroll view, a masked panel). |

```ogt
struct ParticlesDesc {
  doc "UI particles from an element (docs/ui.md §5.5): confetti, sparks, coin streams, rarity bursts. Drawn over it on its layer, clipped by its clip, seeded so a replay throws the same. Nothing under reduced motion."
  desc true
  field rate f32 "Particles a second while it is built." {
    min 0
  }
  field burst u32 "Particles at once on a tick fire is set."
  field fire bool "Throw the burst this tick."
  field lifetimeMs f32 "How long a particle lives; 0 is 900." {
    unit ms
    min 0
  }
  field shape ParticleShape "Its shape."
  field size Vector2 "Its size." {
    default 8 8
    unit cssPx
  }
  field colors List.Paint "Picked by each particle's seed; empty is the accent."
  field travelMin f32 "The shortest way a particle flies over its life." {
    default 40
    unit cssPx
    min 0
  }
  field travelMax f32 "The longest." {
    default 120
    unit cssPx
    min 0
  }
  field direction f32 "Its heading, clockwise from +x (-π/2 is up)." {
    default -1.5707964
    unit rad
  }
  field spread f32 "The spread about the heading (2π is every way)." {
    default 6.2831855
    unit rad
    min 0
  }
  field gravity f32 "Added downward over its life, this much at its end." {
    unit cssPx
  }
  field drag bool "Its flight slows to rest." {
    default true
  }
  field spin f32 "Turns over its life, up to this many either way." {
    min 0
  }
  field align bool "Its rotation follows its heading (a streak flying outward)."
  field from EmitFrom "Where it starts."
  field endScale f32 "Its scale at the end of its life." {
    default 1
    min 0
  }
  field fade bool "Fades out toward its end." {
    default true
  }
  field clip bool "Clipped by the element's clip (a scroll view, a masked panel)." {
    default true
  }
}
```

<sub>`api.ogt` line 15582</sub>

<a id="type-MetaKind"></a>

### `MetaKind` (enum)

What a meta layer draws over the scene.

A string: one of the variants below. The zero value is `"vignette"`.

| Variant | Doc |
|---|---|
| `"vignette"` | Its colour at the edges, clear in the middle (a damage vignette). |
| `"tint"` | A flat tint. |
| `"material"` | The frame under it composited through a UI material (a ripple for sanity, a glitch for a hit). |

```ogt
enum MetaKind {
  doc "What a meta layer draws over the scene."
  variant vignette "Its colour at the edges, clear in the middle (a damage vignette)."
  variant tint "A flat tint."
  variant material "The frame under it composited through a UI material (a ripple for sanity, a glitch for a hit)."
}
```

<sub>`api.ogt` line 15648</sub>

<a id="type-MetaDesc"></a>

### `MetaDesc` (desc struct)

A meta layer (docs/ui.md §14): full screen, over the scene and under the screen's elements, never taking input.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `kind` | `MetaKind` | `"vignette"` | — | — | — | What it draws. |
| `color` | `Paint \| null` | `null` | — | — | — | Vignette and tint: its ink; null is a deep red. |
| `inner` | `number` | `0.55` | 0 … 1 | `ratio` | — | Vignette: clear out to this fraction of the half diagonal. |
| `outer` | `number` | `1` | 0 … 1 | `ratio` | — | Vignette: full colour from this fraction on. |
| `material` | `MaterialName` | `"plain"` | — | — | — | Material: the built-in material, or custom. |
| `custom` | `UiMaterial \| null` | `null` | — | — | — | Material: the custom material. |
| `params` | `readonly number[]` | `[]` | — | — | — | Material: its parameters, up to four vec4s flattened. |
| `intensity` | `number` | `1` | 0 … 1 | — | — | Its opacity. |
| `pulseHz` | `number` | `0` | ≥ 0 | `hz` | — | Pulses this many times a second (a low-health heartbeat); 0 is steady. |
| `pulseDepth` | `number` | `0.5` | 0 … 1 | `ratio` | — | How far a pulse dims it. |

```ogt
struct MetaDesc {
  doc "A meta layer (docs/ui.md §14): full screen, over the scene and under the screen's elements, never taking input."
  desc true
  field kind MetaKind "What it draws."
  field color Option.Paint "Vignette and tint: its ink; null is a deep red."
  field inner f32 "Vignette: clear out to this fraction of the half diagonal." {
    default 0.55
    unit ratio
    min 0
    max 1
  }
  field outer f32 "Vignette: full colour from this fraction on." {
    default 1
    unit ratio
    min 0
    max 1
  }
  field material MaterialName "Material: the built-in material, or custom."
  field custom Option.UiMaterial "Material: the custom material."
  field params List.f32 "Material: its parameters, up to four vec4s flattened."
  field intensity f32 "Its opacity." {
    default 1
    min 0
    max 1
  }
  field pulseHz f32 "Pulses this many times a second (a low-health heartbeat); 0 is steady." {
    unit hz
    min 0
  }
  field pulseDepth f32 "How far a pulse dims it." {
    default 0.5
    unit ratio
    min 0
    max 1
  }
}
```

<sub>`api.ogt` line 15655</sub>

<a id="type-SceneViewportDesc"></a>

### `SceneViewportDesc` (desc struct)

The viewport element (docs/ui.md §5.5): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). Draw it with an og.CameraView into a render target on its own layers, with its own lights; this element shows the target and passes input in.

A desc: every field may be omitted and takes its default; `{}` is valid.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `texture` | `Texture \| null` | `null` | — | — | — | The target's colour (RenderTargetInfo.texture). |
| `width` | `Size \| null` | `null` | — | — | — | Its width; null is 320 CSS px. |
| `height` | `Size \| null` | `null` | — | — | — | Its height; null is 320 CSS px. |
| `aspect` | `number` | `0` | ≥ 0 | — | — | Width over height; 0 is free. |
| `cameraPosition` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | Its camera's position, for rays. |
| `cameraTarget` | `Vector3` | `{ x: 0, y: 0, z: 0 }` | — | — | — | What its camera looks at, for rays. |
| `fov` | `number` | `50` | ≥ 0 | `deg` | — | Its camera's vertical field of view; 0 gives no ray. |
| `background` | `Paint \| null` | `null` | — | — | — | Drawn under the view. |
| `radius` | `number` | `0` | — | `cssPx` | — | Corner radius. |
| `label` | `string` | `""` | — | — | — | What the probe and the accessibility tree call it. |

```ogt
struct SceneViewportDesc {
  doc "The viewport element (docs/ui.md §5.5): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). Draw it with an og.CameraView into a render target on its own layers, with its own lights; this element shows the target and passes input in."
  desc true
  field texture Option.Texture "The target's colour (RenderTargetInfo.texture)."
  field width Option.Size "Its width; null is 320 CSS px."
  field height Option.Size "Its height; null is 320 CSS px."
  field aspect f32 "Width over height; 0 is free." {
    min 0
  }
  field cameraPosition Vector3 "Its camera's position, for rays."
  field cameraTarget Vector3 "What its camera looks at, for rays."
  field fov f32 "Its camera's vertical field of view; 0 gives no ray." {
    default 50
    unit deg
    min 0
  }
  field background Option.Paint "Drawn under the view."
  field radius f32 "Corner radius." {
    unit cssPx
  }
  field label String "What the probe and the accessibility tree call it."
}
```

<sub>`api.ogt` line 15699</sub>

<a id="type-SceneViewportInfo"></a>

### `SceneViewportInfo` (struct)

What the pointer does in a viewport this tick.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `width` | `number` | `0` | — | `cssPx` | — | Its laid-out width, last tick. |
| `height` | `number` | `0` | — | `cssPx` | — | Its laid-out height, last tick. |
| `pixelWidth` | `number` | `0` | — | — | — | The target width that fills it at the device's pixel ratio. |
| `pixelHeight` | `number` | `0` | — | — | — | The target height that fills it. |
| `hovered` | `boolean` | `false` | — | — | — | The pointer is over it. |
| `pressed` | `boolean` | `false` | — | — | — | A press that started in it is held. |
| `clicked` | `boolean` | `false` | — | — | — | Pressed and released in it. |
| `drag` | `Vector2` | `{ x: 0, y: 0 }` | — | `cssPx` | — | The pointer's move while held this tick (orbit, turn). |
| `wheel` | `Vector2` | `{ x: 0, y: 0 }` | — | — | — | The wheel over it (zoom). |
| `at` | `Vector2 \| null` | `null` | — | — | — | The pointer in it, fractions of its box from the top left. |
| `ray` | `Ray \| null` | `null` | — | — | — | The ray into its scene through the pointer. |

```ogt
struct SceneViewportInfo {
  doc "What the pointer does in a viewport this tick."
  field width f32 "Its laid-out width, last tick." {
    unit cssPx
  }
  field height f32 "Its laid-out height, last tick." {
    unit cssPx
  }
  field pixelWidth u32 "The target width that fills it at the device's pixel ratio."
  field pixelHeight u32 "The target height that fills it."
  field hovered bool "The pointer is over it."
  field pressed bool "A press that started in it is held."
  field clicked bool "Pressed and released in it."
  field drag Vector2 "The pointer's move while held this tick (orbit, turn)." {
    unit cssPx
  }
  field wheel Vector2 "The wheel over it (zoom)."
  field at Option.Vector2 "The pointer in it, fractions of its box from the top left."
  field ray Option.Ray "The ray into its scene through the pointer."
}
```

<sub>`api.ogt` line 15722</sub>

<a id="type-UiDocEvent"></a>

### `UiDocEvent` (struct)

Something a document did (docs/ui.md §10.2): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back is `emit`.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `mount` | `string` | `""` | — | — | — | The document's mount: the screen's name, or a component instance's path. |
| `path` | `string` | `""` | — | — | — | The element's path. |
| `kind` | `string` | `""` | — | — | — | click, changed, select, emit, action, set or back. |
| `name` | `string` | `""` | — | — | — | The element's id (click), the prop (changed), the list (select), the event or action's name, or the viewmodel path a `set` wrote. |
| `value` | `unknown` | `null` | — | — | — | The new value (changed, set), the row's index (select); null otherwise. |

```ogt
struct UiDocEvent {
  doc "Something a document did (docs/ui.md §10.2): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back is `emit`."
  field mount String "The document's mount: the screen's name, or a component instance's path."
  field path String "The element's path."
  field kind String "click, changed, select, emit, action, set or back."
  field name String "The element's id (click), the prop (changed), the list (select), the event or action's name, or the viewmodel path a `set` wrote."
  field value Json "The new value (changed, set), the row's index (select); null otherwise."
}
```

<sub>`api.ogt` line 15751</sub>

<a id="type-UiDocResponse"></a>

### `UiDocResponse` (struct)

What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `path` | `string` | `""` | — | — | — | Its root's path; empty when it was not built. |
| `events` | `readonly UiDocEvent[]` | `[]` | — | — | — | Its events. |
| `props` | `unknown` | `null` | — | — | — | A component's props after its writes (a screen's viewmodel is og.ui.screenVm's). |

```ogt
struct UiDocResponse {
  doc "What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes."
  field path String "Its root's path; empty when it was not built."
  field events List.UiDocEvent "Its events."
  field props Json "A component's props after its writes (a screen's viewmodel is og.ui.screenVm's)."
}
```

<sub>`api.ogt` line 15760</sub>

<a id="type-UiLayer"></a>

### `UiLayer` (struct)

A layer of the stack (docs/ui.md §8.1), as its screen declares it.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `serial` | `number` | `0` | — | — | — | Its number, unique for the UI's life: what push answered. |
| `screen` | `string` | `""` | — | — | — | The screen's name; also its layer's id, the first segment of its elements' paths. |
| `kind` | `string` | `""` | — | — | — | hud, menu, modal, popover, toast, tooltip or system. |
| `input` | `string` | `""` | — | — | — | game (input reaches gameplay), menu (input stays in the UI) or all. |
| `capture` | `boolean` | `false` | — | — | — | It takes the pointer on its gaps too. |
| `pause` | `boolean` | `false` | — | — | — | The game pauses while it is on the stack. |
| `under` | `string` | `""` | — | — | — | What happens to the layers under it: none, dim, blur or hide. |
| `back` | `string` | `""` | — | — | — | What Back does: pop, none or `emit <event>`. |
| `focus` | `string` | `""` | — | — | — | Where focus starts, a path below its root (empty: its first control). |
| `remember` | `boolean` | `false` | — | — | — | Focus returns where it was when it is the top again. |
| `top` | `boolean` | `false` | — | — | — | It is the top layer. |
| `hidden` | `boolean` | `false` | — | — | — | A layer above hides it: it is not built. |
| `inert` | `boolean` | `false` | — | — | — | A menu layer above takes its input. |

```ogt
struct UiLayer {
  doc "A layer of the stack (docs/ui.md §8.1), as its screen declares it."
  field serial u32 "Its number, unique for the UI's life: what push answered."
  field screen String "The screen's name; also its layer's id, the first segment of its elements' paths."
  field kind String "hud, menu, modal, popover, toast, tooltip or system."
  field input String "game (input reaches gameplay), menu (input stays in the UI) or all."
  field capture bool "It takes the pointer on its gaps too."
  field pause bool "The game pauses while it is on the stack."
  field under String "What happens to the layers under it: none, dim, blur or hide."
  field back String "What Back does: pop, none or `emit <event>`."
  field focus String "Where focus starts, a path below its root (empty: its first control)."
  field remember bool "Focus returns where it was when it is the top again."
  field top bool "It is the top layer."
  field hidden bool "A layer above hides it: it is not built."
  field inert bool "A menu layer above takes its input."
}
```

<sub>`api.ogt` line 15767</sub>

<a id="type-UiActionTrigger"></a>

### `UiActionTrigger` (enum)

When a UI action fires.

A string: one of the variants below. The zero value is `"press"`.

| Variant | Doc |
|---|---|
| `"press"` | When a binding goes down. |
| `"release"` | When it comes up. |
| `"hold"` | When it has been held holdMs. |

```ogt
enum UiActionTrigger {
  doc "When a UI action fires."
  variant press "When a binding goes down."
  variant release "When it comes up."
  variant hold "When it has been held holdMs."
}
```

<sub>`api.ogt` line 15784</sub>

<a id="type-UiActionDesc"></a>

### `UiActionDesc` (struct)

A named UI action (docs/ui.md §8.2) bound to og-input's controls: a component asks for an action, not a key.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `name` | `string` | `""` | — | — | — | Its name: confirm, back, tabNext, tabPrev, details, or the game's own. |
| `keys` | `readonly string[]` | `[]` | — | — | — | Key codes (KeyboardEvent.code: `Enter`, `KeyE`). |
| `buttons` | `readonly string[]` | `[]` | — | — | — | Standard-mapping pad buttons (GamepadButton's names: `a`, `rightBumper`). |
| `trigger` | `UiActionTrigger` | `"press"` | — | — | — | When it fires. |
| `holdMs` | `number` | `0` | — | `ms` | — | How long a hold is (0: 500). |
| `toggle` | `boolean` | `false` | — | — | — | A hold's toggle alternative (XAG 107): a tap turns it on, the next tap off; og.ui.actionHeld answers true between. |
| `label` | `string` | `""` | — | — | — | What it says when no document gives it a hint. |
| `action` | `string` | `""` | — | — | optional | A gameplay action (og.input.defineAction) it follows instead of keys and buttons: it is down when that action is, and its glyph is that action's binding on the device used last, so the player's rebinding moves the prompt too. Empty: keys and buttons. |

```ogt
struct UiActionDesc {
  doc "A named UI action (docs/ui.md §8.2) bound to og-input's controls: a component asks for an action, not a key."
  field name String "Its name: confirm, back, tabNext, tabPrev, details, or the game's own."
  field keys List.String "Key codes (KeyboardEvent.code: `Enter`, `KeyE`)."
  field buttons List.String "Standard-mapping pad buttons (GamepadButton's names: `a`, `rightBumper`)."
  field trigger UiActionTrigger "When it fires."
  field holdMs f32 "How long a hold is (0: 500)." {
    unit ms
  }
  field toggle bool "A hold's toggle alternative (XAG 107): a tap turns it on, the next tap off; og.ui.actionHeld answers true between."
  field label String "What it says when no document gives it a hint."
  field action String "A gameplay action (og.input.defineAction) it follows instead of keys and buttons: it is down when that action is, and its glyph is that action's binding on the device used last, so the player's rebinding moves the prompt too. Empty: keys and buttons." {
    optional true
  }
}
```

<sub>`api.ogt` line 15791</sub>

<a id="type-UiDevice"></a>

### `UiDevice` (enum)

The device a glyph set is for: the one the player used last.

A string: one of the variants below. The zero value is `"keyboardMouse"`.

| Variant | Doc |
|---|---|
| `"keyboardMouse"` | Keyboard and mouse. |
| `"xbox"` | An Xbox pad. |
| `"playstation"` | A PlayStation pad. |
| `"switch"` | A Nintendo Switch pad (south is B). |
| `"gamepad"` | A generic standard-mapping pad. |
| `"touch"` | A touch screen. |

```ogt
enum UiDevice {
  doc "The device a glyph set is for: the one the player used last."
  variant keyboardMouse "Keyboard and mouse."
  variant xbox "An Xbox pad."
  variant playstation "A PlayStation pad."
  variant switch "A Nintendo Switch pad (south is B)."
  variant gamepad "A generic standard-mapping pad."
  variant touch "A touch screen."
}
```

<sub>`api.ogt` line 15807</sub>

<a id="type-UiGlyph"></a>

### `UiGlyph` (struct)

An action's control on the device used last.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `icon` | `string` | `""` | — | — | — | The icon a style's set draws: `key.enter`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `touch.<action>`. |
| `text` | `string` | `""` | — | — | — | The text that names it: `Enter`, `A`, `×`. |
| `device` | `UiDevice` | `"keyboardMouse"` | — | — | — | The device it is for. |

```ogt
struct UiGlyph {
  doc "An action's control on the device used last."
  field icon String "The icon a style's set draws: `key.enter`, `xbox.a`, `ps.cross`, `switch.b`, `pad.south`, `touch.<action>`."
  field text String "The text that names it: `Enter`, `A`, `×`."
  field device UiDevice "The device it is for."
}
```

<sub>`api.ogt` line 15817</sub>

<a id="type-UiActionHint"></a>

### `UiActionHint` (struct)

One entry of the action bar: an action the top layer accepts, its hint and glyph.

| Field | Type | Default | Range | Unit | Notes | Doc |
|---|---|---|---|---|---|---|
| `action` | `string` | `""` | — | — | — | The action. |
| `hint` | `string` | `""` | — | — | — | What it does here (the document's `actions` line, else its label). |
| `icon` | `string` | `""` | — | — | — | Its glyph's icon. |
| `text` | `string` | `""` | — | — | — | Its glyph's text. |
| `hold` | `boolean` | `false` | — | — | — | It is a hold. |

```ogt
struct UiActionHint {
  doc "One entry of the action bar: an action the top layer accepts, its hint and glyph."
  field action String "The action."
  field hint String "What it does here (the document's `actions` line, else its label)."
  field icon String "Its glyph's icon."
  field text String "Its glyph's text."
  field hold bool "It is a hold."
}
```

<sub>`api.ogt` line 15824</sub>

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

<sub>`api.ogt` line 14800</sub>

```ogt
column UiMaterial {
  doc "A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint."
  array Uint32Array
  stride 2
  value UiMaterial
}
```

<sub>`api.ogt` line 14807</sub>
