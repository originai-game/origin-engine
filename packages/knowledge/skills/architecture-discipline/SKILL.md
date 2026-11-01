---
name: architecture-discipline
description: How an Origin Engine project keeps its shape as it grows — the layers (engine, components, systems, scenes, the game entry, playtests), when and where to split a file, why a line ceiling is a road sign, building the whole game in a working order that keeps it running, every visible subject an asset from its pipeline, failing loudly instead of stubbing, and keeping every tunable number in data. Use when starting a project's code, when a file or system grows a second responsibility, or when reviewing a project's structure.
---

Two failure modes govern a long project's codebase. A scaffold full of
example content invites filling in blanks instead of building systems. A
scaffold with no shape invites everything into one file. The answer to both
is shape without implementation: the layers below, kept from the first line.

The engine's rules for the code inside each layer are og-core's; the
project's files and the game entry are og-game §2 and §3.

## 1. The layers

An Origin Engine project keeps the same shape at every size:

- The engine is the instrument layer: the fixed tick, the seeded `Rng`, the
  probe, the command channel, loading and rendering. None of it lives in the
  project and none of it is reimplemented there; a missing instrument is an
  engine feature to ask for, not a module to write (og-core §1).
- `src/components.ts` (or `src/components/`): every component and resource,
  declared once with its fields, ranges and `tunable` flags (og-core §5).
- `src/systems.ts` (or `src/systems/`, one file per responsibility): the
  rules, one system per responsibility: input, movement, scoring, camera,
  audio, the HUD's state.
- `assets/scenes/*.ogt`: what exists at the start, entities by stable name
  with their component values, the tuning resource included (og-game §4).
- `src/main.ts`: assembly only, `og.game({...})` with the scenes, the
  systems in order and the registered commands. No game rules, ever.
- `tests/*.og.ts`: the playtests `og test .` runs (og-build §3).

Systems never reach another system's module state; they share data through
components and resources. A dependency that wants to run the other way is
telling you the logic is in the wrong layer: move the logic, not the
import.

## 2. When and where to split

Past about 400 lines, split. When a project's tooling enforces a ceiling,
the ceiling is a road sign, not an adversary: do not raise it and do not
add an exemption.

Split along the seam the file already has: a file that grew a second
responsibility usually shows it as a cluster of functions that share no
state with the rest. If no seam is visible, the file is one thing that is
genuinely large, which almost always means logic that belongs to a system
has been inlined into an entity, or a data table has been embedded in code.
Move the table to data (a scene, an `.ogt` asset, a string table) and the
logic to its system.

Splitting for the line count alone, into `foo-part-2.ts`, satisfies nothing
and costs the next reader the seam that used to be obvious. If that is the
only split available, the real problem is upstream.

## 3. The whole game, in a working order

What is made is the whole game as designed, at the quality it launches
with: there is no first version, slice or prototype to show first and
finish later. The order of work is a working order, not a version: it
keeps the build running at every step, and everything it builds is final.

Build and run continuously. Start from the route from boot to play, real
input and the player or piece; input is always present, because people and
the probe both play through it. As soon as it builds, prove that scripted
input moves the player and the tick advances (`og run . --headless
--commands`, og-build §1), and keep proving it as the game grows. Write each
system the design names and register it in the game entry together with its
implementation, never before it and never as a silent no-op (§4); rerun the
earlier playtests each time one is added.

Every visible subject is an asset made through its pipeline before gameplay
code uses it: a `type model` (og-model), a glTF, sprites, each the cook
conforms to the look (og-taste). Never primitive shapes placed from game
code, not even for a moment: what the game code places is the asset the
game launches with.

## 4. Loud failure

A system that is not written yet is not registered, and never registered as
a silent no-op: a hole visible in the game entry and in the plan cannot be
shipped around, while a stub that pretends to work hides it. Engine calls
that can fail return a `Result` with a reason: handle it, never swallow it
(og-core §3).

Which systems exist is the design's decision, written once; a design note
in the project derives from it and never competes with it. A design with
no rooms has no rooms system. Input always remains, for the player and for
probe playback.

## 5. Tuning

Every tunable number is a `tunable` field of a component or resource (its
default where it is declared) or a value of the scene that sets it. A magic
number inline in a system breaks live tuning: the Tune panel and the
probe's `bounds` show the field while the game obeys the inline number, and
whoever is tuning is told a lie about the game they are playing. Move a
number into a field the moment someone may want to change it, and write a
tuned value back to the scene with `save_scene` (og-game §9).
