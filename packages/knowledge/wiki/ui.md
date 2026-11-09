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
| [`TextAlign`](ui/type-TextAlign.md) | enum |  | How the lines of a label align. |
| [`Scheme`](ui/type-Scheme.md) | enum |  | The palette: games run on a stage, so dark is the default. |
| [`DensityMode`](ui/type-DensityMode.md) | enum |  | Control sizes by the input the player holds: auto picks touch sizes on a coarse pointer or a play surface under 768 px, mouse sizes otherwise. |
| [`ThemeColor`](ui/type-ThemeColor.md) | enum |  | A colour role of the UI style in effect (the game's style, or the default style before it picks one). |
| [`TypeStepName`](ui/type-TypeStepName.md) | enum |  | A step of the style's type ramp. |
| [`Insets`](ui/type-Insets.md) | struct |  | Padding or margins on four sides. |
| [`Paint`](ui/type-Paint.md) | desc struct |  | A colour: a theme token, or an sRGB colour as authored (components 0 to 1), with an alpha. |
| [`Viewport`](ui/type-Viewport.md) | struct |  | The play surface the UI lays out in, in og-ui units (CSS px over UiTheme.unit) and device pixels per unit. |
| [`UiDesc`](ui/type-UiDesc.md) | desc struct |  | How the UI looks: the scheme, the density, its fonts, the contrast, the game's primary colour, motion, transparency, the icon atlas, the style and the sounds. |
| [`UiSounds`](ui/type-UiSounds.md) | desc struct |  | Sounds the UI plays through og-audio (the ui bus) on its events; null is silent. |
| [`TypeStep`](ui/type-TypeStep.md) | struct |  | One step of the type ramp. |
| [`ControlSizes`](ui/type-ControlSizes.md) | struct |  | The control, row and header heights and the gutter of the density in effect. |
| [`UiTheme`](ui/type-UiTheme.md) | struct |  | The style values the UI draws with, read-only. |
| [`og.ui.configure`](ui/fn-configure.md) | function | `(world: World, desc?: UiDesc): Result<void>` | Set how the UI looks (the scheme, density, fonts, style, sounds, motion…), typically in GameDesc.start; it takes effect from the next tick. |
| [`og.ui.viewport`](ui/fn-viewport.md) | function | `(world: World): Viewport` | This tick's play surface as the UI lays out on it: the render target's size, safe area and covered region in og-ui units (CSS pixels over UiTheme.unit, docs/ui.md §6.1: 1 on touch and up to 720p, so … |
| [`og.ui.theme`](ui/fn-theme.md) | function | `(world: World): UiTheme` | The style values the UI draws with. |
| [`TextSpan`](ui/type-TextSpan.md) | struct |  | A named attribute range in plain text for a store and the Svelte text spans binding. |
| [`Anchor`](ui/type-Anchor.md) | enum |  | A point of the safe area an anchored group sits at. |
| [`Tone`](ui/type-Tone.md) | enum |  | A status tone of badges, toasts and map markers. |
| [`Breakpoint`](ui/type-Breakpoint.md) | enum |  | The play surface's size class. |
| [`ColorVision`](ui/type-ColorVision.md) | enum |  | A colour vision deficiency (UiDesc.colorVision; CSS @media (-og-color-vision: <name>)). |
| [`ColorVisionFilter`](ui/type-ColorVisionFilter.md) | enum |  | What UiDesc.colorVision does to the frame. |
| [`Contrast`](ui/type-Contrast.md) | enum |  | The theme variant: high contrast remaps muted inks, fills and lines to the style's stronger ones and outlines every control (no new colour). |
| [`NavIntent`](ui/type-NavIntent.md) | enum |  | A focus-navigation intent, as the arrow keys, Tab, Enter, Space, Escape and a gamepad's d-pad, stick, A and B give. |
| [`UiRect`](ui/type-UiRect.md) | struct |  | A rectangle of the play surface. |
| [`IconAtlasDesc`](ui/type-IconAtlasDesc.md) | desc struct |  | The icon atlas: a texture of equal cells in a grid, named in reading order, or an icon set cooked like a font (font). |
| [`UiInlineImage`](ui/type-UiInlineImage.md) | desc struct |  | A picture rich text draws inline by name ([img=<name>]). |
| [`ToastDesc`](ui/type-ToastDesc.md) | desc struct |  | A toast's tone and time. |
| [`UiPointer`](ui/type-UiPointer.md) | struct |  | The primary pointer as the UI reads it this tick. |
| [`og.ui.framePointer`](ui/fn-framePointer.md) | function | `(): UiPointer` | The primary pointer of the frame being built, without the world: for a component that hit-tests a shape it draws itself (a pie's wedges, a wheel's slices) by angle and radius rather than by rectangle… |
| [`og.ui.frameCursor`](ui/fn-frameCursor.md) | function | `(): Cursor` | The pointer's shape the last frame resolved (never auto): the cursor set on what the press holds or the pointer is over, or around it, else its kind's. |
| [`og.ui.frameViewport`](ui/fn-frameViewport.md) | function | `(): Viewport` | The play surface of the frame being built: og.ui.viewport without the world, for code that only builds UI. |
| [`og.ui.hasIcon`](ui/fn-hasIcon.md) | function | `(name: string): boolean` | Whether the icon atlas has an icon of this name (components draw a fallback otherwise). |
| [`og.ui.seconds`](ui/fn-seconds.md) | function | `(): number` | The UI's clock in seconds: frames built over the tick rate (simulation time, never the wall clock). |
| [`og.ui.setFocus`](ui/fn-setFocus.md) | function | `(path: string): void` | Move the focus (the pointer's, the keys' and pad 0's) to the element at a full path when this frame ends, if a focusable control is there then: a component moving the focus with its choice (a radio g… |
| [`og.ui.scrollIntoView`](ui/fn-scrollIntoView.md) | function | `(path: string, center: boolean): void` | Scroll the scroll container around a control (an interactive or focusable element, by its full path) so the control is in its view, by where both were in the last frame: by the least that shows it wh… |
| [`og.ui.frameDirection`](ui/fn-frameDirection.md) | function | `(): string` | The writing direction of the frame being built, `ltr` or `rtl` (og.i18n's locale sets it): code that draws its own arrows, chevrons or progress along a line mirrors them by it; layout mirrors rows an… |
| [`og.ui.frameTheme`](ui/fn-frameTheme.md) | function | `(): UiTheme` | The style values of the frame being built: og.ui.theme without the world, for code that only builds UI (the components and patterns). |
| [`og.ui.bounds`](ui/fn-bounds.md) | function | `(path: string): UiRect \| null` | Where an element was in the last frame, by id or `Parent/Child` path (topmost first), as touch_move_to resolves it. |
| [`ScrollState`](ui/type-ScrollState.md) | struct |  | A scroll container as the last frame laid it out (og.ui.scrollState). |
| [`og.ui.scrollState`](ui/fn-scrollState.md) | function | `(path: string): ScrollState \| null` | A scroll container's offset, view and whole content, by its full path (as the probe's dump names it): what a component virtualises its rows by, since og.ui.bounds is cut to the view and a row scrolle… |
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
| [`og.ui.tick`](ui/fn-tick.md) | function | `(): number` | Frames the UI has built: its clock, for animations by the tick. |
| [`og.ui.previewViewport`](ui/fn-previewViewport.md) | function | `(world: World, viewport: Viewport \| null): void` | Lay the UI out for another play surface (a phone in portrait at 3x, a notch) until cleared with null: for headless previews and playtests of responsive layouts. |
| [`UiProp`](ui/type-UiProp.md) | enum |  | An animatable value of an element (docs/ui.md §3.1): a state, a timeline key, a presence or og.ui.animate moves it. |
| [`Position`](ui/type-Position.md) | enum |  | How an element is placed. |
| [`MotionToken`](ui/type-MotionToken.md) | enum |  | A named motion of the style (docs/ui.md §3.1): components use these, so a style's personality changes every motion at once. |
| [`Repeat`](ui/type-Repeat.md) | enum |  | How a play repeats. |
| [`SpringSpec`](ui/type-SpringSpec.md) | desc struct |  | A spring by how it feels: about how long it takes to settle and how much it overshoots. |
| [`TweenSpec`](ui/type-TweenSpec.md) | desc struct |  | A duration on an ease: a retarget starts a new ease from where it is. |
| [`TokenRef`](ui/type-TokenRef.md) | struct |  | One of the style's motion tokens. |
| [`InstantSpec`](ui/type-InstantSpec.md) | struct |  | At once. |
| [`Transition`](ui/type-Transition.md) | union |  | How a value moves to a new target. |
| [`Cursor`](ui/type-Cursor.md) | enum |  | The pointer's shape over an element (CSS cursor's keywords, CSS UI 4 §5.1, docs/ui.md §2), inherited by its subtree. |
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
| [`og.ui.visualState`](ui/fn-visualState.md) | function | `(path: string, group: string): string` | The state a group of an element is in (last tick); empty for none. |
| [`og.ui.animate`](ui/fn-animate.md) | function | `(id: string, target: number, transition: Transition): number` | Any number on a spring or an ease (docs/ui.md §3.1: every value is animatable): call it every tick with the target and it answers this tick's value. |
| [`og.ui.gesture`](ui/fn-gesture.md) | function | `(path: string): GestureInfo` | What the drag or pinch on an element is doing. |
| [`og.ui.motionEvents`](ui/fn-motionEvents.md) | function | `(): readonly UiMotionEvent[]` | The UI's motion events of the last frame: sounds, haptics, bursts, state changes, callbacks, ends, dismissals, drops and snaps. |
| [`og.ui.overlaps`](ui/fn-overlaps.md) | function | `(): readonly UiOverlap[]` | The last frame's undeclared overlaps between elements (docs/ui.md §2), with both paths. |
| [`og.ui.layoutStats`](ui/fn-layoutStats.md) | function | `(): UiLayoutStats` | What the last frame built, rebuilt, laid out and kept. |
| [`Notation`](ui/type-Notation.md) | enum |  | How a number is written. |
| [`NumberFormatDesc`](ui/type-NumberFormatDesc.md) | desc struct |  | How og.ui.formatNumber and the counters write a number. |
| [`og.ui.formatNumber`](ui/fn-formatNumber.md) | function | `(value: number, format?: NumberFormatDesc): string` | A number written in a notation: grouped, compact, idle letters, scientific or engineering, with a sign, prefix and suffix and the locale's separators, and past 1e308 through its exponent. |
| [`VectorArt`](ui/type-VectorArt.md) | handle |  | A cooked vector (an `.ogt` of `type vector`: an SVG subset) from og.load.vector, drawn by a VectorFill. |
| [`UiMaterial`](ui/type-UiMaterial.md) | handle |  | A custom UI material (an `.ogt` of `type ui-material`) from og.load.uiMaterial, which a LayerDesc composites through. |
| [`VectorArt()`](#column-VectorArt) | column kind |  | A VectorArt handle field (0 is none): a vector og.load.vector answered, kept for a VectorFill. |
| [`UiMaterial()`](#column-UiMaterial) | column kind |  | A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint. |
| [`GradientType`](ui/type-GradientType.md) | enum |  | How a gradient (Gradient.type) runs over the element's box (fractions: 0,0 top left, 1,1 bottom right). |
| [`GradientSpread`](ui/type-GradientSpread.md) | enum |  | What a gradient does past its ends. |
| [`GradientStop`](ui/type-GradientStop.md) | desc struct |  | A gradient's colour at an offset. |
| [`Gradient`](ui/type-Gradient.md) | desc struct |  | A gradient of up to 8 stops, mixed in OKLab (no grey middle between complementary colours) and dithered so a long shallow one shows no bands. |
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

<sub>Module `ui` line 1295</sub>

```ogt
column UiMaterial {
  doc "A UiMaterial handle field (0 is none): a UI material og.load.uiMaterial answered, kept for a LayerDesc or a custom paint."
  array Uint32Array
  stride 2
  value UiMaterial
}
```

<sub>Module `ui` line 1302</sub>
