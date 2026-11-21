# og-ui contracts (TSX)

Game UI is TSX components styled with Tailwind classes, the way NSH
(longbridge/nsh) does it: JSX compiled to Solid-style renderer calls, class
strings resolved to typed records in Rust. There are no `.svelte`
documents, no style sheets, no cascade and no compatibility with the old
forms. This file is the contract between the threads that build it; change
it only together with the owners named below.

| Contract | Owner | Readers |
|---|---|---|
| Component shape, primitives | og-ui coordinator (this thread) | the port of the engine's components, every game |
| JSX compile | coordinator, head 2 | `og build`, `og check` |
| Ops and events | coordinator and the UI thread | the TS renderer (head 2), the arena applier (head 3), input (head 4) |
| Class vocabulary, Record | coordinator, head 1 | paint (head 5), motion, a11y |
| Accessibility props and roles | a11y (head 7) | the TS renderer, the arena applier, the web mirror |
| Theme and packs | coordinator and the looks thread | the cook, the platform's looks |

## Components

A component is a function from props to an element, in a `.tsx` file of
the game or of the engine (`packages/engine/src/ui/`):

```tsx
import { View, Text, Icon } from "@origingame/engine/ui";

export function AbilitySlot(props: { icon: string; cooldown: number; ready: boolean }) {
  return (
    <View class="relative size-14 rounded-lg bg-surface-2 ring-1 ring-edge" data-ready={props.ready}>
      <Icon name={props.icon} class="size-8 text-ink" />
      <Text class="absolute bottom-1 right-1 text-xs tabular-nums">{props.cooldown}</Text>
    </View>
  );
}
```

- **Primitives**, imported from `@origingame/engine/ui` (NSH's rule: tags
  are PascalCase imports; a lowercase tag is an error): `View`, `Text`,
  `Image`, `Icon`, `Scroll`, `Input`, `Svg` with its shapes, and the
  control flow `Show`, `For`, `Switch`/`Match`. Every other tag is a
  component call.
- **Props** keep camelCase names. Reading `props.x` inside JSX is
  reactive; a component body runs once (Solid's model), so derived values
  are functions or `createMemo`.
- **State** comes from the engine's stores (`og.ui.scope`, `UiReactor`) as
  signals; a component never reads the simulation directly.
- **Events** are `onPress`, `onTap`, `onPointerDown`/`Move`/`Up`, `onWheel`,
  `onFocus`, `onBlur`, `onKey`, `onChange`: JS functions, called by id.
- **Mounting**: `og.ui.mount(Component, props, { layer })` on a scope;
  layers are `hud`, `menu`, `modal`, `popover`, `toast`, `tooltip`,
  `system`, as before.
- **Strings** go through `t(key, params)`; text in JSX is literal or bound,
  never concatenated from translated pieces.

## JSX compile (head 2)

`og build` bundles game.js with Bun.build; a plugin runs NSH's JSX
transform (`nsh-bundler` `jsx.rs`/`compile.rs`, oxc) on `.tsx`. A host
element becomes one function that makes it; a static value is set once,
a changing one (an expression that calls or reads a member) is followed.
`og check` runs NSH's class check (`classes.rs`): an unknown class is an
error with its line and column. Class strings the compile sees as
literals are listed for the cook, which resolves them into records ahead
of time; only bound class strings go through the runtime cache.

## Ops (TS renderer → Rust)

One binary batch per tick, which the renderer writes into one reused
`ArrayBuffer` and sends through `og_engine_ui_patch` (the call the store
commands use; a batch starts with the four bytes `OGOP`, which no store
command batch does). The runtime applies it to the tree at once, in order:
the first batch makes the tree, so a mount's build happens while the game
loads and the first frame after Play only commits and paints. Nothing in it
is copied: strings are read in place, and a node's class and style resolve
to a record once per distinct pair (`Tree::set_resolver`).

After `OGOP`: a u32 op count, then each op as a u8 code and its fields. Ids
are u32 the renderer allocates (0 is the root: inserting into 0 mounts a
node); strings are a u32 byte length and UTF-8; numbers are little-endian.

| code | op | fields | effect |
|---|---|---|---|
| 1 | `create` | id, tag u8 | a node, detached |
| 2 | `createText` | id, text | a text run |
| 3 | `setText` | id, text | |
| 4 | `setClass` | id, class | the class string |
| 5 | `setStyle` | id, style | inline declarations as text, over the class |
| 6 | `setProp` | id, name, value (text) | `src`, `name` (icon), `disabled`, `tabIndex`, `value`, `role`, `label`, `data-*` |
| 7 | `listen` | id, event u8, enabled u8 | which events the node wants |
| 8 | `insert` | parent, child, before (u32, `0xFFFFFFFF`: last) | puts or moves the child |
| 9 | `remove` | parent, child | detaches it |
| 10 | `release` | id | frees it and its subtree |
| 11 | `placeholder` | id | an empty anchor that takes no room (`Show`/`For` insert before it) |

Tags: 0 `View`, 1 `Text`, 2 `Image`, 3 `Scroll`, 4 `Input`, 5 `Icon`, 6
`Svg` (its shapes 7 and up). Events: 0 press, 1 tap, 2 pointerdown,
3 pointermove, 4 pointerup, 5 wheel, 6 focus, 7 blur, 8 key, 9 change.

The reply is `OGEV`, a u32 count and the events since the last batch, each
an id (u32), event (u8), x and y on the UI's surface (f32), then key and
value (strings; empty unless `key` or `change`), which the renderer
dispatches to handlers by id. A refused op stops the batch (the ops before
it stay applied) and the reply is `OGER` and the reason. `data-*` props are
the states a class variant reads (`data-ready:` …).

## Classes and the Record (head 1)

Classes are Tailwind v4.1's vocabulary as NSH resolves it
(`third_party/nsh-tailwind`, vendored): later utilities win, variants
(`hover:`, `active:`, `focus-visible:`, `disabled:`, `data-*:`, `group-*:`,
`dark:`, breakpoints) apply after plain utilities, inline style last.
There is no cascade: only `group` states and `focus-within` cross nodes.
Breakpoints read the UI's own box; `touch:` reads the input.

A class list writes one `style::Record`: layout into the node's
rune-taffy style, and these paint fields (fixed size, no allocation):

- `fill`; `gradient` (linear, `corner` aim, up to 8 stops); `radius`
  (tl tr br bl); `border` (t r b l, also layout) and `border_color`;
  `shadows` (4 layers, `ring-*` a spread layer, `inset-shadow-*` inset);
  `outline` (width, offset, colour); `opacity`; `transform` (translate,
  scale, rotate, origin; paint only); `backdrop_blur`;
- text: `ink`, font size, weight, family, line height, tracking, align,
  case, `truncate`/`line-clamp`, `text_shadow`, `decoration`.

A class the record has no field for is reported by `og check` as unknown,
not dropped silently.

## Theme and packs

Owner: the looks thread. The theme is NSH's Tailwind theme: Tailwind
v4.1's defaults extended by `Theme::from_json` once per scheme. What
extends it is a stack of **packs**, each one `pack.json`, laid in order:

1. the engine's default pack, `crates/og-ui-next/default/pack.json`,
   generated from `packages/design/tokens/*.css` by
   `crates/og-ui-next/scripts/gen-pack.sh` (never edited by hand; a test
   holds it equal to the generator);
2. the pack the game's look names (`ui "<name>"`: `assets/styles/<name>/`
   over `@styles/<name>/`);
3. the game's own pack, `assets/styles/game/pack.json`, when it has one.

A later layer replaces an earlier one's entry of the same name: a theme
entry, a variant, an icon, an event's sound. Fonts add faces.

```json
{
  "scheme": "dark",
  "seeds": { "primary": "#ffc71f", "neutral": "#232a73" },
  "theme": {
    "colors": { "gold": "#ffd23f", "surface": { "light": "#f3efe6", "dark": "#1b1d24" } },
    "radius": { "card": 14 },
    "fontFamily": { "display": "Cartoon Display, Cartoon Fallback" },
    "shadow": { "plate": { "light": "0 4px 0 #2a1a10", "dark": "0 4px 0 #000" } }
  },
  "variants": {
    "Panel.frame": "border-2 border-border-strong shadow-panel",
    "Panel.frame.hud": "bg-plate-hi/80 backdrop-blur-md"
  },
  "fonts": [{ "family": "Cartoon Display", "src": "LilitaOne-Regular.ttf", "weight": 400 }],
  "icons": { "star": "art/star.svg" },
  "sounds": {
    "press": { "clips": ["@styles/shared/sfx-tap.ogt"], "volume": [0.24, 0.3], "pitch": [1.04, 1.16] },
    "Button.base": { "hover": { "clips": ["@styles/shared/sfx-tap2.ogt"], "volume": 0.08 } }
  }
}
```

Every key is optional and every key and form is checked (an unknown key
is an error naming where it is, never ignored):

- **`scheme`**: `light` or `dark`, the scheme the pack is drawn in unless
  the game asks for the other.
- **`theme`**: `Theme::from_json`'s keys (`colors`, `spacing`, `rem`,
  `breakpoints`, `container`, `fontSize`, `fontWeight`, `fontFamily`,
  `tracking`, `leading`, `radius`, `shadow`, `insetShadow`, `dropShadow`,
  `blur`, `perspective`, `aspect`, `ease`). **Any leaf may be
  `{ "light": …, "dark": … }`**; the pack layer splits it before NSH reads
  the theme, so each scheme's theme is plain theme JSON and switching
  scheme switches record tables. `light` and `dark` are never shade names.
  A `fontFamily` is a CSS family list.
- **`seeds`**: `primary`, `accent`, `neutral`, `danger`, `warn`, `success`,
  `info`, `team`, `hud` (seed.css), each a CSS colour or a scheme pair.
  When any layer sets one, the cook derives the design palette again from
  the seeds the way the default pack is generated, keeping the design's
  4.5:1 and 3:1 pairs, and the layers' written `colors` lie over it.
- **`variants`**: class strings a component appends after its own, keyed
  `Component.slot` or `Component.slot.variant` (PascalCase, then
  lower-case names). A component writes `packClass("Panel", "frame",
  variant)` last in its `class`; with literal arguments the cook folds it.
  `og check` checks the strings against the merged theme.
- **`fonts`**: faces `{ family, src, weight?, style?, unicodeRange? }`, `src`
  relative to the pack or `@styles/shared/…` / `@styles/cjk/…`.
- **`icons`**: `name → art file`, over the default set, for `<Icon name>`.
- **`sounds`**: by event (`press`, `hover`, `select`, `confirm`, `cancel`,
  `back`, `open`, `close`, `error`, `reward`) for the whole pack, or under a
  `Component.slot[.variant]` key for one slot: `{ clips, volume?, pitch? }`,
  a number or a `[low, high]` range. Input (head 4) plays them.

Motion is classes in `variants` until the motion thread's ease form lands.

### The default pack's names

The theme speaks the design language's words: a name is the token's
without its family prefix. Values are resolved per scheme (seeds and
relative colours evaluated) to `#rrggbb[aa]` and points.

| Section | Names | From |
|---|---|---|
| `colors` | every `--color-*`: `bg`, `bg-subtle`, `surface`, `surface-raised`, `surface-hover`, `card`, `fg`, `fg-muted`, `fg-subtle`, `fg-inverse`, `border`, `border-strong`, `well-rim`, `fill`, `fill-strong`, `accent`, `accent-hover`, `accent-active`, `accent-fg`, `accent-soft`, `accent-ink`, `focus`, `danger`, `warn`, `success`, `info`, `gold`, `health`, `mana`, `xp`, `hud`, `scrim`, `glass`, `glass-hi`, `glass-lo`, `plate-hi`, `plate-lo`, `rim`, `rim-soft`, `rim-strong`, `lit`, `lit-soft`, `shade`, `shade-deep`, `tile`, `tile-pressed`, `raised-*`, `glow`, `glow-soft`, `select-hi`, `select-lo`, `entry-*`, `stage*`, `material-*`, `team1`…`team8`, `common`…`mythic` | `--color-<name>` (color.css) |
| `radius` | `control` `field` `block` `card` `panel` `sheet` `button` `capsule` | `--radius-<name>` (space.css) |
| `fontSize` | `xs` `sm` `base` `lg` `xl`…`8xl` (over Tailwind's), `meta` `caption` `body` `reading` `display-sm`…`display-2xl`; `[size, line height ÷ size]` | `--font-scale-<name>-size` and `-line-height` |
| `tracking` | `xs`…`xl` `wide` (`--font-tracking-*`), and the letter spacing of the scale steps `2xl`…`8xl` and `display-*` (`xl` is `--font-tracking-xl`) | type.css, in em |
| `leading` | `flat` `tight` `snug` `normal` | `--font-leading-<name>` |
| `fontWeight` | `regular` `medium` `semibold` `bold` `body` | `--font-weight-<name>` |
| `fontFamily` | `sans` `display` `mono` `rounded` | `--font-<name>` |
| `shadow` | every `--shadow-*`: `edge` `raised` `float` `overlay` `lift` `rim` `panel` `plate` `raise` `raise-hover` `sunk` `held` `focus-tile` `minimap` `slot-selected` `tab-selected` `lens` | `--shadow-<name>` (effect.css), per scheme |
| `ease` | `out` `exit` `in-out` (the `linear()` springs stay with motion) | `--motion-easing-<name>` |
| `container` | `xs` `sm` `md` `lg` `xl` `xxl` `3xl` | `--measure-<name>` |
| `blur` | `thin` `regular` `thick` `lens` | `--vars-material-blur[-<name>]` |

`spacing` stays Tailwind's 4 pt step. The cook compiles the merged theme
into the theme the runtime loads and every variant string into records,
like any literal class string.
