# Origin Engine — game UI

The design of og-ui as the engine's game UI infrastructure. It is normative
for `crates/og-ui`, the UI pass of `crates/og-render`, the UI members of the
API (`module ui` in `crates/og-api/api/ui.ogt`), the style format, the UI kinds
of the art brief and the UI checks of `og`. `docs/design.md` §2.7 points
here.

Owner's decisions this design follows (2026-11-01):

- UI and UX are the core of a game's picture, interaction animation
  included. The UI is rebuilt to that bar, as a large and serious piece of
  work, not a reskin of the current kit.
- og-ui is extended to meet what games need; what it can do today never
  limits the design.
- Every genre is covered fully. The kit is not a demo.
- UI is a base layer for agents: an agent authors it **and adjusts it**, so
  every value is data that can be read, changed live, checked and saved.
- Quality over reach (design.md §1): current hardware, WebGPU only, no
  tiered fallbacks for low-end devices.
- Generated art is first class: screens reserve explicit, specified places
  for generated images.

## 1. Where it stands

Measured at `c030bfe3` (the survey is in the thread that wrote this).

**Kept, because it is right:**

- Immediate-mode building from TypeScript with the tree laid out
  Clay-style each tick, its retained state (`State::anims`, the toast
  queue, focus, scroll, carets) in `Ui::save_state`, so a snapshot carries
  the UI. Motion counted in ticks, arithmetic only.
- Stable ids and paths (`A/B#3`) shared by hit tests, the probe and the
  accessibility tree. `dump_ui`, `touch_move_to`, `move_to_text`,
  `expect:ui:text`.
- The text stack: MSDF text, bidi, shaping, kerning, font per run, run-time
  CJK glyphs, the editor with IME, rich text.
- One focus model across pointer, keyboard and pads, several pads,
  multitouch gestures, the accessibility tree and its ARIA mirror on the web.
- Styles as complete, validated assets, contrast measured over the real
  atlas texels (`tests/packs.rs`, `tests/pack_contrast.rs`), `og style
  eject|pack`, and `og polish`'s pixel contrast, text size, HUD share,
  transition and sound measures.

**Not enough:**

- No data form for a screen. A screen exists only as per-tick TypeScript,
  so an agent adjusts it by editing code, and nothing in the UI is
  reflected, so Tune cannot touch it.
- Six draw primitives (`DrawCmd`: glass, quad, image, bar, half, text): no
  gradients, strokes, vector shapes, masks beyond rectangles, blend modes,
  shadows outside a style part, or UI shaders.
- No per-element transform, rotation, opacity or z order. Motion is seven
  transition kinds, the hover and press springs, a count-up, a burst, a
  shake, glyph effects and five screen covers: no keyframes, sequences,
  visual states, layout animation or shared elements.
- Styles dress twelve fixed element classes (`StylePart`). A new widget has
  no art class of its own.
- No world-space UI (nameplates are a `worldToScreen` recipe, damage numbers
  are og-render's floating text), and 3D in the UI is a texture with no
  input.
- The fixed templates (`mainMenu`, `racingHud`, `resultsScreen` …) fail
  outside their premise: English words, missing fields, genre assumptions
  (brasswick #4 #10, meridian-loop #8 #10, kestrel-stair #15). Games then
  rebuild them from raw rows and labels and copy their own spacing scales.
- The art brief has no UI kinds; generated art reaches the UI as a plain
  texture or a hand-packed nine-patch.
- `og polish` does not check overlap, overflow, touch size, safe area,
  focus reachability or localization expansion. The ui-kit's phone sheets
  shipped with the platformer timer under the coin pill and the puzzle
  Gallery button over "Target 1,000".

## 2. Principles

1. **Intent is data; the engine lays it out.** A screen is a document of
   elements, bindings, states and motion. Code fills slots and handles
   events. An agent's adjustment is a small diff of data, applied live.
2. **Every value is addressable.** Every element prop, style token, state
   and timeline key has a path, is in the reflection schema, is readable
   through the probe, writable through Tune and the probe, and saved back
   to its file. Nothing is described twice (AGENTS.md, reflection is the
   single schema).
3. **Every rule is checked.** What a UI designer reviews by eye (overlap,
   overflow, hierarchy, touch size, reachability, contrast, missing
   feedback, expansion) is a check with a path and a fix, run by `og ui
   check`, `og polish` and `og test`.
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
   the snapshot. Each budget in §15 is a perf test.

## 3. Coverage

The UI serves every genre the engine targets. The coverage contract:
every row of the inventories in the appendices (screens B, HUD and world
elements C, interactions D, motion E, art slots F) maps to

- the engine capability that makes it possible (a section of this design),
- a component or screen document in the library (§11) that implements it,
- and a test in the kit (§17) that renders it in at least two styles, on
  desktop and phone, with its interaction scripted.

Appendix A lists the genres (35, from platformer to board games) with the
capabilities each depends on. A genre is covered when every row it is
tagged with is covered. The kit proves coverage with one **genre showcase**
per genre family (§17), each a playable screen set, not a static sheet.

## 4. Architecture

Ten parts, from the GPU up. Each section below names the crate that owns
it.

| Part | Owner | Section |
|---|---|---|
| UI render pass: shapes, paints, effects, masks, layers, materials | og-render (`ui` module), og-ui `draw.rs` | §5 |
| Element model and layout | og-ui `ui.rs`, `layout.rs` | §6 |
| Motion: springs, states, timelines, choreography, gestures | og-ui `motion/` | §7 |
| Layers, input routing, actions and glyphs | og-ui `layers.rs`, `nav.rs`, og-input | §8 |
| Data binding and formatting | og-ui `bind.rs`, og-i18n | §9 |
| Documents: `type screen`, `type component` | og-ui `doc/`, og-cook | §10 |
| Component library | og-ui (Rust core widgets), `packages/engine/src/ui/` (documents) | §11 |
| Styles v2 and art slots | og-ui `style_file.rs`, og-cook, knowledge `brief.ts` | §12, §13 |
| World-space and 3D UI | og-ui `world.rs`, og-render | §14 |
| Agent tooling: dump, write, Tune, preview, checks | og-probe, og-cli, `packages/engine/src/node` | §16 |

Text, audio and haptics, and accessibility cut across these parts and have
their own sections (§5.6, §7.8, §15.2).

## 5. Rendering (og-render's UI pass)

The overlay of quads becomes a UI pass of analytic primitives, drawn
instanced by one family of pipelines, after post effects. The shapes are
signed distance functions evaluated per pixel, so edges are exact at any
scale and transform.

### 5.1 Shapes

- Rectangle with four corner radii and a corner shape per element: round,
  **squircle** (continuous corners, superellipse n configurable), bevel,
  notch (inset), so a sci-fi panel's cut corners are geometry, not art.
- Circle, ellipse, capsule, ring and arc segment (start, sweep, thickness,
  round or butt caps): cooldown sweeps, radial menus, gauges.
- Regular polygon and star (sides, inner ratio, corner radius): hex
  talent nodes, badges, burst stars.
- Line and polyline with caps, joins and dashes; Bézier curves: targeting
  arcs, connectors in skill trees and run maps, graphs.
- Vector paths: a `type vector` asset (SVG subset: paths, fills, strokes,
  gradients, groups) cooked to triangles with analytic antialiasing on
  their edges, drawn with the same paints. Icons keep the MTSDF icon sets.

### 5.2 Paints

Each element has a list of fills and a list of strokes, drawn in order
(layered paints, as in design tools):

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
**backdrop** effects (blur, saturation, brightness: the current glass
becomes `backdrop blur`). Opacity, and blend modes (normal, multiply,
screen, add, overlay, soft light, colour dodge) per element against what is
under it.

**Glass** is a backdrop with a material (og-render's `UiGlass`, zero
values the plain frosted backdrop, unchanged): flat glass as the design
language composes it, clear and coloured by what passes under it, with no
specular rim, sheen or refraction. Its parts are a light tint over a
modest blur with raised saturation, a hairline just inside the edge
(CSS `inset 0 0 0 0.5px`), up to two soft shadows outside the shape only
(CSS `box-shadow`s, never darkening what the glass shows), and CSS mixing
(tint, hairline and shadows over sRGB-encoded values, so a tint reads as
on a web page), all drawn by the backdrop's one instance. `solid` draws
the tint opaque and reads nothing under it: a frame whose only glass is
solid builds no blur pyramid. Three weights are presets
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
at 4.5:1 or more over bright, dark and busy backdrops in either scheme,
their current item and selected row included (og-tests
`ui_glass_text_holds_contrast`, worst 2 % of each text box: lowest 4.73,
the dark bar's current item over a bright sky). For that the dark
`regular` tint is 64 %, denser than the platform's 52 % (3.56 there).
`glass-thin` keeps its clear tints and carries **icons and play discs
only, never text**: over a bright sky a dark thin capsule's text read
1.78, a light one over a dark scene 3.24, and holding 4.5 would take
about 68 % (dark) and 50 % (light), no longer thin.

Palette pairs these scenes measured under 4.5:1 (the ui-scale stream
fixes them when it generates the palette from `packages/design`; the
glass is not the cause, most fail on solid glass too):

- `fg-muted` on `fill-strong` (a selected row's shortcut): 3.0–3.8, and
  3.51 on solid glass.
- Amber text (`accent-ink`) on a bar's current-item fill: 1.80–4.31 (4.31
  on solid glass); on a `regular` bar without the fill 2.25–4.78.
- `fg-muted` on a `thick` menu whose scheme does not match the backdrop:
  3.85–3.88 (light menu over a dark scene), 4.10–4.29 (dark menu over a
  bright sky).
- `accent-ink` "Undo" on a light `thick` toast over warm light: 4.26.
- Translucent control tints: the platform's `material-control` (white 13
  %) is specified for CSS's blending over encoded values; the UI pass
  blends flat fills in linear light, where it reads about three times as
  strong over dark glass (a white current item over a bright sky fell to
  3.60). About 5 % draws as CSS's 13 % does there; the palette should
  state its translucent fills for the engine's blending.
- Dark dims are the reverse case: a black fill at alpha `a` over bright
  art leaves `1 - a` of its linear light, which the eye reads as far more
  than `1 - a` of its brightness. A 60 % black sweep over pale ice art
  read as barely darker (U6c, 2026-11-02); 90 % reads as the 55–65 %
  darkening the art bar asks of a cooldown (§15.5), and 75 % as about
  half. State a dim by how it reads and measure it on bright art.

### 5.4 Masks and layers

- Any element can clip its children to its own shape (rounded, squircle,
  polygon, path), antialiased, not only to its rectangle.
- Alpha masks from an image or a gradient: a list fading at its scroll
  edges, a portrait in a brush-stroke shape. A mask's alpha fades as CSS's
  does, not by linear coverage: the composite restates it for the
  engine's linear-light blending against the content's opposite ground
  (white under a dark scrim, black under light text), so a dark scrim
  darkens evenly through its fade instead of cutting in at its end.
- An element marked `layer` draws its subtree into an offscreen target
  that is then composited with a **UI material**. Built-in materials:
  dissolve (noise threshold with an edge glow), shine sweep, hologram
  (scanlines, flicker, chromatic split), glitch, ripple, pixelate,
  colour adjust (hue, saturation, brightness, grayscale), outline of the
  alpha, and blur. All take their time from the UI clock.
- Custom materials: a `type ui-material` asset is a WGSL function of uv,
  the layer's texture, time and declared parameters, validated by naga to
  MSL, SPIR-V and HLSL at cook like every other shader (og-render's
  `tests/shaders.rs` covers the built-ins). Its parameters are reflected,
  so they animate and tune like any prop.

### 5.5 Particles, video and 3D in the UI

- UI particle emitters in UI space (confetti, sparks, coin streams, rarity
  bursts), using og-render's GPU particles in a 2D overlay mode, attached
  to an element and clipped by its layer. `burst` becomes a preset.
- Video textures in image paints (already a texture source).
- The **viewport** element shows a camera's view of a scene layer (a
  character on a turntable, an item to inspect, a car in a garage), with
  its own lights and environment, sized by layout, and passes input
  through: a press or drag inside it becomes a ray into that scene, so a
  game can orbit, pick and rotate inside a menu.

### 5.6 Text effects

On top of the existing text stack: outline (outside, inside, centred, by
the distance field, up to the font's reach), gradient fill, several
shadows, per-glyph transforms (text on a path or arc, per-glyph timelines
for typewriter, wave, bounce, shake and reveal), auto-fit (shrink to fit
down to a minimum type step, then ellipsis: `LabelDesc.fitMin`), and inline
images and icons in rich text with links that open pinnable nested tooltips
(§11). Rich text's tags: `[b]`, `[i]` (a slant of the face), `[u]`, `[s]`,
`[outline=w,colour]`, `[color=]`, `[size=]`, `[font=]`, `[icon=]` (an icon
atlas cell), `[img=]` (any texture the game registers by name,
`UiDesc.images`), `[action=]` (the action's device glyph, resolved every
frame, so a prompt follows the pad the player picks up), `[link=]` and the
per-glyph `[wave]`, `[shake]`, `[pulse]`.

### 5.7 Batching

All shapes, paints and effects other than layers and backdrops draw from
one instance buffer through a small set of pipelines keyed by primitive
family. Painter levels stay as today (overlap plus batch). A layer or a
backdrop starts a new pass only for its region. Budgets in §15.

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

### 6.2 Layout

The current five walks stay, with additions:

- **Grid**: tracks (`fr`, fixed, auto, min/max), gaps, spans, named areas,
  auto-flow; a dense inventory grid and a dashboard are grids.
- **Stack**: children overlaid in the same rectangle (a card's art, frame,
  cost gem and name), each aligned by its own insets.
- **Container queries**: a component chooses its layout by its own width,
  not only by the viewport's breakpoint, so the same card works in a
  four-column shop and a one-column phone list.
- **Breakpoint variants**: a document can override any prop per breakpoint
  (`phonePortrait`, `phoneLandscape`, `tablet`, `desktop`) and per
  orientation, and declare a different arrangement, not only a smaller one.
- **Direction**: layouts mirror under right-to-left locales; insets and
  alignments are `start`/`end`.
- **Baseline alignment** in rows of mixed type sizes.
- **Overlap is an error unless declared.** Siblings in flow cannot overlap;
  absolutely positioned and anchored elements that overlap another element
  must say so (`overlaps: "<path>"`), or `og ui check` reports them with
  both paths (§16.4). This makes the phone-sheet bugs of §1 failures. An
  element is measured where its own and its ancestors' declared transforms
  draw it (a label scaled down to fit a corner covers its scaled box, not
  the line it was laid out on); motion is not part of the check.

### 6.3 Invalidation

A subtree whose inputs (props, bound values, size) have not changed since
the last tick keeps its layout and draw commands. A changing subtree is
re-laid out up to the nearest fixed-size ancestor. The probe reports
elements rebuilt and laid out per tick (§16.1).

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
Completion is reported at the perceptual duration. The existing hover and
press springs become two uses of this.

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
  centre, with a total cap (the current panel stagger is one case);
- `cascade`: each step's start waits for an event of the previous one (a
  score that counts chips, then multiplies, each step pitched higher).

The result screen of a casual game is one timeline: the card springs in,
three stars land in turn with a burst and a rising pitch, the score counts
up, a "New best" ribbon slaps on, rewards stagger in, and the buttons take
input last.

### 7.5 Presence and layout animation

- **Enter and exit** per element: an element removed from the tree is kept
  (inert, not hit-testable, not in the accessibility tree) until its exit
  finishes, so lists, toasts and panels leave the way they arrived.
- **Glitch** (`glitch`, a transition kind beside fade, the slides, scale
  and pop; any style can name it, `motion { enter glitch 0.8 }` with its
  strength): the content draws through the glitch layer material (§5.4:
  slices pushed sideways, colours split about 3.5 px) as it fades in or
  out, over 150–180 ms whatever the style's base duration, weakening to
  nothing as it settles. Its slice height comes from the element's id and
  its slices are re-cut every two ticks of the tick-counted UI clock, so a
  captured frame is the same every run; under reduced motion it is a plain
  fade. An element's own exit (props) has no tear: it fades. A transition
  between whole screens that captures the outgoing one is separate work.
- **Layout animation**: an element whose laid-out rectangle changes
  (reordering, inserting, resizing, wrapping to a new line) moves from
  the old rectangle to the new on a spring (the FLIP technique). A hand of
  cards fanning open, an inventory sorting and a list item expanding are
  this.
- **Shared elements**: elements with the same `match` key on the screen
  being left and the one arriving morph between their rectangles,
  transforms and corner radii, cross-fading their contents (a shop card
  opening into its detail, a character portrait flying into the team
  slot).

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
requires the fallback.

### 7.8 Sound and haptics

Motion events and UI events play sounds and haptics from the style's sets
(§12.4). The event list grows from six (hover, press, open, close,
success, error) to: focus, select, confirm, cancel, back, open, close, tab,
toggle on and off, slider and scroll ticks, typewriter blips, counter
ticks (pitch rising through a cascade), notification, warning, error,
success, reward, and rarity reveals by tier. Haptics are patterns on the
same events: pad rumble, the phone's vibration on the web and native, with
a player setting for intensity and off. A sound is still queued for the
tick of the input that caused it.

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

Input goes to the top layer that accepts it, once per tick. This replaces
each game's own focus and Back handling, which is where the FINDINGS show
most UI bugs (brasswick #9, meridian-loop #9).

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

The current focus model stays (one focus for pointer, keys and pads,
scopes, several local players with coloured rings) and gains: explicit
neighbour overrides, focus groups that wrap or not, focus memory per
layer and per group, an optional pad-driven virtual cursor for dense
screens (a 4X map, an inventory grid), and several simultaneous cursors for
local multiplayer selection screens.

### 8.4 Player-customised HUD

Elements marked `customizable` can be moved, scaled and faded by the
player in a layout editor component and are saved in the game's settings.
Virtual controls on touch (sticks, buttons) are customisable the same way.
Shipped as `ElementProps.customize` (a key, per-axis locks and scale and
opacity limits); placements are kept per player by key and applied before
the frame's transforms, so touches follow them. `og.ui.setHudEdit` is the
edit mode (press to select, drag to move, pinch or wheel to scale, the
stick for pads), `og.ui.hudLayoutEditor` its panel, and
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

Bindings are read once per tick after the game's systems, compared with
the last values, and only changed bindings invalidate their subtrees (§6.3).

## 10. Documents

### 10.1 What they are

A **screen** (`type screen`) or **component** (`type component`) is an
`.ogt` text document: an element tree with props, slots, bindings, state
groups, timelines and breakpoint overrides. og-cook validates it against
og-ui's reflection schema and cooks it into an `og.UiDocument` blob; the
generated wiki documents every prop from the same schema.

```
type component
version 1
---
name "ShopCard"
props { item: og.ItemView, price: number, selected: bool }
parts { frame art name price rarity }
states {
  interaction { normal hover pressed disabled }
  selection { off on = selected }
  availability { available locked = !item.owned && price > wallet.gold }
}
stack frame {
  part frame  fill token(panel) stroke 2 token(line) radius token(card) shape squircle
  art art     slot itemThumb kind item aspect 1 focal 0.5 0.4 safe 0 0 1 0.7
  column {
    label name  text item.name step title maxLines 2 autofit caption
    row price   { icon coin  counter text price format compact }
  }
  badge rarity  text item.rarity tone rarity(item.rarity) anchor topEnd
}
transition interaction hover   spring token(ui)  { translateY -4 shadow token(lift) }
transition selection on        spring token(emphasis) { stroke 3 token(focus) }
timeline unlock { 0 state availability available; 0 burst sparks; 0 sound reward }
```

The grammar above was the sketch for review; phase U3 settled it in
crates/og-ui/README.md §Documents (one construct per `.ogt` line, props
from og-ui's reflected element schema, `og.ui.documentSchema()`).

### 10.2 Documents and code

Immediate-mode building stays: `og.ui.*` calls from a system each tick
build the same element tree, and both can mix. Code mounts a document
(`og.ui.screen("shop", vm)`, `og.ui.component("ShopCard", props)`),
fills its slots with code-built content, and receives its events
(`clicked`, `changed`, named events from timelines). A game written in
code alone still gets states, motion and checks, because they belong to
the element model, not to the document.

### 10.3 Why documents

An agent changes a document by a small diff that `og dev` applies in place
without reloading the game, keeping the UI state. Tune and the probe write
any prop of it live and save the result back into the file (§16.3).
Checks read its static structure: every slot, state, action and binding is
known without running every path of the code.

## 11. Component library

The fixed templates with a field per word (`mainMenu`, `racingHud`,
`resultsScreen` …) are removed. In their place:

- **Components** built only from public primitives and documents, each with
  named parts a style can dress, slots a game fills, a viewmodel contract,
  state groups, motion, sounds, an accessibility role, probe ids, a wiki
  page and tests.
- **Screen documents** for every screen type of Appendix B, which a game
  copies (`og ui eject <screen>`) and edits as its own file, so no screen
  is ever limited by a template's fields again.

Categories (the full lists are the appendices):

- **Foundations**: text, rich text, icon, image and art slot, shape,
  divider, spacer, viewport, particle emitter, layer.
- **Controls**: buttons (all variants and the hold-to-confirm), toggles,
  checkboxes, radios, sliders and range sliders, steppers, selects,
  segmented tabs, text fields and areas, search with suggestions, colour
  picker, key-binding capture, the on-screen keyboard.
- **Containers and navigation**: panels and cards, scroll, tabs with
  bumpers, carousel with snap, accordion, split views, sheets (bottom and
  side, draggable), modals and confirms, popovers, menus and context menus,
  radial wheels, breadcrumbs, the action bar, pagination.
- **Data**: virtualised lists, grids and sortable, filterable tables (10k
  rows and more), trees, pan-and-zoom graphs (skill and tech trees, run
  maps, flowcharts), charts (line, bar, area, radar, pie for stats and
  post-match graphs), key-value and stat-compare rows with deltas, the
  calendar.
- **Feedback**: toasts and the notification stack, badges, tooltips
  (nested, pinnable, with links; compare tooltips), progress and spinners,
  skeletons, counters with notation, banners and ribbons, the
  celebration set (confetti, burst, stars, level-up), subtitles and
  captions to XAG 104.
- **HUD**: meters (segmented, ghost damage, shield cells, orbs, radial
  stamina), ability bars with cooldown sweeps, charges and hotkeys,
  buff and debuff grids with timers and stacks, unit, party and raid frames,
  cast bars, crosshairs and hit markers, ammo, minimap and compass,
  objective tracker, killfeed, scoreboards, timers, combo and judgement
  pops, resource bars with rates, turn order, speedometer, scorebug, flight
  ladder, damage direction, low-health vignette.
- **World** (§14): nameplates, overhead bars, damage numbers, interaction
  prompts, waypoints and pings with edge clamping, placement ghosts, range
  and area telegraphs, path lines, selection rings, loot labels, speech
  bubbles.
- **Genre patterns**: inventory grids with multi-cell items, rotation,
  stacking and splitting; paperdolls; card hands (fan, hover lift, drag to
  play with a targeting arc); pick-one-of-N rewards; gacha banners and pull
  sequences; shops and stores; battle passes; lobbies, party and ready
  checks; character select and versus screens; draft and ban; dialogue
  with choices, timers and checks; the VN text box, backlog and gallery;
  quest log and codex; the world map; level and saga maps; song select;
  garage and tuning; management ledgers; the chat window; command cards
  and control groups; the HUD layout editor; photo mode; the settings
  tree (graphics, audio, controls with remapping, accessibility, language).

### 11.1 Screen documents

The library's screen documents are `packages/engine/src/ui/screens/`, in
the engine package: 59 `type screen` documents, one or more for each row
of Appendix B (`main-menu`, `pause`, `save-load`, `inventory` …), built
from og-ui's elements. Each names its viewmodel (`props`), the events it
emits and which og.ui component or pattern fills each slot at its top; its
words are string keys (`screen.<name>.<word>`) with English fallbacks, and
a `fixture default` previews it (`ui:push:<name>:default`). The directory's
README.md holds the table from each Appendix B row to its documents, which
the cook's test (`crates/og-cook/tests/documents.rs`) holds equal to
Appendix B, cooking every document. Where an og.ui pattern fills a slot
and draws its own title, tabs or actions (the settings tree, the ledger,
photo mode …), the document leaves those to it. `og ui eject <screen>
[project] [--force]` copies one into the game's `assets/ui/<screen>.ogt`
(refusing to overwrite without `--force`), where `og.load.document` loads
it and the game edits it as its own; `og ui eject --list` prints each
with its row.

## 12. Styles v2

### 12.1 What a style is

A style (`type style` version 2) has tokens, part dressings, a motion
personality, a sound and haptic set, fonts, icons and art. It stays one
complete asset a look's `ui` line names. The engine's default style (the
design language the engine shares with origin-agent, `packages/design`,
which replaces `prototype`) is for origin-agent's own surfaces and for
prototyping only (the owner, 2026-11-02): every game ships its own style,
built-in, generated or hand-made, and a publish build refuses a game whose
screens still draw in the default style. The design language binds the
default style alone; a game's style may restyle, re-dress, re-shape or
replace anything above the accessibility floor of §15.2.

The default style carries no hue in its chrome (the owner, 2026-11-02):
its primary is the high-contrast neutral, a near-white fill with dark ink
on dark surfaces, set apart from secondaries by fill and weight, and focus
and selection are a neutral light edge or glow. Hierarchy, frames,
material, type and spacing follow the clean sci-fi direction (concept B):
deep navy surfaces, glass with a lit top edge, letter-spaced uppercase
titles. Semantic roles (danger, success, warn, info) and game roles
(health, mana, xp, gold, rarity) keep their hues because they carry
meaning; a game brings every other colour in its own style.

### 12.2 Tokens

- **Colour**: the 33 roles of today, plus rarity tiers (common, uncommon,
  rare, epic, legendary, mythic), team and player colours (8), damage and
  element types, and chart series. A style may also declare roles of its
  own (a game's corruption, sanity or faction colours) beside the named
  ones; components and game code read them by name like any role, and
  contrast and completeness checks cover them (the owner, 2026-11-02). Each status and rarity colour is paired
  with an icon or pattern, so no meaning is carried by colour alone (XAG
  103).
- **Type**: a scale of display, title, heading, body, label, caption and
  numeric steps per breakpoint, with tracking, weight and line height;
  fonts per role, with CJK and fallback faces as today. An optional
  button step sets a button's words apart from label (the label step
  when left out): a pixel face keeps label at a HUD's 12 and sets
  buttons at body.
- **Space, size and radius** scales; **elevation** (shadow sets per level);
  stroke widths; **shape language** (corner shape and radius per element
  class: squircle, notch, bevel, round, square).
- **Motion**: the springs, eases, durations and stagger of §7.7.

### 12.3 Part dressings

A component names its parts (`ShopCard.frame`, `Meter.fill`,
`Button.base`); a style dresses any part, by variant and state, with
selectors: `Button.base[primary][pressed]`, `Slot.frame[rarity=epic]`. A
dressing sets paints, strokes, effects, shape, nine-slice art, ornaments
(corners, flourishes, dividers, tiles), materials and sounds. A part a style
does not dress is drawn from the tokens by the component's default, so a
new component looks right in every style without new art. The twelve
`StylePart` classes of version 1 become dressings of the matching parts;
`og style upgrade` converts a version 1 style.

A game shipped with a version 1 style keeps the screens it shipped with:
an engine update may draw them better (antialiasing, contrast, a cleaner
fill) but never changes what is on them or where. Its dialogs' titles take
the ramp's xl in the body face, its ramp steps no tracking it did not set
(the prototype ramp's), a grid's selected cell its `primary` underline and a
burst (`ui.burst`, a results screen's earned star) its drawn rays and sparks
rather than particles (`UiTheme.styleVersion` 1); its screens are its own
ejected documents (§11), so engine work never re-lays them out. A version 1
file's slider knobs and switches take the control radius; `og style
upgrade` writes `screens 1`, so the upgraded style keeps all of this but
draws them round, as games shipped once upgraded. A new style (`og style
new`, the packs) draws today's.

### 12.4 Sound and haptic sets

The events of §7.8, each with one sound or a set chosen at random without
repeats, volume and pitch ranges, and a haptic pattern.

### 12.5 Built-in styles and generated styles

Six built-in styles are not enough. The engine ships at least twelve seed
styles covering the look families of `og-taste` §12 (for example ornate
fantasy, storybook, tactical military, sci-fi holographic, cyberpunk neon,
casual cartoon, kawaii and anime gacha, pixel retro, paper and craft,
diegetic horror, sports broadcast, ink and eastern brush); the list is
settled with the owner in phase U4. The minimal modern seed was dropped
(the owner, 2026-11-02): its quiet light choices remain `og style new`'s
start when a look matches no family, but no built-in pack ships them.
The same day the owner settled sixteen packs (fantasy, jade, kawaii,
cartoon, midcore, storybook, craft, pixel, gilded, tactical, wasteland,
dread, scifi, cyber, foundry, broadcast) and folded six into them: bubbly
into cartoon, arcade into pixel, paper into storybook, inkwash into jade,
neon into cyber, holo into scifi. The folded packs' fonts and icon sheets
that seeds draw with live in `templates/styles/shared/`; the folded seeds'
choices stay `og style new`'s start for their look families.

A **style generator** (`og style new`) makes a game its own style from the
look, the premise and a few choices (palette seed, shape language,
ornament level, motion personality): tokens and procedural dressings
first, then the art slots of its frames, buttons, ornaments and icon set
filled through the art brief (§13.3). Each pack owns `choices.json` and
`style.ogt` as independent authored data; directory discovery supplies the
seed catalogue. There is no regenerate-all command or seed-equality test.
A generated style passes parsing and completeness checks, not contrast
thresholds (§17, the owner's decision of 2026-11-04).

### 12.6 Painted art at every density

A style's atlas line may name denser copies of one layout,
`atlas "atlas.png" 384 256 2 3`: these are `atlas@2x.png` and
`atlas@3x.png` beside it, at 2 and 3 texels per CSS px. Regions and insets
stay in 1× texels, so a slice cuts the same art from every copy. Each frame
draws from the copy nearest at or above its device pixels per og-ui unit
(dpr times the resolution-relative unit), and from 3× above that. Phones
top out at 3, and tablets and desktops at 2. `og style art` (docs/art.md
§2.7) packs a style's painted slots into all three.

## 13. Art slots

### 13.1 The slot

An `art` element (or an image paint) is a **slot**: a name, a kind, an
aspect (or the aspects allowed), a focal point that cropping keeps, a safe
rectangle where nothing important may sit because text or controls are
laid over it, a fit, a mask, and a treatment from the style (vignette,
duotone, frame, rarity glow). Its placeholder is drawn from the style and
the kind (a gradient, the kind's silhouette icon, the slot's name in
the default style only), never an empty grey box, and `og ui check` reports
placeholders still present at publish.

### 13.2 Kinds

Appendix F lists them with their aspects: key art and title backgrounds,
store capsules, the app icon, character splash and full body, portraits
and busts with expression sets, VN sprites, event CGs, item and equipment
icons (multi-cell for grid inventories), ability and buff icons, card art
and card frames, unit portraits, level, track and song thumbnails, save
thumbnails, menu backgrounds, banners, player banners and calling cards,
emblems and rank badges, achievement icons, emotes, currency icons, map
glyphs, tech and talent node icons, tutorial illustrations, cosmetic
previews (from a viewport, §5.5), and the style's own UI art (frames,
buttons, ornaments, icon sets).

### 13.3 The art brief

`og ui slots` lists every slot of a game with its spec. The art brief
(`packages/knowledge/scripts/brief.ts`, a public contract) gains the UI
kinds above, each with its aspect, focal point and safe area, the look's
conform rules (`docs/art.md` §2.3), and, for style art, the nine-slice
insets and the states to draw. Adding kinds to `ART_KINDS` is additive; any
change to existing signatures follows AGENTS.md's rule for a break. A
generated image lands in the slot after the look's conform pass, fades in
when loaded, and streams like any texture.

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
  text becomes one use of it.
- **World panels**: a document drawn into a texture on a mesh (cockpit
  screens, terminals, the Dead Space RIG, a shop sign), lit or unlit,
  interactive through a ray from the pointer or a pad cursor.

**Meta UI** (damage vignettes, low-health pulses, sanity distortion) is a
full-screen layer element with a UI material over the scene.

## 15. Quality bars

### 15.1 Performance budgets

Measured by the perf lane on the reference machines (metal, DX12):

- a dense menu screen (300 elements, 40 animating) builds, lays out and
  records its commands in ≤ 0.5 ms of CPU per tick;
- a HUD plus 2,000 mass elements and 100 anchored elements in ≤ 1 ms of
  CPU per tick;
- the UI pass in ≤ 1 ms of GPU at 2560×1440, with layers and backdrops:
  the `ui-layers` and `overlay` passes of `ProbeTimings.gpuPasses` (the
  overlay's glass and screen lines included);
- a 10,000-row table scrolls without a frame over budget.

### 15.2 Accessibility

The accessibility tree, its ARIA mirror and reduced motion and transparency
stay, and the bar rises to the Xbox Accessibility Guidelines:

- text scale to 150 % without loss of content (200 % deferred to a later
  batch, the owner 2026-11-04; the player's text-size setting tops out at
  150 % until then); minimum text 18 px at 1080p
  on desktop and phone-equivalent sizes on touch, for menus, body text and
  controls. A dense HUD (strategy, MMO, simulation readouts) may declare
  its text dense (`ElementProps.dense` on the readouts' group) and go
  smaller, down to 12 px at 1080p, provided it grows with the player's
  text scale and passes the contrast rule (the owner, 2026-11-02; `og ui
  check` holds a dense subtree to 12 px at 1080p times the text scale);
  og-ui reaches the floor by its unit (`UiDesc.unitScale`, `UiTheme.unit`):
  on a mouse or pad surface the UI lays out in units of its 1080p-equivalent
  height over 720 CSS px, 1 to 2, and draws them at that size, so the whole
  UI zooms with the surface: the style's lengths and a game's own px alike
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
- the native host mirrors the tree to the OS through AccessKit.

### 15.3 Localization

Every screen is checked under a pseudo-locale that lengthens text by 40 %
and adds accents and brackets, under a CJK locale and under a right-to-left
locale (mirrored layout). No box is sized to its English text.

### 15.4 The review bar (owner, 2026-11-01)

The UI is held to the level of shipped commercial games. The look comes
first (the owner, 2026-11-02): every UI branch has art direction, the art
director's frame review is part of each round, and its notes bind the
author's next round. Numeric gates serve the look and never force an
ugly fix (a dark slab behind every name, a lit plate behind every label);
where a gate and the art director disagree, the art director decides and
the gate is loosened. The bar:

- Each genre showcase is read beside screenshots of two shipped games of
  its genre, and a screen that reads as a prototype beside them is held.
- Every changed screen is read at desktop 1920×1080, tablet, phone
  portrait 390×844 and phone landscape, in the kit's own style, and under
  the pseudo-locale, a CJK locale, a right-to-left locale and 150 % text (200 % deferred, §15.2).
- **What the review judges (the owner, 2026-11-05).** The kit and the
  style packs are a polished baseline an agent builds from, not a promise
  that every screen fits every style: a game restyles and reworks its own
  UI. So a screen is judged in the kit's own style and a pack on its own
  pages (`og ui sheet`, §17.3), never the cross product of every screen in
  every pack. A UI frame is reviewed by the art director over a plain,
  solid-colour ground, never over a 3D scene or key art the review would
  have to invent: a scene behind a HUD is judged by that scene's own
  tests, and no gate scores a label's legibility or contrast against what
  is behind it. `onScene` and `sceneInk` stay as API for games.
- `og ui check` reports nothing. A declared overlap is not a way to merge
  a known defect: the kit's Gallery overlap is fixed, not carried.
- Every interactive part is captured idle, hovered, pressed, focused and
  disabled, and each transition is recorded as a clip and watched.
- No placeholder survives: no flat-fill loading screen, no button whose
  edge does not show, text that reads at a glance by the art director's
  eye, and no two seed styles that read as one.
- A merge with a defect seen in a frame is held and fixed, never recorded
  as a reference.

### 15.5 The art bar (owner, 2026-11-02)

§15.4 catches defects; a frame can pass it and still read as a
prototype. Every UI frame is also read against this checklist by the art
review (the UI art director, beside the coordinator), and a frame that
fails any item is held like a defect. The target is the look of shipped
commercial games (Diablo IV, League of Legends, Destiny 2, Genshin
Impact, Honkai: Star Rail, Hades, Balatro), desktop browser first.

**The approved concept.** Each main screen and HUD row has an approved
concept image per style direction
(`tests/evidence/ui-art/concepts/<direction>-<screen>.jpg`, chosen by the
owner from the art review's options). A reference frame is read **side by
side with its approved concept** before it is recorded, and the review
names every place where the frame falls short of it. A frame with no
approved concept is read against the nearest one. The concept sets the
bar for hierarchy, material and art, not pixel positions: the layout may
differ where the engine's components or the breakpoint require it.

**Approved directions (owner, 2026-11-02).** B, clean sci-fi, is the
target of the engine's default style and of the UI kit's own frames; A,
dark fantasy, of the fantasy seed style; C, bright stylised anime, of the
anime seed style. Each is approved for every screen it was drawn for:

| Screen | Default style and kit | Fantasy seed | Anime seed |
| --- | --- | --- | --- |
| HUD (abilities, vitals, resources, minimap, buffs) | `B-hud` | `A-hud` | `C-hud` |
| Main menu | `B-menu` | `A-menu` | `C-menu` |
| Gacha banner | `B-gacha` | `A-gacha` | `C-gacha` |
| Shop | `B-shop` | `A-shop` | `C-shop` |
| Inventory and paperdoll | `B-inventory` | `A-inventory` | `C-inventory` |
| Settings | `B-settings` | `A-settings` | `C-settings` |
| Dialogue | `B-dialogue` | `A-dialogue` | `C-dialogue` |

B sets the default's hierarchy, frames, material and art; its colour
values stay the design language's (`packages/design`). **The default's
primary is neutral (owner, 2026-11-02)**: neither the concept's cyan nor
amber. Its chrome (panels, controls, the primary button, selection,
focus, tabs, sliders, rings) is drawn from the neutral ladder and told
apart by tone, lift and weight; hue appears only on semantic roles
(success, warning, danger, info) and game roles (health, mana, xp, gold,
rarity, team), never on the chrome itself. Each game colours itself
through its style; the seed styles keep their own palettes. A default
frame is read against this rule: any hued chrome is a finding. The kit's art
stays painted, and the showcase frames wear the seed styles with the
default as one column beside them. The concepts' README lists every
approved image and the genre sheets that extend the set.

**One selection grammar in the packs (art review, 2026-11-04).** In every
style but the default and a version 1 style, a selected chip, slot or tab
is one mark: the accent's hairline (`primary`, at the part's rim width
where the style dresses a rim), a ground one step up (`controlStrong`) and
its words in the accent's ink at 4.5:1 there (`dress::pack_grammar`). A
selection or focus colour is never a tier hue. An unchecked checkbox or
radio is a well whose rim reads 3:1 on the well and on the panel
(`dress::well_rim`).

**Every pack a seed (owner, 2026-11-02).** Each style pack under
`templates/styles` is a seed with an approved sheet of three screens (its
genre's HUD, a menu or shop, settings) in the showcase archive
(`ui-art/<id>`, docs/ci.md §Showcase), read like the table above; the
README's coverage matrix gives each genre a primary and secondary seed.
The owner settled sixteen packs (2026-11-02); bubbly, arcade, paper,
inkwash, neon and holo were merged into cartoon, pixel, storybook, jade,
cyber and scifi, and the README lists what each absorbing pack takes from
the merged sheet. scifi is real-world spaceflight operations, kept apart
from military sci-fi; cyber is high-contrast yellow, red and cyan with cut
corners. Genre language is free; another game's names, logos, fonts and
exact layouts are not.

| Pack | HUD | Menu or shop | Settings |
| --- | --- | --- | --- |
| fantasy | `A-hud` | `A-shop` | `A-settings` |
| jade | `jade-cultivation` (`ui-art/dba0973d790c`) | `jade-menu` (`ui-art/98481c251aeb`) | `jade-settings` (`ui-art/3f19757c41e2`) |
| kawaii | `C-hud` | `C-shop` | `C-settings` |
| cartoon | `cartoon-hud` (`ui-art/b476bf510b9c`) | `cartoon-menu` (`ui-art/18417a3dbdad`) | `cartoon-settings` (`ui-art/3fd89c603e7c`) |
| midcore | `midcore-battle` (`ui-art/b88efa4bb6ac`) | `midcore-menu` (`ui-art/1aa6c4f07057`) | `midcore-settings` (`ui-art/442a633f59e7`) |
| storybook | `storybook-hud` (`ui-art/dcc5fae146a1`) | `storybook-menu` (`ui-art/e181cf7311d3`) | `storybook-settings` (`ui-art/58673c78f815`) |
| craft | `craft-hud` (`ui-art/ffd3635f6b19`) | `craft-menu` (`ui-art/a9e6efac42b5`) | `craft-settings` (`ui-art/84ffbbb1286f`) |
| pixel | `pixel-hud` (`ui-art/3383564cc379`) | `pixel-menu` (`ui-art/187c8df6ca84`) | `pixel-settings` (`ui-art/41124ea7b1be`) |
| gilded | `gilded-hud` (`ui-art/23b214a1d6ae`) | `gilded-menu` (`ui-art/75f688ef5276`) | `gilded-settings` (`ui-art/6ca093ebfc1e`) |
| tactical | `tactical-hud` (`ui-art/5dc0d4592974`) | `tactical-menu` (`ui-art/9a9110d75f5e`) | `tactical-settings` (`ui-art/673672bc2ab1`) |
| wasteland | `wasteland-hud` (`ui-art/13876c1858fa`) | `wasteland-menu` (`ui-art/37b06d223094`) | `wasteland-settings` (`ui-art/860fbeffcbb7`) |
| dread | `dread-hud` (`ui-art/f3c187324b73`) | `dread-menu` (`ui-art/88d34c09a6b2`) | `dread-settings` (`ui-art/94d363b1c999`) |
| scifi | `scifi-hud` (`ui-art/70c6e287e1a1`) | `scifi-menu` (`ui-art/371bdbc76e8d`) | `scifi-settings` (`ui-art/5c356aaad012`) |
| cyber | `cyber-hud` (`ui-art/46b39cda6040`) | `cyber-menu` (`ui-art/b084a43067cb`) | `cyber-settings` (`ui-art/3cdb44b05813`) |
| foundry | `foundry-build` (`ui-art/a19f7660818a`) | `foundry-menu` (`ui-art/2593651e20db`) | `foundry-settings` (`ui-art/2b3bf48492bf`) |
| broadcast | `broadcast-hud` (`ui-art/342c9b49d2e6`) | `broadcast-menu` (`ui-art/734f4c4d2379`) | `broadcast-settings` (`ui-art/e16be56d19f0`) |

The checklist:

1. **Hierarchy.** Each screen has one dominant element (the key art and
   logo of a title, the splash of a banner, the paperdoll of a character
   screen, the focused row of a settings page) with the most contrast,
   size or saturation. Then at most two more levels: section titles and
   body. Titles use the style's display face and step (30–60 px at
   1080p), section labels its caps or small caps, body 18–22 px, values
   tabular numerals. A frame whose title, labels and values share one
   size and weight fails.
2. **One dominant element per control.** A slot, card or tile shows one
   primary reading (an ability's art, a card's art and name, an item's
   art), and at most one centred numeral (a cooldown, a count-down, a
   price). Secondary values (a cost, a charge count, a stack, a hotkey)
   are small, in a fixed corner or a label outside the art, and never
   two at the same size. A hotkey is a quiet label under or beside the
   slot, never a boxed cap over the art. One primary action per layer,
   in the style's accent and material, visibly larger or brighter than
   its secondaries.
3. **Real art over placeholder glyphs.** Every slot that shows a thing
   (ability, item, buff, card, portrait, character, enemy, banner, level,
   backdrop) shows painted or rendered art from the art pipeline
   (§13), lit, saturated, filling its slot. Line icons are for system
   chrome only (close, back, settings, search, sort, chevrons). A kit or
   reference frame showing a placeholder silhouette, a letter in an empty
   slot, a star glyph as card art or a flat-colour wash as key art fails;
   an empty equipment slot shows its kind's faint silhouette, not a
   letter.
4. **Numerals.** Gameplay numerals (cooldowns, damage, health, prices,
   counts) have a 1–2 px dark outline or shadow, or sit on a scrim, so
   they read over any art at 4.5:1. Tabular figures in columns and
   counters. Units and secondary parts (`/ 100`, `%`, `km/h`) are a step
   smaller than the value. No numeral is coloured by state unless that
   colour is the state (a red cost when unaffordable, a green delta for
   a gain); a key label never carries state colour.
5. **Selection and focus.** Selected, focused, hovered and pressed are
   distinct and read on art: an accent glow (6–8 px at 1080p) or an
   accent rim with a lift (shadow, scale 1.02–1.05), in the style's
   accent. A pure white 1–2 px outline or a white fill as the selection
   fails; flooding the selected item's art with a colour fails. Disabled
   desaturates and darkens the art (about 35 % value) and shows why (a
   lock, a slash, a reason line). Every state is captured (§15.4).
6. **Depth and material.** At least three elevations are distinguishable
   by value and light: the backdrop, the panel, the control (and a raised
   or floating layer for menus and tooltips). Each surface has the
   style's material: a lit edge (a hairline or a top-lit rim), an inner
   shadow on wells and sockets, a drop shadow on raised parts, a
   gradient or texture where the style has one. A flat grey rectangle
   with no edge as a panel, button or socket fails. No translucent grey
   box sits behind a HUD group: HUD pieces float on their own shadows.
7. **Spacing rhythm.** All gaps and paddings come from the style's
   space scale (a 4 or 8 px base), and a screen uses at most four steps
   of it. Labels sit 4–6 px from what they label; groups are separated
   by at least twice the gap inside them. A screen does not leave more
   than a third of its area empty around a small centred card: either
   the composition fills the frame (art, columns) or the card grows.
8. **Motion stills.** The frames of a transition (§15.4's clips) are read
   as stills: every mid-transition frame is a composed frame (no half
   faded text over a hard edge, no element snapped while its neighbour
   eases, no flash of an unstyled state). A key moment (a reveal, a
   level up, a ready flash) has a frame that would work as a screenshot
   of the game.

Each item becomes an `og ui check` rule where it can be measured (the
numeral outline, the white-outline selection, the placeholder slot, the
spacing scale); the rest stays a reviewer's read. The review records its
findings in the stream's thread and, for the kit, in
`tests/evidence/ui-art/`.

## 16. Agent tooling: read, write, preview, check

This is the base an agent stands on to adjust a UI.

### 16.1 Read

`dump_ui` gains per element: the resolved props (box, transform, paints by
token), the style part and dressing, the state of each group, bindings and
their values, animations in flight with their progress, slot specs, the
layer it belongs to, and whether it was rebuilt this tick. A
`dump_ui:layers` lists the layer stack with input modes and focus. A
`dump_ui:motion:<path>` samples a transition or timeline as a list of
values over ticks.

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
or style in place, keeping the UI state and the world.

Loaded `type ui-material` assets also refresh in place. Editing their
`source` WGSL or `param` defaults keeps the material handle and replaces
the cooked shader and defaults after the manifest refresh. The renderer
invalidates its pipeline when the shader or parameter declaration changes.
A failed cook leaves the previous material running.

On macrunner's Metal host, a one-material `og dev` smoke test on 2026-11-05
reported 131.0 ms for an external WGSL edit and 148.1 ms for a parameter
default edit (one sample each, dev profile). Both included a rendered frame
and met the 300 ms reload budget. The captured swatch changed from red to
green without restarting the game. These are the dev command's
rebuild-plus-swap timings, not a throughput benchmark or a bound for large
projects. Direct material-parameter editing in Tune remains separate from
this file-reload path.

### 16.4 Preview

`og ui preview <screen|component>` renders a contact sheet: styles ×
breakpoints (desktop, phone portrait and landscape, tablet) × locales (the
game's, pseudo, CJK, RTL) × states × text scale, from viewmodel fixtures
(`fixtures/*.ogt`), with a JSON index of what each cell shows. `--film
<path>.<timeline|transition>` renders a filmstrip of a motion at chosen
ticks. These are what an agent reads (and the platform shows) before it
claims a UI is done.

### 16.5 Check

`og ui check` (also run by `og polish` and available to `og test`) reports,
each with the element paths and the prop to change:

- overlap not declared (§6.2), across breakpoints and locales;
- text clipped or truncated without ellipsis and a tooltip; overflow under
  the pseudo-locale and at 150 % text (200 % deferred, §15.2);
- elements outside the safe area or under the shell's covered region;
- touch targets under 44 CSS px on a coarse pointer;
- contrast: an error only for live text under 3:1 on pixels (7:1 in the
  high-contrast mode), measured as `og polish` does; under 4.5:1 it is a
  warning for the art director to judge, and dimmed, decorative and
  over-the-scene HUD text is judged by eye, not by the number;
- text under the minimum size;
- focus: an interactive element unreachable by pad from the default
  focus, a focus trap, a layer without Back;
- accessibility names missing or in the wrong locale;
- hierarchy: more than one primary action in a layer;
- feedback: an interactive element without hover or press states, or
  without a sound in a styled game;
- motion: durations outside the style's bounds, missing reduced-motion
  fallbacks, input refused longer than the style's entrance;
- art: slots still on their placeholder at publish; images whose focal
  point falls outside the slot's crop;
- performance: element counts, draws and per-tick times over §15.1.

### 16.6 Knowledge

The `og-ui` skill is rewritten for documents, components, states, motion,
slots and the checks. A new `ui-craft` skill holds the design craft:
hierarchy, layout grammar per genre, motion personality, the conventions
of each genre from Appendix A. `og-taste`'s UI parts point to it.

## 17. Extensions (phase 0, approved 2026-11-04)

The core owns elements, layout, motion, drawing, documents, style grammar
and common components. Genre kits and style packs are directories, not
host plugins. They use only the public TypeScript API and the document
and style formats. Their versions are independent of the engine version.
The first kits are RPG, shooter, puzzle/casual, gacha, strategy/TD and
racing. Fantasy, scifi, cartoon and pixel are the flagship styles; the
other packs are frozen. Contrast is not an acceptance check.

### 17.1 Manifest and installation

An extension has an `extension.json` at its root:

```json
{
  "format": 1,
  "kind": "kit",
  "name": "racing",
  "version": "1.0.0",
  "description": "Race HUDs and results, built from common components",
  "skill": "SKILL.md",
  "entry": "index.ts",
  "types": "index.d.ts"
}
```

`kind` is `kit` or `style`. Names use lowercase ASCII letters, digits and
hyphens, starting with a letter. `format` versions this manifest contract;
`version` versions the extension. All paths are relative to the directory,
without parent traversal or symlinks. `skill` is required and explains
discovery, calling, modifying, and creating a component or style. `entry`
and `types` are optional together for a document-only kit. A style has
`style.ogt` at its root and no executable entry. Documents are discovered
recursively by their `type screen` or `type component` header, rather than
listed in a shared registry. Asset references within a pack are relative;
shared built-in fonts may still use `@styles/`.

Official kits live in `packages/engine/extensions/<name>` and ship there.
Official styles stay in `templates/styles/<name>` and ship in the engine's
`styles/<name>`. Both use the same manifest. A game installs an editable
copy with `og ui add <name|directory> [project]` or
`og style add <name|directory> [project]`. Kits go in
`assets/ui-kits/<name>`, styles in `assets/styles/<name>`. An existing
installation is never silently replaced. No network fetch or code execution
is part of discovery or installation.

`og ui list [project]` lists installed and bundled extensions, their
versions, entry/type/skill paths and every document. The installed copy
wins for a kind/name pair. `og ui eject <kit>/<document> [project]` copies
an extension document to `assets/ui/<kit>/<document>.ogt`, alongside its
companion documents and assets so relative references survive (§17.5). Existing companion edits
are preserved; `--force` replaces only the requested document. Legacy library
screen names remain supported. The installed documents are ordinary
assets: `og.load.document(world, "ui-kits/racing/results.ogt")` uses the
same cooked document on both hosts, with no runtime filesystem discovery.
A look's `ui "fantasy"` finds the game's installed
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
The on-demand `og ui sheet <pack|kit>` renders baseline pages on the pack's
own ground at desktop, phone and 150% text. It is not a gate step.

`og ui sheet <pack|kit> [project] [--out DIR]` discovers extensions in the
project (default `.`), including installed edits. Packs use the engine's
shipped baseline project and its fixed component/pattern/HUD pages. A kit
supplies `sheet/project.ogt` inside its own directory: its `ui { screen
"name" "commands" }` entries are its fixed review pages. Those commands
must establish each page independently, on the kit's style ground rather
than a game scene. No kit name or page list is registered in the core.
The tool copies its inputs to disposable working files, cooks and bundles
them, and renders four cells per page: desktop and phone at 100% and 150%.
It writes full-resolution frames, UI dumps, contact sheets and `index.json`.
Missing captures or missing baseline page roots fail the command; visual
judgment belongs to the reviewer, with no contrast score or reference gate.

### Documents and arbitrary widgets

`tabs` is a document control with `items`, two-way `selected`, and a
`changed` event. For any other `og.ui` widget or an extension component,
documents declare a `slot` and game code fills it after mounting:

```ts
og.ui.screen("garage", vm);
if (og.ui.openSlot("garage/Speed")) {
  racing.speedometer("Speedometer", vm.speed);
  og.ui.close();
}
```

This is the general document-to-widget bridge on both hosts. The slot
owns placement; the typed call owns widget arguments and interactions.
It avoids a second, untyped widget dispatch API or executable strings in
documents. The same bridge accepts a common component such as
`og.ui.slider`, without registering a new document element kind.

### 17.4 Phase 0 dressing and typography contracts

Painted art is addressed by **selector**, not by an engine slot enum.
An `art-requests.json` slot with a new name supplies `size`, `insets` and
`dress: ["Tabs.tab[selected]"]`; `kind: "styleFrame"` packs a nine-slice,
`"ornament"` a whole piece, and `"styleButton"` a five-cell state sheet.
`tint: true` on a frame keeps the widget's colour instead of baking it
into the art. The historical slot names remain defaults; `bar` is an
opt-in tinted `Meter.fill`, never an extra request imposed on every pack.
Tabs, selected rows, ribbons and banners need no new core registration.

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
Old files derive selection from controlStrong, onSelection from ink and
accent from highlight. `sceneInk`, `scenePlate` and `sceneShadow` are
scheme-aware HUD roles (legacy fallback: ink, hud and shadow).
`og.ui.label(..., { onScene: true })` uses sceneInk and sceneShadow;
a HUD container with `background: { token: "scenePlate" }` is the plate
treatment. Games tune these three roles in their style, not per widget.

`og.ui.icon(id, "base", { secondary: "detail", secondaryTint: { token:
"accent" } })` overlays two named icons with independent inks. The
secondary tint defaults to accent; both layers share the icon's bounds.
Document icons accept the same properties.

The stacking order is fixed for authored draws and style dressings:
ground/fills → rim → content/selection → hollow focus → children/raised
badges. Focus never covers art with a background-coloured cut. Put a
decoration that must cover children in a child of its own, not a rim.

`typography` is the drawing hierarchy, including weight, tracking and
breakpoint sizes. The legacy ramp names are aliases, not a second set
of authored numbers:

| Typography role | Legacy ramp names |
| --- | --- |
| caption | xs, caption |
| label | sm, meta |
| body | base, body, reading |
| numeric | lg |
| heading | xl, xl2 |
| title | xl3, xl4, displaySm, displayMd |
| display | xl5, xl6, displayLg |
| hero | xl7, xl8, displayXl, displayXl2 |

`hero` is optional for existing packs; without it, it is 1.5 times
`display` at each breakpoint, up to the format's 400 px limit. The
player's text scale multiplies after selecting the breakpoint, once.
Buttons continue to use `button` (falling back to `label`).

Type roles accept `cjkTracking <em>` and `case upper` after the font and
OpenType tags, for example `label 16 22 0.06 600 body case upper`.
`case none` leaves the source unchanged; a bare `case` remains the
OpenType case-sensitive-forms tag. Uppercase uses ICU's full Unicode
mapping in the active `og.i18n` locale before measuring and wrapping.
It affects displayed labels and rich text, not probe/accessibility source
strings, editable values, cursor offsets or inline icons and key caps.
Greek accents, Turkic dotted I and Lithuanian combining dots therefore
follow their language, on both hosts.

On the next minor upgrade, `og upgrade`'s `ui-typography` codemod converts
v1 styles with the existing style upgrader. For v2 it comments out a
superseded `ramp`, keeping its old numbers for review and preserving other
source bytes. It lists `typography` for manual visual review: two old
aliases with different sizes cannot both keep those sizes when they name
one role. No version bump occurs in a feature stream; the integrating
release head moves these unreleased breaking entries once for the wave.

### 17.5 The first-wave genre kits (phase 1)

The genre components left `og.ui` for six bundled kits under
`packages/engine/extensions/`. Each kit's `index.ts` registers one typed
namespace, and its `types.ts` holds the data types that used to be
generated from `ui-hud.ogt` and `ui-patterns.ogt`:

| Kit | Members |
|---|---|
| `rpg` | `inventoryGrid`, `paperdoll`, `questLog`, `questTracker`, `conversation`, `unitFrame`, `partyFrames`, `raidFrames`, `castBar`, `turnOrder` |
| `shooter` | `crosshair`, `ammoCounter`, `killfeed`, `damageDirection`, `lowHealthVignette`, `scoreboard`, `loadout` |
| `puzzle` | `sagaMap`, `goalChips` |
| `gacha` | `gachaBanner`, `pullSequence` |
| `strategy` | `productionQueue`, `selectionPanel`, `commandCard`, `rtsConsole`, `dragSelect`, `rtsCommands`, `wavePreview`, `resourceBar` |
| `racing` | `speedometer`, `standingsTower`, `lapTimer`, `rearView`, `raceBanner`, `itemSlot`, `driftScore`, `garage` |

Generic lists, slots, meters, timers (`hudTimer`), stars (`starBar`),
counters, stores, rewards, cards and world tags stay in `og.ui`. A game
imports a kit explicitly, from the bundled copy or from its installed
copy after `og ui add <kit>`:

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

`og upgrade` crosses this change with the `ui-kits` codemod. Its
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
kit, never the kit's TypeScript, manifest, skill or review sheet: those
come with `og ui add`. `og ui sheet <kit>` installs the kit into its
working copy of the sheet, so a page loads the kit's documents from
`ui-kits/<kit>/` exactly as a game does.

Data a genre varies is open. A gacha with its own tiers (SSR, SR, R, or a
ladder of ten) gives a featured unit or a pull item a `tier` (`{ label,
rank, color }`) over the engine's `Rarity`: its colour tints the reveal,
its label names it, and its rank orders the summary and, from 5, shines
and bursts as legendary does. Scoreboards take any number of teams, a
resource bar any resources, and a standings tower any field.

### 17.6 Phase 1 common-component additions

- **Locale-aware uppercase.** `LabelDesc.uppercase` (the document label's
  `uppercase` prop) cases a label with the same ICU casing as a type
  role's `case upper`, before it is measured, so Turkish and Greek case
  right; the probe keeps the source text. Components and kits ask for it
  instead of `String.toUpperCase`.
- **Fallback icons.** `UiDesc.fallbackIcons` names an icon set that draws
  what the atlas (the game's icons, else the style's) lacks. A pack keeps
  the icons it paints; `og.ui.icon`, rich text, action glyphs and art
  placeholders read the same chain.
- **Scroll fades in documents.** The `scroll` kind takes `fade` and
  `fadeInto`, as `og.ui.scrollArea` does, so a cut row reads as more.
- **Touch controls.** `joystick`, `dpad` and `actionButton` take
  `disabled` and dress as discs. Each keeps the finger that went down on
  it, saved in the UI state (version 16, upgraded from 15 on load); the
  stick rises from 0 past its dead zone and the d-pad reads eight ways.
- **Scene text.** Text drawn straight over the scene passes `onScene`
  (`LabelDesc.onScene`, the ui-parts `text()` and `btn()`): ink roles
  become `sceneInk` at the same emphasis, with `sceneShadow` separation,
  so a light style's dark ink stays readable over a dark scene.
- **Reward choice.** Its cards scroll between the header and the actions,
  at `<id>/Cards/Options/<key>` (was `<id>/Options/<key>`).

## 17a. The existing coverage kit

`templates/ui-kit` is rebuilt as the coverage proof:

- one **genre showcase** per genre family (action and platformer, shooter
  and battle royale, MOBA and arena, RPG (JRPG, ARPG, CRPG), MMO,
  roguelike and card, strategy (RTS, 4X, tactics, tower defense, auto
  battler), management (city, idle, life), puzzle and casual, gacha and
  collection, rhythm, racing and sports, fighting and party, narrative and
  VN, survival, horror, open world), each a playable set of its screens,
  HUD and world UI, driven by fixtures;
- every seed style on its own pages (§17.3), over its plain ground;
- tests per showcase: reference frames on metal, filmstrips of its key
  motions, scripted interactions (pad, touch, keys), and a clean `og ui
  check`.

The archetypes and examples move to the new components and screens in the
same phase; their FINDINGS about templates close with it.

## 18. Phases

Each phase ends merged on main with its tests, reference frames on metal
and the functional lane green; the wave's end runs `scripts/wave-end.sh`.
At most five threads run at once (AGENTS.md); the split below keeps their
files apart.

| Phase | Content | Owns | Depends on |
|---|---|---|---|
| U1 | UI render pass: shapes, paints, strokes, effects, masks, blend, layers and materials, vector paths, text effects, UI particles, the viewport element | og-render `ui`, og-ui `draw.rs`, og-cook `vector`/`ui-material` | — |
| U2 | Element model (transform, opacity, z, absolute, grid, stack, container queries, overlap rule) and motion (springs, states, timelines, choreography, presence, layout and shared-element animation, gestures) | og-ui `ui.rs`, `layout.rs`, `motion/` | — |
| U3 | Documents and cook, data binding and formatting, the layer stack, actions and glyphs, Tune, probe writes, hot reload | og-ui `doc/`, `bind.rs`, `layers.rs`; og-cook; og-api; og-probe; og-runtime | U2 |
| U4 | Styles v2 (tokens, part dressings, motion, sound and haptic sets), `og style upgrade`, art slots, the art brief's UI kinds, the style generator, the seed styles | og-ui `style_file.rs`; knowledge `brief.ts`; og-cli `style.rs`; `templates/styles` | U1 (paints), U2 (states) |
| U5 | World-space UI: anchored elements, mass elements, world panels, meta UI | og-ui `world.rs`, og-render | U1, U2 |
| U6 | Component library and screen documents for every row of the appendices | `packages/engine/src/ui/`, og-ui widgets | U3, U4 |
| U7 | `og ui preview`, `og ui check`, `og polish` integration, the skills | og-cli, `packages/engine/src/node`, knowledge | U3 |
| U8 | The kit's genre showcases, seed styles on them, archetypes and examples migrated, references, wave-end | templates, examples | U5, U6, U7 |

Waves: **A** U1 ∥ U2 ∥ (U4's grammar and the brief's kinds); **B** U3 ∥ U5
∥ U7's checks on the element model; **C** U4 rest ∥ U6 (split by
category) ∥ U7 rest; **D** U8, split by genre families.

The removal of the fixed templates and of `type style` version 1 at load
(after `og style upgrade`) are breaking changes, released as a minor
version with `changelog.json` breaking entries.

## 19. Open for the owner

- The seed styles (§12.5): which families, how many beyond twelve.
- The document grammar (§10.1) after U2 lands.
- The budgets of §15.1 once U1 and U2 measure.
- Whether `og ui check` failures block `og publish` or only warn in the
  first release.

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

Aspects are starting values for the brief, to be checked against current
platform guidelines in U4.

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

The survey of og-ui at `c030bfe3`; the research behind the appendices and
§5–§8 drew on Unreal's CommonUI, UMG and its Viewmodel, Slate
invalidation and the Retainer Box; Unity's UI Toolkit and its comparison
with uGUI; Godot's containers, themes, Tween and SubViewport; Rive's state
machines and data binding; NoesisGUI's Visual State Manager; Coherent
Gameface; SwiftUI's springs and matched geometry; Flutter's Hero and
staggered animations; the Game UI Database; the Xbox Accessibility
Guidelines (101–118) and the Game Accessibility Guidelines; GDC talks on
the UI of Dead Space (2013), Hearthstone (2015) and Into the Breach
(2019).
