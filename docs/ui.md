# Game UI contract

This document defines the boundaries and acceptance rules for core UI,
genre kits, and style packs. API signatures belong in the generated API;
author workflows belong in the guides. The catalogue below distinguishes
available functionality from packages that are not yet built.

## 1. Scope and ownership

| Owner | Responsibility | Boundary |
| --- | --- | --- |
| Core | Elements, layout, drawing, controls, documents, layers, focus, gestures, motion, text, accessibility, semantic tokens, and public composition helpers | One control set and one API on native and web |
| Genre kit | Domain-specific presentation composed from core controls, editable screens, typed inputs, and returned intents | No gameplay rules, private engine imports, or second widget toolkit |
| Style pack | Palette, typography, spacing, shape, dressings, art, icons, motion, sound, and haptic definitions | No gameplay code or domain state |
| Game | Content, rules, world state, domain validation, and applying UI intents | Owns its screens and chosen style assets |
| Platform | Accounts, money, publishing, threads, agent orchestration, and engine version selection | Consumes released archives and the public host ABI, never engine internals |

The renderer owns the UI pass. `og-ui` owns layout, documents, interaction,
and style interpretation. `packages/engine/src/ui/` composes the public
controls. The public TypeScript API comes from `crates/og-api/api/` for
both hosts. Game authors never need Rust.

Screen UI, world anchors, world panels, and embedded camera views use the
same element model. Backend limits are capability queries, not separate
kit implementations. Web rendering stays off the main thread.

UI presentation state is tick-driven and checkpointed beside the world,
outside `world_hash`. Focus, selection, scroll, caret, and animation state
must retain their meaning after snapshot and restore. Rendering cannot
change simulation results. See [engine design](design.md) for determinism
and the host boundary.

## 2. Composition contract

Each layer has one authoring form:

- Look is the CSS dialect plus semantic tokens.
- Structure is Svelte 5 templates in `.svelte` files without a script
  block. TypeScript does not build UI trees. The `.ogt type screen` dialect
  is removed without a reader.
- Data uses the `svelte/store` model. Games prepare named stores
  and pass them by name when mounting a document; its bindings and keyed
  loops read those sources.

A document uses core UI elements as tags, `{#if}`, keyed
`{#each ... (key)}`, `onclick={...}`, `bind:`, `class:`, and a scoped
`<style>` with the CSS dialect. It reads named stores as `$name`.
The Svelte parser, cook path, and generic screens are implemented; the old
screen reader is deleted. The [screens guide](../crates/og-api/guides/screens.md)
and generated [component catalogue](../crates/og-api/guides/components.md)
describe the supported syntax. Native retained mounting is implemented;
TypeScript mount wiring and removal of TypeScript builders remain pending
(§2.1). The structure-only authoring rule is the
target contract, not a claim that those builders have already been removed.
The document contract has these requirements:

- A `.svelte` file has compile-time `<meta>` declarations, one root element,
  and document-scoped `<style>`. There is no `<template>` wrapper or script.
  Metadata declares screen or component identity, layer, focus, typed
  props, actions, and preview fixtures. IDs are unique within the document.
- Text uses `{expr}` and attributes use `attr={expr}`. Mounted stores use
  `$name`; props and each-row aliases use plain names. Expressions are
  read-only. `onclick={action}` dispatches a named mounted action, not an
  arbitrary call body. `bind:value` writes a writable store or keyed row
  field in the current tick batch; `value` alone is read-only.
- Declared input actions also dispatch named markup events independently
  of clicks. `<meta name="action:confirm" content="Start">` enables
  `onconfirm={start}`; `onclick` is not a substitute for that binding.
- Each loop requires stable, nonempty string keys and supports an empty
  branch. Missing, index-based, empty, and duplicate keys are errors.
  `selected` and `mark` identify collection keys, never row indices.
- Capitalized tags resolve document components. Content uses
  `{#snippet name()}…{/snippet}` and `{@render name?.()}`. Mount content
  references name a document and snippet, never a TypeScript callback.
  `<slot>` is rejected with a replacement diagnostic.
- Visible text uses `t(key, { named: value })` with ICU parameters or
  static `i18n="key"` with fallback content. Unlocalized visible literals
  produce cook warnings and publish errors. Object expressions are allowed
  only for translation parameters and classes. Companion English messages
  use cooked string tables.
- Script, JSX, runes, `{@html}`, `{@const}`, `use:`, `transition:`, and
  `on:click` are unsupported. Motion uses CSS, not template directives.
  Parse errors identify file, line, column, and a fix.

Structure and snippet content remain documents, never TypeScript trees.

Core elements use HTML's tag and attribute names wherever HTML names the
role; a pre-HTML tag is refused with a diagnostic naming its HTML form.
The internal kind names (in the catalogue headings, the IDL descs and the
probe) are unchanged. The mapping:

| Document | Core kind | Notes |
|---|---|---|
| `<input>` (`type` text, password, number, email, url, tel, search) | field | `placeholder`, `maxlength`, `minlength`, `readonly`, `required`, `pattern`, `min`, `max`, `step` (`any`), `aria-invalid`; CSS `appearance: none` drops its frame |
| `<textarea rows>` | field (multi-line) | the same constraints; `rows` lines shown |
| `<input type="range">` | slider | `min`, `max`, `step`, `disabled`, `list` naming a `<datalist>` (below); CSS `-og-fill-origin: <value>` |
| `<datalist id>` of `<option value [label]>` | — | draws nothing: a range's `list` puts a tick at each option and its `label` (or text other than the value) under the track |
| `<input type="checkbox" switch>` | toggle | `checked`, `disabled`; CSS `-og-control-size` |
| `<input type="checkbox">` | toggle | `checked`; drawn as a switch (og-ui has one checkbox control) |
| `<label><input type="checkbox"> Caption</label>` | toggle | the label's text is the caption; the two are one node. The caption sits where it is written (after the switch here, before it in `<label>Caption <input …></label>`); `flex-direction: row-reverse` on the label swaps it. A `<small>` inside is its description line |
| `<label for="id">` | — | joins the control `id` names: a checkbox takes its text as its caption (one node, as the wrapping form); an input, textarea, select, range, meter or progress takes it as its accessible name and the label stays a text beside it |
| `<hr>` | divider | |
| `<img>` | image | `src`, `alt`; CSS `object-view-box: inset(<t>% <r>% <b>% <l>%)` (percentages of the texture) or `-og-uv: u0 v0 u1 v1` shows a part of it |
| `<p>`, `<span>`, `<small>`, `<text>` | label | `<text>` is the core name; inline `<b>`/`<strong>`, `<i>`/`<em>`, `<u>`, `<s>`/`<del>`, `<br>`, a plain `<span>` and `<a href>` style runs of it |
| `<h1>` … `<h6>` | label (heading) | role `heading` with `aria-level` 1–6; UA type steps `xl3`, `xl`, `lg`, `body`, `sm`, `xs` |
| `<a href>` | label (a link) | a click sends a `link` event named by its href through the document's events, as `onclick` sends `emit`; accent ink and an underline |
| `<div>`, `<section>`, `<header>`, `<footer>`, `<nav>`, `<main>`, `<aside>`, `<article>` | box | layout from CSS `display` (below); landmarks take their roles: `navigation`, `main`, `banner`, `contentinfo`, `complementary`, `article`, and `region` for a named `<section>` |
| `<ul>`, `<ol>`, `<li>` | box (list, listitem) | not virtualised (that is `<list>`); markers from `list-style` (below); `<ol start reversed>`, `<li value>` |
| `<button>`, `<progress>` | button, progress | already HTML; a `<button>` holding elements (`<icon/>`, text, `<badge>`) lays them out as a centred row inside its frame, states and press, and its text and icons without a colour of their own take its ink |
| `<select>` with `<option>` and `<optgroup>` | select | `value`, `placeholder`, `disabled`, `aria-invalid`; options take `value`, `label`, `i18n`, `disabled`, `selected`; `select::picker(select) { max-height }` bounds the open list |
| `<dialog>` | dialog | `open`, `modal`, `closedby`, `aria-label`; CSS `appearance: none` drops its frame |
| `<details>` with a first `<summary>` | details | `open`, `disabled` |
| `<meter>` | meter | `value`, `min`, `max`, `low`, `high`, `optimum` |

`<field>`, `<slider>`, `<toggle>`, `<divider>`, `<image>` and a
`<label>` that neither wraps a checkbox nor has `for` are refused. CSS type selectors match the written tag (`input`,
`textarea`), so `input[type=range]` and `textarea` select as in HTML.
Elements HTML has no name for (`column`, `row`, `stack`, `grid`, `panel`,
`scroll`, the virtualised `list` and `cells`, `art`, `icon`, `tabs`,
`badge`, `spacer`) keep their names.

#### HTML blocks and lists

A box (`<div>` and the other block tags) lays out by its CSS `display`:
`block`, the default, stacks its children top to bottom as a `<column>`
does; `flex` lays them out as a `<row>` (`flex-direction: column` turns
it); `grid` takes `grid-template-columns` and the rest as a `<grid>`;
`none` hides it. Other elements' layout is their tag, so `display` on
them takes only `none`. Text between a box's elements is an anonymous
text element, as an HTML anonymous block box is; `i18n` on a box
translates its one run of text.

HTML's user-agent rules come first in every document's cascade: `ul, ol`
indent 40px, `<ul>` markers are `disc` (then `circle`, then `square` when
nested), `<ol>` markers `decimal`, and headings take the type steps in the
table above. `list-style-type` (`none`, `disc`, `circle`, `square`,
`decimal`, `lower-alpha`, `upper-alpha`, `lower-roman`, `upper-roman`),
`list-style-position` (`outside` by default, `inside`) and the
`list-style` shorthand inherit, as in CSS; `list-style: none` removes the
markers. An `outside` marker hangs half an em before the item's start
edge and costs it no room; `inside` puts it first in the item's row. A
disc, circle or square is drawn as a shape on the first line (no font has
to carry it); a number is text with CSS's suffix (`3.`, `iv.`), decimal
outside the alphabetic and roman ranges.

HTML's block margins come with them: `<p>`, `<ul>` and `<ol>` take 1em
above and below (a list inside a list none), and `<h1>` to `<h6>` take
0.67em, 0.83em, 1em, 1.33em, 1.67em and 2.33em of their own type step;
`margin: 0` resets them. `margin`, its longhands (`margin-top`,
`-right`, `-bottom`, `-left`) and the logical `margin-block`,
`margin-inline` and their `-start`/`-end` take px, em (of the element's
type step; an element with none uses the body step), `rem`, viewport
units, a percentage, `auto` or a `calc()` of those. As in CSS 2 §8.3 a
percentage, top and bottom too, reads the parent's content width, and a
negative margin pulls the box out. Horizontal margins never collapse.
In a block's flow an `auto` left and right margin share what the width
leaves (`margin: 0 auto` centres a box with a width), one `auto` takes all
of it, and the top and bottom ones are 0. In a flex line `auto` margins
take the free space before `justify-content` does and, across the line,
align the item in place of `align-items` (CSS Flexbox §8.1); in a grid
area they align the item and its other margins inset it (CSS Grid
§11.2). A stack places its children without margins. In a `display: block` box the
vertical margins collapse as CSS 2 §8.3.1 says: two adjacent siblings'
margins become the larger positive one plus the most negative one, and a
block's first child's top margin (last child's bottom one) collapses
through the block's own edge when no padding separates them and, at the
bottom, the block's height is auto; scrolling, `display: flex` and
`display: grid` boxes keep their children's margins inside. A flex
column adds its children's margins along it, a flex row adds them across
it, a grid insets each area by them, and a stack places its children
without them. An empty block in a block's flow (no padding, an auto
height, and no children or only such empty blocks), and an empty `<p>`,
collapses through itself: it is 0 tall, and its top and bottom margins
join its neighbours' into one set, as CSS 2 §8.3.1 says.

Lengths in `width`, `height` and the margins follow CSS Values 4: a
`calc()` over `%`, px and em (em in margins), with `rem` and `vw`, `vh`,
`vmin` and `vmax` turned into px when the element resolves, reduces to a
linear sum (`calc((100% - 2 * 8px) / 2)` is `calc(50% - 8px)`), and layout
resolves its percentage against the basis it already has for one: the
parent's content width (or height), a grid area, the screen. `min()`,
`max()` and `clamp()` over such sums, up to three, are evaluated at
layout: `width: min(100%, 600px)`. A `var()` term (`calc(var(--w) * 100%
- 20px)`) is substituted first, so a store bound with `style:--w` moves
only its element; an element whose values read a viewport unit resolves
again when the viewport resizes. Times, angles and plain numbers still
take one unit per function; `calc(10px + 20ms)` is an error. Container
units (`cqw`…) are refused for now.

An item's number is its place among its `<li>` siblings, counted from the
list's `start` (1, or the item count when `reversed`), or from the
nearest earlier `<li value>`. It comes from the sibling index the
structural selectors keep for `:nth-child(n of li)`, so a keyed `{#each}`
reorder or insert renumbers only the items whose place changed, in
O(log n) per edit, with no pass of its own.

#### Borders, shadows, outlines and overflow

These follow CSS Backgrounds and Borders Level 3 and CSS Overflow Level 3
on any element. `border` and `outline` take the shorthand (`2px dashed
var(--accent)`) and the longhands: `border-{top,right,bottom,left}`,
`border-width`/`-style`/`-color` with one to four sides, and
`outline-width`/`-style`/`-color`/`-offset`. Styles are `none`, `hidden`,
`solid`, `dashed`, `dotted`, `double`, `groove`, `ridge`, `inset` and
`outset` (an outline takes `auto` in place of `hidden`); `currentColor` is
the element's `color`. A border's width pads the box as `box-sizing:
border-box` does; an outline takes no room and is drawn outside the border
edge, grown by its offset, so `:focus-visible { outline: 2px solid
var(--focus) }` replaces the core focus ring for that element. A uniform
border follows `border-radius`, which also replaces a dressed control's
corner (a `<button>`'s plate); sides that differ are drawn as strips, and
`inset`/`outset`/`groove`/`ridge` darken their shaded sides.
`box-shadow` takes a comma list of `[inset] x y [blur [spread]] [color]`,
the first listed on top, outer shadows under the background and inset ones
over it.

CSS paint. `background` (or `background-image`) takes
`linear-gradient()`, `radial-gradient()` and `conic-gradient()` and
their `repeating-` forms as CSS Images 3 writes them (angles with 0 up
and clockwise, `to <side or corner>`, radial shapes and extents, `from
<angle> at <position>`, stops with one or two positions, at most 8),
layered over the background colour, the first image on top; they mix in
OKLab and are resolved against the placed box, so the line of a
`45deg` gradient reaches the corners of a wide box. `mask-image: <gradient>`
fades the element and its subtree by the gradient's alpha. `clip-path`
takes `inset(… [round r])`, `circle()`, `ellipse()`, `polygon([rule,]
…)` and `path([rule,] "…")` and cuts the element, its background and its
children by the shape's analytic coverage (the path clip of inline SVG),
with either fill rule. `filter` takes `blur()` (the element and its
subtree, a layer blur) and `hue-rotate()`, `saturate()`, `brightness()`
and `grayscale()` (one colour-adjust layer). `backdrop-filter: blur()
[saturate()] [brightness()]` is the glass backdrop. `mix-blend-mode`
takes normal, multiply, screen, plus-lighter, overlay, soft-light and
color-dodge. `text-shadow` takes a comma list of `x y [blur] [color]`.
`-og-material: <name>(<params>…)` composites the element and its subtree
through one of og-render's built-in layer materials (dissolve, shine,
hologram, glitch, ripple, pixelate, color-adjust, outline, blur), each
group of up to four numbers or a colour in the order the material lists
them; it is the engine prefix because no CSS standard has shader
materials. All of it is paint: a change repaints that element, and an
idle retained element builds nothing.

`overflow`, `overflow-x` and `overflow-y` take `visible`, `hidden`,
`clip`, `scroll` and `auto`; as in CSS, `visible` beside a non-visible axis
computes to `auto`. `auto` and `scroll` make the box scroll on that axis,
`hidden` and `clip` clip its children to the padding box.
`scrollbar-width: none` hides the bars of a scrolling box (`auto` and
`thin` show them).
`scroll-snap-type: x | y | both [mandatory | proximity]` on a scrolling
box and `scroll-snap-align: start | center | end` (block, then inline) on
the areas inside it settle a released drag, a fling or a stopped wheel on
the snap position nearest where it would stop, as CSS Scroll Snap 1 does;
`proximity` snaps only within a third of the view. The fling is steered
onto the target and decays onto it exactly, and the areas are read from
last frame's boxes only while the box settles. A modal `<dialog>` with
`-og-sheet-detents: 40% 90%` is a bottom sheet: docked to the bottom at
full width, as tall as its tallest detent, opened at its first, dragged
onto its detents and swiped down to close, which writes `open` back. The
detents are the engine prefix because no CSS standard has them.

A `display: block` box with an auto width fills its block parent's
content width, as in CSS block flow, so a `<div>` inside a `<section>` is
as wide as the section.

Kits use `@origingame/engine`, `@origingame/engine/ui-extensions`, and the
public helpers in `@origingame/engine/ui-parts/hud` and
`@origingame/engine/ui-parts/screen`. Private source-path imports, unsafe
casts that evade the API, and game-side Rust are not kit extension points.

`registerUiExtension` returns a shallow-frozen typed namespace. Games
explicitly import its entry. Registration does not mutate `og.ui`, install
a host binding, or execute discovered manifests. Hot reload can return a
fresh namespace; callers retain the namespace they imported.

Every kit declares its input and output types, semantic parts, state
combinations, documents, and interactions before implementation. Types
describe caller data and returned intents. An intent names the affected
item or command by stable ID, not by a transient row index. The game
decides whether that intent is legal and applies its effects.

UI state lives in the scope's sources and core interaction state (§2.1),
not module globals.
Two worlds cannot share selection accidentally. Reordering or removing
data cannot silently select a different object. Snapshots include the UI
state needed to resume an interaction.

Core owns buttons, fields, toggles, tabs, sliders, menus, selection,
drag-and-drop mechanisms, scrolling, focus, and keyboard or gamepad
navigation. Kits compose these controls rather than draw substitutes.
Generic meters, counters, timers, slots, lists, grids, tables, charts,
tooltips, cards, rewards, confirmations, and node graphs stay in core.
Domain names do not justify duplicate controls.

### 2.1 Retained authoring model

The engine implements the `svelte/store` model: `writable`, `readable`,
`derived`, `get`, and a keyed collection store. Stores are named within
a scope so they can be checkpointed. Rust owns the graph on both hosts;
this is an engine implementation, not a browser Svelte runtime.
Data authoring uses function calls; structure remains in Svelte templates,
not TypeScript trees or JSX. There are no async primitives, Promises,
resources, Suspense, or async transitions. CSS motion remains supported.

Gameplay initialization belongs in `og.game({ start(world) { ... } })`.
UI registration belongs in its separate `ui(world)` callback, which runs
once after `start` and again after hot-reload restore, never per tick.
Registration creates named scopes and stores through `og.ui.scope`,
`writable`, and `derived`. Stable IDs preserve values across hot reload;
re-registered callbacks replace their bodies, and missing IDs are disposed.
The `ui` callback may read the world, but gameplay writes fail registration.
On fresh-process restore, `start` runs, then `ui`, then the engine restores
named presentation values. Hot-reload registration does not rerun `start`.

The landed store entry is `og.ui.scope(world, id): UiScope`. IDs are
explicit, stable, and nonempty. Its signatures are:

```ts
writable<T>(id: string, initial: T): UiWritable<T>;
readable<T>(id: string, initial: T, start?: UiStart<T>): UiReadable<T>;
derived<S extends UiStores, T>(id: string, stores: S, compute: UiDerive<S, T>, initial?: T): UiReadable<T>;
get<T>(store: UiReadable<T>): T;
collection<T>(id: string, items: readonly T[], desc?: UiCollectionDesc): UiCollection<T>;
takeErrors(): readonly string[];
dispose(): Result<void>;
```

Values must be serializable. `subscribe(run, invalidate?)` returns an
unsubscribe function. A readable's synchronous `start(set, update)` runs
for its first subscriber; the returned cleanup runs when its last leaves.
`get` subscribes and immediately unsubscribes. Derived inputs are an
explicit store or readonly tuple of stores, and `compute` is synchronous
and pure. A failed computation preserves the last good value or initial
fallback; without either it reports an error and yields `undefined`.

Writable `set(value)` and `update(updater)` return `Result<void>`.
Collections use `desc.key`, defaulting to the `id` field. They expose
`keys(): readonly string[]` and `row(key): Result<T>`;
`insert(row, before?)`, `remove(key)`, `move(key, before?)`, and
`patch(key, fields)` return `Result<void>`. Duplicate keys and key-changing
patches are refused. Scope errors also remain available through
`takeErrors`, including in publish builds.

For example, registration creates presentation data without a UI tree:

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

If you know Svelte, stores keep its subscription, first/last subscriber,
and synchronous derivation semantics. Here stores also need stable IDs
and a world-owned scope, and write operations return error values. No
Svelte runtime or compiler runs in the game.

Presentation checkpoints live outside `world_hash`. Saves require the
`og-save:3` envelope; older envelopes are refused with both format versions,
not converted. UI feedback uses presentation-only sound: it creates no
gameplay entity or event and is not replayed when a save restores.
`og.audio.playFeedback` exposes that boundary for one-shot sounds.
Gameplay effects cross `og.input.intent(world, event, payload)`, which
records the reflected event name, layout hash, and payload for the next
tick. `takeIntents` records accepted intents and `replayIntent` validates
their schema and tick. `og.PauseIntent` uses this same boundary to apply
simulation time scale; a UI control does not write gameplay state directly.

The native retained host implements dependency-tracked binding scheduling,
document mounts, and keyed `{#each}` connected to the virtualizer. These
behaviors are not yet exposed by TypeScript mount methods on `UiScope`:

- Changed sources reevaluate subscribed bindings. Each evaluation replaces
  its dependencies with the values it read, including after a branch changes.
- Mounts preserve document-node and collection-key identities across row
  insertion, removal, and reordering. Focus stays with the key rather than
  a row index. Virtualized rows outside the materialized range defer styling
  until they enter that range, then use the current style.
- Replacing a mounted document rebuilds its nodes without resetting named
  stores. Unmounting removes its nodes and binding subscriptions. Restore
  reconstructs mounted instances from saved registrations and source values.
- An action inside keyed rows carries the innermost collection's raw key,
  including after restore. Keys are not escaped render paths; actions outside
  keyed rows carry no row key.

A `type screen` document mounted with `scope.mount(id, document, inputs)`
is a layer of the stack for as long as it is mounted. Its
`<meta name="screen" data-layer … data-pause … data-back … data-focus …>`
decides where it sits and what it holds, exactly as for a pushed screen:
input routing, inert layers under a menu, pause, focus memory, and the
`activate` and `deactivate` events. Its bindings read the mount's stores,
not a viewmodel. Back (with `data-back="pop"`) or a `pop` trigger unmounts
it. Its layer's id is the mount's `scope:id`, and its root's path is
`scope:id/<root id>`, as a pushed screen's is `<screen>/<root id>`.
`og.ui.layers` names the mount (`UiLayer.mount`). Each layer adds one
wrapper node per frame; the retained tree under it is written only where
it changed.

Mounted documents use components and snippets as Svelte 5 does. A
capitalised tag instantiates the component document of that name. Its
attributes are props: an expression of the instance's user, read in the
user's context, so `<Card title={item.name}>` follows that row's field and
nothing else re-evaluates. `bind:amount={item.qty}` writes back through the
prop. A `{#snippet body()}` inside the instance is content passed to it.
The instance's other children are its implicit `children` snippet.
`{@render body?.()}` in the component renders the passed content where it
was written, with the user's rows and stores. Otherwise it renders the
document's own snippet of that name, and with `?.()` nothing. A component
reads the mount's stores (`$name`) as its user does, and `{#each items as
…}` iterates a collection its user passed as a prop (`<List
items={$inventory}>`). Content can also be passed at mount, by reference:
`scope.mount("pause", "pause", inputs, { side: "game#pauseSide" })`
renders that document's snippet, or with a bare document name its root,
where the screen writes `{@render side?.()}`. Rendered content's paths sit
under the snippet's name (`…/Card#a/body/Desc`), and a component
instance's root takes the instance's id.

A screen opts into CSS View Transitions with `@view-transition
{ navigation: auto; }`. Its layer joining the stack and leaving it are its
navigations, and the layer is the `root` snapshot. By default the layer
cross-fades, as the spec's UA stylesheet does (`-ua-view-transition-fade-in`
and `-fade-out`, 0.25s). `::view-transition-new(root)` animates the
incoming view (the joining layer) and `::view-transition-old(root)` the
outgoing one (the leaving layer, which stays drawn and inert until its
animations end). Both take `animation` with the document's `@keyframes`.
`::view-transition-group(root)` and `::view-transition-image-pair(root)`
give both their inherited timing (`animation-duration` and the other
longhands except the name). `types` are labels, as in the spec: a rule
qualified by `:active-view-transition-type(slide)` applies when
`@view-transition` lists `slide`, and outranks an unqualified one.

The engine's full-frame covers are the only `-og-` part, because CSS has
no keyframes that cover the whole frame: `-og-iris`, `-og-dissolve`,
`-og-wipe` and `-og-slide`, the last two with `-from-right`, `-from-top`
or `-from-bottom`. On `::view-transition-old(root)` a cover closes over the
outgoing view, and the incoming view shows once it has closed. On
`::view-transition-new(root)` it opens on the incoming view.
`::view-transition { background-color }` is the covers' colour. Reduced
motion ends finite animations at once and makes covers fade. A component
cannot declare a view transition. Shared-element morphs
(`view-transition-name`, `::view-transition-old(card)`) are refused for now
and are a follow-up.

Inline SVG draws data-bound vector graphics in a document: `<svg viewBox>`
with `<g>`, `<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`,
`<polyline>`, `<polygon>`, and `<defs>`, `<linearGradient>`,
`<radialGradient>` and `<stop>`. Every attribute can bind
(`d={$series.path}`, `stroke-dashoffset={1 - $ready}`,
`{#each $bars as b (b.id)}<rect height={b.h}/>{/each}`), and SVG's
presentation properties (`fill`, `stroke`, `stroke-width`,
`stroke-dasharray`, `stroke-dashoffset`, `stroke-linecap`,
`stroke-linejoin`, `stroke-miterlimit`, `fill-rule` and the opacities) are
attributes or CSS (`#Ring circle { stroke: var(--accent); }`), inherited
down the tree as SVG inherits them. `transform`, `pathLength` (a ring with
`pathLength="1"` takes its progress as `stroke-dashoffset`), `url(#id)`
gradients with `gradientUnits`, `gradientTransform` and `href`, and
`currentColor` (the style's ink) work as in browsers. A paint also takes a
style colour, `var(--accent)`, which the cook's vectors cannot.

An `<svg>` is a box of its CSS `width` and `height` (default: its view
box's size); the view box fits it as `xMidYMid meet`, so stroke widths and
dashes scale with it, as in browsers. Each shape is one element: its fill
is an analytic path in og-render's UI pass and its subpaths are its stroked
outlines, so a shape whose bound attribute changes is drawn again and
nothing else is, and a box that only resizes places the same draws again.
The parser is the cook's (`og_ui::svg`, shared with `type vector`).

Definitions are referenced by `#id`, as in browsers:
- `<use href="#id" x y width height>` draws a copy of a `<symbol>` (its
  `viewBox` fitted into the width and height by `preserveAspectRatio`), a
  group or a shape. The copy is made when the document is parsed, so it
  inherits the use's presentation attributes and its own attributes bind
  as the original's do. A bound `x` on a use costs the use and its copy.
- `clip-path="url(#id)"` on a group or a shape cuts it to a `<clipPath>`'s
  shapes. `mask="url(#id)"` cuts it to a `<mask>`'s white shapes, or fades
  it through a mask of one gradient-filled shape (luminance times alpha, as
  SVG masks). One rect, circle or ellipse under an axis-aligned transform
  clips as a box; any other clip, of paths, polygons or turned shapes,
  cuts by the paths' own analytic coverage (the coverage a path fill
  draws with, read from the same band tables): its shapes as one path,
  by their `clip-rule` (a mask's by `fill-rule`), nonzero or evenodd. The
  shapes of one clip take one rule; two evenodd shapes that overlap cut a
  hole where they do.
- `fill="url(#id)"` naming a `<pattern>` tiles its shapes over a rect,
  circle or ellipse, with `patternUnits`, `patternTransform` and
  `viewBox`, cut by the shape (a path by its own outline). The tiles of
  each shape are one path, at most 4096 tiles.
- `<title>` names its parent for assistive technology, as `aria-label`
  does, and can bind (`<title>{$label}</title>`). A named `<svg>` is an
  image. `<desc>` is dropped.

`<text x y dx dy font-size font-weight font-style text-anchor
dominant-baseline fill>` is one line shaped by og-ui's text, as any label
is: its content binds (`<text>{$hp}</text>`), and `<tspan>` runs inside it
take `font-weight`, `font-style`, `text-decoration` and `fill`. It is laid
out at its view-box size (`font-size` in user units as CSS px), its anchor
(`text-anchor` start, middle or end of the line; `dominant-baseline` auto,
middle, central, hanging, mathematical, ideographic, text-before-edge or
text-after-edge) is placed on `(x, y)` fitted into the `<svg>`'s box, and
the line is scaled by the fit about it, glyphs from the font's distance
field. Hit tests and the accessibility tree read the scaled box. Right to
left the `<svg>` moves across the surface with the layout, but its text
stays where its view box shows it, as its shapes do. A group's text
attributes inherit, and its transform moves the point and scales the size
(a turn is not applied). A bound `x` costs that text; moving or resizing
the `<svg>` builds nothing.

`<image href x y width height preserveAspectRatio>` draws an atlas image
icon by name (`href="gem"`) or a render texture by handle
(`href={$target}`), filling its rect (contained by default, `slice`
covers, `none` stretches); it takes no turn or skew.

Definition attributes (gradients, clips, masks, patterns) are literal.
Filters, `<textPath>` and multi-line text are refused with the element
named; a dash array takes
one dash and one gap. Cost: re-tessellating one
dashed ring as a cooked vector takes 438 µs and 840 vertices; changing a
bound attribute on one of 64 inline rings costs 69–150 µs of the frame
(dev profile), and unchanged shapes cost nothing.

Drag and drop and pointer input use HTML's names. `draggable="true"` makes
an element follow a drag that starts on it and spring home on release.
Handlers name actions: `ondragstart` and `ondragend` on the dragged
element, `ondragover` and `ondrop` on the element it is over. An element
with either of those takes drops, filtered by `data-accept="gem coin"`
against the dragged element's `data-drag-type`. Their events carry
`{ dragged, target }`, the two elements' row keys, and `{ x, y }`, the
pointer in the handler's box: `{#each $bag as item (item.id)}<li
draggable="true" ondragstart={lift}>…{/each}<div ondrop={stash}>`.
`onpointerdown`, `onpointermove`, `onpointerup` and `onwheel` (with
`deltaX` and `deltaY`) fire on any element, as in HTML: a move goes to the
held element while a press is down, and an up goes to the element the press
went down on (pointer capture). Each event fires at the element the input is
at and bubbles up the mounted tree to the nearest element that handles it.
These events come from one pass over the held element, the one under the
pointer and a drop's target, so they cost nothing as the tree grows. An
element with a pointer handler can be under the pointer, so like any
hovered control it is written again each tick while hovered.

`<viewport>` shows a camera's render target in the layout, sized by CSS like
an image. It is the native scene viewport (`og.ui.sceneViewport`). `src={$target}`
names the target's colour texture. The camera is written as
`<model-viewer>` writes one: `camera-orbit="30deg 75deg 3m"` (yaw, pitch
from straight up, distance), `camera-target="0m 1m 0m"` and
`field-of-view="50deg"`, each bindable. Svelte's `bind:clientWidth` and
`bind:clientHeight` write its laid-out size when it changes, so the game
sizes the target by them times the pixel ratio. Its pointer events
(`onpointerdown`, `onpointermove`, `onwheel`) also carry `ray`, `{ origin,
direction }` into its scene through the pointer, for picking and orbiting.
`viewport` is not an HTML element. It is named for what it shows, and its
camera attributes reuse `<model-viewer>`'s names rather than `-og-` ones,
because that component is the web's established vocabulary for a 3D view.

On-screen controls are input sources of og-input's gameplay actions, so a
touch layout drives the same `og.input` actions a keyboard or pad does.
`<joystick action="move">` and `<dpad action="move">` are the native stick
and d-pad, sized by their CSS width (128 px and 132 px by default, never
under three touch targets). `<button data-og-touch="jump">` is a document
button, styled by CSS, that feeds `jump`. Each control's laid-out rect goes to
the action map as a zone after the frame, as the host hands it the play
surface. The map samples the touches that started on a zone in the tick, so
the actions stay deterministic and replay with the recorded input:
- A stick reads from its centre over its radius, y up. Past `data-dead-zone`
  (a ratio of the radius, default 0.15), the value rises from 0.
- A d-pad reads eight ways, from -1 to 1 on each axis.
- A button is held while its touch is held.

A control keeps the finger that started on it, whatever else is held, so
one thumb steers while another jumps. A finger that slides onto a control
takes nothing. A disabled control, or one hidden by `display: none`, is no
source. Hide them on a mouse with `@media (pointer: fine) { … display: none }`.
`pointer`, `any-pointer`, `hover` and `any-hover` follow the host's coarse
pointer. A held control is written again each tick while a touch holds it,
and every control is written again on the tick a touch goes down. Otherwise
they cost nothing, and an idle frame hands the map nothing new. `joystick`
and `dpad` are not HTML elements; no web standard names an on-screen game
control. `data-og-touch` takes the engine prefix because HTML leaves
custom data on a standard element to `data-*`, and an og-input action is
an engine concept.

`<anchor>` holds a box over a world point. It is the native world anchor
(`og.ui.openWorldAnchor`), its children laid out as a column:
`<anchor entity={u.entity} offset="0 2.1 0" occlusion="fade"
declutter="stack" clamp="edge">…</anchor>`. `entity` follows an entity's
pose as the host hands it in; `at="x y z"` names a fixed point instead.
`occlusion` (`none`, `fade`, `hide`), `declutter` (`none`, `shift`,
`stack`, `hide`), `clamp="edge"` (held on the screen's edge with an arrow,
a waypoint), `priority`, `max-distance` and `scale-at` are the native
desc's fields. It is placed every tick from the camera and the pose, but
its subtree is built again only when a binding it reads changes: a camera
that moves costs 0 bindings and 0 reconciled elements, and a renamed row
writes that one anchor. `anchor` is not an HTML element; no standard names a
box held over a 3D point.

`<panel world>` is the native world panel (`og.ui.openWorldPanel`): a box
laid out at its CSS size and drawn into the render target `src` names,
which the game shows on a plane: `<panel id="Terminal" world
entity={$console} size="3 2" rotation="0 30 0" src={$screen}>…</panel>`.
`size` is the quad in world units (width height, default `1 1`),
`rotation` turns it in degrees about x, y and z, and `at="x y z"` names a
fixed point instead of an entity. A ray from the pointer that meets the
quad reaches the panel's controls, so `onclick`, focus and Enter work in
it as on the screen; without `entity` or `at` it takes no rays. `world`
is a boolean attribute on HTML's `panel` kind because no web standard
draws a box into a texture.

A keyed `{#each}` of `<anchor mass="name">` becomes mass elements, GPU-drawn
by the thousand, instead of boxes: `{#each $hits as h (h.id)}<anchor
entity={h.entity} mass="damage" value={h.amount}></anchor>{/each}`. The
element defines the template `name`. It is a number, or with `max` a bar
filled to value/max sized by CSS `width` and `height`, tinted by `color`.
Its first CSS `animation` is the template's timeline: the `@keyframes`'
`translate`, `scale` and `opacity` sampled at each key, over the
animation's duration. A row with an animation spawns one instance when it
appears and never again (a damage number). A row without one stands while
its row does (an overhead bar), and the engine pushes it every tick at its
entity with nothing built. A patched value writes that row alone. `mass` is
an attribute on the engine's `anchor`; no standard names a GPU-instanced
element.

Particles and overlays are CSS, under the engine prefix because no web
standard has them. `-og-burst: 24 48px #ffd166` inside a `@keyframes`
step fires rays and sparks from the element when an iteration reaches that
offset. It sits beside `-og-sound`, as a keyframe event. `-og-emitter:
12 diamond 6px #ffd166 #fff` (rate a second, shape, size, colours) emits
particles while the element stands, and `none` stops it. `-og-meta:
vignette #a00` or `tint <colour>` draws a full-screen overlay with the
element that never takes input, its strength the element's `opacity`
(`#Hurt { -og-meta: vignette #a00; opacity: var(--hit); }`). The engine
registers emitters and overlays every tick while their element stands, so
a retained element keeps emitting with nothing built (0 bindings, 0
reconciled). Reduced motion silences bursts and emitters. All of it is
presentation, outside the world's hash.

`<ground>` lays a box flat in the world: `<ground entity={u.entity}
size="3 3" rotation="0 45 0"><svg viewBox="0 0 40 40">…</svg></ground>`.
It is laid out at its CSS size and drawn by the UI itself onto the
horizontal plane `size` world units across at the entity's (or `at`'s)
height, seen in perspective, under the screen's elements and the world's
mass elements: a selection ring, a range marker, a target decal. The
camera moving moves only its quad (`UiFrame.grounds`), read after the tick
and never fed back into the world: the frame's commands, bindings and
layout stay as they were (0 bindings, 0 reconciled, the layer root laid
out as every layer is). Its offscreen layer is sized from the size it is
seen at, rounded to a power of two and capped at 2048 px. It is inert: no
ray reaches it and its controls take no focus. While a corner is behind
the camera it is not drawn. The plane is flat at the entity's height;
draping it over terrain (a decal) is a follow-up. `ground` is not an HTML
element; no web standard lays a box onto a 3D plane. Hosts get the same
member as `WorldPanelDesc.projected`.

An anchor or world panel stands outside the screen's tree, in a world
layer of its own, but its layer is keyed by the element's own path: the
nodes in it have the keys and paths (`test:term/Terminal/Row/Run`) they
would have in place. A retained patch inside it writes the changed element
in place, as anywhere else (1 reconciled, idle 0/0), and either may be its
document's root.

The authoring contract below also includes requirements whose TypeScript
wiring remains pending:

Each screen has one fixed-tick scope with a stable name, named stores and
keyed collections. Documents mount once with named mount IDs and inputs.
Template `$name` reads and keyed `{#each}` consume those inputs. Bindings
are pure, track only the values they read, and replace dependencies when a
branch changes. Lists bind to keyed collections, not copied viewmodels.
Content passed at mount consists of serializable document and snippet
references, never tree-building callbacks.

Named TypeScript callbacks are allowed only for custom drawing and the few
components a document cannot express, with explicit dependency handles.
They are not an alternative tree-building API. Unchanged dependencies skip
the callback.
There is no implicit per-tick callback interaction. Core controls process
input and return intents; ambient values such as time and viewport are
explicit sources when a callback needs them.

Identity is the mount, document node, and collection key;
focus and selection belong to keys,
not row indices or recycled render nodes. Source values, collection order,
mounts, and UI state are checkpointed outside `world_hash`. Restore rebuilds
derived caches without replaying mount events or advancing time.
Lazy diagnostics, including inspection and accessibility with incremental
updates for active subscribers, remain pending. Errors are values that identify
the scope and ID, including duplicate keys, cycles, undeclared reads, and
reentrant writes.
Hot reload re-registers callbacks by ID while sources and mounts survive.

The store signatures above are landed. TypeScript mount wiring and signatures,
removal of TypeScript tree builders, rendering through stable render slots,
and bounded layout and paint remain pending. Performance after-numbers are
also pending; native binding scheduling alone does not establish bounded
rendering costs.
The [components reference](../crates/og-api/guides/components.md) describes the
implemented document dialect; the
[screens guide](../crates/og-api/guides/screens.md) describes its workflow.

Fixtures contain supplied presentation states, not playable economies,
combat loops, matchmaking, or rhythm clocks. `og create` writes one empty
project skeleton with creation constraints. It never installs a genre
fixture as a gameplay template.

### 2.2 Dialogue text and attribute spans

`DialogueView` receives localized plain text and attribute spans shaped as
`{ name, position, length, properties }`. Properties contain typed string,
finite number, or boolean values. Positions and lengths count Unicode
scalar values, not UTF-16 code units, bytes, or grapheme clusters.
A `character` span covers the `Speaker: ` prefix, including the colon and
space. That prefix remains in the delivered text.
The Yarn 3.2.2 dialogue runtime resolves `nomarkup`, escapes, and CLDR
select, plural, and ordinal expressions before delivery. UI does not
parse them again.

UI maps visual attributes such as bold and wave to span styles and motion.
Core accepts plain text plus structured spans directly. The text stays
literal and is never re-parsed as markup. When spans overlap, later spans
in array order override earlier ones only for fields they declare.
Unknown spans, zero-length spans, and their properties remain unchanged
in the response. Parts resolve against the active style through selectors
such as `text::part(name)` and `#Dialogue::part(name)`.

Styled runs share core shaping and line breaking. Span motion is
checkpointed and restores exactly; reduced motion makes it stationary.
Core structured text and the template binding
`<text spans={$spans}>{$body}</text>` with `::part` are implemented.
Both-host execution verification of this binding remains pending.
Regression coverage includes overlapping styles and motion, literal
brackets and escaped text, speaker prefixes, Unicode offsets, wrapping,
active-style changes, and reduced motion. Kits use this core text path,
not a private renderer.

### 2.3 Text input types

`<input>` takes the HTML names `type`, `inputmode`, and `autocomplete`;
`FieldDesc` carries the same three for TypeScript on both hosts. The types
are `text`, `password`, `number`, `email`, `url`, `tel`, and `search`.
Validity follows the HTML Living Standard: a valid floating-point number,
its email expression, and an absolute URL with a host for special schemes.
A nonempty text that fails its type matches `:invalid` and dresses the
invalid frame. An empty field is never a type mismatch. A number field
types only digits, a sign, a point, and an exponent.

`<input>` and `<textarea>` also take HTML's constraints, with its validity
states: `required` (an empty one is `valueMissing`), `minlength`
(`tooShort`), `pattern` (the whole text must match, compiled as
`^(?:pattern)$`; an invalid pattern is a parse error) and, on a number,
`min`, `max` and `step` (`rangeUnderflow`, `rangeOverflow`,
`stepMismatch` from `min`; the default step is 1, `step="any"` turns it
off). Any failure matches `:invalid`, in the retained cascade and the
immediate draw alike. `maxlength` stops typing at its count. `readonly`
drops every edit (typing, paste, cut, delete) and keeps selection, copy
and the focus stop. `<textarea rows>` shows that many lines (0 is 4).

A password is drawn masked with one bullet per character (`*` where the
face lacks U+2022). The probe and the accessibility tree read the mask.
Copy, cut, and the emacs kill buffer never take its text, and the host is
given no selection for it. Word moves treat it as one word. Its reveal
toggle `<id>/Reveal` sits at the field's end (start in RTL), dressed
`<component>.reveal[revealed]` with a `reveal-icon` subpart. It is named
Show password or Hide password and keeps the field's text focus when
pressed. Revealed is UI state and survives save and restore.

The host receives the type, the effective `inputmode` (unset: the type's
own), `autocomplete`, and `secret` with the focused text target. The web
shell stands a hidden `<input>` with those attributes behind a one-line
field so phones show the requested keyboard and autofill reaches it; a
text area keeps its `<textarea>`. A number uses a text input with the
decimal keyboard.

#### Selectable text

CSS `user-select` (`ElementProps.userSelect` in TypeScript) makes read-only
text selectable: `text` selects by characters, `all` takes the element's
whole text with one press, `none` turns it off, and `auto` takes the
parent's. Nothing is selectable by default, and a control's subtree (a
button's label) stays `none` unless it sets the property itself. Only plain
text elements take a selection; rich text and text controls keep their own.

A press and drag selects, a double press selects a word (UAX #29), a triple
press a line, and Shift with a press extends. On a coarse pointer a long
press (500 ms) selects the word under it and a drag that moves at once
scrolls instead. A drag runs across elements in the tree's order, as a page
selection does: one that leaves the text picks the selectable text level
with the pointer, else the nearest above or below. A press anywhere else
clears it. Ctrl+C (Cmd+C under mac keys) copies it to the host's clipboard
unless a text control has the focus. The copy joins elements with line
breaks and puts back what each wrap dropped: a space, nothing between CJK,
or the source's line break. The fill is the text controls' `TextEdit
.selection` part, under the glyphs, and the pointer is the I-beam over
selectable text. The selection is UI state (version 20) and survives save
and restore. `dump_ui` carries it as `selection` (`text`, and each end's
`path` and character), `null` with none. Only the elements whose selected
range changed are painted again.

#### Text on a path

`-og-text-path` (`ElementProps.textPath` in TypeScript) sets an element's
text along a path instead of in lines, as SVG 2's `<textPath>` does: each
grapheme cluster sits with the middle of its advance on the path, turned to
the path's direction there, its baseline on the path. CSS `offset-path`
moves the element itself, so it is not reused.

```css
.title { -og-text-path: path("M0 90 Q170 -20 340 90"); -og-text-path-offset: 50%; text-align: center; }
.seal  { -og-text-path: circle(52px); -og-text-path-side: right; }
```

- `-og-text-path`: `none`, `path("<SVG path data>")` (`M L H V C S Q T A
  Z`, absolute and relative, in CSS px from the content box's origin), or
  `circle([<radius>])`, which is centred in the content box and starts at
  its top, running clockwise. Without a radius, the circle takes the box's
  closest side. An `offset-path` shape such as `ray()` or `inset()` is an
  error that names the difference.
- `-og-text-path-offset`: where the anchor sits along the path, in `px` or
  as a percentage of the path's length (SVG's `startOffset`).
- `text-align` is the anchor at that offset (SVG's `text-anchor`): `start`
  starts the run there, `center` centres it, `end` ends it.
- `-og-text-path-side: right` runs the path backwards (SVG 2's `side`), so
  text on a circle reads counter-clockwise, inside it.

The text is one line, and its line breaks are spaces. A cluster whose
middle falls off an open path is not drawn (SVG 2 §11.8.3), and a circle
wraps. An element with no size of its own is as large as its path: the
path data's extent, or a circle's diameter. A centred path text keeps that
box instead of taking its column's width. Path data is flattened once per
distinct string, and an unchanged path text is not painted again. Path text
is not selectable (`user-select`). Its accessible name is still its text.
In TypeScript, `textPath` is `{ data, radius, offset, percent, reverse }`;
empty `data` is a circle, and data that does not parse keeps the text in
lines. The reference frames are `text-path-1280` and `text-path-390`.

#### Live regions

`aria-live="polite"` or `"assertive"` makes an element a live region, as
`role="status"` (polite), `role="log"` (polite) and `role="alert"`
(assertive) do; `aria-live="off"` turns a role's off. A screen reader
announces what changes in a region: politely after what it is reading, or
assertively at once. This follows ARIA's default `aria-relevant` (`additions
text`). A node that enters a region announces its words, and a node whose
name or value changed announces its new words. Removals and emptied text
announce nothing. A region that enters the tree already holding content
(a pushed screen's status) announces nothing, because ARIA announces a
region's changes, not its arrival; its later changes do announce. A
notification announces as it appears: an `alert`, and a toast
(`og.ui.toast`), whose node is marked `notify`. `aria-atomic="true"` (the default of `status` and
`alert`) announces the region's whole text, in tree order, instead of only
what changed. A region with a hidden node on the way up announces nothing.

```svelte
<text role="status">{$saved}</text>
<column aria-live="polite" aria-atomic="true"><text>Gold</text><text>{$gold}</text></column>
{#each $alerts as a (a.id)}<text role="alert">{a.text}</text>{/each}
```

The first tree a host reads announces nothing. Neither does the update
after a restore (`Ui::restore_state`, which is presentation state, not
news). A dialog opening or closing announces only what changed with it,
such as a toast shown as it closes: the page's nodes stay in the tree with
their words. Each change announces once, from the tree the host last sent. A host that falls behind
hears what changed since its last read, once. On the native host the
AccessKit bridge (`native_a11y::Bridge`) announces through two polite and
two assertive hidden live nodes under its root. Each announcement goes to
the voice that did not speak last, so the same words twice are heard twice.
`Bridge::take_announcements` drains them, holding at most `ANNOUNCEMENTS`
(16), and drops the oldest first. The web mirror writes the frame answer's
`announce` list into the page's polite and assertive regions. Both work in
publish builds. `og.ui.a11y()` gives each node `live` and `atomic`.

Announcing reads only what the bridge already reads: the changed nodes,
each one's ancestors up to its region (O(depth)), and an atomic region's
nodes. An idle frame reads nothing. The web mirror compares the tree it
sends with the one it sent before, only on frames where the tree changed.
The queue is presentation only: it is never checkpointed and never enters
`world_hash`. Under `OG_UI_CHECK_BOUNDED` the whole-tree twin of each
update must announce the same as the incremental update.

### 2.4 Select, dialog, details and meter

These four are core controls with HTML's elements and attributes; kits
dress them and never draw substitutes.

`<select bind:value={$x}>` lists literal `<option>`s, one level of
`<optgroup label>` around them. An option without `value` is valued by its
text, as in HTML; `<option selected>` is the value when none is bound;
`<option disabled>` and `<optgroup disabled>` cannot be picked. A bound
list is written `items={$labels} values={$ids}` on the select instead of
`{#each}`, which is refused. The list opens over the page at the chosen
option. A press, Enter, Space or pad A opens and picks; the arrows and the
d-pad move past disabled options; letters move by typeahead; Escape, pad B
or a press outside closes it. A pick or Back returns the focus to the
trigger. The value, never the label, is written back. Parts:
`Dropdown.trigger[open][invalid]`, `Dropdown.list`, `Dropdown.group`,
`Dropdown.option[selected][disabled]`. Roles: combobox, listbox, option.

`<dialog bind:open={$x} aria-label="…">` is HTML's dialog. With `modal`
it is `showModal()`: a scrim layer that takes every press, the focus moved
to its first control and trapped, Escape, pad B or a press on the scrim
closing it, and the focus returned to its opener. Like HTML's UA style
(top layer, `position: fixed`, `margin: auto`), the scrim covers the
viewport and the panel (`<id>/Panel`) is centred in it, whatever the page
under it. `dialog::backdrop { background: … }` fills the scrim (default
the style's `scrim`); a `::backdrop` rule takes only `background`, and
cascades with states, variables and media as the dialog's own rules do.
To a style pack the scrim is the part `Dialog.backdrop`. Without `modal` it is `show()`: a panel in place
(`Dialog.panel[open]`); the focus returns to what held it when it opened.
Closing writes `open` false back.

What closes it is HTML's `closedby`: `closerequest` (a modal's default)
is Escape, Back or pad B, for an in-place dialog only while the focus is
inside; `any` adds a press on a modal's scrim (light dismiss; an in-place
dialog's light dismiss comes with popovers); `none` (an in-place dialog's
default) leaves closing to the document writing `open` false.
`appearance: none` drops the panel's own padding, fill and dressed part,
so the document's CSS draws it.

A `<small>` next to a select or an input with `aria-invalid` is its
message: `aria-invalid` puts the danger frame on the control
(`Dropdown.trigger[invalid]`, the field's), and the text is the
document's. `select::picker(select) { max-height: 240px }` (CSS's
customizable-select pseudo-element) bounds the open list, which then
scrolls; it takes only `max-height`.

`data-og-instant` on any element settles its first entrance: its
`-og-enter` lands at once, as the screen a game opens on behind the web's
play gate needs; a later entrance, after it left, plays.

`<details bind:open={$x}><summary>…</summary>…</details>` is a disclosure.
The summary row (`Details.summary[open]`, role button with `expanded`)
opens and closes it by press, Enter, Space or pad A; its marker points to
the inline end while closed and down while open. The content
(`Details.content`, inside `Details.root`) is not built while closed. A
literal `open` is the initial state; the player's toggle is UI state that
saves and restores.

A closed dialog or disclosure builds no content: in a retained document
its children are released and built afresh when it opens, so they take
input and the focus from the frame they show.

`<meter>` is HTML's gauge, beside `<progress>`. Its fill is dressed by the
region HTML's rules put the value in, `Meter.fill[region=optimum]`,
`[region=suboptimum]` or `[region=poor]` (undressed: the style's success,
warn and danger), over `Meter.track`. `low` and `high` clamp into the
range and `optimum` defaults to its middle. Role meter with its range.

### 2.5 Split screen

Local multiplayer turns pads 1 to 3 on with `og.ui.setPlayers(n)`. Each pad
then moves a focus of its own (`Response.focusedBy`, bit `k`) and presses
with A (`Response.pad`). A split screen gives each player a region of the
screen: `player="k"` on an element (`ElementProps.player`, Rust
`Props::player`) makes its subtree player `k`'s view, from 0 to 3. Each region
can be its own `og.ui` scene viewport.

- Pad `k`'s focus moves only among the controls inside player `k`'s region.
  Pad 0's focus is also the keyboard's and the pointer's. Its first move
  lands on the control nearest the region's centre.
- Controls outside every region, such as a shared pause button or a lobby
  bar, belong to the pads that have no region. While a dialog is open, every
  pad also reaches the dialog's controls.
- A control pressed or clicked inside a region reports the player in
  `Response.player`. Outside every region it is null, and so it is on the
  ticks the control is not pressed. The pointer can press anywhere; the
  response still names the region it pressed in.
- On a screen with no region, nothing changes: every pad sees every control.

Each hit carries its region as one byte. The region is inherited down the tree
the way the focus group is, so it is computed when a node's ancestry changes,
not each frame. The UI state saves it from state version 21; older states
upgrade with no region.

## 3. Style contract

### 3.1 Tokens and literal values

Kit TypeScript and screen documents take visual values from semantic
tokens or named parts. This covers colour, typography, spacing, radius,
stroke, shadow, feedback, and motion. Games that customize these values
define them in their style rather than scatter them through widgets.

Literal values belong in these places:

- Core default definitions and pack-owned token, typography, motion, and
  dressing definitions, where the visual language acquires actual values.
- Art metadata, including atlas coordinates, slice insets, resource
  dimensions, and import settings.
- Fixture data, including health, elapsed time, scores, and item counts.
- Structural geometry, including ratios, normalized positions, column
  counts, and bounds derived from caller data. Reviewers check the purpose
  of these values; structural labels cannot disguise visual tuning.

Hex colours, literal font paths, arbitrary pixel spacing, and hard-coded
animation durations do not belong in kit composition. `u(17)`, `ts(15)`,
token values multiplied by arbitrary constants, and radius clamps with
literal minima do not become semantic tokens by passing through helpers.
Adding a token for every arbitrary number does not meet this contract.
If the public API cannot express a needed semantic value, the core owner
fixes that API before the kit depends on it.

### 3.2 Parts and states

A component exposes named parts with variants, state, and attributes.
Style selectors dress those parts; the grammar and specificity rules live
in `crates/og-ui/src/style_file/css.rs`. Document CSS lives in
`crates/og-ui/src/doc/css.rs`; generated references come from those sources.
Parser support for a literal does not override the authoring rule above.

A document's `<style>` controls appearance and layout; attributes control
content and behaviour. Declarations win by `!important`, selector
specificity, then source order. Rules are document-scoped and never affect
unrelated mounts. Parsed selectors support element, ID, class, attribute,
descendant, child, adjacent (`+`) and general (`~`) sibling matching, plus
`:not()`, `:is()`, and `:where()`. `:where()` has zero specificity; `:not()`
and `:is()` take the highest specificity of their arguments.

Structural pseudo-classes are `:first-child`, `:last-child`, `:only-child`,
`:nth-child(an+b [of S])`, `:nth-last-child(an+b [of S])`, and `:empty`.
`of S` takes compound selectors and adds their highest specificity. Siblings
and positions are those of the instantiated element tree, after `{#if}`,
keyed `{#each}`, snippets and components; structural blocks are not
siblings. `:empty` means no instantiated element children and no text.
Retained mounts match on the logical sibling order, never render slots.
An edit invalidates only the sibling interval it can affect, in O(log n)
per interval. Virtualized rows outside the materialized range restyle when
they materialize, so a front insert restyles only visible and overscan rows.
Structural selectors cannot select `::part` spans. `:has()` and other
pseudo-classes fail at cook with the selector named.

The ten state selectors are `:hover`, `:active`, `:focus`, `:focus-visible`,
`:focus-within`, `:disabled`, `:checked`, `:selected`, `:invalid`, and
`:placeholder-shown`. An ancestor selector reads that ancestor's own
interaction identity, not the styled child's state.

Custom properties are lexical and inherit through instantiated parents,
including component mounts. Recursive `var()` supports fallbacks, even an
empty fallback. Values resolve on the declaring element before inheritance:
overriding `--a` in a child does not recompute inherited `--b: var(--a)`.
Cycles invalidate participating properties and report their names. An
invalid winning declaration does not revive a lower-priority declaration.
Unshadowed engine tokens resolve against the active style, and `::part`
declarations use the same cascade. These rules do not relax the kit's
token-only rule (§3.1).

Data reaches the cascade through bound custom properties, Svelte's
`style:--name={expression}` directive (or a literal `style:--name="4px"`).
It is an inline declaration on its element: it beats every rule but an
`!important` one and inherits like any custom property, so a child reads
it with `var()`. A number is written plain (`0.5`), a string as is; `null`
leaves the property unset, so `var(--fill, 0)`'s fallback applies. The
expression is one of the element's attributes, so a store change
re-resolves that element (and the children that inherit from it) and
nothing else. Only custom properties take a `style:` directive; `style`
and `style:width` are refused.

`calc()`, `min()`, `max()` and `clamp()` reduce once their `var()`s are
known: at cook for constant values, when the element resolves otherwise.
`width: calc(var(--fill) * 100%)` and `animation-delay: calc(var(--i) *
80ms)` work; `+` and `-` need spaces around them, `*` and `/` take a plain
number on one side, and `s`, `rad` and `turn` convert to `ms` and `deg`.
Lengths in several units stay a sum that layout resolves (§2, HTML
blocks): `calc(100% - 12px)` works.

Media rules use standard conditions, such as
`@media (width < 600px), (height < 600px)`. Supported features are `width`,
`height`, `aspect-ratio`, `orientation`, `prefers-reduced-motion` and
`-og-color-vision` (below), with
inclusive `min-`/`max-` bounds and range comparisons. Commas and `or` select
alternatives, `and` intersects, and `not` negates; mixed operators require
parentheses. Nested media rules intersect. There is no named-breakpoint
reader.

Container queries follow CSS Containment Level 3 size queries.
`container-type: size | inline-size` (with `container-name` and the
`container: <names> / <type>` shorthand) makes an element a query container
for its descendants. `@container [name] <condition>` takes the media range
grammar over `width`, `height`, `inline-size`, `block-size`, `aspect-ratio`
and `orientation`; a block-axis condition needs a `size` container. Each
condition reads the nearest eligible ancestor; with none, or before the first
measurement, it is unknown and false. Conditions settle through the
settlement hook on the first frame and on every resize, before paint:
elements that read a changed container resolve again from their evaluated
inputs and replace their fields, including `display`, list and grid boxes and
Rive frames. There is no size containment, so a query that keeps resizing its
own container fails as a settlement cycle. Style and scroll-state queries,
container units (`cqw` and the rest) and other features fail at cook with the
feature named.

Safe-area values use the supported CSS `env(safe-area-inset-*)` spellings.
Svelte `class:` directives select
states; they do not introduce a second styling language.
The `style:` directive accepts only bound CSS-token references, not raw
numbers, colours, or arbitrary expressions. Other styling belongs in
`<style>`. Title and game-over motion use standard CSS `animation` and
`transition`, with semantic motion tokens and reduced-motion behaviour.
`transition` and `animation` follow CSS Transitions and Animations Level 1:
property lists with per-property duration, delay and timing function, `all`,
several animations per element with delay, iteration count (including
`infinite`), direction, fill mode and play state, and keyframes at percentage
stops with per-keyframe timing functions. The shorthands expand into
longhands at cook, so selectors, state and media choose an element's
animations; an animation starts when its rule begins to match and ends when
it stops. Motion runs on presentation ticks, never a clock, and is saved with
the presentation state outside `world_hash`, so a restore continues exactly.
Reduced motion shows a finite animation's end and stops endless ones;
transitions use the element's reduced-motion fallback. Transitions and
keyframes cover opacity, translate, scale, rotate, `-og-skew` and
`border-radius`; transitions also cover background, color and z-index. Other
properties, `var()` inside a shorthand, and unknown keyframes fail at cook
with the property named.
`@keyframes` may be nested in `@media` and `@container`: the last
definition of a name whose conditions currently hold defines it, and a
switch between definitions restarts the animation as a name change does.
Scoped `@keyframes` can emit a style sound at a frame, such as
`-og-sound: error` for game-over entrance feedback. The event records its
playback rate; rate-valued fields use `playbackRate`, not `pitch`.

CPU motion, GPU mass elements, and camera timing share `Ease`: GSAP names
such as `none`, `power1.out`, and `back.out`, plus `cubic-bezier(...)` and
`steps(...)`. Style tokens retain `EaseRole` indirection. CSS timing
keywords belong to the CSS parser; shared timing does not make them GSAP
names. Lexical custom properties, the cascade, media and container queries
above are implemented.
The supported subset in the component catalogue is not full CSS support.

Core's native settlement hook publishes measured container content boxes
before effects, hit testing, and paint. First measurements include zero
dimensions; unchanged measurements do not notify again. Changed query
styles relayout their roots and dependent floating placement in the same
frame, with safe-area geometry included. Repeated geometry states produce
an error rather than publish an intermediate frame. Container queries use
this hook for immediate and mounted documents. Retained rendering limits and
TypeScript integration are tracked separately in §2.1.

Core parts keep their existing names and meanings. Each kit documents its
additional parts with its own API, so packs can dress them without private
imports. A missing custom dressing uses the readable core token fallback.
It is not permission to leave a required state unfinished.

Missing art uses a small mark fitted to the slot's shorter side and capped
at 32 UI units, centred in a quiet token-defined well. It does not stretch
into a large decorative frame. The `icon` and `placeholder` parts remain
available for styling; missing art does not erase the slot's identity.

Hover, pressed, focus-visible, selected, disabled, and selected+disabled
are distinct requirements. A selected item can also have focus; selection
must not hide the focus indicator. Disabled controls emit no activation
intent. State information cannot depend on colour alone.

Colour-blind modes are a player setting, `UiDesc.colorVision`: `protanopia`,
`deuteranopia` or `tritanopia` (`none` by default). The whole frame, scene
and UI alike, goes through one filter after every other effect
(`crates/og-render/src/color_vision.rs`): correction by default,
Fidaner's 2005 daltonisation, which moves what the viewer loses into the
channels they still tell apart; or, with `colorVisionFilter: "simulate"`,
how such a viewer sees the frame, for checking a screen. Simulation is
Viénot 1999 for protanopia and deuteranopia and Brettel 1997 for
tritanopia, as libDaltonLens publishes them (`third_party/daltonlens`).
`colorVisionStrength` (0 to 1) blends towards the unfiltered frame for
anomalous trichromacy. Typical vision costs nothing; a mode costs one
full-screen pass over the frame, the glass path's source copy. The
filter is presentation only: no world state or hash reads it.

Colour is never the only signal at the token level either. Every status
and rarity colour token pairs its colour with a cue, an icon, a pattern
or pips, distinct within its group, and the style refuses to cook
without one; team and damage colours may take one. A document draws a
token's icon cue with `icon { -og-cue: danger; }`, the icon the style
names for that token (an `icon` attribute wins). `@media
(-og-color-vision)` matches any mode and `(-og-color-vision:
deuteranopia)` one, so a pack can add cues, patterns or stronger
contrast when the player needs them; a retained mount restyles only the
elements that read the feature when the setting changes.

Button glyphs and input prompts follow the active device and switch live:
keyboard and mouse, Xbox, PlayStation, Switch, generic gamepad, and touch.
Core input glyphs and UI action prompts own this behaviour; kits do not
hard-code button letters or assume that every gamepad uses Xbox labels.
Actions declare `actionMap`, and `og.input.setActionMaps` selects the active
maps. `isActionPressed` reads held state; `isActionJustPressed` and
`isActionJustReleased` read tick edges. `actionHeld` retains its `holdMs`
threshold semantics, and physical controls retain their W3C codes.

Selection uses `selection` and `onSelection`; the main action uses
`primary` and `onPrimary`; small emphasis uses `accent`. HUD content over
the scene uses `sceneInk`, `scenePlate`, and `sceneShadow`, with the public
`onScene` treatment. Kits do not choose their own scene-separation colours.

Drawing orders ground and fills, rim, content and selection, hollow focus,
then children and raised badges. Focus does not erase art with an opaque
background cut. Decoration over children is a separate child.

Typography roles, breakpoint sizes, and numeric treatment come from the
style. Ramp aliases do not define another set of sizes. Player text scale
applies once after breakpoint selection. Locale-aware casing uses core
text facilities, not JavaScript uppercasing of labels.

Motion uses named style definitions and the engine clock. Feedback can be
interrupted and retargeted. Reduced motion has an explicit outcome;
decorative loops stop. Sound and haptics come from style event definitions
and respect player settings.

### 3.3 Assets and generation

Each pack dresses the core control set and the kit parts its acceptance
samples require. Art, icons, fonts, import settings, and licences travel
with their owning pack or the shared resource directory. Placeholder
characters are not finished icons. Shared resources and CJK fonts are not
genre kits or style choices. Core font fixtures must not depend on the
survival of a commercial style pack.

`style.ogt` is authored data. `og style new` reads pack-owned `choices.json`
and produces a starting style, procedural dressings, feedback, and
`art-requests.json`. It neither paints finished art nor regenerates the
built-in collection. Unknown families still receive a parseable starting
style. Generated references must resolve to existing resources.

Palette and scheme, shape, ornament, motion, and finish are seed choices.
Authored packs are not required to equal generator output. Art requests
use the [art contract](art.md); the
[art-review guide](../crates/og-api/guides/art-review.md) owns review advice.

### 3.4 Font packs

Fonts ship as packs, like the UI packs: one directory per script under
`templates/fonts/` (`latin`, `cyrillic-greek`, `arabic`, `hebrew`,
`devanagari`, `thai`, `japanese`, `korean`, `chinese-simplified`,
`chinese-traditional`, and `emoji`), each an `extension.json` of kind `font` naming its
`scripts`, `languages` and `faces`, the faces with their import settings,
and their licence. `og font list` names them; `og font add <pack>` installs
one under `assets/fonts/<pack>/`; `og create --lang` installs the packs of
its languages (Chinese by script: `zh-TW` takes the traditional pack) with
no generated code. `templates/fonts/make-packs.py` makes them from
google/fonts; `templates/fonts/LICENSES.md` lists each face's source and
licence.

The cook writes each pack's index (`og.FontPack`: its languages, face ids
and the code points their cmaps map) beside the faces, which cook as fonts
by their own settings to `<id>.face.ogb`. A pack face cooks no atlas (its
space alone) and draws every glyph from its outlines when first shown
(`runtime true`), so it costs about its file's size: Arabic 178 KB, Latin
127 KB, Japanese 1.35 MB, Simplified 1.42 MB, Traditional 2.04 MB; an
index is under 2 KB for an alphabet and under 40 KB for a CJK pack. A pack
changes no other font's cooked bytes, charset or run-time mode.

og.i18n chains the active table's own fonts, then the locale's pack, then
the packs a frame needed, then the other tables' fonts. A frame that draws
a character only an uninstalled-in-chain pack covers loads that pack (the
locale's CJK forms first; a Chinese locale never takes the other script's
pack first) and draws it from the next frame; it is not warned of as
missing. A character no pack covers is still named by the missing-glyph
warning. The web boot leaves `.face.ogb` files out of its preload and the
service worker caches them when fetched; the first frame waits for the
locale's own pack and its glyphs (`og_web_fonts_pending`).

A run-time face renders each glyph's distance field once into a 512² page
of its own and draws from it after, as an atlas font does: steady frames
cost the same (Metal, debug build: 2.8 ms CPU and 0.29 ms GPU either
way). The fields render off the frame: natively a thread per batch, on the
single-threaded web three a frame per face. A glyph draws from the frame
after it arrives; until then it is blank, never warned of. A probe
capture waits for every glyph asked for, so captured frames do not
depend on timing. Glyphs are presentation only: no world state, hash or
replay reads them.

Colour emoji are a font feature, not cooked images. The `emoji` pack is
Noto Color Emoji's COLRv1 face (SIL OFL 1.1, 5.05 MB, 1,508 characters
and their sequences): a pack of no language, named by its script `Zsye`,
tried after every text pack. Its face declares `color true` with `runtime
true`, so a glyph with a COLR paint is painted in colour when first shown,
by the same painter the cook uses (`crates/og-ui/src/colr.rs`, COLR v0
and v1) at `color_px` in the palette the settings name (the blob's
`color-runtime` section). Each glyph goes onto a colour page with a
silhouette distance field beside it for its shadows, glow and outline,
and a glyph in the text's colour (CPAL 0xFFFF) is painted black and white
and mixed at draw, as a cooked colour glyph is. Skin tones, ZWJ sequences
and flags are one glyph each, formed by the face's GSUB. A space or
punctuation after an emoji takes a text face's glyph, never the emoji
face's (whose space is 1.25 em), as browsers fall back. Any project font
can do the same with `font { color true runtime true }`; one with no COLR
glyph is refused.

Measured on Metal (debug build, `run_time_emoji_cost_what_cooked_emoji_cost`,
eight emoji including a skin tone, a family and a flag), against the same
emoji cooked as images: the steady frame costs the same (0.57 ms CPU
against 0.59 ms); the worst first-use frame is 4.8 ms against 6.6 ms,
because the painting runs off the frame. A colour glyph costs about two
distance fields to paint (3.0 ms median natively, unoptimised), so the web
counts it as two of its three glyphs a frame per face. Size: the pack cooks
to 5.08 MB for every emoji, which a game pays only when it installs the
pack; cooking just those eight as images is 431 KB, so a game that shows
a fixed handful of emoji can list them in `chars` with `color true` instead.
The reference frames are `color-emoji-1280` and `color-emoji-390`.

## 4. Shipped catalogue

Core default UI is the only creation `ui` choice during the rebuild.
The nine genre kits and six style packs below are approved scope, not
available packages. None is released before its acceptance is complete.
`materials` remains available as an optional resource pack for UI shader
finishes; it is neither a genre kit nor a seventh style. VFX and unrelated
extensions are outside this rebuild.

| Genre kit | Required presentation | Status |
| --- | --- | --- |
| `rpg` | Inventory and equipment, journal and tracker, dialogue, party and turn order, world map | Not yet built |
| `shooter` | Aim, ammunition, damage-direction HUD, killfeed, loadout | Not yet built |
| `strategy` | Selection and commands, production queue, waves, tower-upgrade inspection | Not yet built |
| `racing` | Telemetry, laps and standings, race banner, garage selection, results document | Not yet built |
| `card` | Hand and piles, collection and deck building, reward-choice and run-map documents | Not yet built |
| `puzzle` | Level path, goals and moves, level summary | Not yet built |
| `idle` | Production and purchase rows, prestige preview, offline-summary document | Not yet built |
| `rhythm` | Song and difficulty selection, judgement presentation, results breakdown | Not yet built |
| `arena` | Roster and character selection, draft, versus, team scoreboard, round HUD, lobby document | Not yet built |

Member signatures belong in each kit's API and the
[kits guide](../crates/og-api/guides/kits.md), not this catalogue.
Arena owns team and match scores; shooter does not duplicate them.
RPG also serves open-world games. Action-platformer and runner use core,
with RPG or puzzle compositions when needed. Kits never calculate damage,
production, purchases, rewards, legal card targets, or note timing.

| Style pack | Visual language | Commercial review references | Status |
| --- | --- | --- | --- |
| `clean` | Clear numeric hierarchy, restrained menus, low-interference HUD | Forza Horizon 5, Portal 2 | Not yet built |
| `cartoon` | Rounded forms, deliberate thickness, clear primary actions | Mario Kart 8 Deluxe, Bloons TD 6 | Not yet built |
| `fantasy` | Painted frames and materials with room for dense content | Baldur's Gate 3, Diablo IV | Not yet built |
| `pixel` | Consistent pixel grid, integer scaling, readable glyphs | Stardew Valley, Octopath Traveler | Not yet built |
| `scifi` | Instrument hierarchy, precise geometry, sparse emphasis | Elite Dangerous, Starfield | Not yet built |
| `storybook` | Paper and illustrated edges, quiet ornament, readable text | Pentiment, Strange Horticulture | Not yet built |

The approved final creation order is `clean`, `cartoon`, `fantasy`, `pixel`,
`scifi`, `storybook`. These values are not exposed before their packages
are ready. The integration owner updates the schema and bilingual labels
together. There are no `auto`, `custom`, or `generated` pseudo-packs.

The 37 look presets in `templates/looks/` and the generated look catalogue
own look-to-UI recommendations. During the rebuild every preset uses core
default UI. A look recommendation is a seed, not proof of visual fit;
an explicit creation UI choice wins. Scifi is also selected by subject,
especially shooters, strategy, and space games, not just by look mapping.
Game-authored styles use explicit asset paths in the actual look. The
creation block records the initial constraint, not the current asset path.

## 5. Package lifecycle

Official kits live in `packages/engine/extensions/<name>`; official styles
live in `templates/styles/<name>` and ship under `styles/<name>`.
`extension.json` describes each package. Discovery reads manifests and
document headers, not a hard-coded runtime name registry. A style has no
executable entry; document-only kits need no TypeScript entry.

`og ui add` copies a kit to `assets/ui-kits/<name>`; `og style add` copies
a style to `assets/styles/<name>`. Installed copies win over bundled copies
of the same kind and name. Discovery never evaluates kit code. Games
import the chosen TypeScript entry explicitly. Installation preserves
relative paths and licences and does not silently replace existing edits.

`og ui eject` copies a document and its companion assets to the game's UI
directory. Companion edits survive; force replacement applies to the
requested document. Add, eject, list, and manifest options are documented
in CLI help rather than duplicated here.

A look resolves an installed style before a bundled style of the same
slug. Explicit asset paths remain valid. The cook includes referenced
style resources and their shared dependencies, not every pack. Tests,
reference frames, and art source files are not runtime asset payload.

Public helper signatures, kit inputs and outputs, parts, and creation
schema are contracts. Breaking changes require a minor release and a
breaking entry with its fix in a new `changelog/unreleased/` fragment.
The rebuild has no old-game migration or legacy aliases. Incomplete kits
and packs can be absent on main; incomplete engine archives cannot ship.
Release waits for the complete approved catalogue and platform schema and
pin coordination. See [release rules](release.md).

## 6. Author and accept

### 6.1 State fixtures

Each kit starts with an input/output and parts inventory, its two primary
pages, and two named commercial games as information-structure references.
References identify an actual page, platform, and screenshot source.
Authors do not copy commercial assets, brands, or layouts.

Every member has reproducible fixtures and observable behaviour assertions
for the applicable cases below. An inapplicable case needs a reason.

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

Core tests use small self-contained styles for core behaviour, including
specificity, font pixel snapping, and probe paths. Pack tests check pack
art and resources. Removing a pack cannot remove core regression coverage.

### 6.2 Frame matrix

Each kit maintains two primary pages, one normal high-frequency page and
one stress/state page. Behaviour tests cover every member. An extra page
requires a named visual behaviour that those two pages cannot cover.

| Kit | Primary pack | Contrast sample | Normal page; stress/state page |
| --- | --- | --- | --- |
| rpg | fantasy | pixel | Inventory and equipment; journal, dialogue, party, map |
| shooter | clean | scifi | Combat HUD; loadout and killfeed |
| strategy | scifi | cartoon | Selection, commands, production; upgrades and waves |
| racing | clean | cartoon | Speed, standings, lap deltas; garage and results |
| card | fantasy | clean | Hand, targets, piles; collection, rewards, run map |
| puzzle | cartoon | storybook | Goals and moves; level path and summary |
| idle | clean | storybook | Production and purchase; prestige, offline, overflow |
| rhythm | scifi | pixel | Songs and difficulty; judgement and results |
| arena | clean | fantasy | Selection, draft, lobby; teams, scores, round results |

Each sampled pair retains four frames: normal desktop at 100%, normal
phone at 100%, stress desktop at 150%, and stress phone at 150%.
Desktop is 1920×1080; phone portrait is 390×844. Landscape-heavy content
also has a genuine landscape fixture, not a rotated portrait capture.
Nine kits × two packs × four frames yields 72 primary references.

`og ui sheet` retains its four cells per page, desktop and phone at both
text scales. Two pages therefore produce eight review captures per pair;
only the four specified above become the baseline references. Locale,
RTL, landscape, and additional states receive risk-based review captures,
not a full Cartesian product of permanent reference images.

Each of the six packs also has a core-control state sheet covering the
whole control set, with desktop and phone references. These 12 references
make 84 baseline frames in total. Large text and localization still need
review on each pack. Additional references name the regression they guard.

All 54 kit/pack combinations run nonvisual loading, parsing, resource
resolution, and runnability checks. Eighteen sampled pairs are not visual
coverage of all combinations. A game's actual combination needs review.

Captures record effective style, theme, viewport, DPR, text scale, locale,
focus, selection, tick, and animation history. Equal ticks alone do not
prove equal state. References are recorded on macrunner with Metal;
other real GPU backends compare using the existing cross-platform
tolerance. A missing GPU is unchecked, never passed.

### 6.3 Acceptance and art sign-off

Technical checks reject parse failures, missing resources, wrong intents,
missing required states, and out-of-bounds references. `og ui check`
reports contrast, layout, and cost measurements; `og ui literals` reports
suspect values. Neither command scores taste. Warnings do not become
automatic aesthetic failures. Author thresholds remain in the screens
and art-review guides, not in a second table here.

The author reviews the core sheet, both kit sample pages, phone and large
text, empty and overflow states, and bright and dark HUD backgrounds.
Motion review includes interruption and cancellation. Commercial page
references must have comparable information density, not just a logo or
promotional image. Review compares hierarchy, numeric readability, focus,
selection, reachability, feedback, and consistency across components.

The designated art reviewer is a separate review stream assigned by the
coordinator. It signs off the actual frames and clips before a new kit or
pack lands through `scripts/land.sh`. A rejection names the
problem and screenshot region; the author fixes and recaptures it. A
palette swap, decorative background, or desktop-only success is not
acceptance. Core-default kit captures prove behaviour, not final art.

The first accepted group is clean and fantasy with RPG and shooter.
Acceptance covers each available pairing; matrix samples that depend on
unbuilt packs remain explicitly pending. The remaining catalogue follows
only after that group proves the complete author-to-review process.
Release still requires the complete matrix. Quality failures delay release
rather than lower the scope or silently waive sign-off.

### 6.4 UI debugger

A development build's inspector (F1) has a UI view, its third tab. Pick
picks the element of the game's UI under the pointer, and the element under
the pointer is outlined while picking. The panel then shows the picked
element:

- Its path and tag.
- Its box model: margin, border and padding in CSS order, and the content
  box and border box with their positions.
- The layout boundaries it sets: `scroll` (a scroll container), `container`
  (a container queries read), `layer` (a layer's root) and `fixed` (a fixed
  width and height, so its children cannot move what is outside it).
- Its computed styles: each winning document declaration with its value and
  the `file:line` it was written on, the style pack's rules that dress its
  part with where each was written, and its bindings with the expression as
  written and the value now.

The frame draws the picked element's margin, border, padding and content
boxes over the game in Chrome's colours. Layout bounds outlines every
element drawn. The filter field keeps the styles, rules and bindings whose
name holds it. `inspect:ui[:<path>]` opens the view from a script, and
`dump_inspector` carries the picked element as `ui`.

The view is built on the on-demand inspection API (`Ui::elements`,
`Ui::element_bounds`): `Ui::debug_pick(x, y)` and `Ui::debug_element(path)`
read the last finished frame, and `og_ui::debug_overlay` draws a debugger's
boxes into its own UI. A pick walks the drawn elements once. Reading the
picked element is one lookup by its render key, and its cascade is the one
its last resolve kept, so nothing is resolved again. The game's UI is read
only while the view is the tab shown. The reference frames are
`ui-debug-1280` and `ui-debug-390`.

`dump_ui:motion[:path]` writes the motion alone, as `dump_ui:layers` writes
the layer stack: each element in a visual state or animating, with its
states, what is moving, what plays and whether it is leaving.

## 7. Maintenance boundaries

The integration owner owns shared schema, labels, looks, creation,
packaging, this contract, and guides. Kit and pack authors own their
package directories and acceptance fixtures. Shared API changes go through
the integration owner before consumers adopt them.

Pack tests and references live outside runtime pack roots under
`templates/styles/tests/<pack>` and `templates/styles/references/<pack>`.
Core fixtures live with core tests. Reference changes identify the affected
frames and their reason; reference recording alone is not art approval.

API declarations, the component catalogue, and the wiki are generated from
IDL, parser metadata, and `crates/og-api/guides/`. Look catalogue changes
follow the existing generator. Generated files are never hand-edited.
Core default design tokens come from `packages/design`; they do not impose
the default appearance on game-authored styles.

Missing core capabilities are tracked in [the roadmap](roadmap.md).
Variable-height virtualization and localized number/date formatting
needed by kits are core prerequisites,
not invitations to private kit workarounds. Per-frame UI cost and its
performance tests are independent measured work; this document claims no
unmeasured performance budget has passed. Author knowledge belongs in guides, not shipped
skills. No kit silently implements those missing facilities itself.

The [gate rules](ci.md) own test lanes, locks, and toolchain policy.
`scripts/land.sh` owns landing on main. Release and platform installation
follow [release rules](release.md), with separate release authorization.
