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
ui.lock().frame().emit(&EmitDesc { atlas, layer: 0, z: 0.0 }, &mut scene.sprites, &mut scene.texts);
scene.camera = og_ui::camera(&viewport);
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

`UiFrame::emit` appends to a `RenderScene`'s `sprites` and `texts`:

- **One atlas.** Every rectangle, rounded or not, is a 9-slice of one 64×64
  white disc (`atlas_texture()`, coverage supersampled, mips), so panels,
  buttons, tracks, thumbs, rings and the caret share one sprite run; a
  texture-less image is a solid slice of it.
- **Painter levels.** og-render sorts by (layer, kind, texture). Each command
  gets a level one above every earlier command it overlaps that batches
  differently, and the level of an overlapping earlier command of its own
  batch; commands of different batches on one level never overlap, so the
  sort reorders nothing visible. The sample screen is 216 sprites and 173
  glyphs in **4 sprite draws** (2 levels × atlas + font; `levels()` reports
  the bound). Put the UI above game sprites with `EmitDesc::layer`.
- **Clipping on the CPU.** og-render exposes neither scissor nor stencil, so
  quads and images are cut to the clip rectangle with their UVs adjusted, and
  text is clipped by whole lines vertically (a half-visible line is hidden)
  and whole glyphs horizontally.
- **Camera.** `camera(&viewport)` is the orthographic camera mapping CSS
  pixels to the target (CSS (x, y) → world (x, −y)); render at
  `width·dpr × height·dpr` with `Tonemap::None` and 0 EV so token colours come
  out as authored.

**What og-render should add** (the minimal change, for its owner):

1. A **screen-space overlay**: `RenderScene::overlay { sprites, texts }` drawn
   after the tonemap into the output, with its own orthographic projection in
   output pixels, no depth test, straight alpha. Today UI sprites are
   world-space: they pass through exposure and the tonemap and are
   depth-tested against the 3D scene, so a UI over a **3D** game cannot be
   drawn in the same frame (2D games with `Tonemap::None` work now, as the
   sample does). One extra pass, reusing the sprite pipeline with another
   view-projection.
2. A **clip rectangle per instance** (`clip: Option<[f32; 4]>` on `Sprite` and
   `Text`, 16 bytes in the instance, `discard` outside in `sprite.wgsl`): exact
   clipping of glyphs and rounded corners at scroll edges without breaking
   runs (a per-run scissor would split runs per clip). og-ui then drops its
   CPU clipping of text lines.

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

## TypeScript shape proposed for the IDL `ui` module

For the IDL agent to transcribe into `crates/og-api/api.ogt` (module `ui`,
TypeScript `og.ui`). Lengths are `cssPx`; every desc's zero value is the
default stated.

```ts
// Sizes along one axis.
type Size =
  | { kind: "fit"; min?: number; max?: number }      // default: fit 0..∞
  | { kind: "grow"; min?: number; max?: number }
  | { kind: "fixed"; px: number }
  | { kind: "percent"; fraction: number };             // ratio of the parent's inner size

interface Insets { left: number; top: number; right: number; bottom: number }   // all 0

enum Direction { row, column }                        // row
enum Align { start, center, end, spaceBetween }       // start (main axis)
enum Cross { start, center, end, stretch }            // start (cross axis)
enum TextAlign { start, center, end }                 // start
enum FontRole { body, display, mono }                 // body
enum ButtonVariant { secondary, primary, quiet, danger }  // secondary
enum ControlSize { small, medium, large }             // medium
enum Scheme { light, dark }                           // dark
enum DensityMode { auto, compact, comfortable, touch }    // auto
// The palette keys of tokens.json (generated from the vendored file, as theme.rs is).
enum ThemeColor { bg, bgSubtle, surface, surfaceRaised, surfaceHover, fill, fillStrong, thumb,
  border, borderStrong, fg, fgMuted, fgSubtle, fgInverse, accent, accentHover, accentActive,
  accentSoft, accentFg, accentInk, focus, danger, success, warn, info, stage, stageSurface,
  stageRaised, stageBorder, stageFg, stageMuted, stageDanger /* … every color key */ }
// The type steps of tokens.json: xs, meta, caption, sm, body, reading, base, lg, xl, xl2 … xl8,
// displaySm, displayMd, displayLg, displayXl, displayXl2.
enum TypeStepName { /* generated */ }

interface Paint { token?: ThemeColor; color?: Color /* sRGB as authored, alpha 1 */; alpha?: number /* default 1 */ }

interface ElementDesc {        // desc true
  direction: Direction; width: Size; height: Size; padding: Insets; gap: number;
  align: Align; cross: Cross; wrap: boolean; scroll: boolean; interactive: boolean;
  background?: Paint; radius: number;
}
interface LabelDesc { step?: TypeStepName /* body */; color?: Paint /* fg */; align: TextAlign; nowrap: boolean; width: Size; font: FontRole }
interface ButtonDesc { variant: ButtonVariant; size: ControlSize; disabled: boolean; width?: Size /* fit */ }
interface PanelDesc { row: boolean; width: Size; height: Size; padding?: Insets /* space.lg */; gap?: number /* space.md */; align: Align; cross: Cross; background?: Paint /* surface */ }
interface ScrollDesc { width: Size; height: Size; padding: Insets; gap: number }
interface ImageDesc { texture?: render.Texture /* none: solid */; uv?: [number, number, number, number] /* whole */; tint?: Paint /* white */; width: Size; height: Size }
interface ProgressDesc { width?: Size /* grow */ }
interface SliderDesc { min: number; max: number /* min == max: 0..1 */; step: number /* 0: continuous */; width?: Size /* grow */ }
interface FieldDesc { placeholder: string; width?: Size /* grow */; maxChars: number /* 0: no limit */ }

interface Response { hovered: boolean; pressed: boolean; clicked: boolean; changed: boolean; focused: boolean; submitted: boolean }

interface Viewport { width: number; height: number; dpr: number; safeArea: Insets; coarsePointer: boolean }
interface UiDesc { scheme: Scheme; density: DensityMode; font?: load.Font; displayFont?: load.Font; monoFont?: load.Font }

// og.ui — call inside a system; the engine runs begin() before the game's
// systems with og.Input and the shell's viewport, and end() after them,
// then emits the frame into the render scene and the probe.
function configure(desc: UiDesc): void;
function viewport(): Viewport;                                   // this tick's play surface
function open(id: string, desc?: ElementDesc): void;
function close(): Response;
function row(id: string, desc: ElementDesc, build: () => void): void;      // TS sugar over open/close
function column(id: string, desc: ElementDesc, build: () => void): void;   // TS sugar
function panel(id: string, desc: PanelDesc, build: () => void): void;      // TS sugar
function scroll(id: string, desc: ScrollDesc, build: () => void): void;    // TS sugar
function label(id: string, text: string, desc?: LabelDesc): void;
function button(id: string, text: string, desc?: ButtonDesc): Response;
function image(id: string, desc: ImageDesc): void;
function progress(id: string, value: number /* ratio */, desc?: ProgressDesc): void;
function slider(id: string, value: number, desc?: SliderDesc): { response: Response; value: number };
function toggle(id: string, on: boolean, label: string): { response: Response; on: boolean };
function textField(id: string, value: string, desc?: FieldDesc): { response: Response; value: string };
function tooltip(text: string): void;
function idi(name: string, index: number): string;              // "name#index"
function theme(): Theme;                                         // the generated tokens, read-only (palette, space, radius, typeRamp, sizes, durationsMs)
```

The value-carrying widgets return the new value instead of taking a
reference (TypeScript has no `&mut`). `row`/`column`/`panel`/`scroll` with a
closure are TypeScript sugar over `open`/`close`, so the binding needs no
callbacks across the boundary. The command channel needs no new actions.

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

Each was checked by breaking the code it covers (grow caps, the
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
- **Animations**: hover, press and focus transitions and overlay entry on the
  token springs and easings (`SPRING_DEFAULT`, `EASE_OUT`), ticked
  deterministically; momentum scrolling on `SPRING_MOMENTUM`; caret blink.
- **Accessibility**: an accessibility tree mirrored to the shell (roles,
  names, values, states) so screen readers and the DOM can read the canvas
  UI; keyboard focus order (Tab, arrows) and visible focus for every control;
  `prefers-reduced-motion` and contrast modes from the shell.
- Materials: glass (blur needs a render-graph pass), shadows and the dark
  edge light; icons (iconoir) as an MTSDF atlas.
- Layout: horizontal and two-axis scrolling, nested scroll chaining,
  aspect-ratio images, absolute/floating elements beyond tooltips (menus,
  dialogs, toasts), z-order control.
- The og-render overlay and per-instance clip above; then drop CPU clipping.
