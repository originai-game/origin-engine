# @origingame/create-game

Scaffolds an Origin Engine game from a project template of the engine
repository, the 3D template `templates/game` or the 2D starter
`templates/game-2d`: `project.ogt`, `src/`, `assets/`, `tests/`,
`package.json` and `tsconfig.json`, with the game's name and id filled in.

```sh
og new my-game --name "My Game"                              # the og launcher: the latest release's create
bun packages/create/bin/create.ts my-game --name "My Game"   # in the engine repository
cd my-game && og cook && og run .
```

`create [dir] [--name "Name"] [--id slug] [--profile 2d|3d] [--archetype name] [--premise "one line"] [--look slug] [--force]`: the
id defaults to the name in lowercase-hyphen form (or the directory's name);
a directory that is not empty is refused unless `--force` is given.
`--profile 2d` writes the 2D starter (a courier's platformer: sprites,
flipbooks, a tilemap with one-way ledges, 2D physics, a HUD and a pause
menu, one playtest), `--profile 3d` the 3D template; without the flag, a
premise that says 2D, platformer, side-scroller, metroidvania, pixel art,
sprites or tiles without saying 3D gets the 2D starter, anything else the
3D template. The output's `Profile:` line says which and why.
`--archetype` starts from a complete game of `templates/archetypes/`
instead of either template: `runner`, `match3`, `topdown`, `tower-defense`,
`roguelike`, `idle`, `rhythm`, `third-person` or `arena` (the published package carries them under
`archetypes/`); an archetype has its own profile, so `--profile` beside it
is refused.

Every game gets its look: `assets/looks/<id>.ogt`, an `.ogt` of `type look`
(og-taste §12 of the knowledge package) that `GameDesc.look` in the entry
names, applied before `start` runs. It `extends` a preset of the style
catalogue (`packages/knowledge/looks/catalogue.json`, 37 styles):

- `--look <slug>` names it (`anime-cel-3d`, `16bit-pixel`, …; a slug with
  no preset is refused, listing those that have one);
- `--premise "<one line>"` picks the style whose premise fits best (a word
  of the premise in a style's genre counts 3, in its mood 2, in its era,
  audience, name or descriptor 1; a tie goes to a style the engine draws
  whole, then to catalogue order), among the styles whose `look.profiles`
  include the game's profile: a 3D game never gets a flipbook-only style,
  a 2D one never a style of lit meshes (`--look` names any). On a
  terminal, with neither flag, create asks for the premise (`--no-prompt`
  does not);
- otherwise the template's own look stays: the plain template extends
  `faceted-low-poly`, the idle archetype carries its UI style and render
  settings, and the other archetypes get a look that extends nothing (their
  frames stay as their `references/` recorded them).

The template's look file is renamed to the game's id, the blocks it sets
are kept under the chosen preset (they win field by field), and an entry
that names no look gets `look: "looks/<id>.ogt"` after `export default
og.game({`. The game's README gains a "The look" section: where the file
is, what it extends and how to change it. For an archetype, a preset moves
the frames its `references/` hold; create says so and how to record them
again.

## Creation choices

`og new game --choices --premise "a warm phone platformer"` resolves a
complete selection and writes `project.ogt`'s `create` block. Any new
choice flag also enables this path. On a terminal, missing fields are
confirmed with their inferred defaults. `--no-prompt` accepts those defaults.
Legacy noninteractive invocations and `create()` without `choices` retain
their previous behavior.

The flags are `--genre`, `--view`, `--look`, `--mood`, `--ui`, `--platform`,
`--input`, `--players`, `--lang en,zh`, and `--music`. Their values live in
`bin/choices.ts`, except look slugs, which retain the existing catalogue.
There are currently 14 installable UI packs; `cjk` and `shared` are asset
directories, not UI styles. `ui` defaults to the selected look's UI pack.

The additive exports are `CREATION_CHOICES`, `CreationChoices`,
`CreationInput`, `creationSchema(lookSlugs)`, `parseCreationInput(value)`,
`inferCreationChoices(premise, input?, defaults?)`,
`resolveCreationChoices(premise, input?, profile?, catalogue?)`, and
`creationBlock(choices)`, and with the mapping `applyCreationChoices(dir,
selection, options?)`, `viewProfile`, `VIEW_PRESETS`, `GENRE_PLAYERS` and
`genrePlayersRefused`. `creationSchema` returns JSON Schema 2020-12.
`inferCreationChoices` applies deterministic English/Chinese keyword rules;
`resolveCreationChoices` also matches and validates the catalogue look.
Explicit fields win. Languages default to Japanese for kana, Chinese for
Han text, or English otherwise. Music defaults to `ambient`.

`create({ dir, premise, choices: {...} })` returns the resolved selection
as `Created.choices`. It installs the chosen UI pack, writes orientation
and DPR limits under `publish`, and generates `src/creation.ts` with
quality and controls. Phone quality starts at `low`, other targets at
`medium`.

### What each choice starts

`bin/starters.ts` is the mapping, as data beside the vocabulary: each
genre's starter per view (and why a view has none), each view's camera
preset, and each starter's controls, UI kit, engine systems and player
counts (with the reason for any it refuses). The view picks the profile:
`side-2d` and `top-down-2d` are 2D (the platform's overlay
`templates/game-2d`), `isometric-2.5d`, `third-person-3d` and
`first-person-3d` 3D (`templates/game`).

| Genre | `side-2d` | `top-down-2d` | `isometric-2.5d` | `third-person-3d` | `first-person-3d` |
|---|---|---|---|---|---|
| action-platformer | `platformer` | `topdown` | — | `third-person` | — |
| shooter | — | — | `shooter` | `shooter` | `shooter` |
| rpg | — | — | `rpg` | `rpg` | `rpg` |
| puzzle-match | `match3` | `match3` | — | — | — |
| strategy-tower-defense | — | `tower-defense` | — | — | — |
| racing | — | — | `racing` | `racing` | `racing` |
| runner | `runner` | — | — | — | — |
| roguelike-card | `deckbuilder` | `roguelike` | — | — | — |
| idle | `idle` | `idle` | — | — | — |
| rhythm | `rhythm` | `rhythm` | — | — | — |
| arena | — | `arena` | `shooter` | `shooter` | `shooter` |
| open-world | — | — | `open-world` | `open-world` | `open-world` |

A dash is a pair create refuses, with the genre's reason (`GENRES[g].refuseView`).
`platformer` is `templates/game-2d` (side-on rooftops, a top-down
courtyard), `game` the 3D overlay `templates/game` in any 3D view; the
rest are `templates/archetypes/<name>`. The kits come from
`engine/extensions`: shooter, rpg (rpg, deckbuilder, open-world),
racing, puzzle (match3) and strategy (tower-defense).

Each view's camera (`VIEW_PRESETS`) is an `ArtCamera` written into the
look and the controller the starter puts on each seat: orthographic
`og.Follow2D` for both 2D views; `og.FollowCamera` behind the hero in
third person; `og.FirstPersonCamera` at the eye in first person; an
orthographic `og.FollowCamera` at true isometric (pitch 35.264°, yaw 45°)
for `isometric-2.5d`, where the move is relative to the screen and the
look turns the hero. The 3D starters share one rig, `src/view.ts`
(`templates/game`'s; the test keeps the copies equal).

### Seats: every input, every player count

Every starter reads only `og.PlayerInput`, one entity per seat, which
`src/seats.ts` (one file in every starter, kept equal by the test) fills
from the keys, the mouse, the pads and the touch controls by the
controls `src/creation.ts` names: `single` is seat 0 from every device;
`local-multiplayer` two seats on one device (WASD and the left keys
against the arrows and the right keys, pad 1 against pad 2, two sets of
touch controls, split screen in 3D); `online-lockstep` this peer's seat
over a room, with a room playtest that compares both peers' hashes. Menu
and HUD buttons press a seat's control (`press`), so a room sees them on
one tick; settings that differ per device stay out of the world. Every
genre plays every count but `idle`, which refuses both multiplayer
counts: its progress is made while the player is away, which a second
seat cannot share and no lockstep room can run.

A premise picks among the catalogue looks that suit its view
(`look.views`); `--look` may name any look of the view's profile. The
look carries the mood; music is a request (`audioBrief`), generated by
the platform.
Changing `create` later does not rewrite source automatically:
`applyCreationChoices(dir, selection)` re-applies a selection to a game
made from a profile's overlay. Do not use `--force` to update an authored
game; it overwrites template files.

`--lang` sets up the languages (bin/languages.ts): a string table per
language under `assets/strings/` (the starter's own, the starters'
translations in `templates/locales/strings/`, or an empty table the cook
reports key by key), the face each script needs from
`templates/locales/fonts/` (Japanese, Korean, Traditional Chinese, Arabic,
Hebrew, Cyrillic and Greek; Simplified Chinese is the UI style's `cjk`
companion) with its licence, `direction rtl` for Arabic and Hebrew, the
pseudo-locale `qps-ploc` and `tools/pseudo-locale.ts` that remakes it, and
`tests/locale-<tag>.og.ts` per language plus `tests/pseudo.og.ts`. The
skills' og-i18n §0 says what each file is for. The additive exports are
`canonicalTag`, `isRtl`, `localeScript`, `LocaleScript`, `PSEUDO_LOCALE`,
`pseudoMessage` and `pseudoTable`. `templates/locales/make-fonts.py` remakes
the faces from google/fonts.

## The art brief

The package also exports the art brief (`import { artBrief, readLook,
presetLook } from "@origingame/create-game"`): a game's look compiled into
the request for one asset of a kind (`sprite`, `tileset`, `portrait`,
`backdrop`, `prop`, `character`, `texture`, `icon`): the prompt (the
style's direction, the palette as named colours with hex, the sizes, the
line, the shape language, what to avoid), the reference images (the
style's preview, then the game's accepted assets) and the generator's
parameters (transparent and size for an image, the face budget and texture
size for a model). `readLook(dir)` reads a scaffold's look (project.ogt's
`look` line down to its preset); `presetLook(slug)` a preset alone, before
the game exists. `bin/brief.ts` is a byte-for-byte copy of the knowledge
package's `scripts/brief.ts`, whose README (§The art brief) documents the
signature and shapes, a public contract kept stable. The prepack copies it
with the presets and built-in palettes it reads (`presets/`), and the
tests fail when the copy differs.

## The audio brief

`audioBrief({ music, mood, genre, premise? })`, from the same
`bin/brief.ts`, turns the creation choices into the request for every music
cue and sound effect the game needs, with the og cook presets that stand in
until the files exist; `readAudioChoices(dir)` reads the create block. The
knowledge package's README (§The audio brief) documents the shapes, a
public contract kept stable, and docs/art.md §2.9 the rules.

## The auto-rig's parts

`rigParts(template)` (from the same `bin/brief.ts`) lists the layers a
character's layered art needs for one of the auto-rig's templates
(`RIG_TEMPLATES`: `humanoid-side`, `humanoid-front`, `quadruped`,
`creature`), back to front: each part's layer name, whether the cook
requires it, its side (`l`, `r` or null), the bone it drives, the parts
it hangs from and a one-line note on how its joint is found from its
overlap (or set by a `pivot` line). Another name throws, naming the four.
The data is og-cook's templates, held equal by og-cook's
`rig_parts_match_the_templates` test; the contract itself is docs/art.md
§2.8 in the engine repository.

The published package carries the 3D template under `template/`, the 2D
starter under `template-2d/`, the style catalogue under `looks/` and the look presets under `presets/`, copied by the `prepack` script
(`scripts/pack-template.ts`); build output (`cooked/`,
`dist/`, `node_modules/`) is never copied. The template's `gitattributes`
(media through Git LFS) is written as the project's `.gitattributes`: under
its own name it would route the engine repository's plain-blob media through
LFS too. `bun run test` scaffolds into a
temporary directory and checks the result.

Authors reach it as `og new <dir> [options]`: the og launcher runs this
package's `bin/create.ts` from the engine's dist (`create/`, docs/release.md
§1). It is no longer published to npm; the versions on npm up to 0.7.8-dev.17
stay there.
