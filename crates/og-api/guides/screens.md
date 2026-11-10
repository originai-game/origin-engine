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

A style pack is a directory of CSS and assets, with no manifest or
`style.ogt`. Structure stays in `.svelte` documents. Haptics stay in `og.*`.

1. Run `og style new my-style --look neon-vector --scheme dark` in your
   project. It writes `assets/styles/my-style/main.css` and refuses to
   overwrite an existing directory. `--out` chooses another directory.
2. Edit its `:root` custom properties and part rules. The generated file
   includes dark and light palettes and core-control states. `--finish`
   accepts flat, gradient, glass, hologram, neon, paper, bevel, or brush.
   `--scheme` selects the fallback palette; the UI's explicit `data-scheme`
   selects its dark or light palette at runtime.
3. Run `og style check assets/styles/my-style`. Fix parser diagnostics
   and unresolved URLs. Contrast reports root-token ratios, not a grade
   or a guarantee about text over images and gradients.
4. Run `og ui sheet assets/styles/my-style` to inspect every core-control
   page at desktopHd and phonePortrait, at 100% and 150% text scale.
   Use `og ui review assets/styles/my-style --locales game,pseudo,zh,ar`
   for locale coverage. Scrollable pages produce overlapping captures down
   to the bottom without changing the viewport. Review frames before
   recording references.

Load and select the directory in your game's startup function:

```ts
const style = og.load.style(engine, "styles/my-style");
if (!style.ok) throw new Error(style.reason);
og.ui.configure(engine, { style: style.value });
```

Write lowercase markup hosts before `::part(name)` and states after it:

```css
:root { --my-accent: #72e8dc; }
:root[data-scheme=light] { --my-accent: #14675f; }
button.primary::part(base) {
  background: var(--my-accent);
  border: 2px solid #184d55;
  box-shadow: inset 0 1px 0 #ffffff60, 0 3px 6px #00000040;
  transition: background 140ms ease-out;
  -og-sound: press url(/@styles/shared/sfx-tap.ogt) volume(0.2 0.3);
}
button.primary::part(base):hover { background: #a9fff2; }
@media (prefers-reduced-motion: reduce) {
  button::part(base) { transition: none; }
}
```

Use `@font-face` for fonts, `mask-image` for icons, `border-image` for
painted frames, and `@keyframes` for animation. Asset URLs resolve beside
the CSS file that contains them. `/@styles/shared/` addresses shared
assets. Copy fonts with their licences. `og style add ./my-style` installs
a local pack without overwriting an installed one.

Direct-child CSS files load in lexical filename order. Put sheets that
need explicit ordering in a subdirectory and use `@import`. Normal
declarations increase in priority from `og-default` through `og-kit`,
`og-pack`, `og-game`, then unlayered CSS. Important declarations reverse
layer priority. Custom properties belong to authors, not a second token
language. The only appearance extensions are `-og-material` and
`-og-sound`; unknown `-og-*` declarations are errors.

Core-widget host rules and inheritance require parts-as-nodes. Until that
lands, style their exposed parts directly. `og style upgrade`, `art`, and
`pack` are removed: CSS images need no style-file conversion or atlas
writer. The game's art-contract measurements remain available through
`og style check-project`.

## 4. Structure and data

- Write one `.svelte` file per screen or component. Declare contracts
  with `<meta>`, structure with core HTML tags, and look in a scoped
  `<style>` block. TypeScript builds no UI trees. A `<script lang="ts">`
  holds only `let { gold = 0 } = $props()` and `const x = $derived(expr)`;
  state and logic live in a TS system that writes stores.
- Read a named store with `{$gold}` or `disabled={$gold < 10}`. Use
  `bind:value={$volume}` for two-way controls. Ordinary attribute bindings
  never write. `onclick={buy}` names an action supplied by the game.
- Controls are HTML's, with a few `-og-` properties where HTML has none:
  - `<button data-hold="1200" onhold={confirm}>` fires after a held press
    (Enter, Space, a pad's A, or its `data-action`'s input hold it too) and
    draws a fill across it as it is held: `button::-og-hold { background }`.
  - `<div id="Tip" popover>` opens against the control whose
    `popovertarget="Tip"` (or `interestfor`) names it. Place it with CSS
    anchor positioning against that control: `position-area: right`, an
    inset such as `left: 8px` for the gap, `position-try-fallbacks:
    flip-inline`. `#Tip::-og-arrow { width: 10px }` draws an arrow on the
    side facing the control. `contextmenu="Menu"` on an element opens
    popover `Menu` at the pointer on a right-click or long press.
  - `<input type="checkbox" switch data-on="On" data-off="Off">` is a
    switch whose on and off words a screen reader reads.
  - `<input type="number" data-format="{value, number} m">` with
    `-og-stepper: buttons` steps; `-og-stepper-wrap: wrap` goes past max to
    min and `-og-stepper-large-step: 10` sets a held button's repeats.
    `data-format` is an ICU message over `value`.
  - A field's icon, suffix and count are markup round an input with
    `appearance` set to `none`, in a row styled with `:focus-within`:
    `<icon>`, `<span>gold</span>`, `<small>{$name.length}/20</small>`.
    `<input type="search">` draws a clear button while it has text.
  - `<select>` with `-og-select-filter: on` puts a filter on its open list
    (`data-filter-placeholder`); `auto` shows one past 10 options.
  - `<tabs items icons badges>` draws an icon and a count badge per tab;
    the tabPrev and tabNext actions (Q and E, LB and RB) step it.
  - `<details name="faq">` is an exclusive accordion: one of a name open
    at once.
- Use `{#if expression}…{:else}…{/if}` and
  `{#each $items as item (item.id)}…{:else}…{/each}`. Every collection row
  needs a stable string key. Index keys and executable JavaScript fail cooking.
- Compose content with `{#snippet name()}…{/snippet}` and
  `{@render name()}`. Use `{@render name?.()}` for optional content.
- `{#key $round}` remakes its content when the value changes, `{@const n =
  item.count}` names an expression in a block, and `{#await $load}…{:then
  v}…{:catch e}{/await}` reads a store holding `{ status, value, reason }`.
- Motion: `transition:fly={{ y: 20, easing: quintOut }}`, `in:`, `out:`
  (fade, fly, slide, scale, blur, crossfade by key) and `animate:flip` in a
  keyed each, or CSS. `style:width={$w + 'px'}` and `style="…"` win over
  rules. `bind:this={panel}` names an element; `use:tooltip={params}` sends
  `use` events (create, update, destroy) to the game.
- Expressions allow property access, comparisons, arithmetic, ternaries,
  pure number formatters, and `t(key, { named: expression })`. Assignments,
  arbitrary calls, `$state`, `$effect`, and raw HTML are errors.
- Style reactive state with `class:active={$step == 'brightness'}` and
  `.active` rules. The selector grammar includes types, IDs, classes,
  attributes (`=`, `~=`, `|=`, `^=`, `$=`, `*=`, with `i`/`s`), descendant,
  child and sibling (`+`, `~`) combinators, `:not()`, `:is()`, `:where()` and
  `:has()`. `:where()` contributes zero specificity. `:not()`, `:is()` and
  `:has()` take the highest specificity of their arguments.
- Structural selectors read the instantiated rows, after `{#if}` and keyed
  `{#each}`: `.row:nth-child(odd)`, `:nth-child(2n of .gem)`,
  `:nth-last-child(2)`, `:first-child`, `:last-child`, `:only-child`, the
  `-of-type` forms, `:empty`, `.card:has(> .hot)` and `.item + .item`
  separators all follow reorders, inserts and removals. Rows of a `list`
  restyle when they scroll into view.
  State selectors name `:hover`, `:active`, `:focus`, `:focus-visible`,
  `:focus-within`, `:disabled`, `:enabled`, `:checked`, `:selected`,
  `:indeterminate`, `:default`, `:invalid`, `:valid`, `:user-invalid`,
  `:required`, `:optional`, `:read-only`, `:read-write`, `:in-range`,
  `:out-of-range`, `:placeholder-shown`, `:open`, `:modal`,
  `:popover-open` and `:target`; also `:lang()`, `:dir()`, `:state(name)`
  (from `state:name={…}`), `:defined`, `:scope` and `:any-link`. Ancestor
  selectors read that ancestor's interaction identity, not the styled
  child's state.
  Rules nest (`.card { &:hover { … } .title { … } @media (…) { … } }`),
  and stylesheets take `@layer`, `@supports`, `@scope`, `@import`,
  `@starting-style` and `@property`, with `inherit`, `initial`, `unset`,
  `revert` and `revert-layer` on every property.
  Use CSS media conditions, for example
  `@media (width < 600px), (height < 600px) { ... }`.
  Supported features are `width`, `height`, `aspect-ratio`, `orientation`,
  `resolution`, `prefers-reduced-motion`, `prefers-color-scheme`,
  `prefers-contrast`, `prefers-reduced-transparency` (these follow the
  theme), and `pointer`/`any-pointer` (`coarse`, `fine`)
  and `hover`/`any-hover` (`hover`, `none`), which follow a touch screen.
  `color-gamut`, `dynamic-range`, `display-mode`, `update`, `scripting`,
  `forced-colors` and `color` answer for the play surface (`srgb`,
  `standard`, `fullscreen`, `fast`, `enabled`, `none`, 8). Use inclusive `min-`/`max-` bounds or
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
  container: that fails as a settlement cycle. `style(--tone: warm)` reads a
  container's custom property (unnamed: the parent's).
  `scroll-state(scrollable: top)`, `snapped`, `stuck` and `scrolled` need
  `container-type: scroll-state`. Container units (`cqw`, `cqh`, `cqi`,
  `cqb`, `cqmin`, `cqmax`) size against the nearest size container.
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
- Generate text with `::before`/`::after { content: … }`: strings,
  `attr(name)`, `counter(name, style)`, `counters(name, ".")` and quotes,
  with `counter-reset`/`counter-increment`/`counter-set` on any element and
  `@counter-style` for your own styles. On a box element each is a box
  of its own that takes every property a `<div>` does (`inline` by
  default); on a text element it is a run of the text and takes text
  properties. `dialog::backdrop` and `select::picker(select)` take a
  box's properties too.
  `li::marker { content: …; color: … }` and `list-style-image: url(name)`
  restyle list markers. `::first-letter`, `::first-line`, `::placeholder
  { color }` and `::selection { background-color; color; text-decoration;
  text-shadow }` style text.
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
  change how. An element with `view-transition-name: hero` on both screens
  morphs from its old box to its new one; style it with
  `::view-transition-group(hero)` (or `(*)`, or `(.card)` for a
  `view-transition-class`). Nest a group in an ancestor's with
  `view-transition-group: contain` on the ancestor (or `nearest`, or the
  ancestor's name, on the element): it then moves with the ancestor, and
  `::view-transition-group-children(card) { overflow: clip; }` keeps it
  inside. `::view-transition { background-image:
  url("ui/covers/stars.png"); }` covers with a texture instead of a colour.
- Compose with components and snippets, not code callbacks:
  `<Card title={item.name}>{#snippet body()}<p>{item.desc}</p>{/snippet}</Card>`,
  rendered in `Card` with `{@render body?.()}`. Pass content to a mount by
  reference: `scope.mount("pause", "pause", inputs, { side: "game#pauseSide" })`.
- Draw rings, bars, charts and badges with inline SVG, binding attributes
  to stores: `<svg viewBox="0 0 40 40"><circle cx="20" cy="20" r="18"
  fill="none" stroke="var(--accent)" pathLength="1" stroke-dasharray="1 1"
  stroke-dashoffset={1 - $ready}/></svg>`. Size the `<svg>` with CSS; its
  view box scales to fit, strokes with it. Gradients are
  `<linearGradient id>` and `fill="url(#id)"`. Icons reuse a `<symbol
  id="Coin" viewBox>` with `<use href="#Coin" x y width height/>`; cut with
  `clip-path="url(#id)"` (any shapes or paths) or a gradient
  `mask="url(#id)"`, hatch with a `<pattern>`, and name the graphic
  with `<title>{$label}</title>`. Label it with `<text x="50" y="20"
  text-anchor="middle" dominant-baseline="central"
  font-size="10">{$hp}</text>`: placed and scaled with the view box, with
  `<tspan font-weight="bold" fill="#f0b429">` runs inside.
- Drag and drop: `<li draggable="true" data-drag-type="gem"
  ondragstart={lift}>` and `<div data-accept="gem" ondrop={stash}>`; the
  event's value is `{ dragged, target, x, y }` (row keys and the point).
  `onpointerdown`, `onpointermove`, `onpointerup` and `onwheel` work on any
  element and bubble to the nearest handler.
- A 3D preview is `<viewport src={$target} camera-orbit={`${$yaw}deg 75deg 3m`}
  bind:clientWidth={$w} bind:clientHeight={$h} onpointermove={orbit}
  onpointerdown={pick}>`: size the render target from `$w`×`$h` times the
  pixel ratio; the events carry `ray` for picking.
- Touch controls feed og-input actions directly: `<joystick action="move">`,
  `<dpad action="move">` and `<button data-og-touch="jump">` (a document
  button CSS sizes and places). Game code reads `og.input.action("move")` as for
  keys and pads; no handler is needed. Size them with CSS `width`, place
  them with `position: absolute` and insets, and hide them with
  `@media (pointer: fine) { #Stick { display: none } }`. A stick's dead zone is `data-dead-zone="0.2"`.
- Name a box over an entity with `<anchor entity={u.entity} offset="0 2 0">`,
  and put a working screen on a mesh with `<panel id="Terminal" world
  entity={$console} size="3 2" src={$screen}>`: the game shows the render
  target `src` on a plane `size` world units across, and pointer rays
  that meet it press its buttons. Lay a selection ring under a unit with
  `<ground entity={u.entity} size="2 2"><svg viewBox="0 0 40 40">…</svg></ground>`:
  drawn flat on the ground in perspective, inert. For many at once (damage
  numbers, overhead bars) write `<anchor entity={h.entity} mass="damage"
  value={h.amount}>` in a keyed each: with a CSS animation (`rise 1s`) each row
  spawns one number that plays `@keyframes rise`; with `max="100"` and no
  animation it is a bar that stays with its row.
- Celebrate with CSS: `@keyframes land { 60% { -og-burst: 24 48px #ffd166
  600ms; -og-sound: success; } }`. Sparkle a rare item with `.rare {
  -og-emitter: sparkles; }` (presets: sparkles, confetti, embers, snow,
  bubbles, dust, sparks, fireworks), throw confetti once with `-og-emitter:
  confetti 70 burst 110 for 1.4s`, or set rate, shape, size, `life`, speed
  (`120px/s`) and colours yourself. The `celebration` component is confetti,
  a burst, a level-up flourish or fireworks. Flash damage with `#Hurt {
  -og-meta: vignette #a00; opacity: var(--hit); }`.
  drawn flat on the ground in perspective, inert.
- Turn a model on a turntable with `<viewport src={$target} camera-controls
  camera-orbit="30deg 75deg 3m" max-camera-orbit="auto 100deg 6m"
  bind:cameraOrbit={$orbit}>`: a drag orbits it and a fling keeps it
  turning, the wheel and a pinch zoom it, the arrows turn it once focused.
  Any element takes `onpinch={zoom}` (its event has `scale` and the centre
  `x`, `y`); the `node-graph` component zooms by it.
- A carousel snaps with `scroll-snap-type: x mandatory` on its box and
  `scroll-snap-align: center` on each card. A bottom sheet is
  `<dialog modal bind:open={$bag}>` with `-og-sheet-detents: 40% 90%`;
  `-og-sheet-edge: inline-end` makes it a side sheet (the right, the left
  right to left) whose detents are widths.
- Paint with standard CSS: `background: linear-gradient(160deg,
  var(--panel), #0000), #123`, `mask-image: linear-gradient(#000 70%,
  transparent)`, `clip-path: polygon(50% 0%, 100% 50%, 50% 100%, 0% 50%)`,
  `backdrop-filter: blur(12px)`, `filter: grayscale(1)` for a locked item,
  `text-shadow: 0 2px 6px var(--accent)`. A shader look is
  `-og-material: shine(-0.5 0.2 0.6 1)`.
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
  dense HUD readout may go to 12 px if its group is marked dense
  (og-ui's `Props::dense`) and it still grows with the player's text scale.
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

## 7. Component documents

Health bars, ability bars and cooldowns, a round countdown timer, a minimap
and compass, a radial weapon wheel chosen with the stick, nameplates and
damage numbers are component documents, not `og.ui` builders: copy one with `og ui eject <name>` (`--list` names them
with the screens), load it with `og.load.document`, and use it in a screen
by its declared name with props from named stores. Each declares its props,
actions and a `fixture:default`, so `og ui preview` shows it at once.

- Bars and meters: `stat-bar` (a health or resource bar with a ghost and a
  shield), `orb-meter`, `radial-meter` (a stamina wheel), `progress-bar`,
  `progress-ring`, `skeleton-group` (a loading placeholder).
- Abilities: `ability-bar`, `ability-slot` and `cooldown` (a cooldown
  sweep over the icon, the seconds left, charges and cost, the hotkey as
  `<kbd data-action>`), `status-grid` (buffs and debuffs with their time).
- Time and score: `hud-timer` (a round countdown that turns warn and
  critical), `star-bar` (a score bar with star thresholds), `celebration`
  (confetti, a burst, a level-up flourish or fireworks, played again by a
  new `shots` id). A number that
  counts is `<p count={$score}>` with `-og-count-duration`.
- Navigation: `minimap` and `minimap-view` (a map in the corner with
  markers for allies, enemies, objectives and pings, a uv region by
  `object-view-box: inset(…)`), `compass` (a heading strip with points of interest).
- Menus and layout: `radial-menu` (a radial weapon wheel whose slice is
  chosen with the right stick or the pointer: SVG wedges, the aim from its
  events), `split-view` (two panes and a dragged
  divider), `carousel` (`scroll-snap-type`), `breadcrumbs`
  (`<nav><ol>` with `aria-current`), `chart` (bars and a line),
  `node-graph` (a skill tree or run map: nodes over SVG links, zoomed and
  panned from its wheel, pinch and pointer events).
- Buttons: `hero-button` (the main call to action with its keycap),
  `action-button` (a touch button feeding an og-input action through
  `data-og-touch`), `hold-button` (hold to confirm, filling as it is held),
  `action-hints` (the action bar: each action's `<kbd data-action>` glyph
  and hint), `ribbon` (a corner ribbon on a card).
- World: `nameplate`, `waypoint`, `loot-label`, `speech-bubble` and
  `interaction-prompt` are `<anchor>`s over an entity or point;
  `selection-ring`, `telegraph`, `path-line` and `placement-ghost` are
  `<ground>` marks; `overhead-bars` and `damage-numbers` are
  `<anchor mass>` rows drawn on the GPU by the thousand.

Expressions compute no geometry: the game passes a chart's line and a path
line's route as SVG path data, a radial menu's slice positions and a split
view's ratio.
