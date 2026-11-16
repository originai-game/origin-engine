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
  data-doc="…">`. Left out, it reads its type's zero value (`""`, `0`,
  `false`, `[]` with no rows, `{}`, null for `any`), so a component is
  usable as `<Name />`; `data-default="…"` gives another: a string's is
  the attribute's text, the other types' are JSON of that type (`5`,
  `true`). A list prop's default is `[]`: its rows come from the
  collection its user binds. Only a `data-required` prop must be passed:
  it is required in the generated `.d.ts`, and a mount leaving it out
  returns an error naming the instance and prop. `$props()` is refused.
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
  On a component instance a custom property is a style prop, written
  bare: `<Meter --fill={$hp / $max}/>`.
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
`text-combine-upright` (down a vertical line, `all` sets the run in one
upright em, one unit of the line), and `font-size`, `line-height` and `letter-spacing` in px or em. A run's
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

`$captions` is read the same way: the lines the engine is saying now, a
collection of `{ id, text }` keyed by `id`, in the order of the timelines
playing and of their caption tracks. A timeline's caption track draws
nothing itself; the built-in `subtitles` screen shows `$captions` after
the game's own `$lines`, and any document may iterate it. A line that
stays keeps its row. `$toasts` is the toasts up (`og.ui.toast`), oldest
first, each `{ id, text, tone, leaving }`: it stays its time, then is
`leaving` for a fast duration while its exit plays; at most three stay up.
The built-in `notifications` screen shows them after its own `$items`; a
document's `onclick={dismissToast}` in a row of `$toasts` dismisses that
toast, answered by the engine and never sent to the game.

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

`<og-counter value duration decimals format from>` counts to its value as
it changes; `from` starts its first count there (a score counting up as its
screen opens), and `format` writes the number as a binding's format does
(`compact`, `percent`, `clock` …), `decimals` then being the format's digit
count.
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
An `interestfor` control's `interest-delay-start` and `-end` (or the
`interest-delay` shorthand) time its hint: shown after the pointer rests
that long (keyboard focus shows it at once), kept that long after the
pointer and focus leave; `normal` is the style's slow and fast durations.
`contextmenu="id"` (engine attribute) opens popover `id` at a secondary or
long press, anchored at that point with no arrow. A `<button>` centres its
label by `text-align: center` as HTML's UA sheet does. A popover is built
every frame; whether it is open, shown or closing is snapshotted. Closed,
it is HTML's `display: none` popover and keeps no style: it opens from its
`@starting-style`, showing from the frame `:popover-open` matches, and its
`opacity`, `transform`, transitions and animations move its layer. An
`allow-discrete` transition of `overlay` (or `display`) keeps a closing
popover up, inert, until it ends. A styled `::backdrop` covers the page
under it while it is up. With no transition or animation of its own, an
`auto` popover keeps the engine's drop-in.

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
  height. `glyph-orientation-vertical` is read as `text-orientation`;
  `text-combine-upright` combines a whole text or a `::part` run (`all`)
  or each short digit run (`digits`), squeezing it across to one em.
- **Ruby and emphasis marks.** `<ruby>` is inline markup of a text: each
  `<rt>` annotates the words before it as one unbreakable box (no `<rtc>`,
  no nested ruby, no `display: ruby` on other elements). `ruby-position`
  takes `over` and `under` (`alternate` is `over`, `inter-character` is
  read as `over`). Emphasis marks are drawn shapes; their side follows
  the content language through the user-agent rules (under for `zh`, over
  for `ja` and `ko`), and a ruby's base takes none.
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
  the text's drop). `::target-text` takes the same four and marks what a
  followed link's text fragment matched (below); its default background
  is the `Mark` system colour. `::file-selector-button` styles nothing. The CSS 2
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
  common reset as UA rule). `stretch` is CSS Sizing 4's stretch-fit size:
  the containing block's size less the box's margins, the same in a block,
  a flex line or a grid area. Across a flex line it fills the line, as
  `align-self: stretch` does. Along one it is the container's whole inner
  size, so siblings that share a row or column say so with `flex: 1` (and
  `min-width: 0` or `min-height: 0` to let content shrink), never with
  `stretch`. A percentage bound against an unknown size is ignored. A flex
  item without `flex` properties keeps its kind: a fit-content item
  shrinks to its content, and a definite or `stretch` one neither grows
  nor shrinks. `order` keeps the document's reading order.
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
re-resolving only their readers. `accent-color` inks checkboxes,
switches, sliders, radios and progress, and what a control draws on its
accent (a check mark) is `contrast-color()` of it; `caret-color` a
field's caret and `caret-shape` its form (a bar, a block under the next
character's advance, or an underscore below it; 1ch at the end); `color-scheme` picks `light-dark()`'s side.

Colours take all of CSS Color 4 and 5 (`crates/og-ui/src/style/css_color.rs`).
Linebender's `color` crate parses absolute colours and owns every space,
conversion and CSS Color 4 §12 interpolation; og-css adds `var()` roles,
`currentColor`, `light-dark()`, `contrast-color()` (white or black,
whichever has the greater WCAG 2 ratio: at least 4.58:1 on any opaque
ground), `color-mix()` over one colour or more (CSS Color 5 §3.3, OKLab
when no space is named), relative colours and gamut mapping
(out-of-gamut colours by OKLCh chroma reduction) into the output's gamut:
sRGB, or Display P3 where the renderer's surface presents it
(`Theme::gamut`, from the viewport), so `color(display-p3 …)` and a wide
`oklch()` keep their range there. A colour that reads no role,
`currentColor` or `light-dark()` and lies in sRGB reduces to sRGB at
cook; a wider one, and the others, resolve against the active style when
drawn. One parse
(`style_file::parse_color`) reads every hex literal.

Transforms follow Transforms 1 and 2 (individual properties first).
`transform-box` `content-box` and `fill-box` are the content box, others
the border box. An element whose plane leaves the screen's draws itself
and its subtree into a layer through its projective map (perspective-correct
text; hits through the inverse). A `preserve-3d` context draws its planes
back to front by centre depth; intersecting planes are not split.
Motion Path 1's `offset-path` (`path()`, `circle()`, `ray()` with its
size; not `contain`, `at <position>` or other shapes), `offset-distance`
(animatable), `offset-rotate`, `offset-anchor` and `offset-position` move
an element's anchor along a path in its parent's laid-out border box, as
one matrix between the individual properties and `transform`; the
`offset` shorthand is not taken.

CSS paint follows Backgrounds 3, Images 3 and 4, Masking 1, Shapes 2
(`shape()`), Filter Effects 1 and 2 and Compositing 1, with these engine
rules and limits:

- `url(<file>)` resolves beside the stylesheet that wrote it (inline styles
  beside the document) and names a cooked texture (SVG included) the
  document load registers; it never selects an atlas icon or a numeric
  handle. A missing image warns. Monochrome icons are `mask-image: url()`
  over `background-color: currentColor`.
- Gradients take at most 128 stops, mixed in OKLab (CSS Images 4's
  default for every gradient), or in the space an `in <colour space>
  [<hue> hue]` names: `color`'s piecewise OKLab approximation of that mix,
  within an eighth of a just-noticeable difference, so a hue sweep shows
  no bands. og-render keeps a gradient's stops in its frame's points table
  and finds a pixel's two by binary search. A mix that leaves the
  output's gamut (sRGB, or Display P3 on a wide surface) is gamut mapped
  per pixel by CSS Color 4 §13.2's OKLCh chroma reduction (color.wgsl's
  `gamut_map`), keeping its lightness and hue. A gradient layer is a
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
  builds nothing. A moving `transform` or `opacity` repaints nothing: the
  element's and its subtree's commands are kept and placed again.

Overflow and scrolling: `scrollbar-width: thin` is half width with a
half-width grab strip; `scrollbar-color`'s track runs the whole edge
(`auto`: the style's thumb, no track). Scroll Snap 1 applies to drags,
flings and wheels; `proximity` snaps within a third of the view;
`scroll-snap-stop: always` stops a wheel at the first such area past where
it started (saved UI state). A fling is steered onto its target exactly.
`overscroll-behavior` `contain` and `none` act alike (og-ui draws no
glow or bounce). `scroll-behavior: smooth` glides a scroll into view as a
fling; reduced motion jumps. `scroll-initial-target: nearest` (Scroll
Snap 2) starts a scroll container that appears scrolled to the first such
element in it, at its view's start, from its first frame (its layout runs
again once); a target added to a container already shown does not move
it, and floating layers' containers are not covered. A drag on a
container that scrolls both ways keeps to the axis its first move runs
along, within a 2:1 slope of it, until released (saved UI state);
`scroll-axis-lock: none` (Overflow 5) lets it move freely. A container
scrolled down from its start follows an anchor box (Scroll Anchoring 1):
the first box in view, searched inside the first one only partly in
view; when content above it changes size, the offset moves by as much on
the same frame (saved UI state). `overflow-anchor: none` turns it off on
a container and keeps a subtree from anchoring; snapping containers and
floating layers are not anchored. A modal
`<dialog data-detents="40% 90%">` is a
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
  max-distance scale-at>` holds a column over a world point, placed every
  tick; a moving camera builds nothing. Decluttered by rank (priority,
  then nearness), each takes the nearest spot a gap clear of all those
  placed before it draw (their scaled and overflowing parts included) and
  of the grounds' text, on the screen and within its reach (`stack`
  straight above, then below, then any way; `shift` any way), and fades
  out while none is.
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

Editing is Parley's (`parley::editing`): the text is laid out as it is
drawn (a face without a usable file through a layout-only face of its own
advances and kerning), and Parley's cursor and selection place the caret
from a press, move it by grapheme cluster on screen (across a bidi edge to
its other place, kept as the cursor's affinity), by word (UAX #29 with
ICU's dictionaries: Thai, Lao, Khmer, Myanmar, Chinese and Japanese by
their words) and by line, break an area's lines (UAX #14), and give
Backspace (a character, a whole emoji or line break) and Delete (a
cluster) what they remove. og-ui keeps the gestures (drag with edge
scroll, double and triple press), scrolling, typed text and composition,
the platform's keys, undo grouping, the input types and the accessible
value.

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
selects, a double press a word (UAX #29 with ICU's dictionaries), a
triple press a line, Shift extends; on a coarse pointer a 500 ms long
press selects a word and an immediate drag scrolls. A drag crosses elements in tree order. Ctrl+C
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
`interactivity: inert` (CSS UI 4) makes an element and its subtree inert
as the `inert` attribute does; a declared value wins over the attribute,
as HTML's `[inert] { interactivity: inert }` rule has it.

#### Containment, resize and will-change

`contain` and `content-visibility` follow Containment 2. A skipped
subtree is left out of the layout walk and costs nothing while idle; an
`auto` one skips from the frame after it is found more than half a
viewport outside its clip with no focus inside, stays in the
accessibility tree, and keeps its last laid-out size under
`contain-intrinsic-size: auto`. `resize` (CSS UI 4) draws a three-row dot
handle in the style's strong control ink; a drag sets width or height in
CSS px within its bounds, presentation state (not saved) written only
while dragged. `will-change` makes now what the properties it names would
(Will Change 1 §2): a stacking property makes a stacking context drawn as
its own group layer, and `transform`, `perspective`, `filter`,
`backdrop-filter` or `contain` also holds fixed descendants; other names
make neither.

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
`<textPath>` does (CSS `offset-path` moves the element itself, above):
each grapheme cluster sits centred on the path, turned to it.

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
(`og.ui.toast`, a row of the `notifications` screen's log) announces as it
appears. Each change
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
afresh when it opens. `details::details-content` (Pseudo-Elements 4)
styles the content box under the summary with every box property over
the disclosure's own padding and gap; a closed disclosure hides the box
whole (CSS keeps its margins, borders and background and hides only its
contents), and `details[open]` follows the element's open state (a
literal or bound `open`, or the player's toggle).

`<meter>` dresses its fill by HTML's region, `Meter.fill[region=optimum |
suboptimum | poor]` (undressed: success, warn, danger), over `Meter.track`;
`low` and `high` clamp into the range, `optimum` defaults to the middle.

A range input, `<progress>` and `<meter>` in a vertical writing mode stand
up (Writing Modes 4 vertical form controls): their `height` is the track's
length (160 px by default), and the value runs top to bottom, or bottom to
top under `direction: rtl` (`sideways-lr` the reverse). A slider's labelled
marks stand beside its track, its bubble beside the knob. `<select>` stays
across.

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

The default stylesheet (`crates/og-ui/default.css`, generated from
`packages/design/tokens/*.css`) sets every value the core controls and
built-in components draw with as a `:root` variable, and the theme reads
them back each frame, so a pack restyles every control by setting tokens,
dressing `::part`s and supplying assets. A control token is
`--<control>-<property>` and defaults to a semantic token or the control's
number.

| Token family | What it controls |
|---|---|
| `--seed-primary`, `--seed-accent`, `--seed-neutral`, `--seed-danger`, `--seed-warn`, `--seed-success`, `--seed-info`, `--seed-team` | The seed colours every colour below is derived from (below) |
| `--color-*`, the role names (`--ink`, `--primary`…), `--team1` to `--team8` | Colours, per scheme, derived from the seeds; `--color-track`, `--color-minimap`, `--color-minimap-edge`, `--color-slot-selected-edge` for the engine's own parts |
| `--common`, `--uncommon`, `--rare`, `--epic`, `--legendary`, `--mythic` | The rarity colours (by default each a status colour: `--line-strong`, `--success`, `--info`, `--xp`, `--warn`, `--danger`) |
| `--space-*`, `--radius-*`, `--size-*`, `--stroke-*` | Spacing, corners, icon and art sizes (`--size-xs` to `--size-7xl`; a layout takes a step, never a multiple of one), line widths (`--stroke-edge` the material hairline) |
| `--type-*`, `--text-*`, `--font-*` | The type ramp and its roles, authored at 100 %: a `--type-*-size` or line height set in px grows with the player's text scale wherever it is set (the root, a pack's sheet, a component), as the ramp widgets draw with does, so text set with `font: var(--text-sm)` follows it; `--font-leading-*` (`flat`, `tight`, `snug`, `normal`), `--font-tracking-*` and `--font-weight-*` for text a part sizes itself; `--font-step-2xs` to `--font-step-6xl` are sizes in `em` of the surrounding text, so a part sized with one follows the player's text scale; `--font-step-px` is one px at 100 % text grown by the scale, for a part that grows with the text as a whole |
| `--density-compact-*`, `--density-pad-*`, `--density-touch-*` | Each input's control, row and header heights, gutter and section |
| `--control-sm/md/lg`, `--row`, `--header`, `--gutter`, `--section`, `--touch-target` | The input's density, and the smallest finger target |
| `--motion-duration-*`, `--motion-easing-*`, `--motion-named-*`, `--motion-scale-*` | Durations (the transition steps, then the loops and long plays: `turn` 800 ms, `pulse`, `flare`, `ping`, `sweep`, `breathe`, `sheen`, `drift`, `fanfare`, `orbit` 6 s), the controls' enter and exit curves (`in-out` for what loops), the named transitions' springs (`ui`, `panel`, `emphasis`, `playful`, `enter`, `exit`), how far a moment scales (`press`, `swell`, `pop`, `shrink`) and how low a waiting thing breathes (`breathe`) |
| `--state-*` | Hover wash, press scales (`press-scale` a button's part, `control-press-scale` a control row's), lift, disabled opacities (`touch-disabled-opacity` the stick, pad and action button), press duration, focus ring and player rings |
| `--opacity-*`, `--filter-*` | How strongly a thing shows (`faint`, `dim` what cannot be used, `muted` what is done, `soft`, `wash` how much of a colour a soft wash lays over what is under it), and what art takes out of reach (`--filter-locked`, `--filter-spent`) |
| `--shadow-*`, `--elevation-*`, `--glass-*` | Shadows per scheme and the controls' inset shadows, the two drop elevations, glass blur and saturation |
| `--frame-*`, `--plate-*`, `--band-*`, `--title-*`, `--row-*`, `--cell-*`, `--badge-*`, `--well-*`, `--figure-*`, `--tag-*`, `--label-*`, `--ink-label/bright/done/disabled/selected/down/mark/on-warn`, `--trend-*`, `--tone-*`, `--mark-*`, `--sfx-*` | What the built-in components' role parts wear (below) |
| `--button-*` | Radius, border width, rest and pressed shadows (stepper, details, keyboard keys and option rows take them); `--button-selected-fill` and `-ink` a button that is on (`aria-pressed`: a toggle, a chip), whatever its variant |
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

**Control hooks.** Everything a pack restyles a core control with is a
token its rule reads, so a pack is `:root` values and no rules. A hook the
default declares (`packages/design/tokens/state.css`) is the value it
draws with. One it does not declare falls back to the hook it refines
(`--button-hover-rim` to `--button-rim`, `--button-primary-image-hover` to
`--button-primary-image`) and last to the control's own value through
`revert-layer`, which a `var()` fallback rolls back as the keyword written
would (CSS Variables 2 §3). A rim the default does not draw takes a
`-rim-width` beside its colour. A state hook is the base name with
`-hover`, `-pressed`, `-disabled`, `-checked` or `-selected`, and a variant's
inserts the variant (`--button-primary-hover-fill`).

| Control | Hooks |
|---|---|
| Button (`button::part(base)`) | `--button-ink/fill/rim/rim-width/base-shadow/image/image-slice/image-width/corner-shape/font/font-size/tracking/transform/text-shadow/pad/pad-inline/pressed-pad/min-height`; per state `-hover-`, `-pressed-`, `-disabled-` fill, ink, rim and image, `--button-hover-shadow`, `--button-base-pressed-shadow`, `--button-disabled-shadow`; per variant `--button-primary-*` (fill, ink, rim, shadow, image and their states), `--button-danger-fill/hover-fill/pressed-fill/disabled-fill/rim/shadow/ink/text-shadow`, `--button-ghost-ink/hover-ink/pressed-ink/rim/hover-fill/pressed-fill`, `--button-quiet-ink/hover-ink/pressed-ink/hover-fill/pressed-fill`, `--button-selected-fill/ink/rim/shadow` (`aria-pressed`). A quiet button is text: it wears no rim, shadow or image. `--button-shadow` and `--button-pressed-shadow` stay the stepper's, details' and keys'. The focus ring follows the button's corners and their shape; `--button-focus-radius` and `--button-focus-corner-shape` give it its own |
| Checkbox, radio, switch | `--checkbox-box-fill/rim/shadow/hover-fill/hover-rim`, `--checkbox-checked-fill/rim/rim-width/shadow/disabled-fill`, `--checkbox-disabled-fill/rim` (a radio ring's too), `--checkbox-row-hover-fill`, `--checkbox-radius` (the box's, by default the field radius), `--checkbox-corner-shape`; `--radio-ring-fill/rim/shadow/hover-rim`, `--radio-checked-fill/rim/rim-width/shadow`, `--radio-dot-fill`, `--radio-option-hover-fill`, `--radio-corner-shape`, `--radio-card-fill/rim/rim-width/hover-fill/hover-rim/checked-fill/checked-rim/checked-shadow/checked-ink/corner-shape`, `--radio-chosen-fill/rim`; `--switch-track-fill/rim/shadow`, `--switch-checked-fill/rim/shadow`, `--switch-knob-fill/rim/shadow`, `--switch-corner-shape` |
| Tabs | `--tabs-track-fill/border/rule/shadow`, `--tabs-tab-fill/hover-fill/pressed-fill/selected-fill/selected-shadow/ink/hover-ink/selected-ink/disabled-ink`, `--tabs-font/font-size/tracking/transform`, `--tabs-indicator-fill/height/glow/radius` |
| Slider | `--slider-track-fill/rim/rim-width/shadow`, `--slider-fill/fill-shadow/tick-fill`, `--slider-knob-fill/rim/rim-width/shadow/hover-shadow/pressed-fill/disabled-fill/corner-shape`, `--slider-core-fill`, `--slider-bubble-fill/rim/shadow/radius` |
| Field, select, stepper | `--field-fill/rim/rim-width/frame-shadow/hover-fill/hover-rim/focus-rim/focus-shadow/disabled-fill/disabled-rim/invalid-rim/ink/font/placeholder-ink/selection-fill/caret-fill/corner-shape`; `--dropdown-list-fill/rim/rim-width/radius`, `--dropdown-option-hover-fill/pressed-fill/selected-fill/selected-ink` (a group's label wears `--label-*`); `--stepper-button-fill/hover-fill/pressed-fill/disabled-fill/rim/hover-rim/ink`, which a number field's spin buttons take too |
| Details | `--details-summary-fill/hover-fill/open-fill/ink/open-ink/border/rule/shadow/font` (by default `--label-font`), `--details-content-ink` |
| Surfaces | `--panel-fill/rim/shadow/glass/corner-shape` (a panel and a dialog's panel; `panel.glass` blurs by default), `--dialog-backdrop`, `--tooltip-fill/rim/rim-width/radius/shadow/glass/corner-shape`, `--hud-fill/rim/rim-width/shadow/glass/corner-shape`; a `-glass` is the `backdrop-filter`, `none` a solid surface; `--popover-arrow-fill`, `--spotlight-edge` |
| Slot, badge, bars | `--slot-fill/rim/rim-width/shadow/hover-rim/hover-shadow/pressed-fill/selected-rim/selected-shadow/corner-shape` (`--slot-shadow` none by default); `--badge-<tone>-fill/ink` (neutral, primary, info, success, warn, danger), `--badge-rim/rim-width` (by default the badge's ink, its tone's), `--badge-font/tracking/transform`; `--meter-track-fill/rim/rim-width/shadow`, `--meter-fill/suboptimum-fill/poor-fill/health-fill/mana-fill`, `--progress-fill` |
| Keyboard, touch | `--keyboard-panel-fill`, `--keyboard-key-fill/rim/hover-fill/hover-rim/pressed-fill/pressed-rim/shadow/ink/font` (its candidates too); `--touch-plate-fill/rim`, `--touch-knob-fill`, `--touch-active-fill` |
| Text | `--heading-flourish` (h1–h3), `--scrollbar-fill/radius`, `--icon-secondary-ink` |

Icon path coordinates and behaviour constants (key repeat, dead zones,
caret blink, typewriter speed) are not tokens.

**Seeds.** The default stylesheet derives every colour token from the seed
tokens as CSS relative colours (`packages/design/tokens/seed.css` and
`color.css`, packages/design/design-language.md §Colour). These are
expressions resolved when drawn. A pack sets the palette by setting seeds in
its `:root`, once for both schemes, and the derivation keeps each scheme's
text and team inks at 4.5:1, its focus ring and rims at 3:1, and teams 1 to
6 0.1 apart in OKLab under every colour vision, whatever the seeds. A pack that sets a derived token itself (`--primary`) keeps it and
gives up the derivation for it. A game sets seeds at run time with
`og.ui.configure`'s `seeds` (UiSeeds), written as root tokens over the
style's. Calling it again with new seeds (a faction, the time of day)
restyles the whole UI from the next tick. A look's `uiSeeds { primary
"#rrggbb" }` block gives them with its style. Seeds are read at the root: a
seed set on an element below it does not re-derive that subtree's colours.

**Role parts.** A built-in component names a part by its role
(`Shop.frame`, `Friends.row`, `Chat.title`), and the default style
dresses every part of a role once, from that role's tokens
(`packages/design/tokens/role.css`). A pack restyles all of them by
setting the tokens; it writes a rule only for what one component alone
wears. The role rules sit in `:where()`, so any pack or document rule wins.
A part named for what it shows declares its role with a part list, as in
HTML: `part="ArenaRoster.portrait ArenaRoster.picture"` is reached by
`::part()` under either name, by its document's `[part~="ArenaRoster.portrait"]`,
and wears the role of its last name. Every name is in the `parts` line.
A control named for a role (`<button part="Bag.cell">`) wears the role
alone, as `appearance: none` gives a browser's button its page's look: its
own part's rules (`button::part(base)`) do not reach it. A control named
for no role (`Shop.buy`) keeps its own dress under the named part's rules.
A `cell` is a tile the player picks or reads as one (an item, a card, an
ability, an equipment slot); a readout is not a cell, and is named for what
it shows (`RacingLapHud.readout`, `RacingStandings.place`,
`ArenaScoreboard.stat`), which no role rule dresses.

| Role (`part$=".<role>"`) | Tokens | States the default dresses |
|---|---|---|
| `frame` | `--frame-fill/rim/rim-width/radius/shadow/pad/corner-image/corner-size/glass` | `class="hud"` wears the plate (`--plate-fill/rim/rim-width/radius/shadow/pad/glass`); `class="bare"` wears nothing (a frame that only lays its children out); a `-glass` is its `backdrop-filter`, which a bare frame and a frame inside another do not take |
| `band`, `title` | `--band-fill/height/pad/rule/rule-width/rule-image/shadow/end-start-image/end-finish-image`, `--title-ink/font/size/weight/tracking/transform/shadow` | `class="lead"` and `class="small"` bands (`--band-lead-fill`, `--band-small-*`, `--title-lead-size`, `--title-small-size`), which set `--title-size` for the title inside them; one step down on a phone on its side |
| `row` | `--row-fill/hover-fill/pressed-fill/selected-fill/selected-edge/radius` | hover, pressed, selected (`aria-selected`, `aria-current`, `data-selected`), disabled, `data-done` |
| `cell` | `--cell-fill/rim/rim-width/selected-rim/radius/shadow/hover-shadow/pressed-fill/empty-shadow`, `--cell-<rarity>-fill/shadow` | hover, pressed, selected, `data-rarity`, `data-empty`, disabled or `data-locked` |
| `badge`, `well`, `figure` | `--badge-fill/ink/radius/shadow`, `--well-fill/radius/shadow`, `--figure-ink/font/shadow` | |
| text over the scene (StatusGrid's time left, Nameplate's name and subtitle, Waypoint's label and distance) | `--tag-halo` its outline, `--tag-fill/pad/radius` a ground of its own (none in the default style) | |
| text over a warn fill (Nameplate's spell) | `--ink-on-warn` | |
| `label`, `heading` | `--label-font/tracking/transform`, `--ink-label` | |
| `name` | `--name-font/size/tracking/transform/ink/shadow`, `--name-selected-ink` (all unset by default: the component's own type stands) | selected |
| `picture` | `--picture-fill/rim/rim-width/radius/shadow` (unset by default) | |
| `tab` | the core tabs' `--tabs-tab-fill/hover-fill/selected-fill/ink/selected-ink`, `--tabs-font/tracking/transform` | hover, selected |
| any part | `--trend-up/down`, `--tone-damage/heal/crit` | `data-trend`, `data-tone` |
| any role part | `--team1`, `--team2` | `data-team=a` and `data-team=b`: the team's colour as its ink and rim |

`--mark-*` are the small signs a pack supplies as images and `--sfx-*`
its press sounds; both are `none` in the default style.

### 3.2 Parts and states

Style selectors dress a component's named parts with their variants,
states and attributes. The pack grammar lives in
`crates/og-ui/src/style_file/css.rs` and document CSS in
`crates/og-ui/src/css/values.rs`; generated references come from them.
Parser support for a literal does not override §3.1. A document's
`<style>` controls appearance and layout, attributes content and
behaviour; rules are document-scoped and never affect unrelated mounts.

**Selectors and cascade.** Selectors follow Selectors 4: every combinator
and attribute operator with `i`/`s` flags (HTML's case-insensitive
attributes compare so unless `s`), `:not()`, `:is()`, `:where()`, `:has()`
and the structural, form, `:open`, `:modal`, `:popover-open`, `:target`
(the element the last `#fragment` link named, scrolled to the start of
its scrollers as HTML's "scroll to the fragment" does; with a text
fragment, `#id:~:text=[prefix-,]start[,end][,-suffix]` per URL Fragment
Text Directives, the nearest element holding the first passage it
matched, its line centred, else the id's), `:lang()` (RFC 4647),
`:dir()`, `:heading` and `:heading(1, 2)` (an `<h1>` to `<h6>`, by level),
`:active-view-transition` and `:active-view-transition-type()` (a
screen's root while its layer's view transition runs, by the types its
`@view-transition` declares) pseudo-classes. Engine meanings and limits:

- `lang`, `dir`, `data-*` and `state:name={…}` are selector-only
  attributes on every element (a field's `lang` also picks its
  keyboard); `:state(name)` reads `state:name`; `:defined` matches every
  element, `:scope` the root, `:any-link` an `<a href>`. An ancestor
  selector reads that ancestor's own interaction state.
- A `<button>`'s `variant` is one of its classes, for its host's rules as
  for its parts': `button.primary` selects `<button variant="primary">`.
- A state pseudo-class after a box pseudo-element reads the
  pseudo-element's own state (`dialog::backdrop:hover` is the pointer on
  the scrim, not on the dialog): `:hover`, `:active`, `:focus`,
  `:focus-visible`, `:disabled`, `:enabled`, and a scroll marker's
  `:target-current`, `:target-before` and `:target-after`, which also
  match the elements marked so (`Ui::set_selector_state` sets them for
  the machinery that decides them). `::before` and `::after` take none.
- A `<video>` or `<audio>` matches `:paused` or `:playing` by its own
  `paused` (not by whether its player has frames left), `:muted` by its
  `muted`, and `:seeking` from a `currentTime` write until the seek
  reaches its player. `:buffering`, `:stalled` and `:volume-locked` parse
  and never match: a player reads a local asset, and the engine mixes its
  own sound.
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
`revert-layer` and `revert-rule` (the property as it is without the
keyword's own rule). The engine's layer order is §5's. `@import "x.css"
[layer()] [supports()] [media]` is read at cook from beside the document
into its blob. `@property` registers syntax, inheritance and initial
value. `@counter-style` and `@position-try` feed counters and anchor
positioning.

**Cascade order.** A declaration ranks by its importance, then its rule's
tree context, then the style attribute, then its layer, specificity and
order (Cascade 5 §6). The default style and a pack are adopted into every
tree, so their rules rank there by layer: `og-default` < `og-components`
(a built-in component's own `<style>`) < `og-pack` < `og-game` < a
document's unlayered rules. A pack beats a built-in component's own rules
and loses to a game's. A built-in control's parts are a shadow tree under
its element. A `::part()` selector may read where that element stands in
its document (`.shop button::part(base)`, `li:first-child >
button::part(base)`), and three contexts rank a part's declarations, lowest
first:

1. the style's `::part()` rules;
2. the control's own box declarations (a `<button>`'s plate, below);
3. its document's own `::part()` rules.

`!important` reverses the contexts, as it reverses layers. Servo's
`selectors` crate parses and matches every selector. og adds three things
on top of it. A selector that reads only the element and its ancestors is
lowered to a reactive condition. The rest is matched on the mounted tree.
And a dependency profile bounds what an edit restyles.

`@font-face` (Fonts 4): the cook bundles each `url()` source, which must
be a project font cooked by its own settings; `local()` names nothing (no
system fonts). The runtime registers the first source the bundle has.
`font-family` tries named and generic families in authored order per
glyph, skipping unavailable names; generics map to the style's roles
(`monospace`, `ui-rounded`, `fantasy`/`cursive`, else body), and a quoted
`"serif"` is a named family. In a family the text takes the nearest
`font-stretch` (`font-width`; at or under normal the narrower faces
first, over it the wider), then `font-style`, then `font-weight`, and per
character the last-declared face whose `unicode-range` has it. After the list come the role's face, the CJK
companion, the locale's and the game's fonts. A missing weight or slant is
synthesised: a computed weight of 600 or more whose face is lighter than
600 is thickened and drawn again one device pixel right, in place of the
style's `textWeight` (a style that wants its own sets
`font-synthesis-weight: none`). A variable face (one whose file has
`fvar` and renders at run time) is drawn at an instance (Fonts 4 §7.2):
`wght` from `font-weight` and `wdth` from `font-stretch`, each clamped to
the face's declared range (an `@font-face` without the descriptor, or
with `auto`, declares the font's axis range), `opsz` from the size in
CSS px under `font-optical-sizing: auto` (the initial value), then
`font-variation-settings`, which overrides them; an axis the face lacks
is ignored. A bold is synthesised against the clamped weight. The
instance is measured and shaped at its location, and the host renders its
glyphs into pages of its own from the frame after it first appears (that
first frame draws the text blank, never at the default location). A
`::part` run keeps the same list. A role `UiDesc` gives no
font takes the first family with a face that the style's `--font-body`
(else `--font-sans`), `--font-display`, `--font-mono` or `--font-rounded`
names, so a pack that brings its faces only through `@font-face` sets the
text widgets draw by role, as a browser's text with no `font-family` of
its own takes the root's.

**Custom properties** are lexical and inherit through instantiated
parents, component mounts included; they resolve on the declaring element
(a child's `--a` does not recompute an inherited `--b: var(--a)`);
recursive `var()` takes fallbacks, empty ones too; cycles invalidate their
properties and report their names; an invalid winning declaration does not
revive a lower one. Unshadowed engine tokens resolve against the active
style. Data reaches the cascade through `style:--name={expr}`, an inline
declaration (a number plain, a string as is, `null` unset) that
re-resolves only its element and inheritors; an instance's
`--name={expr}` reaches its root the same way, as an inherited value. `calc()`, `min()`, `max()`
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
is the device in use) and `-og-color-vision` (below). `color-gamut` and
`dynamic-range` follow the renderer's surface (`Viewport::wide_gamut`,
`high_range`, from `RenderInfo::wide_gamut` and `hdr`): `p3` and `high`
match where it presents Display P3 and extended range, `srgb` and
`standard` always, `rec2020` never. The other display features answer
fixed values: `display-mode: fullscreen`, `display-state: fullscreen`, `update: fast`, `scripting: enabled`,
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
  to or from hidden. Under `interpolate-size: allow-keywords` (inherited)
  a `width` or `height` moves between a length and an intrinsic keyword
  (`auto`, `min-content`, `max-content`, `fit-content`, `stretch`) through
  `calc-size()`, as CSS Values 5 §8.2 has it, by a transition (keyframes
  still step a keyword). A property not declared transitions from its CSS
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
selects maps; `isActionDown` reads the down state, `isActionJustPressed`
and `isActionJustReleased` tick edges; `isActionHeld` keeps its `holdMs`
threshold; physical controls keep their W3C codes. The UI's pointer,
gestures and actions use the same words (`down`, `justPressed`,
`justReleased`, `held` after `holdMs`; the conventions guide §3):
`og.ui.actionFired` is the tick a UI action fires, `og.ui.actionHeld` its
hold.

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

`og style check <directory|pack|default> [--json]` checks CSS and asset
URLs, fails literals outside `:root` (§3.1) and reports contrast as facts.
It builds the core-control sheet's pages (every state held) and every
built-in document from its `fixture:default` under the pack, in both
schemes, without a GPU, and reads each drawn text on what is drawn under
it, composited in linear light: fills, gradients and image texels where
the text is, glass by its tint, layers by their opacity; a ground the UI
covers only partly is read over black and over white and called `scene`.
Each pair carries its WCAG 2 ratio, APCA Lc and, under a `text-shadow` or
stroke, its ratio on that halo. It also reports how far apart (ΔEOK) the
team and state colours stay under protanopia, deuteranopia and
tritanopia (og-draw's simulation). Nothing is graded; the bars are the
screens guide's (§5). `og style check-project` is the game's art-contract
checker.

`og ui sheet <pack>` renders a pack's core-control pages at desktopHd and
phonePortrait at 100 % and 150 % text, scroll pages captured to the
bottom; `og ui review <pack> --locales game,pseudo,zh,ar` adds locales. A
pack is named as a game names it: `fantasy`, or `@styles/fantasy` as
`og.load.style` loads it (a bundled pack, or the project's
`assets/styles/<name>`). Both take `--out DIR` and `--json` and a group's
sheet (§6).
Review frames before recording references. The
[screens guide](../crates/og-api/guides/screens.md#3-style-packs) shows
loading, selectors, tokens and asset URLs.

#### Remote images

A `url()` naming an `http(s)://` image (in a document, an inline style or
a style pack) is fetched at run time, not cooked. og-ui reads the URL
and CSS Values 5's request modifiers, `cross-origin()`, `integrity()` and
`referrer-policy()`, into one request key (`crates/og-ui/src/css/request.rs`),
which a sheet lists in `Sheet::remote`; a package file takes no
modifiers. The runtime (`crates/og-runtime/src/world/remote.rs`) fetches
each key once per engine when its document or pack loads and binds the
decoded image under the key (`Ui::define_texture`, which restyles what is
mounted); a layer naming an unbound key draws nothing and warns nothing.

- **Fetch.** Always CORS mode (a texture needs readable pixels):
  credentials `same-origin` by default, `include` under
  `cross-origin(use-credentials)`, the integrity metadata, and the
  referrer policy (`strict-origin-when-cross-origin` by default). The web
  host fetches from the engine Worker and decodes with the browser (an
  SVG on the page, which a Worker cannot draw); the native host fetches
  with og-net's client (`crates/og-net/src/http.rs`), which applies the
  Fetch standard's CORS check (redirect tainting included), the
  Referrer Policy algorithm and SRI's strongest-hash match, and decodes
  PNG, JPEG and WebP with `image` and SVG with resvg at its own size, as
  the cook does. The request's origin and referrer are the engine's root
  when it is an http(s) URL (the page's directory on the web); a game read
  from files has an opaque origin and sends no referrer. Natively there
  is no cookie jar.
- **Presentation only.** An image is bound at a UI build (the `UI_END`
  system) after it lands; nothing fetched reaches the world or a journal.
  The native host waits there for every fetch started, so executors'
  frames are the same on every run; the web draws what has landed.
  `og.loading.idle` is false while one is pending.
- **Failures** (CORS, integrity, HTTP status, decoding) are one warning
  per key naming the URL and the reason.
- A build without og-runtime's `fetch` feature fetches nothing natively
  and warns per key.
- **Orientation.** Both hosts read the image's EXIF orientation from its
  metadata with one reader (og-runtime's `exif_orientation`: PNG `eXIf`,
  JPEG APP1, WebP `EXIF`) and set it on the texture
  (`RenderScene::set_texture_orientation`); the texels stay as the file
  stores them and `image-orientation` turns them at draw time, as for a
  cooked image. The web's `createImageBitmap` always turns an image
  upright, so the Worker reads the orientation through the wasm
  (`og_web_image_orientation`) and draws the bitmap turned back
  (boot.ts `storedTurn`) before handing its texels over
  (tests/web/remote-images.ts).

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
(`crates/og-ui/src/text/colr.rs`) at `color_px` in the configured palette,
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

Every component and screen document under `packages/engine/src/ui/` is
built into the engine (§5): the generic components and screens, and the
component groups below, each a directory of
`packages/engine/src/ui/components/`. Core default UI is the only creation
`ui` choice until a pack is accepted; a look's `ui` line or
`og.load.style` selects a built-in pack in any game. `materials` is an
optional resource pack for UI shader finishes, neither a component group
nor a style. VFX and unrelated extensions are outside this catalogue.

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
fixtures (§6.1) and a review sheet (§6.2), and both built packs dress it.
Member inputs, intents and parts are documented in each group's README
(`packages/engine/src/ui/components/<group>/README.md`) and each
document's `<meta>` header; the ungrouped components' in
`packages/engine/src/ui/components/README.md`, the screens' in
`packages/engine/src/ui/screens/README.md`. Arena owns team and match
scores; shooter does not duplicate them. RPG also serves open-world games;
action-platformer and runner use core, with RPG or puzzle compositions
when needed. Components never calculate damage, production, purchases,
rewards, legal card targets, or note timing.

| Style pack | Visual language | Commercial review references | Status |
| --- | --- | --- | --- |
| `fantasy` | Painted frames and materials with room for dense content | Baldur's Gate 3, Diablo IV | Built, shipped with the engine; art-approved (pilot) |
| `scifi` | Instrument hierarchy, precise geometry, sparse emphasis | Elite Dangerous, Starfield | Built, shipped with the engine; awaiting art approval |
| `cartoon` | Rounded forms, deliberate thickness, clear primary actions | Mario Kart 8 Deluxe, Bloons TD 6 | Built, shipped with the engine |
| `pixel` | Consistent pixel grid, integer scaling, readable glyphs | Stardew Valley, Octopath Traveler | Not yet built |
| `storybook` | Paper and illustrated edges, quiet ornament, readable text | Pentiment, Strange Horticulture | Not yet built |

A built pack lives in `templates/styles/<name>/` and ships with the engine
as `@styles/<name>`, with the art its CSS names: `base.css` holds every
value as a `:root` token (with a `:root[data-scheme=light]` block),
`controls.css` dresses the core controls, one file per component group
(`rpg.css` … `general.css`) dresses that group's parts (scifi's and
cartoon's `components.css` some ungrouped components), and `motion.css`
and `sounds.css` hold its transitions and `-og-sound` sets; each pack's
README lists its files and tokens. `templates/styles/shared/` holds the
fonts, sounds and art packs share (`/@styles/shared/`) and
`templates/styles/cjk/` the CJK faces; neither is a pack.
`templates/styles/tests/fantasy-frost/` is a token-only variant of
`fantasy` that proves the pack restyles by tokens alone; it is not
shipped.

A pack becomes a creation choice only once it is accepted (§6.3); the
integration owner then updates the schema and bilingual labels together.
There are no `auto`, `custom`, or `generated` pseudo-packs.

The 12 look presets in `templates/looks/` and the generated look catalogue
own look-to-UI recommendations: toy-3d and cartoon-2d name cartoon,
painterly-3d and pixel-3d-hybrid fantasy, neon-vector scifi, the rest core
default. A recommendation is a seed, not proof of fit; an explicit
creation UI choice wins. Game-authored styles use explicit asset paths in
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
  components. `og ui types` types the built-ins the game uses. A built-in
  screen reads its declared props bare (`{title}`, `bind:open={open}`), as
  a component does, so one document is pushed with a viewmodel or mounted
  with stores; `$toasts` and `$captions` are the engine's stores, not
  props.
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

Built-in styles live in `templates/styles/<name>` and ship in the engine
package's `styles/<name>`, loaded as `@styles/<name>`
(`og.load.style(world, "@styles/scifi")`, or a look's `ui "scifi"`); the
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
parts, and the creation schema are contracts: a break needs a minor
release and a breaking entry with its fix in a new `changelog/unreleased/`
fragment. There are no legacy aliases. Planned groups and packs may be
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

A group's review sheet is a project in `packages/engine/sheets/<group>/`;
the engine package ships each one in its `sheets/` (without `references/`
or `verify.ts`), so `og ui sheet <group>` runs on an installed engine. No
game's cook bundles a sheet: it is data `og ui sheet` and `og ui review`
read when asked. Besides the groups, three sheets cover what no group
owns: `components` (the ungrouped components of
`src/ui/components/*.svelte`: a HUD page over a bright sky and a dark
floor, a panels page, a states page), `screens` (each built-in screen of
`src/ui/screens/` on a page of its own, in its document's fixture and the
sheet's own states, `--states` taking any of them) and `world` (the world
tags, ground marks and mass elements anchored over a 3D scene). A sheet's
pages use the group's built-in components by name, with fixtures from its
`src/fixtures.ts`, sample art under `assets/` and strings for the
review's CJK and right-to-left locales in `assets/strings/`;
`og ui review` installs those locales' font packs into its copy (without
them those cells repeat the game locale).
The group's behaviour tests (`packages/engine/tests/<group>.test.ts`)
mount its members the same way.

Core tests use small self-contained styles for core behaviour
(specificity, font pixel snapping, probe paths); pack tests check pack art
and resources. Removing a pack cannot remove core coverage.

### 6.2 Frame matrix

Each group's sheet has a normal page, a stress page (long names, dense or
overflowing content, 150 % text) and, for most groups, a `states` page
with every member in every fixture state it declares; behaviour tests
cover every member. A further page needs a named visual behaviour the
others cannot cover. The sheet's `project.ogt` declares its pages, and its
`STYLES` (`src/main.ts`) the styles it renders them in: each built pack
joins a sheet with a look naming it under `assets/looks/` (which makes the
cook bundle it) and its name in `STYLES`.

| Group | Normal page | Stress page | Other pages | Styles |
| --- | --- | --- | --- | --- |
| rpg | `inventory` | `journal` | `equipment`, `explore`, `combat` | fantasy, scifi, cartoon |
| shooter | `combat` | `loadout` | | fantasy, scifi, cartoon, default |
| strategy | `field` | `defence` | `states` | fantasy, scifi, cartoon, default |
| racing | `race` | `garage` | `states` | fantasy, scifi, cartoon, default |
| card | `table` | `builder` | `run` | fantasy, scifi, cartoon |
| puzzle | `play` | `map` | `states` | fantasy, scifi, cartoon, default |
| idle | `play` | `reset` | `states` | fantasy, scifi, cartoon, default |
| rhythm | `songs` | `play` | `states` | fantasy, scifi, cartoon, default |
| arena | `select` | `match` | `states` | fantasy, scifi, cartoon, default |
| general | `store` | `social` | `progress`, `states` | fantasy, scifi, cartoon |
| components | `hud` | `panels` | `states` | fantasy, scifi, cartoon, default |
| screens | each screen's default state | its `overflow` (or first other) state | every state through `--states` | fantasy, scifi, cartoon, default |
| world | `field` | `crowd` | | fantasy, scifi, cartoon, default |

`og ui sheet <group>` renders the screens `project.ogt` declares at
desktop (1920×1080) and phone portrait (390×844), at 100 % and 150 %
text; `og ui review <group> --style <pack>` renders every page in each
named style across `--locales`, as labelled contact sheets. A pack's own
core-control sheet is `og ui sheet <pack>` (`fantasy` or
`@styles/fantasy`). Landscape-heavy content also has a genuine
landscape fixture. Sheet captures are review
material: locale, right-to-left, landscape and further states get
risk-based review captures, and any permanent reference names the
regression it guards. The rpg sheet alone keeps approved cells
(`sheets/rpg/references/vulkan/`: its five pages at desktop, phone
portrait and phone landscape in fantasy), which its `verify.ts` compares
on demand, never as a gate step. A group's behaviour tests mount its
members from every fixture under the fantasy and scifi packs, on a desktop
and a phone; neither the tests nor a sheet is visual coverage of every
combination, and a game's actual combination needs review.

The references the engine keeps are og-tests-ui's
`references/vulkan/doc-components-<group>-<document>.png` (each document's
default fixture), `doc-screens-<screen>.png` and the `documents/` frames,
in the default style. Captures record effective style, theme, viewport,
DPR, text scale, locale, focus, selection, tick and animation history;
equal ticks do not prove equal state. References are recorded on the GPU
host (NVIDIA L4, Vulkan; docs/ci.md §The GPU host); other GPU backends are
compatibility checks under the cross-platform tolerance. A missing GPU is
unchecked, never passed.

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

## 7. Maintenance boundaries

The integration owner owns shared schema, labels, looks, creation,
packaging, this contract and the guides. Component group and pack authors
own their group or package directories, review sheets and acceptance
fixtures. Shared API changes go through the integration owner before
consumers adopt them.

A group's documents, string tables and README live in
`packages/engine/src/ui/components/<group>/`, its review sheet in
`packages/engine/sheets/<group>/`, its behaviour tests in
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

**The CSS dialect is the §8 inventory ([css-coverage.md](css-coverage.md)) at its pinned version, and it is
closed.** og-ui implements game-UI CSS, not a browser: a row is in scope
when a component, a style pack or a guide needs it, and work goes to
proving rows in use before adding rows. A feature outside the pinned
inventory, or a row the owner marks out of scope, is refused by the parser
by name, never half-rendered. Moving the pin to a newer web-features
version is a decision of its own.

## 8. Standard CSS output-proof checklist

The checklist of every WebDX CSS feature lives in
[css-coverage.md](css-coverage.md), generated by `scripts/css-coverage.py`
from the proofs it lists; it is never edited by hand. A row is
**Supported** when the output assertion it names proves the listed scope
(not every syntax grouped under that WebDX name), **Missing** for an
implementation gap or a missing output proof (the reason says which), and
**Not applicable** when the feature cannot apply to a game engine's UI
(print, cross-document navigation, browser chrome).
