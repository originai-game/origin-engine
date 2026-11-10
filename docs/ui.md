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
  and document-scoped `<style>`. There is no `<template>` wrapper; a
  `<script>` holds only props and derived values (below).
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
  A handler outside any `{#each}` row is the page's and takes its action
  wherever the focus is. A handler inside a row takes it only while a
  focus (the shared one or a pad's) is within that row, as a key event
  reaches the focused element and its ancestors: `ondetails={equip}` on
  every row of a list acts for the focused row alone, with its row key.
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
  use cooked string tables. In the pseudo-locale (`og ui check`'s pseudo
  cells, `og run --locale pseudo`) their text is shown accented, 40 %
  longer and in brackets, as og.i18n.t's is (`og_i18n::pseudo_text`).
- JSX, `$state`, `$effect`, `$bindable` outside a prop's default,
  statements, imports and functions in a script, `{@html}`, `on:click`,
  `<svelte:document>`, `<svelte:boundary>` and the Svelte 4 elements are
  refused, each with the Svelte 5 or TS-system form to use. `transition:`,
  `in:`, `out:`, `animate:`, `use:`, `style:`, `bind:this`, `{@const}`,
  `{#key}`, `{#await}`, `<svelte:window>` and `<svelte:element>` are
  supported as below. Parse errors identify file, line, column, and a fix.

#### Svelte blocks, script and directives

Each is Svelte 5's meaning mapped onto what og-ui already does; none runs
code in the tick.

- `{#key expr}…{/key}` makes its content again when `expr`'s value
  changes: its elements are new (their ids carry the value's hash, so
  their paths change), the old ones leave with their exits and the new
  ones arrive with their entrances.
- `{@const name = expr}` names an expression for the rest of its block;
  it sits directly inside `{#if}`, `{#each}`, `{#key}`, `{#await}`,
  `{#snippet}` or a component. It is replaced by its expression where it
  is read, so it costs no binding. A name for a non-path expression is
  read whole (`{total}`, not `{total.x}`).
- `{#await $load}…{:then value}…{:catch error}…{/await}` (and the short
  `{#await $load then value}`) reads a store or row field holding a
  settled record, `{ status: "pending" | "fulfilled" | "rejected", value,
  reason }` (`Promise.allSettled`'s words): the pending branch until
  `status` is `fulfilled` or `rejected`, then `{:then}` with `value` or
  `{:catch}` with `reason`. The TS system that starts the work writes the
  record; no promise runs in the tick.
- `<script lang="ts">` holds `let { a, b = 1, c = $bindable('x') }: Props
  = $props()` (literal defaults: numbers, strings, booleans, null,
  arrays and objects of them), `const d = $derived(expr)` and `const e =
  $derived.by(() => expr)` (or `() => { return expr; }`) over pure
  expressions, and `interface` and `type` declarations, which are erased.
  A prop's type is its default's (`any` without one) unless a `<meta
  name="prop:…">` states it; a component user who leaves a prop out reads
  its default, and the generated `.d.ts` marks it optional. A declared
  prop with no default is required: a mount whose document, or a
  component it uses, leaves one out returns an error naming the instance
  and the prop, instead of mounting nothing. Derived values
  are names for their expressions, replaced where read. Anything else
  (`$state`, `$effect`, imports, functions, other statements) is refused:
  state and logic live in a TS system that writes the stores.
- `transition:name`, `in:name` and `out:name` (`|local` and `|global`
  accepted) set the element's `enter`, `exit` or both from Svelte's
  `fade`, `fly` (`x`, `y`, `opacity`), `slide` (`axis`: it scales along
  the axis; the layout does not shrink), `scale` (`start`, `opacity`) and
  `blur` (`amount`, default 5 px, and `opacity`: the subtree draws
  through CSS `filter: blur()`'s layer, its blur moving with the
  opacity), with `duration` (ms, default 400), `delay` and
  `easing` named from `svelte/easing` (cubic Béziers; the elastic and
  bounce curves are springs). Params are literal: `transition:fly={{ y:
  20, easing: quintOut }}`. `in:receive`, `out:send` or
  `transition:crossfade` with `{{ key: item.id }}` is the shared-element
  key (`match`). `draw` (`duration`, `delay`, `easing`, default
  `cubicInOut`) on an SVG shape strokes it on from its start, as Svelte's
  does: one dash and one gap the outline's length, the offset moving from
  the length to 0 (`out:draw` takes it off the same way); its fill stays.
  `speed` is refused (it needs the path's length): give `duration`.
- `animate:flip={{ duration, easing }}` on an element directly inside a
  keyed `{#each}` is its layout motion (default 300 ms, `cubicOut`).
- `style="width: 40px; color: red"` and `style:prop="value"` or
  `style:prop={expr}` (any supported property; `style:--name` is a custom
  property, as before) are the element's own declarations: they win over
  every stylesheet rule but an `!important` one, and `style:prop|important`
  wins over that too. A bound value is written with its unit, `style:width
  ={$w + 'px'}`.
- `bind:this={name}` names the element: it is its id, and `name` reads
  as that id wherever an expression names an element (`popovertarget`)
  and as `#name` in the stylesheet.
- `use:name` or `use:name={params}` sends a DocEvent of kind `use` named
  `name` with value `{ phase, params }`: `create` when the element is
  made, `update` when its params' value changes and `destroy` when it
  goes. The game's handler for `name` (a TS system) does what Svelte's
  action function would.
- `<svelte:window onresize={action} bind:innerWidth={w}
  bind:innerHeight={h}/>`, beside the root, sends `action` when the
  screen's size changes and names its size in CSS px (`$ui.width` and
  `$ui.height`, which any document reads too). Keys reach a game as
  input actions, so `onkeydown` is refused with that hint.
- `<svelte:element this="h2">` is that element; `this={$tag}` is the text
  tag it names (`p`, `span`, `small`, `h1`–`h6`) when its content is
  inline, or the block tag (`div`, `section`, `header`, `footer`, `nav`,
  `main`, `aside`, `article`), and nothing for another value. Its
  descendants take no ids, since each tag is a branch of its own.

Structure and snippet content remain documents, never TypeScript trees.

Core elements use HTML's tag and attribute names wherever HTML names the
role; a pre-HTML tag is refused with a diagnostic naming its HTML form.
The internal kind names (in the catalogue headings, the IDL descs and the
probe) are unchanged. The mapping:

| Document | Core kind | Notes |
|---|---|---|
| `<input>` (`type` text, password, number, email, url, tel, search) | field | `placeholder`, `maxlength`, `minlength`, `readonly`, `required`, `pattern`, `min`, `max`, `step` (`any`), `aria-invalid`, `list` naming a `<datalist>` of suggestions (below); CSS `appearance: none` drops its frame |
| `<textarea rows>` | field (multi-line) | the same constraints; `rows` lines shown |
| `<input type="range">` | slider | `min`, `max`, `step`, `disabled`, `list` naming a `<datalist>` (below); `data-fill-origin="<value>"` sets the value its fill grows from |
| `<input type="range" bind:low bind:high>` | slider (range) | two knobs on one track, `low` to `high` (HTML meter's names), the fill between them: a press takes the nearer knob, each knob is its own focus stop that Left and Right step, and neither passes the other; `data-min-gap="<value>"` keeps them apart. The readout is `low – high`; to a screen reader they are two sliders, `<name>, minimum` and `<name>, maximum` |
| `<datalist id>` of `<option value [label]>` | — | draws nothing: a range's `list` puts a tick at each option and its `label` (or text other than the value) under the track. A text input's `list` takes its options' values, or `<datalist id items={$names}>`'s, as suggestions: a list under the field while it has the focus (dressed as a select's, `Dropdown.list` and `Dropdown.option`), those containing the text, case and accents ignored, prefix matches first, at most 8; with no text, the first ones as written. The focus stays in the field as the player types; Down reaches the list, choosing one writes it to the field and returns the focus, and Back closes it until the player types again |
| `<input type="checkbox" switch>` | toggle | `checked`, `disabled`; size `::part(track)` and `::part(knob)` with CSS dimensions; tick geometry and selection travel follow their measured boxes |
| `<input type="checkbox">` | toggle | `checked`; drawn as a switch (og-ui has one checkbox control) |
| `<label><input type="checkbox"> Caption</label>` | toggle | the label's text is the caption; the two are one node. The caption sits where it is written (after the switch here, before it in `<label>Caption <input …></label>`); `flex-direction: row-reverse` on the label swaps it. A `<small>` inside is its description line |
| `<label for="id">` | — | joins the control `id` names: a checkbox takes its text as its caption (one node, as the wrapping form); an input, textarea, select, range, meter or progress takes it as its accessible name and the label stays a text beside it |
| `<hr>` | divider | |
| `<img>` | image | `src`, `alt`; CSS `object-view-box: inset(<t>% <r>% <b>% <l>%)` (percentages of the texture) shows a part of it; `object-fit`, `object-position` and `image-rendering` (see Images below) |
| `<p>`, `<span>`, `<small>`, `<text>` | label | `<text>` is the core name; inline `<b>`/`<strong>`, `<i>`/`<em>`, `<u>`, `<s>`/`<del>`, `<sub>`/`<sup>`, `<br>`, a plain `<span>` and `<a href>` style runs of it |
| `<h1>` … `<h6>` | label (heading) | role `heading` with `aria-level` 1–6; UA type steps `xl3`, `xl`, `lg`, `body`, `sm`, `xs` |
| `<a href>` | label (a link) | a click sends a `link` event named by its href through the document's events, as `onclick` sends `emit`; accent ink and an underline |
| `<div>`, `<section>`, `<header>`, `<footer>`, `<nav>`, `<main>`, `<aside>`, `<article>` | box | layout from CSS `display` (below); landmarks take their roles: `navigation`, `main`, `banner`, `contentinfo`, `complementary`, `article`, and `region` for a named `<section>` |
| `<ul>`, `<ol>`, `<li>` | box (list, listitem) | not virtualised (that is `<list>`); markers from `list-style` (below); `<ol start reversed>`, `<li value>` |
| `<button>`, `<progress>` | button, progress | already HTML; a `<button>` holding elements (`<og-icon/>`, text, `<badge>`) lays them out as a centred row inside its frame, states and press, and its text and icons without a colour of their own take its ink |
| `<select>` with `<option>` and `<optgroup>` | select | `value`, `placeholder`, `disabled`, `aria-invalid`; options take `value`, `label`, `i18n`, `disabled`, `selected`; `select::picker(select)` styles the open list as a box and its `max-height` bounds the rows |
| `<dialog>` | dialog | `open`, `modal`, `closedby`, `aria-label`; CSS `appearance: none` drops its frame |
| `<details>` with a first `<summary>` | details | `open`, `disabled` |
| `<meter>` | meter | `value`, `min`, `max`, `low`, `high`, `optimum` |
| `<table>` with `<caption>`, `<thead>`, `<tbody>`, `<tfoot>`, `<tr>`, `<th>`, `<td>` | box (table, row, cell, columnheader, rowheader) | laid out by CSS table layout: the UA stylesheet gives each part its `display` (`table`, `table-caption`, `table-header-group`, `table-row-group`, `table-footer-group`, `table-row`, `table-cell`; cells `padding: 1px`, `<th>` bold and centred, the caption centred), and the layout reads a cell's `colspan` (1–1000) and `rowspan` (0–1000; 0 runs to the end of its row group); each part sits where HTML puts it, `{#each}` rows included. The first `<caption>` names the table. A `<th>`'s `scope` (`row`, `col`, `rowgroup`, `colgroup`) picks row or column header; without one it heads its column in a `<thead>` or a row of headers, and its row beside data cells |
| `<form onsubmit={save} onreset={clear}>` | box | a `<button>` in it submits it (its default `type`; `type="reset"` resets, `type="button"` neither), sending the form's action as a click sends `onclick`'s; a submit or reset button outside a form is refused. A submit is constraint-validated as HTML's: while a text field in the form is `:invalid` (its `type`, `required`, `pattern`, `minlength`, `min`, `max`, `step` or `invalid` refuse its value; disabled and read-only fields are barred), the action is not sent and the first invalid field, in document order, takes the focus. `novalidate` on the form or `formnovalidate` on the button sends it unchecked. In an `{#each}`, only the button's row's fields count |
| `<fieldset disabled>` with a first `<legend>` | box (group) | the legend's text names the group; `disabled` (literal or bound) disables every control in it but those in the legend, joined with a control's own `disabled` |
| `<figure>` with `<figcaption>` | box (figure) | the figcaption's text names the figure |
| `<output for="ids">` | label (status) | its text, read politely as it changes; `for` names the controls it reads by id |
| `<input type="date">` (`time`, `datetime-local`, `month`, `week`) `bind:value={$when}` | datetime (group) | a pad-friendly picker of og-ui steppers, one per part (Year, Month, Day, Week, Hour, Minute, and Second when `step` or the value has seconds), each a focus stop that Left and Right, a press or a held press step; no OS picker. A part stepped past its end carries into the next (the 30th of February is March 1st), a month or year moved keeps the day in its month. The value is HTML's (`2026-11-08`, `14:30`, `2026-11-08T14:30`, `2026-11`, `2026-W45`, ISO weeks), kept within `min` and `max` and on the `step` grid from `min` (seconds for times, days, months or weeks otherwise; a 900 s step moves the minutes by 15). Empty, it shows `min` (else `max`, else 2000-01-01 00:00; no clock is read) and writes once a part is stepped. `required`, `disabled`, `readonly`; empty and required, or outside `min`…`max`, is `:invalid` and holds a form's submit back. Parts `Stepper.*`; probe `<id>/<Part>/Frame/Inc` |

`<video src={$player}>` and `<audio src={$player}>` play a player the game
owns: the entity (from a store) of an og.VideoPlayer, whose frame draws as
the element's picture, or of an og.AudioSource. Svelte's media bindings
follow it: `bind:paused` plays and pauses it (written back true at the end
without `loop`), `bind:currentTime` is written back as it plays and seeks
it when the store moves it elsewhere, `bind:volume` and `bind:muted` set
its sound's gain, `bind:duration` and `bind:ended` are written back;
`autoplay`, `loop`, `poster={$texture}` (shown until the first frame) and
`controls` (a Play/Pause button and its time). The element sends what it
asks as `og.MediaIntent` through the recorded intent path, as UI pause
does, and the `og.video` system applies it at the next tick, so playback
stays world state; what it reads back is the last tick's. An element
asking nothing sends nothing; a media element is rebuilt every frame while
mounted, as a slot is, since its picture and time change without a
binding. `src` is never a file name: a document does not spawn entities,
so spawn the player in a system with its asset and bind the entity. An
`<audio>` has no `currentTime`, `duration`, `ended` or `poster` (a sound's
playhead is not read).

`<template>` is refused: snippets are its Svelte form. `<input
type="file" accept multiple bind:value={$files}>` is a button only where the
host opens a file picker (`og.ui.filePicker`, the same query on the web and
natively; false on both today): a press sends the host a `filepick` event
with `accept` and `multiple`, and the host writes the names chosen to the
bound value. Elsewhere it is not built and the frame warns.
`<field>`, `<slider>`, `<toggle>`, `<divider>`, `<image>` and a
`<label>` that neither wraps a checkbox nor has `for` are refused. CSS type selectors match the written tag (`input`,
`textarea`), so `input[type=range]` and `textarea` select as in HTML.
Elements HTML has no name for (`column`, `row`, `stack`, `grid`, `panel`,
`scroll`, the virtualised `list` and `cells`, `art`, `icon`, `tabs`,
`badge`, `spacer`) keep their names.

#### Inline objects, span parts and translated markup

A text element's words may hold inline objects, each one em tall in the
line: `<img src="coin" alt="">` (a picture the game registered,
`UiDesc.images`), `<og-icon name="coin">` (the style's atlas) and
`<kbd data-action="confirm"></kbd>` (the action's glyph on the device
used last). Their attributes are literal. A `<span part="big">` names its
run for `p::part(big) { … }` (any text element's `::part`), which takes
`color`, `font`, `font-family` (named and generic families in authored order,
as an element's),
`font-weight`, `font-style`, `text-decoration-line`, `-og-text-motion`,
`vertical-align`, and the
run's `font-size` (px), `line-height` (px, or a multiple of its
font-size) and `letter-spacing` (em, or px of its font-size).

A run's `vertical-align` follows CSS 2.1 §10.8 and CSS Inline 3:
`baseline`, `sub` (a fifth of the paragraph's font size down), `super`
(a third up), `text-top` and `text-bottom` (on the paragraph font's
ascent or descent), `middle` (its middle half the paragraph font's
x-height over the baseline), a length (`4px`, `0.3em` of its own size)
and a percentage of its own line height move it against the
paragraph's baseline; `top` and `bottom` put it on the line box's top
or bottom. Each run is measured with half its leading above and below
it, and its line grows to hold it and the paragraph's own line (its
strut); a line of baseline runs only keeps its height. `<sub>` and
`<sup>` are runs with `vertical-align: sub` and `super`, their size the
smaller one (÷ 1.2), as HTML's user-agent sheet makes them. A translated string
(`i18n`) styles its runs with MessageFormat 2 markup: `{#b}…{/b}`
(`strong`), `{#i}` (`em`), `{#u}`, `{#s}` (`del`) and `{#br/}`; `\{`
escapes a brace.

#### Glyphs and the `$ui` store

`<kbd>Esc</kbd>` is a keycap. `<kbd data-action="jump"></kbd>` is the
action's glyph on the device used last: its key on a keyboard, its button
on a pad (the pad family's), its touch icon under a finger; the style's
icon where the atlas has one, else the name on a cap. It is named by the
action's label and rebuilt when the device changes. With
`data-og-rebind bind:value={$jump}` it is a button whose press captures
the next key (its code, `KeyQ`) into the store; Escape cancels. The game
applies the binding through og.input.

Every document reads the engine's `$ui` store (a mount that binds its own
`ui` keeps it): `$ui.device` (`keyboardMouse`, `xbox`, `playstation`,
`switch`, `touch`…), `$ui.coarsePointer`, and `$ui.actions`, the action
bar of the top layer, each `{ action, hint, icon, text, hold }`. An
action hint row is `{#each $ui.actions as a (a.action)}<kbd data-action={a.action}></kbd><span>{a.hint}</span>{/each}`.
It is published only when it changes, so only what reads it rebuilds.

#### ARIA

Any element takes ARIA over what its kind tells the accessibility tree:
`role` (one of og.ui's roles; another is refused at parse),
`aria-label`, `aria-labelledby` and `aria-describedby` (the text of the
elements their ids name: one id's text follows it when bound, several
ids join their literal texts), `aria-description`, `aria-valuemin`,
`-valuemax`, `-valuenow` and `-valuetext`, and the states
`aria-checked`, `aria-selected`, `aria-expanded`, `aria-pressed`,
`aria-busy`, `aria-disabled` (also inert), `aria-level` and
`aria-current` (the state `current`). A tab bar is
`<div role="tablist"><button role="tab" aria-selected={…}>…</button></div>`,
a chip `<button aria-pressed={…}>`, a labelled rule
`<div role="separator">Or</div>`. They are applied with the element's
other fields, so a bound one updates the tree when its element rebuilds.

#### Checkboxes, radios, steppers, menus and press events

`<input type="checkbox">` is HTML's box with a check, `indeterminate` a
dash in it (a press turns it on); with `switch` it is a switch. `<input
type="radio" value="easy" bind:group={$level}>` is one choice of a group:
checked while the bound store holds its value, which a press writes;
radios binding one store are one group (`name` is accepted and adds
nothing). Each takes its caption from its `<label>` as a checkbox does.
`<input type="number">` keeps an editable text field with increment and
decrement buttons, bounded by `min`, `max` and `step`. `data-wrap` wraps
at those bounds; `data-large-step` controls Shift adjustment and held repeats.
Formatting through `data-format` applies only while unfocused. The raw string,
selection, validation, readonly focus and pad keyboard remain available.
`<menu>` is a box with role menu whose buttons are menu items.
Beyond `onclick`, an element fires `onhold` once per press held for its
`data-hold` ms (default 500), `oncontextmenu` on a secondary press or a
long press, `ontoggle` when a `<details>` or popover opens or closes, and
`onclose` when a `<dialog>` closes.

#### Counters, typewriters, cyclers, spotlights and colour inputs

A `<og-counter value={$score} duration="900" decimals="0">` counts to its
value as it changes; duration is in milliseconds. A
`<og-typewriter rate="20">` reveals extended grapheme clusters per second,
restarts whenever its text changes, shows the whole text under reduced
motion, and fires `onrevealed` once at completion. A `<select data-cycle>` is
previous, its option and next, wrapping past disabled options; one with no
`<option>`s takes its bound `items`. An open `<og-spotlight target="Target">`
dims the page but for the element of that id, which
it outlines, and shows its content as a callout against it; the shade takes
presses, and `closedby="any"` closes it on one. `<input type="color"
bind:value={$tint}>` is a swatch that opens a picker against it (a
saturation and value square over the hue, a hue slider and the hex field)
and writes `#rrggbb` back. Each keeps its progress in widget memory, so a
snapshot restores it mid-count or mid-reveal.

The engine ships component documents in place of the TS builders, under
`packages/engine/src/ui/screens/` beside the screens, each with its props,
actions and a default fixture: `card`, `confirm` (a modal dialog with
cancel and confirm, the danger variant when `$danger`), `pagination`,
`data-list`, `key-values`, `table`, `tree`, `calendar`,
`objective-tracker`, `star-rating`, `banner`, `subtitles` (the HUD layer)
and `notifications` (the toast layer). Their lists are keyed collections
read directly by `{#each}`; a game ejects one (`og ui eject <name>`) to
change it.

The settings screen (`settings.svelte`) builds its tree from data: `$pages`
(a keyed collection of `{ id, name }`, a tab each), `$entries` (a keyed
collection of `{ id, page, name, description, kind, value, … }`, kind
`toggle`, `range`, `select` or `binding`, each value written back as the
player changes it), `$page` and `$pending` (Apply and Revert take presses
only then); with `$page` empty it shows its `tree` content alone.

#### Tooltips and popovers

`title="…"` on any element is its tooltip, shown after the pointer rests
on it, and its description in the accessibility tree. An element with
`popover` (HTML's `auto`, also bare; `manual`; `hint`) is not in the
page: it takes no room there and shows, open, in a layer over it, placed
against the control that opened it (its implicit anchor) by anchor
positioning: `position-area` picks the side and alignment, an inset
offsets it, `position-try-fallbacks` flips it inside the safe area; with
none of them it hangs under the control (above it where there is more
room, kept on screen). It is a raised panel (`--panel-raised`, the
panel radius; a `hint` the tooltip's), as a browser's popover has its
own canvas, and its CSS overrides that box and look; its
`::-og-arrow { width; background }` draws on the side facing the anchor.
Its content keeps HTML's margins, so a `<p>` in it has a line's margin
above and below, as in a browser (`margin: 0` or a `<span>` for a tight
tip). `contextmenu="id"` on an element opens popover `id` at the point of
a secondary press or long press on it, that zero-size point its anchor:
its corner sits at the point and it draws no arrow, as a desktop's and a
game's context menus do. A `<menu>`'s buttons are menu items, each as
wide as the menu with its label at the start. A `<button>` places its
label by its `text-align`, centred by HTML's user-agent sheet (`button {
text-align: center }`), so it does not inherit its parent's. A control's
`popovertarget="id"` opens, closes or toggles it on a press
(`popovertargetaction="show|hide|toggle"`, default toggle) and reports
`expanded`. An `auto` popover closes on a press outside it or Back and
closes the other `auto` ones as it opens; a `manual` one only when a
control closes it. A `hint` is a rich tooltip: `interestfor="id"` on a
control shows it while the pointer rests on the control (the tooltip's
delay) or the control holds the keyboard focus, and while the pointer is
over the hint itself. A popover is built every frame (the controls that
open it are other elements); whether it is open is kept in snapshots.

#### HTML blocks and lists

A box (`<div>` and the other block tags) lays out by its CSS `display`:
`block`, the default, stacks its children top to bottom as a `<column>`
does; `flex` lays them out as a `<row>` (`flex-direction: column` turns
it); `grid` lays them out by CSS Grid, below;
`none` hides it. `display` takes every CSS Display 3 value, single or
multi-keyword (`inline flow-root` is `inline-block`). `flow-root` is a
block whose children's margins stay inside it. `contents` generates no
box: its children lay out as children of its parent (a flex or grid
container's items), and its own background and border are not drawn.
An element displayed `inline`, `inline-block`, `inline-flex`,
`inline-grid` or `inline-table` is inline-level: a block holding one
lays its children out in lines, the texts and images beside it
inline-level too, each block-level child on a line of its own; the
lines align on their baseline, by the block's `text-align`, and an
inline-level child's `vertical-align` places it in its line: `top` and
`bottom` on the line's top or bottom, the other values as a text run's
(below) against the line's baseline and the block's first text, a box
with no text taking its bottom edge as its baseline. Layout places it
while it computes heights: its line box is as tall as the block's strut,
its baseline-aligned boxes and its shifted and top- or bottom-aligned
boxes reach (CSS 2.1 §10.8), so a raised box grows its line. A `<p>`, a
heading or a text displayed `block` is block-level: in a block's flow it
is as wide as the block's content box (CSS 2 §10.3.3), as a block box
is, and on a line of its own among inline-level children; `<span>`,
`<a>`, `<small>` and a text displayed `inline` or `inline-block` are
inline-level and as wide as their text. An inline-level element is laid out whole (an
atomic inline): an `inline` box's text does not break across the
lines of its parent; inline markup inside a text (`<b>`, `<span>`,
`<a>`) is runs of that text and flows across lines. Other elements'
layout is their tag, so `display` on them takes only `none`, `block`,
`inline`, `inline-block` and `flow-root`.

`display: table` lays a box out as a CSS table (CSS 2 §17, with
HTML's table processing model): its `table-row` children, those of its
`table-header-group`, `table-row-group` and `table-footer-group`
children (the first header group first, the first footer group last),
and their `table-cell` children, each cell in the next free slot of
its row, spanning its `colspan` columns and `rowspan` rows (`0`: to
the end of its row group). Children that are no table part are put in
anonymous rows and cells. Columns and rows are as wide and tall as
their cells need, a spanning cell's need spread over its tracks; a
table without a `width` shrinks to fit, and one with a `width` shares
the extra room among its columns. A `table-caption` spans the table
above it, or below it with `caption-side: bottom`. A `table-column`
box's px `width` is its column's least width. Row, row group and
column boxes are drawn over their slots under the cells, so their
backgrounds show. A cell's content sits in the middle of its row
unless its `vertical-align` is `top`, `bottom` or `baseline`. The
`border-spacing` and `border-collapse` properties are not supported:
cells are separate, `gap` spaces them, and a cell or row part outside
a table lays out as a block. Text between a box's elements is an anonymous
text element, as an HTML anonymous block box is; `i18n` on a box
translates its one run of text.

`position` takes `static`, `relative`, `sticky`, `absolute` and `fixed`
(CSS Position 3). `relative` moves the box by its insets after its parent's
layout, its siblings keeping its room; `sticky` keeps it in the flow and
holds it inside its nearest scroller's view by its insets, never past its
parent's content box. An absolute box is placed in its parent's box and
a fixed one in the whole surface. The insets are `top`, `right`,
`bottom`, `left`, the `inset`, `inset-block` and `inset-inline`
shorthands and their logical longhands; each takes px, `%` of the
containing block, `auto` or `anchor()`. `left` is the left in either
direction (Direction and writing modes, below). An absolute or fixed box
is sized and placed as CSS 2.1 §10.3.7 and §10.6.4 say: with an `auto`
width (or height) between two set insets it fills the room between them
less its margins; with one inset or none it shrinks to fit (its
max-content, at most that room, at least its min-content), so a HUD
corner is as wide as its readout. A definite size between two insets
keeps its size and its bounds (`max-width`), and its `auto` margins share
what is left, so `inset: 0; margin: auto` centres it (across, a start
margin never goes below 0); with no `auto` margin an over-constrained box
keeps its start inset. Its margins hold it off the insets it is placed by.

Direction follows CSS Writing Modes 4. `direction` (`ltr`, `rtl`,
inherited) and HTML's `dir` attribute (`ltr`, `rtl`, or `auto` for its
text's first strong character) set an element's inline direction; a
document's root takes the surface's (the locale's, or `og.ui.configure`).
A right-to-left element runs its flex rows, grid columns, columns and
lines from the right, and its `start` text sits on the right. Logical
properties (margins, padding, borders and their radii, insets, sizes,
`overflow-inline`/`-block`, `scroll-margin`/`-padding`) name the sides of
the element's own direction; physical ones (`left`, `margin-left`,
`text-align: left`, `justify-content: right`, transforms) stay where they
say. A right-to-left surface still lays a right-to-left document out
mirrored as a whole; a document's physical sides and transforms keep
their places under it. `unicode-bidi` takes `normal`, `embed`, `isolate`,
`bidi-override`, `isolate-override` and `plaintext`. Each element's text
is a paragraph of its own (`embed` reads as `isolate`), set in its
direction once `dir`, `direction` or `unicode-bidi` is declared on it or
an ancestor. Without one, or under `plaintext`, a paragraph takes the
direction of its first strong character, as og-ui's own text does. The
overrides lay its characters out in its direction, unreordered.

`writing-mode` (`horizontal-tb`, `vertical-rl`, `vertical-lr`,
`sideways-rl`, `sideways-lr`; SVG's `lr`, `rl`, `tb` and their `-tb`/`-rl`
forms; inherited) turns an element's lines. In a vertical mode lines run
down (up in `sideways-lr`) and stack from the right (`-rl`) or the left
(`-lr`); a right-to-left direction starts them at the bottom (the top in
`sideways-lr`). Each element whose writing mode differs from its parent's
lays out in a frame of its own: its logical properties, block flow,
margins, flex lines, grid tracks, tables, columns, floats (`left` is
line-left: the top, or the bottom in `sideways-lr`), `clear`, insets and
scrolling all follow its inline and block axes. To its parent it is an
orthogonal flow (CSS Writing Modes 4 §7.3): its inline size fits the
containing block's size on that axis, else the viewport's, and its block
size is its content's; a block in a vertical writing mode fills its
height. `text-orientation` sets a vertical line's glyphs: `mixed` stands
CJK, kana, Hangul and symbols upright (Unicode's vertical orientation,
UAX #50) and lays other scripts down turned clockwise, `upright` stands
every character up, `sideways` (and the `sideways-*` modes) lays every one
down, `sideways-lr` turned counter-clockwise. An upright character
advances by its font's vertical metrics (`vmtx`, else one em) and takes
its vertical form (`vert`, `vrt2`) from the face that draws it, the
first face of the font chain or a fallback's (the cook keeps every
face's vertical forms of its characters in its atlas); without one,
brackets and dashes turn and the ideographic comma and full stop move to
the em's upper right. `text-combine-upright: all` (and `digits <n>` for a
text of up to n digits) sets an element's text side by side in one upright
em, squeezed to fit. Inline-level boxes lie in vertical lines too, each
line's boxes centred on its central baseline. A justified vertical line
but its paragraph's last takes its free room at its spaces, else between
its upright clusters. Vertical text takes the player's selection down its
lines (a press picks the character nearest along its column),
`::selection`'s ink, shadows and lines (under on the column's left, over
on its right), the style's text glow and drop. Rich text runs down its
columns too, its spans in their fonts, inks, weights and outlines,
pictures and icons upright, decorations along the column; its lines
break by what each unit takes down the column (upright clusters at their
vertical advances, sideways runs at their widths), and it takes the
player's selection, its fill and `::selection`'s ink, shadows and lines,
as vertical text does: its glyphs are cut at the selected stretch and
drawn there in the selection's style, so a translucent `color` does not
blend with the text's own. Text on a path keeps its path's physical box whatever the writing
mode, but in a vertical element the path is a vertical line: each
upright cluster stands on it one vertical advance long, its top back
along the path, and a sideways run lies along it. A
container variant of `Props` (`Query::container`) reads its element's
inline size: a vertical element's height.

CSS Anchor Positioning 1 places an absolute or fixed box against other
boxes as this frame laid them out. `anchor-name: --a` names an anchor
(the last in tree order wins, never one inside the positioned box);
`position-anchor: --a` picks the default one, else its implicit anchor
(a popover's invoker). An inset takes
`anchor([--a] top | right | bottom | left | center | start | end | inside
| outside | <n>%[, <fallback>])`, alone or in `calc()` plus or minus px;
`width` and `height` take `anchor-size([--a] width | height | block |
inline)`, times a number or plus px. `position-area` places the box in
the cell or span of the 3×3 grid the default anchor's edges cut its
containing block into (physical, logical and `span-*` keywords; a
centre or span-all region aligns as `anchor-center`), sized to fit the
area. `position-try-fallbacks` lists `flip-block`, `flip-inline`,
`flip-start`, `@position-try --name` rules (their insets, sizes and
`position-area` over the box's own, `auto` clearing one) and
position-areas: the first option that does not overflow the containing
block wins, else the box's own; `position-try-order` (`most-width`,
`most-height`) and the `position-try` shorthand order them.
`position-visibility: anchors-visible` (the default) hides the box while
its default anchor is clipped out of view, `no-overflow` while no option
fits. The option that won and the anchor side it sits on are kept per
frame for a popover's arrow.

`float` (`left`, `right`, `inline-start`, `inline-end`) and `clear`
(`left`, `right`, `both`, `inline-start`, `inline-end`) follow CSS 2
§9.5 in a block's flow; in a flex, grid, table or multi-column container
`float` does not apply. A float leaves the flow, shrinks to fit (its
`auto` margins are 0) and is shifted to its side of its parent's content
box, as high as it fits: beside the earlier floats on that side, else
below the first that leaves room, never above an earlier float. Floats
belong to their block formatting context, so one in a plain block
reaches the boxes after that block. Every line box beside a float is
shortened to the room it leaves (CSS 2.1 §9.5), each broken at its own
room, and a line whose first word or box does not fit moves down past
the float: a text's lines, a rich text's (its paragraph, with its spans,
`::part` runs and inline elements, shaped once and broken one line at a
time at each line's room, so bidi and soft hyphens see the whole
paragraph), and a block's lines of inline-level boxes (`inline-block`,
images, `inline-flex` and `inline-grid` boxes, texts), filled box by box.
A block that lays its children out in lines takes part in its parent's
formatting context, so a float inside it reaches the lines and boxes
after it. A float among those lines is placed where it comes in the
content: at the top of its line when it fits beside what that line
already holds, else at the next line's top (§9.5.1). A block-level box
among the lines is a block of its own (the anonymous blocks of CSS 2
§9.2.1.1): it spans the line box over the float, and its content flows
around the float as in §9.5. One that starts a context of its own is
fitted beside the float like any other. A box
that starts a formatting context of its own (`flow-root`, a flex or
grid box, a table, a scroller, an image, a widget) does not overlap a
float: one whose
width fills the line is narrowed to the room beside the floats, one
whose width is fixed moves down until it fits. `clear` places a box
below the earlier floats on its sides. A formatting context root (a
`flow-root`, a float, a flex or grid item, a cell) is as tall as its
floats; a plain block is not, and its floats overflow it. A
right-to-left containing block (its `direction`, or the surface's)
puts `inline-start` floats on the right; `left` and `right` stay where
they say.

CSS Multi-column Layout 1 lays a block's flow in columns. `column-count`,
`column-width` and the `columns` shorthand make it a multi-column
container; the used count and width follow §3.4 (as many `column-width`
columns as fit, at most `column-count`, widened to fill), with
`column-gap` between them (`normal`: 1em). Each child lays out at the
column width. `column-fill: balance` (the default, and `balance-all`)
evens the columns' heights; `auto` fills them in turn to the container's
definite height, and columns past the count overflow inline. A
`column-span: all` child spans every column and starts a new balanced set
below it. `column-rule` (and its `-width`, `-style` and `-color`
longhands; `solid`, `dashed`, `dotted` and `double` drawn, the 3D styles
as `solid`) is drawn in the middle of each gap between two columns that
hold content, as tall as the set. The flow breaks between children and
between a text's lines, two lines kept on each side of a break (`orphans`
and `widows` at their initial value); `break-inside: avoid` keeps a text
whole, `break-before` and `break-after` `column`, `page` or `always`
force a break (and the legacy `page-break-*`). Any other box is
monolithic: it moves to the next column whole. A split text's box is its
first fragment; its later lines are drawn in the columns they fell in.

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
outside the alphabetic and roman ranges. `list-style-image: url(name)`
(also in the shorthand) draws a picture the game registered
(`UiDesc.images`) in the marker's place, one em tall.

Generated content follows CSS Generated Content 3 and CSS Lists 3.
`::before` and `::after` take `content` (strings, `attr()`, `counter()`,
`counters()`, `open-quote`, `close-quote`, `no-open-quote` and
`no-close-quote`; text after `/` is alternative text and is dropped),
their own `counter-*` properties, `display: none` and the text properties
a `::part` run takes (`color`, `font`, `font-family`, `font-size`,
`font-weight`, `font-style`, `line-height`, `letter-spacing`,
`text-decoration`, `-og-text-motion`). On a text element they join its text
as styled runs, and their other properties style nothing. On any other
element each is a real box, its first or last child: every property a
`<div>` takes (`display`, which is `inline` by default, sizes, padding,
borders, background, position, transforms, transitions…) applies through
the same pipeline, and the generated text is a run inside it. A
pseudo-element is a synthetic node of its element, made at cook
(`DocNode::pseudo`): its rules are its selector's declarations under the
element's condition, matched against the element (states, attributes,
`:nth-child`), so it is never one of the element's children to
`:nth-child`, `:empty` or `+`. It resolves with its element, inherits from
it and is retained with it: it adds no binding and no reconciled element
of its own. The box exists only while `content` makes one (not `normal`,
`none` or `display: none`). `counter-reset`, `counter-increment`, `counter-set` and `quotes`
work on every element. Counters follow CSS scoping: an instance covers its
element, the element's descendants and its following siblings. `list-item`
is an item's number. Counter styles are the predefined ones (`decimal`,
`decimal-leading-zero`, the roman, latin and greek alphabets, `cjk-decimal`,
`hiragana`, `katakana`, `arabic-indic`, `disc`, `circle`, `square`,
`disclosure-open`, `disclosure-closed`) and the document's
`@counter-style` rules, with `extends`, ranges, `pad`, `negative` and
`fallback`. Counters run in document order as the tree is built. A
retained subtree that is not built again replays what it did to them. A
subtree whose generated text reads them is built again only when they enter
it differently, so an idle frame builds nothing. `::marker` takes `content`
(it replaces the list style's text; `none` removes the marker) and `color`.
`::first-letter` styles the first letter, with the punctuation before it.
`::first-line` styles the first line as the last layout broke it, so its
element builds every frame. `::placeholder` takes `color`, and
`::selection` takes `background-color` (the selection highlight), `color`,
`text-decoration` and `text-shadow`. The selected glyphs of a label or a
field are drawn again in the selection's ink, clipped to the selected runs,
so a glyph split by the selection edge shows both inks, as in browsers.
Lines and shadows draw only under the selected runs. Without a
`text-shadow`, selected text keeps the text's own drop. `::file-selector-button` is accepted and styles
nothing, because a document has no `<input type="file">`. The CSS 2
spellings `:before`, `:after`, `:first-line` and `:first-letter` work too.

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

Lengths follow CSS Values 4. The physical units (`cm`, `mm`, `Q`, `in`,
`pt`, `pc`) are px at 96px to the inch. The viewport units (`vw`, `vh`,
`vi`, `vb`, `vmin`, `vmax`) and the dynamic forms (`dvh`…) read the
play surface as it is now (`vi` and `vb` along the document root's inline
and block axes: its height and width when the root is vertical); the small forms (`svh`…) read it with the
browser's own controls (address bar, toolbars) shown, and the large
forms (`lvh`…) with them hidden (`Viewport.smallWidth` and the rest).
The web shell measures both from the page's `100svh` and `100lvh`
whenever the surface resizes, so `dvh` follows the visible viewport as
a phone's bar collapses while `svh` and `lvh` hold; native hosts have
no such controls, so there all three forms are the surface. `rem`, `rlh` and the root font units (`rch`, `rex`,
`rcap`, `ric`) read the body type step. `em`, `lh`, `ch`, `ex`, `cap` and
`ic` read the element's own step (`font: var(--text-*)`, else the body's)
and the body face's measured `0` advance, ascent and water ideograph
advance (`ex` is 0.5em, as CSS's fallback has it). All of these become px
when the element resolves, except a margin's `em`, which its box keeps.
`env(safe-area-inset-top)` and the other safe-area insets (`max` ones
too) read the viewport's safe area, which the web shell measures from
the page's own `env()` and native hosts pass in, with the host's covered
region (`Viewport.covered`: a published page's round Back, top left)
folded into the one edge whose inset clears it for the least surface, as
a phone folds its status bar into the top inset in portrait and its notch
into a side inset in landscape. On a landscape or desktop surface Back
widens the left inset to its right edge; on a portrait one it deepens the
top inset to its bottom. `env(-og-hud-inset-top)` and the other HUD
insets are those insets with the left and right ones grown alike until
the area between them is at most `UiDesc.hudMaxAspect` times its height
wide (21:9 by default; 0 is the whole safe area): the HUD band, centred,
so on a 32:9 ultrawide a HUD's corners sit where a 21:9 screen's do and
the player's eyes stay near the play. A HUD is a document placed at the
HUD insets, so it is clear of Back at every corner and inside the band:
`position: fixed; left: calc(env(-og-hud-inset-left) + 16px); top:
calc(env(-og-hud-inset-top) + 16px)`; full-screen art and dialogs read
the safe-area insets. Any other name takes its fallback. A change to the safe area, the covered region, the HUD band or
the small or large viewport re-resolves the elements that read them.

The engine root already pads normal-flow content by `Viewport.safe_area`.
With a 47px top inset, an in-flow page with `padding: 8px` starts its content
55px below the viewport top. Adding `env(safe-area-inset-top)` starts
content at 102px. Environment values measure from the viewport edge, as
[CSS Environment Variables specifies](https://drafts.csswg.org/css-env-1/#safe-area-insets),
and do not subtract ancestor padding. A full-viewport page with
`position: fixed; inset: 0` can apply the safe-area environment values
once through its own padding. This also clears the host's covered control
through the covered-region calculation above. Plain padding inside normal flow
uses the hardware safe area only, not the covered-control inset.

`attr(data-size px, 4px)` reads the element's attribute (CSS Values 5):
untyped, it is a string; `number`, a unit keyword or `type(…)` make it a
number, and a missing or non-numeric attribute takes the fallback. HTML's
`data-*` attributes are kept for this and for attribute selectors, and a
bound one re-resolves its element when it changes.

A `calc()` over lengths reduces to a linear sum of its `%`, px and em
terms (`calc((100% - 2 * 8px) / 2)` is `calc(50% - 8px)`), and layout
resolves the percentage against the basis it already has for one: the
parent's content width (or height), a grid area, the screen. The other
math functions of CSS Values 4 §10 evaluate as written: `round()` with
its `nearest`, `up`, `down` and `to-zero` strategies, `mod()`, `rem()`,
`abs()`, `sign()`, `sin()`, `cos()`, `tan()`, `asin()`, `acos()`,
`atan()`, `atan2()`, `pow()`, `sqrt()`, `hypot()`, `log()` and `exp()`,
with the constants `e`, `pi`, `infinity` and `NaN`; a NaN result is 0 and
an infinite one the largest value, as CSS censors them. Times (`s`, `ms`)
and angles (`deg`, `grad`, `rad`, `turn`) mix their units freely. When a
function's lengths mix `%` with px, `min()`, `max()`, `clamp()`,
`round()`, `mod()`, `rem()`, `abs()` and `hypot()` are evaluated at
layout against the percentage's basis (`width: min(100%, 600px)`,
`width: round(down, 50%, 40px)`); such a function stands alone as the
value, and the others refuse mixed units. A `var()` term (`calc(var(--w)
* 100% - 20px)`) is substituted first, so a store bound with `style:--w`
moves only its element; an element whose values read a viewport unit
resolves again when the viewport resizes. `calc(10px + 20ms)` is an
error. Container
units (`cqw`, `cqh`, `cqi`, `cqb`, `cqmin`, `cqmax`) are 1 % of the nearest
query container's settled content box on that axis: `cqi` the nearest
size container (`size` or `inline-size`) along its own inline axis, `cqb`
the nearest `size` one along its block axis, each axis as the container's
writing mode names it (its inline axis is its height in a vertical one);
`cqw` and `cqh` the nearest container that contains its width or height.
`cqmin` and `cqmax` take the smaller and larger of `cqi` and `cqb`. With no
such container they are the viewport units. An element using them resolves again when its container's box
settles, as a container query's reader does.

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
`outline-width`/`-style`/`-color`/`-offset`. The logical borders
`border-block`, `border-inline`, `border-{block,inline}-{start,end}` and
their `-width`/`-style`/`-color` (the two-sided ones take start and end)
map to the physical sides in horizontal-tb, left to right. Styles are `none`, `hidden`,
`solid`, `dashed`, `dotted`, `double`, `groove`, `ridge`, `inset` and
`outset` (an outline takes `auto` in place of `hidden`); `currentColor` is
the element's `color`. A border's width pads the box as `box-sizing:
border-box` does; an outline takes no room and is drawn outside the border
edge, grown by its offset, so `:focus-visible { outline: 2px solid
var(--focus) }` replaces the core focus ring for that element. A uniform
border follows `border-radius`, which also replaces a dressed control's
corner (a `<button>`'s plate); sides that differ are drawn as strips, and
`inset`/`outset`/`groove`/`ridge` darken their shaded sides. A uniform
`dashed` border or outline on round corners is dashed side by side, as
browsers draw it: each side's dashes and gaps, about three widths long,
fit it whole and start and end in half a dash, so every corner sits in
the middle of a dash (a corner whose arc is no longer than a dash and
its gap is drawn solid, a longer one dashed as a side is).
`border-radius` takes one to four corners, the elliptical `/` form and
`%` of the box's sides, with the longhands `border-top-left-radius` (and
the other three, `<h> [<v>]`) and the logical `border-start-start-radius`
forms; radii that would overlap scale down together (§5.5). `corner-shape`
(CSS Borders 4) and its per-corner and per-side longhands take `round`,
`squircle`, `bevel`, `scoop`, `notch`, `square` and `superellipse(<K>)`.
Background, border, outline, shadows, `overflow: hidden` and the element's
clip all follow the corners. One uniform px radius stays the element's
`radius`, which states and transitions animate.
`box-shadow` takes a comma list of `[inset] x y [blur [spread]] [color]`,
the first listed on top, outer shadows under the background and inset ones
over it.

Text and fonts. Elements take CSS Fonts 4 and CSS Text 3/4 properties as
standard CSS beside the house form `font: var(--text-<step>)`, which stays
valid: `font-size` (lengths, `%`, `em` of the parent's size, the absolute
and relative keywords), `font-weight` (numbers, `bold`, `bolder` and
`lighter` against the parent's), `font-style` (`italic`, `oblique
<angle>`), `font-stretch`/`font-width`, `line-height` (a number inherits as
the number, a length as px), `letter-spacing`, `word-spacing`, the `font`
shorthand (a generic family picks the style's body, display or mono role),
every `font-variant-*` keyword and the `font-variant` shorthand as their
OpenType features, `font-feature-settings`, `font-variation-settings`,
`font-kerning`, `font-optical-sizing`, `font-synthesis` and its longhands.
`text-transform` takes `uppercase`, `lowercase`, `capitalize`, `full-width`
and `full-size-kana` (cased for the UI's locale); `white-space` and its
longhands `white-space-collapse` and `text-wrap-mode` process the text as
CSS Text 4 §4 says (`pre`, `pre-wrap`, `pre-line`, `break-spaces`, tabs to
`tab-size` stops); `text-wrap` (`text-wrap-style: balance` breaks a
label's lines at the narrowest width that keeps their count, `pretty`
at the widest that keeps it and leaves no one-word last line, `stable`
as `auto`), `text-overflow` (`clip`, `ellipsis`, or a string the cut
ends in), `hyphens: none` (soft hyphens dropped),
`word-break` (`break-all`, `keep-all`), `overflow-wrap`, `text-indent`,
`text-justify`, `text-align` (`left` and `right` are physical, `start` and
`end` follow the direction, `justify` stretches every line but a
paragraph's last at its spaces) and `text-align-last`. `text-decoration`
(and its `-line`, `-style`, `-color`, `-thickness` longhands,
`text-underline-offset` and `text-underline-position`) draws solid,
double, dotted, dashed and wavy underlines, overlines and line-throughs,
propagated to the text inside as CSS Text Decoration 3 says.
`-webkit-text-stroke`, `-webkit-text-fill-color` and `paint-order` outline
and fill the glyphs. The inherited properties pass down the instantiated
tree as custom properties do, so a change re-resolves only the texts that
read it. `accent-color` is the primary ink of the controls under it
(switches, sliders, radios, progress), `caret-color` a field's caret, and
`color-scheme: light | dark` picks `light-dark()`'s side for its subtree.

Colours. Every CSS colour property takes any CSS Color 4 and 5 colour:
hex (`#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`), all 148 named colours,
`transparent`, `currentColor`, `rgb()`, `hsl()`, `hwb()`, `lab()`, `lch()`,
`oklab()`, `oklch()` (space and legacy comma forms, `none` components,
alpha after `/`) and `color()` over `srgb`, `srgb-linear`, `display-p3`,
`a98-rgb`, `prophoto-rgb`, `rec2020`, `xyz-d50` and `xyz-d65`;
`color-mix(in <space> [<hue> hue], …)` interpolating premultiplied, with
the spec's percentage normalisation and hue methods; relative colours
(`oklch(from var(--accent) calc(l * 0.8) c h)`, channel keywords in
`calc()`, `min()`, `max()`, `clamp()`); and `light-dark()`. Conversions
are CSS Color 4's (`crates/og-ui/src/css_color.rs`), and a colour outside
sRGB is gamut mapped by OKLCh chroma reduction, not clipped. A colour that
reads no role, `currentColor` or `light-dark()` reduces to sRGB at cook;
one that does is resolved against the active style when drawn, so a
style switch recolours it. One parse (`style_file::parse_color`) reads
every hex literal, in a document, a paint entry or a style file, so the
four forms hold wherever a colour is read.

Transforms. Elements take CSS Transforms 1 and 2 whole beside the
individual `translate`, `scale` and `rotate`, which
apply first: `transform` with every function (`translate`, `translateX`,
`-Y`, `-Z`, `translate3d`, `scale` and its axis and 3D forms, `rotate`,
`rotateX`, `-Y`, `-Z`, `rotate3d`, `skew`, `skewX`, `skewY`, `matrix`,
`matrix3d`, `perspective()`), lengths in px or % of the reference box and
angles in `deg`, `rad`, `grad` or `turn`; `transform-origin` with
keywords, lengths, percentages and a z; `perspective` and
`perspective-origin` for the children; `transform-style: preserve-3d`;
`backface-visibility`; and `transform-box` (`content-box` and `fill-box`
are the content box, the others the border box). An element whose plane
leaves the screen's (a 3D turn, or its parent's perspective) draws itself
and its subtree into a layer laid through its projective map
(og-render's plane ground), so its edges and text are perspective-correct;
the pointer hits it where it is seen, through the map's inverse. A
`preserve-3d` element's children are planes of one 3D rendering context,
drawn back to front by the depth of their centres (intersecting planes are
not split); a plane showing its back with `backface-visibility: hidden`
draws nothing.

CSS paint. `background` (or `background-image`) takes comma-separated
layers over the background colour, the first on top, as CSS Backgrounds
3 writes them: each `<image> [<position> [/ <size>]] [<repeat>]`, the
colour only in the last. An image is `url(<file>)`, resolved beside the
stylesheet that authored it (inline styles resolve beside the document).
Document loading registers these cooked textures, including SVG files,
without a game-side texture load. URLs never select atlas icons or numeric
asset handles: `url(7)` names the file `7`. Monochrome icons use
`mask-image: url(icons/coin.svg)` with `background-color: currentColor`.
Missing image assets produce a paint warning. Gradients include
`linear-gradient()`, `radial-gradient()` and
`conic-gradient()` and their `repeating-` forms as CSS Images 3 writes
them (angles with 0 up and clockwise, `to <side or corner>`, radial
shapes and extents, `from <angle> at <position>`, stops with one or two
positions, at most 8), mixed in OKLab and resolved against the placed
box, so the line of a `45deg` gradient reaches the corners of a wide
box. `image-set("art/low.png" 1x, url("art/high.png") 2x)` selects the
lowest supported density at least as high as the viewport's DPR, or the
highest available density otherwise. String URLs resolve beside the source
stylesheet, just like `url()`. Descriptors accept `x`, `dppx`, `dpi` and
`dpcm`, defaulting to `1x`; `type("image/png")` can precede or follow the
density. Unsupported MIME types are excluded before first-density-wins
deduplication, and their files are not cook inputs. Gradients are also valid
candidates. Nested sets are rejected. Zero-density candidates paint nothing;
in a mask an unusable set is transparent black, not an absent mask.

Raster natural dimensions divide the source texels by the selected density:
a 16-pixel image at `2x` occupies 8 CSS pixels under `auto` sizing. This also
applies to mask tiles and automatic border-image widths; border-image slices
still address source pixels. Document images and `button::part(base)` reselect
when DPR changes. Saved leaving drawings preserve the density. Resolution
descriptors accept `calc()`, `min()`, `max()` and `clamp()` with mixed resolution
units, such as `calc(2dppx - 96dpi)`. MIME descriptors accept token and quoted
parameters. Malformed MIME strings discard their candidate before density
deduplication; valid parameters do not change how the selected file is decoded.

An ordinary image layer's natural size is its texels, one a CSS px;
`background-size` takes `cover`, `contain` and one or two of `auto`,
lengths and percentages of the box; `background-position` one to four
values (`right 10px bottom 20%` from those edges); `background-repeat`
`repeat-x`, `repeat-y` and one or two of `repeat`, `no-repeat`, `space`
and `round` (with `auto` on the other axis keeping the ratio). The
longhands take a comma list repeated over the layers, and a shorthand
with an image resets them. og-render draws each layer as one image fill
(`UiFit::Background`): its size, placement and repeat are resolved per
pixel from the texture's own size, so a host never tells the UI an
image's size. A gradient has no natural size (CSS Images 3 §5.3): it is
drawn as a tile of its concrete size (`auto` the positioning area's), its
geometry resolved against the tile, then sized, placed and repeated as an
image is (`UiPaint::GradientImage`). `background-origin` (`padding-box`
by default) is the positioning area and `background-clip` (`border-box`,
`padding-box`, `content-box`, `text`) where a layer paints, its corners
following the border radius less the inset; the last layer's clip cuts
the colour. `text` paints the layers only in the element's own glyphs (a
coverage layer and a source-in composite). `background-attachment:
fixed` places a layer in the viewport and `local` in a scroller's
scrolled content; a fixed layer's element paints again when it moves.
`background-blend-mode` blends each layer with those under it inside an
isolated group (the bottom one, over nothing, draws normal). The shorthand
takes the boxes, `text` and the attachment in a layer.

`border-image` and its longhands (`-source` a gradient or `url()`,
`-slice` numbers or percentages and `fill`, `-width` px, %, numbers or
`auto`, `-outset`, `-repeat` `stretch`, `repeat`, `round`, `space`) draw
the image's nine parts over the border box grown by the outset instead of
the border lines (CSS Backgrounds 3 §6): the edges scaled to their widths,
the middle as the top and left edges, widths that overlap scaled down
together (`UiFit::BorderImage`). Masks follow CSS Masking 1: `mask` and
`mask-image` take layers of `url()` and gradients with `mask-size`,
`-position`, `-repeat`, `-origin` (`border-box` by default), `-clip`
(`no-clip` too), `-mode` (`luminance` reads the colour's luminance) and
`-composite` (`add`, `subtract`, `intersect`, `exclude`, each over the
layers under it). `mask-border` and its longhands cut by a border image's
nine parts, `slice 0` and `width auto` by default, with `mask-border-mode`.
The subtree draws offscreen and is kept where the mask is; the layers
combine through og-render's Porter-Duff blends (`UiBlend::SourceIn`,
`SourceOut`, `DestinationIn`, `Xor`). `clip-path`
takes `inset(… [round r])`, `rect(t r b l [round r])` (`auto` an edge),
`xywh(x y w h [round r])`, `circle()`, `ellipse()`, `polygon([rule,] …)`,
`path([rule,] "…")` and `shape([rule,] from x y, …)` (CSS Shapes 2:
`move`, `line`, `hline`, `vline`, `curve … with c1 [/ c2]`, `smooth`,
`arc … of rx [ry] [cw] [large] [rotate a]`, each `to` or `by`, and
`close`), resolved over a geometry box written beside it (`border-box`
by default, `padding-box`, `content-box`, `margin-box`; on a CSS box
`fill-box` is the content box and `stroke-box` and `view-box` the border
box). A geometry box alone clips to that box with the border radius
grown or shrunk to it, and `url(#id)` clips to a `<clipPath>` anywhere in
the document (inline SVG's definitions), its shapes in the reference
box's CSS px or, with `clipPathUnits="objectBoundingBox"`, in fractions
of it. As CSS has it, the clip cuts everything the element draws (its
shadows, background, borders and outline) and its children, by the
shape's analytic coverage (the path clip of inline SVG), with either
fill rule; `overflow: hidden` still cuts only the children. `filter` takes Filter Effects 1's functions in the
order written: `blur()` (a standard deviation, as CSS), `brightness()`,
`contrast()`, `grayscale()`, `hue-rotate()`, `invert()`, `opacity()`,
`saturate()`, `sepia()` and `drop-shadow([colour] x y [blur])`, whose
shadow follows the alpha of what is drawn (a clipped star casts a star),
not the box. Each function is a layer the next one reads, the first
innermost: a blur and a drop shadow are og-render's layer blur and drop
shadow, and a run of colour functions is one colour matrix over
sRGB-encoded straight colour, split where a function can leave 0..1
(`brightness(2)`), so the clamp CSS does after each one stays where CSS
has it. The last layer carries the element's opacity, blend and mask; as
in CSS, `clip-path` cuts after the filter, so a drop shadow on a clipped
element is cut with it (put the filter on its parent). `url(#id)` in the
list is an inline SVG `<filter>` anywhere in the document, a layer like
the functions around it (`filter: url(#glow) grayscale(1)` glows, then
greys the glow): the element is drawn into a layer over the filter
region (by default the border box grown by 10 % a side, `filterUnits`,
`x`, `y`, `width`, `height`), og-render runs the primitives' graph over
it (each `in`/`in2` names `SourceGraphic`, `SourceAlpha` or an earlier
`result`) and the layer composites the last result. The user space is
the border box's CSS px from its top left, and objectBoundingBox units
are fractions of the border box. A `url()` that names no `<filter>`
drops the whole list, as Filter Effects 1 says. `backdrop-filter`
takes the same functions and `url(#id)` over what is really under the
element, in order (Filter Effects 2): og-render's backdrop layer
captures its target as drawn so far under the region where the
element's composite draws (the screen, or the group the element is drawn
in when an ancestor's opacity, filter, mask or clip makes a layer, the
specification's backdrop root), runs the stages as one filter graph over
it in the box's user space, and lays the result in the element's
border-box shape under its own background. The region is the border box
grown by three deviations of each blur and a drop shadow's reach (and
each `url()` filter's region); reads past the capture take its edge, so
a pane at the screen's edge blurs no dark rim in. A colour run is one
matrix while it stays in 0..1, each matrix clamps as CSS does, its
`opacity()` fades the result over the unfiltered backdrop, and a
`drop-shadow()` casts where the backdrop is transparent (inside a
group). The blur is an exact Gaussian, not the glass pyramid; the
theme's `-og-effect: backdrop` glass keeps the pyramid. Inside a world
panel, a `<ground>` or a 3D plane the backdrop root is that layer: its
backdrop is planned on the layer's own pixels (wherever its rect lies,
on or off the screen) and reads what the layer has drawn so far. `mix-blend-mode`
takes every mode of CSS Compositing 1: normal, multiply, screen, overlay,
darken, lighten, color-dodge, color-burn, hard-light, soft-light,
difference, exclusion, hue, saturation, color, luminosity and
plus-lighter. All but multiply, screen and plus-lighter read what is
under the element and blend sRGB-encoded values, as CSS does; multiply
and screen are fixed-function blends in linear light (a few levels darker
than a browser's mid-tones). As CSS Compositing 1 §3.2 has it, a
blended element is a group: it and its subtree draw into an offscreen
layer over a transparent backdrop, and the layer composites through the
blend. `isolation: isolate` makes an element such a group with a normal
composite, and so do `filter`, `mask-image` and `-og-material`: the
blends inside it blend with what is drawn in the group, never with what
is under it. In a layer, a blend reads a copy of the layer drawn so far
(og-render splits the layer's pass there) and follows the spec's
backdrop alpha, so over the group's transparent pixels a colour is
itself; multiply reads it too there, while screen and plus-lighter stay
fixed-function, exact over any alpha. A world panel's layer
(§1) still drops the blends that read what is under them. `text-shadow` takes a comma list of `x y [blur] [color]`.
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
`scrollbar-width: none` hides the bars of a scrolling box; `thin` draws
them at half the width, with a half-width grab strip and gutter.
`scrollbar-color: <thumb> <track>` paints the thumb in the first colour
and a track along the whole edge under it in the second (`auto`: the
style's thumb, no track); it inherits, so a colour set on a panel paints
every scroller inside it. `scrollbar-gutter: stable` keeps a vertical
scroller's bar room at its end whether or not it overflows, so its content
does not shift when it starts to; `stable both-edges` keeps the same room
at its start too.
`scroll-snap-type: x | y | both [mandatory | proximity]` on a scrolling
box and `scroll-snap-align: start | center | end` (block, then inline) on
the areas inside it settle a released drag, a fling or a stopped wheel on
the snap position nearest where it would stop, as CSS Scroll Snap 1 does;
`proximity` snaps only within a third of the view. `scroll-padding*`
(px or % of the view, all its longhands, logical sides and shorthands) on
the scroller insets the view the areas line up in, and `scroll-margin*` on
an area grows the box that lines up; both apply to a scroll into view too.
`scroll-snap-stop: always` on an area stops a fling or a wheel at it
rather than passing it for a farther one: a wheel settles no farther than
the first such area past the offset it started from, however far the
wheel itself carried the box (the start is UI state, saved with it). The fling is steered
onto the target and decays onto it exactly, and the areas are read from
last frame's boxes only while the box settles. `overscroll-behavior` (and `-x`, `-y`, `-inline`, `-block`)
`contain` or `none` on a scroller stops what a wheel, a drag or two
fingers it cannot take at its end from scrolling the scroller around it
on that axis (og-ui draws no overscroll glow or bounce, so the two act
alike). `scroll-behavior: smooth` on a scroller makes a scroll into view
(a focus move, a pad's step, `og.ui.scrollIntoView`) glide there along
the momentum curve and land on it exactly; reduced motion jumps. A glide
is a fling aimed at its target, so it lives in the snapshotted fling.
A modal `<dialog data-detents="40% 90%">` is a bottom sheet: docked to the bottom at
full width, as tall as its tallest detent, opened at its first, dragged
onto its detents and swiped down to close, which writes `open` back.
`data-edge="inline-start"` or `data-edge="inline-end"` docks it to a side instead: full
height, its detents widths, swiped off sideways; the edges are CSS's logical
names, so they mirror right to left. These data attributes control gesture
behavior. CSS controls the sheet's appearance.

A `display: block` box with an auto width fills its block parent's
content width, as in CSS block flow, so a `<div>` inside a `<section>` is
as wide as the section. The document's root element is a block of the
initial containing block: with an auto width it fills the column or
stack it is mounted in (the screen, a panel), never widened by a wider
child's min-content.

Sizes follow CSS Sizing 3. `width` and `height` (and the logical
`inline-size` and `block-size`) take `auto` (the element's own size: a
block fills its parent, a flex item fits its content), a length, a
percentage, a math function, `min-content`, `max-content`,
`fit-content`, `fit-content(<length>)` and the engine's `stretch` (the
room its parent leaves). `min-width`, `max-width`, `min-height`,
`max-height` and their logical forms take the same (`none` for no
maximum), bound any size, and the minimum wins over the maximum, as in
CSS 2 §10.4: `width: 100%; max-width: 640px` and a bound alone
(`max-width: 640px` on a block, which still fills up to it) both work. A
percentage bound reads the size the parent offers and is ignored where
that is not known yet, as CSS ignores one against an indefinite size.
`aspect-ratio` takes `auto`, `<width> / <height>` or one number. Every
element's `box-sizing` is `border-box`, as if the user-agent sheet had
the common `* { box-sizing: border-box }` reset; `box-sizing:
content-box` makes its width, height and bounds the content's, its
padding and border added outside them. `padding` takes its longhands
(`padding-top`, `-right`, `-bottom`, `-left`) and the logical
`padding-block`, `padding-inline` and their `-start`/`-end`; a later
shorthand resets the sides a longhand set.

Flex boxes follow CSS Flexbox 1. `flex` and its longhands `flex-grow`,
`flex-shrink` and `flex-basis` (`auto`, `content`, a size) size an item
by the flex algorithm (§9.7): positive free space is shared by grow
factor, an overflow is taken back by shrink factor times basis, and an
item stops at its bounds, whose automatic minimum is its content's least
size. An item without them keeps its kind: `width: stretch` grows to
level with its siblings and shrinks, a fit-content item shrinks to its
content, and a definite one neither. `flex-direction` takes
`row-reverse` and `column-reverse`, `flex-wrap` takes `wrap-reverse`,
and `flex-flow` is their shorthand; a column wraps into columns at its
height. `justify-content` takes `flex-start`, `flex-end`, `start`, `end`,
`left`, `right`, `center`, `space-between`, `space-around`,
`space-evenly`, `stretch` and `normal`; `align-items` and an item's
`align-self` take `flex-start`, `flex-end`, `start`, `end`, `self-start`,
`self-end`, `center`, `stretch`, `normal`, `baseline`, `first baseline`
and `last baseline`; each takes a `safe` or `unsafe` prefix, and a `safe`
overflow stays at the start. `align-content` places a wrapping box's
lines (`normal` stretches them, as CSS's does), `justify-items` and
`justify-self` place grid items across their areas, and `place-content`,
`place-items` and `place-self` set both axes. `order` reorders flex and
grid items (their reading order stays the document's), and `row-gap`,
`column-gap` and a two-value `gap` space rows and columns apart.

A `display: grid` box lays out by CSS Grid. `grid-template-columns` and
`grid-template-rows` take track sizes (lengths, percentages, `fr`,
`auto`, `min-content`, `max-content`, `fit-content(<length>)` and
`minmax()` of any two), `[line names]`, `repeat(<n>, …)`, and one
`repeat(auto-fill, …)` or `repeat(auto-fit, …)` of fixed sizes, which
repeats as often as the box's width (or height) holds; `auto-fit`
collapses the repeated tracks no item takes. They also take `subgrid`,
which gives the box the tracks its area spans in its parent grid (and
its items size those tracks), and `masonry`, which packs the items
along that axis into the other axis's tracks, each into the track
that is shortest so far. With `masonry`, an auto-placed item keeps the
width of the track it is first given, so a masonry axis whose tracks
differ in size may leave an item at that track's width.
`grid-auto-columns` and `grid-auto-rows` size the implicit tracks
(repeating, and backwards before the explicit grid), and
`grid-template` and `grid` are their shorthands, with
`grid-template-areas` strings, row sizes and the `auto-flow [dense]`
forms. An item is placed by `grid-column-start`, `grid-column-end`,
`grid-row-start`, `grid-row-end`, their shorthands `grid-column`,
`grid-row` and `grid-area`, as a line number (negative counts from the
end), a line name with an optional count, an area name (its
`<area>-start` and `<area>-end` lines), or `span [<n>] [<name>]`. Lines
past the grid add implicit tracks on either side. The tracks size by
the CSS Grid track sizing algorithm: a `1fr` track holds its item's
min-content, and under `normal` alignment its `auto` tracks share the
room left. `justify-content` and `align-content` align the tracks,
`justify-items`, `justify-self`, `align-items` and `align-self` align
an item in its area, and `normal` and `stretch` fill the area.

Kits use `@origingame/engine` and `@origingame/engine/ui-extensions`. Private source-path imports, unsafe
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
`::view-transition { background-color }` is the covers' colour, and
`background-image: url("ui/covers/stars.png")` covers with that texture
stretched over the frame, tinted by the colour (white without one); the
texture loads with the document. Reduced
motion ends finite animations at once and makes covers fade. A component
cannot declare a view transition.

Shared elements follow CSS View Transitions 1 and 2. An element with
`view-transition-name: hero` (or `auto` and `match-element`, which name it
by its identity) is a group of its own. When a layer starts its view
transition, every named element of last frame is captured as its frozen
draw commands: that is its old image. Its new image is the element of the
same name drawn this frame, outside a leaving layer. In the top layer
(over covers too) `::view-transition-group(hero)` moves from the old box to
the new one (the UA's `-ua-view-transition-group-anim`, 0.25s, fill both).
`::view-transition-image-pair(hero)` holds both images, which fade out and
in (`-ua-view-transition-fade-out` and `-fade-in`) and add with
`plus-lighter` when both exist. A name only in the old view fades out
where it was, and a name only in the new view fades in. The live elements
taking part are hidden until the transition ends. The new screen's rules
style the groups (the leaving screen's when nothing joins).
`::view-transition-group`, `-image-pair`, `-old` and `-new` take `(name)`,
`(*)`, `(.class)` and `(name.class)`, where the classes are the element's
`view-transition-class`. They cascade as the spec has it: `*`, then a name,
then each class and an `:active-view-transition-type()` qualifier, then
source order. The image pair and the images inherit the group's timing.
Each takes `animation` with the document's `@keyframes` (transform and
opacity channels). A name on two elements of the old or the new view skips
the transition with a warning (the spec's `InvalidStateError`), and the
layers change at once. Each image is as wide as the group, with its height
from its own aspect, from the group's top. As in the spec's UA stylesheet
and in browsers, nothing clips it there: an image taller than its group
overflows it. `overflow` (`clip`, `hidden`, `scroll` or `auto`; also
`overflow-x`, `overflow-y` and the logical longhands) on
`::view-transition-group()` or `::view-transition-image-pair()` clips what
that pseudo-element holds to its box, along the axes it names, and turns
with its transform. The pair's `overflow` is its own, not inherited from
the group, and `-old` and `-new` refuse it. A shared element's images are
its own drawing: the shared elements inside it are left out, since each is
drawn by its own group. Groups nest as in Level 2: `view-transition-group`
on a shared element is `normal` (the default: the group goes in the nearest
shared ancestor's group whose value is not normal, else at the top),
`contain` (the same, and its descendants nest in its group), `nearest` (the
nearest shared ancestor's group) or an ancestor's name (that ancestor's
group, else as normal). The old and new views resolve this separately, and
the new view's parent wins (the spec leaves a mismatch open). A nested
group is drawn in its parent's `::view-transition-group-children()`, over
the parent's image pair. It moves from where it was in the old parent's
padding box to where it is in the new one, so it rides along with its
parent, and the parent's transform, opacity and group `overflow` apply to
it. The image pair is then a layer of its own, so the pair's opacity does
not reach the nested groups. `::view-transition-group-children()` sits over
the parent's padding box. It takes `overflow`, which clips the nested
groups there, and `animation` (only its own, because nothing inherits into
it). Reduced motion skips the shared elements unless the
style keeps motion. Capturing costs nothing until a transition starts: a
frame records only where its named elements drew. A running transition is
saved with the UI state.

Inline SVG draws data-bound vector graphics in a document: `<svg viewBox>`
with `<g>`, `<path>`, `<rect>`, `<circle>`, `<ellipse>`, `<line>`,
`<polyline>`, `<polygon>`, `<text>`, `<foreignObject>`, and `<defs>`,
`<marker>`, `<linearGradient>`, `<radialGradient>` and `<stop>`, animated
by SMIL. Every attribute can bind
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
dominant-baseline fill>` is a line shaped by og-ui's text, as any label
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

Several lines are `<tspan>`s that move: each one with `x` (and `y`, or
`dy`, which accumulates from the last `y`; `dx` and `dy` take user units
or `em`) starts a line placed and anchored on its own point, as browsers
draw them. `white-space` (or `xml:space="preserve"`) on the `<text>` keeps
its spaces (`pre`, `pre-wrap`, `break-spaces`) and its line breaks
(`pre`, `pre-wrap`, `pre-line`, `break-spaces`).

`<text><textPath href="#id" startOffset side method spacing>…</textPath></text>`
runs the text along a path or any shape (`href`), or along its own `path`
data, with og-ui's text on a path (`-og-text-path`, below): `startOffset`
in user units (of the shape's `pathLength`) or a percentage, `text-anchor`
on the `<text>` as the anchor at that offset, `side="right"` backwards.
`method` and `spacing` take SVG's values; glyphs are turned rigidly and
keep the font's advances. Presentation attributes go on the `<text>`.

`marker-start`, `marker-mid` and `marker-end="url(#id)"` draw a
`<marker viewBox refX refY markerWidth markerHeight markerUnits orient>`'s
shapes at a shape's first, middle and last vertices: `orient` `auto`
(along the path, the bisector at a corner), `auto-start-reverse` (the
start turned back, for two-headed arrows) or an angle; `markerUnits`
`strokeWidth` (the default, scaled by the stroke width) or
`userSpaceOnUse`; `context-fill` and `context-stroke` take the shape's
paints. `stroke-dasharray` takes any list (an odd one repeats, as SVG
says); one dash and one gap draws natively, a longer list cuts the outline
into its dashes.

`<foreignObject x y width height>` lays document elements out in a box of
`width` by `height` user units at `(x, y)`, clipped to it and scaled with
the view box as the shapes are; its elements take input and are in the
accessibility tree as any others.

SMIL animation plays inside the element it animates: `<animate
attributeName>` (any attribute or presentation property, `opacity`
included), `<set to>`, `<animateTransform type>` (translate, scale,
rotate, skewX, skewY) and `<animateMotion path rotate>` (or an
`<mpath href="#id">` naming any shape; `rotate` auto, auto-reverse or an
angle), with `begin` (clock values, lists, `id.begin` and `id.end` with
offsets), `dur`, `repeatCount`, `repeatDur`, `fill` (freeze or remove),
`values`, `from`, `to`, `by`, `keyTimes`, `keySplines`, `calcMode`
(discrete, linear, paced, spline), `additive` and `accumulate`. Numbers
interpolate within their text (`d` with the same commands, `points`,
lengths) and colours by channel; anything else is discrete. Each
animation is presentation only: its clock is the UI's tick-counted clock,
starting when its element first shows, kept in the element's widget
memory (snapshotted beside the world, outside `world_hash`); an animated
element resolves again each frame while an animation plays or waits to
begin, and nothing else does. Under reduced motion each shows its frozen
end, or nothing. An event `begin` (`click`) is refused: start motion from
input with `og.tween` or a CSS state. Animation attributes are literal; a
`<text>` animates from a `<g>` around it.

`<image href x y width height preserveAspectRatio>` draws an atlas image
icon by name (`href="gem"`) or a render texture by handle
(`href={$target}`), filling its rect (contained by default, `slice`
covers, `none` stretches); it takes no turn or skew.

`filter="url(#id)"` on a `<g>` or a shape draws it (and a group's
children) through a `<filter>` (Filter Effects 1 §9-15), as CSS
`filter: url(#id)` does a box (above): in the element's user space (its
`transform` and the view box's fit to CSS px), its bounding box its
geometry's (a group's literal shapes'), so a zero-height line under
objectBoundingBox units draws nothing. Every primitive is drawn:
`feGaussianBlur`, `feOffset`, `feFlood`, `feComposite` (all operators),
`feMerge`, `feBlend` (every `mix-blend-mode`), `feColorMatrix`,
`feComponentTransfer` (tables keep 16 values), `feMorphology`,
`feDisplacementMap`, `feTurbulence` (the reference implementation's
noise, stitched or not), `feDropShadow` (its blur, offset, flood,
composite and merge), `feImage` (an atlas image or texture handle by
`href`, stretched over its subregion, or an element by `href="#id"`:
built as `<use>` builds it, under the filtered element and inheriting
from it, so it draws whatever the element draws (gradients, patterns,
text, images, groups, nested `<use>`), in the filtered element's user
space, moved by the `feImage`'s `x` and `y` when it gives them; it takes
no input, its anonymous box draws it into a layer that composites
nowhere (`ElementDraw::element_image`) and the filter reads that layer;
an element the filter is on or inside is refused with a warning, and
element images nest four deep at most), `feTile` (filling
the region), `feConvolveMatrix` (up to 64 cells) and
`feDiffuseLighting`/`feSpecularLighting` with a distant, point or spot
light; `primitiveUnits`, subregions (`x`, `y`, `width`, `height`, by
default the union of the inputs' or the region),
`color-interpolation-filters` (`linearRGB` by default, `sRGB`) and
`kernelUnitLength` (a convolution's cell, and the cell a light's surface
normal is taken over, whose slope is then per cell; left out, one device
px) are read. `BackgroundImage`, `BackgroundAlpha`, `FillPaint` and
`StrokePaint` read as transparent black. Each primitive is one or two
full-region passes (a blur reads every device px within three
deviations, at most 128 a side, and a wider one is split into passes of
σ/√n, which compose to the exact Gaussian; a morphology takes every
texel its segment along the axis crosses), run in the filter's own space (the element's user
space; og-render's `UiFilter::space` maps it to the screen): a blur or
a morphology under a turn or skew runs along the element's axes, a
subregion is its own parallelogram on the screen, and lighting computes
its surface and lights in that space, so a turned element is filtered
as it would be unturned and then turned.

Definition attributes (gradients, clips, markers, masks, patterns,
filters) are literal. Cost: re-tessellating one
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
pointer and a drop's target, so they cost nothing as the tree grows.
`onpinch` fires while two touches spread or close, at the element they went
down on, with `scale` (their spread over their spread when they went down,
as Safari's `GestureEvent.scale`) and `x`, `y` their centre in its box; no
web standard has a pinch event. An
element with a pointer handler can be under the pointer, so like any
hovered control it is written again each tick while hovered.

`<viewport>` shows a camera's render target in the layout, sized by CSS like
an image (og-ui's `Ui::scene_viewport`). `src={$target}`
names the target's colour texture. The camera is written as
`<model-viewer>` writes one: `camera-orbit="30deg 75deg 3m"` (yaw, pitch
from straight up, distance), `camera-target="0m 1m 0m"` and
`field-of-view="50deg"`, each bindable. Svelte's `bind:clientWidth` and
`bind:clientHeight` write its laid-out size when it changes, so the game
sizes the target by them times the pixel ratio. Its pointer events
(`onpointerdown`, `onpointermove`, `onwheel`) also carry `ray`, `{ origin,
direction }` into its scene through the pointer, for picking and orbiting.
`camera-controls` makes it orbit itself, as `<model-viewer>`'s does: a drag
turns and tilts the camera and a fling keeps it turning, slowing to rest
(still under reduced motion); the wheel and a two-finger pinch zoom it; with
the focus, Left and Right turn it 15° and Up and Down tilt it 10°. It stays
within `min-camera-orbit` and `max-camera-orbit` (`auto 22.5deg auto` and
`auto 157.5deg auto` by default, `auto` unbounded), and
`bind:cameraOrbit={$orbit}` reads the orbit back for the game to draw the
scene from. The orbit is UI state; a new authored orbit takes over.
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

`<anchor>` holds a box over a world point. It is the native world anchor,
its children laid out as a column:
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

`<panel world>` is the native world panel: a box
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

Particle emission belongs to `<og-emitter effect="sparkles">`.
`-og-burst: 24 48px #ffd166 600ms` inside a
`@keyframes` step fires rays and sparks from the element when an iteration
reaches that offset, flying for the time. It sits beside `-og-sound`, as a
keyframe event. The emitter's `effect` emits particles while the element stands,
and `none` stops it: a preset (`sparkles`, `confetti`, `embers`, `snow`,
`bubbles`, `dust`, `sparks`, `fireworks`) in the style's colours, or a rate
a second, and over either `burst <n>` (that many at once when the element
first stands), `for <time>` (the rate stops after it), `life <time>`, a
speed (`120px/s`, or a range of two), `up`, `down` or an angle, a shape,
a size and colours (`effect="confetti 70 burst 110 for 1.4s"`,
`effect="12 diamond 6px #ffd166 #fff"`). `-og-meta:
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
height, under the screen's elements and the world's mass elements: a
selection ring, a range marker, a target decal. It is laid as a decal
against the frame's depth prepass (og-render's `UiGround`): each screen
point under it reads the scene's view depth, rebuilds the surface there,
and takes the layer where that surface falls on the box along the
plane's normal, so the box drapes over terrain rising and falling within
a quarter of its shorter side of the plane. A surface standing across
the plane (a wall, a unit's legs) fades it out, something standing
nearer the camera than that reach hides it, and where the scene lies
farther behind the plane or there is none (no geometry, the sky) it is
laid on the plane itself, as a sign in the air is. The map from a screen
point and its depth to the box is affine in the point for a perspective
or orthographic camera (`UiDrape`, three rows of two 3-vectors), so the
camera moving moves only it and the box's screen bounds
(`UiFrame.grounds`), read after the tick and never fed back into the
world: the frame's commands, bindings and layout stay as they were (0
bindings, 0 reconciled, the layer root laid out as every layer is).
Its offscreen layer is sized from the size its plane is seen at, rounded
to a power of two and capped at 2048 px; with a corner behind the camera
(the camera among it) it is drawn at the cap over the whole screen. It
is inert: no ray reaches it and its controls take no focus. `ground` is
not an HTML element; no web standard lays a box onto a 3D plane.

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

A text input a pad gives the focus opens og-ui's on-screen keyboard, a
panel docked at the screen's bottom over a scrim: the text with its caret,
rows of keys the pad walks (the focus starts on the first; Y erases, LB
and RB move the caret, Back closes) and a finger presses, letters with
shift (a quick second press locks capitals), symbols and a number pad. Its
keys edit the bound value as the player types; Done submits, as Enter
does, and Done or Back return the focus to the input, which opens it again
only after it has lost the focus. `type` and `inputmode` pick its layout
(`numeric`, `decimal` and `tel` the number pad, a PIN's digits only for a
`password`), `enterkeyhint` labels Done (`Search`, `Send`, `Go`…),
`maxlength` counts down in it, a password shows as dots, and `lang="ja"`
offers the kana table, whose typed kana compose underlined with the voicing
marks and small kana until Enter or a candidate (the hiragana or its
katakana) commits them. `inputmode="none"` (the game's own keyboard) opens
none. Parts: `Keyboard.panel`, `Keyboard.preview`,
`Keyboard.key[wide][action][active][primary]`, `Keyboard.candidates`,
`Keyboard.candidate`.

The host receives the type, the effective `inputmode` (unset: the type's
own), `autocomplete`, and `secret` with the focused text target. The web
shell stands a hidden `<input>` with those attributes behind a one-line
field so phones show the requested keyboard and autofill reaches it; a
text area keeps its `<textarea>`. A number uses a text input with the
decimal keyboard.

#### Selectable text

CSS `user-select` makes read-only
text selectable: `text` selects by characters, `all` takes the element's
whole text with one press, `none` turns it off, and `auto` takes the
parent's. Nothing is selectable by default, and a control's subtree (a
button's label) stays `none` unless it sets the property itself. Only plain
text elements and vertical rich text take a selection; horizontal rich
text and text controls keep their own.

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

#### Pointer, touch and visibility

CSS `cursor` takes every CSS UI 4 keyword (`context-menu`, `cell`,
`vertical-text`, `alias`, `copy`, `no-drop`, `all-scroll`, `col-resize`,
`row-resize`, the eight edge and corner resizes beside the earlier ones),
inherited by the subtree, and `og.ui.frameCursor` answers it. `url(<file>)
[<x> <y>]` images listed before the keyword are tried in
order and the first that loads is shown at its hotspot (default its top
left, clamped into the image), the keyword where none does: the property
inherits whole, so images over `auto` stop the parent's keyword and fall
back to the element's own kind's. Each URL resolves beside its stylesheet
and reads a registered texture asset, including cooked SVG: an RGBA
texture, or a cooked one read again from the bundle (any natively; on the
web a lossless one, `kind pixel`, since the Worker's Basis textures stay on
the GPU). An image larger than 128×128 (the browsers' limit)
or one that cannot be read falls through, with a note. The frame answer
carries the keyword as `cursor` and, on the frames where it changed,
`cursorImage` (`{width, height, rgba: <base64>, hotspot}` or null); the
web shell sets the canvas's CSS cursor to the image as a PNG with the
keyword after it. Origin Desktop's shared frame hears it as the `Cursor`
message (docs/design.md §2.6) and sets the OS cursor. Decoded images are
kept by their source, eight at most.

`pointer-events` inherits: `none` lets the pointer through to what lies
under the element (keys and pads still reach it), and a descendant saying
`auto` takes it again. On inline SVG shapes the SVG values pick what is
hit: `visiblePainted` (the shape's fill and stroke where they are
painted), `visibleFill`, `visibleStroke`, `visible`, `painted`, `fill`,
`stroke`, `all` and `bounding-box`, tested against the shape's geometry
by its fill rule and half its stroke's width. On other elements they act
as `auto`.

`touch-action` (Pointer Events 3) limits what a touch drag starting on an
element does on a coarse pointer: `pan-x`, `pan-left`, `pan-right`,
`pan-y`, `pan-up`, `pan-down` and `pinch-zoom` combine, `manipulation`
is all three, `none` keeps the touch for the element's own handlers. The
used value is its own intersected with its ancestors'; a pan it does not
allow does not scroll, and a zoom view without `pinch-zoom` is not
pinched.

`visibility` inherits: `hidden` keeps the element's place but draws
nothing of its own, takes no pointer and leaves the accessibility tree,
while a child saying `visible` is drawn and taken as usual. `collapse` is
`hidden` that also gives up a flex item's main size. The `visible`
attribute stays: `visible false` removes the element from layout as
`display: none` does. These are read from the frame's orders as
`user-select` is, so a declared value costs nothing while idle.

#### Containment, resize and will-change

CSS `contain` (CSS Containment 2 §3) takes `none`, `strict`, `content`
and any of `size` or `inline-size`, `layout`, `style` and `paint`. Size
containment sizes the box as if it were empty, or as
`contain-intrinsic-size` (and `-width`, `-height`, `-inline-size`,
`-block-size`: `none`, a px length, or `auto` before either) says; its
contents still lay out inside it. Layout and paint containment make it a
formatting context of its own (no margin collapses through it), and paint
containment clips what it and its subtree draw to its box. Style
containment keeps counters and quotes inside it: an increment or set there
makes a counter of its own, and nothing it does is seen after it.

`content-visibility` (§4): `hidden` skips the element's contents always,
`auto` from the frame after it was found far from the screen (more than
half a viewport past what its clip shows) with no focus inside it. Skipped
contents are not laid out, drawn or hit; `hidden` ones also leave the
accessibility tree, `auto` ones stay in it. Both add layout, style and
paint containment, and while skipping, size containment, sized by
`contain-intrinsic-size`; with `auto` there it takes the content size it
had when it last laid its contents out. The layout walk leaves a skipped
subtree out, so it costs nothing while idle; an element that starts laying
its contents out again lays them out afresh.

`resize` (CSS UI 4 §5.2) on an element whose overflow is not `visible`:
`both`, `horizontal` (`inline`) or `vertical` (`block`) puts a grab handle
in its bottom end corner, three rows of dots in the style's strong control
ink with the matching resize cursor over it. A press there holds the
pointer, and its drag sets the element's width or height in CSS px under
its `min-` and `max-` bounds, as a browser writes the element's size. The
size stays on the element (presentation state, not saved) while it says
`resize`; only the element being dragged is written again.

`will-change` accepts `auto` or a list of properties; one that makes a
stacking context (`transform`, `opacity`, `filter`, `mix-blend-mode`,
`clip-path`, `mask`…) draws the element as a group on a layer of its own,
as `isolation: isolate` does, so its change composites one texture.

#### Images

CSS `object-fit` on an `<img>` takes `fill` (the default: stretched to
its box), `contain`, `cover`, `none` (at the texture region's own pixels)
and `scale-down` (`contain`, never above its own size); on `<art>` it
also keeps `nineSlice` and `tile`. `object-position` places what does not
fill the box as `background-position` is written, one to four values with
edge offsets (`right 10px bottom 25%`); its default is the centre, and on
`<art>` it replaces the spec's focal point. The renderer reads the region's
own size from the texture, so an image needs no size of its own.
`image-rendering` inherits: `pixelated` and `crisp-edges` sample every
image the element draws (its `<img>` or `<art>`, its background, border
and mask images) at the nearest texel of the top level, so pixel art
scaled up keeps square, sharp pixels; `auto`, `smooth` and `high-quality`
keep the texture's own sampling (a `kind pixel` texture is nearest
already). The legacy `optimizeSpeed` and `optimizeQuality` are `pixelated`
and `smooth`. All three are read as the element is built.

#### Text on a path

`-og-text-path` sets an element's
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
under it. `dialog::backdrop` is the scrim's box, the same synthetic node
as a `::before`'s: its `background` fills it (default the style's
`scrim`), and `opacity`, `backdrop-filter`, `filter`, padding (which moves
where the panel centres), borders, radius, `box-shadow`, transitions and
the rest of a box's properties draw it. It cascades with states,
variables and media as the dialog's own rules do.
To a style pack the scrim is the part `Dialog.backdrop`. Without `modal` it is `show()`: a panel in place
(`Dialog.panel[open]`); the focus returns to what held it when it opened.
Closing writes `open` false back. `modal` is written as a literal, never
bound: a modal's content is built in the top layer and named under
`<id>/Panel`, so a dialog that is sometimes modal is two dialogs or an
`{#if}`.

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
document's. `select::picker(select)` (CSS's customizable-select
pseudo-element) is the open list's box: background, padding, borders,
radius, shadows, `backdrop-filter`, opacity and the rest of a box's
properties style it over the `Dropdown.list` part, and `max-height: 240px`
bounds its rows, which then scroll.

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
screen: `player="k"` on an element (Rust `Props::player`) makes its subtree player `k`'s view, from 0 to 3. Each region
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
`:not()`, `:is()`, `:where()` and `:has()`. `:where()` has zero specificity;
`:not()`, `:is()` and `:has()` take the highest specificity of their
arguments. Attribute selectors take every Selectors 4 operator (`=`, `~=`,
`|=`, `^=`, `$=`, `*=`) and the `i` and `s` flags; HTML's case-insensitive
attributes (`type`, `lang`, `dir`…) compare case-insensitively unless `s`
is given. `lang`, `dir`, `data-*` and `state:name={…}` are selector-only
attributes.

Structural pseudo-classes are `:first-child`, `:last-child`, `:only-child`,
`:nth-child(an+b [of S])`, `:nth-last-child(an+b [of S])`, `:first-of-type`,
`:last-of-type`, `:only-of-type`, `:nth-of-type(an+b)`,
`:nth-last-of-type(an+b)`, `:empty` and `:has()`. `of S` takes compound
selectors and adds their highest specificity. Siblings
and positions are those of the instantiated element tree, after `{#if}`,
keyed `{#each}`, snippets and components; structural blocks are not
siblings. `:empty` means no instantiated element children and no text.
Retained mounts match on the logical sibling order, never render slots.
An edit invalidates only the sibling interval it can affect, in O(log n)
per interval. Virtualized rows outside the materialized range restyle when
they materialize, so a front insert restyles only visible and overscan rows.
`:has(<relative selectors>)` takes a leading `>`, `+` or `~` and does not
nest. A change under an element restyles only its ancestors whose own facts
match a `:has()` compound, the earlier siblings a `:has(+ …)` or `:has(~ …)`
reaches, and, for a `:has()` before the subject (`.a:has(.b) > .c`), the
descendants of such an anchor whose own facts can be the subject.
Structural selectors cannot select `::part` spans.

The state selectors are `:hover`, `:active`, `:focus`, `:focus-visible`,
`:focus-within`, `:disabled`, `:enabled`, `:checked`, `:selected`,
`:indeterminate`, `:default`, `:invalid`, `:valid`, `:user-invalid`,
`:required`, `:optional`, `:read-only`, `:read-write`, `:in-range`,
`:out-of-range`, `:placeholder-shown`, `:open` (an open `<details>` or
`<dialog>`), `:modal`, `:popover-open` and `:target` (the element whose id
the last `#fragment` link named). `:enabled`, `:valid` and `:optional`
match form controls; `:read-write` matches editable text fields. An
ancestor selector reads that ancestor's own interaction identity, not the
styled child's state. `:lang()` (RFC 4647 ranges) and `:dir(ltr|rtl)` read
the nearest `lang` or `dir`, else the locale; `:state(name)` reads the
element's `state:name`; `:defined` matches every element, `:scope` the root
and `:any-link` an `<a href>`.

Rules nest (CSS Nesting): a nested rule's `&` is its parent's selector
(written in place for one compound, else `:is(parent)`), and one without
`&` is relative to it; `@media`, `@container`, `@supports`, `@layer`,
`@scope` and `@starting-style` nest inside a rule and apply its selector.
`@layer` orders cascade layers (statements, blocks, anonymous and nested
layers, a layer's sublayers before its own rules): unlayered rules beat
every layer, a later layer beats an earlier one, and `!important` reverses
that order; the user-agent rules are below every layer. `@supports` holds
when the cascade takes the declaration as written (`(display: grid)`) or
the selector (`selector(:has(> a))`). `@scope (root) to (limit)` matches
in the root's subtree above the limit; the root adds no specificity.
`@import "x.css" [layer(name)] [supports(…)] [media];` comes before the
rules; the cook reads the file from beside the document (and its own
imports) into the document blob. `@starting-style` is the before-change
style of an element's first style (CSS Transitions 2 §3): each property it
sets that a transition covers, any animatable property, moves from it as
from a style change; with `transition-behavior: allow-discrete`, a
`display: none` or `visibility: hidden` start holds the shown value
throughout. `@property` registers a custom property's
syntax, inheritance and initial value; a value that does not match its
syntax takes the initial value. `@font-face` declares a named family
(CSS Fonts 4): the cook bundles each `url()` source (read relative to the
document; it must be a font of the project, cooked with its own settings,
and `local()` names no font since the engine has no system fonts), and the
runtime registers the first source the bundle has under the family name
when the document loads. `font-family: "Name", monospace` (or the `font`
shorthand) tries named and generic families in authored order for each glyph,
skipping unavailable names. Generics may appear anywhere, including before
named families: `serif, "Symbols"` tries Symbols for glyphs the body face lacks.
Generics map to the style's roles (`monospace`, `ui-rounded`,
`fantasy`/`cursive`, anything else body); quoted `"serif"` is a named family,
not a generic. Of each named family's faces
the text takes those of the nearest `font-style` and then the nearest
`font-weight` range (§5.2's order), and each character comes from the
last-declared face of that set whose `unicode-range` has it, then the next
entry in the list. After the list, fallback tries the role's face, the CJK
companion, the locale's and the game's fonts. A `::part` run's
`font-family` preserves the same ordered list. A weight or slant the family
has no face for is synthesised as for a role. `@counter-style` and
`@position-try` are read into the document's sheet (counters and anchor
positioning use them). `inherit`, `initial`, `unset`, `revert` and
`revert-layer` work on every property: `inherit` takes the parent's
winning value, `revert` rolls back to the user-agent rules and
`revert-layer` to the layers below the declaration's.

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
`height`, `aspect-ratio`, `orientation`, `resolution` (`dppx`, `x`,
`dpi`, `dpcm`: the viewport's device pixel ratio), `prefers-reduced-motion`,
`prefers-color-scheme` (the theme's scheme), `prefers-contrast` (`more`
when the theme is high contrast; `less` and `custom` never match),
`prefers-reduced-transparency` (the theme's flag), `pointer`/`hover` and
their `any-` forms, and `-og-color-vision` (below). Each element
re-resolves only when an input it read changes. The play surface answers
the display features as fixed values: `color-gamut: srgb`,
`dynamic-range: standard`, `display-mode: fullscreen`, `update: fast`,
`scripting: enabled`, `forced-colors: none`, `inverted-colors: none`,
`prefers-reduced-data: no-preference`, `color: 8`, and 0 for `monochrome`,
`color-index` and `grid`. The range features take inclusive
`min-`/`max-` bounds and range comparisons. Commas and `or` select
alternatives, `and` intersects, and `not` negates; mixed operators require
parentheses. Nested media rules intersect. There is no named-breakpoint
reader.

Container queries follow CSS Containment Level 3.
`container-type: normal | [ size | inline-size ] || scroll-state || anchored`
(with `container-name` and the `container: <names> / <type>` shorthand)
makes an element a query container for its descendants. A named element is
a style container whatever its type. `@container [name] <condition>` takes the media range
grammar over `width`, `height`, `inline-size`, `block-size`, `aspect-ratio`
and `orientation`; `inline-size` and `block-size` read the container's
axes as its writing mode names them, and a condition on a container's
block axis (a vertical container's `width`) needs a `size` container. Each
condition reads the nearest eligible ancestor; with none, or before the first
measurement, it is unknown and false. Conditions settle through the
settlement hook on the first frame and on every resize, before paint:
elements that read a changed container resolve again from their evaluated
inputs and replace their fields, including `display`, list and grid boxes and
Rive frames. There is no size containment, so a query that keeps resizing its
own container fails as a settlement cycle.

`style(--name: value)` and `style(--name)` (with `and`, `or` and `not`
inside or around them) compare a container's computed custom property,
written with its whitespace collapsed. An unnamed style query reads the
parent element, because every element is a style container. A named one
reads the named container's own value. A change re-resolves only the elements
that read it. Style queries take custom properties only; `style(color: red)`
fails at cook. `scroll-state(scrollable: top | right | bottom | left | x | y
| block… | inline… | none)`, `scroll-state(snapped: x | y | block | inline
| both | none)`, `scroll-state(stuck: …)` and `scroll-state(scrolled: …)`
read the nearest `scroll-state` container, as the last frame laid it out.
`scrollable` gives the edges it can still scroll toward. `snapped` gives the
axes on which the container, as a snap area, sits at its scroller's snap
position. `scrolled` gives the direction of its latest scroll on each axis.
`stuck` gives the edges of its scrollport a `position: sticky` container is
held to (its offset from its flow place, on the last frame). A change restyles
only its readers, on the next frame. `anchored` is accepted for anchor
positioning's fallback queries. Other features fail at cook with the feature
named.

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
stops with per-keyframe timing functions. Timing functions include CSS
Easing 2's `linear(0, 0.8 25%, 1)` (stops with zero to two input
percentages, canonicalised as the spec says); a saved state names a
`linear()` curve by its points' hash, so its document defines it again
before a restore reads it. The shorthands expand into
longhands at cook, so selectors, state and media choose an element's
animations; an animation starts when its rule begins to match and ends when
it stops. Motion runs on presentation ticks, never a clock, and is saved with
the presentation state outside `world_hash`, so a restore continues exactly.
Reduced motion shows a finite animation's end and stops endless ones;
transitions use the element's reduced-motion fallback. Transitions and
keyframes cover every animatable property. Opacity, translate, scale,
rotate, `border-radius`, background, color and z-index move the
element's channels in the motion step (springs and motion tokens keep their
velocity when retargeted); every other property moves its computed value
before the cascade routes it, so it animates through the same parsing as a
declaration. Values interpolate by type (CSS Values 4 §3): numbers,
lengths and percentages (mixed units as `calc()`), integers rounded,
angles and times in any unit, colours as a premultiplied `color-mix()` (in
sRGB when both are legacy colours, Oklab otherwise), functions and
gradients component by component, shadow and filter lists padded with
their identities, and transform lists function by function, through
decomposed matrices (quaternions) where their primitives differ. Any other
pair is discrete and flips halfway; `display`, `visibility`,
`content-visibility` and `overlay` keep their shown value for the whole of
a move to or from their hidden one. A transition starts from the value in
flight, and one reversed towards where it began is shortened as CSS
Transitions 1 §3 says. A discrete property transitions only with
`transition-behavior: allow-discrete` (also an `allow-discrete` word in the
`transition` shorthand). `animation-composition: replace | add |
accumulate`, on an element or inside a keyframe, combines a keyframe with
the underlying value (Web Animations 1 §5.4.4: lengths sum, scales multiply
when added and accumulate one-based, transform lists concatenate when
added, shadows and filters append when added and sum item by item when
accumulated). A keyframe value may use `var()` and the element's units; a
missing 0% or 100% keyframe takes the underlying value. A transition to or
from a property that is not declared uses its initial value where CSS fixes
one, otherwise it jumps. An element with a computed value in flight builds
again each frame; once settled it costs nothing. `var()` inside a
shorthand, a property that does not animate, and unknown keyframes fail at
cook with the property named.
Scroll-driven animations follow CSS Scroll-driven Animations 1.
`animation-timeline` takes `auto` (presentation ticks), `none`,
`scroll(<nearest | root | self> <block | inline | x | y>)` (the scroll
range of that scroller), `view(<axis> <inset>{1,2})` (the element's own
cover range in its nearest scroller) or a `--name` that `scroll-timeline`
(`-name`, `-axis`) or `view-timeline` (`-name`, `-axis`, `-inset`)
declares on the element or an ancestor; `timeline-scope: --name | all` on
an ancestor lets its descendants see a timeline declared anywhere in its
subtree (two such declarations make it inactive). `animation-range`
(`-start`, `-end`) narrows the active interval: `normal`, a
length-percentage of the timeline, or `cover`, `contain`, `entry`, `exit`,
`entry-crossing` or `exit-crossing` with an optional length-percentage. The
`animation` shorthand resets the timeline to `auto` and the range to
`normal`, so `animation-timeline` comes after it. On a progress timeline
the iterations share the range, a time duration and a positive delay are
proportions of it, the play state is ignored and keyframe events do not
fire. Progress is read from the laid-out geometry each motion step (the
subject's layout box, its transforms left out, so an animation never moves
its own timeline): an element is sampled again only when its progress
moved, and a computed value (`width`, a shadow) one frame after the move.
An inactive timeline (a scroller that cannot scroll, no subject) leaves
the animation without effect. Reduced motion leaves scroll-driven
animations alone: the player drives them.
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

A document names a part with the `part` attribute (`<div part="Killfeed.row"
class:self={row.self}>`). Its classes, `aria-selected` (`:selected`) and
`aria-pressed` or `aria-checked` (`:checked`) qualify it, so
`Killfeed::row.self` and `Loadout::item:selected` dress it. A control keeps
its own variant and states: `<button part="Loadout.item">` is
`Loadout::item.secondary:hover` under the pointer, and its label's ink reads
against that dressing.

Core parts keep their existing names and meanings. Each kit documents its
additional parts with its own API, so packs can dress them without private
imports. A missing custom dressing uses the readable core token fallback.
It is not permission to leave a required state unfinished.

A kit names every frame it draws as a part (`<column part="Inventory.frame">`)
and gives it only a token background as the fallback. A kit's own background,
border image or border on the element outranks the pack's dressing of
that part, so a kit that paints its own frame leaves the pack nothing to
dress. Rarity, selection and other states a pack dresses reach the part as
qualifiers or classes, such as `part={"Slot.frame[rarity=" + item.rarity + "]"}`
or `class:selected`; the kit does not draw its own band or rim for them.

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
token's icon cue with `<og-icon cue="danger">`, the icon the style
names for that token (an `icon` attribute wins). The primary layer takes
`og-icon::part(image) { color: … }`; an optional `secondary` icon takes
`og-icon::part(secondary) { color: … }` independently. `@media
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

`og style new <name> [--look <preset>] [--scheme dark|light] [--finish
<treatment>] [--out DIR]` writes a CSS directory, normally under
`assets/styles/<name>`. It includes both schemes, core-control rules,
transitions and shared sounds through `-og-sound`. Existing directories
are never overwritten. Edit the CSS directly after creation. Finish
treatments are flat, gradient, glass, hologram, neon, paper, bevel and brush.
`--scheme` chooses the fallback root palette; an explicit `data-scheme`
still selects the corresponding runtime palette.

`og style check <directory> [--json]` checks CSS and asset URLs and reports
root-token contrast ratios as facts. Ratios do not describe text over an
image or gradient. The separate `og style check-project` retains the
game's art-contract checker. `og style upgrade`, `art` and `pack` are
removed: CSS uses images directly without style-file or atlas conversion.

`og ui sheet <directory>` renders the core-control pages at desktopHd and
phonePortrait with 100% and 150% text, capturing overlapping scroll pages
to the bottom. `og ui review <directory> --locales
game,pseudo,zh,ar` adds locale coverage. Both accept `--out DIR` and
`--json`. Review the frames before recording references. The
[screens guide](../crates/og-api/guides/screens.md#3-style-packs) shows
directory loading, selectors, tokens and asset URLs.

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

`og ui add` copies a kit to `assets/ui-kits/<name>`, except its review
`sheet/` and its `tests/`, which stay with the kit so no game cooks the
sheet's sample art; `og style add` copies a style to `assets/styles/<name>`. A kit that composes library component
documents names them in its manifest's `components` (`["stat-bar"]`), and
`og ui add` ejects each the game does not already have into `assets/ui/`. Installed copies win over bundled copies
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

A kit's sheet loads its strings in the review's CJK and right-to-left
locales from its own `assets/strings/`, as a game does, and `og ui
review` installs those locales' font packs into its copy of the sheet.
A sheet without them repeats the game locale in those cells.

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

<!-- css-coverage:start -->
## 8. Standard CSS output-proof checklist

Source: [WebDX web-features 3.45.0](https://www.npmjs.com/package/web-features/v/3.45.0). All 454 entries with a `css.*` compatibility key are included: 257 widely available (high), 56 newly available (low), and 141 not Baseline. Including the newer entries keeps game-UI requirements visible rather than treating Baseline as a ceiling.

**Supported means the output assertion named in the row proves the listed scope.** It does not claim every syntax or subfeature grouped under that WebDX name. **Missing means an implementation gap or a missing output proof**, distinguished in the reason. A missing proof is not evidence that the implementation is absent. Parser acceptance, a property name in source, and an entire test file are not support evidence. This is a conservative, incomplete conformance inventory; unresolved rows stay visible.

Part applicability is proved only in the scopes named below: an element test does not prove that a widget `::part` lays out, transforms or paints the property. Inherited, pseudo-element, environment and lifecycle features need their own applicable part scenarios; there is no blanket all-properties claim.

Regenerate with `python3 scripts/css-coverage.py`; `--check` rejects document drift. For offline use, pass `--archive <web-features-3.45.0.tgz>`; its SHA-256 is pinned in the script. Proof entries are manually reviewed assertions, not name matches. Tests below live in `crates/og-ui/tests/` and run in the `og-ui` integration test binary; GPU proofs require Metal.

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
| `border-image` | Widely | Supported (scope →) | Supported (scope →) | [`css_paint::flex_ratios_absolute_fades_and_state_border_images_render`](../crates/og-ui/tests/css_paint.rs). A real button hover changes rendered border pixels from a shorthand source to a longhand gradient source; this does not prove all border-image modes. Part proof: [`dressing::part_host_negation_and_part_states_select_distinct_paint`](../crates/og-ui/tests/dressing.rs). Four primary/hover combinations assert exact resolved opacity and border-image presence for button:not(.primary)::part(base). No document-selector claim. |
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
| `clip-path` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `clip-path-animatable` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `clip-path-boxes` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
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
| `conic-gradients` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::repeating_gradients_repeat_the_stop_interval`](../crates/og-ui/tests/css_paint.rs). Metal pixel assertions for both halves of all four repeating conic quarters. |
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
| `cursor` | Not Baseline | Supported (scope →) | Pending parts-as-nodes | [`css_interact::the_pointer_passes_through_none_and_hidden`](../crates/og-ui/tests/css_interact.rs). Actual cursor routing through pointer-events:none and visibility:hidden, descendant overrides, URL priority/hotspot and accessibility presence. Immediate builder only; OS cursor loading is not covered. |
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
| `field-sizing` | Newly | Missing | Pending parts-as-nodes | No field-sizing layout implementation/output proof; fixed control sizing is not content sizing. |
| `file-selector-button` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `filter` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_filters::css_filters_match_their_reference_frames`](../crates/og-ui/tests/css_filters.rs). Metal references at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains. |
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
| `gradients` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::repeating_gradients_repeat_the_stop_interval`](../crates/og-ui/tests/css_paint.rs). Metal pixels prove repeating radial intervals with positive/negative starts, all eight conic half-quadrants and collapsed linear stops. Subpixel periods and unbounded stop counts are not covered. |
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
| `image-set` | Widely | Supported (scope →) | Supported (scope →) | [`css_paint::image_sets_render_density_correct_backgrounds_masks_and_borders`](../crates/og-ui/tests/css_paint.rs). Metal pixels at DPR 1/2/1 prove density-correct background/mask tiles, MIME filtering before duplicate removal (including quoted parameters and malformed-type rejection), zero-resolution transparency, percentage border slices with auto widths and gradient selection. Resolution math covers calc/min/max/clamp with mixed x/dppx/dpi/dpcm units; retained output also tests fractional DPR. Part proof: [`css_paint::image_sets_render_density_correct_backgrounds_masks_and_borders`](../crates/og-ui/tests/css_paint.rs). Metal pixels prove button::part(base) reselects its background at DPR 1/2/1 while preserving an 8 CSS px tile. Other part image properties are not separately proved. |
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
| `isolation` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
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
| `masks` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::masks_hide_invisible_boxes_and_fade_image_content`](../crates/og-ui/tests/css_paint.rs). Metal pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content. |
| `mathml` | Widely | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `media-pseudos` | Not Baseline | Missing | Pending parts-as-nodes | Output proof not yet reviewed; implementation status unverified. |
| `media-queries` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::media_queries_follow_dimensions_preferences_and_nested_intersections`](../crates/og-ui/tests/documents.rs). Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries. |
| `media-query-range-syntax` | Widely | Supported (scope →) | Pending parts-as-nodes | [`documents::media_queries_follow_dimensions_preferences_and_nested_intersections`](../crates/og-ui/tests/documents.rs). Computed widths at six viewport/preference combinations, including inclusive/exclusive range boundaries and nested queries. |
| `min-max-clamp` | Widely | Supported (scope →) | Pending parts-as-nodes | [`custom_properties::calc_reduces_once_its_variables_are_known`](../crates/og-ui/tests/custom_properties.rs). Computed lengths after variable substitution, typed attr updates/fallbacks, round(down), min/max/clamp and em/lh units. mod/rem are not covered by this round proof. |
| `min-max-content` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `min-max-width-height` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_layout::the_box_model_sizes_as_css_does`](../crates/og-ui/tests/css_layout.rs). Computed content-box padding, auto/percentage bounds, 16:9 ratio, logical dimensions and intrinsic widths. |
| `mix-blend-mode` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
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
| `rect-xywh` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
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
| `shape-function` | Newly | Supported (scope →) | Pending parts-as-nodes | [`css_effects::css_effects_match_their_reference_frames`](../crates/og-ui/tests/css_effects.rs). Metal references at 1280/390 compare isolated/grouped blends and rect, xywh, shape, content-box and SVG-reference clipping tiles. |
| `shape-outside` | Widely | Missing | Pending parts-as-nodes | Rectangular floats have layout tests; shape exclusion contours and shape-margin have no output proof. |
| `shapes` | Widely | Missing | Pending parts-as-nodes | clip-path does not prove shape-outside float exclusion. |
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
| `svg-filters` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_filters::css_filters_match_their_reference_frames`](../crates/og-ui/tests/css_filters.rs). Metal references at 1280/390 compare SVG filter graphs, primitive tiles and CSS function/url composition; this does not prove native glass backdrop chains. |
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
| `visibility` | Widely | Supported (scope →) | Pending parts-as-nodes | [`css_paint::masks_hide_invisible_boxes_and_fade_image_content`](../crates/og-ui/tests/css_paint.rs). Metal pixels prove hidden masked boxes do not paint, visible descendants retain the mask, and gradient masks fade both div and translucent image content. |
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
