<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `ui` module

The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the engine's default style until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it.

The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick.

Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi("Slot", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it.

- **TypeScript**: functions are the namespace `og.ui`; components, resources and events are tokens on `og`.
- **Publish builds**: kept (members may override).
- **Index**: [every module and guide](index.md).

```ogt
module ui {
  doc "The engine's UI: an immediate-mode toolkit. Systems describe the tree every tick with these calls; the engine lays it out Clay-style, styles it with the game's UI style (the engine's default style until the game picks one), draws it over the frame and answers the probe (dump_ui, touch_move_to, move_to_text) from it."
  doc "The engine opens the frame before the game's systems (from og.Input and the play surface) and closes it after them, so calls belong inside systems; outside one they do nothing and log a warning. Interaction is decided against the previous tick's rectangles, which are also what the probe resolves, so a scripted touch_move_to and a click agree; response edges (clicked, changed, submitted) last one tick."
  doc "Lengths are CSS pixels of the play surface (origin top left, y down), the unit of og.Input and the probe. Ids: an element's id is unique among its siblings (`idi(\"Slot\", 3)` is `Slot#3`); a path `Inventory/Slot#3` names it after its nearest ancestors. The UI state (hover, press, focus, caret, scroll) is world state: snapshots and the world hash carry it."
  # declarations follow, each on its own page
```

<sub>Module `ui` line 1</sub>

## Contents

| Name | Kind | Signature | Summary |
|---|---|---|---|
| [`Direction`](ui/type-Direction.md) | enum |  | Which way a container lays out its children. |
| [`Align`](ui/type-Align.md) | enum |  | Where children sit along the main axis. |
| [`Cross`](ui/type-Cross.md) | enum |  | Where children sit across the main axis. |
| [`TextAlign`](ui/type-TextAlign.md) | enum |  | How the lines of a label align. |
| [`FontRole`](ui/type-FontRole.md) | enum |  | Which configured font text uses; display and mono fall back to body, rounded to display, then body. |
| [`ButtonVariant`](ui/type-ButtonVariant.md) | enum |  | Button roles: how the style draws each. |
| [`ControlSize`](ui/type-ControlSize.md) | enum |  | Control heights from the density (h-control-sm, -md, -lg). |
| [`Scheme`](ui/type-Scheme.md) | enum |  | The palette: games run on a stage, so dark is the default. |
| [`DensityMode`](ui/type-DensityMode.md) | enum |  | Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise. |
| [`ThemeColor`](ui/type-ThemeColor.md) | enum |  | A colour role of the UI style in effect (the game's style, or the default style before it picks one). |
| [`TypeStepName`](ui/type-TypeStepName.md) | enum |  | A step of the style's type ramp. |
| [`SizeRange`](ui/type-SizeRange.md) | struct |  | The bounds of a fit or grow size. |
| [`FixedSize`](ui/type-FixedSize.md) | struct |  | A fixed size. |
| [`PercentSize`](ui/type-PercentSize.md) | struct |  | A size relative to the parent. |
| [`Size`](ui/type-Size.md) | union |  | A size along one axis. |
| [`Insets`](ui/type-Insets.md) | struct |  | Padding or margins on four sides. |
| [`Paint`](ui/type-Paint.md) | desc struct |  | A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. |
| [`ArrowClaim`](ui/type-ArrowClaim.md) | desc struct |  | Which arrows a focused element claims (ElementDesc.arrows); none by default. |
| [`ElementDesc`](ui/type-ElementDesc.md) | desc struct |  | A container: its layout and its look. |
| [`LabelDesc`](ui/type-LabelDesc.md) | desc struct |  | How a label draws its text. |
| [`ButtonDesc`](ui/type-ButtonDesc.md) | desc struct |  | A button's look. |
| [`PanelDesc`](ui/type-PanelDesc.md) | desc struct |  | A card: panel, the panel radius, padded and spaced by the style. |
| [`ScrollAxes`](ui/type-ScrollAxes.md) | enum |  | Which ways a scroll container scrolls. |
| [`ScrollDesc`](ui/type-ScrollDesc.md) | desc struct |  | A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); … |
| [`ImageDesc`](ui/type-ImageDesc.md) | desc struct |  | A texture region, or a solid rectangle without a texture. |
| [`ZoomViewDesc`](ui/type-ZoomViewDesc.md) | desc struct |  | A zoomable picture (a map, an artwork): what it shows and its zoom limits. |
| [`ZoomResult`](ui/type-ZoomResult.md) | struct |  | Where a zoom view is. |
| [`ProgressDesc`](ui/type-ProgressDesc.md) | desc struct |  | A progress bar. |
| [`FieldDesc`](ui/type-FieldDesc.md) | desc struct |  | A one-line text field. |
| [`TextAreaDesc`](ui/type-TextAreaDesc.md) | desc struct |  | A multi-line text area. |
| [`Response`](ui/type-Response.md) | struct |  | What happened to an element this tick. |
| [`FieldResult`](ui/type-FieldResult.md) | struct |  | A text field's response and its text after this tick's input. |
| [`Viewport`](ui/type-Viewport.md) | struct |  | The play surface the UI lays out in, in og-ui units (CSS px over UiTheme.unit) and device pixels per unit. |
| [`UiDesc`](ui/type-UiDesc.md) | desc struct |  | How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. |
| [`UiSounds`](ui/type-UiSounds.md) | desc struct |  | Sounds the UI plays through og-audio (the ui bus) on its events; null is silent. |
| [`StylePart`](ui/type-StylePart.md) | enum |  | An element class a style dresses with a nine-slice frame. |
| [`TypeStep`](ui/type-TypeStep.md) | struct |  | One step of the type ramp. |
| [`ControlSizes`](ui/type-ControlSizes.md) | struct |  | The control, row and header heights and the gutter of the density in effect. |
| [`UiTheme`](ui/type-UiTheme.md) | struct |  | The style values the UI draws with, read-only. |
| [`UiBuild`](ui/type-UiBuild.md) | callback |  | Builds a container's children: ui calls made inside it nest under the container. |
| [`og.ui.configure`](ui/fn-configure.md) | function | `(world: World, desc?: UiDesc): Result<void>` | Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. |
| [`og.ui.viewport`](ui/fn-viewport.md) | function | `(world: World): Viewport` | This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in og-ui units (CSS pixels over UiTheme.unit, docs/ui.md §6.1: 1 on touch and up to 720p, so … |
| [`og.ui.theme`](ui/fn-theme.md) | function | `(world: World): UiTheme` | The style values the UI draws with. |
| [`og.ui.open`](ui/fn-open.md) | function | `(id: string, desc?: ElementDesc): void` | Open a container; the calls until its close() build its children. |
| [`og.ui.openPanel`](ui/fn-openPanel.md) | function | `(id: string, desc?: PanelDesc): void` | Open a card, closed by close(): og.ui.panel without the callback. |
| [`og.ui.openScroll`](ui/fn-openScroll.md) | function | `(id: string, desc?: ScrollDesc): void` | Open a vertical scroll container, closed by close(): og.ui.scroll without the callback. |
| [`og.ui.close`](ui/fn-close.md) | function | `(): Response` | Close the open container. |
| [`og.ui.row`](ui/fn-row.md) | function | `(id: string, desc: ElementDesc, build: UiBuild): Response` | A row built by `build`: open(id, {...desc, direction: row}), build(), close(). |
| [`og.ui.column`](ui/fn-column.md) | function | `(id: string, desc: ElementDesc, build: UiBuild): Response` | A column built by `build`: open(id, {...desc, direction: column}), build(), close(). |
| [`og.ui.panel`](ui/fn-panel.md) | function | `(id: string, desc: PanelDesc, build: UiBuild): void` | A card built by `build`: openPanel(id, desc), build(), close(). |
| [`og.ui.scroll`](ui/fn-scroll.md) | function | `(id: string, desc: ScrollDesc, build: UiBuild): void` | A vertical scroll container built by `build`: openScroll(id, desc), build(), close(). |
| [`RichTextDesc`](ui/type-RichTextDesc.md) | desc struct |  | How a rich text paragraph draws: its defaults, which its markup's spans override. |
| [`RichTextResult`](ui/type-RichTextResult.md) | struct |  | What a rich text's links did this tick. |
| [`og.ui.richText`](ui/fn-richText.md) | function | `(id: string, markup: string, desc?: RichTextDesc): RichTextResult` | A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a … |
| [`og.ui.label`](ui/fn-label.md) | function | `(id: string, text: string, desc?: LabelDesc): void` | MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. |
| [`og.ui.button`](ui/fn-button.md) | function | `(id: string, text: string, desc?: ButtonDesc): Response` | A button with a label; `clicked` is a press and a release inside it. |
| [`og.ui.image`](ui/fn-image.md) | function | `(id: string, desc?: ImageDesc): void` | A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other. |
| [`og.ui.zoomView`](ui/fn-zoomView.md) | function | `(id: string, desc?: ZoomViewDesc): ZoomResult` | A zoomable picture (a map, an artwork): two fingers pinch it to zoom about their centre and pan it as they move (og.Pointers), the wheel zooms about the pointer, a drag pans; Tab or a pad focuses it,… |
| [`og.ui.progress`](ui/fn-progress.md) | function | `(id: string, value: number, desc?: ProgressDesc): void` | A track filled to a fraction. |
| [`og.ui.textField`](ui/fn-textField.md) | function | `(id: string, value: string, desc?: FieldDesc): FieldResult` | One line of text: a press focuses it with the caret at the press, a drag selects (and scrolls the text past the field's ends), a double click selects a word and a triple click everything; typed text … |
| [`og.ui.textArea`](ui/fn-textArea.md) | function | `(id: string, value: string, desc?: TextAreaDesc): FieldResult` | Several lines of text: textField's editing, with Enter starting a new line, Up and Down moving between lines and Home and End to a line's ends, a triple click selecting the line between its breaks, t… |
| [`og.ui.tooltip`](ui/fn-tooltip.md) | function | `(text: string): void` | A tooltip for the element just built: shown below it (above without room) once the pointer has rested on it for the slow motion duration, drawn over everything. |
| [`og.ui.idi`](ui/fn-idi.md) | function | `(name: string, index: number): string` | The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`. |
| [`Anchor`](ui/type-Anchor.md) | enum |  | A point of the safe area an anchored group sits at. |
| [`TransitionKind`](ui/type-TransitionKind.md) | enum |  | How a transition moves its content in and out. |
| [`Tone`](ui/type-Tone.md) | enum |  | A status tone of badges, toasts and map markers. |
| [`MeterKind`](ui/type-MeterKind.md) | enum |  | What a meter measures, which sets its colour. |
| [`Breakpoint`](ui/type-Breakpoint.md) | enum |  | The play surface's size class. |
| [`Contrast`](ui/type-Contrast.md) | enum |  | The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour). |
| [`NavIntent`](ui/type-NavIntent.md) | enum |  | A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give. |
| [`UiRect`](ui/type-UiRect.md) | struct |  | A rectangle of the play surface. |
| [`IconAtlasDesc`](ui/type-IconAtlasDesc.md) | desc struct |  | The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font). |
| [`UiInlineImage`](ui/type-UiInlineImage.md) | desc struct |  | A picture rich text draws inline by name ([img=<name>]). |
| [`AnchorDesc`](ui/type-AnchorDesc.md) | desc struct |  | An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls). |
| [`TransitionDesc`](ui/type-TransitionDesc.md) | desc struct |  | How a transition moves, timed in ticks from the style's motion. |
| [`Presence`](ui/type-Presence.md) | struct |  | Where a transition or dialog is this tick. |
| [`ModalDesc`](ui/type-ModalDesc.md) | desc struct |  | A dialog: a scrim over everything and a centred panel. |
| [`ModalResult`](ui/type-ModalResult.md) | struct |  | A dialog's answer this tick. |
| [`SpotlightDesc`](ui/type-SpotlightDesc.md) | desc struct |  | A spotlight: the surface dimmed except a hole around a target. |
| [`ListDesc`](ui/type-ListDesc.md) | desc struct |  | A virtualised list: only the rows in view (and one more each side) are built, fillers keep the scroll range of every row. |
| [`ListRange`](ui/type-ListRange.md) | struct |  | The rows (or cells) to build this tick: first up to, not including, last. |
| [`ListResult`](ui/type-ListResult.md) | struct |  | What happened in a list or grid this tick. |
| [`GridDesc`](ui/type-GridDesc.md) | desc struct |  | A virtualised grid of equal cells (inventories, level select), scrolling vertically. |
| [`ToastDesc`](ui/type-ToastDesc.md) | desc struct |  | A toast's tone and time. |
| [`ChipDesc`](ui/type-ChipDesc.md) | desc struct |  | A chip's state. |
| [`MeterDesc`](ui/type-MeterDesc.md) | desc struct |  | A meter: a track with a fill that animates its changes by the tick. |
| [`CooldownDesc`](ui/type-CooldownDesc.md) | desc struct |  | A cooldown ring's size and centre. |
| [`IconDesc`](ui/type-IconDesc.md) | desc struct |  | An icon's size, ink and name. |
| [`MapMarker`](ui/type-MapMarker.md) | desc struct |  | A marker on a mini-map. |
| [`MinimapDesc`](ui/type-MinimapDesc.md) | desc struct |  | A mini-map frame. |
| [`DialogueDesc`](ui/type-DialogueDesc.md) | desc struct |  | A dialogue box: a portrait slot, the speaker and the line typed out by the tick. |
| [`DialogueResult`](ui/type-DialogueResult.md) | struct |  | A dialogue box's answer this tick. |
| [`JoystickDesc`](ui/type-JoystickDesc.md) | desc struct |  | A virtual joystick: one translucent disc of the HUD ground with a rim (part Joystick.base, which a style dresses) and a knob that follows the finger that started on it, the rim lit toward the push wh… |
| [`JoystickResult`](ui/type-JoystickResult.md) | struct |  | A joystick's vector this tick. |
| [`DpadDesc`](ui/type-DpadDesc.md) | desc struct |  | A d-pad: one round pad (part Dpad.base, which a style dresses) with four chevrons on it, read as one target by the angle of the finger from its centre: eight ways (a diagonal holds both of its arms),… |
| [`DpadResult`](ui/type-DpadResult.md) | struct |  | Which arms of a d-pad are held this tick. |
| [`ActionButtonDesc`](ui/type-ActionButtonDesc.md) | desc struct |  | A touch action button (jump, fire, a skill): a disc with a rim (part ActionButton.face, flagged primary, pressed and disabled, which a style dresses), its icon on it; a label wider than it with no ic… |
| [`SkeletonDesc`](ui/type-SkeletonDesc.md) | desc struct |  | A skeleton: the neutral fill standing in for content that is loading. |
| [`UiItemBuild`](ui/type-UiItemBuild.md) | callback |  | Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it. |
| [`og.ui.breakpoint`](ui/fn-breakpoint.md) | function | `(): Breakpoint` | The size class of the play surface of the frame being built (desktop outside a frame). |
| [`UiPointer`](ui/type-UiPointer.md) | struct |  | The primary pointer as the UI reads it this tick. |
| [`og.ui.framePointer`](ui/fn-framePointer.md) | function | `(): UiPointer` | The primary pointer of the frame being built, without the world: for a component that hit-tests a shape it draws itself (a pie's wedges, a wheel's slices) by angle and radius rather than by rectangle… |
| [`og.ui.frameCursor`](ui/fn-frameCursor.md) | function | `(): Cursor` | The pointer's shape the last frame resolved (never auto): the cursor set on what the press holds or the pointer is over, or around it, else its kind's. |
| [`og.ui.frameViewport`](ui/fn-frameViewport.md) | function | `(): Viewport` | The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI. |
| [`og.ui.hasIcon`](ui/fn-hasIcon.md) | function | `(name: string): boolean` | Whether the icon atlas has an icon of this name (components draw a fallback otherwise). |
| [`og.ui.seconds`](ui/fn-seconds.md) | function | `(): number` | The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock). |
| [`og.ui.lastPath`](ui/fn-lastPath.md) | function | `(): string` | The full path of the element built just before (a container just closed, or a widget), as og.ui.bounds, og.ui.gesture, og.ui.forceState and the probe name it: a component reads its own parts' rectang… |
| [`og.ui.setFocus`](ui/fn-setFocus.md) | function | `(path: string): void` | Move the focus (the pointer's, the keys' and pad 0's) to the element at a full path when this frame ends, if a focusable control is there then: a component moving the focus with its choice (a radio g… |
| [`og.ui.scrollIntoView`](ui/fn-scrollIntoView.md) | function | `(path: string, center: boolean): void` | Scroll the scroll container around a control (an interactive or focusable element, by its full path) so the control is in its view, by where both were in the last frame: by the least that shows it wh… |
| [`og.ui.frameDirection`](ui/fn-frameDirection.md) | function | `(): string` | The writing direction of the frame being built, `ltr` or `rtl` (og.i18n's locale sets it): code that draws its own arrows, chevrons or progress along a line mirrors them by it; layout mirrors rows an… |
| [`og.ui.frameTheme`](ui/fn-frameTheme.md) | function | `(): UiTheme` | The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the components and patterns). |
| [`og.ui.openAnchor`](ui/fn-openAnchor.md) | function | `(id: string, desc?: AnchorDesc): void` | Open an anchored group, closed by close(): og.ui.anchor without the callback. |
| [`og.ui.anchor`](ui/fn-anchor.md) | function | `(id: string, desc: AnchorDesc, build: UiBuild): void` | An anchored group built by `build`: a layer at a point of the safe area, over the tree, for HUD corners and touch controls. |
| [`og.ui.openTransition`](ui/fn-openTransition.md) | function | `(id: string, show: boolean, desc?: TransitionDesc): Presence` | Open a transition container (a column), closed by close(); build its children only while the answer is visible: it stays visible while it leaves. |
| [`og.ui.transition`](ui/fn-transition.md) | function | `(id: string, show: boolean, desc: TransitionDesc, build: UiBuild): Presence` | Content that enters and leaves: fade, slide or scale by the tick from the style's motion. |
| [`og.ui.openModal`](ui/fn-openModal.md) | function | `(id: string, open: boolean, desc?: ModalDesc): ModalResult` | Open a dialog, closed by closeModal(); build its content only while the answer is visible. |
| [`og.ui.closeModal`](ui/fn-closeModal.md) | function | `(): void` | Close a dialog opened with openModal (its panel and backdrop). |
| [`og.ui.modal`](ui/fn-modal.md) | function | `(id: string, open: boolean, desc: ModalDesc, build: UiBuild): ModalResult` | A dialog built by `build` while visible: scrim, centred panel, focus kept inside, Back and the backdrop dismiss it unless locked. |
| [`og.ui.spotlight`](ui/fn-spotlight.md) | function | `(id: string, hole: UiRect \| null, desc?: SpotlightDesc): Response` | A spotlight over everything (a tutorial step): the scrim on four sides of the hole, a primary edge around it. |
| [`og.ui.bounds`](ui/fn-bounds.md) | function | `(path: string): UiRect \| null` | Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it. |
| [`ScrollState`](ui/type-ScrollState.md) | struct |  | A scroll container as the last frame laid it out (og.ui.scrollState). |
| [`og.ui.scrollState`](ui/fn-scrollState.md) | function | `(path: string): ScrollState \| null` | A scroll container's offset, view and whole content, by its full path (og.ui.lastPath after closing it): what a component virtualises its rows by, since og.ui.bounds is cut to the view and a row scro… |
| [`og.ui.openList`](ui/fn-openList.md) | function | `(id: string, desc?: ListDesc): ListRange` | Open a virtualised list (a scroll column) and learn which rows to build: listItem(i) and close() for each, then closeList(). |
| [`og.ui.openGrid`](ui/fn-openGrid.md) | function | `(id: string, desc?: GridDesc): ListRange` | Open a virtualised grid and learn which cells to build: gridItem(i) and close() for each, then closeList(). |
| [`og.ui.openKeyedList`](ui/fn-openKeyedList.md) | function | `(id: string, keys: readonly string[], desc?: ListDesc): ListRange` | Open a variable-height list. |
| [`og.ui.openKeyedGrid`](ui/fn-openKeyedGrid.md) | function | `(id: string, keys: readonly string[], desc?: GridDesc): ListRange` | Open a variable-height grid with persistent keys. |
| [`og.ui.listItem`](ui/fn-listItem.md) | function | `(index: number, selected: boolean): Response` | Open row `index` of the open list (`Row#index`): selectable and focusable; its content follows until close(). |
| [`og.ui.gridItem`](ui/fn-gridItem.md) | function | `(index: number, selected: boolean): Response` | Open cell `index` of the open grid (`Cell#index`): a slot on control, focusable; its content follows until close(). |
| [`og.ui.closeList`](ui/fn-closeList.md) | function | `(): Response` | Close a list or grid opened with openList or openGrid. |
| [`og.ui.dragging`](ui/fn-dragging.md) | function | `(id: string): number \| null` | The row or cell a held press started on in the list or grid of this id (built under the same parent): a drag in progress, to draw what is dragged under the pointer. |
| [`og.ui.list`](ui/fn-list.md) | function | `(id: string, desc: ListDesc, build: UiItemBuild): ListResult` | A virtualised list built by `build(index)` for the rows in view. |
| [`og.ui.grid`](ui/fn-grid.md) | function | `(id: string, desc: GridDesc, build: UiItemBuild): ListResult` | A virtualised grid built by `build(index)` for the cells in view. |
| [`og.ui.chip`](ui/fn-chip.md) | function | `(id: string, text: string, desc?: ChipDesc): Response` | A chip: a control-filled tag; selected inverts to ink. |
| [`og.ui.badge`](ui/fn-badge.md) | function | `(id: string, text: string, tone: Tone): void` | A badge: a small status tag in its tone. |
| [`og.ui.divider`](ui/fn-divider.md) | function | `(id: string, vertical: boolean): void` | A hairline in the border colour across its parent. |
| [`og.ui.spacer`](ui/fn-spacer.md) | function | `(id: string, size: Size \| null): void` | Empty space. |
| [`og.ui.keyValue`](ui/fn-keyValue.md) | function | `(id: string, key: string, value: string, mono: boolean): void` | A key (inkMuted) and its value (ink, at the end) on one row: stats, settings summaries, receipts. |
| [`og.ui.skeleton`](ui/fn-skeleton.md) | function | `(id: string, desc?: SkeletonDesc): void` | A skeleton: the neutral fill standing in for content that is loading, with a sheen that passes every 1.6 s (still under reduced motion). |
| [`og.ui.spinner`](ui/fn-spinner.md) | function | `(id: string, size: number): void` | A loading spinner: eight dots around a circle, the bright one turning by the tick. |
| [`og.ui.icon`](ui/fn-icon.md) | function | `(id: string, name: string, desc?: IconDesc): void` | An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. |
| [`og.ui.meter`](ui/fn-meter.md) | function | `(id: string, value: number, max: number, desc?: MeterDesc): void` | A health, mana or xp bar. |
| [`og.ui.cooldown`](ui/fn-cooldown.md) | function | `(id: string, remaining: number, desc?: CooldownDesc): Response` | A cooldown: a disc whose primary ring fills clockwise from the top as it runs out, bright and full when ready, a label or icon in the middle. |
| [`og.ui.minimap`](ui/fn-minimap.md) | function | `(id: string, desc?: MinimapDesc): void` | A mini-map frame: the map in a lineStrong frame, markers as dots (the player primary and ringed), an N mark and a zone label. |
| [`og.ui.dialogue`](ui/fn-dialogue.md) | function | `(id: string, desc?: DialogueDesc): DialogueResult` | A dialogue box: a portrait slot, the speaker in primary ink and the line typed out by the tick. |
| [`og.ui.joystick`](ui/fn-joystick.md) | function | `(id: string, desc?: JoystickDesc): JoystickResult` | A virtual joystick: a base and a knob that follows a press that started on it. |
| [`og.ui.dpad`](ui/fn-dpad.md) | function | `(id: string, desc?: DpadDesc): DpadResult` | A d-pad of four arms (`Up`, `Down`, `Left`, `Right`), each at least 44 CSS px. |
| [`og.ui.actionButton`](ui/fn-actionButton.md) | function | `(id: string, text: string, desc?: ActionButtonDesc): Response` | A square touch action button, cornered by the style's control radius: pressed while held (jump, fire), clicked on a tap; at least 44 CSS px. |
| [`og.ui.toast`](ui/fn-toast.md) | function | `(world: World, text: string, desc?: ToastDesc): number` | Queue a toast: it stacks at UiDesc.toastAnchor, enters, stays its time and leaves; at most three are up (older ones leave first); a press dismisses it. |
| [`og.ui.dismissToast`](ui/fn-dismissToast.md) | function | `(world: World, toastId: number): void` | Start a toast's exit. |
| [`og.ui.navigate`](ui/fn-navigate.md) | function | `(world: World, intent: NavIntent): void` | Queue a navigation intent for the next tick: a gamepad og.Input does not read yet, a custom binding, a command. |
| [`og.ui.back`](ui/fn-back.md) | function | `(): boolean` | Escape or gamepad B this tick, not taken by a dialog or menu: true once, then taken. |
| [`og.ui.setPlayers`](ui/fn-setPlayers.md) | function | `(count: number): void` | Local multiplayer: how many players this screen takes (1 to 4; 1 by default). |
| [`VirtualCursorDesc`](ui/type-VirtualCursorDesc.md) | desc struct |  | A pad's virtual cursor (docs/ui.md §2.1). |
| [`og.ui.setVirtualCursor`](ui/fn-setVirtualCursor.md) | function | `(desc: VirtualCursorDesc \| null): void` | A pad's virtual cursor, for a screen built for the pointer (a map, a shop grid, a strategy HUD) played on a pad: pad 0's left stick moves it instead of the focus, slowing and sticking over controls; … |
| [`VirtualCursor`](ui/type-VirtualCursor.md) | struct |  | Where a pad's virtual cursor is. |
| [`og.ui.virtualCursor`](ui/fn-virtualCursor.md) | function | `(): VirtualCursor \| null` | The virtual cursor as the last frame left it; null when none is declared. |
| [`HudPlacement`](ui/type-HudPlacement.md) | struct |  | Where the player put a customisable element: moved from its laid-out place, scaled about its centre and faded. |
| [`HudEdit`](ui/type-HudEdit.md) | struct |  | The HUD edit mode as the last frame left it. |
| [`og.ui.setHudEdit`](ui/fn-setHudEdit.md) | function | `(on: boolean): void` | Turns the HUD edit mode on or off (docs/ui.md §2.1; kept until changed). |
| [`og.ui.setHudPlayer`](ui/fn-setHudPlayer.md) | function | `(player: number): void` | Whose layout this screen shows and edits, 0 to 3 (0 by default; kept until changed): each player of a shared screen keeps a layout of their own. |
| [`og.ui.hudEdit`](ui/fn-hudEdit.md) | function | `(): HudEdit` | The HUD edit mode as the last frame left it. |
| [`og.ui.hudLayout`](ui/fn-hudLayout.md) | function | `(player: number): string` | A player's HUD layout as JSON, for the game's settings: each moved, scaled or faded element's key with its x, y, scale and opacity. |
| [`og.ui.setHudLayout`](ui/fn-setHudLayout.md) | function | `(player: number, layout: string): boolean` | Gives a player's HUD layout back (og.ui.hudLayout's JSON; `{}` resets it). |
| [`og.ui.hudSelect`](ui/fn-hudSelect.md) | function | `(key: string): void` | Selects an element to edit by its key (empty: none). |
| [`og.ui.hudStep`](ui/fn-hudStep.md) | function | `(scale: number, opacity: number): void` | Steps the selected element's size and opacity: each -1, 0 or +1 step (a tenth larger or smaller, a tenth more or less opaque), within its limits. |
| [`og.ui.hudReset`](ui/fn-hudReset.md) | function | `(key: string): void` | Puts an element back where it was laid out: its key, empty for the selected one, `*` for every one. |
| [`HudEditorDesc`](ui/type-HudEditorDesc.md) | desc struct |  | The HUD edit mode's panel; empty labels take English ones (pass the locale's). |
| [`HudEditorResult`](ui/type-HudEditorResult.md) | struct |  | What the HUD editor's buttons did this tick. |
| [`og.ui.hudLayoutEditor`](ui/fn-hudLayoutEditor.md) | function | `(id: string, desc?: HudEditorDesc): HudEditorResult` | The HUD edit mode's panel (docs/ui.md §2.1): the selected element's key, size and opacity steps, Next (the next element: how keys and pads choose one), Reset (the selected one, or all with none selec… |
| [`og.ui.focus`](ui/fn-focus.md) | function | `(pad: number): string` | The path of the control pad pad's focus is on in the last frame (pad 0: the shared focus of the pointer, keyboard and pad 0), as touch_move_to resolves it; empty when it is on none. |
| [`A11yNode`](ui/type-A11yNode.md) | struct |  | One node of the accessibility tree: an element with a role (layout containers and spacers have none and are left out). |
| [`A11yTree`](ui/type-A11yTree.md) | struct |  | The last frame's accessibility tree, built from the same nodes the UI draws and takes input on. |
| [`og.ui.a11y`](ui/fn-a11y.md) | function | `(): A11yTree` | The accessibility tree of the last frame: every element with a role, its name, value, states and bounds, and the focus order Tab walks. |
| [`og.ui.feedback`](ui/fn-feedback.md) | function | `(event: string, rise: number): void` | Play a UI event's feedback from the style (docs/ui.md §3.1): one sound of its set, at random and never the one it played last, at a volume and pitch from the set's ranges, and its haptic pattern at t… |
| [`og.ui.setHaptics`](ui/fn-setHaptics.md) | function | `(intensity: number): void` | The player's haptic strength (a settings screen's vibration slider or switch): 0 turns the UI's haptics off, 1 (the default) plays the style's patterns as set; between scales them. |
| [`og.ui.setNavigation`](ui/fn-setNavigation.md) | function | `(on: boolean): void` | Whether keyboard and gamepad navigation moves focus on this screen (on by default). |
| [`og.ui.memory`](ui/fn-memory.md) | function | `(id: string, initial: number): number` | A number the UI keeps for an id under the open element (a component's tab, a count-up's start tick), in snapshots like the rest of the UI state. |
| [`og.ui.remember`](ui/fn-remember.md) | function | `(id: string, value: number): void` | Set a number memory (see memory). |
| [`og.ui.tick`](ui/fn-tick.md) | function | `(): number` | Frames the UI has built: its clock, for animations by the tick. |
| [`CounterDesc`](ui/type-CounterDesc.md) | desc struct |  | A number that counts to its value. |
| [`og.ui.counter`](ui/fn-counter.md) | function | `(id: string, value: number, desc?: CounterDesc): number` | A number label that counts (ease-out cubic, by the tick) from where it was to the value each time the value changes, grouped with commas; a style with motion.pop bumps it when it lands. |
| [`BurstDesc`](ui/type-BurstDesc.md) | desc struct |  | Rays and sparks thrown from an element. |
| [`og.ui.burst`](ui/fn-burst.md) | function | `(desc?: BurstDesc): void` | Throw rays and sparks from the element built just before (a star landing, a reward), once, from when it first appears. |
| [`ShakeDesc`](ui/type-ShakeDesc.md) | desc struct |  | A shake of an element: three sideways swings decaying to rest. |
| [`og.ui.shake`](ui/fn-shake.md) | function | `(desc?: ShakeDesc): void` | Shake the element built just before, from this tick (a wrong answer, a locked door, a refused purchase): call it once, on the tick it happens; the element swings and comes to rest by itself, and snap… |
| [`ScreenWipe`](ui/type-ScreenWipe.md) | enum |  | How a screen transition covers the old screen and reveals the new one. |
| [`WipeSide`](ui/type-WipeSide.md) | enum |  | Where a wipe or a slide comes from. |
| [`ScreenTransitionDesc`](ui/type-ScreenTransitionDesc.md) | desc struct |  | How a screen transition looks. |
| [`ScreenTransitionResult`](ui/type-ScreenTransitionResult.md) | struct |  | Where a screen transition is this tick. |
| [`og.ui.screenTransition`](ui/fn-screenTransition.md) | function | `(id: string, screen: string, desc?: ScreenTransitionDesc): ScreenTransitionResult` | A transition between game screens drawn over the whole frame (through the overlay, over every other UI layer). |
| [`og.ui.previewViewport`](ui/fn-previewViewport.md) | function | `(world: World, viewport: Viewport \| null): void` | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. |
| [`CardVariant`](ui/type-CardVariant.md) | enum |  | How a card stands off what it sits on. |
| [`CardMedia`](ui/type-CardMedia.md) | desc struct |  | A card's media slot: art the game fills, or a placeholder drawn from the style when it has none. |
| [`CardDesc`](ui/type-CardDesc.md) | desc struct |  | A card: media, an eyebrow, a title, a subtitle and a corner badge over a body the game builds. |
| [`ScrollAreaDesc`](ui/type-ScrollAreaDesc.md) | desc struct |  | A scroll area: og.ui's scroll container whose content fades out at an edge where more lies beyond. |
| [`AccordionSection`](ui/type-AccordionSection.md) | desc struct |  | A section of an accordion: its header's title, subtitle and badge. |
| [`AccordionDesc`](ui/type-AccordionDesc.md) | desc struct |  | An accordion: sections whose headers open and close their bodies. |
| [`AccordionResult`](ui/type-AccordionResult.md) | struct |  | What an accordion answered this tick. |
| [`SplitViewDesc`](ui/type-SplitViewDesc.md) | desc struct |  | Two panes with a divider between them that a drag moves. |
| [`SplitResult`](ui/type-SplitResult.md) | struct |  | What a split view answered this tick. |
| [`DialogTone`](ui/type-DialogTone.md) | enum |  | A dialog's tone, shown by a mark as well as its colour. |
| [`DialogAction`](ui/type-DialogAction.md) | desc struct |  | A button of a dialog. |
| [`DialogDesc`](ui/type-DialogDesc.md) | desc struct |  | A dialog: a title, a tone mark, a message, a body the game builds and a row of actions. |
| [`DialogResult`](ui/type-DialogResult.md) | struct |  | What a dialog answered this tick. |
| [`ConfirmResult`](ui/type-ConfirmResult.md) | struct |  | A confirm dialog's answer this tick. |
| [`ConfirmationDesc`](ui/type-ConfirmationDesc.md) | desc struct |  | A confirmation: a message, Cancel and the action. |
| [`FloatSide`](ui/type-FloatSide.md) | enum |  | Where a floating surface hangs off what it belongs to; start and end follow the writing direction. |
| [`FloatAlign`](ui/type-FloatAlign.md) | enum |  | How a floating surface lines up with what it belongs to. |
| [`PopoverDesc`](ui/type-PopoverDesc.md) | desc struct |  | A popover beside an element, flipped where there is no room and kept on screen, with an arrow at the element. |
| [`PopoverResult`](ui/type-PopoverResult.md) | struct |  | What a popover answered this tick. |
| [`MenuEntryKind`](ui/type-MenuEntryKind.md) | enum |  | What a menu's entry is. |
| [`MenuEntry`](ui/type-MenuEntry.md) | desc struct |  | An entry of a menu. An entry whose `parent` names another's id sits in that entry's submenu. |
| [`MenuDesc`](ui/type-MenuDesc.md) | desc struct |  | A menu hanging from an element, or at a point (a context menu). |
| [`MenuChoice`](ui/type-MenuChoice.md) | struct |  | What a menu answered this tick. |
| [`SheetEdge`](ui/type-SheetEdge.md) | enum |  | The edge a sheet slides in from; start and end follow the writing direction. |
| [`SheetDesc`](ui/type-SheetDesc.md) | desc struct |  | A sheet: a surface that slides in from an edge over a scrim; a bottom sheet rests at detents and a drag moves it. |
| [`SheetResult`](ui/type-SheetResult.md) | struct |  | What a sheet answered this tick. |
| [`TabVariant`](ui/type-TabVariant.md) | enum |  | How the selected tab is marked. |
| [`TabItem`](ui/type-TabItem.md) | desc struct |  | A tab. |
| [`TabBarDesc`](ui/type-TabBarDesc.md) | desc struct |  | Tabs over the panel the game builds for the selected one. |
| [`TabBarResult`](ui/type-TabBarResult.md) | struct |  | What a tab bar answered this tick. |
| [`CarouselDesc`](ui/type-CarouselDesc.md) | desc struct |  | Pages side by side that a drag or a flick moves, snapping to a page. |
| [`CarouselResult`](ui/type-CarouselResult.md) | struct |  | What a carousel answered this tick. |
| [`Crumb`](ui/type-Crumb.md) | desc struct |  | A step of a breadcrumb path. |
| [`BreadcrumbsDesc`](ui/type-BreadcrumbsDesc.md) | desc struct |  | The path to where the player is; every crumb but the last is a link back. |
| [`BreadcrumbsResult`](ui/type-BreadcrumbsResult.md) | struct |  | What breadcrumbs answered this tick. |
| [`PaginationDesc`](ui/type-PaginationDesc.md) | desc struct |  | Previous and next, the first and last pages and the pages around the current one. |
| [`PageResult`](ui/type-PageResult.md) | struct |  | What pagination answered this tick. |
| [`ActionHint`](ui/type-ActionHint.md) | desc struct |  | A hint of the action bar: an action, its glyph on the device in use and what it does. |
| [`ActionHintsDesc`](ui/type-ActionHintsDesc.md) | desc struct |  | The action bar (docs/ui.md §2.1): the actions the top layer accepts, each its glyph and its hint, at an edge. |
| [`ActionHintsResult`](ui/type-ActionHintsResult.md) | struct |  | What the action bar answered this tick. |
| [`RadialItem`](ui/type-RadialItem.md) | desc struct |  | A slice of a radial wheel. |
| [`RadialDesc`](ui/type-RadialDesc.md) | desc struct |  | A radial wheel: slices around a centre, chosen by where the pointer or a stick points. |
| [`RadialResult`](ui/type-RadialResult.md) | struct |  | What a radial wheel answered this tick. |
| [`DataStateDesc`](ui/type-DataStateDesc.md) | desc struct |  | What a data view shows instead of its rows: loading, an error, or nothing. |
| [`DataListDesc`](ui/type-DataListDesc.md) | desc struct |  | A virtualised list with its empty, loading and error states. |
| [`DataListResult`](ui/type-DataListResult.md) | struct |  | What a data list or grid answered this tick. |
| [`DataGridDesc`](ui/type-DataGridDesc.md) | desc struct |  | A virtualised grid of equal cells with its empty, loading and error states. |
| [`NumberFormat`](ui/type-NumberFormat.md) | enum |  | How a number reads in a data view. |
| [`TableColumn`](ui/type-TableColumn.md) | desc struct |  | A column of a table: the row field it shows and how. |
| [`TableDesc`](ui/type-TableDesc.md) | desc struct |  | A sortable, filterable table of rows by columns; only the rows in view are built. |
| [`TableResult`](ui/type-TableResult.md) | struct |  | What a table answered this tick: keep the sort and the filter. |
| [`TreeItem`](ui/type-TreeItem.md) | desc struct |  | An item of a tree; an item whose `parent` names another's id sits under it. |
| [`TreeDesc`](ui/type-TreeDesc.md) | desc struct |  | A tree of expandable items; the visible rows are virtualised. |
| [`TreeResult`](ui/type-TreeResult.md) | struct |  | What a tree answered this tick. |
| [`GraphNodeState`](ui/type-GraphNodeState.md) | enum |  | A graph node's state, shown by its ring and a mark as well as its colour. |
| [`GraphNodeShape`](ui/type-GraphNodeShape.md) | enum |  | A graph node's outline. |
| [`GraphNode`](ui/type-GraphNode.md) | desc struct |  | A node of a graph, at a position in the graph's own units. |
| [`GraphLink`](ui/type-GraphLink.md) | desc struct |  | A link between two nodes. |
| [`NodeGraphDesc`](ui/type-NodeGraphDesc.md) | desc struct |  | A pan-and-zoom graph of nodes and links: skill and tech trees, run maps, flowcharts. |
| [`NodeGraphResult`](ui/type-NodeGraphResult.md) | struct |  | What a graph answered this tick. |
| [`ChartKind`](ui/type-ChartKind.md) | enum |  | What a chart draws. |
| [`ChartSeries`](ui/type-ChartSeries.md) | desc struct |  | A series of a chart: its name and a value a category. |
| [`ChartDesc`](ui/type-ChartDesc.md) | desc struct |  | A chart: line, area, bar, radar, pie or donut, with axes, a legend, a tooltip and its states. |
| [`ChartResult`](ui/type-ChartResult.md) | struct |  | What a chart answered this tick. |
| [`KeyValueRow`](ui/type-KeyValueRow.md) | desc struct |  | A row of key-value rows. |
| [`KeyValueRowsDesc`](ui/type-KeyValueRowsDesc.md) | desc struct |  | Key-value rows: a stat sheet, an item's properties. |
| [`StatCompareRow`](ui/type-StatCompareRow.md) | desc struct |  | A stat compared: its value now and the value it would take. |
| [`StatCompareDesc`](ui/type-StatCompareDesc.md) | desc struct |  | Stat-compare rows: an equip screen's comparison, each change a signed delta with a mark. |
| [`CalendarMark`](ui/type-CalendarMark.md) | desc struct |  | Something on a day: a dot in a tone, or a claimed tick for a login calendar. |
| [`CalendarDesc`](ui/type-CalendarDesc.md) | desc struct |  | A month grid of days: today, a selected day, a range, marks, and days that cannot be chosen. |
| [`CalendarResult`](ui/type-CalendarResult.md) | struct |  | What a calendar answered this tick. |
| [`og.ui.card`](ui/fn-card.md) | function | `(id: string, desc: CardDesc, build: UiBuild): Response` | A card over a body the game builds. |
| [`og.ui.scrollArea`](ui/fn-scrollArea.md) | function | `(id: string, desc: ScrollAreaDesc, build: UiBuild): void` | A scroll container whose content fades out at an edge where more lies beyond. |
| [`og.ui.accordion`](ui/fn-accordion.md) | function | `(id: string, desc: AccordionDesc, build: UiItemBuild): AccordionResult` | Sections whose headers open and close their bodies; the sections under one move on the panel spring. |
| [`og.ui.splitView`](ui/fn-splitView.md) | function | `(id: string, desc: SplitViewDesc, start: UiBuild, end: UiBuild): SplitResult` | Two panes with a divider a drag moves and a press of the focused divider steps. |
| [`og.ui.dialog`](ui/fn-dialog.md) | function | `(id: string, open: boolean, desc: DialogDesc, build: UiBuild): DialogResult` | A dialog on the modal layer: tone mark, message, body and actions. |
| [`og.ui.confirm`](ui/fn-confirm.md) | function | `(id: string, open: boolean, desc?: ConfirmationDesc): ConfirmResult` | A confirmation dialog: the message, Cancel and the action (danger when it cannot be undone). |
| [`og.ui.popover`](ui/fn-popover.md) | function | `(id: string, open: boolean, desc: PopoverDesc, build: UiBuild): PopoverResult` | A popover beside an element with an arrow at it; Back or a press outside dismisses it. |
| [`og.ui.menu`](ui/fn-menu.md) | function | `(id: string, open: boolean, desc?: MenuDesc): MenuChoice` | A menu of commands, checks, radios, separators, headers and submenus, hanging from an element or at a point. |
| [`og.ui.contextMenu`](ui/fn-contextMenu.md) | function | `(id: string, open: boolean, at: Vector2, desc?: MenuDesc): MenuChoice` | A menu at a point: where a right press or a long press happened. |
| [`og.ui.sheet`](ui/fn-sheet.md) | function | `(id: string, open: boolean, desc: SheetDesc, build: UiBuild): SheetResult` | A sheet sliding in from the bottom (resting at detents, dragged) or a side. |
| [`og.ui.tabBar`](ui/fn-tabBar.md) | function | `(id: string, desc: TabBarDesc, build: UiItemBuild): TabBarResult` | Tabs with a sliding indicator and bumpers over the panel of the selected one. |
| [`og.ui.carousel`](ui/fn-carousel.md) | function | `(id: string, desc: CarouselDesc, build: UiItemBuild): CarouselResult` | Pages a drag or a flick moves, snapping to a page, with arrows and dots. |
| [`og.ui.breadcrumbs`](ui/fn-breadcrumbs.md) | function | `(id: string, desc?: BreadcrumbsDesc): BreadcrumbsResult` | The path to where the player is, collapsing its middle into a menu when long. |
| [`og.ui.pagination`](ui/fn-pagination.md) | function | `(id: string, desc?: PaginationDesc): PageResult` | Previous, next and the pages around the current one. |
| [`og.ui.actionHints`](ui/fn-actionHints.md) | function | `(id: string, desc?: ActionHintsDesc): ActionHintsResult` | The action bar: each action the top layer accepts as its glyph and hint; on touch, buttons. |
| [`og.ui.radialMenu`](ui/fn-radialMenu.md) | function | `(id: string, open: boolean, desc?: RadialDesc): RadialResult` | A radial wheel chosen by where the pointer or a stick points. |
| [`og.ui.dataList`](ui/fn-dataList.md) | function | `(id: string, desc: DataListDesc, build: UiItemBuild): DataListResult` | A virtualised list with its empty, loading and error states. |
| [`og.ui.dataGrid`](ui/fn-dataGrid.md) | function | `(id: string, desc: DataGridDesc, build: UiItemBuild): DataListResult` | A virtualised grid of cells with its empty, loading and error states. |
| [`og.ui.table`](ui/fn-table.md) | function | `(id: string, desc?: TableDesc): TableResult` | A sortable, filterable table; only the rows in view are built, so 10,000 rows and more stay within budget. |
| [`og.ui.tree`](ui/fn-tree.md) | function | `(id: string, desc?: TreeDesc): TreeResult` | A tree of expandable items, virtualised. |
| [`og.ui.nodeGraph`](ui/fn-nodeGraph.md) | function | `(id: string, desc?: NodeGraphDesc): NodeGraphResult` | A pan-and-zoom graph of nodes and links: skill trees, run maps, flowcharts. |
| [`og.ui.chart`](ui/fn-chart.md) | function | `(id: string, desc?: ChartDesc): ChartResult` | A line, area, bar, radar, pie or donut chart. |
| [`og.ui.keyValueRows`](ui/fn-keyValueRows.md) | function | `(id: string, desc?: KeyValueRowsDesc): void` | Key-value rows: labels at the start, values at the end. |
| [`og.ui.statCompare`](ui/fn-statCompare.md) | function | `(id: string, desc?: StatCompareDesc): void` | Stat-compare rows: current, new and the signed change with its mark. |
| [`og.ui.calendar`](ui/fn-calendar.md) | function | `(id: string, desc?: CalendarDesc): CalendarResult` | A month grid of days. |
| [`TextRole`](ui/type-TextRole.md) | enum |  | A text's role in the hierarchy (docs/ui.md §3.1): its type step, font and weight come from the style's type tokens for the role. |
| [`TextTone`](ui/type-TextTone.md) | enum |  | A text's ink by meaning. |
| [`TextBlockDesc`](ui/type-TextBlockDesc.md) | desc struct |  | How a text block reads: its role, tone and how it fits. |
| [`og.ui.text`](ui/fn-text.md) | function | `(id: string, value: string, desc?: TextBlockDesc): void` | Text in a role (docs/ui.md §2, Foundations): the style's type step, font and weight for display, title, heading, body, label, caption or numeric (monospaced figures), in a tone, cut after maxLines wi… |
| [`ProseDesc`](ui/type-ProseDesc.md) | desc struct |  | A passage of rich text: paragraphs at a readable measure. |
| [`og.ui.prose`](ui/fn-prose.md) | function | `(id: string, markup: string, desc?: ProseDesc): RichTextResult` | A passage of rich text (og.ui.richText's markup: `[b]`, `[color=…]`, `[icon=…]`, `[link=…]`, `[wave]`…), split into paragraphs at blank lines, at a readable measure, each a text element `<id>/P#i` wh… |
| [`IconPlate`](ui/type-IconPlate.md) | enum |  | What an icon sits on. |
| [`IconMarkDesc`](ui/type-IconMarkDesc.md) | desc struct |  | An icon's size, tone, plate and badge. |
| [`og.ui.iconMark`](ui/fn-iconMark.md) | function | `(id: string, name: string, desc?: IconMarkDesc): void` | An icon of the style's set (og.ui.icon) in a tone, on a plate, with a badge or a dot: the foundation every icon in a control is. |
| [`ArtFit`](ui/type-ArtFit.md) | enum |  | How art fills its slot. |
| [`ArtMask`](ui/type-ArtMask.md) | enum |  | What art is cut to. |
| [`ArtSlotDesc`](ui/type-ArtSlotDesc.md) | desc struct |  | An art slot (docs/ui.md §3.3): where a generated or authored image goes, and what it shows until it is there. |
| [`og.ui.artSlot`](ui/fn-artSlot.md) | function | `(id: string, desc?: ArtSlotDesc): void` | An image or art slot: the texture fitted and cut as its desc says, fading in when it arrives (the style's enter motion; at once under reduced motion); until then a placeholder drawn from the style (a… |
| [`ShapePreset`](ui/type-ShapePreset.md) | enum |  | A decorative shape. |
| [`ShapeBoxDesc`](ui/type-ShapeBoxDesc.md) | desc struct |  | A decorative shape: its outline, fill, stroke and glow, in the style's colours. |
| [`og.ui.shapeBox`](ui/fn-shapeBox.md) | function | `(id: string, desc?: ShapeBoxDesc): void` | A decorative shape drawn by the UI pass (docs/ui.md §3.3) in the style's colours: a pip, a gem, a star, a ring, a backdrop blob. |
| [`SeparatorDesc`](ui/type-SeparatorDesc.md) | desc struct |  | A separator: a hairline, upright or across, with an optional word in its middle. |
| [`og.ui.separator`](ui/fn-separator.md) | function | `(id: string, desc?: SeparatorDesc): void` | A separator between groups: the style's line (its divider ornament where the style has one and no label is given), across or upright, with a label in its middle between two lines. |
| [`SpaceStep`](ui/type-SpaceStep.md) | enum |  | A step of the style's spacing scale. |
| [`SpaceDesc`](ui/type-SpaceDesc.md) | desc struct |  | Empty space by the style's scale. |
| [`og.ui.space`](ui/fn-space.md) | function | `(id: string, desc?: SpaceDesc): void` | Empty space of a step of the style's spacing scale (so a style's density moves every gap at once), or all the room left. |
| [`ViewFrameDesc`](ui/type-ViewFrameDesc.md) | desc struct |  | A framed 3D view (docs/ui.md §3.3): a camera's render target in the layout, turned by a drag, the wheel, keys and a pad. |
| [`ViewFrameResult`](ui/type-ViewFrameResult.md) | struct |  | A view frame's turn, tilt and zoom after this tick's input, and its viewport's answer. |
| [`og.ui.viewFrame`](ui/fn-viewFrame.md) | function | `(id: string, desc?: ViewFrameDesc): ViewFrameResult` | A viewport in a frame (docs/ui.md §3.3, Foundations): og.ui.sceneViewport in a style-dressed slot that turns with a drag (keeping the fling's speed for a moment), tilts within its limits, zooms with … |
| [`EmitterPreset`](ui/type-EmitterPreset.md) | enum |  | A particle look. |
| [`EmitterDesc`](ui/type-EmitterDesc.md) | desc struct |  | An ambient particle emitter over the area of its parent. |
| [`og.ui.emitter`](ui/fn-emitter.md) | function | `(id: string, desc?: EmitterDesc): void` | An ambient particle emitter (docs/ui.md §3.3) laid over its parent's area: sparkles on a rare card, embers on a forge, snow over a winter menu, confetti on a win. |
| [`EffectLayerDesc`](ui/type-EffectLayerDesc.md) | desc struct |  | A subtree drawn offscreen and composited through a material, with the material's one number that matters as `amount`. |
| [`og.ui.effectLayer`](ui/fn-effectLayer.md) | function | `(id: string, desc: EffectLayerDesc, build: UiBuild): void` | A layer (docs/ui.md §3.3): its subtree, built by `build`, drawn offscreen and composited through a material (a card dissolving as it is spent, a shine passing over a rare item, a hologram panel, a hi… |
| [`PushButtonDesc`](ui/type-PushButtonDesc.md) | desc struct |  | A button: its role, size, icons, badge and states. |
| [`og.ui.pushButton`](ui/fn-pushButton.md) | function | `(id: string, label: string, desc?: PushButtonDesc): Response` | A button (docs/ui.md §2, Controls): primary, secondary, ghost (outlined), quiet (text) or danger, small, medium or large, with icons, a badge, a loading spinner and a disabled reason. |
| [`IconButtonDesc`](ui/type-IconButtonDesc.md) | desc struct |  | An icon button: an icon, its name and a badge. |
| [`og.ui.iconButton`](ui/fn-iconButton.md) | function | `(id: string, icon: string, desc?: IconButtonDesc): Response` | An icon button: a square (or round) button of the control height around one icon, named by its label (its tooltip, and what a screen reader says), with a badge or a dot. |
| [`HoldButtonDesc`](ui/type-HoldButtonDesc.md) | desc struct |  | A hold-to-confirm button. |
| [`HoldResult`](ui/type-HoldResult.md) | struct |  | A hold button's state after this tick. |
| [`og.ui.holdButton`](ui/fn-holdButton.md) | function | `(id: string, label: string, desc?: HoldButtonDesc): HoldResult` | A hold-to-confirm button: held (a press on it, a touch), it fills from its start edge over holdMs, ticking up in pitch, and confirms with the style's confirm sound and a pop; let go early, it drains … |
| [`LabelSide`](ui/type-LabelSide.md) | enum |  | Which side of a switch or a check its label sits. |
| [`ToggleSwitchDesc`](ui/type-ToggleSwitchDesc.md) | desc struct |  | A switch: its label, description and states. |
| [`SwitchResult`](ui/type-SwitchResult.md) | struct |  | A switch's or a check's state after this tick. |
| [`og.ui.toggleSwitch`](ui/fn-toggleSwitch.md) | function | `(id: string, on: boolean, desc?: ToggleSwitchDesc): SwitchResult` | A switch: a track and a knob that slides from its start to its end as it turns on (its selection group, on the style's motion), the track filling with primary, with a label and a description; the who… |
| [`CheckFieldDesc`](ui/type-CheckFieldDesc.md) | desc struct |  | A checkbox: its label, description and states. |
| [`og.ui.checkField`](ui/fn-checkField.md) | function | `(id: string, checked: boolean, desc?: CheckFieldDesc): SwitchResult` | A checkbox: a box that fills with primary and draws its tick (a dash when mixed) with a pop as it turns on (its selection group), with a label and a description; the whole row takes a press, Enter, S… |
| [`RadioOption`](ui/type-RadioOption.md) | desc struct |  | One choice of a radio group. |
| [`RadioVariant`](ui/type-RadioVariant.md) | enum |  | How a radio group draws its choices. |
| [`RadioGroupDesc`](ui/type-RadioGroupDesc.md) | desc struct |  | A radio group: its choices and layout. |
| [`ChoiceIndexResult`](ui/type-ChoiceIndexResult.md) | struct |  | A choice control's state after this tick. |
| [`og.ui.radioGroup`](ui/fn-radioGroup.md) | function | `(id: string, desc?: RadioGroupDesc): ChoiceIndexResult` | A radio group: one of several choices, as rings with labels or as cards. |
| [`SliderMark`](ui/type-SliderMark.md) | struct |  | A labelled point on a slider's track. |
| [`ValueFormat`](ui/type-ValueFormat.md) | desc struct |  | How a control shows a number. |
| [`ValueSliderDesc`](ui/type-ValueSliderDesc.md) | desc struct |  | A slider: range, step, label, readout, ticks and marks. |
| [`ValueSliderResult`](ui/type-ValueSliderResult.md) | struct |  | A slider's value after this tick. |
| [`og.ui.valueSlider`](ui/fn-valueSlider.md) | function | `(id: string, value: number, desc?: ValueSliderDesc): ValueSliderResult` | A slider: a track, a fill from min (or from an origin, both ways) and a knob. |
| [`RangeSliderDesc`](ui/type-RangeSliderDesc.md) | desc struct |  | A range slider: two knobs on one track. |
| [`RangeResult`](ui/type-RangeResult.md) | struct |  | A range slider's values after this tick. |
| [`og.ui.rangeSlider`](ui/fn-rangeSlider.md) | function | `(id: string, low: number, high: number, desc?: RangeSliderDesc): RangeResult` | A range slider: two knobs on one track and the fill between them. |
| [`NumberStepperDesc`](ui/type-NumberStepperDesc.md) | desc struct |  | A number stepper: minus, the value and plus. |
| [`NumberResult`](ui/type-NumberResult.md) | struct |  | A stepper's value after this tick. |
| [`og.ui.numberStepper`](ui/fn-numberStepper.md) | function | `(id: string, value: number, desc?: NumberStepperDesc): NumberResult` | A number stepper: minus, the value and plus. |
| [`CyclerDesc`](ui/type-CyclerDesc.md) | desc struct |  | An option cycler: a setting's levels in one row. |
| [`og.ui.cycler`](ui/fn-cycler.md) | function | `(id: string, options: readonly string[], index: number, desc?: CyclerDesc): ChoiceIndexResult` | An option cycler, the console settings row: `‹ High ›` with pips. |
| [`SegmentItem`](ui/type-SegmentItem.md) | desc struct |  | One segment of a segmented control. |
| [`SegmentedDesc`](ui/type-SegmentedDesc.md) | desc struct |  | A segmented control: its segments and layout. |
| [`og.ui.segmented`](ui/fn-segmented.md) | function | `(id: string, desc?: SegmentedDesc): ChoiceIndexResult` | A segmented control (segmented tabs): a track whose raised thumb slides to the chosen segment and takes its width on the style's spring (layout animation; it jumps under reduced motion). |
| [`DropdownOption`](ui/type-DropdownOption.md) | desc struct |  | One option of a dropdown. |
| [`DropdownDesc`](ui/type-DropdownDesc.md) | desc struct |  | A dropdown: its options, label and states. |
| [`DropdownResult`](ui/type-DropdownResult.md) | struct |  | A dropdown's state after this tick. |
| [`og.ui.dropdown`](ui/fn-dropdown.md) | function | `(id: string, desc?: DropdownDesc): DropdownResult` | A dropdown (a select): a field-like trigger with the chosen option and a chevron that turns as it opens; its list opens below it (above without room, kept on screen) as a floating layer that takes th… |
| [`InputFieldDesc`](ui/type-InputFieldDesc.md) | desc struct |  | A text field with its label, helper, error and adornments. |
| [`InputResult`](ui/type-InputResult.md) | struct |  | A text field's state after this tick. |
| [`og.ui.inputField`](ui/fn-inputField.md) | function | `(id: string, value: string, desc?: InputFieldDesc): InputResult` | A text field (og.ui.textField's editing: IME, selection, clipboard, undo) in a framed row with a label above, an icon, a suffix, a clear button and a helper, error or counter line below. |
| [`InputAreaDesc`](ui/type-InputAreaDesc.md) | desc struct |  | A text area with its label, helper, error and counter. |
| [`og.ui.inputArea`](ui/fn-inputArea.md) | function | `(id: string, value: string, desc?: InputAreaDesc): InputResult` | A text area (og.ui.textArea's editing) framed as inputField is, with its label, helper, error and counter. |
| [`SearchBoxDesc`](ui/type-SearchBoxDesc.md) | desc struct |  | A search box and its suggestions. |
| [`SearchResult`](ui/type-SearchResult.md) | struct |  | A search box's state after this tick. |
| [`og.ui.searchBox`](ui/fn-searchBox.md) | function | `(id: string, query: string, desc?: SearchBoxDesc): SearchResult` | A search box with suggestions: a field with a lens, a clear button and, while it has the focus and something matches, a list of suggestions under it (recent searches while it is empty) whose matching… |
| [`PickedColor`](ui/type-PickedColor.md) | desc struct |  | A colour as a picker answers it: sRGB components 0 to 1 and alpha. |
| [`ColorPickerDesc`](ui/type-ColorPickerDesc.md) | desc struct |  | A colour picker: its swatches and the parts it shows. |
| [`ColorPickerResult`](ui/type-ColorPickerResult.md) | struct |  | A colour picker's colour after this tick. |
| [`og.ui.colorPicker`](ui/fn-colorPicker.md) | function | `(id: string, color?: PickedColor, desc?: ColorPickerDesc): ColorPickerResult` | A colour picker: a saturation and brightness square with a ring at the colour, a hue strip, an opacity strip, swatches and a hex field, with the colour before and after. |
| [`KeyBindingRowDesc`](ui/type-KeyBindingRowDesc.md) | desc struct |  | A controls screen's row for one action: its name, its bindings on a device, and their capture. |
| [`KeyBindingResult`](ui/type-KeyBindingResult.md) | struct |  | What a key-binding row did this tick. |
| [`og.ui.keyBindingRow`](ui/fn-keyBindingRow.md) | function | `(world: World, id: string, desc?: KeyBindingRowDesc): KeyBindingResult` | A key-binding row (the settings tree's controls page, docs/ui.md §2): the action's name and a slot per binding on a device (keyboard and mouse, or a pad, the device used last by default) showing its … |
| [`KeyboardLayout`](ui/type-KeyboardLayout.md) | enum |  | An on-screen keyboard's keys. |
| [`KeyboardDesc`](ui/type-KeyboardDesc.md) | desc struct |  | An on-screen keyboard: what it edits and how it types. |
| [`KeyboardResult`](ui/type-KeyboardResult.md) | struct |  | What an on-screen keyboard did this tick. |
| [`og.ui.onScreenKeyboard`](ui/fn-onScreenKeyboard.md) | function | `(id: string, open: boolean, desc?: KeyboardDesc): KeyboardResult` | An on-screen keyboard for pads and touch (docs/ui.md §2: search and names with a pad): a panel at the screen's bottom with the text being typed and its caret, a candidate bar while composing, and key… |
| [`HeroTone`](ui/type-HeroTone.md) | enum |  | A call to action's material. |
| [`HeroButtonDesc`](ui/type-HeroButtonDesc.md) | desc struct |  | A call to action: tone, height, a keycap and whether it can be pressed now. |
| [`og.ui.heroButton`](ui/fn-heroButton.md) | function | `(id: string, label: string, desc?: HeroButtonDesc): Response` | The one action a genre screen leads with (End turn, Summon, Play, Ready): a raised face lit from the top over a ledge of its own dark and a soft shadow on the scene, so it reads as a thing to press o… |
| [`StarBarDesc`](ui/type-StarBarDesc.md) | desc struct |  | A level's score toward its stars. |
| [`StarBarResult`](ui/type-StarBarResult.md) | struct |  | What the bar showed this tick. |
| [`og.ui.starBar`](ui/fn-starBar.md) | function | `(id: string, desc?: StarBarDesc): StarBarResult` | A level's score toward its stars (Candy Crush, Homescapes, Angry Birds): the score in the display face over a bar that fills in a warm gold, a star standing on the bar at each threshold, lit as the s… |
| [`MapOverlay`](ui/type-MapOverlay.md) | desc struct |  | One of the map's data overlays (Traffic, Power, Land value), with its legend's ramp. |
| [`OverlayBarDesc`](ui/type-OverlayBarDesc.md) | desc struct |  | A city or strategy map's overlay picker: the overlays, the one shown and its legend. |
| [`OverlayBarResult`](ui/type-OverlayBarResult.md) | struct |  | What the player asked for this tick. |
| [`og.ui.overlayBar`](ui/fn-overlayBar.md) | function | `(id: string, desc?: OverlayBarDesc): OverlayBarResult` | A map's overlay picker (Cities: Skylines' info views, Civilization's lenses): a row of round buttons, each its overlay's art, the shown one lit with its name, and over the bar the shown overlay's leg… |
| [`SettingsCustomBuild`](ui/type-SettingsCustomBuild.md) | callback |  | Builds a custom settings entry's control inside its row: ui calls made inside nest under it. |
| [`SettingKind`](ui/type-SettingKind.md) | enum |  | What a settings entry is: the control its row draws. |
| [`SettingEntry`](ui/type-SettingEntry.md) | desc struct |  | One entry of a settings page, as data: a game adds, drops or reorders entries without touching the pattern. |
| [`SettingsPage`](ui/type-SettingsPage.md) | desc struct |  | One page of the settings tree: a tab, and its entries in order. |
| [`SettingChange`](ui/type-SettingChange.md) | struct |  | One entry changed by the player this tick. |
| [`SettingsTreeDesc`](ui/type-SettingsTreeDesc.md) | desc struct |  | The settings as a tree of pages and typed entries, the pending state, and the confirm countdown. |
| [`SettingsTreeResult`](ui/type-SettingsTreeResult.md) | struct |  | What the player asked for this tick, for the game to apply to its settings. |
| [`og.ui.settingsTree`](ui/fn-settingsTree.md) | function | `(world: World, id: string, desc: SettingsTreeDesc, custom: SettingsCustomBuild): SettingsTreeResult` | A game's settings as data (docs/ui.md §2, roadmap G1-7): the title, the pages as tabs (the tab actions step them), the shown page's entries as rows, each its name (a changed one marked) and its contr… |
| [`UiProp`](ui/type-UiProp.md) | enum |  | An animatable value of an element (docs/ui.md §3.1): a state, a timeline key, a presence or og.ui.animate moves it. |
| [`LayoutMode`](ui/type-LayoutMode.md) | enum |  | How a container lays out its children. |
| [`Position`](ui/type-Position.md) | enum |  | How an element is placed. |
| [`GridFlow`](ui/type-GridFlow.md) | enum |  | Which way auto-placed items fill a grid. |
| [`DragAxis`](ui/type-DragAxis.md) | enum |  | Which way a drag moves its element. |
| [`EaseName`](ui/type-EaseName.md) | enum |  | A named easing curve. |
| [`MotionToken`](ui/type-MotionToken.md) | enum |  | A named motion of the style (docs/ui.md §3.1): components use these, so a style's personality changes every motion at once. |
| [`ReducedMotion`](ui/type-ReducedMotion.md) | enum |  | What an animation does under reduced motion (docs/ui.md §3.1). |
| [`Orientation`](ui/type-Orientation.md) | enum |  | A screen orientation. |
| [`Repeat`](ui/type-Repeat.md) | enum |  | How a play repeats. |
| [`UiTransform`](ui/type-UiTransform.md) | desc struct |  | A transform applied after layout, about its pivot: CSS's translate(x, y) rotate(r) skew(kx, ky) scale(sx, sy) with its transform-origin. |
| [`Edges`](ui/type-Edges.md) | desc struct |  | Insets of an absolute or anchored element from its parent's edges; an absent one is not set. |
| [`TrackPx`](ui/type-TrackPx.md) | struct |  | A fixed track. |
| [`TrackFr`](ui/type-TrackFr.md) | struct |  | A share of the space the other tracks leave. |
| [`TrackRange`](ui/type-TrackRange.md) | struct |  | A track sized by its content, within bounds. |
| [`TrackPercent`](ui/type-TrackPercent.md) | struct |  | A fraction of the grid's inner size. |
| [`TrackAuto`](ui/type-TrackAuto.md) | struct |  | Its largest single-cell item. |
| [`Track`](ui/type-Track.md) | union |  | A grid track's size. |
| [`GridSpec`](ui/type-GridSpec.md) | desc struct |  | A grid's tracks and areas. |
| [`GridPlace`](ui/type-GridPlace.md) | desc struct |  | Where an item sits in its parent grid; the zero value is auto-placed in one cell. |
| [`PropValue`](ui/type-PropValue.md) | struct |  | A value for an animatable prop. |
| [`UiQuery`](ui/type-UiQuery.md) | desc struct |  | When a variant applies: every condition given must hold. |
| [`UiPatch`](ui/type-UiPatch.md) | desc struct |  | What a variant changes: the box's layout, its arrangement and animatable values; absent fields are unchanged. |
| [`UiVariant`](ui/type-UiVariant.md) | struct |  | A prop override under a condition (docs/ui.md §2: breakpoint variants and container queries). |
| [`SpringSpec`](ui/type-SpringSpec.md) | desc struct |  | A spring by how it feels: about how long it takes to settle and how much it overshoots. |
| [`TweenSpec`](ui/type-TweenSpec.md) | desc struct |  | A duration on an ease: a retarget starts a new ease from where it is. |
| [`TokenRef`](ui/type-TokenRef.md) | struct |  | One of the style's motion tokens. |
| [`InstantSpec`](ui/type-InstantSpec.md) | struct |  | At once. |
| [`Transition`](ui/type-Transition.md) | union |  | How a value moves to a new target. |
| [`DismissSpec`](ui/type-DismissSpec.md) | desc struct |  | Swipe to dismiss: a release past distance (or faster than velocity) along the drag's axis throws the element off. |
| [`SnapPoint`](ui/type-SnapPoint.md) | struct |  | An offset a released drag can settle on. |
| [`DragSpec`](ui/type-DragSpec.md) | desc struct |  | The element moves with a drag that starts on it (a card, a sheet, a carousel), released onto a spring that keeps the drag's velocity. |
| [`PinchSpec`](ui/type-PinchSpec.md) | desc struct |  | Two touches on the element scale (and turn) it. |
| [`DropSpec`](ui/type-DropSpec.md) | desc struct |  | The element takes drops of dragged elements. |
| [`PresenceSpec`](ui/type-PresenceSpec.md) | desc struct |  | How an element arrives or leaves (docs/ui.md §3.1): the values it enters from or exits to, and how it moves between them and its own. |
| [`VisualState`](ui/type-VisualState.md) | struct |  | One state of a group: the values it sets, and timelines it starts. |
| [`StateTransition`](ui/type-StateTransition.md) | struct |  | The transition between two states of a group (`*` matches any state). |
| [`StateGroup`](ui/type-StateGroup.md) | struct |  | A visual state group (docs/ui.md §3.1): one of its states is current, and moving between them animates every value that changes. |
| [`StateValue`](ui/type-StateValue.md) | struct |  | The current state of a group. |
| [`NavLinks`](ui/type-NavLinks.md) | desc struct |  | Explicit focus moves from an element (docs/ui.md §2.1), over the spatial pick: each a path (a full one, `Hud/Bag`, or a sibling's id); empty leaves that move to the layout. |
| [`FocusGroup`](ui/type-FocusGroup.md) | desc struct |  | A container whose focusable controls are one group for keys and pads (docs/ui.md §2.1): arrows move among its members first. |
| [`GestureSpec`](ui/type-GestureSpec.md) | desc struct |  | Tap, double tap and long press on any element (docs/ui.md §2.1), answered by Response (tapped, doubleTapped, longPressed, longPress), og.ui.gesture and the frame's motion events. |
| [`Customize`](ui/type-Customize.md) | desc struct |  | An element the player can move, scale and fade in the HUD edit mode (docs/ui.md §2.1): a HUD group, a touch stick or button. |
| [`Cursor`](ui/type-Cursor.md) | enum |  | The pointer's shape over an element (CSS cursor, docs/ui.md §2), inherited by its subtree. |
| [`ElementProps`](ui/type-ElementProps.md) | desc struct |  | Everything an element carries beyond its box (docs/ui.md §2): set with og.ui.props right after the element. |
| [`EaseKey`](ui/type-EaseKey.md) | struct |  | A named ease into a key. |
| [`HoldKey`](ui/type-HoldKey.md) | struct |  | Holds the previous value, then jumps at the key. |
| [`LinearKey`](ui/type-LinearKey.md) | struct |  | Constant speed into the key. |
| [`BezierKey`](ui/type-BezierKey.md) | struct |  | A cubic Bézier into the key. |
| [`KeyEase`](ui/type-KeyEase.md) | union |  | How the segment into a key moves. |
| [`TimelineKey`](ui/type-TimelineKey.md) | struct |  | A keyframe: a value at a time. |
| [`TimelineTrack`](ui/type-TimelineTrack.md) | struct |  | Keys over one prop of the element the timeline plays on, or of a descendant by its path below it (`Stars/Star#2`). |
| [`SoundEvent`](ui/type-SoundEvent.md) | struct |  | A UI sound of the style. |
| [`HapticEvent`](ui/type-HapticEvent.md) | struct |  | A haptic pattern for the host. |
| [`BurstEvent`](ui/type-BurstEvent.md) | struct |  | Rays and sparks from the target. |
| [`StateEvent`](ui/type-StateEvent.md) | struct |  | Sets a state group of the target. |
| [`CallbackEvent`](ui/type-CallbackEvent.md) | struct |  | A callback to game code: it arrives in og.ui.motionEvents. |
| [`TimelineAction`](ui/type-TimelineAction.md) | union |  | What an event does when a play reaches it. |
| [`TimelineEvent`](ui/type-TimelineEvent.md) | struct |  | An event at a time, on the play's element or a descendant. |
| [`TimelineLabel`](ui/type-TimelineLabel.md) | struct |  | A named time, for seeking (PlayDesc.fromLabel). |
| [`UiTimeline`](ui/type-UiTimeline.md) | desc struct |  | Keyframed tracks and events (docs/ui.md §3.1). |
| [`ChoreoPlay`](ui/type-ChoreoPlay.md) | struct |  | A timeline on the play's element or a descendant. |
| [`ChoreoSteps`](ui/type-ChoreoSteps.md) | struct |  | Steps composed. |
| [`ChoreoStagger`](ui/type-ChoreoStagger.md) | struct |  | One timeline on each target, each stepMs after the one before as ordered by from, the whole spread within capMs. |
| [`ChoreoCascade`](ui/type-ChoreoCascade.md) | struct |  | Steps where each starts at the first `on` callback event of the one before (or its end): a score that counts chips, then multiplies, each step pitched higher. |
| [`Choreo`](ui/type-Choreo.md) | union |  | Timelines composed into one play (docs/ui.md §3.1). |
| [`PlayDesc`](ui/type-PlayDesc.md) | desc struct |  | How a play runs. |
| [`PlayInfo`](ui/type-PlayInfo.md) | struct |  | Where a play is. |
| [`GestureInfo`](ui/type-GestureInfo.md) | struct |  | What a drag or pinch on an element is doing (last tick's answer, as responses are). |
| [`UiMotionEvent`](ui/type-UiMotionEvent.md) | struct |  | What happened in the UI's motion this tick. |
| [`UiOverlap`](ui/type-UiOverlap.md) | struct |  | Two elements overlapping where neither declared it (docs/ui.md §2). |
| [`UiLayoutStats`](ui/type-UiLayoutStats.md) | struct |  | What the last frame rebuilt and laid out (docs/ui.md §2). |
| [`og.ui.props`](ui/fn-props.md) | function | `(desc?: ElementProps): boolean` | Give the element built just before (any widget, or a container just closed) the element model's props: transform, opacity, z, placement, grid, variants, overlaps, motion, states, presence, gestures (… |
| [`og.ui.defineTimeline`](ui/fn-defineTimeline.md) | function | `(name: string, timeline?: UiTimeline): void` | Define a timeline by name: configuration, like the theme, defined once at start. |
| [`og.ui.play`](ui/fn-play.md) | function | `(path: string, choreo: Choreo, desc?: PlayDesc): number` | Play a choreography on an element from the end of this tick: its timelines' keys drive the elements' props and its events fire as it passes them (og.ui.motionEvents). |
| [`og.ui.playTimeline`](ui/fn-playTimeline.md) | function | `(path: string, timeline: string, desc?: PlayDesc): number` | Play one timeline (by name) on an element: og.ui.play with a single step. |
| [`og.ui.seek`](ui/fn-seek.md) | function | `(play: number, ms: number): boolean` | Move a play to a time; it fires no events on the way. |
| [`og.ui.pause`](ui/fn-pause.md) | function | `(play: number): boolean` | Hold a play where it is. |
| [`og.ui.resume`](ui/fn-resume.md) | function | `(play: number): boolean` | Let a held play go on. |
| [`og.ui.reverse`](ui/fn-reverse.md) | function | `(play: number): boolean` | Turn a play round where it is: an interrupted opening closes from where it got to. |
| [`og.ui.stop`](ui/fn-stop.md) | function | `(play: number): boolean` | Stop a play: its elements go back to their own values (on their motion, if they have one). |
| [`og.ui.playInfo`](ui/fn-playInfo.md) | function | `(play: number): PlayInfo \| null` | Where a play is; null once it is gone. |
| [`og.ui.forceState`](ui/fn-forceState.md) | function | `(path: string, group: string, state: string \| null): void` | Force a state group of an element (the probe and agents render and check each state this way); null lets it go. |
| [`og.ui.openPath`](ui/fn-openPath.md) | function | `(): string` | The full path of the innermost open container (an open row, panel, anchor or world anchor), as bounds, visualState, forceState, playTimeline and the probe name it; empty at the top level. |
| [`og.ui.visualState`](ui/fn-visualState.md) | function | `(path: string, group: string): string` | The state a group of an element is in (last tick); empty for none. |
| [`og.ui.animate`](ui/fn-animate.md) | function | `(id: string, target: number, transition: Transition): number` | Any number on a spring or an ease (docs/ui.md §3.1: every value is animatable): call it every tick with the target and it answers this tick's value. |
| [`og.ui.gesture`](ui/fn-gesture.md) | function | `(path: string): GestureInfo` | What the drag or pinch on an element is doing. |
| [`og.ui.motionEvents`](ui/fn-motionEvents.md) | function | `(): readonly UiMotionEvent[]` | The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps. |
| [`og.ui.overlaps`](ui/fn-overlaps.md) | function | `(): readonly UiOverlap[]` | The last frame's undeclared overlaps between elements (docs/ui.md §2), with both paths. |
| [`og.ui.layoutStats`](ui/fn-layoutStats.md) | function | `(): UiLayoutStats` | What the last frame built, rebuilt, laid out and kept. |
| [`NotificationVariant`](ui/type-NotificationVariant.md) | enum |  | What a notification is, which sets its look, sound and default time. |
| [`Rarity`](ui/type-Rarity.md) | enum |  | A loot tier (the style's rarity colours, each paired with its pips so no tier is told by colour alone). |
| [`NotificationItem`](ui/type-NotificationItem.md) | desc struct |  | One notification of a stack, as the game's viewmodel holds it. |
| [`NotificationsDesc`](ui/type-NotificationsDesc.md) | desc struct |  | A stack of notifications (toasts) anchored in the safe area. |
| [`NotificationsResult`](ui/type-NotificationsResult.md) | struct |  | What a notification stack answered this tick. |
| [`og.ui.notifications`](ui/fn-notifications.md) | function | `(id: string, desc?: NotificationsDesc): NotificationsResult` | A notification stack (toasts, achievements, rewards, invites): cards anchored in the safe area, newest nearest the anchor's edge, that enter on the style's enter spring from that edge, reflow on its … |
| [`BadgeKind`](ui/type-BadgeKind.md) | enum |  | What a count badge shows. |
| [`CountBadgeDesc`](ui/type-CountBadgeDesc.md) | desc struct |  | A count badge on the corner of the container it is built in (a tab, an inventory icon, a mailbox button). |
| [`og.ui.countBadge`](ui/fn-countBadge.md) | function | `(id: string, desc?: CountBadgeDesc): void` | A count badge: a small pill on a corner of the open container, outset by half its height so it straddles the edge, with a ring in the ground colour that cuts it from what it sits on. |
| [`TooltipStat`](ui/type-TooltipStat.md) | desc struct |  | A row of a tooltip's stats, with an optional comparison. |
| [`TooltipLink`](ui/type-TooltipLink.md) | desc struct |  | What a keyword link in a tooltip's body opens: a nested tooltip of its own. |
| [`TooltipCardDesc`](ui/type-TooltipCardDesc.md) | desc struct |  | A rich tooltip card beside its trigger. |
| [`TooltipCardResult`](ui/type-TooltipCardResult.md) | struct |  | What a tooltip card did this tick. |
| [`og.ui.tooltipCard`](ui/fn-tooltipCard.md) | function | `(id: string, desc?: TooltipCardDesc): TooltipCardResult` | A rich tooltip: a card in the tooltip layer beside its trigger, flipped and clamped inside the safe area, with a title in its rarity's colour and pips, an art slot, rich-text body whose keyword links… |
| [`ProgressStyle`](ui/type-ProgressStyle.md) | enum |  | How a progress bar shows its value. |
| [`ProgressBarDesc`](ui/type-ProgressBarDesc.md) | desc struct |  | A progress bar. |
| [`og.ui.progressBar`](ui/fn-progressBar.md) | function | `(id: string, desc?: ProgressBarDesc): void` | A progress bar: a rounded track, a fill that moves to its value on the style's ui spring (never jumping back on a retarget), a buffer ahead of it, a sweeping sheen while it works, segments for steps,… |
| [`ProgressRingDesc`](ui/type-ProgressRingDesc.md) | desc struct |  | A progress ring or spinner. |
| [`og.ui.progressRing`](ui/fn-progressRing.md) | function | `(id: string, desc?: ProgressRingDesc): void` | A progress ring: an arc filled clockwise from the top on the style's ui spring, or a spinning arc that grows and shrinks as it turns (Material's indeterminate ring, which reads as working at any size… |
| [`SkeletonShape`](ui/type-SkeletonShape.md) | enum |  | What a skeleton stands in for. |
| [`SkeletonGroupDesc`](ui/type-SkeletonGroupDesc.md) | desc struct |  | A skeleton standing in for content that is loading. |
| [`og.ui.skeletonGroup`](ui/fn-skeletonGroup.md) | function | `(id: string, desc?: SkeletonGroupDesc): void` | A skeleton: shapes of the neutral fill where content will arrive, swept by one sheen across the whole group (not each piece on its own clock), cornered as the parts they stand for. |
| [`Notation`](ui/type-Notation.md) | enum |  | How a number is written. |
| [`NumberFormatDesc`](ui/type-NumberFormatDesc.md) | desc struct |  | How og.ui.formatNumber and the counters write a number. |
| [`og.ui.formatNumber`](ui/fn-formatNumber.md) | function | `(value: number, format?: NumberFormatDesc): string` | A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. |
| [`NumberCounterDesc`](ui/type-NumberCounterDesc.md) | desc struct |  | A big number that counts to its value. |
| [`og.ui.numberCounter`](ui/fn-numberCounter.md) | function | `(id: string, value: number, desc?: NumberCounterDesc): number` | A number display for scores, currency and resources: written in its notation (idle letters past trillions, numbers past 1e308), counting to a new value on an ease-out, its digits rolling like an odom… |
| [`BannerKind`](ui/type-BannerKind.md) | enum |  | What a banner announces, which sets its look and timing. |
| [`BannerDesc`](ui/type-BannerDesc.md) | desc struct |  | A full-width announcement. |
| [`BannerResult`](ui/type-BannerResult.md) | struct |  | Where a banner is. |
| [`og.ui.banner`](ui/fn-banner.md) | function | `(id: string, desc?: BannerDesc): BannerResult` | A full-width announcement: a wave or round band that slams in with a squash and a shake, a title card that fades up over a rule that draws out from its centre, a victory band with a burst and confett… |
| [`UiRibbonDesc`](ui/type-UiRibbonDesc.md) | desc struct |  | A ribbon on a card or a result. |
| [`og.ui.ribbon`](ui/fn-ribbon.md) | function | `(id: string, desc?: UiRibbonDesc): void` | A ribbon: a diagonal band wrapping a corner of the open container with its folded tail, a notched banner across its top or bottom, or a stamp slapped across its middle (a New best!: it lands from lar… |
| [`CelebrationKind`](ui/type-CelebrationKind.md) | enum |  | Which celebration plays. |
| [`CelebrationDesc`](ui/type-CelebrationDesc.md) | desc struct |  | A celebration effect. |
| [`og.ui.celebration`](ui/fn-celebration.md) | function | `(id: string, desc?: CelebrationDesc): void` | A celebration: confetti over the whole screen, a burst from the container it is built in, a level-up flourish centred on the screen, or fireworks. |
| [`StarRatingDesc`](ui/type-StarRatingDesc.md) | desc struct |  | A row of stars earned. |
| [`og.ui.starRating`](ui/fn-starRating.md) | function | `(id: string, desc?: StarRatingDesc): void` | Stars earned: empty sockets, each earned star landing in turn from large with a turn and an overshoot (emphasis spring), a burst of sparks as it lands and a pitch rising star by star; the middle star… |
| [`SubtitleSize`](ui/type-SubtitleSize.md) | enum |  | The subtitle text size (XAG 104: the player chooses). |
| [`SubtitleSettings`](ui/type-SubtitleSettings.md) | desc struct |  | The player's subtitle settings (XAG 104), from a settings screen. |
| [`SubtitleLine`](ui/type-SubtitleLine.md) | desc struct |  | A line of dialogue or a sound caption showing now. |
| [`SubtitlesDesc`](ui/type-SubtitlesDesc.md) | desc struct |  | The subtitles showing now. |
| [`og.ui.subtitles`](ui/fn-subtitles.md) | function | `(id: string, desc?: SubtitlesDesc): void` | Subtitles and captions to the Xbox Accessibility Guidelines' 104: centred at the bottom of the safe area, at most two lines of at most about 42 characters each across 70 % of the width (the box wraps… |
| [`StatBarStyle`](ui/type-StatBarStyle.md) | enum |  | A stat bar's shape. |
| [`StatBarDesc`](ui/type-StatBarDesc.md) | desc struct |  | A health or resource bar. |
| [`og.ui.statBar`](ui/fn-statBar.md) | function | `(id: string, desc?: StatBarDesc): void` | A health or resource bar to the bar of action games: blocks (segment), a ghost chunk that holds after a hit and then drains (Dark Souls, fighting games), a heal preview, an absorb overlay (World of W… |
| [`OrbMeterDesc`](ui/type-OrbMeterDesc.md) | desc struct |  | An orb of health or mana. |
| [`og.ui.orbMeter`](ui/fn-orbMeter.md) | function | `(id: string, desc?: OrbMeterDesc): void` | An orb (Diablo's health and mana globes): a glass sphere filled from the bottom with a liquid whose surface sways, a highlight on the glass, a frame ring, the value in the middle; the level moves on … |
| [`RadialMeterDesc`](ui/type-RadialMeterDesc.md) | desc struct |  | A radial stamina wheel. |
| [`og.ui.radialMeter`](ui/fn-radialMeter.md) | function | `(id: string, desc?: RadialMeterDesc): void` | A radial stamina wheel (Breath of the Wild, Elden Ring's spirit gauge): a ring that drains clockwise, outer rings for extra wheels, warn under a third, red and pulsing when exhausted until it refills… |
| [`AbilitySlotDesc`](ui/type-AbilitySlotDesc.md) | desc struct |  | An ability, item or spell on a bar. |
| [`og.ui.abilitySlot`](ui/fn-abilitySlot.md) | function | `(id: string, desc?: AbilitySlotDesc): Response` | An ability slot as shipped action bars draw it (Diablo IV, League, Destiny 2, Genshin): the art fills the frame edge to edge and stays the dominant element in every state; one centred numeral per sta… |
| [`AbilityBarDesc`](ui/type-AbilityBarDesc.md) | desc struct |  | A bar of ability slots. |
| [`AbilityBarResult`](ui/type-AbilityBarResult.md) | struct |  | What an ability bar answered. |
| [`og.ui.abilityBar`](ui/fn-abilityBar.md) | function | `(id: string, desc?: AbilityBarDesc): AbilityBarResult` | An ability bar: abilitySlot for each slot, in a row (or a column) straight on the scene (a style may dress AbilityBar.frame as a slim bar), evenly spaced; a row with more slots than the HUD band hold… |
| [`StatusEffect`](ui/type-StatusEffect.md) | desc struct |  | A buff or debuff on a unit. |
| [`StatusGridDesc`](ui/type-StatusGridDesc.md) | desc struct |  | A grid of buffs and debuffs. |
| [`StatusGridResult`](ui/type-StatusGridResult.md) | struct |  | What a status grid answered. |
| [`og.ui.statusGrid`](ui/fn-statusGrid.md) | function | `(id: string, desc?: StatusGridDesc): StatusGridResult` | Buffs and debuffs to the bar of MMOs: icons in rows, a sweep darkening each as it runs out, the time left under it (12m, 45, 4.2), blinking in its last five seconds, stacks at its corner, debuffs in … |
| [`UnitClass`](ui/type-UnitClass.md) | enum |  | A unit's standing, which dresses its frame. |
| [`MarkerKind`](ui/type-MarkerKind.md) | enum |  | What a map marker is. |
| [`MapMarkerItem`](ui/type-MapMarkerItem.md) | desc struct |  | A marker on a minimap. |
| [`MapShape`](ui/type-MapShape.md) | enum |  | A minimap's outline. |
| [`MinimapViewDesc`](ui/type-MinimapViewDesc.md) | desc struct |  | A minimap. |
| [`MinimapResult`](ui/type-MinimapResult.md) | struct |  | What a minimap answered. |
| [`og.ui.minimapView`](ui/fn-minimapView.md) | function | `(id: string, desc?: MinimapViewDesc): MinimapResult` | A minimap to the bar of MOBAs, shooters and open worlds: the map in a round or square frame, centred on the player and zoomed, fixed with north up and the player's arrow turning or turning with the p… |
| [`CompassPoi`](ui/type-CompassPoi.md) | desc struct |  | A point on a compass strip. |
| [`CompassDesc`](ui/type-CompassDesc.md) | desc struct |  | A compass strip. |
| [`og.ui.compass`](ui/fn-compass.md) | function | `(id: string, desc?: CompassDesc): void` | A compass strip to the bar of Skyrim and Apex: degree ticks every 15° and the bearing under a centre notch, the cardinal and intercardinal letters, points of interest at their bearings with their dis… |
| [`ObjectiveStep`](ui/type-ObjectiveStep.md) | desc struct |  | A step of an objective. |
| [`ObjectiveQuest`](ui/type-ObjectiveQuest.md) | desc struct |  | An objective in the tracker. |
| [`ObjectiveTrackerDesc`](ui/type-ObjectiveTrackerDesc.md) | desc struct |  | The objective tracker. |
| [`og.ui.objectiveTracker`](ui/fn-objectiveTracker.md) | function | `(id: string, desc?: ObjectiveTrackerDesc): void` | An objective tracker to the bar of open worlds and MMOs: each objective's title by its icon with its distance and a countdown, its steps as boxes that tick with a pop when done (dimmed after), counts… |
| [`TimerMode`](ui/type-TimerMode.md) | enum |  | Which way a timer runs. |
| [`HudTimerDesc`](ui/type-HudTimerDesc.md) | desc struct |  | A round, match or turn timer. |
| [`og.ui.hudTimer`](ui/fn-hudTimer.md) | function | `(id: string, desc?: HudTimerDesc): void` | A timer to the bar of fighting and battle royale HUDs: the time in tabular digits on the HUD chip, a ring for a countdown's share left, warn and then danger with a pulse and a tick sound each second … |
| [`Reaction`](ui/type-Reaction.md) | enum |  | How a unit stands toward the player, which colours its tag. |
| [`NameplateDesc`](ui/type-NameplateDesc.md) | desc struct |  | A nameplate over a unit. |
| [`og.ui.nameplate`](ui/fn-nameplate.md) | function | `(id: string, desc?: NameplateDesc): AnchorInfo` | A nameplate to the bar of World of Warcraft and Final Fantasy XIV: the name in its reaction's colour with a level badge and an icon, a title under it, a slim health bar with its absorb and a cast bar… |
| [`OverheadBar`](ui/type-OverheadBar.md) | desc struct |  | One overhead bar. |
| [`OverheadBarsDesc`](ui/type-OverheadBarsDesc.md) | desc struct |  | Overhead bars over many units. |
| [`og.ui.overheadBars`](ui/fn-overheadBars.md) | function | `(id: string, desc?: OverheadBarsDesc): void` | Overhead bars to the bar of RTS and MOBA crowds (StarCraft II, League of Legends): one mass element per unit, thousands a frame, each a dark track with its fill in its reaction's colour and a pale gh… |
| [`DamageKind`](ui/type-DamageKind.md) | enum |  | What a damage number is. |
| [`DamageHitDesc`](ui/type-DamageHitDesc.md) | desc struct |  | A damage number to throw. |
| [`DamageNumbersDesc`](ui/type-DamageNumbersDesc.md) | desc struct |  | Damage numbers. |
| [`og.ui.damageNumbers`](ui/fn-damageNumbers.md) | function | `(id: string, desc?: DamageNumbersDesc): void` | Damage numbers to the bar of Diablo IV, Genshin Impact and World of Warcraft: each hit thrown from where it landed, rising and drifting to its side as it fades, outlined so it reads on any scene; a c… |
| [`InteractionPromptDesc`](ui/type-InteractionPromptDesc.md) | desc struct |  | An interaction prompt over a thing. |
| [`og.ui.interactionPrompt`](ui/fn-interactionPrompt.md) | function | `(world: World, id: string, desc?: InteractionPromptDesc): AnchorInfo` | An interaction prompt to the bar of The Last of Us, Elden Ring and Genshin Impact: the action's glyph for the device used last (a key cap, a pad's face button) and the verb, the thing's name over it,… |
| [`WaypointKind`](ui/type-WaypointKind.md) | enum |  | What a waypoint marks. |
| [`WaypointDesc`](ui/type-WaypointDesc.md) | desc struct |  | A waypoint or a ping in the world. |
| [`og.ui.waypoint`](ui/fn-waypoint.md) | function | `(id: string, desc?: WaypointDesc): AnchorInfo` | A waypoint or ping to the bar of Apex Legends and Destiny: a diamond with its kind's icon in its colour, the distance under it, a label and who pinged it; off screen or behind it stays on the safe ar… |
| [`GroundDesc`](ui/type-GroundDesc.md) | desc struct |  | What a ground shape (a selection ring, a telegraph, a path line, a placement ghost) lies on: its outline is cut into vertices `spacing` apart and each is lifted to the surface under it, so it follows… |
| [`SelectionRingDesc`](ui/type-SelectionRingDesc.md) | desc struct |  | A selection ring on the ground. |
| [`og.ui.selectionRing`](ui/fn-selectionRing.md) | function | `(world: World, id: string, desc?: SelectionRingDesc): void` | A selection ring to the bar of StarCraft II and Baldur's Gate 3: an ellipse on the ground under the unit (the ground circle projected, lying on the terrain or colliders under it with desc.ground), a … |
| [`TelegraphShape`](ui/type-TelegraphShape.md) | enum |  | A telegraph's area. |
| [`TelegraphDesc`](ui/type-TelegraphDesc.md) | desc struct |  | An area an attack will hit. |
| [`og.ui.telegraph`](ui/fn-telegraph.md) | function | `(world: World, id: string, desc?: TelegraphDesc): void` | An attack telegraph to the bar of Final Fantasy XIV and WildStar: the area on the ground (disc, ring, cone or line; with desc.ground over slopes and hills, every vertex on the surface) tinted in dang… |
| [`PathLineDesc`](ui/type-PathLineDesc.md) | desc struct |  | A path on the ground. |
| [`og.ui.pathLine`](ui/fn-pathLine.md) | function | `(world: World, id: string, desc?: PathLineDesc): void` | A path line to the bar of Baldur's Gate 3, XCOM 2 and RTS move orders: the route on the ground (draped on the surface with desc.ground) as a line with chevrons marching toward its end, the part past … |
| [`PlacementGhostDesc`](ui/type-PlacementGhostDesc.md) | desc struct |  | A building's footprint while it is placed. |
| [`og.ui.placementGhost`](ui/fn-placementGhost.md) | function | `(world: World, id: string, desc?: PlacementGhostDesc): void` | A placement ghost to the bar of Age of Empires IV, Anno and Cities: Skylines: the footprint on the ground (draped on the surface with desc.ground) in success or danger (hatched when invalid, so it is… |
| [`LootLabelDesc`](ui/type-LootLabelDesc.md) | desc struct |  | A label on an item lying in the world. |
| [`LootLabelResult`](ui/type-LootLabelResult.md) | struct |  | What a loot label answered. |
| [`og.ui.lootLabel`](ui/fn-lootLabel.md) | function | `(id: string, desc?: LootLabelDesc): LootLabelResult` | A loot label to the bar of Diablo IV, Path of Exile and Borderlands: the name on a dark chip edged in its rarity's colour with pips so tiers do not rest on colour alone, the stack's count, a beam of … |
| [`SpeechBubbleDesc`](ui/type-SpeechBubbleDesc.md) | desc struct |  | A speech bubble over a speaker. |
| [`og.ui.speechBubble`](ui/fn-speechBubble.md) | function | `(id: string, desc?: SpeechBubbleDesc): AnchorInfo` | A speech bubble to the bar of Animal Crossing, Sea of Thieves and The Sims: a rounded bubble over the speaker with a tail pointing down to them, the speaker's name, the text typing out with a soft ti… |
| [`VectorArt`](ui/type-VectorArt.md) | handle |  | A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill. |
| [`UiMaterial`](ui/type-UiMaterial.md) | handle |  | A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through. |
| [`VectorArt()`](#column-VectorArt) | column kind |  | A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill. |
| [`UiMaterial()`](#column-UiMaterial) | column kind |  | A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint. |
| [`ShapeType`](ui/type-ShapeType.md) | enum |  | An element's outline (DrawShape.kind). |
| [`CornerShape`](ui/type-CornerShape.md) | enum |  | How a rect's corners turn. |
| [`LineCap`](ui/type-LineCap.md) | enum |  | The end of an open line, an arc or a dash. |
| [`LineJoin`](ui/type-LineJoin.md) | enum |  | Where a line's segments meet. |
| [`FillRule`](ui/type-FillRule.md) | enum |  | Which points a filled path (DrawShape.filled) covers where its contours overlap or cross. |
| [`DrawShape`](ui/type-DrawShape.md) | desc struct |  | An element's outline; its fills, strokes, effects and clip take it. |
| [`GradientType`](ui/type-GradientType.md) | enum |  | How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right). |
| [`GradientSpread`](ui/type-GradientSpread.md) | enum |  | What a gradient does past its ends. |
| [`GradientStop`](ui/type-GradientStop.md) | desc struct |  | A gradient's colour at an offset. |
| [`Gradient`](ui/type-Gradient.md) | desc struct |  | A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands. |
| [`ImageFit`](ui/type-ImageFit.md) | enum |  | How an image fills the element's box. |
| [`SliceEdges`](ui/type-SliceEdges.md) | enum |  | How a nine-slice's edges and centre fill their length. |
| [`SolidFill`](ui/type-SolidFill.md) | desc struct |  | A colour. |
| [`ImageFill`](ui/type-ImageFill.md) | desc struct |  | A texture region with a fit. |
| [`VectorFill`](ui/type-VectorFill.md) | desc struct |  | A cooked vector contained and centred in the box. |
| [`Fill`](ui/type-Fill.md) | union |  | One layer of an element's fill, or a stroke's paint. |
| [`StrokeAlign`](ui/type-StrokeAlign.md) | enum |  | Which side of the outline a stroke lies on. |
| [`Stroke`](ui/type-Stroke.md) | desc struct |  | A stroke along the element's outline with a paint of its own (a gradient border is one stroke). |
| [`ShadowEffect`](ui/type-ShadowEffect.md) | desc struct |  | A shadow: a drop shadow under the element, or an inner shadow cast inwards. |
| [`GlowEffect`](ui/type-GlowEffect.md) | desc struct |  | A glow: around the element, or along its inside. |
| [`BlurEffect`](ui/type-BlurEffect.md) | desc struct |  | The element and its subtree blurred (drawn offscreen). |
| [`BackdropEffect`](ui/type-BackdropEffect.md) | desc struct |  | What is under the element (the scene and the UI below it) blurred, its saturation and brightness scaled and tinted: glass. |
| [`GlassWeight`](ui/type-GlassWeight.md) | enum |  | How heavy a glass surface is: flat glass, clear and coloured by what passes under it, with no rim, sheen or refraction. |
| [`Elevation`](ui/type-Elevation.md) | enum |  | The soft shadow a glass surface casts. |
| [`Effect`](ui/type-Effect.md) | union |  | An effect of the element. |
| [`ClipMode`](ui/type-ClipMode.md) | enum |  | How an element cuts its children. |
| [`MaskDesc`](ui/type-MaskDesc.md) | desc struct |  | An alpha mask over an element's subtree: a gradient's alpha, else an image's. |
| [`MaterialName`](ui/type-MaterialName.md) | enum |  | What a layer composites through: a built-in UI material, or a custom one. |
| [`LayerDesc`](ui/type-LayerDesc.md) | desc struct |  | The element's subtree drawn offscreen, then composited through a material (a card that dissolves, a hologram panel). |
| [`TextShadowDesc`](ui/type-TextShadowDesc.md) | desc struct |  | A shadow under an element's text. |
| [`TextOutlineDesc`](ui/type-TextOutlineDesc.md) | desc struct |  | An outline around an element's glyphs. |
| [`TextEffectsDesc`](ui/type-TextEffectsDesc.md) | desc struct |  | Effects of the text an element holds (its labels and rich text). |
| [`DrawDesc`](ui/type-DrawDesc.md) | desc struct |  | What an element draws with (og.ui.setDraw), over its background: its shape, fills and strokes in order, effects, opacity and blend, how it clips its children, and whether its subtree draws offscreen … |
| [`BlendMode`](ui/type-BlendMode.md) | enum |  | How a paint mixes with what is under it. |
| [`SemanticsDesc`](ui/type-SemanticsDesc.md) | desc struct |  | What a component tells the accessibility tree about an element it built from containers (docs/ui.md §2): a role other than its kind's, a name, a value and states. |
| [`og.ui.semantics`](ui/fn-semantics.md) | function | `(desc?: SemanticsDesc): boolean` | Give the element built just before (a container just closed, or a widget) a role, a name, a value and states for the accessibility tree and the probe's dump (dump_ui's role, name, value, states). |
| [`og.ui.partAttr`](ui/fn-partAttr.md) | function | `(key: string, value: string): boolean` | Gives the element open now (a grid's cell, a panel, a card) a selector attribute its style dresses by: partAttr("rarity", "epic") in a bag's cell makes it `Slot.frame[rarity=epic]`, so a style that d… |
| [`og.ui.setDraw`](ui/fn-setDraw.md) | function | `(desc?: DrawDesc): void` | Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glow… |
| [`WorldTarget`](ui/type-WorldTarget.md) | desc struct |  | What an anchored element or a world panel is placed on: an entity (its world pose, down og.Parent) or a world point. |
| [`AnchorScaleMode`](ui/type-AnchorScaleMode.md) | enum |  | How an anchored element's size follows its distance from the camera. |
| [`WorldOcclusion`](ui/type-WorldOcclusion.md) | enum |  | What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU. |
| [`Declutter`](ui/type-Declutter.md) | enum |  | How an anchored element that overlaps one of higher priority (or as high and nearer) resolves. |
| [`EdgeClampDesc`](ui/type-EdgeClampDesc.md) | desc struct |  | Edge clamping for a waypoint or a ping: off the screen or behind the camera it stays inside the safe area, on the edge toward its point, with an arrow pointing there. |
| [`WorldAnchorDesc`](ui/type-WorldAnchorDesc.md) | desc struct |  | An element anchored in the world (docs/ui.md §1): laid out at its own size and placed each tick on its target's point projected with the tick's camera. |
| [`AnchorInfo`](ui/type-AnchorInfo.md) | struct |  | Where an anchored element stands: its distance and level of detail from this tick's camera, the rest as last tick placed it. |
| [`og.ui.openWorldAnchor`](ui/fn-openWorldAnchor.md) | function | `(id: string, desc?: WorldAnchorDesc): AnchorInfo` | Open an element anchored in the world, closed by close(): its children are an ordinary subtree (labels, bars, icons, buttons) laid out at its own size, then projected, scaled, depth tested, declutter… |
| [`WorldPanelDesc`](ui/type-WorldPanelDesc.md) | desc struct |  | A world panel (docs/ui.md §1): a UI tree drawn into a render target that a mesh shows (a terminal, a cockpit screen, a shop sign), reached by the pointer through a ray onto its quad. |
| [`PanelInfo`](ui/type-PanelInfo.md) | struct |  | What the pointer does on a world panel this tick. |
| [`og.ui.openWorldPanel`](ui/fn-openWorldPanel.md) | function | `(id: string, desc?: WorldPanelDesc): PanelInfo` | Open a world panel, closed by close(): its children are an ordinary UI (buttons, lists, fields) laid out at width by height and drawn into desc.target; a press on its quad reaches its controls. |
| [`MassKind`](ui/type-MassKind.md) | enum |  | What a mass element template draws. |
| [`MassEase`](ui/type-MassEase.md) | enum |  | How a mass element's timeline moves into a key. |
| [`MassSpace`](ui/type-MassSpace.md) | enum |  | Where a mass element's position is. |
| [`ParticleShape`](ui/type-ParticleShape.md) | enum |  | A particle's shape. |
| [`MassKey`](ui/type-MassKey.md) | desc struct |  | A key of a mass element's timeline, evaluated in the shader. |
| [`MassTemplateDesc`](ui/type-MassTemplateDesc.md) | desc struct |  | A mass element template (docs/ui.md §1): declared once, then instanced thousands of times a frame with a few values each. |
| [`MassInstance`](ui/type-MassInstance.md) | desc struct |  | One mass element: a few values; its template does the rest. |
| [`og.ui.defineMass`](ui/fn-defineMass.md) | function | `(name: string, desc?: MassTemplateDesc): number` | Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle. |
| [`og.ui.spawnMass`](ui/fn-spawnMass.md) | function | `(template: number, instances: readonly MassInstance[]): void` | Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). |
| [`og.ui.pushMass`](ui/fn-pushMass.md) | function | `(template: number, instances: readonly MassInstance[]): void` | Show instances of a template for this tick only (overhead bars, nameplates, status icons rebuilt every tick with their values). |
| [`og.ui.massCount`](ui/fn-massCount.md) | function | `(template: number): number` | The live spawned instances of a template. |
| [`EmitFrom`](ui/type-EmitFrom.md) | enum |  | Where on its element a particle starts. |
| [`ParticlesDesc`](ui/type-ParticlesDesc.md) | desc struct |  | UI particles from an element (docs/ui.md §3.3): confetti, sparks, coin streams, rarity bursts. |
| [`og.ui.particles`](ui/fn-particles.md) | function | `(desc?: ParticlesDesc): void` | Emit particles from the element built just before (a control, or a container just opened). |
| [`MetaKind`](ui/type-MetaKind.md) | enum |  | What a meta layer draws over the scene. |
| [`MetaDesc`](ui/type-MetaDesc.md) | desc struct |  | A meta layer (docs/ui.md §1): full screen, over the scene and under the screen's elements, never taking input. |
| [`og.ui.meta`](ui/fn-meta.md) | function | `(id: string, desc?: MetaDesc): void` | A meta layer this tick: a damage vignette, a low-health pulse, a material over the whole frame. |
| [`SceneViewportDesc`](ui/type-SceneViewportDesc.md) | desc struct |  | The viewport element (docs/ui.md §3.3): a camera's view of a scene layer shown in the layout (a character on a turntable, an item to inspect). |
| [`SceneViewportInfo`](ui/type-SceneViewportInfo.md) | struct |  | What the pointer does in a viewport this tick. |
| [`og.ui.sceneViewport`](ui/fn-sceneViewport.md) | function | `(id: string, desc?: SceneViewportDesc): SceneViewportInfo` | A viewport: a camera's render target shown in the layout, its drag, wheel and pointer ray handed back so a menu can orbit, pick and rotate. |
| [`UiDocEvent`](ui/type-UiDocEvent.md) | struct |  | Something a document did (docs/ui.md §2.1): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back i… |
| [`UiDocResponse`](ui/type-UiDocResponse.md) | struct |  | What mounting a document answers: its events this tick (and, for a screen, those of its builds code did not see), its root's path, and for a component its props after its two-way writes. |
| [`UiLayer`](ui/type-UiLayer.md) | struct |  | A layer of the stack (docs/ui.md §2.1), as its screen declares it. |
| [`UiActionTrigger`](ui/type-UiActionTrigger.md) | enum |  | When a UI action fires. |
| [`UiActionDesc`](ui/type-UiActionDesc.md) | struct |  | A named UI action (docs/ui.md §2.1) bound to og-input's controls: a component asks for an action, not a key. |
| [`UiDevice`](ui/type-UiDevice.md) | enum |  | The device a glyph set is for: the one the player used last. |
| [`UiGlyph`](ui/type-UiGlyph.md) | struct |  | An action's control on the device used last. |
| [`UiActionHint`](ui/type-UiActionHint.md) | struct |  | One entry of the action bar: an action the top layer accepts, its hint and glyph. |
| [`og.ui.screen`](ui/fn-screen.md) | function | `<S extends ScreenName>(name: S, vm?: ScreenProps<S> \| null): ScreenResponse<S>` | Mount a screen document (`type screen`, loaded with og.load.document) as a layer of the stack, pushed when it is not on it, with its viewmodel for this tick, and build it here: code fills its slots a… |
| [`og.ui.component`](ui/fn-component.md) | function | `<C extends ComponentName>(id: string, name: C, props?: ComponentProps<C> \| null): ComponentResponse<C>` | Mount a component document (`type component`) here, inside whatever is open: its props are its viewmodel, its two-way writes come back in the answer's props. |
| [`og.ui.openSlot`](ui/fn-openSlot.md) | function | `(path: string): boolean` | Open a document's slot (a `slot` element built this tick) so code fills it; its children follow until og.ui.close. |
| [`og.ui.push`](ui/fn-push.md) | function | `<S extends ScreenName>(world: World, screen: S, vm?: ScreenProps<S> \| null): Result<number>` | Push a screen onto the layer stack (docs/ui.md §2.1), over the layers of its kind and under those of a later kind (hud, menu, modal, popover, toast, tooltip, system); the active layer remembers its f… |
| [`og.ui.pop`](ui/fn-pop.md) | function | `(world: World): string` | Pop the active layer; the next active one takes its remembered focus. |
| [`og.ui.replace`](ui/fn-replace.md) | function | `<S extends ScreenName>(world: World, screen: S, vm?: ScreenProps<S> \| null): Result<number>` | Replace the active layer with a screen. |
| [`og.ui.clear`](ui/fn-clear.md) | function | `(world: World): void` | Empty the layer stack. |
| [`og.ui.layers`](ui/fn-layers.md) | function | `(world: World): readonly UiLayer[]` | The layer stack, bottom first. |
| [`og.ui.screenVm`](ui/fn-screenVm.md) | function | `<S extends ScreenName>(world: World, screen: S): ScreenProps<S> \| null` | A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack. |
| [`og.ui.docEvents`](ui/fn-docEvents.md) | function | `(world: World): readonly UiDocEvent[]` | Take the events of documents built where code did not see them (layers built at the frame's end); og.ui.screen answers its own. |
| [`og.ui.gameInput`](ui/fn-gameInput.md) | function | `(world: World): boolean` | Gameplay takes input this tick: no layer whose input is `menu` is on the stack. |
| [`og.ui.paused`](ui/fn-paused.md) | function | `(world: World): boolean` | A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had. |
| [`og.ui.defineAction`](ui/fn-defineAction.md) | function | `(world: World, desc: UiActionDesc): Result<void>` | Define (or redefine) a UI action. |
| [`og.ui.action`](ui/fn-action.md) | function | `(name: string): boolean` | The action fired this tick (a document's `on action` fires on the top layer that takes input). |
| [`og.ui.actionHeld`](ui/fn-actionHeld.md) | function | `(name: string): boolean` | A hold action is held past its hold, or its toggle is on. |
| [`og.ui.glyph`](ui/fn-glyph.md) | function | `(world: World, action: string): UiGlyph \| null` | An action's glyph on the device used last: its first binding there. |
| [`og.ui.frameGlyph`](ui/fn-frameGlyph.md) | function | `(action: string): UiGlyph \| null` | An action's glyph on the device used last, as og.ui.glyph answers it, without the world: for a component that shows a prompt (a tab row's bumpers, the action bar) and only builds UI. |
| [`og.ui.device`](ui/fn-device.md) | function | `(world: World): UiDevice` | The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer. |
| [`og.ui.setDevice`](ui/fn-setDevice.md) | function | `(world: World, device: UiDevice): void` | Set the device (the host's when it knows: a pad's family from its id); a pad family also becomes what pad input shows from then on. |
| [`og.ui.actionBar`](ui/fn-actionBar.md) | function | `(world: World): readonly UiActionHint[]` | What an action bar component reads: the actions the top layer's document lists (`actions { confirm "Buy" }`), each with its hint and its glyph on the device used last. |
| [`og.ui.documentSchema`](ui/fn-documentSchema.md) | function | `(): unknown` | The reflected schema documents are validated against (docs/ui.md §2.1): every element kind and every prop, with its value type, the kinds it applies to and whether a control writes it back. |
| [`og.ui.setProp`](ui/fn-setProp.md) | function | `(world: World, path: string, prop: string, value: string): Result<void>` | Set a prop of a document element live (Tune, the probe's ui:set): presentation only, over the document's own line until og.ui.saveEdits writes it into the file. |
| [`og.ui.saveEdits`](ui/fn-saveEdits.md) | function | `(world: World): Result<readonly string[]>` | Write the live edits into their documents' `.ogt` files under the project's assets (each edited element's line of that prop replaced, or one added; every other line kept), as save_scene writes scenes… |

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

<sub>Module `ui` line 6898</sub>

```ogt
column UiMaterial {
  doc "A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint."
  array Uint32Array
  stride 2
  value UiMaterial
}
```

<sub>Module `ui` line 6905</sub>
