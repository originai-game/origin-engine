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
  `Image` (`src` a texture from `og.load.texture`, stretched over its box;
  a class's `nine-<name>` slices a pack's art instead), `Icon`, `Scroll`,
  `Input`, `Line` (a polyline: `points` x, y
  pairs in its box, `closed`, stroked by `stroke-<colour>` and
  `stroke-<width>`; no fill, curves or dashes), `Particles` (an engine
  particle emitter drawn in its box, §Particles), and the
  control flow `<Show when fallback keyed>` and `<For each key>{(item,
  index) => …}</For>` (`item` and `index` are accessors), and
  `<Switch fallback><Match when>…</Match></Switch>` (components). Every other tag
  is a component call. Every optional prop also takes `undefined`.
- **Props** keep camelCase names. Reading `props.x` inside JSX is
  reactive; a component body runs once (Solid's model), so derived values
  are functions or `createMemo`.
- **Radial wipe**: any element with a box image (`Image src` or a
  `nine-<name>` class) takes `wipe` (the share shown, 0 to 1, bound per
  tick: a cooldown or a ring), `wipeStart` (radians, 0 up) and `wipeCcw`.
  They are props, not classes, so a moving share never makes a class
  string.
- **State**: props are plain values or getters the renderer follows, never
  stores. A store (`og.ui.scope`) reaches a component as a signal through
  `useStore(store)`; a component never reads the simulation directly.
  `createSignal`, `createMemo`, `createEffect`, `onCleanup`, `batch`,
  `untrack` and context are exported beside the primitives.
- **Events** are `onPress`, `onTap`, `onPointerDown`/`Move`/`Up`, `onWheel`,
  `onFocus`, `onBlur`, `onKey`, `onChange`, `onLongPress` (a finger held
  still 0.5 s; event 11), `onHold` (with `hold={seconds}`: the held
  press's progress in `value` each frame, "0" when let go early; event
  12): JS functions, called by id at
  the tick boundary (the platform pump), batched and untracked, with
  `{ world, type, x, y, key, value }`. Input actions where focus is:
  `onAction={{ confirm, details }}` (wire: `setProp(id, "actions",
  "confirm details")` and event 10 `action` with the action in `key`). A
  `Text` with `glyph="use"` shows that action's binding glyph (input
  fills it): the pack's `icons` art for the binding on the last-used
  device (og-input's glyph names: `key.keye`, `xbox.a`, `ps.cross`,
  `mouse.left`, `key.command` for the Mac's ⌘), drawn one em square in
  the text colour, else the binding's name as text. It switches on the
  next input from another device. The default pack's art is
  `templates/styles/shared/glyphs/` (single-colour masks); a pack's own
  `icons` replace it by name. `autofocus` takes focus when the node appears.
- **Mounting**: `mount(world, Component, props, { layer })` from
  `@origingame/engine/ui` returns the unmount; layers are `hud`, `menu`,
  `modal`, `popover`, `toast`, `tooltip`, `system` (the mount's root
  carries `setProp("layer", …)`).
- **Strings** go through `t(key, params)` (and `formatNumber`,
  `formatCompact`, `formatDuration`, `formatCurrency`), which read the
  mount's world; text in JSX is literal or bound, never concatenated from
  translated pieces.

## JSX compile (head 2)

`og build` bundles game.js with Bun.build; its plugin
(`packages/engine/src/node/build.ts`) runs NSH's JSX transform, vendored as
`third_party/nsh-jsx` and reached through the addon's `og_node_jsx`, on
every `.tsx`/`.jsx` outside `node_modules`. A host element becomes one
function that makes it; a static value is set once, a changing one (an
expression that calls or reads a member) is followed. The compiled code
imports the renderer, `@origingame/engine/ui/rt` (`packages/engine/src/ui/
rt.ts`), which writes the §Ops batch and sends it when the mount is made
and at every tick boundary. `og check` runs NSH's class check against the
merged default and game packs: an unknown class is error OG9105 with its
line and column. Class strings resolve at runtime, once per distinct string (about 6 µs each
natively); nothing is cooked into records, so a pack or scheme switch
resolves them again.

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
| 6 | `setProp` | id, name, value (text) | `src`, `name` (icon), `disabled`, `tabIndex`, `value`, `role`, `label`, `data-*`, and the a11y props (§Accessibility) |
| 7 | `listen` | id, event u8, enabled u8 | which events the node wants |
| 8 | `insert` | parent, child, before (u32, `0xFFFFFFFF`: last) | puts or moves the child |
| 9 | `remove` | parent, child | detaches it |
| 10 | `release` | id | frees it and its subtree |
| 11 | `placeholder` | id | an empty anchor that takes no room (`Show`/`For` insert before it) |
| 12 | `strings` | table id | registers that bundle string table's base strings with the world, before the ops that follow read it |

Tags: 0 `View`, 1 `Text`, 2 `Image`, 3 `Scroll`, 4 `Input`, 5 `Icon`, 6
`Line` (its `points` and `closed` are setProps: numbers split by commas or
spaces; `viewBox`, `w h` or SVG's `x y w h`, puts the points in that space,
mapped onto the laid-out box as it paints, stretched or with `fit` `contain`
uniform and centred, the stroke staying in px), 7 `Particles` (its
`emitter`, `seed`, `delay` and `restart` are setProps, §Particles); there are no others. What can be an image is an `Image` (an
`.svg` file too: the cook makes it a texture). A line paints one
`PaintKind::Line` quad over its box (og-draw paint.rs): its points in
`Paint.points`, box-relative, width `p.x`, closed `p.y`. Events: 0 press, 1 tap, 2 pointerdown,
3 pointermove, 4 pointerup, 5 wheel, 6 focus, 7 blur, 8 key, 9 change,
10 action, 11 longpress, 12 hold, 13 dragstart, 14 dragmove, 15 dragend, 16 drop,
17 pinch, 18 rotate, 19 swipe.

The reply is `OGEV`, a u32 count and the events since the last batch, each
an id (u32), event (u8), x and y on the UI's surface (f32), then key and
value (strings; empty unless `key` or `change`), which the renderer
dispatches to handlers by id. A refused op stops the batch (the ops before
it stay applied) and the reply is `OGER` and the reason. `data-*` props are
the states a class variant reads: one is on unless its value is empty or
`false`; `data-ready:` reads the node's own, `group-data-ready/<name>:`
(`group-data-ready:` for an unnamed `group`) the nearest ancestor with
`group/<name>`. Only the boolean form: `data-[state=open]:` is not read.

A component's string table is a side-effect import beside it, `import
"./x.en.ogt"`: the build turns it into `strings("<id>")` (`@ui/<path under
src/ui>` for the engine's, the asset id for a game's), the renderer sends
each recorded table once as op 12 ahead of its first batch, and `mount()`
sends them before it builds, so a component's first `t()` finds its text.
The cook bundles the engine components' tables when code imports
`@origingame/engine/ui/components`.

## Input (head 4)

`og-ui-next/src/tree/input.rs` answers the pointer, the keys and the UI's
input actions on op nodes. It decides as NSH's `input.rs` does (vendored
verbatim as `third_party/nsh-input`) and as Rune's focus, modal and press
machines do (ported as logic).

- **Hit testing**: boxes in paint order, inside the clip their parent
  paints in; the last match wins. A node on its way out, or behind an open
  `modal`, is never hit.
- **States** that classes read: `hover:` (the node under the pointer and
  those around it), `active:` (held, while the pointer is on the node or
  within 8 px of it: NSH's press retention), `focus:`, `focus-visible:`
  (focus the keys moved: Tab, Enter or Space, an action or a screen
  reader; a press's focus shows no ring), `focus-within:` and `disabled:`.
  Records resolve once per class entry and the states it reads.
- **Events**: `press` and `tap` on a release within the retention (`value`
  is `"2"` on the second press of a double press: 0.5 s, 4 px);
  `pointerdown`, `pointermove`, `pointerup` and `wheel` (x and y carry the
  wheel's deltas) go to the innermost node under the pointer that listens,
  except that the node that heard `pointerdown` hears the moves and the
  `pointerup` until the release, wherever the pointer goes (a drag). A
  pointer event's `value` is the box of the node that hears it on the
  surface, `"x y width height"`, so a slider reads `(x - box.x) /
  box.width`; `longpress` (a finger held still 0.5 s; its release then
  presses nothing) carries it too;
  `focus` and `blur` as focus moves; `key` (`key` is the
  `KeyboardEvent.code`) to the focused node or the nearest around it that
  listens; `action` to the focused node or the nearest around it whose
  `actions` list the action, with its name in `key`.
- **Focus**: a press focuses the nearest focusable node around what it
  hits, and a press on anything else blurs. Focusable means a press or tap
  listener, an `Input`, or a `tabIndex`. Tab and Shift+Tab go to positive
  `tabIndex` first, then tree order, and wrap; `-1` is focused only by a
  press. The arrows, the d-pad and the left stick (once past half-way, then
  repeating after 400 ms every 180 ms) move the focus to the nearest stop
  that way, in its row or column first, wrapping to the far end of it
  (og-ui's FocusFinder rule, `crate::nav::neighbour`); with nothing focused
  they land on the first stop. A focused node that listens to `key` hears
  the arrows instead (a slider), and the d-pad still moves it. Enter or
  Space presses the focused control; so does `confirm`
  when no node around it lists `confirm`, but not twice in one frame.
- **Modals**: when a `modal` node shows, focus moves to its `autofocus`
  node, else its first Tab stop (else itself), and stays inside it. When it
  goes, focus returns to the node that had it before. An `autofocus` node
  outside one takes the focus once it shows. Focus a node takes by itself
  shows its ring when the player used a pad last.
- **Scroll**: a `Scroll`'s children move by its offset, clipped to its box,
  within what runs past its end (horizontal when only its width runs
  over). The wheel scrolls the innermost one under it along its axis that
  can still move, unless a nearer node listens to `wheel` (NSH's
  `wheel_listener`, `scroll_target`); notches that keep coming within
  200 ms stay on it. On a touch screen a finger past 8 px pans the
  scroller under where it went down, along the way it went, unless what
  heard its `pointerdown` drags; a pan lets the press go, and a pan let go
  while moving flings on, slowing by e each 0.5 s (Rune's touch
  scrolling). Focus the keys move scrolls into view.
- **Hold**: a node with `hold` presses when a press or `confirm` has been
  held on it that long, not on release; `hold` events carry the progress
  in `value` each frame (`"0.42"`, `"1"` as it completes, `"0"` when it
  lets go early: release, leaving it, or a pan).
- **Text fields**: a focused `Input` takes typed text (og-input's
  `og.TextInput` stream, or the printable keys) at its caret and hears
  `change` with its whole text in `value` each time. Backspace and Delete
  remove the grapheme cluster before and after the caret (UAX #29: a base
  and its marks go together), ←/→ step over one, Home and End go to the
  ends; keys with Control, Alt or Command pass on to the game. `value` set
  from outside replaces the text (caret at its end, no `change`);
  `placeholder` shows at half ink while it is empty. One line, no
  selection, undo or input-method composition yet.
- **Focus groups**: a node with `focusGroup` makes its focusable
  descendants one group for the arrows, the d-pad and the stick (og-ui's
  rule): a move picks among the group's members first; at its edge
  `"stop"` leaves it for the nearest control beyond (the group around it
  first) and stays put with none, `"wrap"` goes round inside it. With
  `focusMemory`, focus coming into the group from outside lands on the
  member it last had. Outside any group a move wraps to the far end of its
  row or column. Tab ignores groups.
- **Drag and drop**: a node with `draggable="<data>"` is picked up when a
  press on it moves past the slop (4 px for a mouse, a finger's 8 px pan
  slop) and then clicks nothing; a node with `dropTarget="<id>"` takes it.
  Events 13 `dragstart` (on the source: `key` its data, `value` its box),
  14 `dragmove` (source: `value` the target under it, "" none), 15
  `dragend` (source: `value` the target, "" cancelled) and 16 `drop` (on
  the target: `key` the data, `value` its box). The carried node keeps
  `active:`. Keys and pads: Enter, Space or `confirm` on a focused
  draggable picks it up; while carried, the targets are focus stops (Tab,
  the arrows, the d-pad); Enter or `confirm` puts it down on the focused
  target, Escape or `back` puts it back. An assistive technology's
  activate does the same. Sounds: the pack's `select` on pick-up,
  `confirm` on a drop, `cancel` when put back.
- **On-screen controls**: a node with `touchStick`, `touchPad` or
  `touchButton` is an input source of the og-input action it names, as a
  document's `<joystick action>`, `<dpad action>` and `<button
  data-og-touch>` are (docs/ui.md §3.3): after each layout its box, in
  surface px, goes to the action map as a control zone
  (`Tree::take_zones`), which samples the touches that started on it in
  the tick. No handler runs. A disabled or hidden one is no zone. What
  the map read comes back before the next layout (`Tree::zone_states`):
  the control is `active:` while a touch holds it, a `knob` node
  inside a stick moves by the stick's knob (a paint transform; layout
  stays put), and a held d-pad carries the way it reads as
  `data-dir-up|right|down|left` (two on a diagonal, none in its dead
  middle), which its classes read (`group-data-dir-up/pad:` lights an
  arm). A control that stops being a zone while held reads as let go.
- **Gestures** (NSH's recognisers, every pointer og-input tracks): each
  pointer keeps the node that hears `pinch`, `rotate` or `swipe` it went
  down on. A second finger down on the same node starts events 17 `pinch`
  and 18 `rotate` (`key` `start`, `move`, `end`; `value` the scale and the
  turn in radians, clockwise, since they began; x and y the fingers'
  midpoint), until either lifts. A pointer let go moving at 300 px/s or
  more, 10 px or more from where it went down, fires 19 `swipe` (`key`
  `left`, `right`, `up` or `down`; `value` `"vx vy"` in px/s); a pinch's
  fingers swipe nothing.

| Prop | Value | Means |
|---|---|---|
| `actions` | names, space separated | the input actions it answers (`onAction={{ confirm, details }}`) |
| `glyph` | an action's name | a `Text` node showing that action's binding on the current device (`<Glyph action>`) |
| `autofocus` | bool | it takes the focus once it shows |
| `hold` | seconds | a press or `confirm` must be held this long to press it |
| `value` | text | an `Input`'s text, set from outside |
| `placeholder` | text | what an empty `Input` shows |
| `focusGroup` | `stop` or `wrap` | its focusable descendants are one group for the arrows and pads |
| `focusMemory` | bool | with `focusGroup`, focus coming back lands on its last member |
| `draggable` | data | it can be picked up and dragged, carrying this data |
| `dropTarget` | an id | a carried node can be dropped on it |
| `touchStick` | an action's name | its box is a stick of that action (x, y from its centre) |
| `touchPad` | an action's name | its box is a d-pad of that action (eight directions); held, it carries `data-dir-<way>` |
| `touchButton` | an action's name | its box is a button of that action (1 while held) |
| `deadZone` | 0 to 0.95 | a stick's dead zone, of its radius (0: og-input's 0.15) |
| `knob` | bool | inside a `touchStick`, it moves with the stick's knob, held inside the stick's box |

## Accessibility

What a node is for a screen reader comes from its tag, these props and
its listeners; `og-ui-next/src/a11y.rs` builds the AccessKit tree from
them and the arena's boxes and texts. All are `setProp` names; the value is
its JSON text (unquoted text reads as a string), and `null` or empty
unsets it.

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
`scroll` a scroll view, an `image`/`icon`/`line`/`particles` an image when labelled. A
`view` listening to `press` or `tap` is a button, a modal one a dialog,
one the keyboard reaches or that is labelled a group; any other view is a
plain container, left out with its children joining its parent. Inside a
button, link, checkbox, radio, switch, tab, option, menu item, heading or
text, the texts are its name, not nodes. A node whose record is
`display: none` (`hidden`, the class) is absent, as is an empty text.
Visual hiding is a class; `hidden` the prop hides from the tree alone.

## World

A nameplate, a damage number, an interaction prompt, a waypoint or a
speech bubble is a node with `anchor`: an entity, or a world point
`[x, y, z]` (setProp text: the entity's number, or `x,y,z`; empty puts it
back in flow), and `lift`, a world offset added before projecting. Its
placement props (`occlusion`, `declutter`, `priority`, `clamp`,
`clampMargin`, `lift`) hold in any order with `anchor`.

```tsx
<View anchor={enemy} lift={[0, 2.1, 0]} declutter="stack" occlusion="fade"
      class="-translate-x-1/2 -translate-y-full">…</View>
```

- It is out of its parent's flow (absolute) and laid out at its own size;
  its box's top left sits at the point the tick's camera (og.Camera) sees,
  in the surface's CSS px. Offset and pivot are classes (`translate-*`).
  No scaling with distance.
- Its target off the surface, behind the camera, or a dead entity hides it
  (and a hidden parent): it paints nothing, takes no press, and leaves the
  accessibility tree and the probe's elements, so autoplay never presses
  it.
- `clamp="edge"` (a waypoint, a ping) keeps it instead: when its drawn box
  (after its transform) would leave the surface less `clampMargin` px (24),
  or its target is behind the camera, its box's centre pins on the line
  from the surface's centre toward the target where that line meets the
  margin, as Unreal's and Unity's off-screen indicators do. It carries
  `data-clamped` and `data-edge-top|right|bottom|left` (the side), which
  its classes read (`group-data-edge-left:` turns a chevron). A dead
  entity still hides it.
- `occlusion="fade"` or `"hide"`: at each tick's end the host asks whether
  a visible mesh drawn as the tick started stands between the eye and the
  point (og.render.raycast's record, every layer, the anchored entity's
  own meshes excepted): a line trace, as Unreal's widget components and
  Unity's world-space UI test it. `fade` draws it and its subtree at 0.35
  opacity, still pressable; `hide` hides it as off the screen does. It
  carries `data-occluded`. An edge-clamped node is never occluded, and a
  frame between ticks keeps the tick's answer.
- `declutter="stack"` or `"hide"`: after layout the decluttering nodes are
  taken by `priority` (higher first), then depth (nearer first), then
  order; each that comes within 2 px of one placed before it moves straight
  up clear of it (`stack`, nameplates over a crowd) or hides (`hide`), and
  one that would leave the top hides (MapLibre's collision pass, one
  direction). A node without `declutter` neither moves nor blocks; an
  edge-clamped one does not declutter.
- The host places it at each tick's end, after the game's systems, from the
  tick's camera and entities' world poses (`Tree::place_anchors`, which
  allocates nothing unless a `data-clamped`, `data-edge-*` or
  `data-occluded` changes); the simulation never reads it. On a frame
  drawn between ticks it moves with the drawn camera and its entity's
  drawn (blended) place (`Tree::follow_anchors`), so it stays on what it
  names; one the tick hid stays hidden. Clamping and decluttering settle
  after layout on every frame (`Tree::present`), from this frame's boxes.

## Particles

`<Particles emitter={e} seed={n} delay={s} restart={key} class="…" />` draws
an engine particle emitter in the UI: `emitter` is an `og.load.emitter`
handle (0 none), the same `.ogt` of `type particles` an `og.ParticleSystem`
runs in the world. One simulation, two places it draws: the tree steps
og-render-data's `CpuParticles` and paints each living particle as a quad of
its arena (tree/particles.rs).

- **Space.** The emitter sits at the centre of the node's laid-out box. One
  unit is one CSS px and +y is up, so `gravity 0 -600 0` falls down the
  screen. A `box` shape is the node's box (its x and y half extents are the
  box's, whatever the file writes); any other shape is born at the centre.
  `space local` particles move with the node, `space world` ones stay where
  they were born.
- **Time.** It steps on the UI tick's number, never a wall clock, and
  catches up after ticks it was not painted on. It starts when the node
  first paints, `delay` seconds later when given, and starts again from
  nothing when `restart` or `emitter` changes. Its randomness is `seed` and
  the tick it started: each play differs and a replay draws the same.
  A node with anything still to draw repaints every frame; a finished one
  costs nothing. Placing and stepping allocate nothing per frame
  (`examples/op_costs.rs`).
- **Paint.** Particles draw in painter's order where the node stands, under
  its clip and transform, each a box of its size (a disc with `round true`)
  or its `texture` (a `sheet`'s frame), turned by its rotation or along its
  velocity (`align velocity`). The colour is the node's ink (`text-<role>`,
  faded by its opacity) times the emitter's colour over life, so an emitter
  written white takes a pack's role; several inks are several nodes with
  their own `seed`.
- **What does not draw** is what needs the scene: wind and force fields,
  collision, lighting, distortion, trails, meshes, effect materials, texture
  arrays, glow and sub-emitters, and the arena blends alpha whatever `blend`
  says. The host warns once per node, naming the first such line.
- **Reduced motion** draws none, and a play that would have started then
  never plays late.
- **Built-in emitters.** An engine component keeps its emitters beside it
  (`components/hud/fx/*.ogt`) and imports each: the import's default is the
  asset's id (`@ui/components/hud/fx/confetti.ogt`), which the cook bundles
  for a game whose code reaches the component, and `og.load.emitter` loads.
  A game's own component loads an emitter from its assets the same way, by
  its path.

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
  case, `truncate`/`line-clamp`, `text_shadow`, `decoration`,
  `text_stroke` (`text-stroke-<number>` or `-[<length>]` its width,
  `text-stroke-<color>` its colour, the text's by default: an outline
  drawn under the fill);
- `image`, the box's nine-slice art, from the pack's `nine-<name>`
  classes (§Theme and packs).

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
  "images": { "plate": { "src": "art/plate.png", "slice": [12, 16, 12, 16], "width": 8 } },
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
  A `fontFamily` is a CSS family list. `rem` is the px to one rem in a
  bracketed class value and a style prop; the default pack's is the body
  type step, 14, so `w-[17.143rem]` is the 240 px `w-60` is.
- **`seeds`**: `primary`, `accent`, `neutral`, `danger`, `warn`, `success`,
  `info`, `team`, `hud` (seed.css), each a CSS colour or a scheme pair.
  When any layer sets one, the cook derives the design palette again from
  the seeds the way the default pack is generated, keeping the design's
  4.5:1 and 3:1 pairs, and the layers' written `colors` lie over it.
- **`variants`**: class strings a component appends after its own, keyed
  `Component.slot` or `Component.slot.variant` (PascalCase, then
  lower-case names). A component writes `packClass("Panel", "frame",
  variant)` last in its `class`: it answers the token
  `@Panel.frame.variant`, which the class resolver replaces with the active
  pack's slot string and then the variant's (an absent variant adds
  nothing; an absent slot stays an unknown class), so a pack switch
  restyles it. Nothing is folded at cook time.
  `og check` checks the strings against the merged theme. A variant that
  paints the ground under the text (a fill, or art with a centre drawn in
  its own colours, `fill` and no `tint`) sets the ink for that ground too,
  in its base state and through hover, press and disabled: the
  component's ink was chosen for the component's own ground. `og check`
  reports one that does not (OG9107, `pack::grounds_without_ink`). Art keeps its colours in both schemes, so its
  ink is a colour the pack writes the same in both (the built-in packs'
  `art-fg` and `art-primary-fg`), not a scheme role such as `fg`.
- **`images`**: nine-slice art by name (lower-case words joined by `-`):
  `{ src, slice, width?, fill?, repeat? }`. `slice` is the insets into the
  image in its pixels (one number, or `[top, right, bottom, left]`);
  `width` the insets as drawn, in points (the slice's when unset); `fill`
  whether the centre is drawn (default true); `repeat` `stretch` (default)
  or `tile` for the edges; `tint` true for a shape the element's fill
  colours (a chamfered plate that follows the theme), false (default) for
  a picture drawn as it is. An entry may be a scheme pair. The class
  `nine-<name>` gives an element that art, under any variant
  (`hover:nine-plate-lit`), in `variants` or in a component's own class:
  the last one that holds in the element's state and window wins, as for
  any class. `og check` reports a `nine-<name>` the merged pack lacks.
  A look's distinctive shapes (cut corners, ornate frames, slot rims) are
  this art, never paint features: if it can be an image, the engine does
  not draw it. The cook checks every `nine-<name>` names an image and
  every image is in the bundle.
- **`fonts`**: faces `{ family, src, weight?, style?, unicodeRange? }`, `src`
  relative to the pack or `@styles/shared/…` / `@styles/cjk/…`, with CSS
  `@font-face` meaning. `weight` is a number or a range `[min, max]`: on a
  variable face the part of its `wght` axis the entry serves, on a static
  face the weights it draws unsynthesised; omitted is 400 for a static
  face and the whole axis for a variable one. One file per entry; within a
  family entries are tried in declaration order, an entry skipped for a
  character outside its `unicodeRange`, then the locale chain. Each role
  has its own family, its CJK face last in it with a `unicodeRange`.
- **`icons`**: `name → art file`, over the default set, for `<Icon name>`.
- **`sounds`**: by event (`press`, `hover`, `select`, `confirm`, `cancel`,
  `back`, `open`, `close`, `error`, `reward`) for the whole pack, or under a
  `Component.slot[.variant]` key for one slot: `{ clips, volume?, pitch? }`,
  a number or a `[low, high]` range. A node's slot is the `@Component.slot[.variant]`
  token `packClass` puts in its class: a control plays `hover` as a mouse
  comes over it, `press` when clicked, `confirm` when that action presses
  it, and `confirm`/`back` when its `onAction` takes them. The sound is the
  slot's with its variant, then the bare slot's, then the pack-wide one;
  `confirm` and `back` fall back to `press`. One clip at random, at a
  volume and pitch from their ranges, from a seeded stream (a replay plays
  the same).

Motion is classes in `variants` until the motion thread's ease form lands.

### The default pack's names

The theme speaks the design language's words: a name is the token's
without its family prefix. Values are resolved per scheme (seeds and
relative colours evaluated) to `#rrggbb[aa]` and points.

| Section | Names | From |
|---|---|---|
| `colors` | every `--color-*`: `bg`, `bg-subtle`, `surface`, `surface-raised`, `surface-hover`, `card`, `fg`, `fg-muted`, `fg-subtle`, `fg-inverse`, `border`, `border-strong`, `well-rim`, `fill`, `fill-strong`, `accent`, `accent-hover`, `accent-active`, `accent-fg`, `accent-soft`, `accent-ink`, `focus`, `danger`, `warn`, `success`, `info`, `gold`, `health`, `mana`, `xp`, `hud`, `scrim`, `glass`, `glass-hi`, `glass-lo`, `plate-hi`, `plate-lo`, `rim`, `rim-soft`, `rim-strong`, `lit`, `lit-soft`, `shade`, `shade-deep`, `tile`, `tile-pressed`, `raised-*`, `glow`, `glow-soft`, `select-hi`, `select-lo`, `entry-*`, `stage*`, `material-*`, `team1`…`team8`, `common`…`mythic`, the ink on a fill of a status or rarity colour (`success-fg`, `danger-fg`, `warn-fg`, `info-fg`, `epic-fg`, `legendary-fg`), the HUD's `hud-accent`, `hud-accent-fg`, `hud-accent-hi`, `hud-accent-ink`, `foe`, `foe-fg`, `foe-ink`, `ally`, `ally-ink`, `boon`, `boon-fg`, `boon-ink`, `critical`, `critical-fg`, `fever`, `fever-fg`, `fever-ink`, `late`, `late-ink`, `well`, `key-hi`, `key-lo`, `key-fg`, `gold-fg`, `gold-hi`, `epic-hi` and `chart-easy`…`chart-master` with their `-fg` | `--color-<name>` (color.css) |
| `radius` | `control` `field` `block` `card` `panel` `sheet` `button` `capsule` | `--radius-<name>` (space.css) |
| `fontSize` | `xs` `sm` `base` `lg` `xl`…`8xl` (over Tailwind's), `2xs` `meta` `caption` `body` `reading` `title` `display-sm`…`display-2xl`; `[size, line height ÷ size]` | `--font-scale-<name>-size` and `-line-height` |
| `tracking` | `tight` `xs`…`xl` `wide` `label` `kicker` (`--font-tracking-*`), and the letter spacing of the scale steps `2xl`…`8xl` and `display-*` (`xl` is `--font-tracking-xl`) | type.css, in em |
| `leading` | `flat` `tight` `snug` `normal` `loose` | `--font-leading-<name>` |
| `fontWeight` | `regular` `medium` `semibold` `bold` `body` | `--font-weight-<name>` |
| `fontFamily` | `sans` `display` `mono` `rounded` | `--font-<name>` |
| `shadow` | every `--shadow-*`: `edge` `raised` `float` `overlay` `lift` `rim` `panel` `plate` `raise` `raise-hover` `sunk` `held` `focus-tile` `minimap` `slot-selected` `tab-selected` `lens` `glow-sm` `glow` `glow-lg` `key` | `--shadow-<name>` (effect.css), per scheme, each layer's colour evaluated |
| `insetShadow` | `lit` `lip` `sunk` `glow` `underline` `overline` `bar` | `--inset-shadow-<name>` (effect.css), per scheme |
| `ease` | `out` `exit` `in-out` (the `linear()` springs stay with motion) | `--motion-easing-<name>` |
| `container` | `xs` `sm` `md` `lg` `xl` `xxl` `3xl` | `--measure-<name>` |
| `blur` | `thin` `regular` `thick` `lens` | `--vars-material-blur[-<name>]` |

Every look has these images, white masks coloured by the element's fill
(`tint`), so `bg-*` colours them (`templates/styles/shared/shapes/`):

| Image | Shape | For |
|---|---|---|
| `disc` | a filled circle, stretched whole | a round slot's cooldown sweep (`wipe`) |
| `ring` | a thin circle outline, stretched whole | progress, ultimate and timer rings (`wipe`) |
| `cross` | a plus with arms 40% wide, stretched whole | a d-pad's body |
| `cross-rim` | the cross's outline, stretched whole | a d-pad's rim, over `cross` |
| `cross-arm` | the cross's top arm in the cross's box, stretched whole | a d-pad's pressed way, over `cross`; `rotate-90`, `rotate-180` and `-rotate-90` turn it to the other arms |
| `tile` | a square at `--radius-field` | a square slot's sweep |
| `plate-cut` | leading top and trailing bottom corners cut 6 | a HUD plate |
| `panel-cut` | the same corners cut 12 | a HUD panel |
| `chip-cut` | the trailing top corner cut 6 | a tag, badge or chip |

The slots a pack dresses, each declared in the default pack so a
component's `packClass` on it resolves under every pack:
`Button.plate` (a button's body; `primary` its main action), `Panel.frame`
(a window or panel), `Plate.frame` (a HUD plate over the scene: `bg-glass
text-fg`), `Slot.frame` (an ability or item slot), `Chip.frame` (a tag or
badge), `Field.frame` (a text field's box), `Row.frame` (a row of a list,
table or feed: `bg-raised-lo text-fg`; `selected` the chosen or local
player's row, `bg-surface-hover` with a `hud-accent` leading edge; `muted`
a row out of play, in `text-fg-muted`) and `Head.frame` (a table or list's
head row: `bg-surface text-fg-muted`). A new slot is added here and to
`design::SLOTS` before a component writes it.

### Grounds and inks

Text reads over any scene and in every pack because its ground and ink
are a pair the theme chose, never a component's tuning:

- Text sits on a pack slot (`packClass`) or on an opaque theme ground role
  (`surface`, `surface-raised`, `surface-hover`, `raised-hi`, `raised-lo`,
  `raised-pressed`, `tile`, `glass`, `glass-hi`, `plate-hi`, a state colour
  such as `hud-accent` or `gold`), and its ink is that ground's pair
  (`text-fg` on the neutral grounds, `text-<role>-fg` on a state colour).
  A HUD colour as an ink is its `-ink` role (`text-hud-accent-ink`,
  `text-foe-ink`, `text-ally-ink`, `text-boon-ink`, `text-fever-ink`,
  `text-late-ink`), which holds 4.5:1 on the neutral grounds and the
  `well` in both schemes; `text-hud-accent` itself is a fill's colour
  and does not read on a pale ground.
  A ground under text is at least 85% opaque in both schemes, with no
  opacity modifier taking it below that.
- Text straight over the scene sits on `Plate.frame`, or carries the
  text-stroke halo (paint draws it).
- Muted text is `text-fg-muted` (4.5:1 on the neutral grounds), never
  `text-fg/<n>`; `text-fg-subtle` holds only 3:1 and is for a disabled
  line. An ink is never thinned.
- No `white` or `black` and no hand-written colour (`bg-[#…]`) in a
  ground or an ink: a highlight or shadow away from text is `lit`,
  `lit-soft`, `sheen`, `shade`, `shade-deep` or `scrim`; a rim is `rim`,
  `rim-soft` or `rim-strong`; a team's edge is `border-team<n>`.

`og check` reports each break as warning OG9108 at the class, naming the
slot or role that fixes it (`crates/og-cli/src/ui_grounds.rs`). An
element's ground counts as under text when the element is a `<Text>` or
`<Input>`, or holds one (or literal text) that no nearer element with its
own ground covers; classes reach an element through its `class` literals,
templates, conditionals and the constants and arrow functions they name.

**Steps and roles.** A component writes no value of its own: a size, a
colour, a shadow or a strength is a step or a role the theme names, so a
pack moves it and two components agree.

- A length is a spacing step (`w-60`, `gap-y-0.5`, `h-0.75`; quarter steps,
  `px` for one pixel), a share a fraction (`w-2/5`, `right-3/10`, `full`),
  never a bracket (`w-[17.143rem]`, `size-[40%]`). A grid of tiles names
  its tile's least width in steps: `grid-cols-fill-33`, `grid-cols-fit-48`
  (`repeat(auto-fill | auto-fit, minmax(…, 1fr))`).
- Type is a `fontSize` step, a `tracking` and a `leading` name, a corner a
  `radius` name; a border, ring or stroke is a whole number of px.
- A shadow is the named set: one `shadow-*` (an elevation, `shadow-key`,
  or a glow `shadow-glow-sm | glow | glow-lg`), one `inset-shadow-*`
  (`lit`, `lip`, `sunk`, `glow`, or the mark of state `underline`,
  `overline`, `bar`), a `ring` and an `inset-ring`, each coloured by a role
  (`shadow-glow shadow-gold`, `inset-ring-1 inset-ring-rim`). A face whose
  light fits none of these is the slot's pack art.
- A colour is a role everywhere, not only under text: no `white` or
  `black`, no `[#…]`, `rgb()` or `var(--…)` in a class, and no opacity
  modifier on a role (`bg-well/79`, `border-rim/50`, `text-shadow-bg/55`):
  the whole role, or the role that names the strength (`fg-muted`,
  `rim-soft`, `lit-soft`, `shade`).
- Structure is not a value and stays written: a grid template of `auto`,
  `fr` and `minmax(0,1fr)` tracks, `flex`, `grow`, `row` and `col`
  numbers, an `env()` inset, and an inline `style` that carries data (a
  bar's `width: 40%`).

`og check` reports each break as warning OG9109 at the class, naming the
class that replaces it (`crates/og-cli/src/ui_values.rs`); `--json`
carries it as `fix`, with `exact` false where it is the nearest step or
role and moves the value. A class that also breaks the grounds rule is
reported once, as OG9108.

`spacing` stays Tailwind's 4 pt step. The cook compiles the merged theme
into the theme the runtime loads and every variant string into records,
like any literal class string.
