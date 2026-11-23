# Game UI contract

This document defines the boundaries and acceptance rules for core UI,
the built-in component and screen documents, and style packs. API
signatures belong in the generated API; author workflows belong in the
guides. Where a feature follows a web standard, this document names the
standard and states only the engine's deviations, extensions (`-og-*`)
and limits; what the parser does not support it refuses by name (§7).

## 1. Scope and ownership

| Owner | Responsibility | Boundary |
| --- | --- | --- |
| Core (og-ui) | Capabilities: elements, layout, drawing, documents, layers, focus and navigation, gestures, scrolling, text editing, motion, text, accessibility roles, semantic tokens, and each control element's value, binding and behaviour | Draws no control's look and composes no control; one API on native and web |
| Built-in library | Svelte documents built into the engine under `@ui/`: the control documents (`controls/`, each HTML control's inside, §2.4), the component documents by group (`rpg/`, `shooter/`…) and the screen documents: presentation composed from the controls, named parts, typed inputs, and returned intents | No gameplay rules, literal visual values or private engine imports; one document per control, and no second control for a role HTML names |
| Style pack | Palette, typography, spacing, shape, dressings, art, icons, motion, sound, and haptic definitions, as `:root` tokens and `var()` part rules. Chosen when the game is created, or written for that game | No gameplay code or domain state; not switched while a game runs |
| Game | Content, rules, world state, domain validation, and applying UI intents | Owns its screens and chosen style assets |
| Platform | Accounts, money, publishing, threads, agent orchestration, and engine version selection | Consumes released archives and the public host ABI, never engine internals |

The renderer owns the UI pass. `og-ui` owns layout, documents, interaction,
and style interpretation. `packages/engine/src/ui/` holds the library:
`controls/`, `components/` and `screens/`. The public TypeScript API comes
from `crates/og-api/api/` for both hosts. Game authors never need Rust.

The library exists so that a game's author takes a finished piece instead
of drawing one. Its bar is a polished look in the default style; how a
piece looks in one game is that game's art, decided when the game is
created, through its style pack's tokens and part rules (§3). A common
game UI need with no built-in is a gap in the library, closed by adding
the document, never by a hand-rolled copy inside another built-in.

Screen UI, world anchors, world panels, and embedded camera views use the
same element model. Backend limits are capability queries, not separate
implementations. Web rendering stays off the main thread.

UI presentation state is tick-driven and checkpointed beside the world,
outside `world_hash`. Focus, selection, scroll, caret, and animation state
keep their meaning after snapshot and restore. Rendering cannot change
simulation results. See [engine design](design.md) for determinism and the
host boundary.

## 2. Composition contract

Each layer has one authoring form:

- Look is a style pack's pack.json theme and the class strings the TSX
  core reads (§3).
- Structure is TSX components (`.tsx`): a function returns its tree, with
  its props an exported interface.
- Data uses the `svelte/store` model (§2.1). Games prepare named stores and
  the components read them.

The normative wire, primitive, class-vocabulary, input and accessibility
contracts live in [`crates/og-ui-next/CONVENTIONS.md`](../crates/og-ui-next/CONVENTIONS.md);
the author guide is [`crates/og-api/guides/screens.md`](../crates/og-api/guides/screens.md).
There are no `.svelte` documents, no `<style>` sheets and no CSS cascade.

- **Primitives** are PascalCase imports from `@origingame/engine/ui`
  (lowercase HTML tags are a compile error): `View`, `Text`, `Image` (with
  `src` or a pack's `nine-*` class and radial `wipe`), `Icon`, `Scroll`,
  `Input`, `Line` (polylines with `points`, `viewBox`, `fit`), `Particles`
  (CPU-stepped engine particle emitter), and control flow `<Show>`, `<For>`,
  `<Switch>` / `<Match>`.
- **Components** are `.tsx` functions returning JSX, compiled by `nsh-jsx`
  into Solid-style reactive renderer calls (`@origingame/engine/ui/rt`).
  A component body runs once; reactive props are read inside JSX expressions
  or `createMemo` / `createEffect`, and `useStore(store)` bridges named
  `og.ui.scope` stores into signals.
- **Styling** uses Tailwind v4 utility classes resolved in Rust against the
  active style pack's `pack.json` tokens (checked by `og check` rule
  `OG9105`) plus `packClass("Component", "slot", variant?)` for pack-dressed
  artwork and sound scopes. Interaction states use variants (`hover:`,
  `active:`, `focus:`, `focus-visible:`, `focus-within:`, `disabled:`,
  `data-*:`, `group-*:`), presence uses `starting:` and `exit:`, and layout
  motion uses `glide-*`.
- **Events & actions**: `onPress`, `onTap`, `onPointerDown/Move/Up`,
  `onWheel`, `onFocus`, `onBlur`, `onKey`, `onChange`, `onLongPress`,
  `onHold`, drag-and-drop (`onDragStart/Move/End`, `onDrop`), gestures
  (`onPinch`, `onRotate`, `onSwipe`), `onContextMenu` (a right click,
  Shift+F10 or the Menu key), and `onAction={{ confirm, back, ... }}`
  run at the tick boundary with `{ world, type, x, y, key, value }`.
- **Mounting & layers**: `mount(world, Component, props, { layer })` mounts
  a component on `hud`, `menu`, `modal`, `popover`, `toast`, `tooltip`, or
  `system` and returns its unmount callback.
- **Anchors, touch & accessibility**: `anchor` / `lift` / `occlusion` /
  `declutter` / `clamp` attach nodes to world entities or named UI nodes;
  `touchStick`, `touchPad` and `touchButton` wire on-screen controls into
  `og-input`; `role`, `label`, `description`, `checked`, `pressed`,
  `selected`, `expanded`, `modal`, `live`, and range props populate the
  AccessKit tree (and the web shell's `A11yMirror`).
- **Localization**: visible text uses `t(key, params)` and the formatters
  from `@origingame/engine/ui` with companion `.ogt` string tables imported
  beside the component (`import "./screen.en.ogt"`). Pseudo-locale
  verification runs through `og ui check`.

#### Composition discipline

One control kind has one visual grammar. Reusable interactive controls live in
`packages/engine/src/ui/controls/` (§2.4) and are imported from
`@origingame/engine/ui/components` or ejected with `og ui eject`. Generic
meters, counters, timers, slots, lists, grids, tables, charts, tooltips,
cards, rewards, confirmations and dialogs are shared library components;
domain names do not justify duplicate controls.

### 2.1 Retained authoring model

The engine implements the `svelte/store` model (`writable`, `readable`,
`derived`, `get`) plus a keyed collection store, named within a scope so
they can be checkpointed. Rust owns the graph on both hosts; no Svelte
runtime or compiler runs in the game. Data authoring is function calls;
structure stays in templates. There are no async primitives, Promises,
resources, Suspense, or async transitions.

Flat, single-batch, normal-blended shape paints retain local geometry and
submission order while only positive axis-aligned transforms or opacity
change. These ticks update the retained items' properties without rebuilding
commands or painter levels. Paint, topology, clip, viewport and unsupported
pipeline changes return to ordinary painting; they do not reuse the old
placed command list. Text, images, offscreen layers, backdrops and world
placement are outside this property-only path.

Warmed class changes that only change a built-in button's scalar scale and
opacity targets can keep its control and bindings. The retained builder
validates the cached cascade context, unchanged non-target values, and the
ordered rules matching every shadow part before updating motion targets.
Sibling controls with identical host and part inputs share that proof
within a presentation tick.
Real style or part changes, custom control documents, computed-value
animations and positioned descendants use the ordinary path. Selector and
inherited-value invalidation still runs; declaration ownership changes
revisit only their own names, not every name descendants read.

The shared runtime on both hosts keeps a caller-owned `RetainedEmission`
for an otherwise empty, unshifted UI overlay without strict-palette
processing. It shares emitted shapes and patches properties only against
the exact preceding generation; missed generations rebuild. Ordinary
emission remains the fallback for other content and mixed writers. Renderer
streams compare whole instances and upload adjacent changed ranges, fully
initializing replacement buffers and regrown tails. Rendering, live clocks,
resource validation and renderer metadata construction still run. This is
not a general compositing layer or a promise that all CSS transform and
opacity animations skip painting.

Gameplay initialization belongs in `og.game({ start(world) { … } })`; UI
registration in its `ui(world)` callback, which runs once after `start`
and again after a hot-reload restore, never per tick. Registration creates
named scopes and stores through `og.ui.scope`. Stable IDs keep values
across hot reload; re-registered callbacks replace their bodies, and
missing IDs are disposed. `ui` may read the world; a gameplay write fails
registration. A fresh-process restore runs `start`, then `ui`, then
restores named presentation values; hot reload does not rerun `start`.

`og.ui.scope(world, id): UiScope` takes an explicit, stable, nonempty ID:

```ts
writable<T>(id: string, initial: T): UiWritable<T>;
readable<T>(id: string, initial: T, start?: UiStart<T>): UiReadable<T>;
derived<S extends UiStores, T>(id: string, stores: S, compute: UiDerive<S, T>, initial?: T): UiReadable<T>;
get<T>(store: UiReadable<T>): T;
collection<T>(id: string, items: readonly T[], desc?: UiCollectionDesc): UiCollection<T>;
takeErrors(): readonly string[];
dispose(): Result<void>;
```

Values are serializable. Subscription, first and last subscriber
(`start` and its cleanup), `get` and synchronous pure derivation keep
Svelte's semantics. A failed derivation keeps the last good value or the
initial one; with neither it reports an error and yields `undefined`.
`set` and `update` return `Result<void>`. Collections key by `desc.key`
(default `id`) and expose `keys()`, `row(key)`, `insert(row, before?)`,
`remove(key)`, `move(key, before?)` and `patch(key, fields)`, each a
`Result`; duplicate keys and key-changing patches are refused. Scope errors
stay readable through `takeErrors`, in publish builds too.

```ts
export default og.game({
	start(world) {
		// Initialize gameplay here.
	},
	ui(world) {
		const inventory = og.ui.scope(world, "inventory");
		const gold = inventory.writable("gold", 0);
		inventory.derived("doubled", gold, value => value * 2);
	},
});
```

Presentation checkpoints live outside `world_hash`. Saves require the
`og-save:3` envelope; other envelopes are refused with both format
versions. UI feedback sound (`og.audio.playFeedback`) is presentation
only: no gameplay entity or event, not replayed on restore. Gameplay
effects cross `og.input.intent(world, event, payload)`, which records the
reflected event name, layout hash and payload for the next tick;
`takeIntents` records accepted intents and `replayIntent` validates their
schema and tick. `og.PauseIntent` applies time scale the same way; a UI
control never writes gameplay state.

Retained mounting:

- A binding tracks only the values it read and replaces its dependencies
  on every evaluation, including after a branch changes; unchanged
  dependencies skip it. Bindings are pure. There is no implicit per-tick
  callback; ambient values (time, viewport) are explicit sources.
- Identity is the mount, document node and collection key. Focus and
  selection belong to keys, across insert, remove and reorder. Virtualized
  rows outside the materialized range style when they enter it.
- Replacing a mounted document rebuilds its nodes without resetting
  stores; unmounting removes its nodes and subscriptions. Source values,
  collection order, mounts and UI state are checkpointed outside
  `world_hash`; restore rebuilds derived caches from saved registrations
  without replaying mount events or advancing time. Hot reload
  re-registers callbacks by ID while sources and mounts survive.
- An action inside keyed rows carries the innermost collection's raw key
  (not an escaped path), after restore too; outside rows it carries none.
- Errors are values naming scope and ID: duplicate keys, cycles,
  undeclared reads and reentrant writes.
- Named TypeScript callbacks exist only for custom drawing and the few
  components a document cannot express, with explicit dependency handles;
  they are not a tree-building API. Mount content is serializable document
  and snippet references.

A `type screen` document mounted with `scope.mount(id, document, inputs)`
is a layer of the stack while mounted. Its `<meta name="screen"
data-layer … data-pause … data-back … data-focus …>` decides input
routing, inert layers under a menu, pause, focus memory and the
`activate`/`deactivate` events, as for a pushed screen; its bindings read
the mount's stores. Back (`data-back="pop"`) or a `pop` trigger unmounts
it. Its layer id is `scope:id` and its root's path `scope:id/<root id>`
(a pushed screen's is `<screen>/<root id>`); `og.ui.layers` names the
mount (`UiLayer.mount`). Each layer adds one wrapper node per frame; the
tree under it is written only where it changed.

Components and snippets follow Svelte 5. A capitalised tag instantiates
that component. The explicit form `<ui:name>` instantiates the exact,
case-sensitive name declared by `<meta name="component" content="name">`,
including lowercase and hyphenated names: `<ui:star-rating stars={$stars}
label={$label}/>`, `<ui:key-values rows={$rows}/>` and `<ui:banner
title={$title} subtitle={$subtitle}/>`. The prefix is syntax, not part of
the document name or manifest key; it works for game documents too.
Native tags are unchanged: `<table>` is HTML, `<ui:table>` is the component
named `table`. `<Card>` and `<ui:Card>` name the same component; `StarRating`
is not an alias for `star-rating`. An instance's attributes are props evaluated in the user's context
(`<Card title={item.name}>` follows that row only), `bind:` writes back
through the prop, `{#snippet}` children are passed content and the other
children are `children`. `{@render body?.()}` renders passed content with
the user's rows and stores, else the document's own snippet of that name,
else nothing. A component reads the mount's stores and iterates
collections passed as props. Mount content passes by reference:
`scope.mount("pause", "pause", inputs, { side: "game#pauseSide" })`
(a bare document name renders its root). Rendered content's paths sit
under the snippet name (`…/Card#a/body/Desc`); an instance's root takes
the instance's id.

A component is styled through what it declares, never from outside by
selector. Its user has three handles: a prop; a style prop
(`<Badge --width="66px"/>`, `--width={$w}`), a custom property the
instance's root inherits as if a box around it set it, so the component's
own rules win and `null` sets nothing; and, from a style pack, the parts
it names (`[part="Badge.frame"]`). A document's rule that names an
instance (`#Mine`, `.loud`, `Badge`, `[data-tone]`) is refused at parse
with those three as the fix; a rule that only sweeps over one (`*`,
`#Root > *`) passes it by. `style:` is an element's directive and refused
on an instance, as a bare `--name` is on an element.

Inside the component, `:host` is the instance. An instance and its root
are one element, so `:host { … }` styles the root's box as an ordinary
rule of the component's sheet, and `:host(<compound>)` asks the instance:
an attribute is the prop of that name as the instance passes it
(`:host([tone=warn])`; absent while the prop is null or false), a class a
word of its `class` (`<Card class="wide"/>` passes the component's `class`
prop, which the component declares to read it), a state the root's own
(`:host(:hover)`, `:host(:disabled)`), and a structural pseudo-class the
instance's place among its siblings (`:host(:nth-child(2))`).
`:host(…) .inner` and `:host > .child` follow from it. Two limits: host
and root being one box, a property that `:host` and a root rule both set
cascades to one value where a browser would apply each to its own box;
and a host has no features inside its component, so `:host.wide` and
`:host:hover` are refused for `:host(.wide)` and `:host(:hover)`, as are
an attribute with no prop, an id, and `:host` in a component whose root
is itself an instance.

`<iframe src="news.svelte" items={$news}>` embeds another document of the
game. `src` is a
literal path relative to the embedder, which the cook bundles as it does a
component and refuses when missing; a bound `src` is refused. The other attributes are the embedded document's
props, read in the embedder's context; it reads no store of the mount, and
its events reach the game as a component's do. The frame is a separate
cascade: the embedder's rules never match inside, the topology's parent
stops at the frame (so no selector inside, the style pack's included, sees
the page around it), and inheritance stops at the frame's root. The style
pack and colour scheme still apply. The `<iframe>` is a replaced box,
300 × 150 px on an axis CSS leaves auto, `overflow: clip`; `frame-sizing:
content-…` (CSS Sizing 4) makes that axis the document's laid-out size
when the document declares `<meta name="responsive-embedded-sizing">`. A
document never embeds one it is inside.

A screen opts into CSS View Transitions 1 and 2 with `@view-transition {
navigation: auto; }`. The engine's mapping:

- Its layer joining or leaving the stack is a navigation, and the layer is
  the `root` snapshot (the leaving layer stays drawn and inert until its
  animations end). The default is the spec's UA cross-fade (0.25s).
- `-og-iris`, `-og-dissolve`, `-og-wipe` and `-og-slide` (the last two
  with `-from-right`, `-from-top`, `-from-bottom`) are full-frame covers
  CSS keyframes cannot express: on `::view-transition-old(root)` a cover
  closes over the outgoing view, on `-new(root)` it opens on the incoming.
  `::view-transition { background-color; background-image: url() }` is
  the covers' colour or tinted texture, loaded with the document.
- Shared elements are captured as last frame's frozen draw commands; the
  new image is this frame's element outside a leaving layer. The new
  screen's rules style the groups (the leaving screen's when nothing
  joins). A duplicate name skips the transition with a warning. When old
  and new nesting differ, the new view's parent wins. `overflow` on
  `::view-transition-group()` and `-image-pair()` clips to that box (the
  pair's is its own; `-old` and `-new` refuse it); a nested image pair is
  its own layer.
- Reduced motion ends finite animations at once, makes covers fade and
  skips shared elements unless the style keeps motion. Only a screen opts
  its layer in. Capturing costs nothing until a transition starts; a
  running one is saved with the UI state.
- Scoped view transitions (Level 2 §4): `og.ui.startViewTransition(path)`
  is `element.startViewTransition()`, the update being whatever the game
  changes in the same tick. Its old view is the shared elements of that
  element's subtree in last frame's commands; its groups draw right after
  the element's own commands (over its children, outside its clip and
  opacity, under what follows) and are styled by the `::view-transition-*`
  rules of the element's document, a component's too (the UA's without
  any). Any number run at once, beside a layer's; starting one on the
  same element replaces it. A name is found by the transition of the
  nearest of its element and its ancestors with `view-transition-scope:
  all` or a running scoped transition, else by a layer's. The element is
  not given layout containment: og-ui lays out the whole frame each tick,
  so nothing outside it moves with its subtree.

#### Inline SVG

Inline SVG draws data-bound vector graphics: `<svg viewBox>`, `<g>`,
`use`, `text`, `tspan`, `textPath`, the basic shapes, `image`,
`foreignObject`, `defs`, `symbol`, `marker`, the SMIL elements, `mpath`,
`clipPath`, `mask`, `pattern`, gradients and `stop`, `filter` with its
`fe*` primitives, `title` and `desc`; any other tag is refused. Every
attribute can bind; presentation properties are attributes or CSS and
inherit as in SVG; a paint also takes a style colour (`var(--accent)`).
The parser is the cook's (`og_ui::svg`, shared with `type vector`).

- An `<svg>` is a box of its CSS size (default its view box) fitted
  `xMidYMid meet`, strokes and dashes scaling with it. Each shape is one
  element (an analytic fill path and stroked outlines), so a changed
  attribute redraws that shape only and a resize builds nothing.
- `<use>` copies a symbol, group or shape at parse, inheriting from the
  use; a bound `x` costs the use and its copy. `clip-path` and `mask`
  cut by analytic coverage with `clip-rule`/`fill-rule` (one rule per
  clip; a mask of one gradient-filled shape fades by luminance × alpha).
  A `<pattern>` fill tiles a rect, circle, ellipse or path, at most 4096
  tiles a shape. `<title>` names its parent (bindable; a named `<svg>` is
  an image); `<desc>` is dropped.
- `<text>` is a line of og-ui text laid out at its view-box size, anchored
  on `(x, y)` by `text-anchor` and `dominant-baseline` (the font's
  baselines, as HTML text takes them) and scaled by the
  fit; `<tspan>` takes weight, style, decoration and fill, and with `x`
  starts a new line. `white-space` (or `xml:space="preserve"`) keeps
  spaces and breaks. A group's transform moves and scales text but does
  not turn it. Right to left, SVG text stays where its view box shows it.
  `<textPath>` uses og-ui's text on a path (below): glyphs turned rigidly
  on the font's advances.
- Markers follow SVG (`orient`, `markerUnits`, `context-fill`,
  `context-stroke`). A one-dash `stroke-dasharray` draws natively; a
  longer list cuts the outline.
- `<foreignObject>` lays document elements out in its box, clipped and
  scaled with the view box; they take input and accessibility.
- SMIL (`animate`, `set`, `animateTransform`, `animateMotion`) runs on the
  UI's tick clock from when its element first shows, kept in widget memory
  outside `world_hash`; an animating element resolves each frame. Event
  `begin` values (`click`) are refused: start motion with `og.tween` or a
  CSS state. Animation and definition attributes are literal; a `<text>`
  animates from a `<g>` around it. Reduced motion shows the frozen end.
- `<image href>` draws an atlas icon by name or a render texture by
  handle, contained by default; it takes no turn or skew.
- `filter="url(#id)"` (and CSS `filter: url(#id)`) runs every Filter
  Effects 1 primitive in the element's user space, a turned element
  filtered as if unturned. Limits: `feComponentTransfer` tables keep 16
  values, `feConvolveMatrix` at most 64 cells, a blur reads at most 128 px
  a side (wider ones run as σ/√n passes), `BackgroundImage`,
  `BackgroundAlpha`, `FillPaint` and `StrokePaint` read transparent black.
  A CSS-box filter region defaults to the border box grown 10 % a side, in
  its CSS px. A CSS `filter` function list on an SVG element
  (`drop-shadow()`, `blur()`…) draws as a box's does, and a rule's `filter`
  overrides the attribute, the property it presents.
- `<feImage href="#id">` draws that SVG element as `<use>` would (Filter
  Effects 1 §15.17): in the filtered element's user space, moved by the
  primitive's `x` and `y`, inheriting the filtered element's computed
  values. The element is mounted again where the filter is, in the
  filtered element's context: its bindings follow their stores and its
  `{#if}`, `{#each}` and components are built, reading what the filtered
  element reads (its stores, its row, its component's props). One that
  reads a row the filtered element is not in draws nothing, with a
  warning, as does one that is the filtered element or an ancestor of it.
  The image is a picture: nothing in it takes input, sends an event,
  writes a store or runs a `use:` action, and to selectors it is no child
  of the filtered element. Images of images nest four deep. An idle image
  builds nothing, a changed one writes only what changed, and it is
  unmounted with its filter, its `href` or its element.

#### Pointer, drag and drop

`draggable="true"`, `ondragstart`, `ondragend`, `ondragover` and `ondrop`
use HTML's names; a dragged element springs home on release, drops filter
by `data-accept` against `data-drag-type`, and events carry `{ dragged,
target }` (row keys) and `{ x, y }` in the handler's box. `onpointerdown`,
`onpointermove`, `onpointerup` and `onwheel` fire on any element with
pointer capture and bubble to the nearest handler; `onpinch` (no web
standard) carries `scale` and centre `x`, `y`. These come from one pass
over the held, hovered and drop-target elements, so they cost nothing as
the tree grows; a hovered element with a pointer handler is written each
tick.

#### Engine elements

None of these has a web standard; each is named for what it does.

- `<viewport src={$target}>` shows a camera's render target sized by CSS
  (`Ui::scene_viewport`), with `<model-viewer>`'s camera attributes
  (`camera-orbit`, `camera-target`, `field-of-view`, `min-`/`max-camera-orbit`,
  default `auto 22.5deg auto` to `auto 157.5deg auto`, `camera-controls`,
  `bind:cameraOrbit`). `bind:clientWidth`/`clientHeight` report its size;
  its pointer events add `ray` (`{ origin, direction }`). With
  `camera-controls` a drag orbits, a fling coasts (also under reduced
  motion), wheel and pinch zoom, and with focus Left/Right turn 15° and
  Up/Down tilt 10°. The orbit is UI state; a new authored orbit takes over.
- `<joystick action>` and `<dpad action>` (128 and 132 px by default,
  never under three touch targets) and `<button data-og-touch="jump">`
  feed og-input actions: their laid-out rects become zones after the
  frame, and the action map samples touches that started on a zone in the
  tick, so actions replay with the recorded input. A stick reads from its
  centre over its radius, y up, past `data-dead-zone` (default 0.15); a
  d-pad reads eight ways; a button is held while touched. A control keeps
  the finger that started on it; a finger sliding onto one takes nothing;
  disabled or `display: none` controls are no source. Hide them with
  `@media (pointer: fine)`. They are written only while touched. The
  built-in `TouchControls` lays a stick or d-pad and its action buttons
  out this way.
- `<anchor entity|at offset occlusion declutter clamp priority
  max-distance scale-at scale-min scale-max>` holds a column over a world point, placed every
  tick; a moving camera builds nothing. Decluttered by rank (priority,
  then nearness), each takes the nearest spot a gap clear of all those
  placed before it draw (their scaled and overflowing parts included) and
  of the grounds' text, on the screen and within its reach (`stack`
  straight above, then below, then any way; `shift` any way), and fades
  out while none is.
  Mark a screen element `reserve="hud"` to reserve its current border box;
  placement follows layout and transforms each tick. All tags respect HUD
  reserves. `keep-visible` changes only the no-fit declutter fallback: the
  tag stays at its own point rather than fading (targeted or casting plates).
  It does not change LOD, distance or occlusion; `lod-lock` remains independent.
  An empty `<anchor reserve="player" entity={player} reserve-radius="0.6"
  reserve-height="1.9" />` reserves an upright world volume based at the
  entity or `at` point. Its projected screen bounds are reported as the
  inert anchor box. Opt into it with `avoid="player"`: prompts, bubbles,
  loot, or a boss plate can yield while an enemy plate stays on its unit.
  `<ground danger>` reserves its whole projected bounds, including the rim,
  only for anchors with `avoid="danger"`; combine as `avoid="player danger"`.
  `placement-box` on descendants replaces the whole-subtree collision
  rectangle with the union of their transformed border boxes. Without a
  mark the visible drawn subtree is used; overflow clips exclude clipped
  decoration. Drawing, containment and hit testing do not change. Use it on
  the chip of a label whose thin decorative beam must not reserve an empty
  full-width column.
  `max-shift` caps declutter translation in CSS px, independently of text
  scale (zero pins it; omitted keeps ordinary reach). It also disables
  an otherwise unbounded edge fallback. With `avoid="danger"`, if no
  fully clear slot fits inside the cap, placement first takes the nearest
  slot clear of reserves and other tags, even if it intersects the danger
  bounds, and reports `[data-blocked]`. If even that is impossible the
  ordinary least-overlap fallback still respects the cap and hard reserves.
  The built-in LootLabel marks its chip, opts into danger avoidance and
  caps displacement at 128 CSS px. Rarity ranks legendary 50, epic 40,
  rare 30, uncommon 20 and common/overflow 10. Its ground glow stays at
  the item; beam decoration does not consume chip collision space.
  HUD/player reserves take precedence over danger, then other tags in
  descending priority. Boss/casting plates at 400 therefore precede a
  prompt at 300; no extra obstacle class is needed. A danger-avoiding tag
  with no clear reachable candidate stays visible at the candidate with
  least danger overlap, then least tag overlap; its root gets `[data-blocked]`
  on the next tick. Hard reserves still win when no candidate clears them.
  Normal moves stay within reach. An edge-clamped tag slides only along
  its edge to clear reserves. A player-avoiding tag with `clamp="edge"`
  may fall back to the nearest clear safe edge, reporting the normal
  clamp/angle facts for its tail. Fades still use `fade_ticks`.
  `scale-at="12" scale-min="0.72" scale-max="1"` scales the whole box by
  distance, never above its designed size; omitted bounds remain 0.5–2.
  Opt into `text-floor="11"` to keep text legible: its drawn size is the
  smaller of its designed size and the larger of its distance-scaled size
  and 11 px times user text scale. Omit it or use zero to disable. Each run
  has its own designed-size cap, including an `em` child; a title designed
  below the floor keeps its size rather than growing to the floor. Text's
  `em`/`ch` measures and em tracking follow its floored size. Pixel measures,
  padding, icons and bars still scale with distance; the plate grows only
  around the text it now measures, before decluttering. A 150 px scrolling
  name measure still draws 108 px wide at scale 0.72; use an em measure if
  its width should follow the text instead. Ground text and mass bars do
  not take the floor.
  Keyed loot anchors with the same `stack-group="pile-12"` form one
  column. Rank is priority, then distance, then stable build order; the
  highest-ranked item supplies the point, scale, text floor's distance,
  max-distance, occlusion and avoidance for the whole column. Its
  `stack-max="5"` bounds visible item rows (0 means unlimited); the game
  can pass 1 on a phone. Best is at the bottom, growing upward with
  `stack-gap="3"` CSS px before distance scaling. Rows share the widest
  visible item's natural width; author a max-width/measure on each item.
  An optional `<anchor stack-group="pile-12" stack-more
  bind:hidden={$hidden}>…</anchor>` borrows that point without `at` or
  `entity`, and occupies the top row only while there is overflow. Its
  two-way binding writes hidden members' each keys, ranked, into a
  **string-list signal** on the next tick: render `+{$hidden.length}` and
  look up rarity dots in game data. `hidden` without `bind:` keeps its
  ordinary HTML visibility meaning. Without a stack-more row overflow
  simply fades out; hidden rows take neither hits nor obstacle space.
  The entire column declutters as one box. Keyed pickup, insert and rank
  changes reuse the existing shift easing and fade_ticks; a row leaving
  its group fades at its old column point before returning to its own.
  Hover transforms change ink, not the column's widths or row slots.
  Column transition and mass hit-trail state are included in world snapshots
  (version 3, which still reads version 1 and 2 snapshots).
  A mass `<anchor mass="overhead" value={hp} max={hpMax} shield={shield}
  segments="5" resource={mana} resource-max={manaMax} bar-ghost>` draws
  health, optional shield above and resource below in one GPU instance.
  CSS width/height define health, distance-scaled but floored at 30×3 CSS
  px independently of user text scale. Shield is `shield/max` of the same
  width, height `max(1px, round(designed height × 0.4) × drawn/designed height)`;
  zero shield draws no track or gap and never moves health. Resource is
  `resource/resource-max` of 80% width, height using 0.6 instead of 0.4;
  zero/omitted resource-max hides it. Track gaps scale from 1 CSS px.
  `color` is health ink; `--bar-back`, `--bar-rim`, `--bar-shield`,
  `--bar-resource`, `--bar-ghost` and `--bar-low` supply CSS colors.
  At or below `--bar-low-at` (default 0.25), the 1px border uses low ink.
  `segments` cuts health/ghost into equal parts with 1px back-ink gaps;
  cuts disappear when the remaining segment width would be under 3px.
  Bare `bar-ghost` enables a 400ms hold and 300ms linear drain after damage;
  `false`/`none` disables it, and a legacy color value also enables it and
  sets ink unless `--bar-ghost` overrides it. Keyed rebuilds and snapshots
  preserve the trail. Mass bars support `occlusion="fade|hide"` (fade alpha
  0.35), max-distance's last-tenth fade and the normal scale bounds.
  World bars paint far-to-near across templates, beneath all text anchors;
  they do not declutter or become placement obstacles.
  `clamp="edge"` pins off-screen and behind-camera points with an arrow;
  `clamp-margin` (default 24) and `arrow-size` (default 14, zero disables it)
  are CSS px multiplied by user text scale. `arrow-color` sets its ink.
  `clamp="contain"` pushes the whole box into the safe area, inset by
  `clamp-margin` (default theme small spacing), and fades it as its point
  leaves the edge; it does not pin arbitrarily distant off-screen points.
  `contain-fade="none"` disables only that edge fade, keeping an off-screen
  boss's contained plate fully readable; distance, occlusion and declutter
  still fade normally. The default is `contain-fade="edge"`.
  To pin a speech bubble without an arrow use `clamp="edge" arrow-size="0"`.
  Placement feedback reaches the anchor root on the next tick:
  `[data-clamped]` and `[data-edge="top|right|bottom|left"]` while edge-pinned,
  and inherited `--anchor-angle` in degrees clockwise from +x. Containment
  does not set clamped feedback. A behind-camera target follows the direction
  the player would turn, meeting the first safe edge rather than always bottom.
  `lod="25 45"` supplies strictly ascending positive world distances: level 0
  before 25, 1 from 25, 2 from 45, read as `[data-lod="0|1|2"]`. A bound
  `lod-lock={targeted || casting}` forces level 0 without changing scale,
  clamp, fade or declutter. LOD is distance-only; crowd fallback is not implied.
  Style child details from these facts; placement itself does not animate on
  a feedback change (an authored child opacity transition is independent).
- `<panel world entity|at size rotation src>` lays a box out at its CSS
  size into the render target `src`, which the game shows on a quad of
  `size` world units; pointer rays meeting the quad reach its controls
  (none without `entity` or `at`).
- A keyed `{#each}` of `<anchor mass="name">` is GPU-instanced mass
  elements: a number, or with `max` a bar of CSS `width`/`height` tinted by
  `color`. Its first CSS `animation`'s `translate`, `scale` and `opacity`
  keys are the template's timeline: a row with one spawns one instance when
  it appears (a damage number); one without stands with its row (an
  overhead bar), pushed every tick with nothing built.
  The same `scale-at`, `scale-min`, and `scale-max` configure its native
  distance scaling (mass defaults: constant size, bounds 0.25–4).
  `bar-ghost` sets the ghost track's ink, not a hold/drain animation.
  Numbers and texts spawned in the world on one tick are laid apart as a
  stack, clear of the tags and of those still rising (a target's hits
  stand one over another, never on each other); one with no room in
  reach is not drawn.
- `<og-emitter effect>` emits UI particles while its element stands: a
  preset (`sparkles`, `confetti`, `embers`, `snow`, `bubbles`, `dust`,
  `sparks`, `fireworks`) in the style's colours or a rate, with `burst
  <n>`, `for`, `life`, speed, direction, shape, size and colours. A
  keyframe's `-og-burst: <rays> <length> <colour> <time>` fires a burst
  as the iteration reaches it. `<og-overlay effect="vignette <colour> |
  tint <colour> | material …">` draws a full-screen overlay that takes no
  input, its strength the element's `opacity`. Both register every tick
  with nothing built; reduced motion silences bursts and emitters.
- `<ground entity|at size rotation>` lays a box as a decal over the
  frame's depth prepass (`UiGround`): it drapes over terrain within a
  quarter of its shorter side of the plane, fades on surfaces across it,
  is hidden by nearer geometry and lies on the plane where there is none.
  The camera moving changes only its affine map (`UiDrape`) and screen
  bounds (`UiFrame.grounds`), never the world. Its layer is sized from its
  seen size, a power of two up to 2048 px (the cap over the whole screen
  when a corner is behind the camera). What overflows its box (a path's
  badge past its end) is drawn too, on its plane. Its texts and badges
  draw over every ground's other drawing (rings, lines, fills), so a
  label reads over the decals laid after it. It is inert.

An anchor or world panel lives in a world layer of its own keyed by its
element's path, so its nodes keep the keys and paths they would have in
place and a retained patch writes only the changed element. All of these
are presentation, outside `world_hash`.

Fixtures supply presentation states, never playable economies, combat
loops, matchmaking or rhythm clocks.

### 2.2 Dialogue text and attribute spans

`DialogueView` receives localized plain text and attribute spans
`{ name, position, length, properties }` with typed string, finite number
or boolean properties. Positions and lengths count Unicode scalar values.
A `character` span covers the `Speaker: ` prefix, which stays in the text.
The Yarn 3.2.2 runtime resolves `nomarkup`, escapes and CLDR select,
plural and ordinal expressions before delivery; UI never re-parses text as
markup.

`<text spans={$spans}>{$body}</text>` maps span names to `::part` styles
and motion (`text::part(name)`, `#Dialogue::part(name)`). Overlapping
spans override earlier ones, in array order, only for fields they declare;
unknown and zero-length spans pass through unchanged. Styled runs share
core shaping and line breaking. Span motion is checkpointed and restores
exactly; reduced motion makes it stationary. Components use this core text
path, never a private renderer.

### 2.3 Text input (`Input`)

`<Input>` from `@origingame/engine/ui` is the engine's text-entry primitive.
It accepts `value`, `placeholder`, `type` (`text`, `password`, `number`,
`email`, `url`, `tel`, `search`), `inputMode`, `autoComplete`, `secret`,
`multiline`, `maxLength`, and `readOnly`, alongside standard `HostProps`
(`onChange`, `onFocus`, `onBlur`, `onKey`, `disabled`, `tabIndex`, `label`).

- **Editing & keys**: typed text arrives through `og-input`'s `og.TextInput`
  stream (with printable-key fallback when no stream is active). Backspace
  and Delete step by UAX #29 grapheme cluster; `Ctrl`/`Alt` + Arrow /
  Backspace / Delete step by word; `Shift` extends selection; `Ctrl`/`Cmd` +
  `A`/`C`/`X` handle select-all, copy, and cut; `Enter` inserts a newline
  when `multiline` is set.
- **IME composition**: active preedit text (`input.composition`) is rendered
  inline at the selection with an underline and does not fire `onChange`
  until committed. Blurring clears uncommitted preedit text.
- **Password & read-only fields**: `type="password"` or `secret={true}`
  masks displayed characters with `•`, reports an empty selection to the
  host, and blocks copy/cut. `readOnly={true}` keeps focus, selection, and
  copy while ignoring edits. `maxLength` caps Unicode scalar count.
- **Host text-entry bridge**: every frame while an `<Input>` is focused,
  the runtime includes `text` (`{ path, rect, multiline, selection,
  composing, type?, inputmode?, autocomplete?, secret? }`) in CSS px of the
  play surface and `copy` (when copied/cut) in the frame answer. The web
  shell (`packages/engine/src/web/text-input.ts`) positions its hidden
  `<input>` or `<textarea>` over `rect` so mobile keyboards, IME candidate
  windows, autofill, and browser clipboard events work seamlessly.

#### Live regions

A node with a `live` prop (`"polite"` or `"assertive"`), or the role
`status`, `log` (polite) or `alert` (assertive), is a live region in the
accessibility tree (`crates/og-ui-next/CONVENTIONS.md` §Accessibility);
`status` and `alert` are atomic. The web mirror sets `aria-live` and
`aria-atomic` on the mirrored node, and native windows forward AccessKit
updates to the OS screen reader.

### 2.4 Built-in controls

Reusable controls live in `packages/engine/src/ui/controls/` and are
re-exported from `@origingame/engine/ui/components` (and individually under
`@origingame/engine/ui/controls/<name>`). Each control uses pack tokens and
`packClass(...)` slots, exposes full keyboard/gamepad/ARIA semantics, and
ships `fixtures` for `og ui check` and `og ui preview`. Run
`og ui eject controls/<name>` to copy any control into a game to customize.

| Control | Pack slots | ARIA role & behaviour |
|---|---|---|
| `Art` | pack nine-slice class | Stretched or nine-sliced pack artwork |
| `Button` | `Button.plate[.primary\|danger]` | `button`; variants (`primary`, `danger`, `ghost`, `quiet`), `small`, `pressed`, `hold` |
| `Checkbox` | `Checkbox.box` | `checkbox`; boolean or `"mixed"` indeterminate state |
| `Counter` | — | Animated numeric readout (`tabular-nums`) |
| `Dialog` | `Modal.scrim`, `Panel.frame`, `Modal.frame` | `dialog` / `alertdialog`; scrim, focus trap (`modal={true}`), `back` dismiss, scrollable body & footer |
| `Meter` | `Bar.fill` | `meter`; scalar gauge with low/high/optimum regions |
| `Progress` | `Bar.fill` | `progressbar`; determinate 0..1 progress bar |
| `Select` | `Select.trigger`, `Select.menu` | `combobox` + `listbox` + `option`; press to open, Arrow keys to cycle, Escape/`back` to close |
| `Slider` | `Slider.track`, `Slider.fill`, `Slider.thumb` | `slider`; pointer drag and Arrow/Home/End stepping |
| `Switch` | `Switch.track`, `Switch.thumb` | `switch`; binary toggle with animated thumb |
| `Tabs` | `Tab.bar`, `Tab.indicator` | `tablist` + `tab`; focus-group wrapping and active indicator |
| `Tooltip` | `Tooltip.frame` | `tooltip`; lightweight label/detail anchored to a named UI node |

### 2.5 Split screen

`og.ui.setPlayers(n)` turns pads 1 to 3 on; each pad moves its own focus
(`Response::focused_by`, bit `k`) and presses with A (`Response::pad`).
`player="k"` (Rust `Props::player`, 0 to 3) makes a subtree player `k`'s
region, which may be its own scene viewport.

- Pad `k`'s focus moves only within player `k`'s region; its first move
  lands nearest the region's centre. Pad 0's focus is also the keyboard's
  and pointer's.
- Controls outside every region belong to pads with no region; an open
  dialog's controls are reachable by every pad.
- `Response::player` names the region a control was pressed in, `None`
  outside every region and on unpressed ticks; the pointer presses
  anywhere.
- A screen with no region is unchanged: every pad sees every control.

A hit carries its region as one byte, inherited like the focus group and
computed when ancestry changes; it is saved with the UI state.

## 3. Style contract

### 3.1 Tokens and literal values

Every visual value a built-in component or screen draws with (colour,
type, spacing, size, radius, stroke, shadow, motion, sound) is a theme
step or role, written as a Tailwind class over NSH's theme. A component
writes no value of its own; the rule, its exceptions and what `og check`
reports are stated once, in `crates/og-ui-next/CONVENTIONS.md` §Theme and
packs ("Steps and roles", "Grounds and inks"). In short:

- A ground under text is a pack slot (`packClass("Plate", "frame")`) or an
  opaque theme ground role, and its ink is that ground's pair (`text-fg`,
  `text-<role>-fg`, a HUD colour's `-ink`). Text straight over the scene
  sits on `Plate.frame`; only a figure of 24 px or more may stand on the
  text-stroke halo. OG9108 reports a break at the class.
- No hand-written colour, length, shadow or opacity modifier: a bracket
  value or a literal `white`/`black` is OG9109 with the nearest step or
  role named. An inline `style` carries only what a prop computes (a
  fill's width, a node's position).
- Literal values belong only in the token sources and a pack's
  `pack.json`, art metadata (slice insets, atlas cells), fixture data and
  structural geometry derived from props.

The default theme is generated: `packages/design/tokens/*.css` is the
source (seeds, colours per scheme, type, space, radii, shadows, motion),
and og-ui-next writes from it the default pack,
`crates/og-ui-next/default/pack.json` (`scripts/gen-pack.sh`), and
`packages/design/tokens.json` for the platform (`scripts/gen-tokens.sh`).
Both are drift-checked by og-ui-next's tests; neither is edited by hand.
When the theme lacks a role a component needs, the role is added to the
token sources, not written in the component.

The token families of those sources, which the default pack is generated
from and a pack's `theme` overrides:

| Token family | What it controls |
|---|---|
| `--seed-*` | The seed colours every colour is derived from (`seed.css`; a pack's `seeds`) |
| `--color-*` | Colours per scheme: grounds, inks and their `-fg` pairs, status, HUD and team colours (`text-*`, `bg-*`, `border-*`) |
| `--space-*`, `--size-*`, `--radius-*`, `--stroke-*` | Spacing steps, icon and art sizes, corners, line widths |
| `--font-*` | The type ramp: families, sizes, weights, line heights, tracking |
| `--shadow-*`, `--inset-*`, `--elevation-*`, `--glass-*`, `--opacity-*` | Shadows and inset shadows per scheme, drop elevations, glass blur, opacity steps |
| `--motion-*` | Durations, easings and named animations |
| `--density-*`, `--state-*` | Each input's control and row heights; hover, press and disabled states |

### 3.2 Packs, slots and schemes

A style pack is one `pack.json` (with its fonts, art and sounds beside
it) laid over the default pack and under the game's own
`assets/styles/game/pack.json`; a later layer replaces an earlier one's
entry of the same name. It carries `scheme`, `seeds`, `theme` (entries
per scheme), `variants` (the classes each slot gets), `images` (nine-slice
art), `icons`, `fonts` and `sounds`. CONVENTIONS.md §Theme and packs gives
the format and every name the default pack defines.

A component exposes the surfaces a pack restyles as slots, ending a class
with `packClass(slot, part, variant)`: `Button.plate` (`primary` its main
action), `Panel.frame`, `Plate.frame`, `Slot.frame`, `Chip.frame`,
`Field.frame`, `Row.frame` and `Head.frame`. `Focus.ring` dresses the
ring every control shows while a keyboard or pad focuses it, unless the
control styles `focus-visible:` itself. `Press.feedback` is the squash
every enabled control that answers a press shows under a finger (its
`active:` pose, in on its `active:` curve and back on its spring), unless
the control moves itself under `active:`; `Screen.frame` is how a
full-screen built-in enters and leaves (a screen that replaces one enters
once the leaving one has left), `Modal.frame` how a dialog pops in and
leaves, `Modal.scrim` the scrim it dims the screen with, fading on its own;
`List.cascade` and `List.item` how a list's rows enter one after another,
`Reward.pop` how a reward pops in, and `Count.roll` how a number rolls to a
new value (`stagger-*` and `transition-count`, CONVENTIONS.md);
`Tab.indicator` the mark that glides to a newly selected tab or segment
(`glide-<name>`), `Bar.fill` and `Bar.trail` how a bar eases and its lost
part drains after it, and `Attention.cue` what a control plays once when it
becomes enabled.
The default pack derives the four from motion.css's timing table (press,
enter, exit, pop), so a pack restyles the whole UI's feel
by its tokens or its own variants. A slot that paints its own
fill owns the ink on it. A new slot is declared in the default pack and in
`design::SLOTS` before a component writes it.

The page runs in one scheme (`dark` by default; the player may choose
`light`). A slot whose art reads the same in both schemes names its own
with `scheme-dark` or `scheme-light`, so the roles and nine-slice images
inside it resolve in that scheme; a nearer `scheme-*` wins. The packs' dark
plate, chip and button art carry `scheme-dark`, and so do the title and
main-menu screens, which sit over a picture.

### 3.3 Assets and the gallery

Art, icons, fonts, import settings and licences travel with their pack
(`templates/styles/<name>/`) or the shared resource directory. Cut
corners, sockets and frames are nine-slice art as `nine-<name>` classes,
tinted by `bg-*`; they shrink their insets in proportion when the box is
smaller than the art (CSS border-image). `nine-none` draws no art, so a
state drops the art its base named and its fill shows
(`data-kind-owned:nine-none` on a chip whose component sets
`data-kind-owned:bg-success`); no pack image takes that name. Placeholder characters are not
finished icons, and core font fixtures never depend on a commercial pack.

The UI gallery (§6.5) draws every built-in component's fixtures under every
pack in both schemes and flags a mount that throws, a frame that draws
nothing, a glyph no face has and text under the contrast bar on the ground
it is drawn on. It is how a pack change is seen; frames are judged by what
they show. The components and their props are listed in the `components`
guide, generated from their source (`og api show components --guide`).

### 3.4 Font packs

Fonts ship as packs, one directory per script under `templates/fonts/`
(`latin`, `cyrillic-greek`, `arabic`, `hebrew`, `devanagari`, `thai`,
`japanese`, `korean`, `chinese-simplified`, `chinese-traditional`,
`emoji`), each an `extension.json` of kind `font` naming `scripts`,
`languages` and `faces` with import settings and licence. `og font list`
names them; `og font add <pack>` installs one under `assets/fonts/<pack>/`;
`og create --lang` installs its languages' packs (`zh-TW` the traditional
one) with no generated code. `templates/fonts/make-packs.py` makes them
from google/fonts; `templates/fonts/LICENSES.md` lists sources and
licences.

The cook writes each pack's index (`og.FontPack`: languages, face ids,
covered code points) beside its faces, each cooked whole to
`<id>.face.ogb`, so a pack costs about its files' size. A pack changes no
other font's cooked bytes.

A pack also carries the hyphenation dictionaries of its languages
(`<lang>.hyph`, listed under `hyphenation` in its `extension.json`):
`latin` 27 and `cyrillic-greek` 7 of hypher's hyph-utf8 tries (MIT, from
TeX's patterns; `make-packs.py` fetches them). The cook puts a dictionary
in the pack's index (`hyphenation:<lang>`) only when a module the game's
bundle holds (its own or an engine component it imports) writes a
`hyphens-auto` class, or a style sheet declares `hyphens: auto`, and one of its string tables'
locales (else `en`) takes that language, so a game without automatic
hyphenation ships none. A shipped locale whose pack has no dictionary
for it is warned of at cook; its text breaks only at soft hyphens.
og.i18n installs a pack's dictionaries when it loads the pack, and the
next layout hyphenates with them.

ICU's word dictionaries for the scripts written without spaces ship the
same way, so the engine links none of them (about 3 MB of wasm): `cjdict`
(Chinese and Japanese), `thaidict`, `laodict`, `khmerdict` and
`burmesedict`. The cook copies one out of ICU's compiled data
(og-i18n's `segmenter::compiled`) into the index of the first pack (by
path) that names a language needing it, as `segmentation:<dictionary>`,
when one of the game's string tables' locales takes that language. A
shipped locale that needs a dictionary no pack names is warned of at
cook. og.i18n installs them with the pack (`Ui::add_segmentation`);
Parley's line breaks (through a hook patched into parley_engine,
`third_party/parley_engine/PATCHED.md`) and `og_i18n::word_segments` then
cut by them, and without them those scripts break by UAX #14 and #29's
rules alone.

og.i18n chains the active table's fonts, the default pack's body face,
the locale's pack, the packs a frame needed, then the other tables'
fonts, so a character a pack's faces lack falls back the way a browser's
does. A character only an unchained pack covers loads that pack (a
Chinese locale's own script first); until a face lands its characters
draw from the next face of the chain, and the layout takes the face over
when it arrives, never blank and never warned of. A character no face
covers keeps the missing-glyph warning. The web boot does not preload
`.face.ogb` files (the service worker caches them on fetch); the first
frame waits only for a character no loaded face has
(`og_web_fonts_pending`).

Every glyph is rastered from its face's outlines the first time it is
drawn at a size, a subpixel bin and a form (fill, stroke or blurred
shadow) into the frame's glyph atlas (`og_text::strike`): coverage at the
drawn size for text (Skia's mask gamma, Chrome on Linux's gamma 1.2 and
contrast 0.2, keyed on the ink's luminance), a distance field for world
text and the mass HUD that scale freely, and colour pages for COLR
glyphs. Steady frames cost a lookup a glyph. Glyphs are presentation
only.

A text shadow (a theme `text-shadow-<name>` class, or a pack theme's
`textShadow` entry; arbitrary `text-shadow-[…]` values are not
supported) is a list of up to eight layers, painted last first under the
fill as CSS paints them; a layer in `currentColor` or
`contrast-color(currentColor)` is resolved against each text's ink, so a
halo written on a HUD's root rings every text in its own contrast. The
default pack's `text-shadow-tag` is `--tag-halo`. Each layer draws each glyph's
silhouette blurred by a Gaussian of standard deviation `blur-radius / 2`,
rastered once per glyph and radius into the atlas. Its paint bounds
include glyph overhang, the offset and three standard deviations,
independently of the line height. Shadows draw back to front below the
foreground ink and composite source-over in sRGB-encoded space, as in
CSS; ordinary UI paint keeps its existing blending. An authored overflow
clip still cuts the final shadow. A text stroke (`text-stroke-*`) is each
glyph's outline stroked at its width (`og_text::raster`), drawn below the
fill.

Colour emoji are a font feature. The `emoji` pack is Noto Color Emoji's
COLRv1 face (SIL OFL 1.1; script `Zsye`, no language, tried after every
text pack): COLR v0 and v1 glyphs are painted when first shown
(`og_text::colr`) into the atlas's colour pages in the text's palette;
CPAL 0xFFFF layers take the text colour. GSUB forms skin tones, ZWJ
sequences and flags; spaces and punctuation after an emoji take a text
face's glyph. The pack is about 5 MB, paid only when installed. Any
project font with COLR glyphs draws them in colour. References:
`color-emoji-1280`, `color-emoji-390`.

CSS `font-palette` (Fonts 4 §9, inherited) draws such a font in another
CPAL palette: `light` and `dark` take the first palette the font marks for
that background (else its first), and a `--name` takes the last
`@font-palette-values --name` in the document's stylesheet whose
`font-family` lists one of the text's named families, from its
`base-palette` (an index past the font's palettes: its first) with its
`override-colors`; a name with no rule for the family is `normal`, the
cooked palette. The text is drawn with an instance of the font in that
palette, whose glyphs are painted into the atlas the first time they are
drawn.

`palette-mix([in <method>,] <palette> [<n>%], …)` mixes its palettes
entry by entry as `color-mix()` mixes colours (OKLab when no method is
named; `normal` in it is the font's first palette), and `font-palette`
transitions and animates through such a mix in OKLab, so each frame of
the move is a palette of its own. Instances are shared by their 8-bit
colours.

Text is shaped in its language (CSS Fonts 4 §6.13): the element's `lang`
attribute, inherited (an empty one is unknown), or the OpenType language
system `font-language-override` names (`"ROM"`), so a face's `locl` forms
for it apply (Romanian Ș for Ş in Noto Sans). Parley, which lays out
paragraphs and edited text, takes a BCP 47 language only: a tag is given
as the language HarfBuzz shapes in it first
(`scripts/gen-ot-languages.py`), and one no language shapes in first
applies only outside Parley. `@font-face`'s `font-display` changes
nothing: a document's faces load with it, before its first frame.

`font-variant-emoji` (Fonts 4 §6.12, inherited) picks the face an emoji
character is drawn in where both a text face and a colour face of the
text's fonts have it: `emoji` the colour face, `text` the text face, and
`unicode` (and `normal`) as its Unicode `Emoji_Presentation` says (⌚ as
emoji, © as text). A variation selector in the text wins: U+FE0F draws
the character before it as emoji, U+FE0E as text. A keycap base (a digit,
`#` or `*`) is text except in a keycap sequence.

`font-size-adjust` (Fonts 5 §2.5, inherited) draws each face of a text at
the size where its metric is the given number of the text's size: the
x-height by default, or `cap-height`, `ch-width` (the advance of "0"),
`ic-width` or `ic-height` (those of 水). `from-font` takes the first face's
own value, so a fallback face (CJK beside a Latin face, say) is drawn to
match it. A face is scaled with its advances, in plain and rich text; a
face without the glyph measured takes CSS Values 4's fallback (half an em
for `ch`, an em for `ic`). A face's extents scale with it, which places
its baseline; the line height stays the text's, and vertical text draws
its faces unadjusted.

An `@font-face`'s `size-adjust` (Fonts 5 §11.3) draws its face that much
larger, its advances, extents and x-height with it, so `font-size-adjust`
matches the adjusted face. `ascent-override` and `descent-override`
(§11.4) replace the face's extents, which place its baseline in the line
box. With `line-gap-override`, any of the three sets `line-height:
normal` in the face to their sum, the font's own ascent and descent
(`hhea`) standing in for one not given and no gap for a gap not given; a
face without overrides uses its font's selected vertical metrics for
`normal`: OS/2 typo metrics when USE_TYPO_METRICS is set, otherwise hhea,
falling back to nonzero OS/2 typo metrics and then win metrics when hhea
is empty (the Chromium/FreeType selection). Ascent, descent and line gap
are rounded separately at the used font size, then added. Only a face
without usable metrics keeps the UI's 1.2. Numeric and length line heights
remain authored values. `size-adjust` scales the metrics and overrides too.

Where a face has no small-capital form of a lowercase letter
(`font-variant-caps: small-caps`, Fonts 4 §6.3), or under `all-small-caps`
none of a capital, the letter is drawn as its capital at 80 % of the size,
letter by letter (Gecko's factor); a letter whose capital is more than one
character (ß) keeps its own. Where a face lacks the superscript or
subscript form of any character of a run but spaces
(`font-variant-position`, §6.5), the whole run is drawn at 82 % of the
size up to 20 px and two-thirds from 45 px (in proportion between; Gecko's
ratios), its baseline a third of the paragraph's size above or a fifth
below. A text that synthesises either is drawn as rich text.
`font-synthesis-small-caps: none` and `font-synthesis-position: none` draw
it as it is; petite capitals are not synthesised.

## 4. Shipped catalogue

Every component and screen document under `packages/engine/src/ui/` is
built into the engine (§5): the generic components and screens, and the
component groups below, each a directory of
`packages/engine/src/ui/components/`. Core default UI is the only creation
`ui` choice until a pack is accepted; a look's `ui` line or
`og.load.style` selects a built-in pack in any game. `materials` is an
optional resource pack for UI shader finishes, neither a component group
nor a style. VFX and unrelated extensions are outside this catalogue.

World components use the same placement solver as authored anchors:
`Waypoint` opts into `avoid="player"` and disables the native arrow with
`arrow-size="0"`, retaining its own chevron. `InteractionPrompt` opts into
`avoid="danger"`, and `Telegraph` declares its projected bounds as danger.
Games still author their player reserve; the world sheet's painted player
has an explicit silhouette proxy in its fixture data, not an inferred entity.
`Nameplate` uses `clamp="contain"` with a text-scaled 24 px inset and caps its
plate's width through `--world-nameplate-room`, accounting for the targeted scale. Containment uses
drawn bounds, including descendant transforms. Its name scrolls inside the
remaining header width after the nonshrinking level chip and icon, with
the usual edge fade; the subtitle and cast row share the plate's width limit.

| Component group | Required presentation |
| --- | --- |
| `rpg` | Inventory and equipment, journal and tracker, dialogue, party and turn order, world map |
| `shooter` | Aim, ammunition, damage-direction HUD, killfeed, loadout |
| `strategy` | Selection and commands, production queue, waves, tower-upgrade inspection |
| `racing` | Telemetry, laps and standings, race banner, garage selection, results screen |
| `card` | Hand and piles, collection and deck building, reward-choice and run-map screens |
| `puzzle` | Level path, goals and moves, level summary |
| `idle` | Production and purchase rows, prestige preview, offline-summary screen |
| `rhythm` | Song and difficulty selection, judgement presentation, results breakdown |
| `arena` | Roster and character selection, draft, versus, team scoreboard, round HUD, lobby screen |
| `general` | Shop, achievements, daily quests and rewards, chat, friends |

Every group is built token- and part-only (§3.1, §3.2), with state
fixtures (§6.1), the gallery renders them (§6.5), and every built pack dresses it.
Member inputs, intents and parts are documented in each group's README
(`packages/engine/src/ui/components/<group>/README.md`) and each
document's `<meta>` header; the `base/`, `hud/`, `world/` and `panels/`
components' in `packages/engine/src/ui/components/README.md`, the screens' in
`packages/engine/src/ui/screens/README.md`. Arena owns team and match
scores; shooter does not duplicate them. RPG also serves open-world games;
action-platformer and runner use core, with RPG or puzzle compositions
when needed. Components never calculate damage, production, purchases,
rewards, legal card targets, or note timing.

| Style pack | Visual language | Commercial review references | Status |
| --- | --- | --- | --- |
| `fantasy` | Painted frames and materials with room for dense content | Baldur's Gate 3, Diablo IV | Deleted (2026-11-23); rebuilt as a theme recipe |
| `scifi` | Instrument hierarchy, precise geometry, sparse emphasis | Elite Dangerous, Starfield | Deleted (2026-11-23); rebuilt as a theme recipe |
| `cartoon` | Rounded forms, deliberate thickness, clear primary actions | Mario Kart 8 Deluxe, Bloons TD 6 | Deleted (2026-11-23); rebuilt as a theme recipe |
| `pixel` | Consistent pixel grid, integer scaling, readable glyphs | Stardew Valley, Octopath Traveler | Not yet built |
| `storybook` | Paper and illustrated edges, quiet ornament, readable text | Pentiment, Strange Horticulture | Not yet built |

The engine ships only the default pack, in plain tokens with no frame
art, until the theme recipes replace the packs (the owner, 2026-11-23).
A built pack lives in `templates/styles/<name>/` and ships with the engine
as `@styles/<name>`, with the art it names. A pack is one `pack.json`
over the engine's default pack (crates/og-ui-next/CONVENTIONS.md §Theme and
packs): its seeds, the theme values it sets, its faces, its nine-slice art,
its sounds and the classes of the slots it dresses. It selects no component
part: a built-in component asks for its slot's dress with `packClass`. Each
pack's README lists what it sets.
`templates/styles/shared/` holds the fonts, sounds and art packs share (`/@styles/shared/`) and
`templates/styles/cjk/` the CJK faces; neither is a pack.

A pack becomes a creation choice only once it is accepted (§6.3); the
integration owner then updates the schema and bilingual labels together.
There are no `auto`, `custom`, or `generated` pseudo-packs.

The 12 look presets in `templates/looks/` and the generated look catalogue
own look-to-UI recommendations; every look names `default` until the
theme recipes land. A recommendation is a seed, not proof of fit; an explicit
creation UI choice wins. Game-authored styles use explicit asset paths in
the actual look; the creation block records the initial constraint, not
the current path.

## 5. Package lifecycle

Built-in documents live in the engine package: components in
`packages/engine/src/ui/components/`, filed by function: `base/` (building
blocks), `hud/` (drawn over play), `world/` (anchored in the world),
`panels/` (data, menus, inventories, logs), `meta/` (shop, social,
rewards), and a genre's own screens in its kit (`rpg/`, `card/`…). A piece
any genre uses is filed by function, never under a genre. Screens
and the generic components beside them in `packages/engine/src/ui/screens/`.
Their ids are under the reserved `@ui/` root: `@ui/components/<path>` and
`@ui/screens/<file>`. This section owns the use, load and eject workflow.

- A game's document uses a built-in component by its declared name
  (`<RpgBackpack items={$items} />`) with no copy or load. The cook
  resolves, transitively, every name a game's documents use that no game
  component declares, and bundles those documents with their string tables
  and files; two built-in components of one name fail the cook. Each
  cooked document records the documents defining its components, and
  `og.load.document` loads them with it, once each.
- A game shows a built-in screen by loading its id and pushing its name
  (`og.load.document(world, "@ui/screens/pause.svelte")`, then
  `og.ui.push(world, "pause", vm)`). The cook bundles each built-in screen
  whose id the game's code names as a string literal, with its strings and
  components. A built-in
  screen reads its declared props bare (`{title}`, `bind:open={open}`), as
  a component does, so one document is pushed with a viewmodel or mounted
  with stores; `$toasts` and `$captions` are the engine's stores, not
  props.
- A game document of the same name wins. `og ui eject <name>`
  (`panels/backpack` by its path; `--list` names all) copies a built-in
  document and its companions into `assets/ui/`; the copy then replaces it
  everywhere, and loading its `@ui/` id loads `ui/<file>`. Companion edits
  survive; `--force` replaces only the requested document.

Normal declarations rise through the cascade layers `og-default` (the
default stylesheet), `og-components` (every `<style>` and import of a
document under `@ui/`), `og-pack` (the style pack's imports, wrapped by
the loader), `og-game`, then unlayered CSS; `!important` reverses the
order, and sublayers stay below their parent's normal declarations after
sheets merge. A pack thus dresses every built-in component and a game
overrides both.

A style pack is a directory of `.css` files and assets named by its
basename, with no manifest, `style.ogt` or executable entry.
`og.load.style(engine, "styles/<name>")` loads the cooked directory and
`og.ui.configure(engine, { style })` selects it (`StylePack::parse`,
`Ui::set_style_pack`). Direct-child CSS files apply in lexical filename
order, nested sheets through `@import`. Imports and URLs resolve beside
the file containing them; a leading slash names an asset-root path
(`/@styles/shared/`, `/@styles/cjk/`). Pack CSS reaches authored document
elements and exposed `::part()`s.

A document is parsed once, against the style in effect: the pack's rules
for its elements are placed with its own as its text is read
(`Ui::parse_document`), and defining it, a build or another
`og.ui.configure` reads nothing again (`UiDocument::parses` counts the
readings). A pack's tokens and its plain `::part()` rules are the UI's
and follow the pack at once. So a game selects its style before it loads
its documents. When `og.ui.configure` changes the style under loaded
documents, as a style edited under `og dev` does, the runtime reads
again, once, each loaded document the change reaches
(`Ui::restyled_documents`): a document records the style's rules that
reached its elements, so another pack, or a changed selector, layer,
keyframes or at-rule, reads them all; a changed rule body reads those it
reached; a changed token (a `:root` custom property, the root cascade's)
reads none. The cook does not style documents: a cooked
document is its text, and a parsed document has no cooked form.

Built-in styles live in `templates/styles/<name>` and ship in the engine
package's `styles/<name>`, loaded as `@styles/<name>`
(`og.load.style(world, "@styles/<name>")`, or a look's `ui "<name>"`); the
cook bundles only a style the game names. Resource and effect packs live in
`packages/engine/extensions/<name>` and font packs in
`templates/fonts/<name>`, each described by its `extension.json`.
Discovery reads directories and manifests, never a hard-coded name
registry, and never evaluates pack code.

`og style add` copies a style to `assets/styles/<name>`. `og ui add
materials` copies the materials pack (kind `materials`, without its review
`sheet/`) to `assets/materials/materials/`, its presets
`materials/materials/<preset>.ogt`. Installed copies win over bundled ones
of the same kind and name; installation keeps relative paths and licences
and never silently replaces edits. Options are in CLI help.

A look resolves an installed style before a bundled one of the same slug;
explicit asset paths stay valid. The cook includes referenced style
resources and their shared dependencies, not every pack. Tests, reference
frames and art sources are not runtime payload.

Public helper signatures, built-in component names, inputs, intents and
parts, and the creation schema are contracts: a break needs a breaking
entry with its fix in a new `changelog/unreleased/` fragment, released in
the next patch (docs/release.md §3). There are no legacy aliases. Planned groups and packs may be
absent on main; incomplete engine archives cannot ship. Release waits for
the complete approved catalogue and platform schema and pin coordination
([release rules](release.md)).

## 6. Author and accept

### 6.1 State fixtures

Each component group starts with an input/output and parts inventory, its
normal and stress pages (§6.2), and two named commercial games as
information-structure references, each an actual page, platform and
screenshot source. Authors copy no commercial assets, brands or layouts.

Every member has reproducible fixtures and observable behaviour assertions
for each applicable case; an inapplicable case needs a reason.

| Cases | Required evidence |
| --- | --- |
| Default, hover, pressed, focus-visible | Pointer exit restores state; focus remains visible independently of selection |
| Selected, disabled, selected+disabled | Distinct combined states; disabled emits no activation |
| Empty, single, normal, overflow | Clear empty outcome; long names and dense content scroll, wrap, or intentionally truncate; primary action remains reachable |
| Stable IDs and changing data | Select the third item, reorder, then remove the first; selection retains identity; snapshot and restore retain valid UI state |
| Input and cancellation | Pointer, keyboard, gamepad, and phone touch; drag, confirm, and cancel return the intended result once |
| Input prompts | Switch live between keyboard and mouse, each supported gamepad family, and touch; glyphs and prompts follow the active device |
| Accessibility and localization | 100% and 150% text; pseudo-locale +40%, CJK, RTL; semantic names, roles, states, focus order, and safe areas |
| Motion | Start, midpoint, completion, interruption, and reduced motion; a short clip as well as stills |

A component's fixtures are its `fixtures` export, beside it in its
`.tsx`: one named prop set per state it has (empty, filled, selected,
disabled, overflow, as the component does). A fixture other than
`default` is laid over `default`: it names only what its state changes.
The gallery (§6.5) renders
every one of them; the group's behaviour tests
(`packages/engine/tests/<group>.test.ts`) mount the same components.

Core tests use small self-contained styles for core behaviour
(specificity, font pixel snapping, probe paths); pack tests check pack art
and resources. Removing a pack cannot remove core coverage.

### 6.2 Frame matrix

Every fixture of every built-in component is a page of the gallery
(§6.5), under the default style and every pack in `templates/styles`; a
stress fixture (long names, dense or overflowing content) is a fixture
like any other. Gallery frames are review material: locale, right to left,
landscape and further states get risk-based review captures, and any
permanent reference names the regression it guards. Neither the tests nor
the gallery is visual coverage of every combination, and a game's actual
combination needs review.

The references the engine keeps are og-tests-ui's
`references/vulkan/doc-components-<group>-<document>.png` (each document's
default fixture), `doc-screens-<screen>.png` and the `documents/` frames,
in the default style. Captures record effective style, theme, viewport,
DPR, text scale, locale, focus, selection, tick and animation history;
equal ticks do not prove equal state. References are recorded on the GPU
host (NVIDIA L4, Vulkan; docs/ci.md §The GPU host); other GPU backends are
compatibility checks under the cross-platform tolerance. A missing GPU is
unchecked, never passed.

Document previews supply game-owned sample textures and an icon atlas; empty
texture handles in a source fixture still mean no art in a game. Film overlays
are reviewed over a sample still. The preview ground paints below world UI,
not as an opaque screen panel over it. World document previews aim at the
authored subject: labels use a level camera 6 m behind their anchor pivot;
ground marks use a 35° downward view fitted to their world extent. Their
reference frame crops actual paint plus the subject post with 24 px padding
(36 at 150%). Edge-clamped fixtures keep their original camera and full frame.
Nameplate, SpeechBubble and Waypoint crop to their own paint plus that padding;
their ground-level subject post does not enlarge the frame into empty grid.
A faint metre grid, stronger every fifth line and fading by 30 m, and a 1.8 m
subject post are renderer-only context below the component, not findings.
World frames use the dark HUD ground `rgb(5 7 12)`. Missing art paints a neutral mark, never a slot path;
its slot identifier remains in `art_slots` and probe detail, not visible text.

### 6.3 Acceptance and art sign-off

Technical checks reject parse failures, missing resources, wrong intents,
missing required states and out-of-bounds references. `og ui check`
reports contrast, layout and cost measurements; `og ui literals` reports
suspect values; neither scores taste. Overlaps are measurements:
`og.ui.overlaps` (and the check's overlap finding) lists every absolute,
anchored or floating element over another's drawing and content across
sibling flow areas, including descendants overflowing a flex or grid cell.
It uses visible boxes and emitted text bounds, respects ancestor clips and
zero opacity, and applies no minimum-area threshold or result-count cap.
Stacks and engine-declared overlays (a badge on its item) are intentional.
A floating screen used as the base page is checked within itself. Its
background-only fills covering the safe surface beneath all content are
intentional underlays if they contain no content or interaction. A
negative-z fill may instead cover its parent's content box, as a local
shade; padding need not be covered.
Other page content, including nonempty generated text, is checked; an
empty generated text box draws nothing and does not overlap.
A document can mark intentional overlap with
`data-og-overlap="full/probe/path another/full/path"`; a path covers that
element's descendants, declarations cover the declaring subtree, and `*`
permits all overlaps for that subtree. This attribute only changes the
diagnostic, never layout or paint. Other intersections remain facts for
the author to judge. Warnings never become automatic aesthetic
failures. Thresholds and the CPU cost bars live in the screens guide (§5)
and the art-review guide, not here; `crates/og-ui/tests/perf.rs` and
`scripts/perf.sh` measure the cost.

The author reviews the core sheet, both group sample pages, phone and
large text, empty and overflow states, and bright and dark HUD
backgrounds; motion review includes interruption and cancellation.
Commercial page references have comparable information density, not a
logo or promotional image. Review compares hierarchy, numeric readability,
focus, selection, reachability, feedback and consistency across
components.

The designated art reviewer, a separate stream the coordinator assigns,
signs off the actual frames and clips before a new component group or
pack lands through `scripts/land.sh`. A rejection names the problem and
screenshot region; the author fixes and recaptures. A palette swap, a
decorative background or a desktop-only success is not acceptance;
core-default captures of a group prove behaviour, not final art.

Release requires every group signed off under every built pack; quality
failures delay release rather than lower scope or waive sign-off.

### 6.4 UI debugger

A development build's inspector (F1) has a UI view, its third tab. Pick
outlines the element under the pointer and picks it; the panel shows its
path and tag; its box model (margin, border, padding in CSS order, content
and border boxes with positions); the layout boundaries it sets (`scroll`,
`container`, `layer`, `fixed`: a fixed width and height); and its
computed styles: each winning document declaration with its value and
`file:line`, the pack rules dressing its part with their source, and its
bindings with expression and current value. The frame draws the picked
element's margin, border, padding and content boxes, as Chrome does;
Layout bounds outlines every drawn element; the filter field keeps
styles, rules and bindings whose name holds it. The panel is a built-in
document (`inspector`, mounted in the inspector's own UI and given its
state as values each frame), dressed by tokens alone: the boxes take
`--warn`, `--gold`, `--success` and `--info`, the bounds `--danger`. `inspect:ui[:<path>]` opens the view from a script and
`dump_inspector` carries the pick as `ui`.

The view reads the last finished frame through the inspection API
(`Ui::elements`, `Ui::element_bounds`, `Ui::debug_pick(x, y)`,
`Ui::debug_element(path)`); `og_ui::debug_overlay` draws a debugger's
boxes into its own UI. A pick walks the drawn elements once; reading the
picked element is one lookup by render key, its cascade the one its last
resolve kept. The game's UI is read only while the view is shown.
References: `ui-debug-1280`, `ui-debug-390`.

`dump_ui:motion[:path]` writes the motion alone, as `dump_ui:layers` the
layer stack: each element in a visual state or animating, with its
states, what moves, what plays and whether it is leaving.
`dump_ui`'s `moving` says whether the TSX tree is still on its way to
rest: a transition or layout move in flight, an exit, or keyframes that
end (an entrance's pop) still playing. A loop that never ends (a
spinner, a shimmer) leaves it false, so a capture that waits for
`moving: false` judges an entered screen, not its entrance.

### 6.5 UI gallery

`bun packages/engine/src/node/gallery.ts [--out DIR] [--packs a,b] [--schemes
dark,light] [--only TEXT] [--size WxH] [--og <og>]` renders, in a checkout, every fixture of
every built-in component (`src/ui/components`, `controls`, `screens`)
under every pack: it reads each `.tsx`'s component and `fixtures`, writes
one project (`target/ui-gallery/project`) that mounts one fixture per page
on a two-tone stage (a bright sky over a dark floor, captioned with the
component and fixture), cooks it once, and runs it headless once per pack and scheme,
switching both with `og.ui.configure` and capturing every page 12 ticks
after it mounts; a page whose `dump_ui` still says `moving` (an entrance
still popping) is captured again a second after it mounts, so every
frame is judged at rest. The
frames land in `<out>/<pack>/` (default `target/ui-gallery/out`; the light
scheme's in `<out>/<pack>-light/`) with `index.html`, a row per fixture and
a column per pack and scheme. `--schemes` is `dark` alone unless given. `--only` keeps the
fixtures whose `<dir>/<file> · <fixture>` contains the text. A component
without fixtures has no page. The run draws on the GPU og finds; a frame
is judged for what it shows, as any other.

## 7. Maintenance boundaries

The integration owner owns shared schema, labels, looks, creation,
packaging, this contract and the guides. Component group and pack authors
own their group or package directories and their components' fixtures. Shared API changes go through the integration owner before
consumers adopt them.

A group's documents, string tables and README live in
`packages/engine/src/ui/components/<group>/`, its behaviour tests in
`packages/engine/tests/<group>.test.ts`, and each document's cooked
frames in og-tests-ui's
`references/vulkan/doc-components-<group>-<document>.png`. A pack's files
and README live in `templates/styles/<pack>/`; pack test material lives
outside the runtime pack roots, under `templates/styles/tests/`, which
the cook skips. Core fixtures live with core tests. Reference changes name
the affected frames and their reason; recording is not art approval.

API declarations, the component catalogue and the wiki are generated from
IDL, parser metadata and `crates/og-api/guides/`, and are never
hand-edited; look catalogue changes follow its generator. Core default
tokens come from `packages/design` and do not bind game-authored styles.

Missing core capabilities are tracked in [the roadmap](roadmap.md).
Variable-height virtualization and localized number and date formatting
are core prerequisites; no component works around a missing facility
privately. Per-frame UI cost is measured work; this document claims no
unmeasured budget has passed. Author knowledge belongs in guides, not
shipped skills.

og-ui is crates along its mechanisms. `og-ui-model` holds geometry and codec
primitives, `og-rive` Rive artboards and their player, `og-reactive`
bindings and the reactive session. `og-ui` holds the toolkit and the
documents in one crate: the stages and their state, input, layout, the CSS
cascade (the style stage, which styles immediate widgets too), motion,
paint and draw, the document model, and the documents themselves (the
Svelte parse and writeback, build and mount, the SVG element build,
layers, style packs and the probe). The stages call the documents directly:
the frame's end builds the layers no host built, and settlement restyles
the mounted elements whose query containers changed. The documents' methods
on the stage types are inherent, written beside the documents, and reach
the stages' state only through each stage's accessors. The documents'
stage (`ui::Docs`) is four groups, each with one writer: the library
(`Library`: the documents defined, what defining them told about every
build, and the assets they name, written by the `define_*` calls), the
layer stack (`LayerStack`), the query containers (`Containers`: the boxes
layout settles and the scroll-state flags sampled as a tick starts), and
the build's state (`DocState`: build cursor, the tick's outputs, mounts'
memory, the reactive session). It is the same for a UI that built no
document; what a snapshot carries of it is the layer stack and `DocMemory`.

The [gate rules](ci.md) own test lanes, locks and toolchain policy;
`scripts/land.sh` owns landing on main. Release and platform installation
follow [release rules](release.md), with separate release authorization.

