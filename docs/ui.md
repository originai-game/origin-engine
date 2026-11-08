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

`<field>` takes the HTML names `type`, `inputmode`, and `autocomplete`;
`FieldDesc` carries the same three for TypeScript on both hosts. The types
are `text`, `password`, `number`, `email`, `url`, `tel`, and `search`.
Validity follows the HTML Living Standard: a valid floating-point number,
its email expression, and an absolute URL with a host for special schemes.
A nonempty text that fails its type matches `:invalid` and dresses the
invalid frame. An empty field is never a mismatch. A number field types
only digits, a sign, a point, and an exponent.

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

Media rules use standard conditions, such as
`@media (width < 600px), (height < 600px)`. Supported features are `width`,
`height`, `aspect-ratio`, `orientation`, and `prefers-reduced-motion`, with
inclusive `min-`/`max-` bounds and range comparisons. Commas and `or` select
alternatives, `and` intersects, and `not` negates; mixed operators require
parentheses. Nested media rules intersect. There is no named-breakpoint
reader. Container-query runtime support remains pending.

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
Scoped `@keyframes` can emit a style sound at a frame, such as
`-og-sound: error` for game-over entrance feedback. The event records its
playback rate; rate-valued fields use `playbackRate`, not `pitch`.

CPU motion, GPU mass elements, and camera timing share `Ease`: GSAP names
such as `none`, `power1.out`, and `back.out`, plus `cubic-bezier(...)` and
`steps(...)`. Style tokens retain `EaseRole` indirection. CSS timing
keywords belong to the CSS parser; shared timing does not make them GSAP
names. Lexical custom properties and the cascade above are implemented.
Standard CSS completion remains pending for container queries.
The supported subset in the component catalogue is not full CSS support.

Core's native settlement hook publishes measured container content boxes
before effects, hit testing, and paint. First measurements include zero
dimensions; unchanged measurements do not notify again. Changed query
styles relayout their roots and dependent floating placement in the same
frame, with safe-area geometry included. Repeated geometry states produce
an error rather than publish an intermediate frame. This hook is landed;
the CSS query adapter is still pending. Retained rendering limits and
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
unmeasured performance budget has passed. Colour-blind
modes, live regions, and font packs require explicit core delivery before
authors can claim support. Author knowledge belongs in guides, not shipped
skills. No kit silently implements those missing facilities itself.

The [gate rules](ci.md) own test lanes, locks, and toolchain policy.
`scripts/land.sh` owns landing on main. Release and platform installation
follow [release rules](release.md), with separate release authorization.
