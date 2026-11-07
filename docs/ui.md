# Origin Engine — game UI

The design of og-ui as the engine's game UI infrastructure. It is normative
for `crates/og-ui`, the UI pass of `crates/og-render`, the UI members of the
API (`module ui` in `crates/og-api/api/ui*.ogt`), the style format, the UI kinds
of the art brief and the UI checks of `og`. `docs/design.md` §2.7 points
here.

Goals:

- UI and UX, interaction animation included, are the core of a game's
  picture, held to the level of shipped commercial games.
- Every genre is covered fully (the appendices).
- UI is a base layer for agents: an agent authors it **and adjusts it**, so
  every value is data that can be read, changed live, checked and saved.
- Quality over reach (design.md §1): current hardware, WebGPU only, no
  tiered fallbacks for low-end devices.
- Generated art is first class: screens reserve explicit, specified places
  for generated images.

## 2. Principles

1. **Intent is data; the engine lays it out.** A screen is a document of
   elements, bindings, states and motion. Code fills slots and handles
   events. An agent's adjustment is a small diff of data, applied live.
2. **Every value is addressable.** Every element prop, style token, state
   and timeline key has a path, is in the reflection schema, is readable
   through the probe, writable through Tune and the probe, and saved back
   to its file. Nothing is described twice (AGENTS.md, reflection is the
   single schema).
3. **Every rule is measured.** What a UI designer reviews by eye (overlap,
   overflow, touch size, reachability, contrast, missing feedback,
   expansion) is a finding or a measure with a path, reported by `og ui
   check` and `og test` (§16.5). The engine reports facts; what a game
   should want lives in the skills.
4. **Motion is driven by state and is interruptible.** Widgets declare
   states; the engine animates between them on springs that keep their
   velocity when retargeted. Authored timelines are for sequences.
5. **Art has places.** Every image in a screen is a slot with a spec
   (kind, aspect, focal point, safe area, mask) that the art pipeline reads.
   A placeholder is drawn from the style, never a grey box.
6. **A style is a system, not a skin.** Tokens, a shape language, part
   dressings, a motion personality, a sound and haptic set and its art.
   Any component's parts can be dressed; a part no style dressed is drawn
   from the tokens and still looks finished.
7. **One element model everywhere.** The same documents draw on the
   screen, anchored to an entity, on a panel in the world and over a 3D
   viewport.
8. **Deterministic, fast and measured.** UI state stays tick-driven and in
   the snapshot (`Ui::save_state`: animations, the toast queue, focus,
   scroll, carets). Motion is counted in ticks. §15.1 sets the budgets.

## 3. Coverage

Every row of the inventories in the appendices (screens B, HUD and world
elements C, interactions D, motion E, art slots F) maps to the engine
capability that makes it possible (a section of this design), a component
or screen document that implements it (§11, §17), and a test that renders
it. Appendix A lists the genres with the capabilities each depends on; a
genre is covered when every row it is tagged with is covered.

## 4. Architecture

| Part | Owner | Section |
|---|---|---|
| UI render pass: shapes, paints, effects, masks, layers, materials | og-render (`ui` module), og-ui `draw.rs` | §5 |
| Element model and layout | og-ui `ui.rs`, `layout.rs` | §6 |
| Motion: springs, states, timelines, choreography, gestures | og-ui `motion/` | §7 |
| Layers, input routing, actions and glyphs | og-ui `layers.rs`, `nav.rs`, og-input | §8 |
| Data binding and formatting | og-ui `bind.rs`, og-i18n | §9 |
| Documents: `type screen`, `type component` | og-ui `doc/`, og-cook | §10 |
| Component library | og-ui (Rust core widgets), `packages/engine/src/ui/` (documents), `packages/engine/extensions/` (kits) | §11, §17 |
| Styles v2 and art slots | og-ui `style_file.rs`, og-cook, knowledge `brief.ts` | §12, §13 |
| World-space and 3D UI | og-ui `world.rs`, og-render | §14 |
| Agent tooling: dump, write, Tune, preview, checks | og-probe, og-cli, `packages/engine/src/node` | §16 |

Text, audio and haptics, and accessibility cut across these parts (§5.6,
§7.8, §15.2).

## 5. Rendering (og-render's UI pass)

The UI pass draws analytic primitives, instanced by one family of
pipelines, after post effects. Shapes are signed distance functions
evaluated per pixel, so edges are exact at any scale and transform.

### 5.1 Shapes

- Rectangle with four corner radii and a corner shape per element: round,
  **squircle** (continuous corners, superellipse n configurable), bevel,
  notch (inset), so a sci-fi panel's cut corners are geometry, not art.
- Circle, ellipse, capsule, ring and arc segment (start, sweep, thickness,
  round or butt caps): cooldown sweeps, radial menus, gauges.
- Regular polygon and star (sides, inner ratio, corner radius).
- Line and polyline with caps, joins and dashes; Bézier curves.
- Vector paths: a `type vector` asset (SVG subset: paths, fills, strokes,
  gradients, groups) cooked to triangles with analytic antialiasing on
  their edges, drawn with the same paints. Icons keep the MTSDF icon sets.
- Run-time vector paths (og-render's `UiGeometry::Path`, a `UiPath` of
  moves, lines, quadratic and cubic Béziers and closes, nonzero or
  even-odd), nothing cooked, so a path can change every frame: charts,
  morphs (`UiPath::lerp` between paths of the same verbs) and imported
  animation. A path is a shape like any other: its signed distance per
  pixel, from its curves as quadratics sorted into bands (Slug's banding
  and root eligibility, Inigo Quilez's distance to a quadratic;
  `third_party/vector-paths/`), so it takes every paint and mode. Strokes
  of a path are a band around its outline (round where it turns); a line
  with joins, caps and dashes is `UiPath::flatten` into polylines.

### 5.2 Paints

Each element has a list of fills and a list of strokes, drawn in order:

- solid; linear, radial, conic and diamond gradients, up to 8 stops,
  interpolated in OKLab, with a dither to stop banding;
- image (a texture region) with fit `cover`, `contain`, `fill`, `none` and a
  focal point, tint, and nine-slice with `stretch`, `repeat`, `round` edges;
- pattern (a tiled region with scale, rotation and offset, which can
  scroll by the tick);
- a stroke has a width, an alignment (inside, centre, outside), dashes, and
  its own paint, so a gradient border is one stroke.

Every colour in a paint is a token reference or a literal; a token is
resolved through the style at draw time, so a style change restyles the
paints.

### 5.3 Effects

Per element, in order: drop shadows (several, each with offset, blur,
spread, colour), inner shadow, outer and inner glow, layer blur, and
**backdrop** effects (blur, saturation, brightness). Opacity, and blend
modes (normal, multiply, screen, add, overlay, soft light, colour dodge)
per element against what is under it.

**Glass** is a backdrop with a material (og-render's `UiGlass`; zero
values are the plain frosted backdrop): flat glass, clear and coloured by
what passes under it, with no specular rim, sheen or refraction. Its parts
are a light tint over a modest blur with raised saturation, a hairline just
inside the edge (CSS `inset 0 0 0 0.5px`), up to two soft shadows outside
the shape only, and CSS mixing (tint, hairline and shadows over
sRGB-encoded values), all drawn by the backdrop's one instance. `solid`
draws the tint opaque and reads nothing under it: a frame whose only glass
is solid builds no blur pyramid. Three weights are presets
(`UiGlassStyle`): `thin` (10 px of CSS blur, the lightest tint: small
controls over art), `regular` (22 px: bars, capsules) and `thick` (32 px
and a denser tint: menus, popovers, dialogs, sheets, toasts), each with an
elevation (`raised` the hairline alone, `float` one short shadow, `lift`
two deeper ones), 200 % saturation, in the light or dark scheme. Under
reduced transparency glass is the solid raised surface with its hairline
and shadow; under high contrast solid with a strong 1 px edge. The scrim
(`ui_scrim`) dims and blurs (8 px, 120 % saturation), and under either
fallback dims only. Controls on glass are flat fills, never a second glass
or a border. A style reaches it through `effect backdrop … glass <weight>
[elevation <e>] [edge <w> [<colour>]] [solid] [css]` and the IDL's
`BackdropEffect` (`glass`, `elevation`, `edge`, `edgeColor`, `solid`,
`cssMix`); og-ui's glass primitive (`DrawCmd::Glass`) carries a
saturation and the material.

The weights that carry text, `regular` and `thick`, hold the scheme's `fg`
at 4.5:1 or more over bright, dark and busy backdrops in either scheme
(for that the dark `regular` tint is 64 %). `glass-thin` carries **icons
and play discs only, never text**.

The UI pass blends flat fills in linear light, not over encoded values as
CSS does. A translucent fill specified for CSS reads stronger here (white
at 13 % in CSS draws like about 5 % here over dark glass), and a black dim
at alpha `a` over bright art reads far lighter than `1 - a` (90 % reads as
a 55–65 % darkening, 75 % as about half). State translucent fills and dims
for the engine's blending, and measure a dim on bright art.

### 5.4 Masks and layers

- Any element can clip its children to its own shape (rounded, squircle,
  polygon, path), antialiased, not only to its rectangle.
- Alpha masks from an image or a gradient: a list fading at its scroll
  edges, a portrait in a brush-stroke shape. A mask's alpha fades as CSS's
  does, not by linear coverage: the composite restates it for the
  engine's linear-light blending against the content's opposite ground
  (white under a dark scrim, black under light text), so a dark scrim
  darkens evenly through its fade.
- An element marked `layer` draws its subtree into an offscreen target
  that is then composited with a **UI material**. Built-in materials:
  dissolve (noise threshold with an edge glow), shine sweep, hologram
  (scanlines, flicker, chromatic split), glitch, ripple, pixelate,
  colour adjust (hue, saturation, brightness, grayscale), outline of the
  alpha, and blur. All take their time from the UI clock.
- Custom materials: a `type ui-material` asset is a WGSL function of uv,
  the layer's texture, time and declared parameters, validated by naga to
  MSL, SPIR-V and HLSL at cook like every other shader. Its parameters are
  reflected, so they animate and tune like any prop. Its `source` is read
  relative to the material file first, then from `assets/`.
- Material presets: the `materials` kit (`og ui add materials`,
  `packages/engine/extensions/materials`) ships foil, holo, burn and rarity
  glow as `type ui-material` assets with tunable parameters; a style
  dresses a part with one (`material "ui-kits/materials/foil.ogt" 0.7`) or
  an element's layer composites through it. Each preset's parameters are
  in its `.ogt`.

### 5.5 Particles, video and 3D in the UI

- UI particle emitters in UI space (confetti, sparks, coin streams, rarity
  bursts), using og-render's GPU particles in a 2D overlay mode, attached
  to an element and clipped by its layer. `burst` is a preset.
- Video textures in image paints.
- The **viewport** element shows a camera's view of a scene layer (a
  character on a turntable, an item to inspect, a car in a garage), with
  its own lights and environment, sized by layout, and passes input
  through: a press or drag inside it becomes a ray into that scene, so a
  game can orbit, pick and rotate inside a menu.

### 5.6 Text effects

The text stack: MSDF text, bidi, shaping, kerning, font per run, run-time
CJK glyphs, the editor with IME, rich text. On top of it: outline
(outside, inside, centred, by the distance field, up to the font's reach),
gradient fill, several shadows, per-glyph timelines for typewriter, wave,
bounce, shake and reveal, auto-fit (shrink to fit down to a minimum type
step, then ellipsis: `LabelDesc.fitMin`), and inline images and icons in
rich text with links that open pinnable nested tooltips (§11). Text on a
path or arc is not implemented. Rich text's tags: `[b]`, `[i]` (a slant of
the face), `[u]`, `[s]`, `[outline=w,colour]`, `[color=]`, `[size=]`,
`[font=]`, `[icon=]` (an icon atlas cell), `[img=]` (any texture the game
registers by name, `UiDesc.images`), `[action=]` (the action's device
glyph, resolved every frame, so a prompt follows the pad the player picks
up), `[link=]` and the per-glyph `[wave]`, `[shake]`, `[pulse]`.

`LabelDesc.uppercase` (the document label's `uppercase` prop) cases a label
with the same ICU casing as a type role's `case upper` (§17.4), before it
is measured; the probe keeps the source text. Components and kits use it
instead of `String.toUpperCase`.

### 5.7 Batching

All shapes, paints and effects other than layers and backdrops draw from
one instance buffer through a small set of pipelines keyed by primitive
family. Painter levels order overlap and batches. A layer or a backdrop
starts a new pass only for its region. Budgets in §15.1.

## 6. Element model and layout

### 6.1 Props of every element

- **Box**: size (`fit`, `grow`, `fixed`, `percent`, `aspect`, min and max),
  padding, margin, gap.
- **Position**: in flow, or `absolute` with insets relative to the parent
  (left, top, right, bottom, centre), or `screen` with insets relative to
  the whole surface (full bleed: a screen's art or backdrop under the notch
  and the home bar, while its layer keeps the content in the safe area), or
  anchored (the nine anchors of the safe area, an entity in the world §14,
  or another element's rectangle).
- **Transform**: translate, scale x and y, rotation, skew, and a pivot.
  Transforms apply after layout and are honoured by hit tests, the probe
  and the accessibility tree's bounds.
- **Draw**: fills, strokes, effects, opacity, blend, shape and corner shape,
  clip (none, rect, shape, mask), `layer` and its material.
- **Order**: `z` inside its layer, so a lifted card draws over its
  neighbours without leaving the flow.
- **Interaction**: interactive, focusable, hit slop, cursor, drag source and
  drop target, gesture recognizers (§8.5).

Ids and paths (`A/B#3`) are stable and shared by hit tests, the probe and
the accessibility tree.

### 6.2 Layout

The tree is laid out Clay-style each tick, with:

- **Grid**: tracks (`fr`, fixed, auto, min/max), gaps, spans, named areas,
  auto-flow.
- **Stack**: children overlaid in the same rectangle (a card's art, frame,
  cost gem and name), each aligned by its own insets.
- **Container queries**: a component chooses its layout by its own width,
  not only by the viewport's breakpoint.
- **Breakpoint variants**: a document can override any prop per breakpoint
  (`phonePortrait`, `phoneLandscape`, `tablet`, `desktop`) and per
  orientation, and declare a different arrangement, not only a smaller one.
- **Direction**: layouts mirror under right-to-left locales; insets and
  alignments are `start`/`end`.
- **Baseline alignment** in rows of mixed type sizes.
- **Overlap is a finding unless declared.** Siblings in flow cannot
  overlap; absolutely positioned and anchored elements that overlap another
  element must say so (`overlaps: "<path>"`), or `og ui check` reports them
  with both paths (§16.5). An element is measured where its own and its
  ancestors' declared transforms draw it (a label scaled down to fit a
  corner covers its scaled box, not the line it was laid out on); motion is
  not part of the check.

### 6.3 Invalidation

A subtree whose inputs (props, bound values, size) have not changed since
the last tick keeps its layout and draw commands. The probe reports
elements rebuilt and laid out per tick (§16.1). Re-laying out a changing
subtree only up to its nearest fixed-size ancestor (a layout boundary) is
not implemented.

## 7. Motion

Everything in this section runs by the tick, keeps its state in the UI
state, and is saved with the world.

### 7.1 Animatable props

Every numeric or colour prop is animatable: transform, opacity, size (as a
layout animation, §7.5), corner radii, paint colours and gradient stops,
stroke width and dash offset, effect parameters, material parameters, text
tracking, weight, outline and glyph timelines, scroll offsets.

### 7.2 Springs, the default

A change to an animatable prop moves to its new value on a spring when the
element (or its style) gives the prop a motion. A spring is two numbers,
**duration** (perceptual, ms) and **bounce** (−1 to 1), converted to
stiffness and damping. Retargeting a spring in flight keeps its current
value and velocity, so an interrupted hover, press or drag never jumps.
Completion is reported at the perceptual duration. The hover and press
springs are two uses of this.

### 7.3 Visual states

A component declares **state groups** (for example `interaction`: normal,
hover, pressed, disabled; `selection`: off, on; `availability`: available,
locked, new; `validity`: valid, invalid; `load`: loading, ready), each state
a set of prop values. The engine derives the interaction states from input
and the others from bindings (`locked = !item.owned`). Moving between
states animates every changed prop with the transition the component, the
style or the pair of states gives (`hover -> pressed: spring 120 0.2`).
A state can also start a timeline on entry or exit. States are in
`dump_ui` and settable through the probe (§16.2) so an agent can render
and check each one.

### 7.4 Timelines and choreography

A timeline is keyframed tracks over element props, with eases or springs
per key, and event tracks (a UI sound, a haptic, a particle burst, a
state change, a callback to game code). Timelines play, reverse, loop,
seek, and can be labelled and scrubbed. Choreography operators compose
them:

- `sequence` and `parallel`;
- `stagger` over a list by index, by distance from a point or from the
  centre, with a total cap;
- `cascade`: each step's start waits for an event of the previous one (a
  score that counts chips, then multiplies, each step pitched higher).

### 7.5 Presence and layout animation

- **Enter and exit** per element: an element removed from the tree is kept
  (inert, not hit-testable, not in the accessibility tree) until its exit
  finishes, so lists, toasts and panels leave the way they arrived.
- **Glitch** (`glitch`, a transition kind beside fade, the slides, scale
  and pop; `motion { enter glitch 0.8 }` with its strength): the content
  draws through the glitch layer material (§5.4: slices pushed sideways,
  colours split about 3.5 px) as it fades in or out, over 150–180 ms
  whatever the style's base duration, weakening to nothing as it settles.
  Its slice height comes from the element's id and its slices are re-cut
  every two ticks of the UI clock, so a captured frame is the same every
  run; under reduced motion it is a plain fade. An element's own exit
  (props) has no tear: it fades. A transition between whole screens that
  captures the outgoing one is not implemented.
- **Layout animation**: an element whose laid-out rectangle changes
  (reordering, inserting, resizing, wrapping to a new line) moves from
  the old rectangle to the new on a spring (FLIP).
- **Shared elements**: elements with the same `match` key on the screen
  being left and the one arriving morph between their rectangles,
  transforms and corner radii, cross-fading their contents.

### 7.6 Gestures

Drag with axis locks and bounds, rubber-banding past bounds, flick with
momentum and snap points (carousels, card fans, sheets), swipe to dismiss,
pull to refresh, pinch and rotate on any element, each with thresholds and
velocities the motion system takes over on release.

### 7.7 Motion personality and reduced motion

A style defines motion tokens (§12.2): named springs (`ui`, `panel`,
`emphasis`, `playful`), enter and exit eases, durations, stagger, and the
overshoot it allows. Components use tokens, so a style's personality
(crisp and military, heavy and ornate, bouncy and casual) changes every
motion at once. Under reduced motion each animation uses its declared
fallback (a fade, or none) and decorative loops stop; `og ui check`
reports a missing fallback.

The design language's timing table (`packages/design`, `motion.timing`;
its Motion section has the table) sets what each moment moves on: hover
stickiness (the hover spring and a pad cursor's magnet), press squash,
enter and exit, staggered cascades, toasts and screen transitions. A
style's motion block overrides any row: `hoverSpring`, `magnet <0–1>`,
`pressSpring`, `ease widgetEnter|widgetExit`, `stagger`, `staggerCap`,
`toast <kind>`, `toastHold <ms> [<error ms>]` and `screen <ms>`. A row a
style leaves out takes the table's.

### 7.8 Sound and haptics

Motion events and UI events play sounds and haptics from the style's sets
(§12.4): hover, press, focus, select, confirm, cancel, back, open, close,
tab, toggle on and off, slider and scroll ticks, typewriter blips, counter
ticks (pitch rising through a cascade), notification, warning, error,
success, reward, and rarity reveals by tier. Haptics are patterns on the
same events: pad rumble, the phone's vibration on the web and native, with
a player setting for intensity and off. A sound is queued for the tick of
the input that caused it.

## 8. Layers, input routing, actions and glyphs

### 8.1 The layer stack

A game's UI is a stack of **layers**: HUD, menu, modal, popover, toast,
tooltip, system. `og.ui.push(screen, params)`, `pop`, `replace` and `clear`
manage it; a document declares its layer kind. Each layer declares:

- its **input mode**: `game` (input reaches gameplay; the HUD), `menu`
  (input stays in the UI) or `all`; whether the pointer is captured;
- its **default focus** and whether to restore focus when it returns to
  the top;
- what **Back** does (pop, a named action, or nothing for a root);
- whether it **pauses** the game, and what happens to layers under it
  (dimmed, blurred through a backdrop, hidden, untouched);
- its enter and exit transitions.

The stack is ordered by kind in that list's order, then by push order: a
modal pushed before a menu stays over it, and a toast or tooltip stays over
both. The **active** layer is the top one whose input mode is `menu` or
`all` (the topmost when none is). Input goes to it, once per tick. It takes
Back, `pop` (og.ui.pop's and a document's; a document's pops its own
layer), `replace` and focus. While a layer is over it, it keeps the focus
it had, and it hears `activate` and `deactivate` as it becomes and stops
being the active one.

The model is Unreal CommonUI's activatable widgets:

| CommonUI | og-ui |
|---|---|
| Layer groups (Game, GameMenu, Menu, Modal) in a primary layout | layer kinds, ordered by kind, then push order |
| `GetDesiredInputConfig` (Game, Menu, All) | `input game\|menu\|all`; `capture` for the mouse |
| The active activatable takes input; widgets below it are deactivated | the active layer; the layers under a `menu` layer are inert |
| `bIsBackHandler`, `NativeOnHandleBackAction` | `back pop\|none\|emit <event>` |
| `GetDesiredFocusTarget`, `bAutoRestoreFocus` | `focus "<path>"`, `remember` |
| `OnActivated`, `OnDeactivated` | `activate`, `deactivate` events |
| The action bar from the active widget's bindings | `og.ui.actionBar`: the active layer's `actions` |
| `bIsModal` | a `menu` or `modal` layer: `capture`, `under dim` |

One difference is deliberate: CommonUI shows one modal at a time; here a
second modal stacks over the first (a confirm over a confirm).

A pointer takes the topmost element whose box holds it, with two reaches
past the box. An element's `hitSlop` reaches that far on any pointer; on a
coarse pointer every control's reach also grows to the 44 CSS px touch
target on each side it is shorter. A point held only by reaches goes to
the nearest box among them: a sibling's own box takes it, a box around them
(a dialog's backdrop, a panel, a scroller) never does. So a phone's packed
map of 20 px nodes takes taps between them for the nearest one, and a
gap between two small buttons splits at its middle.

### 8.2 Actions and glyphs

UI actions are named (`confirm`, `back`, `tabNext`, `tabPrev`, `details`,
and any the game defines) and bound to og-input actions, on press, release
or hold. A component asks for an action, not a key. The **action bar**
component shows the actions the top layer accepts, each with its glyph.
Glyph sets cover keyboard and mouse, Xbox, PlayStation, Switch, generic
pads and touch, and switch live to the device last used. Every hold has a
toggle alternative (XAG 107).

### 8.3 Focus

One focus for pointer, keys and pads, with scopes and several local
players with coloured rings; explicit neighbour overrides, focus groups
that wrap or not, focus memory per layer and per group, an optional
pad-driven virtual cursor for dense screens (a 4X map, an inventory grid),
and several simultaneous cursors for local multiplayer selection screens.
Inside a focus group Home and End go to its ends and PageUp and PageDown
as far as one view reaches; in a list's group (role listbox, menu, list,
tree, tablist) letters typed under a second apart pick a member by name;
Up and Down open a closed dropdown (rune-kit's keyboard model,
`third_party/rune-kit`). Multitouch gestures and the accessibility tree (with its ARIA mirror on
the web) follow the same model.

### 8.4 Player-customised HUD

Elements marked `customizable` (`ElementProps.customize`: a key, per-axis
locks and scale and opacity limits) can be moved, scaled and faded by the
player; virtual controls on touch (sticks, buttons) are customisable the
same way. Placements are kept per player by key and applied before the
frame's transforms, so touches follow them. `og.ui.setHudEdit` is the edit
mode (press to select, drag to move, pinch or wheel to scale, the stick for
pads), `og.ui.hudLayoutEditor` its panel, and
`og.ui.hudLayout()`/`setHudLayout(json)` move a layout in and out of the
game's settings. The pad-driven virtual cursor of §8.3 is
`og.ui.setVirtualCursor` (speed, fine-aim acceleration, magnet, right-stick
scroll); A is its pointer button, the style draws it.

### 8.5 Gesture recognizers

Tap, double tap, long press (the touch tooltip), drag, swipe, pinch, rotate,
with hit slop, priorities and failure requirements (a tap waits for a
double tap only where one is declared).

## 9. Data binding

A **viewmodel** is what a screen reads and writes: a reflected resource or
component of the game (the single schema), or an object passed from code.
A document binds props to paths (`text = player.gold`, `value =
hero.health / hero.maxHealth`, `state.locked = !level.unlocked`) and lists
to collections (`for item in inventory.items`, with a template per item
type, virtualised). Controls write back through two-way bindings. Binding
changes can trigger states and timelines (`on change gold: pop`).

Formatting is part of binding, by locale through og-i18n: grouping,
compact (1.2K, 3.4M), engineering and scientific notation, letter notation
for idle games, numbers past 1e308 through a mantissa and exponent value,
durations and countdowns, percentages, plurals, ordinals, dates.

Bindings are read once per tick after the game's systems. Invalidating
only the subtrees of changed bindings (§6.3) is not implemented.

## 10. Documents

### 10.1 What they are

A **screen** (`type screen`) or **component** (`type component`) is an
`.ogt` text document: an element tree with props, slots, bindings, state
groups, timelines and breakpoint overrides. og-cook validates it against
og-ui's reflection schema and cooks it into an `og.UiDocument` blob; the
generated wiki documents every prop from the same schema. The grammar is
`crates/og-ui/README.md` §Documents (one construct per `.ogt` line, props
from og-ui's reflected element schema, `og.ui.documentSchema()`).

### 10.2 Documents and code

Immediate-mode building stays: `og.ui.*` calls from a system each tick
build the same element tree, and both can mix. Code mounts a document
(`og.ui.screen("shop", vm)`, `og.ui.component("ShopCard", props)`),
fills its slots with code-built content, and receives its events
(`clicked`, `changed`, named events from timelines). A game written in
code alone still gets states, motion and checks, because they belong to
the element model, not to the document.

`tabs` is a document control with `items`, two-way `selected`, and a
`changed` event. For any other `og.ui` widget or a kit component, a
document declares a `slot` and game code fills it after mounting:

```ts
og.ui.screen("garage", vm);
if (og.ui.openSlot("garage/Speed")) {
  racing.speedometer("Speedometer", vm.speed);
  og.ui.close();
}
```

This is the document-to-widget bridge on both hosts: the slot owns
placement, the typed call owns widget arguments and interactions. There is
no untyped widget dispatch API and no executable strings in documents.

Every screen and component is typed from its document. `og ui types`
(and `og check`, `og run`, `og build` and `og dev` before they check)
writes `.og/types/ui/<name>.d.ts` per document, which adds it to the
engine's `UiScreens` or `UiComponents` registry: its props as the contract
declares them (a list's items and an object's fields as its fixtures show
them), the events it can send (an `emit` trigger's name, `back emit`'s,
its elements' ids for clicks and gestures, and those of the components it
uses) and its slots. `og.ui.screen`, `push`, `replace`, `screenVm` and
`og.ui.component` take the name as a type parameter, so a declared name
checks the viewmodel and narrows the answer's events; any other name stays
untyped. The project template's tsconfig includes `.og/types`; an older
game adds `".og/types/**/*.d.ts"` to its `include`.

### 10.3 Why documents

`og dev` applies a changed document in place without reloading the game,
keeping the UI state. Tune and the probe write any prop of it live and save
the result back into the file (§16.3). Checks read its static structure:
every slot, state, action and binding is known without running every path
of the code.

### 10.4 The CSS dialect (version 2)

A version 2 document writes an element's look and layout as CSS
declarations and keeps its structure and behaviour as `.ogt` lines
(`crates/og-ui/src/doc/css.rs`, whose `STYLE` table is the whole mapping):

```ogt
type screen
version 2
---
name "card"
column Card {
  width: stretch; max-width: 640px
  padding: 8px 12px 16px 4px
  background: var(--panel-raised)
  transition: -og-spring(300ms, 0.1)
  @media phone-portrait and (orientation: portrait) {
    padding: 2px 6px
  }
  label Title {
    text "Title"
    font: var(--text-caption)
    line-clamp: 2
  }
  :hover {
    translate: 0 -4px
  }
  -og-state-transition: var(--motion-ui)
}
```

- **Properties** use CSS names: `width`, `min-width`, `max-width` (`fit-content`,
  `stretch`, `<n>px`, `<n>%`), `padding` and `gap` (CSS order), `justify-content`,
  `align-items`, `flex-direction`, `flex-wrap`, `grid-template-columns|rows|areas`,
  `grid-auto-flow`, `grid-area`, `grid-column`, `grid-row`, `background`,
  `color`, `border-radius`, `opacity`, `translate`, `scale`, `rotate`,
  `transform-origin`, `z-index`, `position` with `top`, `bottom`,
  `inset-inline-start|end`, `aspect-ratio`, `object-fit`, `text-align`,
  `text-transform`, `white-space`, `text-overflow`, `line-clamp`, `cursor`,
  `font` (a type step: `var(--text-<step>)`), `font-family`
  (`var(--font-<role>)`) and `transition`.
- **States** are pseudo-classes: `:hover`, `:active` (pressed),
  `:focus-visible`, `:disabled`. A block holds the values of that state;
  `-og-state-transition` sets how the group moves between them. Named state
  groups keep their `state <name> { … }` lines.
- **Breakpoints** stay named, written as media queries: `@media phone`,
  `@media tablet, desktop`, `@media phone-portrait and (orientation:
  landscape)`. A `max-width` query is an error that names them.
- **Tokens** are custom properties: `var(--panel-raised)` (a palette role),
  `var(--text-caption)`, `var(--motion-ui)`. `var()` takes no fallback.
- **Extensions** are `-og-` properties: `-og-frame` (an art slot or nine-slice
  part), `-og-spring(<ms>ms, <bounce>)`, `-og-enter`, `-og-exit`,
  `-og-layout-transition`, `-og-anchor`, `-og-hit-slop`, `-og-overlaps`,
  `-og-tint`, `-og-scroll-fade`, `-og-on-scene`, `-og-reduced-motion` and the
  rest of the table.
- **Bindings**: `bind(path)` or `bind("expression")` is a whole
  declaration's value.

The subset is strict. An unknown property or value, a unitless length
(`gap: 12`), a version 1 prop line where a declaration belongs (`width
520`), `:focus`, `float`, `margin`, `font-size` and `!important` are errors
that name the form to write. There is no cascade beyond the element's own
declarations: its `@media` blocks and states override them, and the later
declaration wins.

`og ui migrate <file|dir>... [--check]` writes a version 1 document as
version 2, line for line with its comments, after checking that both read as
the same document; a version 1 document no longer loads, and its error
names the command. Tune and agent edits (`ui:save`, §16.3) write a version
2 document's props back as declarations.

## 11. Component library

There are no fixed screen templates with a field per word. The library is:

- **Components** built only from public primitives and documents, each with
  named parts a style can dress, slots a game fills, a viewmodel contract,
  state groups, motion, sounds, an accessibility role, probe ids, a wiki
  page and tests.
- **Screen documents** for every screen type of Appendix B, which a game
  copies (`og ui eject <screen>`) and edits as its own file.

Generic components are `og.ui`'s; genre components and screens are kits'
(§17.5). Categories (the full lists are the appendices):

- **Foundations**: text, rich text, icon, image and art slot, shape,
  divider, spacer, viewport, particle emitter, layer.
- **Controls**: buttons (all variants and the hold-to-confirm), toggles,
  checkboxes, radios, sliders and range sliders, steppers, selects,
  segmented tabs, text fields and areas, search with suggestions, colour
  picker, key-binding capture, the on-screen keyboard; the touch controls
  `joystick`, `dpad` and `actionButton`, which take `disabled`, dress as
  discs and keep the finger that went down on them in the UI state.
  The switch (`toggleSwitch`), the checkbox (`checkField`), the radio
  group, the segmented control, the slider (`valueSlider`), the number
  stepper and the dropdown are one native set in og-ui
  (`crates/og-ui/src/widgets/controls`): TypeScript, documents
  (`slider`, `toggle` and `tabs` elements) and the inspector call the same
  control, so it looks and behaves the same everywhere. Their rims and
  shadows are part dressing (the default style's or a pack's), never a
  control's own draw, and the dropdown's list is an anchored popover layer
  (under its trigger, above it where there is more room, kept on screen).
  The others are composed in TypeScript from og.ui's elements.
- **Containers and navigation**: panels and cards, scroll (with `fade` and
  `fadeInto`, in documents as in `og.ui.scrollArea`), tabs with bumpers,
  carousel with snap, accordion, split views, sheets (bottom and side,
  draggable), modals and confirms, popovers, menus and context menus,
  radial wheels, breadcrumbs, the action bar, pagination.
- **Data**: virtualised lists, grids and sortable, filterable tables (10k
  rows and more), trees, pan-and-zoom graphs, charts (line, bar, area,
  radar, pie), key-value and stat-compare rows with deltas, the calendar.
- **Feedback**: toasts and the notification stack, badges, tooltips
  (nested, pinnable, with links; compare tooltips), progress and spinners,
  skeletons, counters with notation, banners and ribbons, the
  celebration set (confetti, burst, stars, level-up), subtitles and
  captions to XAG 104.
- **HUD**: meters (segmented, ghost damage, shield cells, orbs, radial
  stamina), ability bars with cooldown sweeps, charges and hotkeys,
  buff and debuff grids with timers and stacks, timers, minimap and
  compass, objective tracker, low-health vignette; genre HUDs are kits'.
- **World** (§14): nameplates, overhead bars, damage numbers, interaction
  prompts, waypoints and pings with edge clamping, placement ghosts, range
  and area telegraphs, path lines, selection rings, loot labels, speech
  bubbles.
- **Patterns**: the settings tree (graphics, audio, controls with
  remapping, accessibility, language), the HUD layout editor, `heroButton`,
  `starBar`, `overlayBar`, generic lists, slots, stores, rewards, cards and
  world tags.

`UiDesc.fallbackIcons` names an icon set that draws what the atlas (the
game's icons, else the style's) lacks; `og.ui.icon`, rich text, action
glyphs and art placeholders read the same chain.

### 11.1 Screen documents

Generic system screens (boot and splash, title, main menu, mode select,
settings, pause, save and load, loading, results, leaderboard,
calibration, tutorial, game over, HUD editor, creator tools, credits) are
`type screen` documents in `packages/engine/src/ui/screens/`; genre
screens live beside their kit's `extension.json` (§17). Each names its
viewmodel (`props`), the events it emits and which og.ui component or
pattern fills each slot at its top; its words are string keys
(`screen.<name>.<word>`) with English fallbacks, and a `fixture default`
previews it (`ui:push:<name>:default`). Where an og.ui pattern fills a
slot and draws its own title, tabs or actions, the document leaves those
to it.

`packages/engine/extensions/tests/screen-catalogue.md` maps each Appendix B
row to its documents; `packages/engine/extensions/tests/documents.rs`
holds its rows equal to Appendix B's (it parses this file's Appendix B
table) and cooks every document. `og ui eject <screen> [project]
[--force]` copies a generic screen into the game's `assets/ui/<screen>.ogt`
(refusing to overwrite without `--force`), where `og.load.document` loads
it; `og ui eject <kit>/<screen>` ejects a kit's (§17.1); `og ui eject
--list` prints each with its row.

### 11.2 Rive components

A `.riv` file under `assets/` is converted at cook time into og-ui component
blobs, one per artboard; the Rive runtime is not embedded. The pinned
format is major version 7 (the documented format,
https://rive.app/docs/runtimes/advanced-topic/format). A file of any other
major version is a cook error that asks for a re-export. Luau scripts, and
any feature the import does not convert, are cook errors that name the
feature and the object it is on. The cook never drops a feature silently.

**Settings.** The usual import-settings sidecar, `<file>.riv.ogt`:

```text
type import
version 1
---
artboards Chest Lid        # which to cook (default: every artboard)
artboard Chest {
  state-machine Open       # the default machine (default: the first)
  role button              # a11y role and name, per artboard
  name "@chest.open"       # a string key or literal
  fit contain              # contain cover fill none scale-down fit-width fit-height
  align center             # where the fit places it
}
font Inter {               # a font asset, by its name in Rive or font<n>
  source "fonts/Inter.ttf" # a referenced (not embedded) font's file
  runtime true             # text set at run time renders from its outlines
  chars "0123456789"       # characters beyond those its runs show
  charset latin            # named sets beyond them
}
```

**Identity.** An artboard's id is its asset path, `#`, and its name:
`ui/chest.riv#Chest`. A document uses it as a component by that id. A
style fills a frame with it:
`Part::name.variant:state { -og-frame: rive("ui/chest.riv#Chest") }`.
The id is the only way to spell it.

**What the component exposes**, which documents and TS bind like any
component's (its `.d.ts` is generated from them):

| Rive | og-ui |
|---|---|
| View-model properties: number, string, boolean, colour, enum, nested view model | Props: `f32`, `string`, `bool`, `Color`, a union of the enum's values, a nested props object. Lists and image or artboard properties are cook errors until phase 3. |
| View-model trigger | A callable trigger, `fire(<name>)` |
| State-machine inputs: number, boolean | Props `f32`, `bool`, named `<machine>.<input>` unless a view-model property binds them |
| State-machine trigger input | A callable trigger, `fire(<machine>.<input>)` |
| Rive events, general and open-url | og-ui named events, with the event's custom properties as the payload; an open-url event carries its URL as data, and the engine opens nothing |
| Artboard width and height | Intrinsic size, 1 Rive unit = 1 CSS px, placed by `fit` and `align` |

**At run time.** An imported artboard runs on the UI tick with a fixed step:
its animations, state machine and bindings advance with the world and are
saved and restored with og-ui's state. It never reads the wall clock.
Each Rive shape, text run, image and nested artboard is an og-ui element
(id `<artboard>/<object name>`, its source the `.riv` path and the Rive
object id). These elements show in `dump_ui`, `describe` and `pick` like any
other element and draw with the UI pass's shapes and paths (§5.1).

Phases: (1) artboards, shapes, gradients, strokes, images, text, keyframes
of transform, opacity and colour, nested artboards; (2) state machines as
og-ui state groups, and view models as data binding; (3) path morphing and
bones.

**Status.** Converted today (og-cook `riv`, og-ui `rive`): artboards and
their backgrounds, groups, shapes of points paths, rectangles, ellipses,
triangles, polygons and stars, fills (both rules) and centred strokes of
solid colours and linear and radial gradients, blend modes normal,
multiply, screen, overlay, soft light and colour dodge, linear animations
(hold, linear, cubic ease and cubic value keys of transforms, opacity,
colours and the paths' numbers; one-shot, loop, ping-pong, work areas),
nested artboards with their simple animations, embedded images (each
cooks to its own texture, `<file>.riv#image<n>`, drawn at its pixel size
about its origin and bound when the artboard loads), and a state machine
whose entry plays one animation, and text (below). Each artboard cooks to an `og.RiveArtboard`
blob; `og_ui::rive::Player` advances it by the tick's step, draws it as UI
shapes and lists its drawn objects as elements (`Player::elements`).

Text: a text's runs, each in its style's font, size, line height and
letter spacing, filled and stroked by the style's solid or gradient
paints; its box sized automatically, by width or fixed; left, centre and
right alignment and top, middle and bottom vertical alignment; the
origin at the box's top or the first baseline; paragraph spacing;
wrapping with normal or break-word breaks; overflow visible, hidden,
clipped, ellipsis, fit and fit font size; and keys of the runs' words,
the styles' fonts, sizes, line heights and letter spacing and the boxes'
size, origin and spacing. Its layout is rive-runtime's (`Text::update`
and `buildRenderStyles`, `GlyphLine`, `OrderedLine`); its words are
measured, shaped and drawn by og-ui's text, each piece an og-ui text
command (styled for a gradient, a stroke or a turned or scaled text).
Each font a text uses cooks beside the artboard as `<file>.riv#font<n>`
(an embedded one from the file, a referenced one from its settings'
`source`), loaded with it: its charset is the characters its runs show
at rest and as keyed (and the ellipsis a text that ellipsizes draws),
plus its settings' `chars` and `charset`; it ships its file for run-time
glyphs only with `runtime true`. A character the font lacks is the
font cook's note and og-ui's missing glyph; a font with none of its
runs' characters is a cook error naming them. A text's element carries
its words. A variable font's axes at other than their defaults, font
features, text modifiers and follow-path, backgrounds, global styles,
text inputs, vertical trim, break-all word breaks, runs from view-model
lists, a text's blend mode other than normal and a referenced font the
settings give no source are cook errors naming the asset and feature.

Hosted (not embedded) images, images with a blend mode other than
normal, inner and outer strokes, gradient transforms and every other
state machine are still cook errors.

In documents: a container's `-og-frame: rive("<file>.riv#<Artboard>")`
(the `rive` prop) plays that artboard in its box over its background.
`og.load.document` loads the artboards its document names (one missing
refuses the document; an edited `.riv` reloads under `og dev`). Each
element keeps its player in og-ui's state, advanced once per tick from the
tick after it first builds; a save carries each player's bytes
(`Player::save`) beside its element, and a restored one is rebuilt
(`Player::new`, then `restore`) when its element next builds. Bytes that
do not fit the artboard are reported and kept, not reset, and the frame
draws nothing until they fit. Its drawn objects are elements of kind
`rive` under it, their `detail.source` `<file>.riv#<artboard>/<object>`.
Not yet: a style dressing's `-og-frame: rive(…)`, an artboard used as a
component by its id with its view-model props and events, and pointer
input to the artboard.

## 12. Styles

### 12.1 What a style is

A style (`type style` version 3, the CSS dialect of §12.7) has tokens, part dressings, a motion
personality, a sound and haptic set, fonts, icons and art. It is one
complete, validated asset a look's `ui` line names (`og style eject|pack`
take it apart and back). The engine's default style is the design language
the engine shares with origin-agent (`packages/design`), the one a game
draws in until it picks another. A game may ship it, a built-in pack, or a
style of its own, generated or hand-made: restyling is the game's choice,
never a requirement of the engine, its checks or a publish build. The
design language binds the default style alone; a game's style may restyle,
re-dress, re-shape or replace anything above the accessibility floor of
§15.2.

The default style carries no hue in its chrome: its primary is the
high-contrast neutral (a near-white fill with dark ink on dark surfaces),
set apart from secondaries by fill and weight; panels, controls,
selection, focus, tabs, sliders and rings are drawn from the neutral
ladder and told apart by tone, lift and weight. Hierarchy, frames,
material, type and spacing follow the clean sci-fi direction (concept B,
§15.5): deep navy surfaces, glass with a lit top edge, letter-spaced
uppercase titles. Semantic roles (danger, success, warn, info) and game
roles (health, mana, xp, gold, rarity, team) keep their hues because they
carry meaning.

### 12.2 Tokens

- **Colour**: the palette roles, plus rarity tiers (common, uncommon,
  rare, epic, legendary, mythic), team and player colours (8), damage and
  element types, and chart series. A style may also declare roles of its
  own (a game's corruption, sanity or faction colours); components and
  game code read them by name like any role, and the completeness checks
  cover them. Each status and rarity colour is paired with an icon or
  pattern, so no meaning is carried by colour alone (XAG 103). The
  selection, primary and accent roles and the scene roles are §17.4's.
- **Type**: a scale of display, title, heading, body, label, caption and
  numeric steps per breakpoint, with tracking, weight and line height;
  fonts per role, with CJK and fallback faces. An optional button step
  sets a button's words apart from label (the label step when left out):
  a pixel face keeps label at a HUD's 12 and sets buttons at body. §17.4
  has the typography roles.
- **Space, size and radius** scales; **elevation** (shadow sets per level);
  stroke widths; **shape language** (corner shape and radius per element
  class: squircle, notch, bevel, round, square).
- **Motion**: the springs, eases, durations and stagger of §7.7.

The optional `motion` lines `ease widgetEnter <curve>` and
`ease widgetExit <curve>` control built-in widget arrivals and departures:
layer transitions, toasts, meter trails, staggered children and bursts.
They accept four Bézier control points, `linear`, or `steps <1–60>`.
Omitted, they are enter `0.23 1 0.32 1` and exit `0.4 0 1 1`. They are
separate from the `enter` and `exit` tokens, which built-in widgets do not
read.

### 12.3 Part dressings

A component names its parts (`ShopCard.frame`, `Meter.fill`,
`Button.base`); a style dresses any part, by variant and state, with
selectors: `Button.base[primary][pressed]`, `Slot.frame[rarity=epic]`. A
dressing sets paints, strokes, effects, shape, nine-slice art, ornaments
(corners, flourishes, dividers, tiles), materials and sounds. A part a style
does not dress is drawn from the tokens by the component's default, so a
new component looks right in every style without new art. The twelve
`StylePart` classes of version 1 are dressings of the matching parts;
`og style upgrade` converts a version 1 style.

An element-backed part also accepts an `appearance` block. It uses the
document property's reflected type and literal grammar (§10); properties
marked `dressing` in `documentSchema` are accepted. `opacity` applies to
the whole element, including its label and children; `motion` controls
changes to its animatable properties. `color` and `step` apply when the
part is a text element. `padding` and `gap` override the part's container
spacing before layout, including the measured size of a fit container
(four padding values are left, top, right, bottom; one value applies to
all sides). `width` and `height` set an element's layout size with the
document size grammar; `x` and `y` translate the element and its children.
These overrides apply after the widget and responsive variants are built,
before layout and motion, and never change hit slop or input routing.
Unset properties retain the widget's defaults; more-specific selectors
override only the properties they set.

```ogt
ActionButton::face:disabled {
  opacity: 0.3
}
Card::frame {
  padding: 3px 7px 11px 13px
  gap: 17px
}
```

In version 3 these are the documents' own declarations (§10.4) in the
dressing's block; version 2 wrote them in an `appearance { }` block.

Decorative parts accept paints, `ink`, the same fills, shapes, strokes,
effects and materials as other parts, and `appearance { width … height …
x … y … opacity … }`: dimensions resize the decoration about its centre,
then the offsets move it in CSS pixels; `percent` and `grow` use the owning
widget's box; `fit` keeps the decoration's default size within its bounds;
opacity multiplies the widget's. A widget's outer size and each
decoration's size are independent (enlarging a joystick's base does not
enlarge its knob). An undressed decoration keeps its original drawing;
dressing only its fill keeps its radius and adds no outline or shadow. The
decorative parts:

- `Meter.trail` and `Meter.separator`, with the meter's
  `[kind=health|mana|xp|primary|custom]`; the trail has `[loss]` when it
  follows a decrease. `Joystick.knob`, `Joystick.rim`, `Joystick.shadow`,
  `Joystick.sheen` and `Joystick.direction`, with `[active]` and
  `[disabled]`.
- `Slider.track`, `Slider.fill`, `Slider.knob`, `Toggle.track`,
  `Toggle.knob`, and a progress bar's `Meter.track` and `Meter.fill`. On
  segmented tracks the override sizes each segment after distribution;
  `segments` still owns their count and spacing.
- `Minimap.marker[player]`, `Minimap.marker[tone=…]`,
  `Minimap.markerOutline[player]`, `Skeleton.sheen`, `Spinner.dot[age=0…7]`
  (zero is the leading dot), `ArtSlot.placeholder` (the disc used without
  an icon set), `Toast.indicator[tone=…]`, `Spotlight.edge`,
  `Slot.indicator[selected]` (the version 1 selection underline),
  `Stepper.mark[axis=horizontal|vertical]` (hover, pressed, disabled) and
  `Radio.mark` (on, hover, pressed, focused).

Embedded strokes, sectors and labels accept the same geometry and opacity
properties, plus `appearance.color` (or `ink`); labels also accept
`appearance.step`. They retain their primitive drawing: a sector stays a
sector. These parts are:

- `Checkbox.mark[stroke=short|long][on]`,
  `Select.chevron[side=left|right][open][disabled]`,
  `Dpad.chevron[direction=Up|Down|Left|Right][pressed]`,
  `Dialogue.continue[side=left|right]`, and `Burst.ray`.
- `Cooldown.track`, `Cooldown.sweep[ready]`, `Cooldown.mask`, and
  `Cooldown.core[pressed][ready]`; the same parts on `ActionButton` dress
  its charge ring. The mask covers the unused half of a partial sector;
  give it the track's colour. The core covers the middle; its size
  controls ring thickness.
- `Meter.label[surface=fill|track]`, `Cooldown.label[ready]`,
  `ActionButton.label[disabled][pressed]`, `Minimap.north`,
  `Dialogue.initial`, and `ArtSlot.label`. Meter label clipping follows the
  current value whatever its type or colour.
- `Icon.image`, `ArtSlot.icon`, `Cooldown.icon[ready]`,
  `ActionButton.icon[disabled][pressed]`, `Dialogue.portraitIcon`, and
  `Minimap.image` accept geometry, tint (`color` or `ink`), and opacity.
  Atlas images use the requested rectangle; distance-field icons keep
  their square em box centred in it. The icon's layout box is unchanged.
  `Dpad.indicator[pressed][direction=Up|Down|Left|Right]` and `Burst.spark`
  accept the rectangle decoration properties.
- `HudEditor.outline[side=top|bottom|left|right]` (flagged `selected` on
  the selected element) and `HudEditor.handle[selected]`: decorations
  only; drag constraints, scale steps and the saved placement still use
  the customised element's box.
- `TextEdit.selection`, `TextEdit.composition`, and `TextEdit.caret`
  dress single-line fields and text areas without changing selection,
  caret placement or IME composition. The caret's `appearance.motion`
  duration sets its blink half period, rounded to ticks; the blink stays
  binary. Unset is 530 ms; `instant` keeps it visible, as reduced motion
  does (`motion tween 100 linear` blinks six ticks shown, six hidden at
  60 Hz). An edit or caret move restarts the shown phase.
- Focus outlines use the control's component with part `focus`, such as
  `Button.focus[player=0]`; `player=1`, `2` and `3` are the extra
  local-player rings, and the accessibility outline also has `[outline]`.
  Controls without a part use `Element.focus`. These change only the ring;
  an unset stroke keeps the default ring, `stroke none` removes it.
- `Skeleton.frame` controls the stack's spacing and bounds;
  `Skeleton.line[last]` its shortened last line and `Skeleton.line` every
  line; `Spinner.base` the spinner's bounds. `Skeleton.sheen` and the
  unqualified `Spinner.dot` use their `appearance.motion` duration as the
  cycle period and a tween's curve as the phase curve; `instant` stops at
  phase zero. Unset, the periods are 1600 ms and twice `slow`.
- `Scroll.bar[axis=horizontal|vertical]` controls scrollbar geometry,
  radius, paint and effects. `Heading.flourish[side=left|right]` controls
  the atlas ornament's size, offset, tint and opacity; a matching dressing
  opts headings below 28 px in, alignment chooses which sides fit, and
  `opacity 0` hides a side. The style supplies the ornament asset.

The engine keeps input and layout rules, which are not appearance: touch
target floors, hit slop, pointer ownership, dead zones, value mapping,
scroll extents, focus order, text shaping and selection, HUD placement
constraints, meter interpolation. Primitive topology, clipping, pixel
snapping, sector angles and antialias overlap also stay in Rust. Paints,
materials and assets change presentation, never the interaction model.

With no dressing, the default drawing keeps its numbers (text shadow reach,
pixel-font effect quantization). Authored text effects through
`og.ui.setDraw` replace the default text drop. An authored value goes
through the reflected property grammar, never a second list of
widget-specific numeric tokens.

A game shipped with a version 1 style keeps the screens it shipped with:
an engine update may draw them better (antialiasing, contrast, a cleaner
fill) but never changes what is on them or where (`UiTheme.styleVersion`
1). Its dialogs' titles take the ramp's xl in the body face, its ramp
steps no tracking it did not set, a grid's selected cell its `primary`
underline and a burst (`ui.burst`) its drawn rays and sparks rather than
particles; its screens are its own ejected documents (§11). A version 1
file's slider knobs and switches take the control radius; `og style
upgrade` writes `screens 1`, which keeps all of this but draws them round.
A new style (`og style new`, the packs) draws the current defaults.

### 12.4 Sound and haptic sets

The events of §7.8, each with one sound or a set chosen at random without
repeats, volume and pitch ranges, and a haptic pattern.

### 12.5 Built-in styles and generated styles

The engine ships sixteen style packs under `templates/styles` (in a
release, `styles/<name>`): fantasy, jade, kawaii, cartoon, midcore,
storybook, craft, pixel, gilded, tactical, wasteland, dread, scifi, cyber,
foundry, broadcast. Fantasy, scifi, cartoon and pixel are the flagship
packs; the others are frozen. scifi is real-world spaceflight operations,
kept apart from military sci-fi; cyber is high-contrast yellow, red and
cyan with cut corners. Fonts and icon sheets shared by several packs live
in `templates/styles/shared/`. Genre language is free; another game's
names, logos, fonts and exact layouts are not.

A **style generator** (`og style new`) makes a game its own style from the
look, the premise and a few choices (palette seed, shape language,
ornament level, motion personality, finish): tokens and procedural
dressings first, then the art slots of its frames, buttons, ornaments and
icon set filled through the art brief (§13.3). With a look that matches no
family it starts from quiet, light minimal choices. Each pack owns
`choices.json` and `style.ogt` as independent authored data; directory
discovery supplies the seed catalogue. There is no regenerate-all command
or seed-equality test. A generated style passes parsing and completeness
checks; contrast is a measure, not a threshold (§16.5).

### 12.6 Painted art at every density

A style's atlas line may name denser copies of one layout,
`atlas "atlas.png" 384 256 2 3`: these are `atlas@2x.png` and
`atlas@3x.png` beside it, at 2 and 3 texels per CSS px. Regions and insets
stay in 1× texels, so a slice cuts the same art from every copy. Each frame
draws from the copy nearest at or above its device pixels per og-ui unit
(dpr times the resolution-relative unit), and from 3× above that. Phones
top out at 3, and tablets and desktops at 2. `og style art` (docs/art.md
§2.7) packs a style's painted slots into all three.

Painted slices draw as authored, but a style's `recolour "<from>" <role>`
lines make their accents follow the palette: the cook recolours every
density of the atlas when the role's colour is no longer the one the art
was painted in (docs/art.md §2.7, step 4), so a game's copy of a pack
takes the game's accent.

### 12.7 The CSS dialect of styles (version 3)

A version 3 style writes each dressing as a selector block and each colour
and length token as a custom property. Its art geometry, fonts, icons,
typography, elevation, shapes, motion and sounds keep their `.ogt` lines
(`crates/og-ui/src/style_file/css.rs`, whose `DRESS` table is the whole
mapping):

```ogt
type style
version 3
---
name "fantasy"
palette {
  --panel-raised: #211a15
}
space {
  --space-xs: 4px
}
Button::base.primary:hover {
  -og-slice: 328 401 168 44 12 14 12 14 authored
  -og-tone: 0.783 0.408 0.068 0.99
  box-shadow: 0 3px 4px 0 #00000073
}
Slot::frame[rarity=epic] {
  -og-stroke: 2px #b77ae8 inside
  -og-effect: outer-glow 8 #b77ae866
}
```

- **Selectors**: the part is a pseudo-element (`Button::base`), a variant a
  class (`.primary`), an attribute stays `[rarity=epic]`, and a state is a
  pseudo-class: `:hover`, `:active` (pressed), `:focus-visible`, `:disabled`,
  `:selected`, `:checked` (on), `:expanded`. The specificity rule is
  §12.3's: more qualifiers apply later, and blocks of one selector merge.
- **Tokens**: `palette`, `roles` and `colors` hold `--<role>: <colour> [cue]`;
  `space`, `size`, `radius` and `stroke` hold `--<block>-<name>: <n>px`.
- **Dressing properties**: `-og-slice`, `-og-tone`, `-og-recolour`,
  `box-shadow`, `-og-glow`, `-og-glass`, `-og-corners`, `-og-crest`,
  `-og-edges` and `-og-tile` (`true`), `-og-shape`, `-og-press`,
  `-og-hover`, `-og-fill`, `-og-stroke`, `-og-effect`, `-og-material`,
  `-og-ornament`, `-og-ornament-tint`, `mix-blend-mode`, `-og-mask`,
  `-og-sound`, `-og-segments`, `-og-ink` and `-og-step` (a text part's).
  Their values are §12.3's grammar written as CSS: roles as `var(--line-strong)`,
  colours bare (`#ffcc00`), keywords in kebab case (`inner-shadow`), lengths
  in px or unitless. `-og-fill`, `-og-stroke` and `-og-effect` declarations
  stack in order, like the layers of a CSS list.
- **Element properties** a dressing overrides are the documents'
  declarations (§10.4): `opacity`, `transition`, `color`, `font`,
  `padding`, `gap`, `width`, `height` and `translate`.

The subset is strict: `background`, `border`, `filter`, `border-radius` or
any other property it does not have, a `dress "…" { }` block, `.hover` for a
state, `:focus`, a comma between values or another unit are errors that name
the form to write. Version 1 and 2 styles still load. `og ui migrate`
writes a version 2 style as version 3, line for line with its comments,
after checking that both parse to the same style; `og style upgrade`,
`og style new`, `og style pack` and `og style art` write version 3.

## 13. Art slots

### 13.1 The slot

An `art` element (or an image paint) is a **slot**: a name, a kind, an
aspect (or the aspects allowed), a focal point that cropping keeps, a safe
rectangle where nothing important may sit because text or controls are
laid over it, a fit, a mask, and a treatment from the style (vignette,
duotone, frame, rarity glow). Its placeholder is drawn from the style and
the kind (a gradient, the kind's silhouette icon, the slot's name in
the default style only), never an empty grey box, and `og ui check` reports
slots still on their placeholder.

### 13.2 Kinds

Appendix F lists them with their aspects (og-ui's `SlotKind`,
`crates/og-ui/src/slot.rs`): key art and title backgrounds, store
capsules, the app icon, character splash and full body, portraits and
busts with expression sets, VN sprites, event CGs, item and equipment
icons (multi-cell for grid inventories), ability and buff icons, card art
and card frames, unit portraits, level, track and song thumbnails, save
thumbnails, menu backgrounds, banners, player banners and calling cards,
emblems and rank badges, achievement icons, emotes, currency icons, map
glyphs, tech and talent node icons, tutorial illustrations, cosmetic
previews (from a viewport, §5.5), and the style's own UI art (frames,
buttons, ornaments, icon sets).

### 13.3 The art brief

`og ui slots` lists every slot of a game with its spec and whether its art
is set or it still draws the placeholder. With `--json` each slot is a
whole generation spec: its kind, aspect, focal point, safe area, fit,
mask, transparency, tiling, legible size, nine-slice insets (`slice`),
multi-cell split (`itemCells`, which sets the aspect when no `ratio` is
given), states, the source file and line, and its set (`artSet`, the
kind's name by default): every slot in the game drawn with one look, and
as reference the first member whose art is set, so a generator matches
it. The art brief
(`packages/knowledge/scripts/brief.ts`, a public contract) has the UI kinds
above, each with its aspect, focal point and safe area, the look's conform
rules (`docs/art.md` §2.3), and, for style art, the nine-slice insets and
the states to draw. Adding kinds to `ART_KINDS` is additive; any change to
existing signatures follows AGENTS.md's rule for a break. A generated image
lands in the slot after the look's conform pass, fades in when loaded, and
streams like any texture.

## 14. World-space and 3D UI

Three forms, by volume:

- **Anchored elements**: any element or document anchored to an entity or
  a world point with an offset in world or screen units. Projected each
  tick with the tick's camera. Options: scale with distance or keep a
  screen size, fade or hide when occluded (a depth test against the
  scene), level of detail by distance, **declutter** (overlapping anchored
  elements resolve by priority: shift, stack or hide), and **edge
  clamping** for waypoints and pings (kept inside the safe area with an
  arrow toward the target and the distance).
- **Mass elements**: a pooled, instanced path for damage numbers, overhead
  bars, nameplate text, loot labels and status icons at thousands per
  frame. A template is declared once; each instance carries a few values
  (position, value, colour, start tick); its motion is a timeline evaluated
  in the shader; there is no per-instance layout. og-render's floating
  text is one use of it.
- **World panels**: a document drawn into a texture on a mesh (cockpit
  screens, terminals, a shop sign), lit or unlit, interactive through a ray
  from the pointer or a pad cursor.

**Meta UI** (damage vignettes, low-health pulses, sanity distortion) is a
full-screen layer element with a UI material over the scene.

## 15. Quality bars

### 15.1 Performance budgets

On the reference machines (metal, DX12):

- a dense menu screen (300 elements, 40 animating) builds, lays out and
  records its commands in ≤ 0.5 ms of CPU per tick;
- a HUD plus 2,000 mass elements and 100 anchored elements in ≤ 1 ms of
  CPU per tick;
- the UI pass in ≤ 1 ms of GPU at 2560×1440, with layers and backdrops:
  the `ui-layers` and `overlay` passes of `ProbeTimings.gpuPasses` (the
  overlay's glass and screen lines included);
- a 10,000-row table scrolls without a frame over budget.

No perf test measures these budgets yet.

### 15.2 Accessibility

The accessibility tree, its ARIA mirror on the web, reduced motion and
reduced transparency, held to the Xbox Accessibility Guidelines:

- text scale to 150 % without loss of content (the player's text-size
  setting tops out at 150 %; 200 % is not implemented); minimum text 18 px
  at 1080p on desktop and phone-equivalent sizes on touch, for menus, body
  text and controls. A dense HUD (strategy, MMO, simulation readouts) may
  declare its text dense (`ElementProps.dense` on the readouts' group) and
  go smaller, down to 12 px at 1080p times the text scale, provided it
  grows with the player's text scale. og-ui reaches the floor by its unit
  (`UiDesc.unitScale`, `UiTheme.unit`): on a mouse or pad surface the UI
  lays out in units of its 1080p-equivalent height over 720 CSS px, 1 to
  2, and draws them at that size, so the whole UI zooms with the surface
  (a 1280×720 layout fills a 1080p window; body 14 is 21 CSS px, the
  smallest step 12 is 18), the player's text scale multiplying over it. A
  touch surface keeps CSS px; a game may pin it. What the UI gives a game
  (`og.ui.viewport`, `og.ui.bounds`) is in units; what leaves for the page
  (the probe, the accessibility mirror, the input method's window) is CSS
  px;
- a high-contrast mode at 7:1;
- no meaning by colour alone;
- every hold has a toggle; every timed choice has a setting;
- remapping through the settings components;
- subtitles per XAG 104;
- the native host sends incremental AccessKit trees over
  `ORIGIN_FRAMES_SOCKET` (docs/design.md §2.6). The window-owning Origin
  Desktop applies them to the OS and returns actions; its adapter is
  platform ask 25.

### 15.3 Localization

Every screen is checked under a pseudo-locale that lengthens text by 40 %
and adds accents and brackets, under a CJK locale and under a right-to-left
locale (mirrored layout). No box is sized to its English text.

### 15.4 The review bar

The kit's and the packs' frames are reviewed by the art runner against
shipped commercial games; `og ui check` and the gates report facts and
never score a look (§16.5). How a frame is reviewed:

- A UI frame is drawn and reviewed over a plain, solid-colour ground (the
  pack's), never over a 3D scene or key art: a scene behind a HUD is
  judged by that scene's own tests, and no gate scores a label's
  legibility or contrast against what is behind it. `onScene` and
  `sceneInk` (§17.4) remain API for games.
- The kits and the style packs are a polished baseline an agent builds
  from, not a promise that every screen fits every style. A screen is
  judged in its kit's own style and a pack on its own pages (`og ui
  sheet`, §17.3), never as the cross product of every screen in every
  pack.
- A changed screen is read at desktop 1920×1080, tablet, phone portrait
  390×844 and phone landscape, and under the pseudo-locale, a CJK locale,
  a right-to-left locale and 150 % text.
- Every interactive part is captured idle, hovered, pressed, focused and
  disabled, and its transitions are watched.
- A declared overlap (§6.2) is not a way to carry a known defect, and a
  frame with a defect is fixed, never recorded as a reference.

### 15.5 The art bar

§15.4 says how frames are reviewed; this is the checklist the art review
reads them against, aiming at the look of shipped commercial games
(Diablo IV, League of Legends, Destiny 2, Genshin Impact, Honkai: Star
Rail, Hades, Balatro), desktop browser first. It is a reviewer's read, not
an engine check.

**Concepts.** Review images live in the art review, not committed evidence.
Clean sci-fi is the target of the engine's default style (§12.1) and the
kit's own frames; A, dark fantasy, of the fantasy pack; C, bright
stylised anime, of the kawaii pack. A frame is read beside its concept (or
the nearest one) for hierarchy, material and art, not pixel positions. In
the default style any hued chrome is a finding.

**One selection grammar in the packs.** In every style but the default and
a version 1 style, a selected chip, slot or tab is one mark: the accent's
hairline (`primary`, at the part's rim width where the style dresses a
rim), a ground one step up (`controlStrong`) and its words in the accent's
ink at 4.5:1 there (`dress::pack_grammar`). A selection or focus colour is
never a tier hue. An unchecked checkbox or radio is a well whose rim reads
3:1 on the well and on the panel (`dress::well_rim`).

#### 15.5.1 Hierarchy

Each screen has one dominant element (the key art and logo of a title, the
splash of a banner, the paperdoll of a character screen, the focused row
of a settings page) with the most contrast, size or saturation, then at
most two more levels: section titles and body. Titles use the style's
display face and step (30–60 px at 1080p), section labels its caps or
small caps, body 18–22 px, values tabular numerals.

#### 15.5.2 One dominant element per control

A slot, card or tile shows one primary reading (an ability's art, a card's
art and name) and at most one centred numeral. Secondary values (a cost, a
charge count, a stack, a hotkey) are small, in a fixed corner or a label
outside the art, never two at the same size; a hotkey is a quiet label,
never a boxed cap over the art. One primary action per layer, in the
style's accent and material, larger or brighter than its secondaries.

#### 15.5.3 Real art over placeholder glyphs

Every slot that shows a thing (ability, item, buff, card, portrait,
character, enemy, banner, level, backdrop) shows painted or rendered art
from the art pipeline (§13), lit, saturated, filling its slot. Line icons
are for system chrome only (close, back, settings, search, sort,
chevrons). A kit or reference frame showing a placeholder silhouette, a
letter in an empty slot, a star glyph as card art or a flat-colour wash as
key art falls short; an empty equipment slot shows its kind's faint
silhouette, not a letter. Games supply their own painted art; the reusable
UI packs supply controls and styles, not a gallery's inventory of art.

#### 15.5.4 Numerals

Gameplay numerals (cooldowns, damage, health, prices, counts) have a
1–2 px dark outline or shadow, or sit on a scrim, so they read over any
art. Tabular figures in columns and counters (a type step's OpenType
features: small caps, tabular and lining figures, case-sensitive forms,
`crates/og-ui/src/shape.rs`). Units and secondary parts (`/ 100`, `%`,
`km/h`) are a step smaller than the value. No numeral is coloured by state
unless that colour is the state (a red cost when unaffordable, a green
delta for a gain).

#### 15.5.5 Selection and focus

Selected, focused, hovered and pressed are distinct and read on art: an
accent glow (6–8 px at 1080p) or an accent rim with a lift (shadow, scale
1.02–1.05). A pure white 1–2 px outline or a white fill as the selection,
or flooding the selected item's art with a colour, falls short. Disabled
desaturates and darkens the art (about 35 % value) and shows why (a lock,
a slash, a reason line).

#### 15.5.6 Depth and material

At least three elevations read by value and light: the backdrop, the
panel, the control (and a floating layer for menus and tooltips). Each
surface has the style's material: a lit edge, an inner shadow on wells and
sockets, a drop shadow on raised parts, a gradient or texture where the
style has one. No flat grey rectangle with no edge as a panel, button or
socket; no translucent grey box behind a HUD group: HUD pieces float on
their own shadows.

#### 15.5.7 Spacing rhythm

Gaps and paddings come from the style's space scale (a 4 or 8 px base),
at most four steps of it per screen. Labels sit 4–6 px from what they
label; groups are separated by at least twice the gap inside them. A
screen does not leave more than a third of its area empty around a small
centred card.

#### 15.5.8 Motion stills

Every mid-transition frame is a composed frame (no half-faded text over a
hard edge, no element snapped while its neighbour eases, no flash of an
unstyled state). A key moment (a reveal, a level up, a ready flash) has a
frame that would work as a screenshot of the game.

## 16. Agent tooling: read, write, preview, check

### 16.1 Read

The component catalogue is the `components` guide (`og api show
components`): every element kind a document can use, one section each,
with its props as the document writes them (the CSS property and the
value it takes, or the attribute) and its states. `og api gen` writes it
from og-ui's document schema (`crates/og-api/guides/components.md`) and
`og api check` holds it equal, so `og api find "toggle" --kind guide`
answers with the kind's own section and what a document may use is
exactly what the parser accepts.

`dump_ui` gives per element: the resolved props (box, transform, paints by
token), the style part it is drawn as (`part`, with its variant and state
flags) and the style rules that dress it (`rules`: each selector with its
file and line, in the order they apply), the state of each group, bindings
and their values, animations in flight with their progress, slot specs,
the layer it belongs to, the document line that built it (`source`), and
whether it was rebuilt this tick. The probe's `pickUi(x, y)` (wire member
`pick_ui`) answers the topmost element under the point that shows
something or takes input, with its id, kind, text, rectangle, `source`,
`part` and `rules`; a point on a layout container that draws nothing has
none. A tool asks it first, then `pick` for the scene, whose `source` names
the scene line that declares the entity and whose `ui` carries the same
element. `dump_ui:layers`
lists the layer stack with input modes and focus. `dump_ui:motion:<path>`
(a transition or timeline sampled over ticks) is not implemented.

### 16.2 Write

Probe commands, for tests and live adjustment:

- `ui:set:<path>.<prop>=<value>`: set any prop (presentation only; marked
  as a probe edit in the dump);
- `ui:state:<path>.<group>=<state>`: force a state to render and check it;
- `ui:play:<path>.<timeline>`, `ui:seek:<path>.<timeline>=<t>`;
- `ui:push:<screen>` and `ui:pop` with a viewmodel fixture.

### 16.3 Tune and hot reload

Documents and style tokens are reflected, so Tune edits them while the
game runs, and saving writes the value back into the `.ogt` that declared
it (as `save_scene` does for scenes). `og dev` applies an edited document
or style in place, keeping the UI state and the world, within a 300 ms
reload budget.

Loaded `type ui-material` assets also refresh in place. Editing their
`source` WGSL or `param` defaults keeps the material handle and replaces
the cooked shader and defaults after the manifest refresh; the renderer
invalidates its pipeline when the shader or parameter declaration changes.
A failed cook leaves the previous material running. Custom materials in
`dress` blocks resolve relative to the style file, are included in its
cooked dependencies, and bind when the style is configured; reloading
rebinds their defaults and preserves the style's part dressings.

`dump_ui.materials` lists each loaded asset's parameter names, WGSL types,
defaults and current values from the cooked material declaration.
`ui:material:effect.ogt.amount=0.6` changes the default uniform without a
cook or shader compilation. Vectors take space-separated numbers; an empty
value resets that parameter to its file default. Explicit per-layer
parameters take precedence. These are session edits: a changed material
reloads its file defaults, and `ui:save` does not write material edits.
Persist a chosen value in the material's `param` line (or the style's
explicit material parameters).

### 16.4 Preview

`og ui preview <screen|component>` renders a contact sheet: styles ×
breakpoints (desktop, phone portrait and landscape, tablet) × locales (the
game's, pseudo, CJK, RTL) × states × text scale, from viewmodel fixtures
(`fixtures/*.ogt`), with a JSON index of what each cell shows. `--film
<path>.<timeline|transition>` renders a filmstrip of a motion at chosen
ticks. These are what an agent reads before it claims a UI is done.

### 16.5 Check

`og ui check` (also available to `og test`) reports facts, never verdicts.
It exits 0 when it ran, whatever it found, and 1 only when it could not
run. Its **findings**, each with the element paths and the prop that sets
it:

- overlap not declared (§6.2), across breakpoints and locales;
- text clipped or truncated without ellipsis and a tooltip; overflow under
  the pseudo-locale and at 150 % text;
- elements outside the safe area or under the shell's covered region;
- focus: an interactive element unreachable by pad from the default
  focus, a focus trap, a layer without Back;
- accessibility names missing, or the same in another locale;
- feedback: an interactive element without hover or press states, or
  without a sound in a styled game;
- motion: durations outside the style's bounds, missing reduced-motion
  fallbacks, input refused longer than the style's entrance;
- art: slots still on their placeholder. Images whose focal point falls
  outside the slot's crop are not checked.

Its **measures**, numbers held to no bar (the `screens` guide, `og api
show screens`, says what a game usually wants):

- each live text's contrast on the drawn glyphs (dimmed, decorative and
  over-the-scene text is not measured);
- each text's size, and at 150 % and 200 % text the percentage of its
  100 % size it is drawn at;
- each control's shorter side on a coarse pointer;
- the buttons filled with the primary, per layer;
- the element count and draw commands per frame.

`og ui literals [path...]` lists the style values a kit or document writes
itself instead of taking from the style: a `#rrggbb` or `{ r, g, b }`
colour, a corner radius other than 0, a font family that is no font role.
Each comes with its file, line and the token to use. A kit holds structure
and behaviour; a value it writes is one no style pack can change. A line
holding `style-literal-ok` is skipped, for a value that is not the style's.
Like the check it reports and exits 0 when it ran.

### 16.6 Review

`og ui review <kit> [--style <pack>]... [--states a,b]` renders what an art
review reads, in one pass: every page of the kit's review sheet
(`sheet/project.ogt`, §17.3; a screen's `style:` step is the style axis)
× each state fixture (`state:<name>`, which the sheet's script puts the
page in) × each style (`--style`, else every style the sheet names), at
1920×1080 and on a phone portrait (390×844) and landscape (844×390), 100 %
and 150 % text, in the game's locale, Chinese, Arabic and the
pseudo-locale. It writes a contact sheet per page, state and style
(`review-<page>-<state>-<style>.png`: a row per device and scale, a column
per locale, each picture labelled), `review.json` with every cell and
measure, and `review.txt` with each measure's count, minimum, median and
maximum. The scratch project lives in the worktree's `.og/review/<kit>`,
so repeated reviews share one resident host.

It measures and never judges. Beside §16.5's measures: text drawn past
its box (`overflow`, CSS px), text its clip cuts (`clip`, CSS px²), each
element's farthest edge from the grid (`grid`, CSS px, `--grid` default
4) and the gaps between the children of each row and column (`gap`: how
many distinct gaps, with their values).

## 17. Extensions

The core owns elements, layout, motion, drawing, documents, style grammar
and common components. Genre kits and style packs are directories, not
host plugins. They use only the public TypeScript API and the document
and style formats, and their versions are independent of the engine
version. Their suites run through `scripts/check.sh kits` (`ui` is an
alias), separately from the engine's quick and functional lanes. Contrast
is not an acceptance check.

### 17.1 Manifest and installation

An extension has an `extension.json` at its root:

```json
{
  "format": 1,
  "kind": "kit",
  "name": "racing",
  "version": "1.0.0",
  "description": "Race HUDs and results, built from common components",
  "entry": "index.ts",
  "types": "index.d.ts"
}
```

`kind` is `kit`, `style` or `vfx` (an effect pack: `type vfx` effects with
the emitters, effect materials and textures they name). Names use lowercase ASCII letters, digits and
hyphens, starting with a letter. `format` versions this manifest contract;
`version` versions the extension. All paths are relative to the directory,
without parent traversal or symlinks. Official packs carry no skill: how
to use, change and review them is the `kits`, `ui-materials` and `screens`
guides (`og api show kits`). An optional `skill` path is still
validated and printed for a game's own pack. `entry`
and `types` are optional together for a document-only kit. A style has
`style.ogt` at its root and no executable entry; neither has a vfx pack. Documents are discovered
recursively by their `type screen` or `type component` header, rather than
listed in a shared registry. Asset references within a pack are relative;
shared built-in fonts may still use `@styles/`.

Official kits live in `packages/engine/extensions/<name>` and ship there.
Official styles stay in `templates/styles/<name>` and ship in the engine's
`styles/<name>`. Both use the same manifest. A game installs an editable
copy with `og ui add <name|directory> [project]` or
`og style add <name|directory> [project]`. Kits go in
`assets/ui-kits/<name>`, styles in `assets/styles/<name>`; effect packs
install with `og vfx add <name|directory> [project]` into
`assets/vfx/<name>`, where their effects name each other as
`vfx/<name>/<file>`. An existing
installation is never silently replaced. No network fetch or code execution
is part of discovery or installation.

`og ui list [project]` lists installed and bundled extensions, their
versions, entry/type paths and every document. The installed copy
wins for a kind/name pair. `og ui eject <kit>/<document> [project]` copies
an extension document to `assets/ui/<kit>/<document>.ogt`, alongside its
companion documents and assets so relative references survive (§17.5).
Existing companion edits are preserved; `--force` replaces only the
requested document. Library screen names (§11.1) are still accepted. The
installed documents are ordinary assets:
`og.load.document(world, "ui-kits/racing/results.ogt")` uses the same
cooked document on both hosts, with no runtime filesystem discovery. A
look's `ui "fantasy"` finds the game's installed
`assets/styles/fantasy/style.ogt` before the bundled pack. Explicit asset
paths remain valid. Installation preserves the directory tree and licences.

### 17.2 Typed TypeScript kits beside the frozen core

Kits import `registerUiExtension` from
`@origingame/engine/ui-extensions` and export its result:

```ts
export const racing = registerUiExtension("racing", { speedometer, results });
```

Registration returns a shallow-frozen, inferred component namespace; the
kit's own declarations describe its functions. It never mutates `og.ui`,
adds a host binding or evaluates a manifest's entry automatically. The
game explicitly imports the installed kit's entry. Re-importing during hot
reload may register a fresh object: each importer retains its own namespace,
never a global function table that another registration can overwrite.
The registry is one TypeScript implementation shared by native and web.
Components keep state in the engine's public world/UI memory APIs, not
module globals, so snapshots and multiple worlds retain their meaning.

### 17.3 Pack authoring and review

`style.ogt` is authored data. Each pack's choices file is local to that
pack; `og style new` only bootstraps a new style, never regenerates the
built-in collection. Adding a pack or kit requires no shared name list.

`og ui sheet <pack|kit> [project] [--out DIR]` renders a pack's or kit's
baseline pages on demand; it is not a gate step. It discovers extensions
in the project (default `.`), including installed edits. Packs use the
engine's shipped baseline project and its fixed component/pattern/HUD
pages. A kit supplies `sheet/project.ogt` inside its own directory: its
`ui { screen "name" "commands" }` entries are its fixed review pages,
whose commands must establish each page independently, on the kit's style
ground rather than a game scene. No kit name or page list is registered in
the core. The tool copies its inputs to disposable working files, cooks
and bundles them, and renders four cells per page: desktop and phone at
100 % and 150 % text. It writes full-resolution frames, UI dumps, contact
sheets and `index.json`. Missing captures or missing baseline page roots
fail the command; visual judgment belongs to the reviewer (§15.4), with no
contrast score or reference gate.

### 17.4 Dressing and typography contracts

Painted art is addressed by **selector**, not by an engine slot enum.
An `art-requests.json` slot with a new name supplies `size`, `insets` and
`dress: ["Tabs.tab[selected]"]`; `kind: "styleFrame"` packs a nine-slice,
`"ornament"` a whole piece, and `"styleButton"` a five-cell state sheet.
`tint: true` on a frame keeps the widget's colour instead of baking it
into the art. The standard slot names remain defaults; `bar` is an
opt-in tinted `Meter.fill`. Tabs, selected rows, ribbons and banners need
no new core registration.

A paint colour can carry `alpha <0..1>`, including a palette or custom
role: `fill linear 90 primary alpha 0.25 0 accent alpha 0.8 1`.
The alpha belongs to the preceding colour; the next number is still the
gradient stop position. Literal alpha multiplies the literal's own alpha.
Tokens remain symbolic until drawing, so scheme changes and state
transitions keep their meaning.

`ornamentTint <colour>` on a dressing overrides its corners, crest,
edges, divider and heading flourishes, including procedural corners.
It does not recolour the atlas or other components using the same art.

The palette separates `selection`/`onSelection` (selected rows and tabs),
`primary`/`onPrimary` (the main action), and `accent` (small emphasis).
Files without them derive selection from controlStrong, onSelection from
ink and accent from highlight. `sceneInk`, `scenePlate` and `sceneShadow`
are scheme-aware HUD roles (fallback: ink, hud and shadow). Text drawn
straight over the scene passes `onScene` (`LabelDesc.onScene`,
`og.ui.label(..., { onScene: true })`, the ui-parts `text()` and `btn()`):
ink roles become `sceneInk` at the same emphasis, with `sceneShadow`
separation. A HUD container with `background: { token: "scenePlate" }` is
the plate treatment. Games tune these three roles in their style, not per
widget.

`og.ui.icon(id, "base", { secondary: "detail", secondaryTint: { token:
"accent" } })` overlays two named icons with independent inks. The
secondary tint defaults to accent; both layers share the icon's bounds.
Document icons accept the same properties.

The stacking order is fixed for authored draws and style dressings:
ground/fills → rim → content/selection → hollow focus → children/raised
badges. Focus never covers art with a background-coloured cut. Put a
decoration that must cover children in a child of its own, not a rim.

`typography` is the drawing hierarchy, including weight, tracking and
breakpoint sizes. The ramp names are aliases, not a second set of
authored numbers:

| Typography role | Ramp names |
| --- | --- |
| caption | xs, caption |
| label | sm, meta |
| body | base, body, reading |
| numeric | lg |
| heading | xl, xl2 |
| title | xl3, xl4, displaySm, displayMd |
| display | xl5, xl6, displayLg |
| hero | xl7, xl8, displayXl, displayXl2 |

`hero` is optional; without it, it is 1.5 times `display` at each
breakpoint, up to the format's 400 px limit. The player's text scale
multiplies after selecting the breakpoint, once. Buttons use `button`
(falling back to `label`).

Type roles accept `cjkTracking <em>` and `case upper` after the font and
OpenType tags, for example `label 16 22 0.06 600 body case upper`.
`case none` leaves the source unchanged; a bare `case` remains the
OpenType case-sensitive-forms tag. Uppercase uses ICU's full Unicode
mapping in the active `og.i18n` locale before measuring and wrapping.
It affects displayed labels and rich text, not probe/accessibility source
strings, editable values, cursor offsets or inline icons and key caps.
Greek accents, Turkic dotted I and Lithuanian combining dots therefore
follow their language, on both hosts.

`og upgrade`'s `ui-typography` codemod converts v1 styles with the style
upgrader. For v2 it comments out a superseded `ramp`, keeping its old
numbers for review and preserving other source bytes, and lists
`typography` for manual visual review: two old aliases with different
sizes cannot both keep those sizes when they name one role.

### 17.5 The genre kits

Genre components are not in `og.ui`; they are bundled kits under
`packages/engine/extensions/`. Each kit's `index.ts` registers one typed
namespace, and its `types.ts` holds its data types:

| Kit | Members |
|---|---|
| `rpg` | `inventoryGrid`, `paperdoll`, `questLog`, `questTracker`, `conversation`, `unitFrame`, `partyFrames`, `raidFrames`, `castBar`, `turnOrder`, `worldMap` |
| `shooter` | `crosshair`, `ammoCounter`, `killfeed`, `damageDirection`, `lowHealthVignette`, `scoreboard`, `loadout` |
| `puzzle` | `sagaMap`, `goalChips` |
| `gacha` | `gachaBanner`, `pullSequence` |
| `strategy` | `productionQueue`, `selectionPanel`, `commandCard`, `rtsConsole`, `dragSelect`, `rtsCommands`, `wavePreview`, `resourceBar`, `ledger` |
| `racing` | `speedometer`, `standingsTower`, `lapTimer`, `rearView`, `raceBanner`, `itemSlot`, `driftScore`, `garage` |
| `card` | `cardHand`, `rewardChoice`, `deckBuilder` |
| `idle` | `idleGenerator`, `prestigePanel` |
| `liveops` | `storeFront`, `battlePass` |
| `story` | `backlog`, `barks`, `cgGallery`, `routeChart` |
| `versus` | `lobby`, `characterSelect`, `versus`, `draft`, `chat` |
| `rhythm` | `songSelect`, `laneKeys`, `comboCounter`, `judgement` |
| `photo` | `photoMode` |
| `sports` | `scorebug`, `matchStats`, `goalTimeline`, `playerPlate`, `substitution` |
| `flight` | `flightHud` |

`materials` is a kit of UI material presets (§5.4). Flight and photo are
optional kits, not dependencies of every racing or action game. Generic
lists, slots, meters, timers (`hudTimer`), stars (`starBar`), counters,
stores, rewards, cards, world tags, `heroButton`, `settingsTree`,
`overlayBar`, ability slots and bars, status grids, navigation and the
Rust meter/cooldown/minimap/zoom-view/joystick primitives stay in `og.ui`.
A game imports a kit explicitly, from the bundled copy or from its
installed copy after `og ui add <kit>`; types come from the same entry:

```ts
import { racing, type StandingsEntry } from "@origingame/engine/extensions/racing/index.ts";
import { racing } from "../assets/ui-kits/racing/index.ts";
```

Kit code imports only public modules: `@origingame/engine` (og and its
types), `@origingame/engine/ui-extensions`, and the authoring helpers in
`@origingame/engine/ui-parts/hud` (HUD placement in the safe area, parts
a style dresses, the frame's tokens and roles, motion by the tick) and
`@origingame/engine/ui-parts/screen` (surfaces, buttons, symbols, art
slots, price tags, a phone's room). The ui-parts modules only re-export
helpers the common components already share; their names and signatures
are a contract within a minor version, and changing one is a breaking
changelog entry. Every helper is built from `og.ui`'s public members and
keeps its state in the UI's memory, so a kit runs unchanged on both hosts.

`og upgrade`'s `ui-kits` codemod moves a game onto the kits. Its
changelog entry lists each kit's members and types, so a later kit needs
no edit to the codemod. It rewrites `og.ui.<member>` to `<kit>.<member>`,
moves the kit's types out of `@origingame/engine` imports and
`Api.<Type>` uses, and adds one import per kit. An existing import of
the kit, bundled or installed, is kept. A call through another alias of
`og.ui`, or a file where the kit's name is already bound, is listed for a
manual edit. A second run changes nothing.

A kit's composed screens are documents, so a game changes their layout or
adds a field in text instead of copying a fixed-field function. The racing
kit's `race-results.ogt` (`type screen`, name `race-results`) holds the
place, the best lap and points, the classification as a `for row in rows`
block and the rewards, emits `continue`, `restart` and `replay`, and
leaves a `Podium` slot for the game's own podium. A game loads it from
its installed kit (`og.load.document(world,
"ui-kits/racing/race-results.ogt")`) or ejects it alone (`og ui eject
racing/race-results`, loaded as `ui/racing/race-results.ogt`). Eject
copies the document's companions, the other documents and assets of the
kit, never the kit's TypeScript, manifest or review sheet: those
come with `og ui add`. `og ui sheet <kit>` installs the kit into its
working copy of the sheet, so a page loads the kit's documents from
`ui-kits/<kit>/` exactly as a game does.

Data a genre varies is open. A gacha with its own tiers (SSR, SR, R, or a
ladder of ten) gives a featured unit or a pull item a `tier` (`{ label,
rank, color }`) over the engine's `Rarity`: its colour tints the reveal,
its label names it, and its rank orders the summary and, from 5, shines
and bursts as legendary does. Scoreboards take any number of teams, a
resource bar any resources, and a standings tower any field.

---

## Appendix A. Genres and what they need

Codes are used in the other appendices. Each row lists the parts of this
design the genre leans on most; every genre also uses the foundations,
layers, focus and styles.

| Code | Genre (exemplars) | Leans on |
|---|---|---|
| PLT | Platformer (Celeste, Hollow Knight, Ori) | pip meters, world and level maps, loadout grid, death-to-retry covers, hit-stop with UI punch |
| RUN | Runner (Subway Surfers, Alto) | big counters, power-up rings, revive countdown, shops, missions, swipe gestures |
| TPA | Third-person action (God of War, Spider-Man) | compass, context prompts with glyphs, skill trees (pan and zoom), gear compare, photo mode, viewport turntables |
| SHO | Shooter (Destiny 2, Valorant, CoD) | crosshair editor, hit markers, killfeed, damage direction, weapon and ping wheels, gunsmith, scoreboard, battle pass |
| BR | Battle royale (Fortnite, Apex, PUBG) | circle timers, squad frames, armor cells, pings with distance, loot lists with compare, drop map, item shop, HUD layout editor |
| MOBA | MOBA and arena (LoL, Dota 2, Pokémon Unite) | ability bars with cooldown sweeps, minimap with pings, 100+ overhead bars, shop with build paths, draft and ban with timers, post-game graphs |
| JRPG | JRPG (Persona 5, FF VII Remake) | nested command menus with focus memory, turn order, party panels, stylised screen transitions, EXP cascades |
| ARPG | Action RPG (Diablo IV, PoE, Hades) | multi-cell grid inventory, compare tooltips, passive trees of 1,000+ nodes, thousands of damage numbers, loot labels with filters |
| CRPG | CRPG (Baldur's Gate 3, Pathfinder) | hotbars, initiative, dice-roll feedback, path cost previews, nested hyperlinked tooltips, dialogue with checks |
| MMO | MMO (WoW, FFXIV, GW2) | 40 raid frames, buff grids, cast bars, chat with links and IME, auction house tables, movable HUD, nameplate declutter |
| RGL | Roguelike and roguelite (Hades, StS, Vampire Survivors) | run maps, pick one of three with rarity, synergy tooltips, run summaries, mass world elements |
| TD | Tower defense (Bloons TD 6, Kingdom Rush) | placement ghosts, range circles, radial upgrades, wave banners, overhead bars in the hundreds |
| RTS | Real-time strategy (StarCraft II, AoE IV) | box select, control groups, command card with hotkeys, production queues, 200–400 overhead bars, post-match graphs |
| 4X | 4X and grand strategy (Civ VI, Stellaris, CK3) | huge tech trees, pinnable nested tooltips, sortable ledgers, notification stacks, map labels with level of detail, diplomacy |
| CITY | City and management (Cities: Skylines, Factorio) | build toolbars, overlays, budget graphs, status icons on thousands of agents, placement with snapping |
| IDLE | Idle and incremental (Cookie Clicker, Melvor) | numbers past 1e308 with notations, buy multipliers, hold to repeat, prestige sequences, long lists |
| M3 | Match-3 and puzzle (Royal Match, Baba Is You) | saga maps of thousands of nodes, goals and moves, boosters, cascade feedback, hint shimmer, confetti |
| CARD | Card and deckbuilder (StS, Hearthstone, Balatro) | card hand fan, drag to play with a targeting arc, hover lift and tilt, score cascades, collections with filters, pack opening |
| GACHA | Gacha and collection (Genshin, Star Rail, Arknights) | banners, pull sequences with rarity teases, roster grids, ascension checklists, events hub, mailbox, login calendar, layered menu backdrops |
| RHY | Rhythm (osu!, Beat Saber, Project Sekai) | judgement pops, combo, song select with preview, calibration, results breakdowns, beat-synced UI |
| RACE | Racing (Forza Horizon, Mario Kart) | speedometer and tach, position and lap deltas, track map, garage with viewport, tuning sliders with graphs, livery layers |
| SPT | Sports (EA FC, NBA 2K, Rocket League) | scorebug, shot meters, formation drag, play calling, pack walkouts, market tables |
| FGT | Fighting (Street Fighter 6, Tekken 8) | twin bars with recoverable damage, round pips, character select with simultaneous cursors, versus screens, input display |
| SURV | Survival and crafting (Minecraft, Valheim, Subnautica) | hotbars, crafting grids and recipe books, containers with quick transfer, build radials with snapping |
| HOR | Horror (Dead Space, RE, Alien: Isolation) | diegetic world panels, minimal HUD, item inspect in a viewport, meta UI distortion |
| VN | Narrative and visual novel (Ace Attorney, Disco Elysium) | text box with typewriter, backlog, save grids with thumbnails, galleries, route flowcharts, timed choices, sprite expressions |
| OW | Open world (Elden Ring, BotW, Witcher 3) | world maps with fog, filters and pins, compass with POIs, quest tracker, waypoints with edge clamping, item wheels |
| PARTY | Party and social (Mario Party, Among Us, Fall Guys) | lobbies, vote screens, emote wheels, podiums, bouncy motion, up to 60 players |
| LIFE | Farming and life sim (Stardew, Animal Crossing, The Sims) | calendars, relationship hearts, build and buy catalogues with search, placement on grids, pattern editors |
| CAS | Casual, hyper-casual, educational (Duolingo, Wordle) | streaks, lesson paths, leagues, one-thumb gestures, strong juice |
| TAC | Tactics (XCOM 2, Fire Emblem, Into the Breach) | move and threat overlays, hit chance and attack forecasts, intent icons, undo |
| AUTO | Auto battler (TFT, Super Auto Pets) | bench and board drag, shop rolls, trait thresholds, merge feedback |
| FLY | Flight and space sim (MSFS, Elite Dangerous) | pitch ladder, target boxes and lead indicators, 3D nav maps, cockpit world panels |
| SBX | Sandbox and UGC (Roblox, Dreams) | property inspectors, hierarchies, gizmos, avatar editors, docking |
| BRD | Board, casino and word (chess.com, Mahjong Soul, poker) | legal-move highlights, chip sliders, tile racks, clocks, move lists |

## Appendix B. Screens

| Screen | Genres |
|---|---|
| Boot, splash, title, main menu, mode select | all |
| Settings (graphics, audio, controls and remapping, accessibility, language, HUD layout editor) | all; HUD editor: MMO, SHO, BR, MOBA |
| Pause and system menu | all real-time |
| Save and load with thumbnails | JRPG, CRPG, VN, OW, SURV, LIFE, 4X, CITY |
| Loading with tips, lore and progress | SHO, BR, MOBA, OW, MMO, RPG |
| Character creation (sliders, presets, turntable) | CRPG, MMO, OW, LIFE, SBX, SPT |
| Character, loadout, paperdoll and stats | TPA, ARPG, CRPG, JRPG, MMO, SHO, BR, GACHA |
| Inventory (grid, list, weighted, nested containers) | ARPG, CRPG, MMO, SURV, HOR, OW, LIFE, BR |
| Stash, storage, bank | ARPG, MMO, SURV, LIFE |
| Skill, talent, passive and tech trees | TPA, ARPG, CRPG, MMO, RGL, 4X, RTS, GACHA |
| Crafting and recipe book | SURV, ARPG, MMO, LIFE, OW, TPA |
| Shop and vendor in the world | most |
| Store (bundles, rotations) | BR, GACHA, MOBA, SHO, CAS, M3, SPT, CARD |
| Gacha banner and pull results; pack opening | GACHA, CARD, SPT |
| Battle pass and season track | BR, SHO, MOBA, GACHA, CARD, PARTY |
| Daily login, missions, events hub | GACHA, M3, CAS, RUN, IDLE, BR |
| Mailbox | GACHA, MMO, CARD, M3 |
| World map (fog, markers, fast travel) | OW, TPA, PLT, MMO, SURV, RPG, 4X |
| Level and saga map | M3, PLT, TD, CAS, RUN |
| Run map (branching nodes) | RGL, CARD |
| Reward choice (pick N of M) | RGL, CARD, AUTO |
| Quest log, journal, codex, bestiary, encyclopedia | RPG, MMO, OW, TPA, 4X, RGL |
| Dialogue (choices, checks, timers) | CRPG, JRPG, VN, OW, TPA, MMO |
| Backlog, gallery, route flowchart | VN, JRPG |
| Deck builder and collection | CARD, GACHA, AUTO |
| Roster, team builder, character detail and ascension | GACHA, TAC, SPT, JRPG, FGT |
| Character select and versus | FGT, MOBA, PARTY, SHO |
| Draft and ban with timers | MOBA, CARD, SPT |
| Lobby, party, matchmaking, ready check | SHO, BR, MOBA, PARTY, MMO, RACE, FGT, AUTO, BRD |
| Scoreboard | SHO, BR, MOBA, SPT, RACE, PARTY |
| Results and post-match (grades, XP cascades, graphs) | most; graphs: RTS, MOBA, 4X |
| Leaderboards (global, friends, leagues) | RHY, RACE, IDLE, CAS, RUN, M3 |
| Song and track select with preview | RHY, RACE |
| Garage, tuning, livery editor | RACE, FLY |
| Management dashboards (ledgers, budgets, graphs, overlays) | CITY, 4X, SPT, IDLE |
| Diplomacy and trade | 4X, MMO |
| Auction house and market | MMO, SPT, SURV |
| Guild, friends, chat | MMO, SHO, BR, GACHA, PARTY, SBX |
| Training mode with frame data | FGT |
| Calibration (audio offset, brightness, controller) | RHY, HOR, all |
| Photo mode | OW, TPA, RACE, LIFE |
| Replay and spectator | RTS, FGT, MOBA, RACE, SPT, BR |
| Prestige and meta-progression hub | IDLE, RGL |
| Tutorial and onboarding overlays | all; heavy in CAS, GACHA, M3 |
| Death, retry, revive offer, game over | PLT, RUN, RGL, CAS, HOR |
| Creator tools (inspector, hierarchy, catalogues) | SBX, CITY |
| Credits | all |

## Appendix C. HUD and world elements

| Element | Genres |
|---|---|
| Health and resource bars (segmented, ghost damage, shield cells) | all action; FGT, BR |
| Orbs, radial stamina meters | ARPG, OW, SURV |
| Ammo and magazine | SHO, BR, HOR, TPA |
| Crosshair, spread, hit markers | SHO, BR, TPA, FLY |
| Ability bar with cooldown sweep, charges, hotkeys | MOBA, MMO, ARPG, SHO, CRPG |
| Buff and debuff grid with timers and stacks | MMO, ARPG, MOBA, CRPG, RGL |
| Unit, target, party and raid frames | MMO, CRPG, JRPG, MOBA, BR |
| Cast and channel bars | MMO, SURV, ARPG |
| Turn order, initiative, ATB | JRPG, CRPG, TAC |
| Minimap (rotating or fixed, fog, pings, click to move) | MOBA, RTS, SHO, OW, RACE, MMO |
| Compass strip with POIs and distance | OW, BR, SHO, SURV |
| Objective tracker | OW, MMO, TPA, RPG |
| Killfeed, notifications, achievement toasts | SHO, BR, MOBA, all |
| Chat window | MMO, SHO, BR, MOBA, PARTY, SBX |
| Score, combo, multiplier, judgement | RHY, FGT, M3, RUN, CARD, IDLE |
| Timers (round, circle, turn, choice) | FGT, BR, MOBA, VN, PARTY, BRD |
| Resource bar with rates | RTS, 4X, CITY, IDLE |
| Selection panel, command card, production queue | RTS, CITY |
| Hotbar and toolbar | SURV, LIFE, MMO, SBX |
| Hand, piles, mana | CARD, AUTO |
| Speedometer, tach, delta, position | RACE |
| Scorebug, shot meters | SPT |
| Flight HUD (ladder, target box, lock) | FLY |
| Text box with name plate, auto, skip | VN, JRPG, CRPG |
| Context prompts with device glyphs | all |
| Damage direction, low-health vignette, meta effects | SHO, BR, HOR, TPA |
| World: nameplates with LOD, occlusion and declutter | MMO, SHO, BR, MOBA, PARTY |
| World: overhead bars at scale | RTS, MOBA, TD, ARPG, AUTO, CITY |
| World: damage numbers | ARPG, MMO, RGL, SHO, JRPG, CRPG |
| World: interaction prompts | OW, TPA, SURV, HOR, LIFE |
| World: waypoints and pings with edge clamping | OW, SHO, BR, MOBA, RACE, FLY |
| World: placement ghosts with validity, snapping and cost | RTS, CITY, TD, SURV, LIFE, SBX |
| World: range, area and skillshot telegraphs, path lines | MOBA, TD, CRPG, TAC, MMO, RTS |
| World: selection rings, rally and queue lines | RTS, CRPG, TAC |
| World: loot labels and beams with filters | ARPG, BR, MMO |
| World: status icons on agents | CITY, LIFE, RTS |
| World: map labels on huge maps | 4X, OW |
| World: diegetic panels | HOR, FLY, SBX |
| World: speech and emote bubbles | PARTY, SBX, MMO, LIFE |

## Appendix D. Interactions

| Pattern | Genres |
|---|---|
| Pad focus navigation (spatial, memory, bumpers) | all with pads |
| Drag and drop (items, cards, units, formations, teams) | ARPG, CRPG, SURV, CARD, AUTO, GACHA, SPT, LIFE |
| Grid inventory with rotate, split, stack, quick transfer | ARPG, SURV, HOR, BR |
| Compare tooltips with stat deltas | ARPG, TPA, SHO, MMO, RPG |
| Nested, pinnable tooltips with keyword links | 4X, CRPG, CARD, ARPG, MMO |
| Drag to play with a targeting arc | CARD, TAC, MOBA |
| Box select, select same type, control groups, shift queue | RTS, CITY, TAC |
| Radial wheels (stick or mouse, with slowdown) | SHO, BR, OW, TPA, SURV, MOBA, PARTY |
| Pan and zoom canvases | ARPG, 4X, OW, MMO, VN, CITY |
| Painting and drag placement with snapping | CITY, SURV, SBX, LIFE, TD |
| Pick one of N, reroll, banish | RGL, CARD, AUTO |
| Timed choices, QTEs, hold to confirm | VN, TPA, SHO, PARTY |
| Sortable, filterable virtualised tables and grids | MMO, 4X, CITY, GACHA, CARD, SPT, BRD |
| Search with text input (IME, on-screen keyboard) | MMO, MOBA, CARD, GACHA, LIFE |
| Sliders and curves | RACE, CRPG, MMO, RHY |
| Layered editors (livery, pattern, avatar) | RACE, LIFE, SBX, SHO |
| Touch gestures and customisable virtual controls | CAS, M3, RUN, MOBA, BR |
| Keybinding and HUD layout customisation | MMO, BR, SHO, MOBA |
| Simultaneous local cursors | FGT, PARTY |
| Chat links, mentions, channels | MMO, BRD |
| Undo and rewind | TAC, M3, RACE, VN |
| Confirm and cancel modal stacks, purchase confirmation | all; heavy in GACHA and stores |
| Mass actions (buy max, sell junk, auto sort) | IDLE, ARPG, SURV, GACHA |

## Appendix E. Motion and feedback

| Pattern | Genres |
|---|---|
| Hit-stop and shake tied to a UI punch | PLT, FGT, ARPG, RGL, SHO |
| Count-ups, rolling digits, escalating cascades | CARD, IDLE, SHO, JRPG, LIFE, SPT |
| Card physics: hover lift, tilt to the pointer, wobble, fan relayout | CARD, AUTO, BRD |
| Rarity reveal sequences (tease, burst, skip) | GACHA, CARD, SPT, BR, ARPG |
| Stylised screen transitions and stingers | JRPG, SPT, FGT, PARTY |
| Judgement pops and combo milestones on the beat | RHY, FGT, M3 |
| Cooldown sweep and ready flash | MOBA, MMO, ARPG, SHO |
| Toast and notification queues | all |
| Celebrations (confetti, banners, podium, level up) | CAS, M3, PARTY, BR, RPG |
| Elastic press feedback | CAS, M3, PARTY, GACHA |
| Map and fog reveals, region title cards | OW, 4X, RGL |
| Warning pulses | SHO, BR, RTS, SURV, HOR |
| Distortion and diegetic glitch | HOR, SHO |
| Typewriter text, sprite shake and fade | VN, JRPG |
| Idle hint shimmer | M3, CAS |
| Parallax and animated menu backdrops | GACHA, JRPG, FGT |
| Beat-synced pulses | RHY |
| Damage number styling (crit scale, type colour, merging) | ARPG, MMO, RGL, SHO |
| Layout animation of lists, hands and grids | CARD, ARPG, GACHA, all lists |
| Shared-element transitions between screens | GACHA, SHO, RACE, all stores |

## Appendix F. Art slot kinds

Aspects are starting values for the brief.

| Kind | Aspect | Genres |
|---|---|---|
| Key art and title background | 16:9 with safe extension to 21:9; 9:16 portrait | all |
| Store capsules | about 2.14:1 header, 2:3 vertical, about 3.1:1 hero, transparent logo | all shipped |
| App icon and game tile | 1:1 | all |
| Character splash and full body | 1:2 to 9:16; 16:9 splashes | GACHA, MOBA, FGT, VN, JRPG, SHO |
| Portraits and busts, expression sets | 1:1 or 3:4 | RPG, VN, TAC, MMO, MOBA, AUTO, GACHA |
| VN sprites (layered, transparent) | about 1:2 | VN, JRPG, CRPG |
| Event CG | 16:9 | VN, GACHA, JRPG |
| Item and equipment icons, multi-cell items | 1:1; 1×2, 2×3, 2×4 cells | ARPG, MMO, SURV, RPG, BR, LIFE, GACHA |
| Ability, skill and buff icons | 1:1, legible at 24 px | MOBA, MMO, ARPG, CRPG, RGL, SHO |
| Card art and card frames | 5:7 card, art window about 4:3 | CARD, AUTO, GACHA, BRD |
| Unit and building portraits | 1:1 | RTS, TD, CITY, 4X, TAC |
| Level, track and song thumbnails | 16:9 or 4:3; song jackets 1:1 | PLT, M3, RACE, RHY, SBX |
| Save thumbnails | 16:9 | VN, RPG, OW |
| Menu backgrounds and patterns | 16:9 with bleed; tiling | all |
| Event and gacha banners | 2:1 to 16:9; carousel about 3:1 | GACHA, M3, CAS, BR |
| Player banners and calling cards | 4:1 to 5:1; tall 1:2 | SHO, BR, MMO, FGT |
| Emblems, crests, rank badges, flags | 1:1 | SHO, MOBA, 4X, SPT, MMO |
| Achievement icons | 1:1 | all |
| Emotes and stickers | 1:1 | PARTY, MOBA, CARD, GACHA, MMO |
| Currency and resource icons | 1:1, 16–48 px | all economies |
| Map glyphs and POI icons | 1:1 | OW, 4X, MMO, BR, RTS |
| Tech and talent node icons | 1:1 or hex | 4X, ARPG, MMO |
| Tutorial and loading-tip art | 16:9 or 4:3 | all |
| Cosmetic previews (viewport) | 1:1 or 3:4 | BR, SHO, GACHA, RACE, MMO, SBX |
| Style art: frames, buttons by state, ornaments, icon sets | nine-slice with declared insets | all |

## Sources

The appendices and §5–§8 draw on Unreal's CommonUI, UMG and its Viewmodel,
Slate invalidation and the Retainer Box; Unity's UI Toolkit and its
comparison with uGUI; Godot's containers, themes, Tween and SubViewport;
Rive's state machines and data binding; NoesisGUI's Visual State Manager;
Coherent Gameface; SwiftUI's springs and matched geometry; Flutter's Hero
and staggered animations; the Game UI Database; the Xbox Accessibility
Guidelines (101–118) and the Game Accessibility Guidelines; GDC talks on
the UI of Dead Space (2013), Hearthstone (2015) and Into the Breach
(2019).
