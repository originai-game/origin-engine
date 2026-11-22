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

The [screens guide](../crates/og-api/guides/screens.md) describes the
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
- `{#each labels as label, i (label)}` iterates a string-list prop or
  writable value, keyed by each string itself. Strings must be nonempty
  and unique; repeated labels need object rows with distinct keys, not
  index keys. Literal string-list defaults work too. Reordering preserves
  the kept strings' controls, focus and state; changing a string removes
  its old row and creates a new one. Strings remain strings when read or
  passed to another component, and cannot be written through the row alias.
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
- JSX, `$state` (instance state is metadata, see Components in
  documents), `$effect`, `$bindable` outside a prop's default,
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
  Alternatively, `out:fade={{ motion: 'exit' }}` reads a style's whole
  motion token (`ui`, `panel`, `emphasis`, `playful`, `enter`, `exit`) at
  playback, including its duration and easing. `motion` cannot be mixed
  with `duration` or `easing`; `delay` and the transition's geometry still
  apply. `animate:flip={{ motion: 'panel' }}` uses the same tokens.
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

Native `<tabs>` scrolls a long set horizontally by default; `fill` makes
its tabs equally wide, without cutting their labels. For a short set that
must all be visible, write `<tabs fit items={$labels}>`: side padding
compresses to `space.xs` first, then the tabs share the track in proportion
to their contents. Labels wrap at word boundaries to at most two lines;
an overwide word ellipsizes on its own line instead of splitting, and the
full text remains the accessible name. `fit fill` keeps equal widths
when those fit and uses proportional widths when they do not. Give
icon-only tabs `items={$emptyLabels} icons={$icons} aria-labels={$names}`:
the parallel `aria-labels` string list supplies their accessible names.

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

#### Inline elements, inline objects, span parts and translated markup

A text element's inline markup is elements (kind `inline`, CSS 2 §9.2.2):
`<span>`, `<a href>`, `<b>`, `<strong>`, `<i>`, `<em>`, `<u>`, `<s>`,
`<del>`, `<sub>`, `<sup>`, `<ruby>`, `<rb>`, `<rt>`, `<rtc>`, `<rp>` and an
SVG `<tspan>`, each with any attributes (`class`, `id`, `data-*`, `lang`,
`style`, `part`). Selectors match them as any element: class, id,
attribute, structural (among their text's inline elements), child and
descendant, and states (`a:hover`, from the pointer over a link's words or
the probe). Every inherited text property and `vertical-align` and
`display` apply to the words in them. Their margin, border and padding
take room on the line before their first character and after their last
(CSS 2 §10.3.1), the first and last characters shaped apart; across the
line they grow the element's fragments, never the line (§10.6.1). Under
`box-decoration-break: clone` each line holds both edges, and the line
breaks with room for them (CSS Fragmentation 3 §5.4). An edge's side is
its words' direction's: a wrapped element whose words run against its
own direction (Chinese in a `dir="rtl"` span) takes its left edge on its
first line, not CSS 2 §8.6's right. `width`, `height`
and percentage or `auto` margins and padding take no room on them. Each
one styling its words or taking room is a node under its text (kind
`inline`, in the probe), out of its flow: its fragments are one per line
it has words on, each its border box around its words' content area (CSS
2 §9.4.2), and its box their union. A link is hit on its fragments, not
across the lines between them. HTML's look is the UA sheet's (`b { font-weight:
bold }`, `sub { vertical-align: sub; font-size: smaller }`, the ruby
`display`s), so a rule overrides it. An inline element takes no event
handlers, `bind:`, `use:` or transitions: put them on the text. In a block,
a formatting element joins the words around it into an anonymous text; a
`<span>` or `<a>` with attributes there is a text of its own, as a flex
item is.

A text element's words may hold one-em inline objects with literal
attributes: `<img src="coin" alt="">` (a registered picture,
`UiDesc.images`), `<og-icon name>` (the icon layers) and `<kbd
data-action>` (the action's glyph). Icon lookup uses the game's/style's
atlas, then `UiDesc.fallbackIcons`, then the engine's Lucide vectors.
`og_ui::DEFAULT_ICON_NAMES` lists that last set; its SVG filenames under
`third_party/lucide/icons/` give the same names for repository checks.
The vectors allocate and parse only when drawn, once per used icon; a
game supplying every requested name loads no default geometry or texture.
Unknown names still warn. `<span part="big">` names its element
for `p::part(big)`, whose declarations win over its element's rules and
which takes `color`, `font`, `font-family`, `font-weight`,
`font-style`, `text-decoration-line`, `-og-text-motion`, `vertical-align`,
`text-combine-upright` (down a vertical line, `all` sets the run in one
upright em, one unit of the line), and `font-size`, `line-height` and `letter-spacing` in px or em. A run's
`vertical-align` follows CSS 2.1 §10.8 and CSS Inline 3 §4: the
shorthand sets `alignment-baseline`, `baseline-shift` and
`baseline-source`, and a box's alignment baseline (its parent's
`dominant-baseline` for `baseline`) meets the same baseline of the
paragraph's root text. A font's baselines are its `BASE` table's, the rest
synthesized as Appendix A.2 suggests (ideographic over and under from the
ascent and descent, mathematical at the minus sign's centre, else the
x-middle, hanging at 0.6em); `middle`, `text-top` and `text-bottom` place
a box as CSS 2.1 does. `baseline-shift: center` centres a box on its line
box; on a run of text itself it keeps its baseline place (Parley has no
line-relative centre). An inline block aligns by its last baseline
(`baseline-source: auto`): a text's last line, or its last in-flow
child's. Runs align against the paragraph's root text, not an enclosing
run's. A translated
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
1em, nested lists none; headings 0.67em to 2.33em of their step). Heading
defaults set size, weight and tracking, not a `font` shorthand: headings
and `<small>` retain the document's inherited font family and line height.
The engine differs or limits as follows:

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
  cell centres its content unless `vertical-align` is `top` or
  `text-top` (the start), `bottom` or `text-bottom` (the end), or a
  baseline (the start).
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
- **Ruby and emphasis marks.** `<ruby>`, `<rb>`, `<rt>`, `<rtc>` and
  `<rp>` are inline elements of a text with the UA sheet's ruby
  `display`s, and any inline element takes `display: ruby`, `ruby-base`,
  `ruby-text`, `ruby-base-container`, `ruby-text-container` and `none`
  (and `ruby-position`), from a rule or its `::part`. CSS Ruby 1 §2.2's anonymous boxes and §2.3's pairing
  make each segment one unbreakable box of its line, with any number of
  levels and nested rubies; `ruby-merge` is not read (`separate`).
  Emphasis marks are drawn shapes; their side follows the content language through the user-agent rules (under for `zh`, over
  for `ja` and `ko`); a ruby's base takes them outside its annotations,
  its `<rt>` none.
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
  adds no binding and exists only while `content` makes it. Their
  `content`'s alternative text (after `/`: strings, counters and
  `attr()`) is what the accessibility tree reads in place of what they
  draw, inside their text element's text or as their own text record;
  an empty one leaves them out. Counter
  styles are the predefined ones plus `@counter-style`. Counters run in
  document order; a retained subtree replays its counter effects and
  rebuilds only when counters enter it differently. `::marker` takes
  `content` and `color`; `::first-line` rebuilds its element every frame;
  `::placeholder` takes `color`; `::selection` takes `background-color`,
  `color` and `text-decoration`, its glyphs redrawn clipped to the
  selected runs (selected text keeps the text's theme shadow). `::highlight(<name>)` takes the same four
  and styles the ranges the game registers under that name with
  `og.ui.setHighlight` (CSS Custom Highlight API); a highlight with no rule
  paints nothing. The overlays stack under the selection's by priority;
  of equal priorities, the highlight registered later is above. In
  both, each property comes from the nearest of the text's element and
  its ancestors that declares it, `background-color` too (highlight
  inheritance), so `:root::selection { … }` styles every element. `::file-selector-button` is a file input's button, a box of its own beside the names chosen: its rules style it as they style a `<button>`, and the input takes the press and the focus. The CSS 2
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
  functions refuse mixed units. Margins accept these functions too: their
  percentages read the containing block's width on every side, including
  logical margins, while font-relative terms compute before layout.
  NaN is 0 and infinity the largest value.
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
  item without `flex` properties is CSS's `flex: 0 1 auto`: it does not
  grow, and shrinks with its siblings when the line overflows. `order`
  keeps the document's reading order.
- **Grid.** One `repeat(auto-fill | auto-fit, …)` per axis, of fixed
  sizes or `minmax(<fixed>, <flexible>)` (`repeat(auto-fill,
  minmax(min(140px, 100%), 1fr))`, whose tile shrinks to a grid narrower
  than one). A track size is a length, a percentage, a math function of px
  and % (`calc()`, `min()`, `max()`, `clamp()`; its % reads the grid's
  inner size), `fr`, a content keyword, `minmax()` or `fit-content()`.
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

Core control focus outlines follow the painted part, not its larger input
reach. `input[type=range]::part(focus)` follows the moving knob, with
`--state-focus-ring-offset` outside it; the track keeps the slider's focus
stop, keyboard adjustment and accessibility role.

Text and fonts follow Fonts 4, Text 3 and 4 and Text Decoration 3, beside
the house form `font: var(--text-<step>)`. A generic family picks the
style's body, display or mono role. `text-transform` cases for the UI's
locale. `text-wrap-style: balance` breaks at the narrowest width keeping
the line count, `pretty` at the widest that keeps it with no one-word last
line, `stable` is `auto`. `hyphens: none` drops soft hyphens.
CJK spacing follows Text 4 §8.4 and §8.5, planned on the lines a pass
broke and shaped again until it holds: `text-autospace` puts 1/8ic
between an ideograph and a Latin letter or digit on one line (never at a
break, so it never pushes a character to the next line), and French
no-break spaces under `punctuation`; `text-spacing-trim` sets fullwidth
punctuation half-width by the face's `halt` (a face without it keeps the
full width), a line's start and end by its keywords, `allow-end` pulling
the next line's character and closing bracket back when only trimmed it
fits. Both are the paragraph's values, horizontal lines only.
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

- Painter levels preserve submission order where paint overlaps. A font
  followed by a shape can share a level because the renderer already
  sorts fonts before shapes; the reverse crossing advances. Ambiguous
  pipeline crossings keep conservative levels. Top-level untransformed
  round-rectangle outer shadows use the analytic shader's compact Y
  support with device-pixel guards, not a filled rectangle around the
  unshifted box. Other geometry, transforms and offscreen targets keep
  broad bounds. Retained levels use the same bounds and invalidate when
  the viewport's raster scale changes. Emit into an overlay with the
  viewport's size and DPR, as required by `UiFrame::emit`; a separately
  rescaled overlay is not that viewport's raster target.
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
  `<clipPath>`; clipping is by analytic path coverage. `polygon()` points
  accept mixed length-percentage math such as `calc(100% - 12px)`;
  horizontal percentages use the reference box's used width, vertical
  percentages its used height, and pixel offsets stay fixed on resize.
- `filter` functions are layers in order; a run of colour functions is one
  matrix, split where a value can leave 0..1, so CSS's clamps hold.
  `url(#id)` names an inline SVG `<filter>` (below); a `url()` naming none
  drops the list.
- `backdrop-filter` captures what is drawn under the element in its
  backdrop root (the screen, an ancestor's group, or a world panel's,
  ground's or 3D plane's layer), over the border box grown by three
  deviations of each blur and the shadows' reach, clamping reads at the
  capture's edge. Its blur is an exact Gaussian; the theme's
  `-og-effect: backdrop` glass keeps the pyramid. Backdrop sampling is
  opt-in for the whole UI (below); off submits no backdrop operation,
  including standalone filter chains, rather than a zero-blur pass.
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
floating layers are not anchored. `scroll-marker-group: before | after`
(Overflow 5 §3) builds a `::scroll-marker-group` box beside a scroll
container holding the `::scroll-marker` (given a `content`) of each
element inside it, in tree order; a marker scrolls its element to the
container's start when pressed. `::scroll-button(<dir> | *)` with a
`content` builds a button before the container that pages 85% of its
view, and matches `:disabled` where it cannot scroll further. The marker
for the section in view matches `:target-current` (those before and
after it `:target-before` and `:target-after`), by §3.1.8's example
algorithm at the offset a glide comes to rest at; the one pressed stays
current while the container stays where it scrolled to. On
`scroll-target-group: auto` the `<a href="#id">` links inside take the
same states. The states follow one frame after the scroll (saved UI
state). A modal
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

A document holds UI state of its own as instance state:
`<meta name="state:open" content="bool">` (a `number`, `string` or `bool`,
with `data-default`) gives every instance of the document, and every host
of a control document, a value of its own. The document reads it by its
name and writes it with `bind:` (`<details bind:open={open}>`,
`<input type="checkbox" bind:checked={open}>`); nothing outside the
instance reads it, and a state shares no name with a prop. It is for what
the game has no stake in: a picker being open, the tab shown, a row
expanded. What the game must know stays a store the game binds. The value
is a store of the UI's reactive session named by the instance's mount and
path, so a snapshot carries it and a restored instance finds its own; it
goes when the instance does (a row removed and added again starts from
the default). `$state` stays refused.

A control is its HTML element (§2.4): og-ui holds the element's value,
binding, role, focus and behaviour, and the element's inside is a control
document of the library. Selection, drag-and-drop, scrolling, focus, and
keyboard and gamepad navigation are og-ui capabilities any element takes
by attribute or CSS. Components compose the control elements rather than
draw substitutes: a strip of buttons is not tabs, and a `<div>` with a
width is not a `<progress>`. Generic meters, counters, timers, slots,
lists, grids, tables, charts, tooltips, cards, rewards, confirmations,
and node graphs are ungrouped library documents. Domain names do not
justify duplicate controls.

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

`disabled` removes the focus stop and rejects edits, including when a focused
field becomes disabled. Its frame and value parts match `:disabled`; this
does not make a `readonly` field disabled.

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

A scroll container with no scroll range on either axis does not take a
press from a control around it. Its scroll-container membership remains
for wheel handoff and overscroll boundaries. A container with overflow
can still take a press to drag-scroll; its actionable children keep their
own presses.

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

A node with a `live` prop, or the role `status`, `log` (polite) or
`alert` (assertive), is a live region in the accessibility tree
(crates/og-ui-next CONVENTIONS.md §Accessibility); `status` and `alert`
are atomic. The engine keeps no announcement queue: the web mirror gives
the node `aria-live` and `aria-atomic` with its text as content, and a
native window's AccessKit adapter announces it, so the platform says
what changes in it. Both work in publish builds.

```svelte
<text role="status">{$saved}</text>
<column aria-live="polite" aria-atomic="true"><text>Gold</text><text>{$gold}</text></column>
{#each $alerts as a (a.id)}<text role="alert">{a.text}</text>{/each}
```

### 2.4 Controls and control documents

A control is HTML's element with HTML's attributes: `<input>`, `<select>`,
`<progress>`, `<meter>`, `<details>`, `<dialog>` and the rest keep their
tags, bindings, roles and keys, and a game's markup names no component
for them. og-ui keeps what a document cannot express: the value and its
binding, the role, focus, the behaviour and the element's states. The
element's inside is a control document of the library
(`packages/engine/src/ui/controls/`), instantiated under every element its
selector matches, as a browser's user-agent shadow tree is. Components and
packs dress controls and never draw substitutes.

A control document is a component document that names its host in place
of a component name: `<meta name="control" content="progress">`, a tag
with optional attribute tests (`input[type=range]`). Of two that match an
element, the one with more tests is its control. Its contract:

- **Props read the host.** A declared prop reads the host's authored
  attribute of its name, bound or literal (`bind:value` passes a writable
  prop), else its default; an `any` prop is null while the attribute is
  absent. A state og-ui derives for the host is a prop of the same kind
  (`<meter>`'s `region`). A control document takes no content prop its
  host's element does not have.
- **`:host`.** `:host` and `:host(<compound>)` select the host element,
  and a `:host` rule takes every property there: the control's default
  size and display are its document's. `:host([value])` reads the
  attribute a prop of that name reads, `:host(:hover)` the host's live
  state; the state goes inside the parentheses. `:host(<compound>) .inner`
  styles an element of the document by the host.
- **Parts.** An element with `part="track"` is the host's
  `::part(track)`, listed in the document's `<meta name="parts">`. A part
  keeps its name for as long as the control exists; where CSS names the
  box with a standard pseudo-element (`::backdrop`, `::details-content`,
  `::picker(select)`, `::placeholder`, `::selection`) the control takes
  the standard name alone. A control written inside a control document
  makes parts of its own inside parts of that document's host with
  `exportparts` (CSS Shadow Parts §4.2): `<progress
  exportparts="fill: bar-fill">` in the meter's document lets a page
  write `meter::part(bar-fill)`. Only a control in a control document
  exports; a component instance exports nothing.
- **Standard pseudo-elements.** An element declared `pseudo="name"` is
  the host's standard pseudo-element of that name, a namespace apart from
  `part`: `backdrop` (a `<dialog>`'s or a popover's), `details-content`
  (`<details>`), `picker` and `picker-icon` (`<select>`, styled as
  `::picker(select)` and `::picker-icon`), `checkmark` (`<option>`) and
  `file-selector-button` (`input[type=file]`). One element of the
  document is each, only in a control document whose host HTML gives
  that pseudo-element; a summary's marker stays CSS's `::marker`. The
  host's document and the style reach it as they reach a part
  (`details::details-content`, depth +1); no `exportparts` names it. A
  state pseudo-class after it reads the element's own state
  (`input::file-selector-button:hover` is the pointer on the button). The
  document styles its own element as `[pseudo="backdrop"]`; its
  `:host::backdrop` is refused.
- **Rank.** The document's own rules style its elements at depth 0
  (§3.2), in the `og-components` layer. A `::part()` rule of the default
  style, a pack or the document the host stands in reaches a part at
  depth +1 and wins over them; the document's `:host` rules reach the
  host at depth -1 and lose to every rule written for the host where it
  stands.
- **Inheritance.** The host's custom properties and inherited properties
  pass into the document, so a token set on a control reaches its parts.
- **Values.** Its `<style>` reads tokens only (§3.1), as every built-in
  document's does; the control's hook tokens (`--meter-*`, `--slider-*`)
  are what a pack sets.
- **Loading.** The cook bundles a control document when a game's
  documents use its element, and a screen's load brings the controls its
  elements need. `og ui eject controls/<name>` copies one into the game,
  whose copy then is the control for that element everywhere.

An element with no control document loaded is drawn by og-ui's own
fallback for its kind while the controls move to documents
([roadmap](roadmap.md)); the documents so far are `progress` and
`meter`. `<hr>` has no inside: the default style's `hr` rule draws it as
a `--stroke-hairline` line in `currentColor` (`--line`), across its
container or, with `vertical`, down it, and any rule for `hr` restyles it.

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

`select { appearance: base-select }` is HTML's customizable select: the
select is built from its own children, not the native dropdown, and the
document styles every part. Its first `<button>` child is the trigger's
content (with none, the chosen option's label), and `<selectedcontent>` in
that button shows a copy of the chosen option's content. A press on the
select opens `::picker(select)`, an auto popover in the top layer anchored
to it (CSS anchor positioning, under it by default), holding its other
children: `<option>`s with any content, `<optgroup>`s and `<hr>`s. A press
on an enabled option chooses it, writes its value (to the binding, else as
the select's own), fires `onchange` and closes the picker; a press outside
or Back closes it unchanged. The chosen option matches `:checked`, a
disabled one `:disabled`, and the select `:open` while its picker is.
`::picker-icon` and `::checkmark` come with control documents (their
`pseudo=` elements); until then a document draws its own.

`<dialog bind:open aria-label>` follows HTML. `modal` (a literal, never
bound: a modal's content is built in the top layer under `<id>/Panel`) is
`showModal()`: a scrim taking every press, focus moved in, trapped and
returned to the opener, the panel centred in the viewport.
The panel's border box is capped to the safe surface after styling,
including an author's width, height and minimums. The scrim's padding
and device/host-covered insets leave the available room; a smaller
dialog keeps its preferred size. Unmarked content scrolls as a whole.
For fixed title and actions, put `data-layout="shell"` on the dialog
and wrap only its body in `data-layout="scroll" data-fill`. The body
shrinks to the remaining space and scrolls independently; the engine
does not infer regions from heading, paragraph or footer tags. Keep
fixed children small enough to fit together; the whole-panel fallback
remains available when they cannot. These modal constraints do not
change non-modal dialogs, popovers or generic overlay primitives.
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
`Field.frame`, `Row.frame` and `Head.frame`. A slot that paints its own
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
smaller than the art (CSS border-image). Placeholder characters are not
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

A text shadow (a theme `text-shadow-<name>` class; arbitrary
`text-shadow-[…]` values are not supported) draws each glyph's
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
| `fantasy` | Painted frames and materials with room for dense content | Baldur's Gate 3, Diablo IV | Built, shipped with the engine; art-approved (pilot) |
| `scifi` | Instrument hierarchy, precise geometry, sparse emphasis | Elite Dangerous, Starfield | Built, shipped with the engine; awaiting art approval |
| `cartoon` | Rounded forms, deliberate thickness, clear primary actions | Mario Kart 8 Deluxe, Bloons TD 6 | Built, shipped with the engine |
| `pixel` | Consistent pixel grid, integer scaling, readable glyphs | Stardew Valley, Octopath Traveler | Not yet built |
| `storybook` | Paper and illustrated edges, quiet ornament, readable text | Pentiment, Strange Horticulture | Not yet built |

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
own look-to-UI recommendations: toy-3d and cartoon-2d name cartoon,
painterly-3d and pixel-3d-hybrid fantasy, neon-vector scifi, the rest core
default. A recommendation is a seed, not proof of fit; an explicit
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

