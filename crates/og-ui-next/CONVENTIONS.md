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

One batch per tick, sent through `og_engine_ui_patch` (the same call and
wire codec as the store commands: an array of maps tagged by `op`),
applied in order before the UI's commit. NSH's `ops.rs` subset:

| `op` | Fields | Effect |
|---|---|---|
| `create` | `id`, `tag` (`view` `text` `image` `icon` `scroll` `input` `svg`…) | a node, detached |
| `createText` | `id`, `text` | a text run |
| `placeholder` | `id` | an empty anchor (`Show`/`For` insert before it) |
| `setText` | `id`, `text` | |
| `setClass` | `id`, `class` | the class string; resolved once per distinct string |
| `setStyle` | `id`, `style` | inline declarations as text, over the class record |
| `setProp` | `id`, `name`, `value` (JSON scalar) | `src`, `name` (icon), `disabled`, `tabIndex`, `value`, `role`, `label`, `data-*`, and the a11y props (§Accessibility) |
| `listen` | `id`, `event`, `enabled` | which events the node wants |
| `insert` | `parent`, `child`, `before` (id or null) | |
| `remove` | `parent`, `child` | detaches |
| `release` | `id` | frees it and its subtree |

Ids are u32 the renderer allocates; a mount's root is the id `mount`
answers. `data-*` props are the states a class variant reads
(`data-ready:` …). The reply to a batch carries the events since the last
one: `[{ id, event, payload }]`, which the renderer dispatches to handlers
by id.

## Accessibility

What a node is for a screen reader comes from its tag, these props and
its listeners; `og-ui-next/src/a11y.rs` builds the AccessKit tree from
them and the arena's boxes and texts. All are `setProp` names; the value is
a JSON scalar, and `null` unsets it.

| Prop | Value | Means |
|---|---|---|
| `role` | an ARIA role name (below) | what it is, over its tag's role |
| `label` | string | its name; without one a control is named by the text inside it |
| `description` | string | read after the name |
| `value` | string | an input's text |
| `valueText` | string | a range's value as words |
| `valueMin`, `valueMax`, `valueNow` | number | a range's bounds and value |
| `checked` | `true`, `false` or `"mixed"` | a checkbox's, radio's or switch's state |
| `pressed` | `true`, `false` or `"mixed"` | a toggle button's state |
| `selected`, `expanded` | bool | an option's or tab's, a disclosure's |
| `busy`, `current` | bool | loading; the current item of a set |
| `disabled` | bool | unavailable: no actions |
| `tabIndex` | number | `>= 0` the keyboard reaches it, `-1` it does not |
| `level` | 1–6 | a heading's level |
| `live` | `"polite"`, `"assertive"`, `"off"` | how its changes are announced (`status` and `log` are polite, `alert` assertive) |
| `modal` | bool | a dialog: while one is shown the tree holds only the topmost and what contains it |
| `hidden` | bool | left out of the tree with its subtree; it still draws |

Roles: `button` `link` `heading` `image` `text` `checkbox` `radio`
`switch` `slider` `spinbutton` `textbox` `combobox` `listbox` `option`
`tab` `tablist` `tabpanel` `radiogroup` `progressbar` `meter` `timer`
`status` `alert` `log` `dialog` `alertdialog` `tooltip` `separator` `list`
`listitem` `menu` `menubar` `menuitem` `toolbar` `grid` `row` `gridcell`
`region` `navigation` `main` `group`, and `none` (left out, its children
join its parent). Another name is a group.

Without `role`: a `text` node is static text, an `input` a text box, a
`scroll` a scroll view, an `image`/`icon`/`svg` an image when labelled. A
`view` listening to `press` or `tap` is a button, a modal one a dialog,
one the keyboard reaches or that is labelled a group; any other view is a
plain container, left out with its children joining its parent. Inside a
button, link, checkbox, radio, switch, tab, option, menu item, heading or
text, the texts are its name, not nodes. A node whose record is
`display: none` (`hidden`, the class) is absent, as is an empty text.
Visual hiding is a class; `hidden` the prop hides from the tree alone.

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

The theme is NSH's Tailwind theme (Tailwind's defaults, then
`Theme::from_json`'s keys: `colors`, `spacing`, `breakpoints`,
`fontFamily`, `fontSize`, `fontWeight`, `radius`, `shadow`, `tracking`,
`leading`, `blur`, `aspect`, `ease`, `container`), extended in this order:

1. the engine's design tokens (`packages/design/tokens/*.css`), built into
   theme JSON by the cook: semantic colours (`surface-1..3`, `edge`, `ink`,
   `ink-muted`, `accent`, `danger`, `warn`, `success`, `reward`, the tier
   colours `common`…`mythic`), radii, shadows and type;
2. the game's style pack;
3. the game's own `theme.json`.

A **style pack** is a directory with `theme.json` (the keys above; a
colour may be `{ "light": …, "dark": … }`) and an optional `variants`
object: named class strings a component appends after its own, keyed
`Component.slot` or `Component.slot.variant`:

```json
{
  "colors": { "accent": "#e0a33a", "surface-2": { "dark": "#1b1d24", "light": "#f3efe6" } },
  "radius": { "card": 10 },
  "variants": { "Panel.frame": "border-2 border-edge shadow-card", "Panel.frame.hud": "bg-black/40 backdrop-blur-md" }
}
```

A component asks for its slot's pack classes with `packClass("Panel",
"frame", variant)` and puts them last in its `class`. The cook compiles a
pack's `theme.json` into the theme the runtime loads, and its variant
strings into records like any literal class string.
