# Game UI contract

This document defines the boundaries and acceptance rules for core UI,
the built-in TSX library and style packs. API signatures belong in the
generated API; author workflows belong in the guides. CSS terminology
describes layout and paint semantics, not a stylesheet authoring API.

## 1. Scope and ownership

| Owner | Responsibility | Boundary |
| --- | --- | --- |
| Core (`og-ui-next`) | Elements, layout, paint, layers, focus, navigation, gestures, scrolling, text editing, motion, accessibility and typed class records | One implementation contract on native and web; controls are composed in TSX |
| Built-in library | `packages/engine/src/ui/controls/`, `components/` and `screens/`: primitives composed into controls, presentation patterns and screens | No gameplay rules, literal visual values or private engine imports |
| Style pack | Palette, typography, spacing, shape, art, icons, motion, sound and haptics in `pack.json`, optionally compiled from a `theme.json` recipe | No gameplay code or domain state; style and scheme may change at runtime |
| Game | Content, rules, domain validation and applying UI intents | Owns its screens and chosen style assets |
| Platform | Accounts, money, publishing, threads, agent orchestration and engine version selection | Consumes released archives and the public host ABI, never engine internals |

The renderer owns the UI pass. `og-ui-next` owns the retained tree, layout,
interaction and class interpretation. The TypeScript UI entry point is
`@origingame/engine/ui`; world configuration and readback are `og.ui`.
Game authors never need Rust. Web rendering stays in the Worker.

The library exists so a game takes a finished piece rather than redrawing
it. A common need with no built-in is a library gap, closed by a shared
component, not a private duplicate inside another built-in. One control
kind has one visual grammar. A game's look is its pack's tokens and slots.

UI presentation is outside `world_hash`; rendering never decides a
simulation result. Gameplay effects cross `og.input.intent(world, event,
payload)`. Domain rules, rewards, legal targets, damage, purchases and
clocks that decide outcomes stay in the game.

## 2. Composition contract

The normative wire, primitive, class, input and accessibility contracts
are in [`crates/og-ui-next/CONVENTIONS.md`](../crates/og-ui-next/CONVENTIONS.md).
The author workflow is the [screens guide](../crates/og-api/guides/screens.md).
There are no `.svelte` documents, `<style>` sheets, HTML host tags or CSS
selector cascade.

- **Structure:** a `.tsx` function returns a tree and exports its props
  interface. `nsh-jsx` compiles JSX into Solid-style renderer calls. A
  component body runs once; read reactive props in JSX expressions,
  accessor functions or `createMemo`, not a one-time local snapshot.
- **Primitives:** PascalCase imports `View`, `Text`, `Image`, `Icon`,
  `Scroll`, `Input`, `Line`, `Particles`. Control flow is `Show`, keyed
  `For`, and `Switch` / `Match`. `ErrorBoundary` handles synchronous
  rendering errors. Lowercase host tags are errors.
- **Look:** Tailwind v4.1 class strings resolve in Rust against the active
  pack to typed records. Unknown or unsupported utilities are reported by
  `og check` (OG9105). Put `packClass("Component", "slot", variant?)`
  after the component's own classes for pack artwork, motion and sound.
- **State:** `data-*`, `classList` and variants (`hover:`, `active:`,
  `focus-visible:`, `focus-within:`, `disabled:`, `group-*:`) dress state;
  `starting:` / `exit:` dress presence. There are no `bind:`, `class:` or
  `style:` directives. A controlled input uses `value` and `onChange`.
- **Content:** props and children carry content; optional content can be
  a function returning JSX. There are no snippet references or document
  name resolution. Components are ordinary explicit imports.
- **Mount:** `mount(world, Component, props, { layer })` returns an
  unmount callback. Layers are `hud`, `menu`, `modal`, `popover`, `toast`,
  `tooltip`, `system`. A mount supplies the surface-sized root; screens
  normally fill it with `absolute inset-0`. Choosing `menu` does not by
  itself pause simulation; the game owns pause and screen state.
- **Events:** `onPress`, `onTap`, pointer, wheel, focus, key, change,
  long-press, hold, drag/drop, pinch, rotate, swipe, context-menu and
  scroll handlers run at the tick boundary, with `UiEvent` fields
  `{ world, type, x, y, key, value }`. `onAction={{ confirm, back }}`
  handles named input actions where focus is. Payload meanings are in
  `HostProps`, not browser DOM events.
- **Strings:** import typed locale declarations (`import "../../content/ui/screen.en.ts"`)
  and use `t(key, params)`, `formatNumber`, `formatCompact`,
  `formatDuration`, `formatCurrency` inside the mount. Never concatenate
  translated fragments.

### 2.1 Retained authoring model

Named presentation stores follow the `svelte/store` contract: `writable`,
`readable`, `derived`, `get`, plus a keyed collection. This is a data-model
comparison, not Svelte markup or a Svelte runtime. `useStore(store)`
subscribes for a component's lifetime and returns a signal accessor.
Signals, memos, effects, batching, cleanup and context are exported from
the UI entry point. There are no async UI resources or Suspense.

Gameplay initialization belongs in `og.game`'s `start(world)`. UI
registration belongs in `ui(world)`, once after start and again after a
hot-reload restore, never per tick. Registration creates scopes and mounts;
stable store IDs preserve values and collection order on re-registration.
Previous registration mounts are disposed and remounted on reload.
Local component signals are not a replacement for checkpointed stores.

For example, a named store reaches a TSX component through `useStore`:

```tsx
import { og } from "@origingame/engine";
import type { UiReadable, World } from "@origingame/engine";
import { Text, formatNumber, mount, useStore } from "@origingame/engine/ui";

function GoldReadout(props: { gold: UiReadable<number> }) {
  const gold = useStore(props.gold);
  return <Text class="text-title tabular-nums">{formatNumber(gold())}</Text>;
}

export default og.game({
  ui(world: World) {
    const inventory = og.ui.scope(world, "inventory");
    const gold = inventory.writable("gold", 0);
    mount(world, GoldReadout, { gold }, { layer: "hud" });
  },
});
```

The same scope/store IDs reconnect to their saved values on reload;
`initial` is not a reset. A gameplay system publishes its presentation
value with `gold.set(value)`; a UI callback requests a gameplay change
through an intent rather than deciding an award or purchase itself.

`og.ui.scope(world, id)` takes an explicit, stable, nonempty ID:

```ts
writable<T>(id: string, initial: T): UiWritable<T>;
readable<T>(id: string, initial: T, start?: UiStart<T>): UiReadable<T>;
derived<S extends UiStores, T>(id: string, stores: S, compute: UiDerive<S, T>, initial?: T): UiReadable<T>;
get<T>(store: UiReadable<T>): T;
collection<T>(id: string, items: readonly T[], desc?: UiCollectionDesc): UiCollection<T>;
takeErrors(): readonly string[];
dispose(): Result<void>;
```

Subscription immediately reports the current value. Derived stores have
explicit dependencies and settle before subscribers; failed derivations
keep the last good value or the initial fallback, otherwise report an
error and yield `undefined`. Setters return `Result<void>`. Errors name
their scope and ID and remain readable in publish builds.

Collections key by `desc.key` (default `id`); `keys()`, `row(key)`,
`insert(row, before?)`, `remove(key)`, `move(key, before?)`, and
`patch(key, fields)` preserve stable identity and reject duplicate keys
or key-changing patches. In TSX, `For` keeps nodes while their key stays;
its row and index arguments are accessors. An object's `id` is the default
key; use `key` for another identity. Pass row IDs explicitly through
handlers, rather than assuming `UiEvent.key` is a collection key.

Reactive reads track their current dependencies, including branch changes.
Unmount disposes nodes and subscriptions. The renderer sends only changed
node operations; layout, motion and paint own their retained state. There
is no document parse/restyle lifecycle or serializable snippet mount.
Presentation checkpoints and saves stay outside `world_hash`; saves use
the `og-save:4` envelope. UI feedback is presentation-only and does not
create gameplay entities or replay a sound on restore.

### 2.2 Text and structured content

`Text` with multiple strings, nested `Text` spans, inline `Icon` or `Image`
forms one paragraph: shared shaping, wrapping, alignment and clipping.
Spans keep their own style and events; `onPress` on a span hits its glyphs.
`ruby` annotates a span. `select-text` enables read-only selection and
copy; `writing-vertical-rl` lays text out in upright columns. See
CONVENTIONS.md §Paragraphs for limits.

Dialogue supplies localized plain text and attribute spans
`{ name, position, length, properties }`; offsets count Unicode scalar
values. The dialogue runtime resolves markup before delivery. These are
not `RichSpan` values and `Text` has no `spans` prop or `::part` mapping.
`RpgConversation` takes a plain `line`, speaker, replies and optional
tick-driven reveal rate. `RichText` takes explicit runs (`text`, `tone`,
`bold`, `italic`, `icon`, `keyword`, `gloss`). A game adapting dialogue
attributes must explicitly choose its presentation; the UI never treats
localized text as executable markup.

### 2.3 Text input and accessibility

`Input` takes `value`, `placeholder`, `type` (`text`, `password`, `number`,
`email`, `url`, `tel`, `search`), `inputMode`, `autoComplete`, `secret`,
`multiline`, `rows`, `maxLength`, `readOnly`, and standard `HostProps`.
`onChange` receives the whole edited string in `event.value`.

- Text arrives through `og.TextInput` with printable-key fallback.
  Backspace/Delete and arrows step grapheme clusters; Ctrl/Alt steps
  words, Shift selects, Ctrl/Cmd+A/C/X selects/copies/cuts. Multiline
  Enter inserts a newline.
- IME preedit is underlined inline and commits before `onChange`; blur
  clears uncommitted preedit. Password/secret masks the text and blocks
  copy/cut. Read-only keeps focus, selection and copy. `maxLength` counts
  Unicode scalars.
- While focused, frame `text` describes the text target and surface-pixel
  rect; `copy` carries copied text. The web text-entry bridge positions
  its hidden input/textarea for keyboards, IME, autofill and clipboard.

Roles, names, descriptions, checked/mixed, pressed, selected, expanded,
busy, invalid, ranges and modal state populate AccessKit and the web
accessibility mirror. `live="polite"` / `"assertive"` announce changes;
`status` and `log` default polite, `alert` assertive; status and alert are
atomic. `hidden` the prop hides from accessibility only; `hidden` the
class removes the visual node. Focus groups, memory and roving Tab order
are explicit props, not private navigation code in each component.

### 2.4 Built-in controls

Reusable controls live in `packages/engine/src/ui/controls/`, exported
from `@origingame/engine/ui/components` and individual control paths.
Their exported props and `fixtures` are authoritative. The library also
includes radio groups, toolbars, sheets, popovers, menus, numeric fields,
rich text and other mechanisms; the generated `components`
guide lists the complete current exports. `og ui eject controls/<name>`
copies one for customization.

| Control | Pack slots | Role / behaviour |
| --- | --- | --- |
| `Art` | pack nine-slice class | Stretched or nine-sliced artwork |
| `Button` | `Button.plate[.primary\|danger]` | `button`; variants, pressed state, hold-to-confirm |
| `Checkbox` | `Checkbox.indicator`, `Checkbox.mark`, `Checkbox.dash` | `checkbox`; boolean or mixed |
| `Counter` | `Count.roll`, `Counter.value` | Animated numeric readout |
| `Dialog` | `Modal.scrim`, `Dialog.frame`, `Modal.frame`, `Dialog.title`, `Dialog.body`, `Dialog.actions` | `dialog` / `alertdialog`; modal focus, back, body and footer |
| `Meter`, `Progress` | `Bar.fill` | `meter` / `progressbar` |
| `Select` | `Select.trigger`, `Select.list`, `Select.option` | Combobox/listbox/options, keyboard selection |
| `Slider` | `Slider.track`, `Slider.range`, `Slider.thumb` | Pointer drag and Arrow/Home/End stepping |
| `Switch` | `Switch.track`, `Switch.thumb` | Binary toggle |
| `Tabs`, `Tab` | `Tab.list`, `Tab.item`, `Tab.mark`, `Tab.indicator` | Roving tablist with active indicator |
| `Tooltip` | `Tooltip.frame` | Label/detail against a named UI node |

### 2.5 World, geometry and effects

`anchor` on a node attaches it to an entity, a world point `[x,y,z]` or a
named UI node. `lift` offsets it; translate classes set its pivot. World
anchors support `occlusion`, `declutter`, `priority`, `clamp`,
`clampMargin`, `scaleAt`, `scaleMin`, `scaleMax`, `maxDistance` and `lod`.
Class feedback is `data-clamped`, `data-edge-*`, `data-occluded` and
`data-lod-near|mid|far`. Anchors follow drawn camera/entity poses without
feeding back into simulation. `spotlight` shades outside a named UI node
and intercepts presses there. CONVENTIONS.md §World owns placement rules.

Ground marks (selection rings, telegraphs, placement ghosts and paths)
are `og.Decal` / `og.render.lines`, not `<ground>` UI hosts. A camera
render target is an `Image src`; it does not supply orbit controls or
pointer rays. Lines use numeric `points` and optional `viewBox` / `fit`;
inline SVG markup and SVG path parsing are not TSX host APIs. Use texture
or pack artwork for shapes that can be images. Radial sweeps use `wipe`,
`wipeStart`, `wipeCcw` on a box image.

`Particles` draws an `og.load.emitter` handle in its box, in CSS px with
+y up. `seed`, `delay` and `restart` control presentation playback;
reduced motion draws none. Scene-only particle features do not draw in
UI. `Celebration` and `Flourish` compose engine emitters, pack ink and
sound. There are no `<og-emitter>`, `<og-overlay>` or CSS burst directives.
Per-element custom shader attachment is not a TSX prop; see the
[materials guide](../crates/og-api/guides/ui-materials.md) for that boundary.

On-screen sticks, pads and buttons use `touchStick`, `touchPad`,
`touchButton`, `deadZone` and `knob`, feeding the same input actions as
keys/pads. `floatingStick` shifts a stick's ring/subtree to touch-down at
paint without moving its activation rect. `touchLook` instead sends drag
deltas per fixed tick in CSS px (x right, y up), zero while stationary or
released; the action map checkpoints prior pointer positions and applies
device sensitivity/inversion. `TouchControls` exposes `floatingStick`
for move only (ignored for `pad`) and `lookPad` for drag look instead of
the second stick. Drag/drop uses string
`draggable` data and `dropTarget` IDs; gesture payloads and keyboard
equivalents are in CONVENTIONS.md §Input. Do not substitute browser event
names or DOM payloads.

## 3. Style contract

### 3.1 Tokens and literal values

Built-in visual values are theme steps or roles, never local tuning:
colour, type, spacing, size, radius, stroke, shadow, motion and sound.
CONVENTIONS.md §Theme and packs (Grounds and inks; Steps and roles) owns
the complete rules.

- Text sits on a pack slot or an opaque theme ground in its paired ink
  (`text-fg`, `text-<role>-fg`, HUD `-ink`). Small text straight over the
  scene needs `Plate.frame`; only a figure at least 24 px may rely on the
  text-stroke halo. OG9108 reports a ground/ink break at the class.
- No handwritten colour, length, shadow or opacity modifier. OG9109
  names the nearest theme step or role. Inline `style` carries only
  computed structure/data (bar width, position), not a private palette.
- Literal values belong in token sources, pack/theme data, art metadata,
  fixtures and structural geometry derived from props.

`packages/design/tokens/*.css` generates the default pack
`crates/og-ui-next/default/pack.json` (`scripts/gen-pack.sh`) and
`packages/design/tokens.json` (`scripts/gen-tokens.sh`). Both are
drift-checked, never hand-edited. Those CSS files are design-token source,
not game stylesheets. Add missing design roles there.

### 3.2 Packs, slots and schemes

A style is a `pack.json` or a `theme.json` recipe plus its assets, not a
directory of CSS. Layers are default pack, the look's style, then the
game's `assets/styles/game/pack.json`. Later named entries replace earlier
ones; fonts add faces. A theme recipe resolves `extends`, ordered `with`
mixins and its own values, then compiles dimensions and colours into a
pack and generated art. Its `pack` override can replace generated art.

A pack carries `scheme`, `seeds`, `theme`, `variants`, `images`, `icons`,
`iconSet`, `fonts`, `sounds` and optional feedback settings. A theme leaf
can be a `{ light, dark }` pair. Seed colours derive both palettes;
runtime `og.ui.configure` can change seeds, scheme or style. There is no
document reparsing when configuration changes: class records resolve
against the new pack/environment.

`packClass("Panel", "frame", variant)` returns a slot token that resolves
at runtime. A slot painting its ground owns the paired ink in every state.
Slots include `Button.plate`, `Panel.frame`, `Plate.frame`, `Slot.frame`,
`Chip.frame`, `Field.frame`, `Row.frame`, `Head.frame`. New slots are
declared in the default pack and `design::SLOTS` before use.

`Focus.ring` and `Press.feedback` provide shared focus/press treatment;
`Screen.frame`, `Modal.frame`, `Modal.scrim` presence; `List.cascade`,
`List.item`, `Reward.pop`, `Count.roll` list/reward/count motion;
`Tab.indicator`, `Bar.fill`, `Bar.trail`, `Attention.cue` selection,
gauges and enabled-state attention. `transition-layout` animates a box's
new layout; `glide-<name>` shares indicator placement. Motion is
interruptible and reduced-motion aware; timing belongs to the pack.

`scheme-dark` / `scheme-light` on a slot fixes its subtree's palette for
art that reads the same in both schemes; a nearer scheme wins.

### 3.3 Assets and the gallery

Art, icons, fonts, import settings and licences travel with their style
or shared resources. `nine-<name>` names pack artwork; tinted masks read
`bg-*`. Insets shrink proportionally for a box smaller than the image.
`nine-none` removes art for a state whose own fill replaces it. Nine-slice
frames own their edge: do not add a second border or ring over them.

Images in TSX use loaded texture handles; icons use names from the game's
atlas, the pack's icon set and the engine fallback. There is no CSS
`url()` / `integrity()` syntax on a TSX element. Font files list their own
families/weights; a pack's `theme.fontFamily` sets ordered family lists,
not `@font-face` rules. Resource paths and licences are cook dependencies.

The gallery (§6.5) draws component fixtures under packs/schemes. Frames are
judged for what they show, never pixel distance. Placeholders are not
finished art; core font coverage cannot depend on a commercial pack.

### 3.4 Font packs

Script packs under `templates/fonts/` carry `extension.json`, faces,
import settings, languages and licences. `og font list`, `og font add`
and `og create --lang` install them. The cook writes a FontPack index and
whole `<id>.face.ogb` faces; adding one must not change another font's
cooked output, charset or mode.

Language hyphenation and ICU segmentation dictionaries ship on demand,
not as a fixed engine cost. `hyphens-auto` in reachable TSX enables the
relevant hyphenation data; locale and installed packs determine which
dictionaries ship. Missing dictionaries warn. Font fallback chains include
the active table, style, locale and loaded packs; faces load on demand,
never blank while a usable fallback exists. Truly missing glyphs warn.

Glyphs raster from outlines into coverage, distance-field or colour atlas
pages as needed. Colour emoji (COLR v0/v1) is a font capability, not a UI
image substitute. Pack family lists and supported font classes are the
authoring surface; the old document's `@font-face`, `font-palette-values`
and arbitrary CSS font overrides are not supported TSX authoring forms.
Text shadows use named theme `textShadow` entries (`text-shadow-tag` for
the HUD halo), not arbitrary shadow lists in components. A layer using
`currentColor` or `contrast-color(currentColor)` follows the text ink.

## 4. Shipped catalogue

The library is filed by function: `base/`, `hud/`, `world/`, `panels/`,
`meta/`, plus genre-specific kits `rpg`, `shooter`, `strategy`, `racing`,
`card`, `puzzle`, `idle`, `rhythm`, `arena`. Screens and generic screen
pieces live in `screens/`. A piece any genre uses is not filed under a
genre merely because it first needed it. Arena owns match/team scores;
shooter does not duplicate them. Components never supply gameplay rules.

Each directory's README lists its exports; exported props and `fixtures`
in `.tsx` files are the source of truth. The `components` guide is
generated from the barrel and source, not handwritten. `og ui eject
--list` enumerates current screens, controls and components.

Styles may be game-authored packs or built-in theme recipes selected by
the look. Theme recipes replace the deleted fantasy/scifi/cartoon
handwritten packs; do not describe those old CSS packs as installed
runtime libraries. `templates/styles/shared/` is shared art/fonts/sound,
not a selectable pack. The materials extension is a resource pack, not a
component group or a UI style. Look-to-UI recommendations belong in the
look presets and generated catalogue; an explicit game choice wins.

## 5. Package lifecycle

- Import components from `@origingame/engine/ui/components` or individual
  component paths, screens from `@origingame/engine/ui/screens/<file>.tsx`.
  The bundler follows imports and companion string/asset dependencies.
  There is no implicit component-name registry or `og.load.document`.
- `mount` constructs the TSX tree on a layer and returns its disposer.
  Props are the exported interface, not `<meta>` declarations or stores
  implicitly resolved by name. Callback props report intents to the game.
- `og ui eject panels/backpack`, `og ui eject screens/pause`, or
  `og ui eject controls/button` copies the TSX into `assets/ui/<name>.tsx`,
  copies its locale tables with `game.`-prefixed keys and rewrites matching
  imports in game code. Relative component imports keep using engine
  pieces. It does not replace a component globally inside engine code.
  Existing files are refused without `--force`; new imports must point at
  the copy explicitly.
- `og.load.style(world, "styles/my-style")` loads a cooked style;
  `og.ui.configure(world, { style })` selects it. Theme recipes are cooked
  to packs/art; only referenced styles/resources enter the bundle. Files
  resolve relative to their pack; `@styles/shared/` addresses shared data.
- `og style add` installs a style; `og ui add materials` installs the
  resource extension without its review sheet. Installed resources take
  precedence over bundled ones of the same kind/name and are never
  silently overwritten. Font packs use `og font add`.

Public helpers, component props/events and creation schema are contracts:
a break needs a breaking changelog fragment with its fix, in the next
patch. Generated declarations and wiki are not edited by hand. Release
and platform pins follow [release rules](release.md).

## 6. Author and accept

### 6.1 State fixtures

Start a component group with inputs, outputs and pack-slot inventory,
normal/stress pages and two commercial information-structure references
(actual page, platform, screenshot source). Copy no commercial assets,
brands or layouts. Fixtures supply presentation states, never playable
economies, combat loops, matchmaking or rhythm clocks.

Each `.tsx` exports `fixtures`: named prop sets merged over `default`.
Provide applicable default/hover/pressed/focus, selected/disabled/mixed,
empty/single/normal/overflow, stable-ID/reorder/removal, input/cancel,
device-prompt, accessibility/localization and motion states. Explain
inapplicable cases. Focus remains visible independent of selection;
disabled emits no activation. Long text must wrap, scroll or intentionally
truncate without losing actions. Named stores and stable keys retain
identity through updates and checkpoints.

Core checks are self-contained; pack checks cover pack resources.
Removing a style cannot remove core coverage. Tests follow the repository
contract rules; quality is reviewed by eye, not turned into a test.

### 6.2 Frame matrix

Review default and stress fixtures under the default pack and each style,
phone and desktop, 100% and 150% text. Capture pseudo-locale (+40%), CJK,
RTL, landscape and further input/motion states according to risk. A
gallery is not proof of every combination; the game's combination needs
review. References name a regression, not a combinatorial checklist.

Capture effective style, scheme, viewport, DPR, text scale, locale, focus,
selection, tick and animation history. Equal ticks do not prove equal
state. Motion review includes start, midpoint, finish, interruption,
cancellation, reduced motion and a clip. Review HUD ink over bright and
dark scenes, not only a plain gallery ground. Empty texture handles mean
no art in a game even when review scaffolding supplies a sample.

GPU references and quality decisions come from Colab NVIDIA Vulkan
(L4 by default); other backends are compatibility checks. Missing GPU is
unchecked, never passed. A frame is judged for what it shows, not pixel
difference; never record black, blank, broken or clipped output.

### 6.3 Acceptance and art sign-off

Technical checks catch type/compile errors, missing resources, invalid
host/shader operations and contract failures. `og ui check` reports
measurements and findings; `og ui literals` reports suspect values.
Neither decides taste. Thresholds live in the screens and art-review
guides: contrast, readable text, touch reach, focus, localization and CPU
cost all require review. Overlap is a fact, not automatically a defect;
an intentional badge/ribbon overlap is explained, not redesigned merely
to silence a finding. Do not prescribe old document diagnostic attributes
as TSX layout controls.

Review hierarchy, numeric readability, selection, feedback, consistency
and reachability at comparable information density, not against a logo
or promotional image. New groups and styles need the designated art
reviewer's approval of actual frames/clips. A palette swap or desktop-only
success is not acceptance. Core-default captures prove behavior, not
final art. Quality failures delay release rather than reduce scope.

### 6.4 UI debugger

The development inspector (F1) includes a UI view for picking element
paths and inspecting their boxes and resolved look. Its panel is itself
TSX (`packages/engine/src/ui/inspector.tsx`), mounted on the `system`
layer while open; the host supplies its state and receives events.
Class/record inspection is not a document declaration/selector cascade.
The probe's `dump_ui` reads the retained tree; `dump_ui:motion[:path]`
and `dump_ui:layers` expose motion and layers.

`dump_ui.moving` reports finite transitions, layout moves, exits and
entrances still settling, not endless loops such as a spinner. Wait for
rest when capturing an entered screen, not an arbitrary entrance frame.

### 6.5 UI gallery

`bun packages/engine/src/node/gallery.ts [--out DIR] [--packs a,b]
[--schemes dark,light] [--only TEXT] [--size WxH] [--og <og>]` reads TSX
exports and fixtures under components, controls and screens. It writes one
project (`target/ui-gallery/project`), cooks once, then captures pages per
pack/scheme on a two-tone stage. It captures after 12 ticks and retries
moving entrances after a second. The default scheme is dark.

Output is `<out>/<pack>/` and `<out>/<pack>-light/` (default
`target/ui-gallery/out`), with `index.html` comparing fixtures across
packs/schemes. `--only` filters by directory/file and fixture text; a
component with no fixtures has no page. Findings are evidence, not an art
score. A run uses the GPU it finds; only the approved GPU host's captures
establish visual quality.

## 7. Maintenance boundaries

The integration owner owns shared schema, labels, looks, creation,
packaging, this contract and guides. Component authors own their source,
READMEs and fixtures; style authors own their recipe/pack and resources.
Shared API changes are coordinated before consumers adopt them.

`og-ui-next` is organized by mechanism: typed records/classes, retained
arena/tree, input, layout, motion, text, paint, accessibility and packs.
The TypeScript renderer writes node ops; Rust owns the host tree on both
platforms. There is no separate Svelte parse/build/writeback subsystem.

API declarations, component catalogue and wiki are generated from IDL,
TypeScript UI exports and guides; never hand-edit them. Token output and
look catalogues follow their generators. Missing capabilities belong in
the roadmap, not private component workarounds. Per-frame budgets require
measurement, never an assertion that unmeasured work passed. Author
knowledge belongs in guides, not shipped skills.

The [gate rules](ci.md) own checks, locks, toolchain and landing policy;
release and platform installation follow [release rules](release.md).
