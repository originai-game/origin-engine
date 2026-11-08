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
| [`ScrollAxes`](ui/type-ScrollAxes.md) | enum |  | Which ways a scroll container scrolls. |
| [`ScrollDesc`](ui/type-ScrollDesc.md) | desc struct |  | A scroll container: the wheel (both axes, deltaX and deltaY) or a drag scrolls it, and a released drag keeps its momentum, slowing over motion.spring.momentum's response (none under reduced motion); … |
| [`ImageDesc`](ui/type-ImageDesc.md) | desc struct |  | A texture region, or a solid rectangle without a texture. |
| [`Response`](ui/type-Response.md) | struct |  | What happened to an element this tick. |
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
| [`og.ui.openScroll`](ui/fn-openScroll.md) | function | `(id: string, desc?: ScrollDesc): void` | Open a vertical scroll container, closed by close(). |
| [`og.ui.close`](ui/fn-close.md) | function | `(): Response` | Close the open container. |
| [`og.ui.row`](ui/fn-row.md) | function | `(id: string, desc: ElementDesc, build: UiBuild): Response` | A row built by `build`: open(id, {...desc, direction: row}), build(), close(). |
| [`og.ui.column`](ui/fn-column.md) | function | `(id: string, desc: ElementDesc, build: UiBuild): Response` | A column built by `build`: open(id, {...desc, direction: column}), build(), close(). |
| [`RichTextDesc`](ui/type-RichTextDesc.md) | desc struct |  | How a rich text paragraph draws: its defaults, which its markup's spans override. |
| [`RichTextResult`](ui/type-RichTextResult.md) | struct |  | What a rich text's links did this tick. |
| [`TextSpan`](ui/type-TextSpan.md) | struct |  | A named attribute range in plain text for a store and the Svelte text spans binding. |
| [`og.ui.richText`](ui/fn-richText.md) | function | `(id: string, markup: string, desc?: RichTextDesc): RichTextResult` | A paragraph of inline spans from markup, for a string-table message (og.i18n.t) so a translation carries its own emphasis: `[b]bold[/b]` (thickened in the distance field as far as it reaches, plus a … |
| [`og.ui.label`](ui/fn-label.md) | function | `(id: string, text: string, desc?: LabelDesc): void` | MSDF text that wraps at its width (word breaks; between CJK characters under kinsoku, katakana words kept whole; long words between characters) unless nowrap. |
| [`og.ui.image`](ui/fn-image.md) | function | `(id: string, desc?: ImageDesc): void` | A texture region, or a solid rectangle; with ImageDesc.aspect, a fitting side follows the other. |
| [`og.ui.idi`](ui/fn-idi.md) | function | `(name: string, index: number): string` | The id of the index-th of several siblings of one name: `idi("Slot", 3)` is `Slot#3`. |
| [`Anchor`](ui/type-Anchor.md) | enum |  | A point of the safe area an anchored group sits at. |
| [`TransitionKind`](ui/type-TransitionKind.md) | enum |  | How a transition moves its content in and out. |
| [`Tone`](ui/type-Tone.md) | enum |  | A status tone of badges, toasts and map markers. |
| [`Breakpoint`](ui/type-Breakpoint.md) | enum |  | The play surface's size class. |
| [`ColorVision`](ui/type-ColorVision.md) | enum |  | A colour vision deficiency (UiDesc.colorVision; CSS @media (-og-color-vision: <name>)). |
| [`ColorVisionFilter`](ui/type-ColorVisionFilter.md) | enum |  | What UiDesc.colorVision does to the frame. |
| [`Contrast`](ui/type-Contrast.md) | enum |  | The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour). |
| [`NavIntent`](ui/type-NavIntent.md) | enum |  | A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give. |
| [`UiRect`](ui/type-UiRect.md) | struct |  | A rectangle of the play surface. |
| [`IconAtlasDesc`](ui/type-IconAtlasDesc.md) | desc struct |  | The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font). |
| [`UiInlineImage`](ui/type-UiInlineImage.md) | desc struct |  | A picture rich text draws inline by name ([img=<name>]). |
| [`AnchorDesc`](ui/type-AnchorDesc.md) | desc struct |  | An anchored group: a layer at a point of the safe area, over the tree (HUD corners, a centred title, a bar of touch controls). |
| [`ModalDesc`](ui/type-ModalDesc.md) | desc struct |  | A dialog: a scrim over everything and a centred panel. |
| [`ModalResult`](ui/type-ModalResult.md) | struct |  | A dialog's answer this tick. |
| [`ToastDesc`](ui/type-ToastDesc.md) | desc struct |  | A toast's tone and time. |
| [`IconDesc`](ui/type-IconDesc.md) | desc struct |  | An icon's size, ink and name. |
| [`UiItemBuild`](ui/type-UiItemBuild.md) | callback |  | Builds one row or cell of a virtualised list or grid: ui calls made inside nest under it. |
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
| [`og.ui.openModal`](ui/fn-openModal.md) | function | `(id: string, open: boolean, desc?: ModalDesc): ModalResult` | Open a dialog, closed by closeModal(); build its content only while the answer is visible. |
| [`og.ui.closeModal`](ui/fn-closeModal.md) | function | `(): void` | Close a dialog opened with openModal (its panel and backdrop). |
| [`og.ui.bounds`](ui/fn-bounds.md) | function | `(path: string): UiRect \| null` | Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it. |
| [`ScrollState`](ui/type-ScrollState.md) | struct |  | A scroll container as the last frame laid it out (og.ui.scrollState). |
| [`og.ui.scrollState`](ui/fn-scrollState.md) | function | `(path: string): ScrollState \| null` | A scroll container's offset, view and whole content, by its full path (og.ui.lastPath after closing it): what a component virtualises its rows by, since og.ui.bounds is cut to the view and a row scro… |
| [`og.ui.divider`](ui/fn-divider.md) | function | `(id: string, vertical: boolean): void` | A hairline in the border colour across its parent. |
| [`og.ui.spacer`](ui/fn-spacer.md) | function | `(id: string, size: Size \| null): void` | Empty space. |
| [`og.ui.icon`](ui/fn-icon.md) | function | `(id: string, name: string, desc?: IconDesc): void` | An icon of the atlas (UiDesc.icons) by name: a texture cell, or an icon set's distance-field glyph fitted to its square; an unknown name draws an empty slot and warns. |
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
| [`og.ui.previewViewport`](ui/fn-previewViewport.md) | function | `(world: World, viewport: Viewport \| null): void` | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. |
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
| [`DataStateDesc`](ui/type-DataStateDesc.md) | desc struct |  | What a data view shows instead of its rows: loading, an error, or nothing. |
| [`og.ui.popover`](ui/fn-popover.md) | function | `(id: string, open: boolean, desc: PopoverDesc, build: UiBuild): PopoverResult` | A popover beside an element with an arrow at it; Back or a press outside dismisses it. |
| [`og.ui.menu`](ui/fn-menu.md) | function | `(id: string, open: boolean, desc?: MenuDesc): MenuChoice` | A menu of commands, checks, radios, separators, headers and submenus, hanging from an element or at a point. |
| [`og.ui.contextMenu`](ui/fn-contextMenu.md) | function | `(id: string, open: boolean, at: Vector2, desc?: MenuDesc): MenuChoice` | A menu at a point: where a right press or a long press happened. |
| [`og.ui.sheet`](ui/fn-sheet.md) | function | `(id: string, open: boolean, desc: SheetDesc, build: UiBuild): SheetResult` | A sheet sliding in from the bottom (resting at detents, dragged) or a side. |
| [`UiProp`](ui/type-UiProp.md) | enum |  | An animatable value of an element (docs/ui.md §3.1): a state, a timeline key, a presence or og.ui.animate moves it. |
| [`LayoutMode`](ui/type-LayoutMode.md) | enum |  | How a container lays out its children. |
| [`Position`](ui/type-Position.md) | enum |  | How an element is placed. |
| [`GridFlow`](ui/type-GridFlow.md) | enum |  | Which way auto-placed items fill a grid. |
| [`DragAxis`](ui/type-DragAxis.md) | enum |  | Which way a drag moves its element. |
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
| [`Cursor`](ui/type-Cursor.md) | enum |  | The pointer's shape over an element (CSS cursor's keywords, CSS UI 4 §5.1, docs/ui.md §2), inherited by its subtree. |
| [`TextPathSpec`](ui/type-TextPathSpec.md) | desc struct |  | An element's text set along a path (CSS -og-text-path, SVG 2 textPath, docs/ui.md §2): each character turned to the path where it sits, its baseline on the path. |
| [`UserSelect`](ui/type-UserSelect.md) | enum |  | Whether the player can select an element's text and copy it (CSS user-select, docs/ui.md §2). |
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
| [`Rarity`](ui/type-Rarity.md) | enum |  | A loot tier (the style's rarity colours, each paired with its pips so no tier is told by colour alone). |
| [`TooltipStat`](ui/type-TooltipStat.md) | desc struct |  | A row of a tooltip's stats, with an optional comparison. |
| [`TooltipLink`](ui/type-TooltipLink.md) | desc struct |  | What a keyword link in a tooltip's body opens: a nested tooltip of its own. |
| [`TooltipCardDesc`](ui/type-TooltipCardDesc.md) | desc struct |  | A rich tooltip card beside its trigger. |
| [`TooltipCardResult`](ui/type-TooltipCardResult.md) | struct |  | What a tooltip card did this tick. |
| [`og.ui.tooltipCard`](ui/fn-tooltipCard.md) | function | `(id: string, desc?: TooltipCardDesc): TooltipCardResult` | A rich tooltip: a card in the tooltip layer beside its trigger, flipped and clamped inside the safe area, with a title in its rarity's colour and pips, an art slot, rich-text body whose keyword links… |
| [`Notation`](ui/type-Notation.md) | enum |  | How a number is written. |
| [`NumberFormatDesc`](ui/type-NumberFormatDesc.md) | desc struct |  | How og.ui.formatNumber and the counters write a number. |
| [`og.ui.formatNumber`](ui/fn-formatNumber.md) | function | `(value: number, format?: NumberFormatDesc): string` | A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. |
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
| [`og.ui.setDraw`](ui/fn-setDraw.md) | function | `(desc?: DrawDesc): void` | Draws the element added last (a container just opened, or the last control) with desc: shapes beyond rectangles, layered gradient, image and vector fills, strokes with their own paints, shadows, glow… |
| [`WorldOcclusion`](ui/type-WorldOcclusion.md) | enum |  | What the scene in front of an anchored element does to it: a depth test of its point against the frame's depth, on the GPU. |
| [`MassKind`](ui/type-MassKind.md) | enum |  | What a mass element template draws. |
| [`MassSpace`](ui/type-MassSpace.md) | enum |  | Where a mass element's position is. |
| [`ParticleShape`](ui/type-ParticleShape.md) | enum |  | A particle's shape. |
| [`MassKey`](ui/type-MassKey.md) | desc struct |  | A key of a mass element's timeline, evaluated in the shader. |
| [`MassTemplateDesc`](ui/type-MassTemplateDesc.md) | desc struct |  | A mass element template (docs/ui.md §1): declared once, then instanced thousands of times a frame with a few values each. |
| [`MassInstance`](ui/type-MassInstance.md) | desc struct |  | One mass element: a few values; its template does the rest. |
| [`og.ui.defineMass`](ui/fn-defineMass.md) | function | `(name: string, desc?: MassTemplateDesc): Result<number>` | Declare a mass element template by name and answer its handle; declaring the name again replaces it and keeps the handle. |
| [`og.ui.spawnMass`](ui/fn-spawnMass.md) | function | `(template: number, instances: readonly MassInstance[]): void` | Start instances of a template on this tick: each plays its template's timeline over its lifetime and leaves after it (damage numbers, pickups). |
| [`og.ui.massCount`](ui/fn-massCount.md) | function | `(template: number): number` | The live spawned instances of a template. |
| [`UiSource`](ui/type-UiSource.md) | interface |  | A store's stable checkpoint identity. |
| [`UiCollectionDesc`](ui/type-UiCollectionDesc.md) | struct |  | A keyed collection's identity rule. |
| [`UiScope`](ui/type-UiScope.md) | interface |  | Named presentation stores for one world, following the svelte/store contract. |
| [`og.ui.scope`](ui/fn-scope.md) | function | `(world: World, id: string): UiScope` | The entry to named svelte/store-compatible presentation data and retained document mounts. |
| [`UiDocEvent`](ui/type-UiDocEvent.md) | struct |  | Something a document did (docs/ui.md §2.1): a click, a changed control, a list row selected, a named event (`emit`, a timeline's callback), an action, a two-way write, or Back on a layer whose Back i… |
| [`UiLayer`](ui/type-UiLayer.md) | struct |  | A layer of the stack (docs/ui.md §2.1), as its screen declares it. |
| [`UiActionTrigger`](ui/type-UiActionTrigger.md) | enum |  | When a UI action fires. |
| [`UiActionDesc`](ui/type-UiActionDesc.md) | struct |  | A named UI action (docs/ui.md §2.1) bound to og-input's controls: a component asks for an action, not a key. |
| [`UiDevice`](ui/type-UiDevice.md) | enum |  | The device a glyph set is for: the one the player used last. |
| [`UiGlyph`](ui/type-UiGlyph.md) | struct |  | An action's control on the device used last. |
| [`UiActionHint`](ui/type-UiActionHint.md) | struct |  | One entry of the action bar: an action the top layer accepts, its hint and glyph. |
| [`og.ui.push`](ui/fn-push.md) | function | `<S extends ScreenName>(world: World, screen: S, vm?: ScreenProps<S> \| null): Result<number>` | Push a screen onto the layer stack (docs/ui.md §2.1), over the layers of its kind and under those of a later kind (hud, menu, modal, popover, toast, tooltip, system); the active layer remembers its f… |
| [`og.ui.pop`](ui/fn-pop.md) | function | `(world: World): string` | Pop the active layer; the next active one takes its remembered focus. |
| [`og.ui.replace`](ui/fn-replace.md) | function | `<S extends ScreenName>(world: World, screen: S, vm?: ScreenProps<S> \| null): Result<number>` | Replace the active layer with a screen. |
| [`og.ui.clear`](ui/fn-clear.md) | function | `(world: World): void` | Empty the layer stack. |
| [`og.ui.layers`](ui/fn-layers.md) | function | `(world: World): readonly UiLayer[]` | The layer stack, bottom first. |
| [`og.ui.screenVm`](ui/fn-screenVm.md) | function | `<S extends ScreenName>(world: World, screen: S): ScreenProps<S> \| null` | A layer's viewmodel as its two-way bindings left it (the top layer of that screen); null when it is not on the stack. |
| [`og.ui.docEvents`](ui/fn-docEvents.md) | function | `(world: World): readonly UiDocEvent[]` | Take the events of the documents built this frame: pushed layers and store-driven mounts. |
| [`og.ui.gameInput`](ui/fn-gameInput.md) | function | `(world: World): boolean` | Gameplay takes input this tick: no layer whose input is `menu` is on the stack. |
| [`og.ui.paused`](ui/fn-paused.md) | function | `(world: World): boolean` | A layer that pauses the game (`pause`) is on the stack: the engine holds the time scale at 0 while one is, and gives back the one it had. |
| [`og.ui.defineAction`](ui/fn-defineAction.md) | function | `(world: World, desc: UiActionDesc): Result<void>` | Define (or redefine) a UI action. |
| [`og.ui.action`](ui/fn-action.md) | function | `(name: string): boolean` | The action fired this tick (a document's `on action` fires on the top layer that takes input). |
| [`og.ui.actionHeld`](ui/fn-actionHeld.md) | function | `(name: string): boolean` | A hold action is held past its hold, or its toggle is on. |
| [`og.ui.glyph`](ui/fn-glyph.md) | function | `(world: World, action: string): UiGlyph \| null` | An action's glyph on the device used last: its first binding there. |
| [`og.ui.frameGlyph`](ui/fn-frameGlyph.md) | function | `(action: string): UiGlyph \| null` | An action's glyph on the device used last, as og.ui.glyph answers it, without the world: for a component that shows a prompt (a tab row's bumpers, the action bar) and only builds UI. |
| [`og.ui.device`](ui/fn-device.md) | function | `(world: World): UiDevice` | The device the player used last (glyphs follow it live): a key or the mouse, a pad of the family set with og.ui.setDevice (Xbox by default), or a touch on a coarse pointer. |
| [`og.ui.filePicker`](ui/fn-filePicker.md) | function | `(world: World): boolean` | Whether this host opens a file picker for a document's `<input type="file">` (a capability query, the same on every host that has one): false on the web and natively today, where such an input is ref… |
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

<sub>Module `ui` line 2458</sub>

```ogt
column UiMaterial {
  doc "A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint."
  array Uint32Array
  stride 2
  value UiMaterial
}
```

<sub>Module `ui` line 2465</sub>
