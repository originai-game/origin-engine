# UI and the UI packs

Screens and HUDs for an Origin Engine game. The UI packs (genre kits and style packs) and how to install and change them, documents versus code, the thresholds for text, contrast and touch, og ui check and preview, and languages with their fonts.

The API: `og api show ui` (elements, components, layers, motion, styles)
and `og api show i18n` (string tables, plurals, fonts, line breaking,
right to left). `og ui list` names every installed and bundled kit and
style with its documents.

## 1. Order of work

1. List the screens the game needs from the brief: title, the HUD, pause,
   settings, results, and the genre's own (inventory, shop, deck).
2. Start from a library screen (`og ui eject --list`) or compose core
   controls in a screen document. No genre kits are bundled during the rebuild.
3. Fill it with the game's named data inputs; never edit text
   into a document that the game should bind.
4. Dress it with the look's style (§3), then preview and check (§5)
   at every breakpoint the create block's `platform` implies.

## 2. Genre kits

A kit is a directory with an `extension.json`, screen and component
documents, and typed data helpers registered with `registerUiExtension`.
Genre kits are not yet built. For local kits, see kits §1 to kits §5;
the retained shader finishes are documented in ui-materials §1.

- `og ui add <kit>` installs an editable copy under `assets/ui-kits/<kit>`;
  the game imports the installed `index.ts`. Edit the copy freely; an
  installation never replaces an existing one silently.
- `og ui eject <kit>/<screen>` copies one screen document with its
  companions into `assets/ui/`, where the game owns it.
- A kit never adds members to `og.ui`; its components are the namespace
  the kit exports. UI state lives in named scope sources and core
  interaction state, not module globals (determinism §1).
- New components of the game's own register with `registerUiExtension`
  the same way, so they are dressed by styles like the kit's.

## 3. Style packs

A style pack (`assets/styles/<name>/style.ogt`, or a bundled one named by
slug) sets tokens (colours, type, radii, spacing, motion), part dressings,
sounds and haptics for every component. The look's `ui "<slug>"` picks it;
`og style list` names the bundled ones.

Creation offers only `ui "default"` during the rebuild. This selects the
core style and adds no pack assets. No genre style packs are bundled yet;
local styles and `og style new` remain available.

- Change a style by ejecting it (`og style eject <slug>`) and editing the
  copy, or by `og style add`. Never restyle components one by one in code:
  a style change then misses them.
- `og style new` makes a game its own style from the look and a few
  choices, then fills its art slots through the art brief (art-review §2).
- Hand-painted frames go through `og style pack <dir>` (nine-patches named
  `<part>.9.png`), which writes the atlas and slice lines.
- In a pack, `style.ogt` is the data, in the CSS dialect (docs/ui.md
  §3.2): palette custom properties set colour (`--primary: #d4993a`),
  `typography` sets text roles, selector blocks paint named parts and
  states (`Button::base.primary:hover { -og-fill: var(--primary-hover) }`). Edit it directly and run `og cook`; never regenerate a pack.
  `choices.json` only seeds `og style new` and overwrites nothing.
- Fonts, atlases, import settings and their licences stay with the pack;
  asset paths are relative to the style unless they share `@styles/`
  assets. `og style add ./path` installs a local pack.
- A results screen is a document, not style: eject it
  (`og ui eject --list`), edit it under `assets/ui/` and keep game data in
  its named stores.

## 4. Structure and data

- Write one scriptless `.svelte` file per screen or component. Declare
  contracts with `<meta>`, structure with core HTML tags, and look in a
  scoped `<style>` block. TypeScript builds no UI trees.
- Read a named store with `{$gold}` or `disabled={$gold < 10}`. Use
  `bind:value={$volume}` for two-way controls. Ordinary attribute bindings
  never write. `onclick={buy}` names an action supplied by the game.
- Use `{#if expression}…{:else}…{/if}` and
  `{#each $items as item (item.id)}…{:else}…{/each}`. Every collection row
  needs a stable string key. Index keys and executable JavaScript fail cooking.
- Compose content with `{#snippet name()}…{/snippet}` and
  `{@render name()}`. Use `{@render name?.()}` for optional content.
- Expressions allow property access, comparisons, arithmetic, ternaries,
  pure number formatters, and `t(key, { named: expression })`. Assignments,
  arbitrary calls, scripts, runes, and raw HTML are errors.
- Style reactive state with `class:active={$step == 'brightness'}` and
  `.active` rules. The selector grammar includes types, IDs, classes,
  attributes, descendant, child and sibling (`+`, `~`) combinators, `:not()`,
  `:is()`, and `:where()`. `:where()` contributes zero specificity. `:not()`
  and `:is()` take the highest specificity of their arguments. `:has()`
  fails cooking.
- Structural selectors read the instantiated rows, after `{#if}` and keyed
  `{#each}`: `.row:nth-child(odd)`, `:nth-child(2n of .gem)`,
  `:nth-last-child(2)`, `:first-child`, `:last-child`, `:only-child`,
  `:empty`, and `.item + .item` separators all follow reorders, inserts and
  removals. Rows of a `list` restyle when they scroll into view.
  Interaction selectors name `:hover`, `:active`, `:focus`,
  `:focus-visible`, `:focus-within`, `:disabled`, `:checked`, `:selected`,
  `:invalid`, and `:placeholder-shown`. Ancestor selectors read that
  ancestor's interaction identity, not the styled child's state.
  Use CSS media conditions, for example
  `@media (width < 600px), (height < 600px) { ... }`.
  Supported features are `width`, `height`, `aspect-ratio`, `orientation`,
  and `prefers-reduced-motion`. Use inclusive `min-`/`max-` bounds or
  range comparisons, including `(600px <= width < 1024px)`.
  Commas and `or` mean alternatives; `and` intersects; `not` negates.
  Group mixed operators with parentheses. Nested `@media` rules intersect.
  `screen` and `all` are the supported media types. Square viewports are
  portrait. Lengths use the viewport's logical coordinates: `px`, absolute
  CSS units, and `em`/`rem` at the engine's initial 16px font size, independent
  of document styling. Unsupported features, zero-denominator ratios,
  viewport units, and `calc()` fail cooking instead of silently matching.
  Legacy names such as `@media phone` are not supported.
- Size a component to its slot with container queries: give the slot
  `container-type: inline-size` (or `size` for height conditions), optionally
  `container-name: card` or the shorthand `container: card / inline-size`,
  then write `@container card (width < 320px) { ... }`. Conditions take the
  media range syntax over `width`, `height`, `inline-size`, `block-size`,
  `aspect-ratio` and `orientation`, and read the nearest matching ancestor
  container, never the element itself. They settle before the first frame
  paints and again on every resize. Do not let a query resize its own
  container: that fails as a settlement cycle. Style queries, scroll-state
  queries and container units such as `cqw` fail cooking.
- Declare custom properties in any scoped rule, such as
  `.card { --spacing: 12px; gap: var(--spacing); }`. Declarations win by
  `!important`, specificity, then source order. Custom properties inherit
  from the instantiated parent, including through component mounts.
  They substitute on the declaring element before inheritance: changing
  `--a` in a child does not recompute an inherited `--b: var(--a)`.
  Recursive `var()` accepts fallbacks, including an empty fallback.
  Cycles invalidate the participating properties at computed-value time
  and report their names. A winning declaration with an invalid value
  does not revive a lower-priority declaration. Unshadowed engine tokens
  resolve from the active style. `::part` declarations use the same cascade.
- Transition values with standard CSS: `transition: opacity 120ms ease-out,
  translate 200ms ease-out 50ms` or the longhands `transition-property`,
  `-duration`, `-timing-function` and `-delay`; `all` covers every
  animatable value, and a later entry wins for its property. The timing slot
  also takes `-og-spring(300ms, 0.2)` and `var(--motion-ui)`.
- Animate with scoped `@keyframes` and `animation` or its longhands
  (`animation-name`, `-duration`, `-timing-function`, `-delay`,
  `-iteration-count`, `-direction`, `-fill-mode`, `-play-state`). An element
  runs several animations; later ones win per value. Keyframes take `from`,
  `to` and percentages; a missing 0% or 100% uses the element's own value,
  and `animation-timing-function` inside a keyframe times the segment after
  it. Rules choose animations like any property, so `#Card.open { animation:
  … }` or `:hover` starts one and removing the match ends it. Reduced motion
  shows a finite animation's end and stops endless ones.
  `@keyframes` may sit inside `@media` or `@container`, for example a
  shorter entrance on phones: the last definition of a name whose
  conditions hold is the one that plays, and switching to another
  definition restarts the animation.
  A keyframe's `-og-sound: error` emits the active style's named sound once
  when playback crosses that frame, at the recorded playback rate.
- Bind structured rich text with `<text spans={$spans}>{$body}</text>`.
  Each `TextSpan` has `name`, `position`, `length`, and scalar `properties`.
  Offsets count Unicode scalars, not bytes or UTF-16 units. Style spans with
  `text::part(name)` and property selectors such as
  `text::part(name)[rarity="rare"]`. Later overlapping declarations win per
  property. Unknown span names remain metadata; the text is never markup.
- Use `<meta name="screen" content="pause" data-layer="menu"
  data-focus="Menu/Resume" data-pause="true">` for a screen.
  Components use `name="component"`. Props use `name="prop:volume"`,
  `content="number"`, and `data-doc="…"`; actions use `name="action:back"`;
  fixtures use `name="fixture:default"` with JSON content.
- Mount a screen with stores, not a viewmodel: `og.ui.scope(world,
  "menus").mount("pause", "pause", { volume })` puts it on the layer stack
  while it is mounted; Back unmounts it. Give it a CSS view transition with
  `@view-transition { navigation: auto; }`: it cross-fades by default, and
  `::view-transition-new(root) { animation: 300ms rise; }` or the engine's
  covers (`::view-transition-old(root) { animation: 400ms -og-iris; }`)
  change how.
- Compose with components and snippets, not code callbacks:
  `<Card title={item.name}>{#snippet body()}<p>{item.desc}</p>{/snippet}</Card>`,
  rendered in `Card` with `{@render body?.()}`. Pass content to a mount by
  reference: `scope.mount("pause", "pause", inputs, { side: "game#pauseSide" })`.
- A declared input action also exposes a named event: after declaring
  `action:confirm`, use `onconfirm={start}` to handle it independently of
  pointer clicks. The handler names a mounted game action, just like `onclick`.
- Visible words use translation keys. Declare the base string table with
  `<meta name="strings" content="./pause.en.ogt">`; the cook tracks it as
  a dependency, and the loader preserves game translations when merging it.
- Probe edits preserve surrounding markup and save style edits as scoped
  rules. Diagnostics name the source file, line, column, and correction.

## 5. Thresholds, preview and check

`og ui check` lays out every screen per style, breakpoint and locale and
reports findings and measures; it never passes or fails a screen. The
bars it does not hold, this guide does:

- **Text size**: at least 18 px at 1080p for menus, body and controls. A
  dense HUD readout may go to 12 px if its group sets
  `ElementProps.dense` and it still grows with the player's text scale.
- **Text scale**: every screen usable at 150 % text, nothing cut or
  overlapping.
- **Contrast**: live text at least 4.5:1 against what it is drawn on,
  3:1 for text over 24 px and for icons and control outlines; 7:1 in the
  high-contrast mode. No meaning by colour alone.
- **Touch**: a control's shorter side at least 44 CSS px on a coarse
  pointer, and 8 px between neighbouring targets; primary actions in the
  bottom half on a phone held in portrait.
- **Pad**: every interactive element reachable from the default focus,
  Back on every layer, no focus trap.
- **Primary actions**: one filled primary button per layer.
- **Cost**: a dense menu (300 elements, 40 animating) within 0.5 ms of
  CPU a tick, a HUD within 1 ms; the element count `og ui check` reports
  is where to look first.

Every finding is fixed (an undeclared overlap, cut text, the safe area,
missing names, missing feedback, placeholder art). A declared overlap is
for elements that are meant to overlap, never to silence a finding.

`og ui preview` renders the contact sheet (a row per breakpoint, a column
per locale) and, with `--film`, a motion's filmstrip: read it before
saying a screen is done. Review UI frames over the style's plain ground,
not a busy scene; a HUD over the scene is judged by the scene's frames.
Read a phone layout at 390×844 and a desktop one at 1920×1080.

## 6. Languages and fonts

- Every create-block language needs its string table
  (`assets/strings/<lang>.ogt`) and a font that covers its script. Fonts
  ship as one pack per script (`og font list`): `og create --lang`
  installs the packs of its languages, `og font add <pack>` adds one
  later, and og.i18n chains them with no code: the locale's own pack
  first, another pack loaded the first frame its characters are drawn
  (a player's name in another script). `og check` warns while a
  language has neither a pack nor a table with its own fonts line.
- A pack's glyphs are drawn from its outlines when first shown and the
  web fetches its face only when needed, so installing a pack costs the
  download of a language in use, not the boot.
- `og i18n extract --check` lists missing keys per table; run it before
  each build.
- Size boxes for the pseudo-locale (40 % longer) and check a CJK and a
  right-to-left locale with `og ui check --locales`; a box sized to its
  English text overflows in German and mirrors wrongly in Arabic.
- Numbers, plurals and dates go through og.i18n; never concatenate
  translated fragments.
