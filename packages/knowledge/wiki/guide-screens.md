<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `screens` guide: UI and the UI packs

Build screens and HUDs from TypeScript TSX components, dress them with
typed utility classes and style-pack slots, and review them at the sizes,
languages and input modes the game supports. The named stores follow the
`svelte/store` data model; the UI does not use Svelte markup or CSS sheets.

The API: `og api show ui` (configuration, stores and readback),
`og api show components --guide` (the current exported library) and
`og api show i18n` (translations, fonts, numbers and plurals).
`og ui eject --list` lists components, controls and screens available to
copy; `og ui list` lists resource packs and `og style list` lists styles.

<a id="guide-screens-1"></a>
### `screens §1` Order of work

1. List the screens the game needs: title, HUD, pause, settings, results,
   and domain pages such as inventory, shop or deck.
2. Import a built-in screen/component (§2), or compose core controls in a
   `.tsx` screen. No installation or document loading is required.
3. Pass typed props and handlers from the game's presentation stores;
   keep domain decisions in gameplay systems.
4. Dress the shared slots through the look's style (§3), then preview and
   check (§5) at every supported size, text scale, locale and input mode.

<a id="guide-screens-2"></a>
### `screens §2` Built-in screens and components

Import components from `@origingame/engine/ui/components` or their
individual paths, and screens from `@origingame/engine/ui/screens/…`.
Their exported `<Name>Props` interfaces describe inputs and callbacks;
`fixtures` in the same `.tsx` file supply named presentation examples.
The bundler follows imports and their string tables/assets transitively.
There is no `<ui:name>` syntax, implicit component registry,
separate document loader or screen-name push operation.

```tsx
import { mount } from "@origingame/engine/ui";
import { PauseScreen } from "@origingame/engine/ui/screens/pause.tsx";

// In the game's UI registration, with world supplied by the engine:
const unmount = mount(world, PauseScreen, {
  objective: "",
  played: 0,
  onResume: () => { /* Resume through the game's own state/intent. */ },
}, { layer: "menu" });
// Call unmount() when the game closes this screen.
```

The `menu` layer orders presentation; the game owns simulation pause and
which screen is open. Mounting a screen does not infer pause behavior from
its file name or its exported preview metadata.

Built-in action rows are generally conditional on callback props:
`PauseScreen` with `onResume`, `onSettings`, `onQuitToMenu` has those
three built-in rows. Omit a handler (or pass `undefined`) to omit its row;
an action unavailable only now stays drawn but disabled by its state prop
(Save uses `canSave` with `onSave`). Pause also accepts game rows:
`rows={[{ key: "restart", label: "Restart stage", after: "resume" }]}`
and `onRow={(event, key) => …}`. Data lists are not hidden just because a
handler is absent; inspect each component's props for its exact contract.

To customize, `og ui eject screens/pause`, `og ui eject panels/backpack`
or `og ui eject controls/button` copies TSX to `assets/ui/<name>.tsx`,
copies imported locale tables with keys prefixed by `game.`, and rewrites
matching game imports to the copy. Engine components that import the
original keep doing so: this is import rewriting, not a global name
override. New uses import the copy explicitly. Existing files are refused
unless `--force` is given. Preserve pack slots and accessibility semantics
when changing layout.

Components draw data and report callbacks with stable IDs; they do not
decide purchases, rewards, legality, damage or timing rules. A game's own
component is an ordinary TSX function imported the same way.

### A weapon level from Pips, Counter and Bar

The engine has no weapon-level rule. Compose the discrete level, number
and remaining power-up time from reusable themeable pieces:

```ts recipe ticks=10
import { og } from "@origingame/engine";
import type { World } from "@origingame/engine";
import { Text, View, mount } from "@origingame/engine/ui";
import { Counter, HudBar, Pips } from "@origingame/engine/ui/components";

const MAX = 5;

function WeaponLevel(p: { level: number; timer: number }) {
  const items = Array.from({ length: MAX }, (_, i) => ({ id: `lv${i}`, value: i < p.level ? 1 : 0 }));
  og.assert.always(items.filter((u) => u.value === 1).length === p.level, "one lit bolt a level", { level: p.level });
  return View({
    class: "flex-col gap-1 w-40",
    children: [
      View({
        class: "flex-row items-center gap-2",
        children: [
          Pips({ label: "Power", icon: "bolt", tone: "warn", items }),
          Text({ class: "text-title", children: ["P", Counter({ value: p.level })] }),
        ],
      }),
      HudBar({ value: p.timer, kind: "stamina", name: "Power-up time" }),
    ],
  });
}

export default og.game({
  start(world: World) {
    mount(world, WeaponLevel, { level: 3, timer: 0.6 }, { layer: "hud" });
  },
});
```

This recipe mounts fixed props. For changing data use reactive JSX/getters
as described in §4; a component body runs once.

<a id="guide-screens-3"></a>
### `screens §3` Style packs

A style is a directory with a `pack.json` or `theme.json` and its assets,
not CSS files. A theme recipe compiles its dimensions, colour choices,
mixins and overrides into a pack and generated art (the themes guide).
A handwritten pack sets only what it changes over the default pack.

The layers are default pack, the look's chosen style, then
`assets/styles/game/pack.json` if present. Named theme entries, variants,
images, icons and sounds in a later layer replace earlier entries; fonts
add faces. Palette seeds derive the grounds, inks and state colours in
both schemes. A pack can set `"seeds": { "primary": "#c8102e" }` without
rewriting each role. Theme leaves can be `{ "light": …, "dark": … }`.

For example, the following `pack.json` dresses a shared slot using theme
steps and roles; the same slot is used by every imported Button:

```json
{
  "seeds": { "primary": "#14675f" },
  "variants": {
    "Button.plate.primary": "bg-accent text-accent-fg hover:bg-accent-hover active:bg-accent-active rounded-button"
  }
}
```

Components append `packClass("Button", "plate", variant)` to their class
list. The token resolves against the active pack at runtime. A slot
painting its ground also supplies the paired ink in every state. Common
slots include `Panel.frame`, `Plate.frame`, `Slot.frame`, `Chip.frame`,
`Field.frame`, `Row.frame`, `Head.frame`. There are no part selectors.

1. Create `assets/styles/my-style/pack.json` (or derive a theme recipe).
2. Set seeds, theme roles/type/radii, variant classes, fonts, nine-slice
   `images`, icons and sounds. Asset paths are relative to the pack;
   shared engine resources can use `@styles/shared/…`. Fonts travel with
   licences and name their own families; `theme.fontFamily` lists them.
3. Select the style in the look and capture the game's screens (§5).

To select a loaded style explicitly:

```ts
const style = og.load.style(world, "styles/my-style");
if (!style.ok) throw new Error(style.reason);
og.ui.configure(world, { style: style.value });
```

Style, seed and scheme changes resolve class records again; there is no
document reload or CSS parse-order requirement. `scheme-dark` or
`scheme-light` fixes a subtree's palette for artwork that stays the same
in both schemes. `nine-<name>` selects pack images, optionally tinted by
`bg-*`; `nine-none` removes art for a state with its own fill. A frame
owns its edge; do not draw a second border over it.

Use theme utilities for type, spacing, radii, gradients, shadows and
motion. Inline `style` is for computed data/structure, not visual tokens.
Images in TSX take loaded texture handles; icons take names. CSS
`url()`, `@font-face`, `@keyframes`, `::part`, `-og-material` and
`-og-sound` declarations are not supported TSX authoring forms. Shader
resource presets and the current attachment boundary are in ui-materials.

<a id="guide-screens-4"></a>
### `screens §4` Structure and data

- Import PascalCase primitives: `View`, `Text`, `Image`, `Icon`, `Scroll`,
  `Input`, `Line`, `Particles`. Reusable controls (`Button`, `Checkbox`,
  `Slider`, `Select`, `Switch`, `Tabs`, `Dialog`, `Sheet`, `Popover`,
  `DropdownMenu`, `ContextMenu`, `NumberInput`, `TextField` and others) come from
  `@origingame/engine/ui/components`. Lowercase HTML/SVG tags are errors.
- Create checkpointed presentation values with `og.ui.scope(world, id)`
  in `og.game`'s `ui(world)` callback and bridge stores with `useStore`.
  Call the returned accessor in JSX (`{gold()}`), not `{$gold}`. Local
  `createSignal` values are reactive but not named checkpointed sources.
- Read props inside JSX or an accessor/`createMemo`: a plain
  `const ready = props.value > 0` in the component body is a snapshot.
  Controlled fields use `<Input value={name()} onChange={e =>
  nameStore.set(e.value)} />`, not two-way directives. Event handlers
  receive `UiEvent`, not browser DOM events.
- `Show` conditionally renders; `For` retains keyed rows and gives its
  children `(item, index)` accessors. Use `item().id`, not `item.id`.
  Objects default to their `id`; `key={item => item.slot}` chooses another
  stable identity. Never key reorderable data by its index. `Switch` /
  `Match` chooses the first matching case. Optional content is a prop
  returning JSX, invoked where it belongs, not a Svelte snippet.
- Layout uses block/flex/grid semantics through classes. On a shared row
  use `flex-1 min-w-0` to take remaining width and allow truncation; in a
  column use `flex-1 min-h-0`. `w-full` is a full parent width, not the
  remainder after siblings. `Scroll` supplies scrolling and a draggable
  pack scrollbar (`scrollbar={false}` hides it). Safe alignment classes
  (`justify-center-safe`, `items-center-safe`) keep oversized content's
  start reachable.
- Screen roots fill their layer with `absolute inset-0`. Apply safe-area
  padding once to the content with `env(hud-inset-top|right|bottom|left)`
  (for example `pt-[env(hud-inset-top)]`); the mount itself is the surface,
  not an automatically inset content document. Backgrounds can fill the
  surface while content stays safe. Built-in screens supply their own
  insets; do not add the same inset around them twice.
- Responsive variants read the UI surface: `compact:` is under 600 px
  wide or 500 px tall, `narrow:` and `short:` cover individual axes.
  `pointer-coarse:` / `touch:` follow touch input; `motion-reduce:` handles
  reduced motion. There are no document media/container query sheets.
- States use boolean `data-*` flags and variants, including named groups:
  `data-ready={ready()}`, `data-ready:bg-success`,
  `group/slot` and `group-data-ready/slot:…`. Use one flag per enum state
  (`data-rarity-rare`), not a CSS attribute-value selector.
- `role`, `label`, `description`, checked/pressed/selected/expanded,
  ranges, `live`, `modal`, `tabIndex`, focus groups and focus memory supply
  accessibility semantics. `hidden` the prop affects accessibility;
  `hidden` the class removes display. Name icon-only controls.
- Motion uses `transition-*`, `duration-*`, `ease-*`, `animate-*`,
  `starting:` and `exit:` classes, plus pack slots. `transition-layout`
  and `glide-*` move retained elements (§8). Reduced motion suppresses or
  shortens movement; do not pace gameplay from an animation finishing.
- Paragraphs use nested `Text`, inline `Icon`/`Image`, `ruby`, selectable
  text and supported font classes. `RichText` accepts explicit rich runs;
  it does not parse dialogue attribute spans or arbitrary HTML.
- Geometry is typed: `Line` takes numeric x/y `points`, optional `closed`,
  `viewBox` and `fit`; `Image` takes a texture. `wipe`, `wipeStart`,
  `wipeCcw` reveal a box image radially. Shapes that can be images are
  pack artwork, not inline SVG hosts or SVG path strings.
- Drag/drop uses `draggable={id}`, `dropTarget={id}` and
  `onDragStart/Move/End`, `onDrop`. Keyboard and gamepad can pick up and
  drop too. `onContextMenu` handles a right click or keyboard request;
  `onPinch`, `onRotate`, `onSwipe` expose gesture phases/data. Read
  `HostProps` for each event's string payload, not DOM event fields.
- Touch controls feed actions through `touchStick`, `touchPad`,
  `touchButton`, `touchLook`, `deadZone`, `knob`; `floatingStick` centres
  a stick at touch-down without moving its activation box. `touchLook`
  sends drag deltas in CSS px per fixed tick (x right, y up), zero while
  stationary/released, with input sensitivity/inversion applied.
  `TouchControls` composes these (`floatingStick` for move, `lookPad` for
  drag look). The game reads the same input actions as keys/pads.
  World labels use `anchor`
  and `lift`, occlusion, declutter and edge clamping. `scaleAt` and bounds
  set distance scale; `maxDistance` and `lod` control distance presentation.
  Ground marks use decals/lines, not UI hosts.
- `Particles` takes a loaded emitter, seed, delay and restart key, in CSS
  px (+y up). `Celebration` and `Flourish` compose emitters and pack cues.
  Camera views are loaded render targets in `Image`; orbit/picking is
  game code, not `<viewport>` attributes. UI world panels and custom
  shader attachment are not exposed by the TSX host props.
- Visible words use `t` with imported `.ogt` string tables, and locale
  formatters for numbers/durations/currency. Use named actions through
  `onAction` and `glyph` for device-adaptive input hints, not baked key
  letters. `sound`, `exitSound` and `haptic` request pack feedback.

<a id="guide-screens-5"></a>
### `screens §5` Thresholds, preview and check

`og ui check` runs game screens through styles, breakpoints, locales and
text scales, reports findings and measurements, and does not score taste.
Use game command scripts to show each screen (`--screen name=commands`,
or project screen declarations); exporting `fixtures` alone is not how
a game's screen is selected. Check command help for the matrix options.
The bars for author review are:

- **Text:** at least 18 px at 1080p for menus/body/controls. Dense HUD
  readouts may reach 12 px if still readable and text-scalable. Do not use
  an old document `dense` prop as an exemption: it is not a TSX host prop.
- **Scale:** usable at 150% text, with no lost actions or cut content.
- **Contrast:** 4.5:1 for live text; 3:1 for text over 24 px, icons and
  outlines; 7:1 in high contrast. APCA additionally asks |Lc| 75 for body,
  60 for text over 24 px or bold over 18 px, 45 for large headings, 30
  for disabled text/placeholders. Review actual ink on actual art.
- **HUD grounds:** inspect translucent/scene-backed UI over black and
  white and keep the worse contrast. Small text needs a readable plate;
  a faint offset shadow is no halo. The theme's `text-shadow-tag` is the
  explicit multi-layer halo, not `text-shadow-2xs`. A shadow inherits in
  each text's own ink unless that text supplies another shadow.
- **Colour vision:** semantic colours should stay ΔEOK 0.1 apart under
  protanopia, deuteranopia and tritanopia; below 0.05 they read as one.
  Where they cannot, add a shape, icon or label. More than six teams need
  a second cue, not colour alone.
- **Steps and roles:** OG9108 flags invalid grounds/inks; OG9109 flags
  handwritten values and names a replacement. Use `text-title`, `w-60`,
  `border-rim-soft`, named shadows and whole colour roles, not bracket
  colours/lengths or arbitrary opacity modifiers. A rem is the theme's
  body step (14 px in the default pack); spacing steps are 4 px. Grid
  tracks, flex factors, safe insets and computed inline geometry remain
  structural, not visual literals.
- **Touch:** shorter side at least 44 CSS px, 8 px between targets;
  primary phone actions in the lower half in portrait.
- **Navigation:** every control reachable; Back on each layer; no
  unintended focus trap. Focus is distinct from selection.
- **Actions:** one filled primary action per layer.
- **Cost:** aim for a 300-element menu with 40 animating elements within
  0.5 ms CPU/tick and a HUD within 1 ms; measure, do not infer from tests.

Fix cut text, safe areas, missing names/feedback/art and accidental
overlap. Keep intentional overlaps (badge on icon, corner ribbon) and
explain them; do not redesign merely to silence a finding.

`og ui preview` writes a contact sheet; `--film` captures motion at chosen
times. Read the frames before calling the screen finished: 390×844 phone,
1920×1080 desktop, bright/dark HUD scenes, empty/overflow/disabled,
keyboard/gamepad/touch, large text, reduced motion, high contrast and RTL.
GPU quality review requires a real supported GPU; missing GPU is unchecked.

<a id="guide-screens-6"></a>
### `screens §6` Languages and fonts

- Each create-block language needs a string table and script coverage.
  `og font list`, `og font add <pack>`, `og create --lang` manage packs;
  locale and fallback chaining select faces without handwritten font CSS.
- Faces and script dictionaries ship/load on demand, not as a fixed boot
  cost. A font addition must not change other fonts' cooked output.
- Run `og i18n extract --check` before builds; use locale formatters, never
  concatenate translated pieces. Import companion base tables beside TSX.
- Check pseudo-locale (+40%), CJK and RTL with the preview/check locale
  matrix. English-sized boxes and fixed physical-side layout are not
  proof of localized layout.

<a id="guide-screens-7"></a>
### `screens §7` Component catalogue

Use the generated components guide for the complete exports and props.
The source directories are organized by function, not markup tag names:

| Need | Library area |
| --- | --- |
| Controls, fields, dialogs, sheets, menus, rich text | `controls/` (also re-exported by `components`) |
| Slots, avatars, currencies, pips, spinners, empty/error states | `components/base/` |
| Bars, meters, cooldowns, abilities, timers, feedback, touch controls | `components/hud/` |
| Nameplates, waypoints, prompts, overhead bars, maps/compass | `components/world/` |
| Inventory, quests, conversation, tables, graphs, layout patterns | `components/panels/` |
| Shop, achievements, rewards, social | `components/meta/` |
| Genre-specific pieces | `rpg/`, `shooter/`, `strategy/`, `racing/`, `card/`, `puzzle/`, `idle/`, `rhythm/`, `arena/` |
| Generic game screens and data/feedback pieces | `screens/` |

These are TSX components, not `og.ui` tree builders or component documents.
Geometry is passed as numbers/points, stable IDs accompany data, and the
game retains every domain rule. A theme dresses their shared slots without
copying their structure.

<a id="guide-screens-8"></a>
### `screens §8` Moving elements between layouts

Give an element `transition-layout` (with `duration-*` and `ease-*`) and
change the state that places it. A moved/resized box slides from its old
layout rather than jumping. Use keyed `For` rows so reordered items retain
their element identity. `glide-<name>` lets a new indicator start from a
nearby indicator of the same name, as a shared layout ID would.

A change mid-move retargets from the drawn box. Reduced motion jumps to
the new layout; animation never determines whether that layout is valid.
