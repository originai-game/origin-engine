---
name: recipe-craft
description: Working with reference and example code in an Origin Engine game — the retrieval ladder (the engine first, then verified example modules such as the engine's archetypes, then the pinned reference, then the open web), the vendor-not-depend discipline for adopting a module into a game, proving a vendored module is integrated, and distilling proven game code into a reusable module. Use when a mechanic, effect or UI pattern might already exist somewhere, before copying code into a game, or when extracting a mechanic for reuse.
---

Reference code has a retrieval ladder and a custody rule. This skill keeps
found code from silently becoming load-bearing code.

Research about games (genre conventions, norms, formulas) is not code and is
never a rung of this ladder; finished art (models, textures, sounds) is an
asset decision with its own licence and provenance, recorded beside the
file in `assets/`.

## 1. The retrieval ladder

When a mechanic, effect or UI pattern is needed, look in this order:

1. **The engine itself.** Origin Engine owns the world, the tick,
   rendering, animation, physics, navigation, audio, UI, loading and the
   probe. Look in og-game's `reference/index.md` (generated from the IDL of
   the engine the game pins, so it is exact) and in og-game §1's table of
   skills before writing any of them: a hand-integrated capsule, a
   home-grown A* or a second renderer is fresh authorship the engine
   already made unnecessary, and it runs outside the engine's determinism
   and performance guarantees. A missing engine feature is asked for as
   one, not rebuilt in the game (og-core §1).
2. **Verified example modules.** The engine's archetypes (`runner`,
   `match3`, `topdown`, `tower-defense`, `roguelike`, `idle`, `rhythm`,
   `third-person`, `arena`: whole games `create-game --archetype` writes,
   each with its playtests) and the engine's templates are code that
   passed the engine's own gates against the version that ships them, as
   is any registry of verified modules your environment offers. Only a
   module whose dependencies are nothing but `@origingame/engine` (or
   nothing at all, for plain-data logic) fits an Origin Engine game; one
   that needs a renderer package or a browser API is reference only.
3. **The pinned reference.** The engine's API pages are og-game's
   `reference/`, at the version in `project.ogt`; never trust a recalled
   signature over them. Engine vocabulary follows three.js's naming
   (`position`, `quaternion`, `metalness`) but nothing else of three.js
   applies: its examples are not this API (`guides/three-to-og.md` beside
   og-game maps one onto the other).
4. **The open web**, last, with the usual scepticism about version drift.

## 2. Vendor, don't depend

Adopting a verified module means **copying its files into the game** under
`src/registry/<category>/<name>/`, beside a `registry.provenance.json` the
game writes
from what it copied: where it came from (the archetype or registry and the
module's id), its version (the engine version that shipped it, or the
registry's content digest; never invented), what it requires and the list
of files. Then the game owns the code fully:

- Confirm the module's pins match the game's (`@origingame/engine` at the
  version `project.ogt` pins) before vendoring, and adapt the code, not the
  game's pins, when they differ.
- The module imports only its own files and the packages it declares. Two
  modules that need each other are composed in the game's own wiring (its
  systems and game entry), never by one importing the other.
- Vendored code is game source from the moment it lands: it obeys og-core
  (no wall clock, no `Math.random()`, state in components and resources),
  type-checks with the game and is edited freely afterwards. Its constants
  are its defaults; the game binds them to its own `tunable` fields
  (architecture-discipline §5). There is no update channel and no run-time
  fetch: a newer version is adopted by deliberate re-vendoring, never
  silently.
- Everything below rung 2 is reference only: read it in a scratch
  directory, never copy it into the game. The game's version is written
  fresh against the game's conventions.

## 3. Vendoring is not integration

Name the module that imports a vendored one, and who owns it, before
adopting it. Show the registration in the game entry and the behaviour live
with `og run` or `og test .`, including the transition into and out of it
in the game's flow. Files that are never imported or called are not
delivered systems, and a large inventory of modules says nothing about the
parts of the game nobody played. Record the source digest, the adapted
version, the game's revision and the run that proved it together.

## 4. Distilling back

A mechanic that proved itself in a real game is worth extracting into a
reusable module. Distilling is deliberate authorship, not an export:

- **Evidence first**: the code passed the game's playtests and shipped in a
  playable build. Hollow or speculative modules are not extracted.
- **Inject its dependencies**: no imports from game internals and no global
  state; its only peer the engine at an exact version; its tunables typed
  parameters with their defaults and ranges.
- Give it a manifest (id, title, description, tags, what it requires, its
  tunables, its files in exact two-way closure, the source of the idea
  where it has one), its source files, and a README stating what it does
  and the evidence it was verified with.
- Publish it only when its owner asks, following the destination's own
  contribution and verification rules; extracting a module never adds a
  release process to the game's own production.
