# r4/author-loop

Branch `r4/author-loop` from `origin/main` at 1bb341e5 (0.4.0). The
findings the dogfood games left open that are the engine's or the
knowledge's. Each was checked on main before any change; three were
already fixed there.

| Item | Source | State | Commit | Test |
|---|---|---|---|---|
| 1 | brasswick #10 (level select has no pointer exit) | already fixed on main | 80008fbb | `packages/engine/tests/ui-levels.test.ts`: Back button and scrim tap answer `closed` (5 pass here) |
| 1 | hollowbell #10 (cutscene takes no focus) | already fixed on main | 3a78409c | `crates/og-ui/tests/widgets.rs` `a_dialogue_or_a_dialog_that_appears_takes_the_focus_so_enter_answers_it` (ok here) |
| 1 | kestrel-stair #14 (mainMenu takes no focus) | already fixed on main | da50fea3 | `crates/og-ui/tests/widgets.rs` `a_menu_that_appears_takes_the_focus_and_a_move_from_nothing_lands_on_its_first_item` (ok here) |
| 2 | rumpus-isle #1 (2D look for a 3D premise) | fixed here | fa9ead9b | `packages/create/tests/create.test.ts` "the premise's style is one drawn for the project's profile"; `packages/knowledge/tests/validate.test.ts` refuses a bad `look.profiles` |
| 3 | rumpus-isle #2 (no `Font()` / `Style()`) | fixed here | 5a8ca6a4 | `packages/engine/tests/api-surface.test.ts` "every handle type has a field factory of its own kind, and a handle round-trips through it" |
| 4 | rumpus-isle #3, #4, #5, #6, #8, #11 (docs) | fixed here | 5bc4e92d, 04e09288 | knowledge validator (names resolve); claims checked by runs, below |
| 4b | module action names (follow-up decision) | fixed here | 4af7a193 | og-runtime `engine::tests::a_game_command_named_as_a_module_action_is_refused`, `engine::tests::the_module_actions_are_the_reserved_list` |
| 5 | rumpus-isle #13 (polish wants floating text on every hit) | fixed here | 9a870fa3 | `packages/engine/tests/polish.test.ts` "an impact with three of the four signals owes no fourth" |
| 6 | r3 terrain-generate (one-line `layer { ... }`) | the terrain example was already fixed on main (7a2c1498); another one-line example fixed here, and the validator now checks | ab8d92c0 | `packages/knowledge/tests/grammar.test.ts` "every .ogt example in the skills has the parser's line shape" |
| 1 | brasswick #11, #12, #15, #16 (follow-up) | already fixed on main | 80008fbb (#15 with 41f49c78) | og-ui `groups_and_icons_take_the_names_their_descs_give`, `a_corner_group_steps_down_past_the_control_when_moving_along_lands_on_another`; `packages/engine/tests/ui-levels.test.ts`, `ui-hud-corner.test.ts` (10 pass here); og-platform-sdk §3's system order for #16 |
| 7 | r3 model-review (og-build lacks `og compare`; og-model lacks `type model`) | already fixed on main | 3da5b8a5 | og-build §4 names the views, the turntable and `og compare`; og-model §5 has "The same model as an asset: `type model`" |

## What changed

**2. Looks by profile.** No catalogue field said whether a look draws for 2D
or 3D (only `assets.faceBudget`'s prose, "none (2D)"). Every style now has
`look.profiles`: `["2d"]` for the twelve flipbook, hand-pixel and cut-out
styles, `["3d"]` for the fifteen of lit meshes, toon materials and voxels,
both for the ten post treatments that frame either. The validator refuses a
style without one, or a profile other than `2d` and `3d`. `pickStyle` takes
the project's profile (3D unless `2d`; create passes the template's actual
profile, so an archetype's counts) and never offers a style outside it.
`--look` still names any style. The Rumpus premise matched no 3D style's
genre, only a descriptor word ("screen") of `muted-cinematic-3d`, a bleak
horror style, so `sculpted-clay` gained "party games" in its genre and
"playful" in its mood: the premise now picks `sculpted-clay` (score 3) for 3D
and `rubber-hose-animation` for a 2D game. og-taste §12 and the create
README say how profiles are read. The old pick fails the new test.

**3. Handle factories.** The IDL had five handle types without a column:
`Font`, `Style`, `Look`, `Shader` and `Rig`. Each gains one in
`crates/og-api/api.ogt` (Uint32Array, stride 2, like every handle), a
runtime export in `packages/engine/src/index.ts`, and og-runtime's codec
reads it as a 64-bit handle on both hosts. The declarations, bindings,
schema, host tables and wiki are `og api gen` outputs (`og api check`
passes). The test reads every `x-og-kind: handle` of the schema, so a handle
added later without a factory fails it; it writes a distinct value past 2^32
to each handle field (a live entity for `Entity`) and reads it back, and does
the same through `list(Font(), 2)`. og-core's handle rule and api.ogt's
column doc name `Font()` and `Style()`. CHANGELOG `Unreleased` has the line.

**5. og polish, per impact.** The table compared per-signal totals with the
moment count, so every moment lacking floating text was listed even where
three other signals held it. Now each damage moment counts its four signals
(hit stop, shake, floating text, a sound); a moment with three is whole and
owes nothing; a moment with fewer is asked for each signal it lacks. The
score is each moment's min(3, signals) of three, averaged, so one moment's
signal no longer covers another's lack (the topdown archetype still scores
100). `damage` gains `whole` and `credit`. The new test plays a bump (hit
stop, shake, floating text) and a fall (shake only): the old code reports
"2 of 2 ... no a sound"; the new one reports the fall's three missing
signals, "1 of 2" each, and scores 67. og-feel §7 and og-build §5 state the
rule.

**6. .ogt examples.** The terrain example the r3 notes named had been
rewritten on main (7a2c1498). A scan of every code block in the skills
found one more, og-game §24's baked-lighting scene (`Transform { ... }` and
`LightVolume { ... }` on one line), now written out. og-game §5 says the
prose's inline `texture { kind pixel }` is shorthand and shows the file
form. The validator's new `ogtShape` check reads every ```text or bare
block as og-core's parser does (`crates/og-core/src/ogt.rs`): a `{` ends its
line, a `}` stands alone, blocks close, a file that opens with `type` has
`version` and `---` next. It mirrors the parser's line rules in TypeScript
because the validator also runs in the published package, where there is no
parser to call. It does not cook the examples: references such as
`models/hall.glb` are not checked.

**4. Docs.** Each claim was checked against the code or a run on a scratch
project (dev `og`, `OG_RENDERER=none`):

- #4: `CORE_ACTIONS` in `crates/og-probe/src/command.rs`, 31 names, listed
  in og-game §3. A game command named `pause` stops the game at start
  ("command 'pause': 'pause' is a core action; register under another
  name"). A game command named after a module's registered action (`view`)
  is **not** refused: the game's handler replaces the engine's
  (`help` shows `view (registered)` with the game's empty doc). The
  coordinator decided to refuse it; see "Module action names" below.
- #5: `command::parse` splits fields on `:` and `\` makes the next character
  literal; `0.1:records:12:34.5` set `best` 12 and `seconds` 34.5.
- #6: `Probe::emit` suffixes a named clash at any time with a WARN; the run
  wrote `rec-2.json` and the WARN quoted in og-build §1.
- #8: `get:box.og.Transform` wrote `"position":[1.0,2.5,-3.0]`;
  `set:box.og.Transform.position=4,5,6` then `get` read `[4.0,5.0,6.0]`.
- #11: game code reads `f32(2.7)` back as `2.700000047683716`
  (`JSON.stringify` gives the same); `get` and dumps print `2.7`. The docs
  say both.
- #3: the pattern is Rumpus's own `Tags` system (`src/feel.ts`), with
  `screen` and `layer` as TextDesc documents them.

**Module action names (follow-up, 4af7a193).** The game's commands were
registered first (`rprobe::install`), then the modules' with `let _ =`, so a
module's registration of a name the game had taken failed silently and the
game's handler ran. Reordering alone would not hold in a publish build,
which registers fewer module actions (no dumps, `view`, `cover`). So
og-runtime now reserves the names: `probe::MODULE_ACTIONS` (18 names) and
`probe::reserved(name)`. `register_one` (the game's `GameDesc.commands` at
start, a hot reload's new ones) and `og_commands_register`
(`og.commands.register`) refuse a reserved name with "'view' is a
registered engine action; register under another name", beside the core
message. Both hosts run this code (og-runtime is shared by the addon and the
wasm). The check is by name, so the order the modules and the game register
in does not matter. The test tries both orders and checks that a module
registered after a refused game command keeps its own handler. The modules'
registration is one function, `engine::register_module_actions`; a second
test holds it to `MODULE_ACTIONS`, so a module that adds an action without
reserving it fails. A scratch game with `commands: { view }` now stops at
start: "command 'view': 'view' is a registered engine action; register
under another name". No template, example or test registers one of the 18
names. og-game §3 lists them as refused.

**Brasswick #11, #12, #15, #16 (follow-up).** Each checked against the code
and its tests: #11 `IconDesc`/`ImageDesc.label` and the level card's names;
#12 `SettingsDesc.sectionLabel`/`languagesLabel` and the group labels; #15
anchored groups keep out of `Viewport.covered` (41f49c78) and a corner group
steps down past it (80008fbb), and Brasswick's HUD is anchored groups, so
it gets this; #16 og-platform-sdk §3's system order. All four rows are
marked fixed. Brasswick's own `tests/locales.og.ts` asserts the old English
names (#11's repro), so that suite needs updating with the game; I did not
run it.

## Verification

Run one at a time, as AGENTS.md asks, with the dev `og` and addon copied
into `packages/engine/native/` (`cargo build -p og-host-node -p og-cli`):

- `cargo test -p og-api`, `cargo test -p og-runtime --lib` (17 pass),
  `cargo fmt --check -p og-runtime`, `og api check`.
- `bun run validate` and `bun test tests/` in packages/knowledge (49 pass).
- `bun run test` in packages/create (16 pass).
- In packages/engine: `bun run typecheck`, `tests/api-surface.test.ts`
  (5 pass), `tests/polish.test.ts` (8 pass), `tests/ui-levels.test.ts`
  (5 pass).
- `cargo test -p og-ui --test widgets` (27 pass) and `--test a11y` (6 pass),
  for item 1's rows.
- Follow-up: `cargo test -p og-runtime --lib` (19 pass),
  `cargo clippy -p og-runtime --all-targets -- -D warnings`, `cargo fmt`;
  packages/engine `tests/api-surface.test.ts`, `tests/runtime.test.ts`
  (18 pass), `tests/ui-hud-corner.test.ts` (5 pass); `bun run validate` in
  packages/knowledge.

The new tests fail on the old code: the create test picks
`rubber-hose-animation`, the api-surface test finds five handles without
factories, the polish test sees "2 of 2" for the sound, and the grammar
test's one-line `LightVolume` is reported.

Not run: `scripts/check.sh`, `og test` on whole projects, web-reference,
the wasm build, reference captures, the full packages/engine suite.

## For the coordinator

- **Metal reference frames:** none of these changes should move one. The
  level select, the focus rows and the terrain example were already fixed
  on main. Nothing here touches rendering, the UI kit, a template's look or
  a game's code. The catalogue's new field and `sculpted-clay`'s premise
  words change only what `create --premise` picks for a new scaffold.
- **Brasswick's `tests/locales.og.ts`** still asserts the English icon
  names from before 80008fbb. It belongs to the game and was not run here.
- **Module action names:** a game whose command is one of the 18 names now
  fails at start. None in the repository does; a game outside it would
  need a rename.
- **Rumpus-isle's workaround** (#2: the font as an `f64` cast back) can now
  be a `Font()` field; the game was not edited.
