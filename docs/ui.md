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
There is no script block, JSX, or runes. The document-format stream owns
the exact grammar and metadata specification; it is pending review.
Structure and slot content remain documents, never TypeScript trees.
Keys identify data items, not row indices. Every visible string must be
translatable, and parse errors identify file, line, column, and a fix.

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
resources, Suspense, or transitions.

Each screen has one fixed-tick scope with a stable name, named stores and
keyed collections. Documents mount once with named mount IDs and inputs.
Template `$name` reads and keyed `{#each}` consume those inputs. Bindings
are pure, track only the values they read, and replace dependencies when a
branch changes. Lists bind to keyed collections, not copied viewmodels.

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
Inspection and accessibility materialize lazily, with incremental updates
for active subscribers. Errors are values that identify the scope and ID,
including duplicate keys, cycles, undeclared reads, and reentrant writes.
Hot reload re-registers callbacks by ID while sources and mounts survive.

This authoring contract is approved. Public signatures remain pending the
core stream's API landing and confirmation; no proposed call spelling is
yet frozen here. Incremental scheduling follows the API, and measured
update costs are reported separately. The current
[components reference](../crates/og-api/guides/components.md) describes the
implemented document dialect; the
[screens guide](../crates/og-api/guides/screens.md) describes its workflow.

Fixtures contain supplied presentation states, not playable economies,
combat loops, matchmaking, or rhythm clocks. `og create` writes one empty
project skeleton with creation constraints. It never installs a genre
fixture as a gameplay template.

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
content and behaviour. Selectors are scoped to the document, ordered by
specificity then source order, and never affect unrelated mounts.
Element, ID, class, descendant, child, attribute, and supported control
pseudo-class selectors use the same reflected properties. Named
breakpoints use `@media phone`; safe-area values use the supported CSS
`env(safe-area-inset-*)` spellings. Svelte `class:` directives select
states; they do not introduce a second styling language.

Core parts keep their existing names and meanings. Each kit documents its
additional parts with its own API, so packs can dress them without private
imports. A missing custom dressing uses the readable core token fallback.
It is not permission to leave a required state unfinished.

Hover, pressed, focus-visible, selected, disabled, and selected+disabled
are distinct requirements. A selected item can also have focus; selection
must not hide the focus indicator. Disabled controls emit no activation
intent. State information cannot depend on colour alone.

Button glyphs and input prompts follow the active device and switch live:
keyboard and mouse, Xbox, PlayStation, Switch, generic gamepad, and touch.
Core input glyphs and UI action prompts own this behaviour; kits do not
hard-code button letters or assume that every gamepad uses Xbox labels.

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
Border-radius token expressions, variable-height virtualization, and
localized number/date formatting needed by kits are core prerequisites,
not invitations to private kit workarounds. Per-frame UI cost and its
performance tests are independent measured work; this document claims no
unmeasured performance budget has passed. Password fields, colour-blind
modes, live regions, and font packs require explicit core delivery before
authors can claim support. Author knowledge belongs in guides, not shipped
skills. No kit silently implements those missing facilities itself.

The [gate rules](ci.md) own test lanes, locks, and toolchain policy.
`scripts/land.sh` owns landing on main. Release and platform installation
follow [release rules](release.md), with separate release authorization.
