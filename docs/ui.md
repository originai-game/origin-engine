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
| Core | Elements, layout, drawing, controls, documents, layers, focus, gestures, motion, text, accessibility, semantic tokens, and public composition helpers | One control set and one API on native and web |
| Built-in components (core) | Component documents by group (`rpg/`, `shooter/`…) and screen documents, built into the engine under `@ui/`: presentation composed from core controls, named parts, typed inputs, and returned intents | No gameplay rules, literal visual values, private engine imports, or second widget toolkit |
| Style pack | Palette, typography, spacing, shape, dressings, art, icons, motion, sound, and haptic definitions, as `:root` tokens and `var()` part rules | No gameplay code or domain state |
| Game | Content, rules, world state, domain validation, and applying UI intents | Owns its screens and chosen style assets |
| Platform | Accounts, money, publishing, threads, agent orchestration, and engine version selection | Consumes released archives and the public host ABI, never engine internals |

The renderer owns the UI pass. `og-ui` owns layout, documents, interaction,
and style interpretation. `packages/engine/src/ui/` composes the public
controls. The public TypeScript API comes from `crates/og-api/api/` for
both hosts. Game authors never need Rust.

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

- Look is the CSS dialect plus semantic tokens (§3).
- Structure is Svelte 5 templates in `.svelte` files. TypeScript builds no
  UI tree: a document is the only way to author structure.
- Data uses the `svelte/store` model (§2.1). Games prepare named stores and
  pass them by name when mounting a document; its bindings and keyed loops
  read those sources.

The [screens guide](../crates/og-api/guides/screens.md) and the generated
[component catalogue](../crates/og-api/guides/components.md) describe the
syntax. A document must meet these requirements:

- A `.svelte` file has compile-time `<meta>` declarations, one root
  element, and document-scoped `<style>`; there is no `<template>` wrapper.
  Metadata declares screen or component identity, layer, focus, typed
  props, actions, and preview fixtures. IDs are unique within the document.
- Text uses `{expr}` and attributes `attr={expr}`. Mounted stores are
  `$name`; props and each-row aliases are plain names. Expressions are
  read-only. `onclick={action}` dispatches a named mounted action, not a
  call body. `bind:value` writes a writable store or keyed row field in the
  current tick batch; `value` alone is read-only.
- `{#each $items as item, i (item.id)}` names the row's position `i`: from
  0, in the list's order, following a move, insert or remove. It is for
  display (a rank, a slot number); a key that uses it is refused, since a
  row keeps its identity when the list reorders.
- `{#each row.cells as cell, i (cell.id)}` inside an each reads a list held
  in a row: an array of objects, each with a nonempty string key field,
  none twice. Its rows follow the list as the holding field is patched
  (new keys are made, missing ones go, kept ones move), and `bind:` on a
  nested row's field writes the holding row. The key is a field of the
  item, written `(cell.id)`.
- A declared input action dispatches a named markup event of its own:
  `<meta name="action:confirm" content="Start">` enables
  `onconfirm={start}`. A handler outside any `{#each}` row is the page's
  and acts wherever the focus is; one inside a row acts only while a focus
  (the shared one or a pad's) is within that row, with its row key.
- Each loop needs stable, nonempty string keys and may have an empty
  branch; missing, index-based, empty, and duplicate keys are errors.
  `selected` and `mark` name collection keys, never row indices.
- Capitalized tags resolve document components. Content uses
  `{#snippet name()}…{/snippet}` and `{@render name?.()}`; mount content
  names a document and snippet, never a TypeScript callback. `<slot>` is
  refused with its replacement.
- Visible text uses `t(key, { named: value })` with ICU parameters or
  static `i18n="key"` with fallback content. Unlocalized visible literals
  warn at cook and fail publish. Object expressions are allowed only for
  translation parameters and classes. Companion English messages use cooked
  string tables. The pseudo-locale (`og ui check`'s pseudo cells,
  `og run --locale pseudo`) shows them accented, 40 % longer and in
  brackets (`og_i18n::pseudo_text`).
- JSX, `$state`, `$effect`, `$bindable` outside a prop's default,
  statements, imports and functions in a script, `{@html}`, `on:click`,
  `<svelte:document>`, `<svelte:boundary>` and the Svelte 4 elements are
  refused, each with the form to use. Parse errors name file, line,
  column, and a fix.

#### Svelte blocks, script and directives

Each keeps Svelte 5's meaning mapped onto og-ui; none runs code in the
tick.

- `{#key expr}` remakes its content when `expr` changes: new elements
  (ids carry the value's hash), the old leaving with their exits.
- `{@const name = expr}` sits directly inside a block or component and is
  replaced by its expression where read (no binding). A name for a
  non-path expression is read whole.
- `{#await $load}…{:then value}…{:catch error}{/await}` reads a store or
  row field holding `{ status: "pending" | "fulfilled" | "rejected", value,
  reason }` (`Promise.allSettled`'s words), written by a TS system; no
  promise runs in the tick.
- A prop is declared once, as `<meta name="prop:name" content="type"
  data-doc="…">`. `data-default="…"` gives its default: a string's is the
  attribute's text, the other types' are JSON of that type (`5`, `true`,
  `[1, 2]`). A prop with a default is optional in the generated `.d.ts`; a
  prop with none is required, and a mount leaving it out returns an error
  naming the instance and prop. `$props()` is refused.
- `<script lang="ts">` holds only `const name = $derived(expr)` and
  `$derived.by(() => expr)` over pure expressions. Derived values are
  replaced where read. State and logic live in a TS system.
- `transition:`, `in:` and `out:` (`|local`, `|global` accepted) set the
  element's `enter`/`exit` from `fade`, `fly`, `slide` (scales along its
  axis; layout does not shrink), `scale`, `blur` (through `filter: blur()`)
  and `draw` (an SVG shape's stroke, as Svelte's; `speed` is refused, give
  `duration`), with literal `duration` (default 400 ms), `delay` and
  `svelte/easing` names (elastic and bounce are springs).
  `in:receive`/`out:send`/`transition:crossfade` with `{{ key }}` is the
  shared-element key (`match`).
- `animate:flip` on a keyed `{#each}` child is its layout motion (default
  300 ms, `cubicOut`).
- `style="…"`, `style:prop="value"` and `style:prop={expr}` (any supported
  property; `style:--name` a custom property) are the element's own
  declarations: they beat every rule but an `!important` one, and
  `style:prop|important` beats that too. A bound value carries its unit.
- `bind:this={name}` makes `name` the element's id, readable wherever an
  expression names an element and as `#name` in CSS.
- `use:name={params}` sends a DocEvent of kind `use` named `name` with
  `{ phase, params }`, phase `create`, `update` (params changed) or
  `destroy`; the game's TS handler does the action's work.
- `<svelte:window onresize bind:innerWidth bind:innerHeight/>` beside the
  root reports the screen size in CSS px (also `$ui.width`, `$ui.height`).
  Keys reach a game as input actions, so `onkeydown` is refused.
- `<svelte:element this>` takes a text tag (`p`, `span`, `small`,
  `h1`–`h6`) for inline content or a block tag; another value is nothing.
  Its descendants take no ids.

Core elements use HTML's tag and attribute names wherever HTML names the
role; a pre-HTML tag (`<field>`, `<slider>`, `<toggle>`, `<divider>`,
`<image>`, a `<label>` that neither wraps a checkbox nor has `for`) is
refused with its HTML form. CSS type selectors match the written tag
(`input[type=range]`). Elements HTML has no name for (`column`, `row`,
`stack`, `grid`, `panel`, `scroll`, the virtualised `list` and `cells`,
`art`, `icon`, `tabs`, `badge`, `spacer`) keep their names. The mapping:

| Document | Core kind | Notes |
|---|---|---|
| `<input>` (`type` text, password, number, email, url, tel, search) | field | `placeholder`, `maxlength`, `minlength`, `readonly`, `required`, `pattern`, `min`, `max`, `step` (`any`), `aria-invalid`, `list` (below); `appearance: none` drops its frame |
| `<textarea rows>` | field (multi-line) | the same constraints; `rows` lines shown |
| `<input type="range">` | slider | `min`, `max`, `step`, `disabled`, `list`; `data-fill-origin` sets the value the fill grows from; `::part(reach)`, `track`, `fill`, `tick`, `knob`, `core` and `bubble` are boxes CSS sizes and paints, the slider keeping input, focus and role |
| `<input type="range" bind:low bind:high>` | slider (range) | two knobs on one track, the fill between: a press takes the nearer, each knob is a focus stop, neither passes the other; `data-min-gap` keeps them apart. Readout `low – high`; to a screen reader two sliders, `<name>, minimum` and `<name>, maximum` |
| `<datalist id>` of `<option value [label]>` | — | draws nothing. A range's `list` ticks each option and labels it under the track. A text input's `list` offers options (or `items={$names}`) as suggestions under the focused field (dressed `Dropdown.list`, `Dropdown.option`): those containing the text, case and accents ignored, prefix matches first, at most 8. Focus stays in the field; Down reaches the list, a choice writes the value and returns focus, Back closes it until the player types |
| `<input type="checkbox" switch>` | toggle | `checked`, `disabled`; `::part(track)`, `::part(knob)` sized by CSS |
| `<input type="checkbox">` | toggle | `checked`, `indeterminate` (a dash; a press turns it on) |
| `<label><input type="checkbox"> Caption</label>` | toggle | one node; the caption sits where written, `flex-direction: row-reverse` swaps it; a `<small>` inside is its description |
| `<label for="id">` | — | a checkbox takes its text as caption; an input, textarea, select, range, meter or progress takes it as accessible name and the label stays a text |
| `<hr>` | divider | |
| `<img>` | image | `src`, `alt`; `object-view-box: inset(…%)` of the texture; `object-fit`, `object-position`, `image-rendering` (Images below) |
| `<p>`, `<span>`, `<small>`, `<text>` | label | inline `<b>`/`<strong>`, `<i>`/`<em>`, `<u>`, `<s>`/`<del>`, `<sub>`/`<sup>`, `<br>`, `<span>` and `<a href>` style runs |
| `<h1>` … `<h6>` | label (heading) | `aria-level` 1–6; UA type steps `xl3`, `xl`, `lg`, `body`, `sm`, `xs` |
| `<a href>` | label (a link) | a click sends a `link` event named by its href; accent ink, underline |
| `<div>`, `<section>`, `<header>`, `<footer>`, `<nav>`, `<main>`, `<aside>`, `<article>` | box | layout from `display`; landmarks take their ARIA roles, `region` for a named `<section>` |
| `<ul>`, `<ol>`, `<li>` | box (list, listitem) | not virtualised (that is `<list>`); `<ol start reversed>`, `<li value>` |
| `<button>`, `<progress>` | button, progress | a `<button>`'s children lay out as a centred row in its frame and take its ink when they have none |
| `<select>`, `<option>`, `<optgroup>` | select | §2.4 |
| `<dialog>` | dialog | §2.4; `appearance: none` drops its frame |
| `<details>` with a first `<summary>` | details | §2.4 |
| `<meter>` | meter | §2.4; `::part(track)` a box CSS sizes and paints |
| `<table>`, `<caption>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>` | box (table, row, cell, columnheader, rowheader) | CSS table layout with HTML's UA displays and `colspan` (1–1000), `rowspan` (0–1000); `{#each}` rows included. The first `<caption>` names the table; `scope` (or position) makes a `<th>` a row or column header |
| `<form onsubmit onreset>` | box | a `<button>` submits (`type="reset"` resets, `type="button"` neither); a submit or reset button outside a form is refused. Enter in one of its single-line text fields submits it as a press of its first submit button would (nothing while that button is `disabled`, or when the form has none). Submit is constraint-validated as HTML's: while a field is `:invalid` the action is not sent and the first invalid field takes focus; `novalidate`/`formnovalidate` skip it. In an `{#each}` only the button's row's fields count |
| `<fieldset disabled>` with a first `<legend>` | box (group) | the legend names it; `disabled` disables every control but the legend's |
| `<figure>` with `<figcaption>` | box (figure) | the figcaption names it |
| `<output for="ids">` | label (status) | read politely as it changes |
| `<input type="date">` (`time`, `datetime-local`, `month`, `week`) `bind:value` | datetime (group) | og-ui steppers per part (Year, Month, Day, Week, Hour, Minute, Second when `step` or the value has seconds), each a focus stop; no OS picker. Overflow carries (30 February is 1 March); a moved month keeps the day in range. HTML's value forms, kept in `min`…`max` and on the `step` grid from `min`. Empty, it shows `min` (else `max`, else 2000-01-01 00:00; no clock is read) and writes once stepped. Empty-and-required or out of range is `:invalid`. Parts `Stepper.*`; probe `<id>/<Part>/Frame/Inc` |

`<video src={$player}>` and `<audio src={$player}>` play a player the game
owns, the entity of an og.VideoPlayer or og.AudioSource; `src` is never a
file name (spawn the player in a system). `bind:paused`,
`bind:currentTime`, `bind:volume`, `bind:muted`, `bind:duration`,
`bind:ended`, `autoplay`, `loop`, `poster` and `controls` follow Svelte and
HTML. Requests go as `og.MediaIntent` through the recorded intent path and
apply at the next tick, so playback stays world state; read-backs are the
last tick's. A media element rebuilds every frame while mounted. An
`<audio>` has no `currentTime`, `duration`, `ended` or `poster`.

`<template>` is refused (snippets replace it). `<input type="file" accept
multiple bind:value>` is built only where `og.ui.filePicker` is true (false
on both hosts today; elsewhere the frame warns): a press sends the host a
`filepick` event and the host writes the chosen names. `field-sizing`
(CSS UI 4) applies to text inputs, textareas (along the inline axis),
selects and file inputs.

#### Inline objects, span parts and translated markup

A text element's words may hold one-em inline objects with literal
attributes: `<img src="coin" alt="">` (a registered picture,
`UiDesc.images`), `<og-icon name>` (the style's atlas) and `<kbd
data-action>` (the action's glyph). `<span part="big">` names a run for
`p::part(big)`, which takes `color`, `font`, `font-family`, `font-weight`,
`font-style`, `text-decoration-line`, `-og-text-motion`, `vertical-align`,
and `font-size`, `line-height` and `letter-spacing` in px or em. A run's
`vertical-align` follows CSS 2.1 §10.8 and CSS Inline 3; `<sub>` and
`<sup>` are runs at ÷ 1.2 size as HTML's UA sheet has them. A translated
string styles runs with MessageFormat 2 markup: `{#b}…{/b}`, `{#i}`,
`{#u}`, `{#s}`, `{#br/}`; `\{` escapes a brace.

#### Glyphs and the `$ui` store

`<kbd>Esc</kbd>` is a keycap. `<kbd data-action="jump"></kbd>` is the
action's glyph on the device used last (key, the pad family's button, or
touch icon; the style's icon where the atlas has one, else the name on a
cap), named by the action's label. With `data-og-rebind bind:value` it
captures the next key code into the store (Escape cancels); the game
applies it through og.input.

Every document reads the engine's `$ui` store unless a mount binds its own
`ui`: `$ui.device` (`keyboardMouse`, `xbox`, `playstation`, `switch`,
`touch`…), `$ui.coarsePointer`, `$ui.width`, `$ui.height`, and
`$ui.actions`, the top layer's action bar, each `{ action, hint, icon,
text, hold }`. It publishes only on change.

#### ARIA

Any element takes ARIA over its kind's semantics: `role` (one of og.ui's
roles; another is refused at parse), `aria-label`, `aria-labelledby` and
`aria-describedby` (one id's text follows it when bound; several ids join
their literal texts), `aria-description`, `aria-value*`, and the states
`aria-checked`, `-selected`, `-expanded`, `-pressed`, `-busy`, `-disabled`
(also inert), `-level` and `-current`. A bound one updates the tree when
its element rebuilds.

#### Checkboxes, radios, steppers, menus and press events

`<input type="radio" value bind:group={$level}>` is checked while the
store holds its value; radios binding one store are one group (`name` adds
nothing). `<input type="number">` is an editable field with increment and
decrement buttons bounded by `min`, `max`, `step`; `data-wrap` wraps,
`data-large-step` sets Shift and held repeats, `data-format` (an ICU
message over `value`) applies only while unfocused. `<menu>` is a box with
role menu whose buttons are menu items, each as wide as the menu, label at
its start. Beyond `onclick`: `onhold` once per press held `data-hold` ms
(default 500; the button draws `::-og-hold` as it fills), `oncontextmenu`
on a secondary or long press, `ontoggle` for `<details>` and popovers,
`onclose` for `<dialog>`.

#### Counters, typewriters, cyclers, spotlights and colour inputs

`<og-counter value duration decimals>` counts to its value as it changes.
`<og-typewriter rate>` reveals grapheme clusters per second, restarts when
its text changes, shows the whole text under reduced motion and fires
`onrevealed` once. `<select data-cycle>` is previous, option and next,
wrapping past disabled options (its bound `items` without `<option>`s).
An open `<og-spotlight target>` dims all but the target element, outlines
it and shows its content as a callout; the shade takes presses
(`closedby="any"` closes on one). `<input type="color" bind:value>` is a
swatch opening a saturation/value square, hue slider and hex field, and
writes `#rrggbb`. Each keeps its progress in widget memory, so a snapshot
restores it mid-way.

Generic component documents live beside the screens in
`packages/engine/src/ui/screens/`, each with props, actions and a default
fixture: `card`, `confirm` (a modal with cancel and confirm, danger when
`$danger`), `pagination`, `data-list`, `key-values`, `table`, `tree`,
`calendar`, `objective-tracker`, `star-rating`, `banner`, `subtitles` (HUD
layer) and `notifications` (toast layer). The settings screen builds its
tree from `$pages` (`{ id, name }`, a tab each), `$entries` (`{ id, page,
name, description, kind, value, … }`, kind `toggle`, `range`, `select` or
`binding`, written back as changed), `$page` and `$pending` (Apply and
Revert take presses only then); with `$page` empty it shows its `tree`
content alone. Use and ejection: §5.

#### Tooltips and popovers

`title` is the element's tooltip, shown after the pointer rests, and its
accessible description. `popover` (`auto`, `manual`, `hint`), its
invokers (`popovertarget`, `popovertargetaction`, `interestfor`) and
light dismiss follow HTML. An open popover sits in a layer over the page,
placed by CSS anchor positioning against its invoker; with no placement it
hangs under the control (above it with more room, kept on screen). It is a
raised panel (`--panel-raised`, the panel radius; a `hint` the tooltip's)
that its CSS overrides, and `::-og-arrow { width; background }` draws on
the side facing the anchor. Its content keeps HTML margins.
`contextmenu="id"` (engine attribute) opens popover `id` at a secondary or
long press, anchored at that point with no arrow. A `<button>` centres its
label by `text-align: center` as HTML's UA sheet does. A popover is built
every frame; whether it is open is snapshotted.

#### HTML blocks and lists

Layout follows CSS Display 3, CSS 2 block flow with margin collapsing
(§8.3.1), CSS 2.1 §10.8 line boxes, CSS 2 §17 tables with HTML's table
model, Position 3, Writing Modes 4, Anchor Positioning 1, floats (CSS 2
§9.5) with Shapes 1 contours, Multi-column 1, Lists 3, Generated Content 3,
Values 4 and 5, Sizing 3, Flexbox 1 and Grid (with `subgrid` and
`masonry`). HTML's UA rules come first in every cascade: list indents and
markers, the heading steps above, and block margins (`<p>`, `<ul>`, `<ol>`
1em, nested lists none; headings 0.67em to 2.33em of their step). The
engine differs or limits as follows:

- **Display.** Only boxes take every `display` value; other elements take
  `none`, `block`, `inline`, `inline-block` and `flow-root`. An
  inline-level element is atomic: an `inline` box's text does not break
  across its parent's lines, while inline markup inside a text (`<b>`,
  `<span>`, `<a>`) flows as runs. Text between elements is an anonymous
  text element; `i18n` on a box translates its one run. A block's auto
  width fills its block parent; a document root fills the column or stack
  it is mounted in, never widened by a child's min-content. A stack places
  its children without margins.
- **Tables.** `border-spacing` and `border-collapse` are not supported:
  cells are separate and `gap` spaces them. A table part outside a table
  lays out as a block; a `table-column` px `width` is a least width. A
  cell centres its content unless `vertical-align` is `top`, `bottom` or
  `baseline`.
- **Position.** A fixed box is placed in the whole surface and keeps its
  static position on an axis with no inset; an absolute one with no inset
  on an axis stands at its parent's start there.
- **Direction.** A document root takes the surface's direction (the
  locale's, or `og.ui.configure`); a right-to-left surface mirrors the
  document whole, its physical sides and transforms keeping their places.
  Each element's text is its own bidi paragraph (`embed` reads as
  `isolate`); with no declared direction a paragraph takes its first
  strong character's.
- **Writing modes.** SVG's `lr`, `rl`, `tb` values are accepted. An
  element whose mode differs from its parent's lays out in a frame of its
  own (an orthogonal flow sized from its containing block, else the
  viewport). Upright glyphs take the face's `vmtx` and `vert`/`vrt2`
  forms, which the cook keeps for every face in the chain; without them
  brackets and dashes turn and the ideographic comma and full stop move to
  the em's upper right. Selection, `::selection`, shadows, glow and rich
  text work in vertical lines. Text on a path keeps its physical box. A
  container variant (`Query::container`) reads a vertical element's
  height.
- **Anchors.** Anchors are this frame's boxes; the winning
  `position-try` option and its side are kept per frame for a popover's
  arrow.
- **Floats.** No `float` in flex, grid, table or multi-column containers.
  `shape-outside: url()` reads the texture's alpha on the CPU when the
  document loads: any texture natively, a lossless one (`kind pixel`) on
  the web; an unreadable one keeps the float's box, with a note.
- **Columns.** Flows break between children and between a text's lines,
  with `orphans` and `widows` at 2; `break-inside: avoid` keeps a text
  whole; every other box is monolithic. 3D `column-rule` styles draw
  solid. A split text's box is its first fragment.
- **Lists and generated content.** An `outside` marker hangs half an em
  before the item and costs it no room. Disc, circle and square markers are
  drawn shapes; `list-style-image` draws a registered picture one em tall.
  An item's number comes from the sibling index the structural selectors
  keep, so a keyed reorder renumbers only moved items, O(log n) per edit.
  `::before`/`::after` on a text element are runs of its text (text
  properties only); on any other element they are real boxes (`inline` by
  default) that take every box property. A pseudo-element is a synthetic
  node made at cook: it is never a child to `:nth-child`, `:empty` or `+`,
  adds no binding and exists only while `content` makes it. Counter
  styles are the predefined ones plus `@counter-style`. Counters run in
  document order; a retained subtree replays its counter effects and
  rebuilds only when counters enter it differently. `::marker` takes
  `content` and `color`; `::first-line` rebuilds its element every frame;
  `::placeholder` takes `color`; `::selection` takes `background-color`,
  `color`, `text-decoration` and `text-shadow`, its glyphs redrawn clipped
  to the selected runs (without its own `text-shadow`, selected text keeps
  the text's drop). `::file-selector-button` styles nothing. The CSS 2
  single-colon spellings work.
- **Units.** Physical units are 96 px to the inch. Viewport units read the
  play surface; the small and large forms read it with the browser's
  controls shown and hidden (`Viewport.smallWidth`…), which the web shell
  measures from `100svh` and `100lvh` on resize; natively all forms are
  the surface. `rem` and the root font units read the body type step; `em`,
  `lh`, `ch`, `ex` (0.5em), `cap` and `ic` read the element's step (else
  the body's) and the body face's metrics. All become px when the element
  resolves, except a margin's `em`. A `calc()` of lengths reduces to a sum
  of `%`, px and em terms that layout resolves; `min()`, `max()`,
  `clamp()`, `round()`, `mod()`, `rem()`, `abs()` and `hypot()` mixing `%`
  with px are evaluated at layout and must stand alone as the value; other
  functions refuse mixed units. NaN is 0 and infinity the largest value.
  `var()` substitutes first, so a `style:--w` store moves only its element;
  viewport and container unit readers resolve again on resize. Container
  units with no container are viewport units. `attr()` takes Values 5's
  types; `data-*` attributes are kept for it and for selectors.
- **Safe area and HUD band.** The engine root pads normal-flow content by
  `Viewport.safe_area`; `env(safe-area-inset-*)` measures from the
  viewport edge as the spec says, so adding it in normal flow counts the
  inset twice (a 47 px inset under 8 px padding starts content at 55 px;
  adding the env starts it at 102 px). A full-viewport page uses
  `position: fixed; inset: 0` and applies the env values once. The host's
  covered region (`Viewport.covered`: a published page's round Back, top
  left) folds into the one edge whose inset clears it for the least
  surface: on landscape or desktop it widens the left inset, on portrait
  it deepens the top; plain normal-flow padding clears only the hardware
  safe area. `env(-og-hud-inset-*)` are those insets with left and right
  grown alike until the band is at most `UiDesc.hudMaxAspect` (21:9 by
  default; 0 the whole safe area) times its height, centred. A HUD is
  placed at the HUD insets (`left: calc(env(-og-hud-inset-left) + 16px)`);
  full-screen art and dialogs read the safe-area insets. A change to any
  of these re-resolves only their readers.
- **Sizes.** Every element is `box-sizing: border-box` by default (the
  common reset as UA rule). `stretch` is the room the parent leaves. A
  percentage bound against an unknown size is ignored. A flex item without
  `flex` properties keeps its kind: `width: stretch` grows and shrinks, a
  fit-content item shrinks to its content, a definite one neither. `order`
  keeps the document's reading order.
- **Grid.** One `repeat(auto-fill | auto-fit, …)` of fixed sizes per axis.
  A masonry item keeps the width of the track it is first given.

#### Borders, shadows, outlines and overflow

Borders, outlines, radii and shadows follow Backgrounds and Borders 3 and
`corner-shape` from Borders 4; overflow follows Overflow 3. A border pads
the box (border-box sizing); an outline takes no room, so `:focus-visible
{ outline }` replaces the core focus ring for that element. Differing
sides draw as strips; a uniform `dashed` border or outline on round
corners is dashed as browsers do, half a dash at each corner. One uniform
px radius is the element's `radius`, which states and transitions animate.
A `<button>`'s own box declarations are its plate part's (§3.2).

Text and fonts follow Fonts 4, Text 3 and 4 and Text Decoration 3, beside
the house form `font: var(--text-<step>)`. A generic family picks the
style's body, display or mono role. `text-transform` cases for the UI's
locale. `text-wrap-style: balance` breaks at the narrowest width keeping
the line count, `pretty` at the widest that keeps it with no one-word last
line, `stable` is `auto`. `hyphens: none` drops soft hyphens.
`-webkit-text-stroke`, `-webkit-text-fill-color` and `paint-order` outline
and fill glyphs. Inherited properties pass down as custom properties do,
re-resolving only their readers. `accent-color` inks switches, sliders,
radios and progress; `caret-color` a field's caret; `color-scheme` picks
`light-dark()`'s side.

Colours take all of CSS Color 4 and 5 (`crates/og-ui-core/src/style/css_color.rs`);
out-of-gamut colours are mapped by OKLCh chroma reduction. A colour that
reads no role, `currentColor` or `light-dark()` reduces to sRGB at cook;
others resolve against the active style when drawn. One parse
(`style_file::parse_color`) reads every hex literal.

Transforms follow Transforms 1 and 2 (individual properties first).
`transform-box` `content-box` and `fill-box` are the content box, others
the border box. An element whose plane leaves the screen's draws itself
and its subtree into a layer through its projective map (perspective-correct
text; hits through the inverse). A `preserve-3d` context draws its planes
back to front by centre depth; intersecting planes are not split.

CSS paint follows Backgrounds 3, Images 3 and 4, Masking 1, Shapes 2
(`shape()`), Filter Effects 1 and 2 and Compositing 1, with these engine
rules and limits:

- `url(<file>)` resolves beside the stylesheet that wrote it (inline styles
  beside the document) and names a cooked texture (SVG included) the
  document load registers; it never selects an atlas icon or a numeric
  handle. A missing image warns. Monochrome icons are `mask-image: url()`
  over `background-color: currentColor`.
- Gradients take at most 8 stops, mixed in OKLab. A gradient layer is a
  tile of its concrete size (`UiPaint::GradientImage`); each layer is one
  image fill whose size, placement and repeat og-render resolves per pixel
  from the texture's own size (`UiFit::Background`), so a host never tells
  the UI an image's size.
- `image-set()` takes the lowest density at least the DPR, else the
  highest; `x`, `dppx`, `dpi`, `dpcm` (with math), `type()` before or after.
  Unsupported MIME types are dropped before density deduplication and are
  not cook inputs; malformed MIME strings drop their candidate; nested sets
  are refused; zero density paints nothing; an unusable set in a mask is
  transparent black. The chosen density divides natural size (mask tiles
  and automatic border-image widths too; slices stay in source pixels).
  Images reselect when the DPR changes.
- `background-clip: text` paints the layers through the glyphs' coverage.
  A `fixed` layer's element repaints when it moves.
- `border-image` draws through `UiFit::BorderImage`; masks and
  `mask-border` combine through og-render's Porter-Duff blends.
- `clip-path` on a CSS box treats `fill-box` as the content box and
  `stroke-box`/`view-box` as the border box; `url(#id)` names an inline SVG
  `<clipPath>`; clipping is by analytic path coverage.
- `filter` functions are layers in order; a run of colour functions is one
  matrix, split where a value can leave 0..1, so CSS's clamps hold.
  `url(#id)` names an inline SVG `<filter>` (below); a `url()` naming none
  drops the list.
- `backdrop-filter` captures what is drawn under the element in its
  backdrop root (the screen, an ancestor's group, or a world panel's,
  ground's or 3D plane's layer), over the border box grown by three
  deviations of each blur and the shadows' reach, clamping reads at the
  capture's edge. Its blur is an exact Gaussian; the theme's
  `-og-effect: backdrop` glass keeps the pyramid.
- `mix-blend-mode`: `multiply` and `screen` are fixed-function blends in
  linear light (a few levels darker than a browser's mid-tones) and
  `plus-lighter` is fixed-function; every other mode reads the backdrop and
  blends sRGB-encoded values. `filter`, `mask-image` and `-og-material`
  isolate as `isolation: isolate` does. A world panel's layer (§1) drops
  the blends that read what is under them.
- `-og-material: <name>(<params>…)` composites the element and subtree
  through a built-in layer material (dissolve, shine, hologram, glitch,
  ripple, pixelate, color-adjust, outline, blur) or a custom one
  (ui-materials guide), groups of up to four numbers or a colour in the
  material's order; no CSS standard has shader materials.
- Paint changes repaint only their element; an idle retained element
  builds nothing.

Overflow and scrolling: `scrollbar-width: thin` is half width with a
half-width grab strip; `scrollbar-color`'s track runs the whole edge
(`auto`: the style's thumb, no track). Scroll Snap 1 applies to drags,
flings and wheels; `proximity` snaps within a third of the view;
`scroll-snap-stop: always` stops a wheel at the first such area past where
it started (saved UI state). A fling is steered onto its target exactly.
`overscroll-behavior` `contain` and `none` act alike (og-ui draws no
glow or bounce). `scroll-behavior: smooth` glides a scroll into view as a
fling; reduced motion jumps. A modal `<dialog data-detents="40% 90%">` is a
bottom sheet (full width, opened at its first detent, dragged between
detents, swiped down to close, writing `open` back); `data-edge=
"inline-start" | "inline-end"` docks it to a side, its detents widths. The
attributes control gestures; CSS controls looks.

#### Components in documents

Built-in components are documents: a game uses one by the name its
`<meta name="component">` declares, and a game's own component is a
document of the same form (§5). Game code reaches the UI through
`@origingame/engine` only; private source imports, casts that evade the
API and game-side Rust are not extension points.

Every component group declares its members' input and output types,
semantic parts, state combinations, documents, and interactions before
implementation. An intent names the affected item or command by stable ID,
never a row index; the game decides whether it is legal and applies it.
UI state lives in scope sources and core interaction state (§2.1), never
module globals: two worlds cannot share selection, reordering or removing
data cannot select a different object, and snapshots keep what an
interaction needs to resume.

Core owns buttons, fields, toggles, tabs, sliders, menus, selection,
drag-and-drop, scrolling, focus, and keyboard and gamepad navigation;
components compose them rather than draw substitutes. Generic meters,
counters, timers, slots, lists, grids, tables, charts, tooltips, cards,
rewards, confirmations, and node graphs stay in core. Domain names do not
justify duplicate controls.

### 2.1 Retained authoring model

The engine implements the `svelte/store` model (`writable`, `readable`,
`derived`, `get`) plus a keyed collection store, named within a scope so
they can be checkpointed. Rust owns the graph on both hosts; no Svelte
runtime or compiler runs in the game. Data authoring is function calls;
structure stays in templates. There are no async primitives, Promises,
resources, Suspense, or async transitions.

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
that component; its attributes are props evaluated in the user's context
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
  skips shared elements unless the style keeps motion. A component cannot
  declare a view transition. Capturing costs nothing until a transition
  starts; a running one is saved with the UI state.

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
  on `(x, y)` by `text-anchor` and `dominant-baseline` and scaled by the
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
  a side (wider ones run as σ/√n passes), `feImage` of an element nests
  four deep and refuses the filtered element or its ancestors,
  `BackgroundImage`, `BackgroundAlpha`, `FillPaint` and `StrokePaint` read
  transparent black. A CSS-box filter region defaults to the border box
  grown 10 % a side, in its CSS px.

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
  `@media (pointer: fine)`. They are written only while touched.
- `<anchor entity|at offset occlusion declutter clamp priority
  max-distance scale-at>` holds a column over a world point, placed every
  tick; a moving camera builds nothing.
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
  when a corner is behind the camera). It is inert.

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

### 2.3 Text input types

`<input>` takes HTML's `type` (`text`, `password`, `number`, `email`,
`url`, `tel`, `search`), `inputmode` and `autocomplete`; `FieldDesc`
carries the same three. Validity follows the HTML Living Standard,
including `required`, `minlength`, `pattern` (compiled as `^(?:pattern)$`;
an invalid pattern is a parse error), `min`, `max` and `step` (default 1,
`any` off); an empty field is never a type mismatch. Any failure matches
`:invalid` in retained and immediate drawing. A number field types only
digits, a sign, a point and an exponent. `maxlength` stops typing;
`readonly` drops every edit and keeps selection, copy and the focus stop.
`<textarea rows>` shows that many lines (0 is 4).

A password draws one bullet per character (`*` where the face lacks
U+2022); the probe and accessibility tree read the mask; copy, cut and the
kill buffer never take it, the host gets no selection, and word moves treat
it as one word. Its reveal toggle `<id>/Reveal` sits at the field's end,
dressed `<component>.reveal[revealed]` with a `reveal-icon` subpart, named
Show or Hide password, keeping the text focus; revealed is saved UI state.

A field's `::part(frame)` is a box over its padding box that draws the
text and takes padding, border, background, radius and states (without its
own padding it keeps the control's inset); the control keeps size, focus,
editing and role. `::part(reveal)`, `::part(clear)` and a number's
`::part(button)` (`.inc`, `.dec`) are boxes filling their buttons.

A text input focused by a pad opens og-ui's on-screen keyboard, docked at
the bottom over a scrim: the pad walks its keys (Y erases, LB and RB move
the caret, Back closes), shift locks on a quick second press, and Done (or
Back) returns focus to the input, which reopens it only after losing
focus. `type` and `inputmode` choose its layout (`numeric`, `decimal`,
`tel` the number pad; a `password` PIN digits only), `enterkeyhint` labels
Done, `maxlength` counts down, and `lang="ja"` offers kana composition
with voicing marks and candidates. `inputmode="none"` opens none. Parts:
`Keyboard.panel`, `.preview`, `.key[wide][action][active][primary]`,
`.candidates`, `.candidate`.

The host receives type, effective `inputmode`, `autocomplete` and `secret`
with the focused target; the web shell stands a hidden `<input>` (or
`<textarea>`) with them so phones show the right keyboard and autofill; a
number uses a text input with the decimal keyboard.

#### Selectable text

`user-select` (default `none`; a control's subtree stays `none` unless it
sets it) makes plain text elements and vertical rich text selectable;
horizontal rich text and text controls keep their own selection. A drag
selects, a double press a word (UAX #29), a triple press a line, Shift
extends; on a coarse pointer a 500 ms long press selects a word and an
immediate drag scrolls. A drag crosses elements in tree order. Ctrl+C
(Cmd+C under mac keys) copies unless a text control has focus, joining
elements with line breaks and restoring what wraps dropped. The fill is
`TextEdit.selection`; the pointer is an I-beam. The selection is saved UI
state, read by `dump_ui` as `selection`; only changed ranges repaint.

#### Pointer, touch and visibility

`cursor` takes every CSS UI 4 keyword and `url()` images (a registered
texture, cooked SVG included; natively any, on the web a lossless `kind
pixel` one), tried in order, larger than 128×128 or unreadable falling
through with a note; `og.ui.frameCursor` answers it, and the frame answer
carries `cursor` and, when it changed, `cursorImage`. The web shell sets
the canvas cursor; Origin Desktop sets the OS cursor from the `Cursor`
message (docs/design.md §2.6). At most eight decoded images are kept. A
`clip-path` bounds where an element's cursor applies; presses still hit
its box.

`pointer-events` follows CSS, and inline SVG shapes take SVG's values
against their geometry. `touch-action` (Pointer Events 3) applies on a
coarse pointer, intersected with ancestors'. `visibility` follows CSS;
the `visible false` prop removes an element from layout. These are read
from the frame's orders, so a declared value costs nothing while idle.

#### Containment, resize and will-change

`contain` and `content-visibility` follow Containment 2. A skipped
subtree is left out of the layout walk and costs nothing while idle; an
`auto` one skips from the frame after it is found more than half a
viewport outside its clip with no focus inside, stays in the
accessibility tree, and keeps its last laid-out size under
`contain-intrinsic-size: auto`. `resize` (CSS UI 4) draws a three-row dot
handle in the style's strong control ink; a drag sets width or height in
CSS px within its bounds, presentation state (not saved) written only
while dragged. `will-change` on a stacking property draws the element as
its own group layer.

#### Images

`object-fit` and `object-position` follow CSS Images on `<img>`; on
`<art>` `object-fit` also keeps `nineSlice` and `tile` and
`object-position` replaces the spec's focal point. The renderer reads the
region's size from the texture. `image-rendering: pixelated | crisp-edges`
(and legacy `optimizeSpeed`) samples every image the element draws at the
nearest top-level texel; `auto`, `smooth`, `high-quality` keep the
texture's sampling.

#### Text on a path

`-og-text-path` sets an element's text along a path as SVG 2's
`<textPath>` does (CSS `offset-path` moves the element, so it is not
reused): each grapheme cluster sits centred on the path, turned to it.

```css
.title { -og-text-path: path("M0 90 Q170 -20 340 90"); -og-text-path-offset: 50%; text-align: center; }
.seal  { -og-text-path: circle(52px); -og-text-path-side: right; }
```

- `-og-text-path`: `none`, `path("<SVG path data>")` in CSS px from the
  content box's origin, or `circle([<radius>])` centred in the content box
  from its top, clockwise (default radius the closest side). `ray()`,
  `inset()` and other offset shapes are errors.
- `-og-text-path-offset`: the anchor's place in px or % of the length;
  `text-align` is the anchor (`start`, `center`, `end`).
- `-og-text-path-side: right` runs the path backwards.

The text is one line (breaks are spaces); a cluster off an open path is
not drawn and a circle wraps. An element with no size of its own is its
path's extent. Path data flattens once per string; an unchanged path text
does not repaint. It is not selectable; its accessible name is its text.
TypeScript's `textPath` is `{ data, radius, offset, percent, reverse }`
(empty `data` a circle; unparsable data keeps lines). References:
`text-path-1280`, `text-path-390`.

#### Live regions

`aria-live`, `role="status"`, `"log"` and `"alert"` and `aria-atomic`
follow ARIA with the default `aria-relevant` (`additions text`): entering
nodes and changed names or values announce; removals, emptied text, a
region arriving with content, a hidden ancestor, the first tree a host
reads and the update after a restore announce nothing. A toast
(`og.ui.toast`, marked `notify`) announces as it appears. Each change
announces once from the tree the host last read. Natively the AccessKit
bridge (`native_a11y::Bridge`) speaks through two polite and two assertive
hidden live nodes, alternating so repeats are heard; `take_announcements`
holds at most `ANNOUNCEMENTS` (16), dropping the oldest. The web mirror
writes the frame answer's `announce` list into the page's regions. Both
work in publish builds; `og.ui.a11y()` gives each node `live` and
`atomic`. Announcing reads only changed nodes and their ancestors to the
region (O(depth)); an idle frame reads nothing. The queue is presentation
only, never checkpointed. Under `OG_UI_CHECK_BOUNDED` the whole-tree twin
of each update must announce the same.

```svelte
<text role="status">{$saved}</text>
<column aria-live="polite" aria-atomic="true"><text>Gold</text><text>{$gold}</text></column>
{#each $alerts as a (a.id)}<text role="alert">{a.text}</text>{/each}
```

### 2.4 Select, dialog, details and meter

These are core controls with HTML's elements and attributes; components
and packs dress them and never draw substitutes.

`<select bind:value>` lists literal `<option>`s in at most one level of
`<optgroup>`, valued by text when `value` is absent; `selected` and
`disabled` follow HTML. A bound list is `items={$labels} values={$ids}`;
`{#each}` inside is refused. The list opens over the page at the chosen
option; press, Enter, Space or pad A opens and picks, arrows skip disabled
options, letters typeahead, Escape, pad B or an outside press closes, and
focus returns to the trigger. The value, never the label, is written.
`select::picker(select)` styles the open list's box over `Dropdown.list`
(its `max-height` bounds the rows, which scroll). Parts:
`Dropdown.trigger[open][invalid]`, `Dropdown.list`, `Dropdown.group`,
`Dropdown.option[selected][disabled]`; roles combobox, listbox, option.

`<dialog bind:open aria-label>` follows HTML. `modal` (a literal, never
bound: a modal's content is built in the top layer under `<id>/Panel`) is
`showModal()`: a scrim taking every press, focus moved in, trapped and
returned to the opener, the panel centred in the viewport.
`dialog::backdrop` is the scrim's box (default background the style's
`scrim`) and takes every box property; to a style pack it is
`Dialog.backdrop`. Without `modal` it is `show()` in place
(`Dialog.panel[open]`). `closedby` follows HTML: `closerequest` (a
modal's default) is Escape, Back or pad B, for an in-place dialog only
with focus inside; `any` adds a scrim press; `none` (an in-place dialog's
default) leaves closing to the document. Closing writes `open` false.

A `<small>` beside a select or input with `aria-invalid` is its message;
`aria-invalid` puts the danger frame on the control.

`data-og-instant` on an element lands its first entrance's
`@starting-style` at once (the screen behind the web's play gate); a later
entrance plays.

`<details bind:open><summary>` is a disclosure: the summary
(`Details.summary[open]`, role button with `expanded`) toggles it by
press, Enter, Space or pad A; its marker points to the inline end closed
and down open. A literal `open` is the initial state; the player's toggle
is saved UI state. `<details name>` is an exclusive group. A closed dialog
or disclosure builds no content: its children are released and built
afresh when it opens.

`<meter>` dresses its fill by HTML's region, `Meter.fill[region=optimum |
suboptimum | poor]` (undressed: success, warn, danger), over `Meter.track`;
`low` and `high` clamp into the range, `optimum` defaults to the middle.

### 2.5 Split screen

`og.ui.setPlayers(n)` turns pads 1 to 3 on; each pad moves its own focus
(`Response.focusedBy`, bit `k`) and presses with A (`Response.pad`).
`player="k"` (Rust `Props::player`, 0 to 3) makes a subtree player `k`'s
region, which may be its own scene viewport.

- Pad `k`'s focus moves only within player `k`'s region; its first move
  lands nearest the region's centre. Pad 0's focus is also the keyboard's
  and pointer's.
- Controls outside every region belong to pads with no region; an open
  dialog's controls are reachable by every pad.
- `Response.player` names the region a control was pressed in, null
  outside every region and on unpressed ticks; the pointer presses
  anywhere.
- A screen with no region is unchanged: every pad sees every control.

A hit carries its region as one byte, inherited like the focus group and
computed when ancestry changes; it is saved with the UI state.

## 3. Style contract

### 3.1 Tokens and literal values

Built-in components, screen documents and a style pack's part rules take
every visual value (colour, typography, spacing, size, radius, stroke,
shadow, feedback, motion) from semantic tokens or named parts. A game that
customises them defines them in its style, not across widgets. This
section is the one statement of that rule; others link here.

Literal values belong only in:

- Core default definitions (`packages/design/tokens/*.css`) and a style
  pack's `:root` token rules (`:root[data-scheme=light]` among them), art
  URLs included.
- Art metadata: atlas coordinates, slice insets, resource dimensions,
  import settings.
- Fixture data: health, elapsed time, scores, item counts.
- Structural geometry: ratios, normalized positions, column counts, bounds
  derived from caller data. Reviewers check the purpose; a structural
  label cannot disguise visual tuning.

Hex colours, font paths, arbitrary pixel spacing and hard-coded durations
do not belong in a component's or screen's `<style>`. `u(17)`, `ts(15)`,
tokens times arbitrary constants and radius clamps with literal minima are
not semantic tokens, and a token per arbitrary number does not meet this
contract. When the API cannot express a semantic value, the core owner
fixes the API before a component depends on it. A pack's `::part` and other
rules read its tokens with `var()`; `og style check` fails a declaration
outside a `:root` rule that uses a literal colour, a length other than 0, a
duration or a `url()`.

The default stylesheet (`crates/og-ui-core/default.css`, generated from
`packages/design/tokens/*.css`) sets every value the core controls and
built-in components draw with as a `:root` variable, and the theme reads
them back each frame, so a pack restyles every control by setting tokens,
dressing `::part`s and supplying assets. A control token is
`--<control>-<property>` and defaults to a semantic token or the control's
number.

| Token family | What it controls |
|---|---|
| `--color-*`, the role names (`--ink`, `--primary`…) | Colours, per scheme; `--color-track`, `--color-minimap`, `--color-minimap-edge`, `--color-slot-selected-edge` for the engine's own parts |
| `--space-*`, `--radius-*`, `--size-*`, `--stroke-*` | Spacing, corners, icon and art sizes (`--size-xs` to `--size-7xl`; a layout takes a step, never a multiple of one), line widths (`--stroke-edge` the material hairline) |
| `--type-*`, `--text-*`, `--font-*` | The type ramp and its roles; `--font-leading-*` (`flat`, `tight`, `snug`, `normal`), `--font-tracking-*` and `--font-weight-*` for text a part sizes itself; `--font-step-2xs` to `--font-step-6xl` are sizes in `em` of the surrounding text, so a part sized with one follows the player's text scale |
| `--density-compact-*`, `--density-pad-*`, `--density-touch-*` | Each input's control, row and header heights, gutter and section |
| `--control-sm/md/lg`, `--row`, `--header`, `--gutter`, `--section`, `--touch-target` | The input's density, and the smallest finger target |
| `--motion-duration-*`, `--motion-easing-*`, `--motion-named-*`, `--motion-scale-*` | Durations, the controls' enter and exit curves (`in-out` for what loops), the named transitions' springs (`ui`, `panel`, `emphasis`, `playful`, `enter`, `exit`), how far a moment scales (`press`, `swell`, `pop`, `shrink`) and how low a waiting thing breathes (`breathe`) |
| `--state-*` | Hover wash, press scales (`press-scale` a button's part, `control-press-scale` a control row's), lift, disabled opacities (`touch-disabled-opacity` the stick, pad and action button), press duration, focus ring and player rings |
| `--opacity-*`, `--filter-*` | How strongly a thing shows (`faint`, `dim` what cannot be used, `muted` what is done, `soft`), and what art takes out of reach (`--filter-locked`, `--filter-spent`) |
| `--shadow-*`, `--elevation-*`, `--glass-*` | Shadows per scheme and the controls' inset shadows, the two drop elevations, glass blur and saturation |
| `--frame-*`, `--plate-*`, `--band-*`, `--title-*`, `--row-*`, `--cell-*`, `--badge-*`, `--well-*`, `--figure-*`, `--label-*`, `--ink-label/bright/done/disabled/selected/down/mark`, `--trend-*`, `--tone-*`, `--mark-*`, `--sfx-*` | What the built-in components' role parts wear (below) |
| `--button-*` | Radius, border width, rest and pressed shadows (stepper, details, keyboard keys and option rows take them) |
| `--field-*` | Radius, border width and well shadow of text boxes, triggers, steppers, checkbox boxes and radio rings |
| `--checkbox-*` | Box size, radius and mark ratios, the mark's pop spring and its exit |
| `--radio-*` | Ring size, dot ratio, pop spring; `--radio-card-*` a card's radius and shadow |
| `--switch-*` | Track heights by size, width and inset ratios |
| `--slider-*` | Knob and rail by pointer, tick width, value bubble height and exit |
| `--segmented-pad-ratio`, `--tabs-radius`, `--chip-radius` | The segmented track's inset; tabs, badges and chips |
| `--dropdown-*`, `--menu-open-duration` | Trigger and option rows, the width before layout, the list's shadow; a menu's opening |
| `--details-icon`, `--modal-width`, `--panel-*`, `--hud-radius` | The disclosure icon, a modal's width, panels and lists, HUD frames |
| `--slot-*`, `--meter-*` | Slot radius and well; meter and progress radius, groove and fill sheen |
| `--keyboard-*` | Key height, its floor and the candidate row |
| `--joystick-size`, `--dpad-size`, `--action-size` | The touch controls' default sizes |

Icon path coordinates and behaviour constants (key repeat, dead zones,
caret blink, typewriter speed) are not tokens.

**Role parts.** A built-in component names a part by its role
(`Shop.frame`, `Friends.row`, `Chat.title`), and the default style
dresses every part of a role once, from that role's tokens
(`packages/design/tokens/role.css`). A pack restyles all of them by
setting the tokens; it writes a rule only for what one component alone
wears. The role rules sit in `:where()`, so any pack or document rule wins.

| Role (`part$=".<role>"`) | Tokens | States the default dresses |
|---|---|---|
| `frame` | `--frame-fill/rim/rim-width/radius/shadow/pad/corner-image/corner-size` | `class="hud"` wears the plate (`--plate-fill/rim/rim-width/radius/shadow/pad`); `class="bare"` wears nothing (a frame that only lays its children out) |
| `band`, `title` | `--band-fill/height/pad/rule/rule-width/rule-image/shadow/end-start-image/end-finish-image`, `--title-ink/font/size/weight/tracking/transform/shadow` | `class="lead"` and `class="small"` bands (`--band-lead-fill`, `--band-small-*`, `--title-lead-size`, `--title-small-size`), which set `--title-size` for the title inside them; one step down on a phone on its side |
| `row` | `--row-fill/hover-fill/pressed-fill/selected-fill/selected-edge/radius` | hover, pressed, selected (`aria-selected`, `aria-current`, `data-selected`), disabled, `data-done` |
| `cell` | `--cell-fill/rim/rim-width/selected-rim/radius/shadow/hover-shadow/pressed-fill/empty-shadow`, `--cell-<rarity>-fill/shadow` | hover, pressed, selected, `data-rarity`, `data-empty`, disabled or `data-locked` |
| `badge`, `well`, `figure` | `--badge-fill/ink/radius/shadow`, `--well-fill/radius/shadow`, `--figure-ink/font/shadow` | |
| `label`, `heading` | `--label-font/tracking/transform`, `--ink-label` | |
| any part | `--trend-up/down`, `--tone-damage/heal/crit` | `data-trend`, `data-tone` |

`--mark-*` are the small signs a pack supplies as images and `--sfx-*`
its press sounds; both are `none` in the default style.

### 3.2 Parts and states

Style selectors dress a component's named parts with their variants,
states and attributes. The pack grammar lives in
`crates/og-ui/src/style_file/css.rs` and document CSS in
`crates/og-ui-core/src/css/values.rs`; generated references come from them.
Parser support for a literal does not override §3.1. A document's
`<style>` controls appearance and layout, attributes content and
behaviour; rules are document-scoped and never affect unrelated mounts.

**Selectors and cascade.** Selectors follow Selectors 4: every combinator
and attribute operator with `i`/`s` flags (HTML's case-insensitive
attributes compare so unless `s`), `:not()`, `:is()`, `:where()`, `:has()`
and the structural, form, `:open`, `:modal`, `:popover-open`, `:target`
(the element the last `#fragment` link named), `:lang()` (RFC 4647) and
`:dir()` pseudo-classes. Engine meanings and limits:

- `lang`, `dir`, `data-*` and `state:name={…}` are selector-only
  attributes; `:state(name)` reads `state:name`; `:defined` matches every
  element, `:scope` the root, `:any-link` an `<a href>`. An ancestor
  selector reads that ancestor's own interaction state.
- Structural selectors read the instantiated tree after `{#if}`, keyed
  `{#each}`, snippets and components (blocks are not siblings; `:empty`
  means no element children and no text) and never select `::part` spans.
  An edit invalidates only the sibling interval it can affect, O(log n)
  per interval; virtualized rows restyle when they materialize.
- `:has()` takes a leading `>`, `+` or `~` and does not nest; a change
  restyles only the anchors whose own facts match a `:has()` compound and
  the subjects such an anchor can reach.

The cascade follows Cascade 5 and 6 and CSS Nesting: `!important`,
`@layer` (UA rules below every layer), `@scope`, `@supports` (declarations
and `selector()`), and `inherit`, `initial`, `unset`, `revert`,
`revert-layer`. The engine's layer order is §5's. `@import "x.css"
[layer()] [supports()] [media]` is read at cook from beside the document
into its blob. `@property` registers syntax, inheritance and initial
value. `@counter-style` and `@position-try` feed counters and anchor
positioning.

`@font-face` (Fonts 4): the cook bundles each `url()` source, which must
be a project font cooked by its own settings; `local()` names nothing (no
system fonts). The runtime registers the first source the bundle has.
`font-family` tries named and generic families in authored order per
glyph, skipping unavailable names; generics map to the style's roles
(`monospace`, `ui-rounded`, `fantasy`/`cursive`, else body), and a quoted
`"serif"` is a named family. In a family the text takes the nearest
`font-style`, then `font-weight`, and per character the last-declared face
whose `unicode-range` has it. After the list come the role's face, the CJK
companion, the locale's and the game's fonts. A missing weight or slant is
synthesised. A `::part` run keeps the same list.

**Custom properties** are lexical and inherit through instantiated
parents, component mounts included; they resolve on the declaring element
(a child's `--a` does not recompute an inherited `--b: var(--a)`);
recursive `var()` takes fallbacks, empty ones too; cycles invalidate their
properties and report their names; an invalid winning declaration does not
revive a lower one. Unshadowed engine tokens resolve against the active
style. Data reaches the cascade through `style:--name={expr}`, an inline
declaration (a number plain, a string as is, `null` unset) that
re-resolves only its element and inheritors. `calc()`, `min()`, `max()`
and `clamp()` reduce once their `var()`s are known (at cook when constant);
`s`, `rad` and `turn` convert to `ms` and `deg`. None of this relaxes
§3.1.

**Media queries** follow Media Queries 4 and 5 with standard range
syntax, `and`, `or`, `not` and commas (mixed operators need parentheses;
nested rules intersect). Features: `width`, `height`, `aspect-ratio`,
`orientation`, `resolution` (the DPR), `prefers-reduced-motion`,
`prefers-color-scheme` (the theme's scheme), `prefers-contrast` (`more`
in high contrast; `less` and `custom` never match),
`prefers-reduced-transparency` (the theme's flag), `pointer`, `hover` and
their `any-` forms (the host's coarse pointer), `(-og-input: pad)` (a pad
is the device in use) and `-og-color-vision` (below). The display features
answer fixed values: `color-gamut: srgb`, `dynamic-range: standard`,
`display-mode: fullscreen`, `update: fast`, `scripting: enabled`,
`forced-colors: none`, `inverted-colors: none`, `prefers-reduced-data:
no-preference`, `color: 8`, and 0 for `monochrome`, `color-index` and
`grid`. There is no named-breakpoint reader. Each element re-resolves only
when an input it read changes.

**Container queries** follow Containment 3 (`size`, `inline-size`,
`scroll-state`, `anchored`; every named element is a style container).
Size conditions read the nearest eligible ancestor along its own writing
mode's axes (a block-axis condition needs a `size` container) and are
false before the first measurement. Core's settlement hook publishes
measured content boxes (zero included; unchanged ones do not notify)
before effects, hit testing and paint, on the first frame and every
resize; dependent elements re-resolve, relayout and replace their fields
in the same frame. A query that keeps resizing its own container fails as
a settlement cycle instead of publishing. `style()` compares custom
properties only (with collapsed whitespace; unnamed, the parent's);
`style(color: red)` fails at cook. `scroll-state()` reads `scrollable`,
`snapped`, `stuck` and `scrolled` from last frame's layout and restyles
its readers on the next frame. Other features fail at cook, named.

**Transitions and animations** follow Transitions 1 and 2, Animations 1,
Easing 2 (`linear()`), Web Animations' `animation-composition`, Values 4
interpolation and Scroll-driven Animations 1. Svelte's `class:` selects
states; it is not a second styling language. Engine rules:

- Motion runs on presentation ticks, never a clock, and is saved with the
  presentation state, so a restore continues exactly. A saved state names a
  `linear()` curve by its points' hash, so its document must define it
  again before a restore.
- Shorthands expand at cook; an animation starts when its rule begins to
  match and ends when it stops. Opacity, translate, scale, rotate,
  `border-radius`, background, colour and z-index move the element's motion
  channels (springs keep velocity on retarget); every other animatable
  property moves its computed value, and an element with one in flight
  builds each frame until it settles. `display`, `visibility`,
  `content-visibility` and `overlay` keep their shown value through a move
  to or from hidden. A property not declared transitions from its CSS
  initial value where one is fixed, else it jumps. `var()` inside a
  shorthand, a non-animatable property and unknown keyframes fail at cook.
- `@starting-style` gives a first style; an exit transitions to
  `display: none` with `allow-discrete` while the element stays mounted,
  its subtree retiring when display reaches none. Removing an `{#if}`
  branch is not a style change; use `out:` for that.
- `@keyframes` may nest in `@media` and `@container`: the last definition
  whose conditions hold plays, and a switch restarts it.
- Scroll-driven timelines read laid-out geometry each motion step (the
  subject's box without its transforms), sampling an element only when its
  progress moved; on a progress timeline durations and delays are
  proportions, play state is ignored and keyframe events do not fire. An
  inactive timeline has no effect.
- CPU motion, GPU mass elements and camera timing share `Ease`: GSAP names
  (`none`, `power1.out`, `back.out`), `cubic-bezier()` and `steps()`;
  style tokens keep `EaseRole` indirection; the transition timing slot
  also takes `-og-spring(<duration>, <bounce>)` and `var(--motion-*)`. CSS
  keywords stay the CSS parser's.

**Reduced motion** (the one statement; other sections link here): a
finite animation shows its end and an endless one stops; a transition
written under a `prefers-reduced-motion` media query plays as written;
any other transition takes the engine's fallback, in which movement,
scale and computed values jump while opacity and colour fade quickly.
Scroll-driven animations are left alone (the player drives them).
Feedback can be interrupted and retargeted, every motion has an explicit
reduced outcome, and decorative loops stop.

**Sound.** `-og-sound` is the one sound extension (CSS has no interaction
audio): `none | [ <event> [ <url>+ [volume(<n> <n>?)]? [pitch(<n> <n>?)]?
| none ] ]#`, up to four samples an event, volume an ordered range in 0..1
and pitch an ordered positive range (one number a constant range), e.g.
`-og-sound: press url(a.wav) url(b.wav) volume(0.28 0.34) pitch(0.98
1.02), hover none`. Sample choice never repeats the last and comes from a
presentation random stream seeded by the engine and checkpointed for
replay. Inside a `@keyframes` step `-og-sound: <event>` plays the style's
sound when playback crosses it, at the recorded `playbackRate`. Sound and
haptics come from the style's event definitions and respect player
settings.

**Parts.** A document names a part with `part` (`<div part="Killfeed.row"
class:self={row.self}>`); its classes, `aria-selected` (`:selected`) and
`aria-pressed` or `aria-checked` (`:checked`) qualify it
(`Killfeed::row.self`, `Loadout::item:selected`). A control keeps its own
variant and states (`Loadout::item.secondary:hover`), and its label's ink
reads against that dressing. Core parts keep their names and meanings.

- Each built-in component lists its parts in `<meta name="parts">` and its
  group's README documents them. A component names every frame, band,
  title, row and cell it draws as a part and selects them by
  `[part="…"]`; classes stay for inner structure and `data-*` for state.
  Its `<style>` holds structure only (layout, token sizes and spacing) and
  gives a frame at most a token background as fallback, since a
  component's own background, border image or border outranks the pack's
  dressing of that part. States reach the part as qualifiers, classes or
  `data-*` (`part={"Slot.frame[rarity=" + item.rarity + "]"}`,
  `RpgBackpack::cell[data-rarity=epic]`); the component draws no band or
  rim for them.
- A `<button>` is its plate: its own `background*`, `border*`,
  `padding*`, corner and `box-shadow` are the plate part's, above the
  style's `::part` rules and below the document's own
  (`button.buy::part(base):hover`), so the host draws no second box and
  keeps the corners for its clip and outline. An own `background` clears
  the style's `background-image` and an own border its
  `border-image-source` unless the author writes them.
- A missing dressing uses the readable core token fallback; it is not
  permission to leave a required state unfinished. Missing art draws a
  small mark fitted to the slot's shorter side, at most 32 UI units,
  centred in a quiet token well; the `icon` and `placeholder` parts stay
  stylable and the slot keeps its identity.
- Hover, pressed, focus-visible, selected, disabled, and selected+disabled
  are distinct. Selection never hides focus. Disabled controls emit no
  activation. State never depends on colour alone.
- Drawing order: ground and fills, rim, content and selection, hollow
  focus, then children and raised badges. Focus does not cut art with an
  opaque background; decoration over children is a separate child.
- Selection uses `selection`/`onSelection`, the main action
  `primary`/`onPrimary`, small emphasis `accent`; HUD content over the
  scene uses `sceneInk`, `scenePlate` and `sceneShadow` with `onScene`.
  Components choose no scene-separation colours of their own.
- Typography roles, breakpoint sizes and numeric treatment come from the
  style; ramp aliases define no other sizes; player text scale applies
  once after breakpoint selection; casing uses core text facilities.

**Colour vision.** `UiDesc.colorVision` (`protanopia`, `deuteranopia`,
`tritanopia`, default `none`) runs the whole frame through one final
filter (`crates/og-render/src/post/color_vision.rs`): Fidaner 2005
daltonisation by default, or with `colorVisionFilter: "simulate"` Viénot
1999 (protan, deutan) and Brettel 1997 (tritan) as libDaltonLens publishes
them (`third_party/daltonlens`). `colorVisionStrength` (0 to 1) blends
toward the unfiltered frame. A mode costs one full-screen pass; none costs
nothing; it is presentation only. Every status and rarity colour token
pairs with a cue (an icon, pattern or pips, distinct in its group) or the
style refuses to cook; team and damage colours may. `<og-icon cue="danger">`
draws the token's icon cue (`icon` wins); `og-icon::part(image)` and
`::part(secondary)` colour its layers. `@media (-og-color-vision)` and
`(-og-color-vision: deuteranopia)` let a pack add cues; a retained mount
restyles only the readers when it changes.

**Prompts.** Glyphs and prompts follow the active device and switch live
(keyboard and mouse, Xbox, PlayStation, Switch, generic pad, touch); core
input glyphs and action prompts own this, and components hard-code no
button letters. Actions declare `actionMap`; `og.input.setActionMaps`
selects maps; `isActionPressed` reads held state, `isActionJustPressed`
and `isActionJustReleased` tick edges; `actionHeld` keeps its `holdMs`
threshold; physical controls keep their W3C codes.

### 3.3 Assets and generation

Each pack dresses the core control set and the component parts its
acceptance samples need. Art, icons, fonts, import settings and licences
travel with their pack or the shared resource directory. Placeholder
characters are not finished icons. Shared resources and CJK fonts are
neither components nor styles, and core font fixtures never depend on a
commercial style pack.

`og style new <name> [--look <preset>] [--scheme dark|light] [--finish
<treatment>] [--out DIR]` writes a CSS directory (default
`assets/styles/<name>`, never overwritten) with both schemes, core-control
rules, transitions and shared `-og-sound`s. Finishes: flat, gradient,
glass, hologram, neon, paper, bevel, brush. `--scheme` picks the fallback
root palette; `data-scheme` still selects at run time.

`og style check <directory> [--json]` checks CSS and asset URLs, fails
literals outside `:root` (§3.1) and reports root-token contrast ratios as
facts (not over images or gradients). `og style check-project` is the
game's art-contract checker.

`og ui sheet <directory>` renders a pack's core-control pages at
desktopHd and phonePortrait at 100 % and 150 % text, scroll pages captured
to the bottom; `og ui review <directory> --locales game,pseudo,zh,ar` adds
locales. Both take `--out DIR` and `--json` and a group's sheet (§6).
Review frames before recording references. The
[screens guide](../crates/og-api/guides/screens.md#3-style-packs) shows
loading, selectors, tokens and asset URLs.

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
covered code points) beside its faces, cooked by their own settings to
`<id>.face.ogb` with no atlas (`runtime true`), so a pack costs about its
files' size. A pack changes no other font's cooked bytes, charset or
run-time mode.

og.i18n chains the active table's fonts, the locale's pack, the packs a
frame needed, then the other tables' fonts. A character only an unchained
pack covers loads that pack (a Chinese locale's own script first) and
draws from the next frame, unwarned; a character no pack covers keeps the
missing-glyph warning. The web boot does not preload `.face.ogb` files
(the service worker caches them on fetch); the first frame waits for the
locale's pack and glyphs (`og_web_fonts_pending`).

A run-time face renders each glyph's distance field once into 512² pages
off the frame (natively a thread per batch; on the web three glyphs a
frame per face), so steady frames cost what an atlas font's do. A glyph
is blank until the frame after it arrives, never warned of; a probe
capture waits for every requested glyph. Glyphs are presentation only.

Colour emoji are a font feature. The `emoji` pack is Noto Color Emoji's
COLRv1 face (SIL OFL 1.1; script `Zsye`, no language, tried after every
text pack), declared `color true` with `runtime true`: COLR v0 and v1
glyphs are painted when first shown by the cook's painter
(`crates/og-ui-core/src/text/colr.rs`) at `color_px` in the configured palette,
onto colour pages with a silhouette field for shadow, glow and outline;
CPAL 0xFFFF layers take the text colour at draw. GSUB forms skin tones,
ZWJ sequences and flags; spaces and punctuation after an emoji take a text
face's glyph. On the web a colour glyph counts as two of the three a
frame. The pack cooks to about 5 MB, paid only when installed; a game
showing a fixed handful of emoji lists them in `chars` with `color true`
instead. Any project font may declare `font { color true runtime true }`;
one with no COLR glyph is refused. References: `color-emoji-1280`,
`color-emoji-390`.

## 4. Shipped catalogue

Core default UI is the only creation `ui` choice until a pack is
accepted. Every component and screen document under
`packages/engine/src/ui/` is built into the engine (§5): the generic
components and screens, and the component groups below, each a directory
of `packages/engine/src/ui/components/`. The six style packs below are
approved scope. `fantasy` ships with the engine package
(`engine/styles/fantasy/`, with the art its CSS names), so a look's
`ui "fantasy"` resolves in an installed engine; it is not yet a creation
choice. The others are not released before their acceptance is complete.
`materials` is an optional resource pack for UI shader finishes, neither a
component group nor a seventh style. VFX and unrelated extensions are
outside this catalogue.

| Component group | Required presentation | Status |
| --- | --- | --- |
| `rpg` | Inventory and equipment, journal and tracker, dialogue, party and turn order, world map | Built, dressed through its parts; art re-review pending |
| `shooter` | Aim, ammunition, damage-direction HUD, killfeed, loadout | Built (pilot), not yet accepted |
| `strategy` | Selection and commands, production queue, waves, tower-upgrade inspection | Built, dressed in `fantasy` through its parts; not yet accepted |
| `racing` | Telemetry, laps and standings, race banner, garage selection, results screen | Built, dressed in `fantasy` through its parts; not yet accepted |
| `card` | Hand and piles, collection and deck building, reward-choice and run-map screens | Built, dressed in `fantasy` through its parts; not yet accepted |
| `puzzle` | Level path, goals and moves, level summary | Built, dressed in `fantasy` through its parts; not yet accepted |
| `idle` | Production and purchase rows, prestige preview, offline-summary screen | Built, dressed in `fantasy` through its parts; not yet accepted |
| `rhythm` | Song and difficulty selection, judgement presentation, results breakdown | Built, dressed in `fantasy` through its parts; not yet accepted |
| `arena` | Roster and character selection, draft, versus, team scoreboard, round HUD, lobby screen | Built, dressed in `fantasy` through its parts; not yet accepted |
| `general` | Shop, achievements, daily quests and rewards, chat, friends | Built, dressed in `fantasy` through its parts; not yet accepted |

Each group is built token- and part-only (§3.1, §3.2), with state
fixtures (§6.1) and a review sheet (§6.2). Member inputs, intents and
parts are documented in each group's README
(`packages/engine/src/ui/components/<group>/README.md`) and each
document's `<meta>` header. Arena owns team and match scores; shooter does
not duplicate them. RPG also serves open-world games; action-platformer
and runner use core, with RPG or puzzle compositions when needed.
Components never calculate damage, production, purchases, rewards, legal
card targets, or note timing.

| Style pack | Visual language | Commercial review references | Status |
| --- | --- | --- | --- |
| `clean` | Clear numeric hierarchy, restrained menus, low-interference HUD | Forza Horizon 5, Portal 2 | Pilot, in progress (references only) |
| `cartoon` | Rounded forms, deliberate thickness, clear primary actions | Mario Kart 8 Deluxe, Bloons TD 6 | Not yet built |
| `fantasy` | Painted frames and materials with room for dense content | Baldur's Gate 3, Diablo IV | Built, art-approved (pilot); shipped with the engine |
| `pixel` | Consistent pixel grid, integer scaling, readable glyphs | Stardew Valley, Octopath Traveler | Not yet built |
| `scifi` | Instrument hierarchy, precise geometry, sparse emphasis | Elite Dangerous, Starfield | Not yet built |
| `storybook` | Paper and illustrated edges, quiet ornament, readable text | Pentiment, Strange Horticulture | Not yet built |

The creation order is `clean`, `cartoon`, `fantasy`, `pixel`, `scifi`,
`storybook`. These values are not exposed before their packages are
ready; the integration owner updates the schema and bilingual labels
together. There are no `auto`, `custom`, or `generated` pseudo-packs.

The 37 look presets in `templates/looks/` and the generated look catalogue
own look-to-UI recommendations; until a pack is accepted every preset uses
core default UI. A recommendation is a seed, not proof of fit; an explicit
creation UI choice wins. Scifi is also chosen by subject (shooters,
strategy, space games). Game-authored styles use explicit asset paths in
the actual look; the creation block records the initial constraint, not
the current path.

## 5. Package lifecycle

Built-in documents live in the engine package: components in
`packages/engine/src/ui/components/` (a group's in `<group>/`), screens
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
  components. `og ui types` types the built-ins the game uses.
- A game document of the same name wins. `og ui eject <name>`
  (`rpg/backpack` in a group; `--list` names all) copies a built-in
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

Official styles live in `templates/styles/<name>` and ship under
`styles/<name>`. Resource and effect packs live in
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
parts, and the creation schema are contracts: a break needs a minor
release and a breaking entry with its fix in a new `changelog/unreleased/`
fragment. There are no legacy aliases. Planned groups and packs may be
absent on main; incomplete engine archives cannot ship. Release waits for
the complete approved catalogue and platform schema and pin coordination
([release rules](release.md)).

## 6. Author and accept

### 6.1 State fixtures

Each component group starts with an input/output and parts inventory, its
two primary pages, and two named commercial games as information-structure
references, each an actual page, platform and screenshot source. Authors
copy no commercial assets, brands or layouts.

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

A group's review sheet is a project in `packages/engine/sheets/<group>/`,
outside the package's shipped files. Its pages use the group's built-in
components by name, with fixtures from its `src/fixtures.ts`, sample art
under `assets/` and strings for the review's CJK and right-to-left
locales in `assets/strings/`; `og ui review` installs those locales' font
packs into its copy (without them those cells repeat the game locale).
The group's behaviour tests (`packages/engine/tests/<group>.test.ts`)
mount its members the same way.

Core tests use small self-contained styles for core behaviour
(specificity, font pixel snapping, probe paths); pack tests check pack art
and resources. Removing a pack cannot remove core coverage.

### 6.2 Frame matrix

Each group's sheet keeps two primary pages, one normal high-frequency page
and one stress/state page; behaviour tests cover every member. An extra
page needs a named visual behaviour the two cannot cover. `og ui sheet
<group>` and `og ui review <group>` take the group's sheet (or a sheet
project directory); a pack's own sheet is reviewed by its directory.

| Group | Primary pack | Contrast sample | Normal page; stress/state page |
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

The `general` group is built with `fantasy` as its primary pack; its
contrast sample is not yet chosen, and the counts below cover the nine
groups above.

Each sampled pair keeps four reference frames: normal desktop at 100 %,
normal phone at 100 %, stress desktop at 150 %, stress phone at 150 %.
Desktop is 1920×1080, phone portrait 390×844; landscape-heavy content also
has a genuine landscape fixture. Nine groups × two packs × four frames is
72 primary references. `og ui sheet` captures four cells per page, so two
pages give eight review captures per pair, of which the four above are
references. Locale, RTL, landscape and further states get risk-based
review captures, not permanent references.

Each of the six packs also has a core-control state sheet with desktop and
phone references: 12 more, 84 baseline frames in total. Large text and
localization are still reviewed per pack; any further reference names the
regression it guards. All 54 group/pack combinations run nonvisual
loading, parsing, resource-resolution and runnability checks; the 18
sampled pairs are not visual coverage of all, and a game's actual
combination needs review.

Captures record effective style, theme, viewport, DPR, text scale, locale,
focus, selection, tick and animation history; equal ticks do not prove
equal state. References are recorded on the GPU host (NVIDIA L4, Vulkan;
docs/ci.md §The GPU host); other GPU backends are compatibility checks
under the cross-platform tolerance. A missing GPU is unchecked, never
passed.

### 6.3 Acceptance and art sign-off

Technical checks reject parse failures, missing resources, wrong intents,
missing required states and out-of-bounds references. `og ui check`
reports contrast, layout and cost measurements; `og ui literals` reports
suspect values; neither scores taste. Overlaps are measurements:
`og.ui.overlaps` (and the check's overlap finding) lists every absolute,
anchored or floating element over another's drawing and flow siblings
over each other, leaving out only what the engine places on purpose (a
badge on its item); whether one is meant is the author's call, and no
style property hides one. Warnings never become automatic aesthetic
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

The first accepted set is clean and fantasy with RPG and shooter;
samples that depend on unbuilt packs stay explicitly pending. The rest of
the catalogue follows only after that set proves the author-to-review
process. Release requires the complete matrix; quality failures delay
release rather than lower scope or waive sign-off.

### 6.4 UI debugger

A development build's inspector (F1) has a UI view, its third tab. Pick
outlines the element under the pointer and picks it; the panel shows its
path and tag; its box model (margin, border, padding in CSS order, content
and border boxes with positions); the layout boundaries it sets (`scroll`,
`container`, `layer`, `fixed`: a fixed width and height); and its
computed styles: each winning document declaration with its value and
`file:line`, the pack rules dressing its part with their source, and its
bindings with expression and current value. The frame draws the picked
element's boxes in Chrome's colours; Layout bounds outlines every drawn
element; the filter field keeps styles, rules and bindings whose name
holds it. `inspect:ui[:<path>]` opens the view from a script and
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

## 7. Maintenance boundaries

The integration owner owns shared schema, labels, looks, creation,
packaging, this contract and the guides. Component group and pack authors
own their group or package directories, review sheets and acceptance
fixtures. Shared API changes go through the integration owner before
consumers adopt them.

A group's documents, string tables and README live in
`packages/engine/src/ui/components/<group>/`, its review sheet (with its
sheet references) in `packages/engine/sheets/<group>/`, its behaviour
tests in `packages/engine/tests/<group>.test.ts`, and each document's
cooked frames in og-tests-ui's
`references/vulkan/doc-components-<group>-<document>.png`. Pack tests and
references live outside runtime pack roots, under
`templates/styles/tests/<pack>` and `templates/styles/references/<pack>`.
Core fixtures live with core tests. Reference changes name the affected
frames and their reason; recording is not art approval.

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

og-ui is crates along its stages. `og-ui-model` holds geometry and codec
primitives, `og-rive` Rive artboards and their player, `og-reactive`
bindings and the reactive session. `og-ui-core` holds the immediate-mode
toolkit: the stages and their state, input, layout, the CSS cascade (the
style stage, which styles immediate widgets too), motion, paint and draw,
the document model and the documents' state. `og-ui` holds the documents:
the Svelte parse and writeback, build and mount, the SVG element build,
layers, style packs and the probe. Core calls them through
`doc::build::Documents`; they add methods to core's types through the
extension traits in `og_ui::prelude` and reach the stages' state only
through each stage's accessors. Their own state (`DocState`: build cursor,
the tick's outputs, mounts' memory, the reactive session) is held by core
behind `doc::build::DocumentState` and reached only through its
operations; a UI that built no document holds core's `NoDocuments`, with
what a snapshot carries (`DocMemory`). `og-ui` re-exports core under the
same paths (`og_ui::…`).

The [gate rules](ci.md) own test lanes, locks and toolchain policy;
`scripts/land.sh` owns landing on main. Release and platform installation
follow [release rules](release.md), with separate release authorization.

**The CSS dialect is the §8 inventory at its pinned version, and it is
closed.** og-ui implements game-UI CSS, not a browser: a row is in scope
when a component, a style pack or a guide needs it, and work goes to
proving rows in use before adding rows. A feature outside the pinned
inventory, or a row the owner marks out of scope, is refused by the parser
by name, never half-rendered. Moving the pin to a newer web-features
version is a decision of its own.

<!-- css-coverage:start -->
## 8. Standard CSS output-proof checklist

Source: [WebDX web-features 3.45.0](https://www.npmjs.com/package/web-features/v/3.45.0). All 454 entries with a `css.*` compatibility key are included: 257 widely available (high), 56 newly available (low), and 141 not Baseline. Including the newer entries keeps game-UI requirements visible rather than treating Baseline as a ceiling.

**Supported means the output assertion named in the row proves the listed scope.** It does not claim every syntax or subfeature grouped under that WebDX name. **Missing means an implementation gap or a missing output proof**, distinguished in the reason. A missing proof is not evidence that the implementation is absent. Parser acceptance, a property name in source, and an entire test file are not support evidence. This is a conservative, incomplete conformance inventory; unresolved rows stay visible.

Part applicability is proved only in the scopes named below: an element test does not prove that a widget `::part` lays out, transforms or paints the property. Inherited, pseudo-element, environment and lifecycle features need their own applicable part scenarios; there is no blanket all-properties claim.

Regenerate with `python3 scripts/css-coverage.py`; `--check` rejects document drift. For offline use, pass `--archive <web-features-3.45.0.tgz>`; its SHA-256 is pinned in the script. Proof entries are manually reviewed assertions, not name matches. Tests below live in `crates/og-ui/tests/` (what the UI builds) or `crates/og-tests-ui/tests/` (the frames drawn of it), each one integration test binary; GPU proofs require Metal.

| WebDX feature | Baseline | Document elements | Widget ::part | Proof scope or missing reason |
| --- | --- | --- | --- | --- |
| `abs-sign` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `absolute-positioning` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::positioning_places_as_css_positioning_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered. |
| `accent-color` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `active-view-transition` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `align-content-block` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `alignment-baseline` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `all` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `alpha` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `alt-text-generated-content` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `anchor-positioning` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_layout::positioning_places_as_css_positioning_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered. |
| `anchor-positioning-animations` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `anchor-positioning-position-visibility-plurals` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `anchor-positioning-transforms` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `animation-composition` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `animations-css` | Widely | Supported (scope →) | Supported (scope →) | [`documents::scoped_css_motion_uses_shared_curves_emits_sound_and_restores`](../crates/og-ui/tests/documents.rs). A cubic-bezier(0,1,0,1) keyframe animation reaches 7/8 displacement at 1/8 duration; twelve subsequent positions replay exactly after checkpoint restoration. Pushed-screen path, not every animation mode. Part proof: [`dressing::retained_part_nodes_animate_without_rebuilding_the_host`](../crates/og-ui/tests/dressing.rs). A retained button part emits alpha 0.2/0.5/0.8 at ticks 0/6/12 for a 200ms linear animation, then paints nothing while idle. Other animation modes are not covered. |
| `appearance` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `aspect-ratio` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `attr` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `attr-contents` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `autofill` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `autonomous-custom-elements` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `backdrop` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `backdrop-filter` | Newly | Missing | Pending parts-as-nodes | Ordinary staged backdrop chains have fixtures; native glass rejects chains requiring intermediate clamps, drop-shadow and SVG url filters. |
| `background` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-attachment` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-blend-mode` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-clip` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-clip-border-area` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-clip-text` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-color` | Widely | Missing | Supported (scope →) | Output proof not yet reviewed; implementation status unverified. Part proof: [`dressing::tabs_and_select_parts_keep_boxes_states_and_host_input`](../crates/og-ui/tests/dressing.rs). Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts. |
| `background-image` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::image_urls_resolve_files_not_atlas_names_or_handles`](../crates/og-ui/tests/css_paint.rs). Retained drawing selects the registered relative file and numeric filename, rejects a missing path despite an atlas basename collision, and reports the missing asset. css_paint_matches_its_reference_frames covers background tiling and gradients on Metal. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks proves cooked SVG pixels, imported masks and inline cursor URL provenance. |
| `background-origin` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-position` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-repeat` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `background-size` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `baseline-shift` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `baseline-source` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `before-after` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `border-image` | Widely | Supported (scope →) | Supported (scope →) | [`css_paint::flex_ratios_absolute_fades_and_state_border_images_render`](../crates/og-tests-ui/tests/css_paint.rs). A real button hover changes rendered border pixels from a shorthand source to a longhand gradient source; this does not prove all border-image modes. Part proof: [`dressing::part_host_negation_and_part_states_select_distinct_paint`](../crates/og-ui/tests/dressing.rs). Four primary/hover combinations assert exact resolved opacity and border-image presence for button:not(.primary)::part(base). No document-selector claim. |
| `border-radius` | Widely | Supported (scope →) | Supported (scope →) | [`box_decor::corners_take_css_radii_and_shapes`](../crates/og-ui/tests/box_decor.rs). Both builders emit fill/stroke ellipse radii with overlap scaling, percentages, slash syntax and bevel corners. Part proof: [`box_decor::border_radius_rounds_a_dressed_button`](../crates/og-ui/tests/box_decor.rs). Emitted button base radius replaces its default in both immediate and retained documents. |
| `border-shape` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `borders` | Widely | Supported (scope →) | Supported (scope →) | [`box_decor::borders_pad_the_box_and_outlines_take_no_room`](../crates/og-ui/tests/box_decor.rs). Computed child offsets prove border widths/styles and logical sides affect layout; outline/offset take no room. Part proof: [`dressing::tabs_and_select_parts_keep_boxes_states_and_host_input`](../crates/og-ui/tests/dressing.rs). Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts. |
| `box-decoration-break` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `box-shadow` | Widely | Supported (scope →) | Pending parts-as-nodes | [`box_decor::decorations_paint_in_css_order_over_any_draw`](../crates/og-ui/tests/box_decor.rs). Both builders assert outer shadow offset/blur, inset spread and paint order relative to the fill. |
| `box-sizing` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `calc` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `calc-constants` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `calc-size` | Not Baseline | Missing | Pending parts-as-nodes | No calc-size evaluation or animated output proof. |
| `cap` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `caret-color` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `caret-shape` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `cascade-layers` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `case-insensitive-attributes` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `case-sensitive-attributes` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `ch` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `charset` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `clip` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `clip-path` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `clip-path-animatable` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `clip-path-boxes` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `color` | Widely | Supported (scope →) | Supported (scope →) | [`dressing::host_font_and_color_reach_button_ink_and_divider_output`](../crates/og-ui/tests/dressing.rs). Emitted button text and divider fill match authored hex colors; an explicit button part color overrides inherited host ink. This does not cover every color syntax or space. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. |
| `color-adjust` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `color-function` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `color-gamut` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `color-mix` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `color-mix-variadic` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `color-scheme` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `colrv0` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `colrv1` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `column-breaks` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_layout::multi_column_lays_out_as_css_multicol_does`](../crates/og-ui/tests/css_layout.rs). Computed balanced columns, width/count, spanning blocks, auto fill, explicit breaks and split text lines. |
| `column-fill` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::multi_column_lays_out_as_css_multicol_does`](../crates/og-ui/tests/css_layout.rs). Computed balanced columns, width/count, spanning blocks, auto fill, explicit breaks and split text lines. |
| `column-pseudo` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `column-span` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::multi_column_lays_out_as_css_multicol_does`](../crates/og-ui/tests/css_layout.rs). Computed balanced columns, width/count, spanning blocks, auto fill, explicit breaks and split text lines. |
| `column-wrap` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `conic-gradients` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::repeating_gradients_repeat_the_stop_interval`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixel assertions for both halves of all four repeating conic quarters. |
| `contain` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `contain-inline-size` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_contain::size_containment_sizes_as_if_empty`](../crates/og-ui/tests/css_contain.rs). Computed empty size, intrinsic replacement dimensions and inline-only containment. |
| `contain-intrinsic-size` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_contain::size_containment_sizes_as_if_empty`](../crates/og-ui/tests/css_contain.rs). Computed empty size, intrinsic replacement dimensions and inline-only containment. |
| `contain-layout` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `contain-paint` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `contain-size` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_contain::size_containment_sizes_as_if_empty`](../crates/og-ui/tests/css_contain.rs). Computed empty size, intrinsic replacement dimensions and inline-only containment. |
| `contain-style` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `container-anchor-position-queries` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `container-name-queries` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `container-queries` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::container_queries_restyle_lists_grids_and_rive_frames_before_paint`](../crates/og-ui/tests/documents.rs). Viewport changes immediately change list/grid padding and Rive visibility under a named inline-size query. Pushed-screen path; style/scroll-state queries and container units are not covered. |
| `container-scroll-state-queries` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `container-style-queries` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `content` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `content-visibility` | Newly | Supported (scope →) | Pending parts-as-nodes | [`css_contain::content_visibility_skips_contents_off_screen`](../crates/og-ui/tests/css_contain.rs). Computed offscreen placeholder/remembered sizes, re-entry and hidden accessibility contents. |
| `contrast-color` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `corner-shape` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`box_decor::corners_take_css_radii_and_shapes`](../crates/og-ui/tests/box_decor.rs). Both builders emit fill/stroke ellipse radii with overlap scaling, percentages, slash syntax and bevel corners. |
| `counter-reset-reversed` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `counter-set` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `counter-style` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `counters` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `crisp-edges` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `cross-document-view-transitions` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `cross-fade` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `cubic-bezier-easing` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::scoped_css_motion_uses_shared_curves_emits_sound_and_restores`](../crates/og-ui/tests/documents.rs). A cubic-bezier(0,1,0,1) keyframe animation reaches 7/8 displacement at 1/8 duration; twelve subsequent positions replay exactly after checkpoint restoration. Pushed-screen path, not every animation mode. |
| `currentcolor` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `cursor` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_interact::plain_boxes_choose_cursors_in_clipped_transformed_paint_order`](../crates/og-ui/tests/css_interact.rs). Both builders assert cursor inheritance over noninteractive boxes, passive overlays above buttons, pointer-events:none, hidden and overflow-clipped children, 90-degree transforms and image hotspots. Runtime ui_configure::svg_asset_urls_supply_cursor_pixels_and_masks checks cooked SVG cursor pixels on a plain div. css_interact::cursors_apply_only_inside_clip_paths proves, in both builders, cursors only inside circle(), polygon() over content-box, evenodd path(), shape(), objectBoundingBox url(#id) and an ancestor's inset() clip. Clip-path press hit testing and every cursor keyword are not covered. |
| `custom-ellipses` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `custom-media-queries` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `custom-properties` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::bound_custom_properties_feed_the_cascade`](../crates/og-ui/tests/custom_properties.rs). Computed sizes follow bound/inherited variables, important precedence, null fallback and keyed insertion. |
| `customizable-select` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `default` | Widely | Missing | Pending parts-as-nodes | documents::boolean_attribute_selectors_follow_svelte_presence proves retained :default follows false/present checked attributes. The immediate builder does not expose :default state; full support is missing. |
| `details-content` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `device-posture` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `device-queries` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `dir-pseudo` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-animation` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-contents` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_layout::display_lays_out_as_css_display_does`](../crates/og-ui/tests/css_layout.rs). Computed contents flattening, inline flow-root, margin boundary, table spans and caption position; not border collapse/spacing. |
| `display-flow-root` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::display_lays_out_as_css_display_does`](../crates/og-ui/tests/css_layout.rs). Computed contents flattening, inline flow-root, margin boundary, table spans and caption position; not border collapse/spacing. |
| `display-list-item` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-mode` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-ruby` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-state` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `display-table` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::display_lays_out_as_css_display_does`](../crates/og-ui/tests/css_layout.rs). Computed contents flattening, inline flow-root, margin boundary, table spans and caption position; not border collapse/spacing. |
| `dominant-baseline` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `dynamic-range` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `dynamic-range-limit` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `element` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `em-unit` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `empty` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::pushed_screens_match_sibling_and_structural_selectors_on_instantiated_rows`](../crates/og-ui/tests/documents.rs). Computed widths/heights after keyed reorder and conditional removal prove filtered nth-child, nth-last-child, sibling and empty matching. |
| `ex` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `exp-functions` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `field-sizing` | Newly | Supported (scope →) | Pending parts-as-nodes | [`control_props::fields_size_to_displayed_content_and_respect_constraints`](../crates/og-ui/tests/control_props.rs). Both builders prove text-input value/placeholder sizing, caret-width empty inputs, textarea wrapping and newline heights, ignored rows, explicit dimensions, min/max width, noninheritance and removal of content sizing. Retained editing and checkpoint restore preserve focus/caret and update height. selects_files_and_vertical_fields_size_to_content proves, in both builders, a select fitting its shown option (fixed fills, explicit width wins), file inputs fitting chosen names while fixed keeps the default label width, and vertical-rl inputs/textareas sizing height to content with a line-wide width and caret-high empty height, wrapping into columns under a fixed height. Listbox selects and fixed sizing in vertical modes are not covered. |
| `file-selector-button` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `filter` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_filters::css_filters_match_their_reference_frames`](../crates/og-tests-ui/tests/css_filters.rs). Metal references at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains. |
| `filter-function` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `first-letter` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `first-line` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `fit-content` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `fit-content-function` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `fixed-positioning` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::positioning_places_as_css_positioning_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered. |
| `flexbox` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::flex_lays_out_as_css_flexbox_does`](../crates/og-ui/tests/css_layout.rs). Computed grow/shrink shares, reverse/order, safe alignment, wrapping, row and column gaps. |
| `flexbox-flex-wrap-balance` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `flexbox-gap` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::flex_lays_out_as_css_flexbox_does`](../crates/og-ui/tests/css_layout.rs). Computed grow/shrink shares, reverse/order, safe alignment, wrapping, row and column gaps. |
| `float-clear` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::floats_lay_out_as_css_floats_do`](../crates/og-ui/tests/css_layout.rs). Computed opposing/stacked floats, paragraph exclusion, flow-root avoidance, clear:both and enclosing float height. Rectangular exclusion only. |
| `focus-visible` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `focus-within` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_interact::focus_within_follows_real_focus_between_retained_ancestors`](../crates/og-ui/tests/css_interact.rs). Real clicks transfer focus between nested retained branches; computed widths prove shared ancestors stay focused and old ancestors reset. Parts and the legacy immediate builder are not covered. |
| `font-display` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-face` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-family` | Widely | Supported (scope →) | Supported (scope →) | [`runs::ordered_named_families_preserve_primary_glyphs_and_inherit`](../crates/og-ui/tests/runs.rs). Both builders assert exact glyph/font handles for inherited lists, rich spans, reversed named/generic order, names after generics and quoted generic names. Generics map to engine font roles, not installed system fonts. Part proof: [`runs::ordered_named_families_preserve_primary_glyphs_and_inherit`](../crates/og-ui/tests/runs.rs). Both builders emit button::part(base) glyphs from ordered named families, including a named fallback after an intervening generic. |
| `font-family-math` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-family-system` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-family-ui` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-feature-settings` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-kerning` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-language-override` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-metric-overrides` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-optical-sizing` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-palette` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-palette-animation` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-shorthand` | Widely | Supported (scope →) | Pending parts-as-nodes | [`control_props::root_relative_text_sizes_follow_the_root_not_the_parent`](../crates/og-ui/tests/control_props.rs). Both builders emit 1.75rem at 24.5/36.75 px under 100/150 percent text scale, distinct from 1.75em under a 10px parent; shorthand is checked too. |
| `font-size` | Widely | Supported (scope →) | Supported (scope →) | [`control_props::root_relative_text_sizes_follow_the_root_not_the_parent`](../crates/og-ui/tests/control_props.rs). Both builders emit 1.75rem at 24.5/36.75 px under 100/150 percent text scale, distinct from 1.75em under a 10px parent; shorthand is checked too. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. |
| `font-size-adjust` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-stretch` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-style` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-synthesis` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-synthesis-position` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-synthesis-small-caps` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-synthesis-style` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-synthesis-weight` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-alternates` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-caps` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-east-asian` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-emoji` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-ligatures` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-numeric` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variant-position` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-variation-settings` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-weight` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `font-width` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `forced-colors` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `form-validity-pseudos` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `frame-sizing` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `fullscreen` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `function` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `gap-decorations` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `glyph-orientation-vertical` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `gradient-interpolation` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `gradients` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::repeating_gradients_repeat_the_stop_interval`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixels prove repeating radial intervals with positive/negative starts, all eight conic half-quadrants and collapsed linear stops. Subpixel periods and unbounded stop counts are not covered. |
| `grid` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::grid_lays_out_as_css_grid_does`](../crates/og-ui/tests/css_layout.rs). Computed named/negative/spanning tracks, auto-fit/fill, fr minimums, areas, alignment and inherited subgrid tracks. |
| `grid-animation` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `grid-lanes` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hanging-punctuation` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `has` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `has-slotted` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `heading-selectors` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `highlight` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `host` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `host-context` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hsl` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hwb` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hyphenate-character` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hyphenate-limit-chars` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `hyphens` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `ic` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `if` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `image-orientation` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `image-rendering` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_interact::images_take_object_fit_position_and_inherited_image_rendering`](../crates/og-ui/tests/css_interact.rs). Emitted image paint has contain fit, right-edge offset and inherited nearest sampling. |
| `image-set` | Widely | Supported (scope →) | Supported (scope →) | [`css_paint::image_sets_render_density_correct_backgrounds_masks_and_borders`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixels at DPR 1/2/1 prove density-correct background/mask tiles, MIME filtering before duplicate removal (including quoted parameters and malformed-type rejection), zero-resolution transparency, percentage border slices with auto widths and gradient selection. Resolution math covers calc/min/max/clamp with mixed x/dppx/dpi/dpcm units; retained output also tests fractional DPR. Part proof: [`css_paint::image_sets_render_density_correct_backgrounds_masks_and_borders`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixels prove button::part(base) reselects its background at DPR 1/2/1 while preserving an 8 CSS px tile. Other part image properties are not separately proved. |
| `ime-mode` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `import` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `indeterminate` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `individual-transforms` | Widely | Supported (scope →) | Supported (scope →) | [`css_interact::translate_percentages_move_by_the_elements_own_box`](../crates/og-ui/tests/css_interact.rs). Percentage translate changes computed box and hit testing; this proof covers translate, not individual rotate/scale. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. |
| `inherit-value` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `initial-letter` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `initial-value` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `input-selectors` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `interaction` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `interactivity` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `interest-invokers` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `interpolate-size` | Not Baseline | Missing | Pending parts-as-nodes | No intrinsic-size interpolation implementation or output proof. |
| `inverted-colors` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `is` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `isolation` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `justify-self-block` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `lab` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `lang` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `layout-direction-override` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::direction_lays_out_as_css_writing_modes_does`](../crates/og-ui/tests/css_layout.rs). Computed flex positions, logical/physical margins and absolute placement distinguish dir and CSS direction on LTR/RTL surfaces. |
| `letter-spacing` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `lh` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `light-dark` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `light-dark-image` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `line-break` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `line-clamp` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `line-height` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `linear-easing` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `link-parameters` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `link-selectors` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `list-style` | Widely | Supported (scope →) | Pending parts-as-nodes | [`html_text::list_markers_follow_css_and_renumber_on_reorder`](../crates/og-ui/tests/html_text.rs). Both build paths assert marker text/shapes, position, inheritance and keyed reorder. Explicit initial resetting is tracked separately. |
| `logical-properties` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `margin` | Widely | Supported (scope →) | Pending parts-as-nodes | [`html_text::block_margins_follow_the_ua_stylesheet_and_collapse`](../crates/og-ui/tests/html_text.rs). Both builders assert positive/negative sibling and parent-child margin collapse, padding/fixed-height boundaries, noncollapsing flex margins, UA em margins and keyed insertion. |
| `margin-trim` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `marker` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `mask-border` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `mask-type` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `masks` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::masks_hide_invisible_boxes_and_fade_image_content`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content. |
| `mathml` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `media-pseudos` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `media-queries` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::media_queries_follow_dimensions_preferences_and_nested_intersections`](../crates/og-ui/tests/documents.rs). Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries. |
| `media-query-range-syntax` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::media_queries_follow_dimensions_preferences_and_nested_intersections`](../crates/og-ui/tests/documents.rs). Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries. |
| `min-max-clamp` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `min-max-content` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `min-max-width-height` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `mix-blend-mode` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `modal` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `motion-path` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `multi-column` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::multi_column_lays_out_as_css_multicol_does`](../crates/og-ui/tests/css_layout.rs). Computed balanced columns, width/count, spanning blocks, auto fill, explicit breaks and split text lines. |
| `named-color` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `namespace` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `nesting` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `not` | Widely | Missing | Supported (scope →) | Output proof not yet reviewed; implementation status unverified. Part proof: [`dressing::part_host_negation_and_part_states_select_distinct_paint`](../crates/og-ui/tests/dressing.rs). Four primary/hover combinations assert exact resolved opacity and border-image presence for button:not(.primary)::part(base). No document-selector claim. |
| `nth-child` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::pushed_screens_match_sibling_and_structural_selectors_on_instantiated_rows`](../crates/og-ui/tests/documents.rs). Computed widths/heights after keyed reorder and conditional removal prove filtered nth-child, nth-last-child, sibling and empty matching. |
| `nth-child-of` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::pushed_screens_match_sibling_and_structural_selectors_on_instantiated_rows`](../crates/og-ui/tests/documents.rs). Computed widths/heights after keyed reorder and conditional removal prove filtered nth-child, nth-last-child, sibling and empty matching. |
| `nth-of-type` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `object-fit` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_interact::images_take_object_fit_position_and_inherited_image_rendering`](../crates/og-ui/tests/css_interact.rs). Emitted image paint has contain fit, right-edge offset and inherited nearest sampling. |
| `object-position` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_interact::images_take_object_fit_position_and_inherited_image_rendering`](../crates/og-ui/tests/css_interact.rs). Emitted image paint has contain fit, right-edge offset and inherited nearest sampling. |
| `object-view-box` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`control_props::an_image_shows_its_object_view_box`](../crates/og-ui/tests/control_props.rs). Both build paths emit asymmetric inset/calc crop UVs; the removed -og-uv property is rejected. |
| `oklab` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `opacity` | Widely | Missing | Supported (scope →) | Output proof not yet reviewed; implementation status unverified. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. Part proof: [`dressing::part_host_negation_and_part_states_select_distinct_paint`](../crates/og-ui/tests/dressing.rs). Four primary/hover combinations assert exact resolved opacity and border-image presence for button:not(.primary)::part(base). No document-selector claim. Part proof: [`dressing::retained_part_nodes_animate_without_rebuilding_the_host`](../crates/og-ui/tests/dressing.rs). A retained button part emits alpha 0.2/0.5/0.8 at ticks 0/6/12 for a 200ms linear animation, then paints nothing while idle. Other animation modes are not covered. |
| `opacity-svg` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `open-pseudo` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `outline` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `outlines` | Widely | Supported (scope →) | Pending parts-as-nodes | [`box_decor::borders_pad_the_box_and_outlines_take_no_room`](../crates/og-ui/tests/box_decor.rs). Computed child offsets prove border widths/styles and logical sides affect layout; outline/offset take no room. |
| `overflow` | Widely | Supported (scope →) | Pending parts-as-nodes | [`box_decor::overflow_scrolls_and_clips`](../crates/og-ui/tests/box_decor.rs). Wheel input changes only the auto axis; computed viewport/content extents and offsets are asserted. |
| `overflow-anchor` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overflow-clip` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overflow-clip-margin` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overflow-overlay` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overflow-shorthand` | Widely | Supported (scope →) | Pending parts-as-nodes | [`box_decor::overflow_scrolls_and_clips`](../crates/og-ui/tests/box_decor.rs). Wheel input changes only the auto axis; computed viewport/content extents and offsets are asserted. |
| `overflow-wrap` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overlay` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `overscroll-behavior` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`box_decor::overscroll_contains_and_scroll_padding_insets_snaps`](../crates/og-ui/tests/box_decor.rs). Wheel input proves nested containment and scroll-padding changes the final mandatory snap offset. |
| `padding` | Widely | Supported (scope →) | Supported (scope →) | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. Part proof: [`dressing::tabs_and_select_parts_keep_boxes_states_and_host_input`](../crates/og-ui/tests/dressing.rs). Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts. |
| `page-break-aliases` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `page-breaks` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `page-orientation` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `page-selectors` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `page-setup` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `paint` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `paint-order` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `path-shape` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `physical-properties` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `picture-in-picture` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `placeholder` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `placeholder-shown` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `pointer-events` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_interact::the_pointer_passes_through_none_and_hidden`](../crates/og-ui/tests/css_interact.rs). Actual cursor routing through pointer-events:none and visibility:hidden, descendant overrides, URL priority/hotspot and accessibility presence. Immediate builder only; OS cursor loading is not covered. |
| `popover` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `position` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `prefers-color-scheme` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `prefers-contrast` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `prefers-reduced-data` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `prefers-reduced-motion` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::media_queries_follow_dimensions_preferences_and_nested_intersections`](../crates/og-ui/tests/documents.rs). Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries. |
| `prefers-reduced-transparency` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `print-color-adjust` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `progress-function` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `q-unit` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `quotes` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `random-function` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rcap` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rch` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `read-write-pseudos` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `reading-flow` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rect-xywh` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `registered-custom-properties` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `relative-color` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `relative-positioning` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::positioning_places_as_css_positioning_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered. |
| `rem` | Widely | Supported (scope →) | Pending parts-as-nodes | [`control_props::root_relative_text_sizes_follow_the_root_not_the_parent`](../crates/og-ui/tests/control_props.rs). Both builders emit 1.75rem at 24.5/36.75 px under 100/150 percent text scale, distinct from 1.75em under a 10px parent; shorthand is checked too. |
| `resizable` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `resize` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_contain::a_resize_handle_drags_the_size_onto_the_element`](../crates/og-ui/tests/css_contain.rs). Pointer drags change computed dimensions subject to axis, overflow and max-width. |
| `resolution` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `resolution-compat` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `revert-rule` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `revert-value` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rex` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rgb` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `ric` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `rlh` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `root` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `round-mod-rem` | Newly | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `ruby-align` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `ruby-overhang` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `ruby-position` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `safe-area-inset` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::safe_area_insets_clear_the_hosts_covered_control`](../crates/og-ui/tests/custom_properties.rs). Computed fixed and normal-flow positions prove viewport-relative env values, the engine root's existing safe-area padding, and covered-control inset updates in portrait/landscape. Insets do not subtract ancestor padding. |
| `scope` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scope-pseudo` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scripting` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-axis-lock` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-behavior` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-buttons` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-driven-animations` | Newly | Supported (scope →) | Pending parts-as-nodes | [`documents::view_timelines_follow_the_subject_through_its_scroller`](../crates/og-ui/tests/documents.rs). Both builders assert width and translation at rest and after wheel input for an anonymous view() cover timeline. Named timeline scopes and all range keywords are not covered. |
| `scroll-initial-target` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-marker-targets` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-markers` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-snap` | Widely | Supported (scope →) | Pending parts-as-nodes | [`box_decor::overscroll_contains_and_scroll_padding_insets_snaps`](../crates/og-ui/tests/box_decor.rs). Wheel input proves nested containment and scroll-padding changes the final mandatory snap offset. |
| `scroll-target-group` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scroll-triggered-animations` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scrollbar-color` | Newly | Supported (scope →) | Pending parts-as-nodes | [`box_decor::a_wheel_stops_at_snap_stop_always_and_scrollbar_color_inherits`](../crates/og-ui/tests/box_decor.rs). Emitted thumb is inherited red; repeated wheel input stops at the mandatory intermediate snap area. |
| `scrollbar-gutter` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `scrollbar-width` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `selection` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `selectors` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `shadow-parts` | Widely | Missing | Supported (scope →) | Host/part/state selector registry is present; all-property part applicability awaits the parts-as-nodes stream and its output tests. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. Part proof: [`dressing::tabs_and_select_parts_keep_boxes_states_and_host_input`](../crates/og-ui/tests/dressing.rs). Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts. |
| `shape-function` | Newly | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-tests-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `shape-outside` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::shape_outside_changes_line_exclusion_not_float_placement`](../crates/og-ui/tests/css_layout.rs). Both builders prove triangular line exclusion, unchanged float stacking/BFC avoidance/clear, and dynamic none/restoration. shape_outside_uses_the_whole_line_band_and_reference_box proves circles, ellipses, insets, margin/border/content boxes, margin-box clipping, degenerate insets and dropping a wide line along a slope. shape_margin_expands_by_distance_and_clips_to_the_float_box proves distance-based shape-margin, containing-block percentages, clipping and dynamic zero restoration. shape_images_wrap_lines_around_their_alpha proves, in both builders, url() alpha contours per pixel row, shape-image-threshold as number and percentage, an empty shape, shape-margin around them, content-box placement and the box kept without alpha; runtime ui_configure::shape_outside_images_take_their_contour_from_cooked_alpha reads a cooked SVG's alpha natively. Gradient images, Basis textures on the web, other syntax, elliptical box corners and writing modes remain unverified. |
| `shapes` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::shape_outside_changes_line_exclusion_not_float_placement`](../crates/og-ui/tests/css_layout.rs). Both builders prove triangular line exclusion, unchanged float stacking/BFC avoidance/clear, and dynamic none/restoration. shape_outside_uses_the_whole_line_band_and_reference_box proves circles, ellipses, insets, margin/border/content boxes, margin-box clipping, degenerate insets and dropping a wide line along a slope. shape_margin_expands_by_distance_and_clips_to_the_float_box proves distance-based shape-margin, containing-block percentages, clipping and dynamic zero restoration. shape_images_wrap_lines_around_their_alpha proves, in both builders, url() alpha contours per pixel row, shape-image-threshold as number and percentage, an empty shape, shape-margin around them, content-box placement and the box kept without alpha; runtime ui_configure::shape_outside_images_take_their_contour_from_cooked_alpha reads a cooked SVG's alpha natively. Gradient images, Basis textures on the web, other syntax, elliptical box corners and writing modes remain unverified. |
| `sibling-count` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `slot` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `smooth` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `speak` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `speak-as` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `spelling-grammar-error` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `starting-style` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `state` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `static-positioning` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `steps-easing` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `sticky-positioning` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::positioning_places_as_css_positioning_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert relative flow reservation, asymmetric absolute insets, viewport-fixed corners, sticky wheel scrolling, anchor()/anchor-size(), position-area and position-try. Anchor animation and visibility are not covered. |
| `stretch` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `style-query-range-syntax` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `subgrid` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::grid_lays_out_as_css_grid_does`](../crates/og-ui/tests/css_layout.rs). Computed named/negative/spanning tracks, auto-fit/fill, fr minimums, areas, alignment and inherited subgrid tracks. |
| `supports` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `supports-at-rule` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `supports-compat` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `supports-named-feature` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `svg` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `svg-discouraged` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `svg-filters` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_filters::css_filters_match_their_reference_frames`](../crates/og-tests-ui/tests/css_filters.rs). Metal references at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains. |
| `symbols-function` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `system-color` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `system-color-accentcolor` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `tab-size` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `table` | Widely | Missing | Pending parts-as-nodes | Table spans/captions have computed output tests, but border-spacing and border-collapse have no implementation/output proof. |
| `target` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `target-text` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-align` | Widely | Supported (scope →) | Pending parts-as-nodes | [`control_props::a_button_label_takes_text_align`](../crates/og-ui/tests/control_props.rs). Both builders assert emitted label positions for default centre, explicit start and menu item alignment; parent alignment does not override the button default. |
| `text-align-last` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-autospace` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-box` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-combine-upright` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-line-blink` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-selection` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-skip-ink` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-skip-ink-all` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-skip-spaces` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-decoration-spelling-grammar` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-emphasis` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-fit` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-indent` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-indent-each-line` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-indent-hanging` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-justify` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-orientation` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-overflow` | Widely | Supported (scope →) | Pending parts-as-nodes | [`html_text::text_wrap_styles_and_overflow_strings_lay_out_as_css_does`](../crates/og-ui/tests/html_text.rs). Both builders inspect emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line. |
| `text-shadow` | Widely | Supported (scope →) | Pending parts-as-nodes | [`control_props::scene_text_shadows_follow_the_font_size`](../crates/og-ui/tests/control_props.rs). Both builders emit the same shadows as independently resolved px values for em/lh and inherited font size; none removes them. |
| `text-size-adjust` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-spacing-trim` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-stroke-fill` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-transform` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-underline-offset` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-underline-position` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-wrap` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `text-wrap-balance` | Newly | Supported (scope →) | Pending parts-as-nodes | [`html_text::text_wrap_styles_and_overflow_strings_lay_out_as_css_does`](../crates/og-ui/tests/html_text.rs). Both builders inspect emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line. |
| `text-wrap-pretty` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`html_text::text_wrap_styles_and_overflow_strings_lay_out_as_css_does`](../crates/og-ui/tests/html_text.rs). Both builders inspect emitted lines: balance reduces width spread at the same line count; pretty avoids the final single-word line; custom overflow text replaces the ellipsis on one clipped line. |
| `time-relative-selectors` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `touch-action` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `transform-box` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `transforms2d` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `transforms3d` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `transition-behavior` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `transitions` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `trig-functions` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `two-value-display` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::display_lays_out_as_css_display_does`](../crates/og-ui/tests/css_layout.rs). Computed contents flattening, inline flow-root, margin boundary, table spans and caption position; not border collapse/spacing. |
| `unset-value` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `update` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `url-cross-origin` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `url-integrity` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `url-referrer-policy` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `user-action-pseudos` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `user-pseudos` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `user-select` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `vertical-align` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::vertical_align_places_inline_boxes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert sup/sub text baselines, inline-block baseline/middle/top, px/percentage offsets and line-box growth. |
| `vertical-form-controls` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `video-dynamic-range` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `view-transition-class` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `view-transitions` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `view-transitions-element-scoped` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `viewport-segments` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `viewport-unit-variants` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `viewport-units` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::vi_and_vb_follow_the_roots_writing_mode`](../crates/og-ui/tests/css_layout.rs). Computed vi/vb dimensions follow the root writing mode on an 800x900 viewport, independently of child writing modes. Dynamic/small/large viewport units are not covered. |
| `visibility` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::masks_hide_invisible_boxes_and_fade_image_content`](../crates/og-tests-ui/tests/css_paint.rs). Metal pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content. |
| `webvtt` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `webxr-dom-overlays` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `where` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `white-space` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `white-space-collapse` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `widows-orphans` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `width-height` | Widely | Supported (scope →) | Supported (scope →) | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. Part proof: [`dressing::button_parts_keep_separate_boxes_and_host_event_ownership`](../crates/og-ui/tests/dressing.rs). Button base dimensions, independent host/part translate, inherited font size/color, opacity keyframes, click ownership and remove/reinsert output. Limited to this button fixture. Part proof: [`dressing::tabs_and_select_parts_keep_boxes_states_and_host_input`](../crates/og-ui/tests/dressing.rs). Tab indicator height, select group height, list padding+border inset, group text padding and distinct tab/option fills are asserted after real clicks. Semantic roles remain on hosts. |
| `will-change` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `window-controls-overlay` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `window-drag` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `word-break` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `word-break-auto-phrase` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `word-break-break-word` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `word-spacing` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `writing-mode` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::writing_modes_lay_out_as_css_writing_modes_does`](../crates/og-ui/tests/css_layout.rs). Both builders assert vertical-rl/lr, sideways-lr, flex axes, logical padding/margins, orthogonal flow and vertical float geometry. |
| `writing-mode-svg-values` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `z-index` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `zoom` | Newly | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
<!-- css-coverage:end -->
