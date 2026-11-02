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

### 5.4 Masks and layers

- Any element can clip its children to its own shape (rounded, squircle,
  polygon, path), antialiased, not only to its rectangle.
- Alpha masks from an image or a gradient: a list fading at its scroll
  edges, a portrait in a brush-stroke shape.
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
down to a minimum type step, then ellipsis), and inline images and icons in
rich text with links that open pinnable nested tooltips (§11).

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
  (left, top, right, bottom, centre), or anchored (the nine anchors of the
  safe area, an entity in the world §14, or another element's rectangle).
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
  both paths (§16.4). This makes the phone-sheet bugs of §1 failures.

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
  fonts per role, with CJK and fallback faces as today.
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

### 12.4 Sound and haptic sets

The events of §7.8, each with one sound or a set chosen at random without
repeats, volume and pitch ranges, and a haptic pattern.

### 12.5 Built-in styles and generated styles

Six built-in styles are not enough. The engine ships at least twelve seed
styles covering the look families of `og-taste` §12 (for example ornate
fantasy, storybook, tactical military, sci-fi holographic, cyberpunk neon,
casual cartoon, kawaii and anime gacha, pixel retro, paper and craft,
minimal modern, diegetic horror, sports broadcast, ink and eastern
brush); the list is settled with the owner in phase U4.

A **style generator** (`og style new`) makes a game its own style from the
look, the premise and a few choices (palette seed, shape language,
ornament level, motion personality): tokens and procedural dressings
first, then the art slots of its frames, buttons, ornaments and icon set
filled through the art brief (§13.3). A generated style passes the same
contrast and completeness tests as a built-in one.

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
- the UI pass in ≤ 1 ms of GPU at 2560×1440, with layers and backdrops;
- a 10,000-row table scrolls without a frame over budget.

### 15.2 Accessibility

The accessibility tree, its ARIA mirror and reduced motion and transparency
stay, and the bar rises to the Xbox Accessibility Guidelines:

- text scale to 200 % without loss of content; minimum text 18 px at 1080p
  on desktop and phone-equivalent sizes on touch, for menus, body text and
  controls. A dense HUD (strategy, MMO, simulation readouts) may declare
  its text dense and go smaller, down to 12 px at 1080p, provided it grows
  with the player's text scale and passes the contrast rule (the owner,
  2026-11-02);
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

The UI is held to the level of shipped commercial games, and the
coordinator reviews every UI merge from U6 on against it, frame by frame:

- Each genre showcase is read beside screenshots of two shipped games of
  its genre, and a screen that reads as a prototype beside them is held.
- Every changed screen is read at desktop 1920×1080, tablet, phone
  portrait 390×844 and phone landscape, in every built-in style, and under
  the pseudo-locale, a CJK locale, a right-to-left locale and 200 % text.
- `og ui check` reports nothing. A declared overlap is not a way to merge
  a known defect: the kit's Gallery overlap is fixed, not carried.
- Every interactive part is captured idle, hovered, pressed, focused and
  disabled, and each transition is recorded as a clip and watched.
- No placeholder survives: no flat-fill loading screen, no button whose
  edge does not show, secondary text at 4.5:1 on its own scrim, and no two
  seed styles that read as one.
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
  the pseudo-locale and at 200 % text;
- elements outside the safe area or under the shell's covered region;
- touch targets under 44 CSS px on a coarse pointer;
- contrast under 4.5:1 (7:1 in high contrast), measured on pixels as
  `og polish` does;
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

## 17. The kit

`templates/ui-kit` is rebuilt as the coverage proof:

- one **genre showcase** per genre family (action and platformer, shooter
  and battle royale, MOBA and arena, RPG (JRPG, ARPG, CRPG), MMO,
  roguelike and card, strategy (RTS, 4X, tactics, tower defense, auto
  battler), management (city, idle, life), puzzle and casual, gacha and
  collection, rhythm, racing and sports, fighting and party, narrative and
  VN, survival, horror, open world), each a playable set of its screens,
  HUD and world UI, driven by fixtures;
- every seed style worn by every showcase, desktop and phone;
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
