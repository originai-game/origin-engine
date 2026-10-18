# og-ui

The Origin Engine UI (`docs/design.md` §2.7, M6 in `docs/phase-1.md`): an
immediate-mode toolkit whose tree is declared every tick, laid out
Clay-style, themed by the platform's design tokens
(`origingame/docs/design-language.md`), drawn through og-render's sprite and
MSDF text batches, and addressable by the probe through stable string ids
(`touch_move_to`, `move_to_text`, `dump_ui`).

It depends on og-core, og-ecs, og-input, og-render and og-probe (no
default features) only. No layout or GUI library: the layout is ~600 lines in `src/layout.rs`.

![The sample screen](references/metal/sample-screen.png)

## Using it

```rust
let ui = SharedUi::new(Ui::new(UiDesc { fonts: Fonts { body: UiFont::new(handle, &msdf), ..Default::default() }, ..Default::default() }));
let (desc, run) = ui.system("ui", || viewport, |ui, _ctx| {
    ui.panel("Menu", PanelDesc::default(), |ui| {
        ui.label("Title", "Origin", LabelDesc { step: Some(ui.theme().type_ramp.display_sm), ..Default::default() });
        if ui.button("Start", "Start", ButtonDesc { variant: ButtonVariant::Primary, ..Default::default() }).clicked {
            // start
        }
        ui.tooltip("Starts a new run");
    });
    Ok(())
});
world.add_system(desc, run)?;          // builds the UI inside the tick from og.Input
probe.set_ui(ui.probe());              // dump_ui, touch_move_to, move_to_text
// after the tick, when rendering:
let atlas = scene.add_texture(og_ui::atlas_texture());   // once
let mut overlay = Overlay::new(viewport.width, viewport.height, viewport.dpr);
ui.lock().frame().emit(&EmitDesc { atlas, layer: 0 }, &mut overlay);
scene.overlay = Some(overlay);                           // over the finished frame, any camera
```

`Ui::begin(input, viewport)` … `Ui::end()` is the whole protocol; widgets
return a `Response { hovered, pressed, clicked, changed, focused, submitted }`
whose edges last one tick. Interaction is decided against the previous
tick's rectangles (the immediate-mode rule), which are also what the probe
resolves against, so a scripted `touch_move_to` and a click agree.

## Widgets

| Call | What it is | Probe text |
|---|---|---|
| `open(id, Element)` / `close() -> Response` | a container (`row`, `column`, `container` take a closure) | — |
| `panel(id, PanelDesc, f)` | a card: `surface`, `radius.card`, `space.lg` padding, `space.md` gap, a column unless `row` | — |
| `scroll(id, ScrollDesc, f)` | a vertical scroll column: wheel or drag scrolls it, it clips its content, a capsule indicator shows the position | — |
| `label(id, text, LabelDesc)` | MSDF text; wraps at its width (word breaks, any CJK character, long words between characters) unless `nowrap`; `align` start/center/end; type step from the ramp | the text |
| `button(id, text, ButtonDesc) -> Response` | a capsule of `h-control-sm/md/lg`; `Secondary` (neutral fill, default), `Primary` (amber, one per screen), `Quiet`, `Danger`; `disabled` turns neutral and inert; pressed shrinks by `press-scale` | the label |
| `image(id, ImageDesc)` | a texture region (`uv`, `tint`), or a solid rectangle without a texture | — |
| `progress(id, value, ProgressDesc)` | a capsule track filled to 0–1 | — |
| `slider(id, &mut f32, SliderDesc) -> Response` | track, amber fill, thumb; a press anywhere on it sets the value, dragging follows; `min`/`max`/`step` | — |
| `toggle(id, &mut bool, label) -> Response` | a switch (amber on) with a label; a click flips it | the label |
| `text_field(id, &mut String, &FieldDesc) -> Response` | one line: a press focuses it with the caret at the press, typed text inserts at the caret, Backspace/Delete/←/→/Home/End edit, Enter submits and leaves, Escape leaves, a press elsewhere blurs; placeholder; `max_chars`; focus ring in `focus` | the text, or the placeholder while empty |
| `tooltip(text)` | for the element just built: shown below it (above without room, clamped to the safe area) once the pointer has rested on it for `motion.duration.slow`; drawn over everything | the text |

Any container with `interactive: true` reports its response from `close()`.

### The game widget set (`src/widgets.rs`)

| Call | What it is | Probe ids and text |
|---|---|---|
| `tabs(id, items, &mut usize, TabsDesc)` | a segmented control: `fill` track, the chosen tab on `thumb` | `Tab#i`, the labels |
| `select(id, options, &mut usize, &SelectDesc)` | a field-like trigger and chevron; opens `<id>.menu` (a popover layer and focus scope) with `Option#i`; a press outside or Back closes it | the chosen label |
| `stepper(id, &mut f32, StepperDesc)` | `Dec`, `Value`, `Inc`; bounded; Left/Right step while a button is focused | `−`, the value, `+` |
| `radio(id, options, &mut usize, RadioDesc)` | `Option#i` rows: a ring, amber with a dot when chosen | the labels |
| `checkbox(id, &mut bool, label)`, `checkbox_list(id, labels, &mut [bool])` | a box, amber with a tick (two rotated bars) | the label(s) |
| `chip(id, text, ChipDesc)` | a capsule on `fill`; selected inverts to `fg` | the text |
| `badge(id, text, Tone)` | a small capsule in a tone (neutral, success, warn, danger, info, accent, inverse) | the text |
| `key_value(id, key, value, mono)` | `Key` (fg-muted) and `Value` (end) on one row | both |
| `divider(id, vertical)`, `spacer(id, Option<Size>)` | a hairline in `border`; empty space | — |
| `skeleton(id, SkeletonDesc)`, `spinner(id, size)` | the neutral fill with a passing sheen (still under reduced motion); eight dots turning by the tick | — |
| `icon(id, name, IconDesc)` | a cell of the icon atlas (`set_icons`) by name, tinted | the name |
| `meter(id, value, max, &MeterDesc)` | health (green, warn under 30 %, danger under 15 %), mana (blue hue), xp (purple hue), accent or custom; a drop leaves a pale trail that follows after `fast`, a rise grows over `slow`, by the tick; notches; a label inside | the label |
| `cooldown(id, remaining, &CooldownDesc)` | a disc whose amber ring fills clockwise from the top as the cooldown runs out (two rotated half discs), full when ready; a label or icon in the middle | the label |
| `minimap(id, &MinimapDesc)` | the map texture in a `border-strong` frame, dots for markers (the player amber and ringed), N, a zone label | `Map`, `Zone` |
| `dialogue(id, &DialogueDesc)` | portrait slot (texture, icon or initial), speaker in `accent-ink`, the line typed at `chars_per_second` by the tick; a press finishes the line, the next answers `advanced` | `Body/Line` shows what is typed |
| `joystick(id, JoystickDesc)` | a base and a knob following a press that started on it; answers −1..1 with a dead zone; at least three touch targets wide | — |
| `dpad(id, DpadDesc)` | `Up`, `Down`, `Left`, `Right` arms (each ≥ 44 CSS px) answering which are held | the arm names |
| `action_button(id, text, &ActionButtonDesc)` | a round touch button (≥ 44): `pressed` while held, `clicked` on a tap; primary is amber | the text |
| `open_list`/`list_item`/`close_list`, `list(id, ListDesc, selected, row)` | a virtualised scroll list: only rows in view (and one each side) are built, fillers keep the full scroll range; `Row#i` rows select and focus | `Row#i` |
| `open_grid`/`grid_item`/`close_list`, `grid(...)` | the same for a wrapping grid of equal cells (`Cell#i`) | `Cell#i` |
| `spotlight(id, Option<Rect>, SpotlightDesc)` | the scrim on four sides of a hole (`Top`, `Left`, `Right`, `Bottom`, `Hole`) with an amber edge; presses on the shades stop there, presses in the hole reach what is under it | — |
| `toast(text, ToastDesc)`, `dismiss_toast(id)` | a queue in the UI state: at most three up (older leave first), 3.2 s (6 s danger), enter and leave by the tick, a press dismisses; stacked at `set_toast_anchor` | `Toasts/Toast#n`, the text |
| `memory(id, initial)`, `remember(id, value)` | a number kept for an id in the UI state (a template's tab) | — |

Widget memory between ticks (a select being open, a meter's trail, the
dialogue's start, a transition's progress, a drag's source) lives in
`State::anims`, keyed by the element's path hash and dropped when the widget
is not built; it, the toast queue, the UI clock, queued navigation and the
focus scope are all in `save_state`, so a restored world replays the UI.

**Drag and drop.** A press on a list row or grid cell marks it the source; a
release over another row or cell answers `Response::dropped = Some(source)`
there, and `dragging(id)` answers the source while the press is held; the
cell under a drag shows the focus ring.

## Layers

Elements outside the tree, laid out after it and drawn over it in rank order:
anchored groups (`anchor`, rank 1; `over: true` raises one to rank 3), dialogs
(`open_modal`/`close_modal`, `modal`: a scrim backdrop that takes presses,
padded by the safe area, with a centred `Panel`, rank 2), popovers (a
select's menu, rank 3), the toast stack (rank 4) and tooltips (rank 5). A
layer's probe path is its own id. Anchored groups sit at one of nine points of
the safe area (`Anchor::TopLeft` … `BottomRight`, `Center`), inset by the
density's gutter (or `margin`), moved by `offset`.

## Focus navigation (`src/nav.rs`)

One focus for pointer, keyboard and gamepad. Intents come from the arrow
keys, Tab/Shift+Tab, Enter, Space and Escape (`InputFrame::keys`, `shift`),
from a gamepad (`InputFrame::gamepad`: d-pad and A/B edges, the left stick
past half-way once, then after `motion.duration.slow` every
`motion.duration.fast`) and from `navigate(Nav)` (queued for the next tick).

- Directions pick the nearest focusable control whose centre lies that way,
  scoring distance along it plus three times the gap across it; with none,
  focus wraps to the far end of the same row or column. Next/Prev walk
  reading (draw) order and wrap.
- Activate presses the focused control (`Response::clicked`); Left/Right
  adjust a focused slider or stepper; while a text field is edited, Left,
  Right, Enter, Space and Escape belong to it.
- Back closes the topmost dialog or menu, else `back()` answers true once.
- A dialog or open menu is a focus scope: while it is the topmost, focus
  stays inside; when it opens, focus moves to its first control (or the one a
  widget asks for, the select's chosen option); when it closes, focus returns.
- A press focuses the control without the ring; navigation shows the ring
  (amber `focus`, a gap of what is behind, then the control).
- Navigation that lands on a control outside its scroll container's view
  scrolls it in (rows just outside a virtualised list are reachable).
- `set_navigation(false)` turns arrows and Tab off for a gameplay screen whose
  keys move the player (the HUD templates do); an open scope turns it on.

`InputFrame::from_world` reads og-input's `og.Gamepad` (pad 0:
`GamepadFrame::from_gamepad`, the press edges of the standard buttons and
the left stick) on both hosts. The browser shell still maps a pad's buttons
to keys too (A Space, B Escape, d-pad and stick arrows, Back Tab, Start
Enter) as the fallback for a page without the resource; an intent that
arrives both ways in one tick is taken once (`nav.rs`).

**Several touches.** `InputFrame::pointers` is og-input's `og.Pointers`
(pointer 0 is `og.Input`'s pointer; 1 to 4 are further touches). The touch
controls (`joystick`, `dpad`, `action_button`) answer any of them: a stick
keeps the finger that started on it, a button answers whichever is over it,
so a thumb on the stick and another on Hit work together
(`templates/ui-kit/tests/multitouch.og.ts`). Everything else is
single-pointer.

## Motion (`src/motion.rs`)

`transition(id, show, TransitionDesc, f)` (`open_transition` + `close`) moves
its content in and out: fade, slide up/down/left/right (by `space.xl`) or
scale (from 92 %), over `motion.duration.base` unless given, eased by the
token curves (`ease_out` entering, `ease_exit` leaving, cubic Béziers solved
by bisection: arithmetic only), advancing one tick per frame. Content is
built while visible (it stays visible while it leaves) and takes no input
while it moves. Dialogs scale in over a fading scrim; toasts slide. Under
`Theme::reduced_motion` every movement becomes a fade and skeleton sheens stop.
The transform applies to the laid-out rectangles (so drawing, hit testing and
the probe agree), to opacity and to text size.

## Skins (`src/skin.rs`)

The tokens are the default look; a game's own is a **skin** the theme
carries by value (`Theme::skin`, `Copy`). `Skin::default()` dresses nothing,
so every draw path is exactly the token one without a skin (the reference
frames of `tests/gfx.rs`, `tests/kit_gfx.rs` and the UI kit's
`frames-desktop`/`frames-phone` are unchanged, and were pixel-identical
before and after the skin layer: `tests/evidence/E3/break-and-restore.md`).

A skin file is an `.ogt` of `type skin` (`parse_skin`, grammar in its doc
comment; og-cook validates it and cooks it as an `og.UiSkin` blob,
`og.load.skin` loads it with its assets). It declares:

| Part | What it does |
|---|---|
| `base`, `palette`, `radius`, `ramp`, `durations` | the token scheme it starts from and the ThemeColor, radius, TypeStepName and duration values it changes: these go into the `Theme` itself, so every widget follows |
| `atlas "<id>" <w> <h>`, `scale` | the texture every frame, ornament and tile is cut from |
| `slice <part> x y w h <inset…> [tint\|authored]` | a nine-slice frame per element class (`SkinPart`): `panel` (dialogs, anchored screen panels), `card` (`Ui::panel`), `button`, `buttonHover`, `buttonPressed`, `buttonDisabled`, `field`, `slot` (grid cells), `tooltip`, `capsule` (HUD capsules, toasts), `track` and `fill` (progress). `tint` frames are white art multiplied by the element's colour (so primary and secondary buttons keep their palette); `authored` ones keep their own colours |
| `soft` | a blurred white rectangle: soft shadows and glows |
| `corner` + `corners <parts>`, `divider`, `flourish` | ornaments: four mirrored corners on the named parts, an ornament over `Ui::divider`, a flourish beside a heading (`LabelDesc::heading`; both sides when centred). Ink `authored`, `accent`, `fg`, `muted`, `border` or `focus` |
| `tile x y w h <tw> <th> <alpha> [inset] <parts>` | a texture tiled over the parts' fill (parchment, scanlines, grain) |
| `shadow <part> dx dy blur spread "#rrggbbaa"`, `glow <part> size ink alpha` | per class (button states share the button's); blur 0 is a hard shadow |
| `textShadow <role> dx dy blur colour`, `textGlow <role> size ink alpha` | per font role (`body`, `display`, `numeric`) |
| `press sx sy dy`, `hover dy scale` | the press squash and sink, the hover lift (eased over `instant`) |
| `fonts { body display numeric fallback }`, `icons "<id>" cols rows { names … }` | fonts per role (numeric is the `Mono` role) and the CJK `fallback`: a text whose role font lacks one of its characters is measured and drawn in the fallback (`Fonts::for_text`); the icon set |
| `motion { enter stagger toast progress shake pop }` | dialog and panel entrance (`pop` springs past 1 and settles), the stagger of a panel's controls (shortened so a long list ends within one more entrance), the toast arrival, eased progress bars, the refused-press shake, the count-up landing pop |
| `sound <event> "<id>"` | UI sounds (below) |

Six packs to start from live in `templates/ui-kit/assets/skins/` (pixel,
cartoon, fantasy, scifi, paper, neon), drawn by
`templates/ui-kit/tools/make-skins.py`; `tests/packs.rs` holds each to 4.5:1
for every ink a template puts on a skinned surface, measured over the frame
art really under the text.

## Juice and screen transitions (`src/juice.rs`)

- `counter(id, value, CounterDesc)`: a label that counts (ease-out cubic,
  by the tick, f64) from what it shows to `value` whenever `value` changes;
  the first count starts at `from` (none: no count). Grouped with commas,
  decimals, prefix and suffix; a skin's `motion.pop` bumps it on landing.
- `burst(BurstDesc)`: rays and sparks from the element built just before,
  once, from when it first appears (the results stars); off under reduced
  motion.
- `screen_transition(id, wanted, ScreenTransitionDesc) -> ScreenTransition`:
  call every tick with the screen the game wants and build `shown`. A change
  starts a cover over the old screen (half the duration), switches `shown`
  at full cover, and reveals the new one. `Fade`, `Wipe`, `Iris` (2 px
  bands), `Dissolve` (24 px cells in a fixed hash order) and `Slide`, in a
  colour or a texture region stretched over the surface, from any side; drawn
  after every other command (over every layer) through the overlay pass. The
  state (`shown`, `wanted`, the start tick) is in `Ui::save_state`. Under
  `reduced_motion` every kind is a fade.

## UI sounds

The UI queues an event when the skin (or `UiDesc.sounds` through the host)
has a sound for it: `Hover` (the pointer reaches a navigable control),
`Press`, `Open` and `Close` (a dialog's first tick each way), `Success`
(a success toast), `Error` (a danger toast or a refused press).
`Ui::take_sounds()` hands them to the host; the runtime's `og.ui.sound`
system (after `og.audio`) plays each through og-audio on the `ui` bus, one
voice per event kept in the UI state (`Ui::voices_mut`), restarted on a
repeat. The source starts in og-audio's next pass, one tick after the input.

## Responsive layout and the safe area

`Viewport::breakpoint()` / `Ui::breakpoint()`: `PhonePortrait` and
`PhoneLandscape` (shorter side under 600), `Tablet` (under 1024 wide),
`Desktop`. Density `Auto` already raises controls to touch size on a coarse
pointer or a surface under 768 px; touch widgets never go under the touch
`h-control-sm` (44). The root and every anchored group keep to the safe area;
dialog backdrops cover the whole surface and pad their content by it.

## Theme variants

`Theme::dark()` (default) and `light()`; `Theme::high_contrast(scheme)`
remaps `fg-muted` → `fg`, `fg-subtle` → `fg-muted`, `fill` → `fill-strong`,
`fill-strong` → `border-strong`, `border` → `border-strong`, deepens the scrim
and outlines every control in `border-strong` (token values only);
`with_accent(Srgba)` and `with_hue(HueName)` set a game's accent and derive
its hover, active, ink, soft and focus tones the way the tokens derive them
from amber (the same mix towards white or black, the same alpha; amber in
gives the token palette back exactly), picking `accent-fg` or the light `fg`
as the ink on it by contrast. `reduced_motion` is a theme flag. The generator
still owns every value (`src/theme.rs`); variants are mappings over it.



## Layout

Sizes per axis: `Size::Fit { min, max }` (content; the default),
`Size::Grow { min, max }` (content, then the parent's free space),
`Size::Fixed(px)`, `Size::Percent(f)` (of the parent's inner size). An
`Element` has `direction` (`Row` default, `Column`), `padding`, `gap`,
`align` (main axis: start, center, end, space-between), `cross` (start,
center, end, stretch), `wrap` (rows), `scroll`, `interactive`, `background`,
`radius`.

One layout per frame, in five walks over the tree (`src/layout.rs`):

1. widths bottom-up: preferred (content) and minimum widths (a wrapping
   text's longest word; a wrapping row's widest child);
2. widths top-down: a row gives each child its preferred width, then grows
   `Grow` children into the free space by water-filling (the smallest rise
   first, until they level or reach their `max`), or, when it overflows,
   shrinks `Fit`/`Grow` children the same way from the widest down to their
   minimums; a wrapping row breaks its children into lines at its inner width
   and grows within each line; a column gives `Grow` (or `Cross::Stretch`)
   children its inner width and caps the others at it; text then wraps at its
   final width;
3. heights bottom-up from the wrapped lines (`lineHeight` of the type step,
   the CJK leading when the text has CJK) and the line breaks;
4. heights top-down: the same grow/shrink along a column; a scroll container
   never shrinks its children;
5. positions (alignment, scroll offsets clamped to the content), clip
   rectangles (the intersection of every scrolling ancestor), and device-pixel
   snapping of every rectangle (`Viewport::dpr`).

Everything is in CSS pixels of the play surface (origin top left, y down),
the unit of `og.Input` and the probe. The root is the viewport, padded by
`Viewport::safe_area` (its background still covers the whole surface).
`Theme::density` `Auto` picks the touch control sizes on a coarse pointer or a
viewport under 768 px, compact otherwise, as the design language says.

## Ids and paths

```text
id    := name | name '#' index      ui_id("Start") → "Start", ui_idi("Slot", 3) → "Slot#3"
path  := id ('/' id)*                "Inventory/Slot#3": the element, after its nearest ancestors
```

- An element's probe id is its own id; `touch_move_to` takes an id or a path
  (og-probe's `find_by_id`: the last segment is the element, earlier segments
  its nearest ancestors, topmost match first).
- An element without an id is `<kind>#<n>` (`row#0`, `label#2`), `n` counting
  that kind's anonymous siblings, so an unchanged layout keeps its ids.
- Siblings must differ: a repeat becomes `<id>~<n>` with a frame warning.
  A `/` inside an id becomes `_` with a warning. `:` and `;` are legal but
  must be escaped in command scripts (`\:`).
- The interaction state is keyed by the hash of the full path, so two
  `Buy` buttons under `Row#1` and `Row#2` are distinct.

## Theme

`src/theme.rs` is **generated** from the platform's
`packages/tokens/tokens.json`, vendored verbatim as `tokens/tokens.json`:

```sh
crates/og-ui/scripts/gen-theme.sh            # copy the platform's tokens.json ($OG_TOKENS or ../origingame), regenerate
crates/og-ui/scripts/gen-theme.sh --offline  # regenerate from the vendored copy
crates/og-ui/scripts/gen-theme.sh --check    # fail if the vendored copy or theme.rs is stale
```

The generator (`scripts/theme_codegen.rs`, run as the `gen-theme` example)
emits both palettes (`LIGHT`, `DARK` = `color` overlaid with `dark.color`;
every semantic key as sRGB bytes plus alpha), both hue sets, the spacing
scale, the radii, the type ramp (size, leading, tracking, CJK leading), the
three densities (touch inherits compact's gutter and section), durations,
springs, the cubic easings and `press-scale`. `Theme::light()`/`dark()`
(dark is the default: games run on a stage) read only those constants;
widgets map semantic tokens the way the design language does (amber only
for the primary, the slider/progress/switch fill and the caret; neutral
fills for secondary controls and selection; `fg-subtle` placeholders;
`surface-raised` tooltips). `tests/theme.rs` regenerates from the vendored
file and fails on any difference, so a hand edit or a token change without
regeneration does not pass. Colours convert to linear for og-render
(`Srgba::linear`).

## Drawing

`UiFrame::emit` appends to an og-render `Overlay` (CSS pixels, origin top
left, y down), drawn after every post effect, so token colours come out
exactly as authored over any game, at any camera, sharp under TAA:

- **One atlas.** Every rectangle, rounded or not, is a 9-slice of one 64×64
  white disc (`atlas_texture()`, coverage supersampled, mips), so panels,
  buttons, tracks, thumbs, rings and the caret share one sprite run; a
  texture-less image is a solid slice of it.
- **Painter levels.** og-render sorts by (layer, kind, texture). Each command
  gets a level one above every earlier command it overlaps that batches
  differently, and the level of an overlapping earlier command of its own
  batch; commands of different batches on one level never overlap, so the
  sort reorders nothing visible. The sample screen is 231 sprites and 181
  glyphs in **4 sprite draws** over 4 painter levels (`levels()` reports the
  bound; the test allows two draws per level). Put the UI above other
  overlay sprites with `EmitDesc::layer`.
- **Clipping per pixel.** Each scroll container and clipped panel adds its
  rectangle once to `Overlay::clips` (deduplicated); every quad, image,
  rotated shape and glyph inside names it in `clip`, and the overlay pass
  discards outside it. A row half out of a list is cut mid-glyph; rotated
  shapes (`DrawCmd::Bar`, `DrawCmd::Half`: chevrons, ticks, the d-pad's
  arrows, cooldown pies) are cut at the edge instead of dropped
  (`tests/gfx.rs` `a_scroll_container_clips_everything_it_draws` reads the
  pixels on both sides of the edge).

The runtime (`og_runtime::modules::emit`) writes the frame into
`RenderScene::overlay` with scale 0 (fit the output's height), so the UI
shows over 3D games at any camera; the camera-plane workaround is gone.
Recapturing on the overlay moved only clipped content: the store list's top
row now shows the lower half of "Item 0" and its Buy button, where the CPU
clip hid the whole line (`tests/evidence/E2/ui-sample-screen-before-after.png`);
eleven kit references moved the same way and were recaptured on `metal`.

## Input

The UI reads one `InputFrame` per tick from any `InputSource`. `OgInput`
is the world resource it reads: og-input's `og.Input` (version 2), whose
one-tick edges (`button_pressed`/`button_released`, `pressed`) are the
press and release edges the widgets use; og-ui keeps no input state of its
own. Pointing it at another resource is the one line
`pub type OgInput = og_input::Input;` in `src/input.rs` plus an
`impl InputSource`. Typed text: `og.Input` has key codes but no
characters, so text is derived from the printable keys pressed this tick
(US layout, Shift held); injected `modifiers` never reach the resource, so
`type:Hello` types `hello`. og-input owes a typed-text stream (the shell's
`beforeinput`/composition events) for real keyboards and IME.

og-probe still ships its interim `og.Input` (version 1) and `WorldInput`;
a host must create the probe with `Probe::with_input` over og-input's layer
until og-probe re-exports og-input (its README says how). The tests do that
with a 20-line bridge (`tests/support/mod.rs`, `ProbeInput`) that converts
the probe's `InjectedInput` through its protocol 5 JSON.

## Probe contract

`SharedUi::probe()` is og-probe's `UiTree`: `elements(world)` returns the
last completed frame's elements:

- **draw order** (later is on top), so og-probe's "topmost first" means the
  last drawn match; floating tooltips come last;
- `id` the element's own id; `parent` its parent's id (empty under the root;
  a tooltip's parent is the element it describes, so `Buy/tooltip` resolves);
- `text` what the element shows: a label's text, a button's or toggle's
  label, a field's text or its placeholder while empty, a tooltip's text;
- bounds in CSS pixels, **cut to the clip** (what is on screen); an element
  scrolled or clipped entirely out of view is not listed (a probe click on it
  would land on something else); a zero-size element is listed where it sits.

`touch_move_to:<id or path>`, `move_to_text:[=]text[@N]` and `dump_ui[:path]`
then work as `crates/og-probe/README.md` specifies. `UiFrame::element(path)`
and `element_by_text(query)` are the same lookups in Rust.

**Determinism.** Layout and interaction use only `+ − × ÷`, comparisons and
rounding; the UI reads no clock (delays are ticks from `ticks_per_second`).
`Ui::save_state()` / `restore_state(bytes)` carry what persists between
ticks (hot, active, focus, caret, hover ticks, pointer, scroll offsets and
the last frame's hit rectangles); a host snapshot that stores them with the
world replays the UI exactly (`tests/probe.rs` proves it through the probe's
snapshot and restore).

## In TypeScript

The IDL module `ui` (`crates/og-api/api.ogt`, `og.ui` in
`packages/engine/types/index.d.ts`, `packages/knowledge/wiki/ui.md`) carries
every call above with the same names in lowerCamelCase: value-carrying
widgets return the new value (`og.ui.tabs(...).selected`,
`og.ui.stepper(...).value`); containers have `open*`/`close` pairs and
`host false` sugar taking a build callback (`row`, `column`, `panel`,
`scroll`, `anchor`, `transition`, `modal`, `list`, `grid`); `toast`,
`dismissToast`, `navigate` and `previewViewport` take the world and work
between ticks; `frameTheme`, `frameViewport`, `breakpoint`, `hasIcon`,
`seconds`, `tick` read the frame being built. The bindings are in
`og_runtime::modules` (both hosts call them).

**The UI kit** (`packages/engine/src/ui-kit`, merged into `og.ui`): screen
templates written in TypeScript over those calls, each one call per tick with
data in and what happened out, declared in the IDL as `host false`:
`mainMenu`, `pauseMenu`, `settingsScreen` (Audio, Graphics, Controls,
Language over `GameSettings`, `onChange`/`onSave` hooks), `platformerHud`,
`shooterHud`, `puzzleHud`, `idleHud`, `racingHud` (with touch controls on
phones and tablets), `cutscene`, `shopScreen`, `inventoryScreen` (drag and
drop), `levelSelect`, `resultsScreen` (stars landing, the score counting up,
a share hook), `leaderboardScreen` (loading, error and empty states),
`tutorialOverlay` (spotlight and callout), `loadingScreen` (real progress or
a spinner, tips), `confirmDialog`, `alertDialog`, and the toast service
`og.ui.toast`. `templates/ui-kit` shows them all and is the reference source
of `references/<tier>/kit-*.png`; `packages/knowledge/skills/og-ui` teaches a
Maker which to pick and how to theme it.

## Tests

```sh
export CARGO_TARGET_DIR=$PWD/target/agent-ui
cargo test -p og-ui                                   # everything below
OG_UI_GOLDEN=1  cargo test -p og-ui --test layout     # re-record the layout golden
OG_UI_CAPTURE=1 cargo test -p og-ui --test gfx        # re-record this tier's reference frame
cargo check --target wasm32-unknown-unknown -p og-ui
```

| Test | Asserts |
|---|---|
| `tests/layout.rs` | a nested tree (grow with a cap, wrap with a growing child, shrink-and-wrap, percent, centre, space-between, stretch, the safe area) gives the rectangles reasoned in the test and `tests/golden/layout.json` |
| `tests/probe.rs` | through og-probe's command channel only: a click needs press and release inside (press-inside-release-outside, press-outside-release-inside and held presses do not click); `dump_ui` lists every visible element with bounds and text, in draw order; `move_to_text` exact, substring, `@N`, `@-N` and misses (which cancel the next touch); wheel scrolling moves rows, clips them to the list and stops at the content's end; the text field, toggle and slider edit game state; restoring the world and `save_state` replays identical frames |
| `tests/gfx.rs` | the sample screen rendered headless matches `references/<tier>/sample-screen.png` (og-tests' tolerance: 32/255 per channel, ≤ 0.5 % outliers, RMSE ≤ 0.01) in at most 2 sprite draws per painter level, no missing glyph; a scroll container cuts every sprite and text line it draws to its rectangle |
| `tests/theme.rs` | `src/theme.rs` equals the generator's output for the vendored tokens |
| `tests/widgets.rs` | focus: arrows pick the aligned neighbour and wrap, Tab/Shift+Tab walk reading order and wrap, Enter and Space press, the gamepad d-pad and A, the stick at once then after 24 and every 11 ticks; a dialog takes focus, keeps Tab inside, blocks presses to what is under it, closes on the backdrop and Back (not when locked) and returns focus; a select opens, chooses (mouse and keys) and closes on a press outside; a 1000-row list builds at most 13 rows, reaches its last row and scrolls a keyboard-focused row into view; toasts cap at three, leave after 3.2 s and come back with a restored state; a transition advances 1/15 per tick, is inert while moving, leaves; a meter's trail follows a drop and two runs draw identical frames; the dialogue types 15 characters in 30 ticks at 30/s, a press finishes, the next advances; anchors keep to the safe area and touch controls to 44 px; a joystick answers (1, 0) at its edge; amber in gives the token palette back, a hue accent derives lighter hovers in dark, high contrast takes `fg` for muted ink; a drag from cell 0 to cell 6 drops once and a click is no drop |
| `tests/skin.rs` | a skinned card is nine atlas pieces that cover it with fixed corners, its shadow and corners reach past it; the default theme emits no skin piece; a skin file names the wrong line (a typo, a region outside the atlas, a glow without `soft`, a bad colour); every screen-transition kind keeps the old screen while covering, switches at full cover, ends, and is one fade per tick under reduced motion; a counter starts at `from`, never goes back, lands at 900 ms and replays after a restore; a refused press shakes and comes to rest, queues `Error` only, an enabled press queues `Press` |
| `tests/packs.rs` | the six packs parse, name only existing assets, and every ink a template draws on a skinned surface is at least 4.5:1 over the frame art under it (and `fg` on a selected tab) |
| `tests/kit_gfx.rs` | the widget set (tabs, an open select menu with the focus ring, stepper, radio, checkboxes, chips, badges, meters with trail, notches and label, skeleton, four cooldown rings, stick, d-pad, action button, spinner, a toast) matches `references/<tier>/widget-set.png` in 4 sprite draws; the pies are the expected half-disc angles |
| `templates/ui-kit/tests/*.og.ts` | through the command channel on the native host: every template at 1280×720 and 390×844@3 matches `references/<tier>/kit-{desktop,phone}-<screen>.png`; every screen in each style pack matches its cell of `references/<tier>/kit-skin-<pack>.png`; an iris transition, the count-up and UI voices (`juice`); touch, keyboard and gamepad flows, drag and drop, the phone's safe area and 44 px targets, the tutorial spotlight (`templates/ui-kit/README.md`) |

Each was checked by breaking the code it covers (the kit round's breaks are
in `tests/evidence/E1/ui-break-and-restore.md`; earlier: grow caps, the
release-inside rule, label probe text, element order, scroll clipping, wheel
scrolling, a theme constant, the primary colour, the saved scroll state,
Backspace, the toggle's click). A tier without its own reference compares
with the `metal` one and says so; the `vulkan-cpu` reference (CI's lavapipe)
is recorded with `OG_UI_CAPTURE=1` in the sandbox og-tests uses. The test
font is og-tests' committed MSDF atlas (Noto Sans SC subset, OFL).

## What phase 2 owes

- **Rich text**: inline runs of colour, weight and size; the display and
  rounded faces cooked as their own atlases; tracking from the type ramp
  (needs per-glyph spacing in og-render's `Text`); kerning and shaping
  (rustybuzz, with og-render); RTL; ellipsis truncation; selection in fields;
  multi-line text areas; IME composition.
- **Animations**: springs (`SPRING_DEFAULT`) for the skin's hover and press
  (eased curves today); momentum scrolling on `SPRING_MOMENTUM`; caret blink.
- **Skins**: phone references for the packs (`kit-skin-*` are desktop
  only); per-glyph tracking from a skin's ramp; text glows are four offset
  copies (a real blur needs an MSDF outline in og-render); nine-slices
  stretch rather than tile their edges.
  (Overlay entry and exit, toasts, meters and typing run on the token
  easings by the tick since the kit round.)
- **Accessibility**: an accessibility tree mirrored to the shell (roles,
  names, values, states) so screen readers and the DOM can read the canvas
  UI; `prefers-reduced-motion` and `prefers-contrast` read from the shell
  into `UiDesc` (the theme flags exist; nothing sets them from the page yet).
- Materials: glass (blur needs a render-graph pass) and the dark edge
  light (skins have shadows and glows); icons (iconoir) as an MTSDF atlas.
- Layout: horizontal and two-axis scrolling, nested scroll chaining,
  aspect-ratio images.
- **Input**: pads beyond the first; several touches for widgets other than
  the touch controls (two-finger scroll, pinch).
- `vulkan-cpu` references for og-ui's own scenes (`sample-screen`,
  `widget-set`) and the kit templates: only `metal` exists, other tiers
  compare with it.
- og-probe re-exports og-input since 5c79c75, so the `ProbeInput` bridge in
  `tests/support/mod.rs` and the "og-probe still ships its interim
  `og.Input`" note in §Input are out of date; the bridge can go.
