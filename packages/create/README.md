# @origingame/create-game

`og create` (also `og new`) writes one empty project from `template/`:
`project.ogt`, `src/main.ts`, `tsconfig.json`, `package.json`, `tests/`,
and `assets/`. The entry exports `og.game({})`. There is no 2D or 3D
variant, sample scene, gameplay, or installed art.

```sh
og create my-game --name "My Game" --view side-2d --look 16bit-pixel --no-prompt
```

The agent chooses genre, view, look, mood, UI style, platform, input,
players, languages, and music. These choices are constraints and hints in
`project.ogt`'s `create` block. Creation also writes `assets/looks/<id>.ogt`
with the chosen preset, mood and UI pack, named by the manifest's top-level
`look`. It generates no gameplay or art, and changes no publish settings.
The agent builds the game with the engine API and skills after creation.
`profile` remains an optional runtime/build setting, omitted here;
`create.view` does not replace it. Each requested language installs its
script's font pack under `assets/fonts/<pack>/` with its licence (docs/ui.md
§3.4), which og.i18n chains with no generated code; the languages still need
authored string tables. `og check` warns about a language with no font for
its script.

## Options and return value

`create [dir] [--name "Name"] [--id slug] [--premise "one line"] [--force]`
accepts `--genre`, `--view`, `--look`, `--mood`, `--ui`, `--platform`,
`--input`, `--players`, `--lang en,zh`, and `--music`.

On a terminal, missing choices are prompted with inferred defaults.
`--no-prompt` accepts those defaults. Explicit choices win. `--choices`
remains accepted but is unnecessary: every invocation writes the block.
`--archetype` and `--profile` are removed. Use `--view` to record the view.

The id defaults to the name in lowercase-hyphen form, or the directory's
name. A nonempty directory is refused unless `--force` is given. Force
overwrites skeleton files, so it is not an update command for authored games.

`create({ dir, name?, id?, premise?, look?, choices?, force?, catalogue? })`
returns `{ dir, name, id, files, choices }`. The old `template`, `profile`,
and `archetype` options and `Created.look`, `Created.profile`, and
`Created.notes` fields are removed.

## Choice schema

`CREATION_CHOICES`, `CreationChoices`, `CreationInput`,
`creationSchema(lookSlugs)`, `parseCreationInput(value)`,
`inferCreationChoices(premise, input?, defaults?)`,
`resolveCreationChoices(premise, input?, profile?, catalogue?)`, and
`creationBlock(choices)` expose the selection contract.

`creationSchema` returns JSON Schema 2020-12 with stable enum IDs. Its
`x-labels` map supplies English and Simplified Chinese labels and
descriptions for enum choices. Look entries supply Chinese copy; English
look names and descriptions remain in the catalogue.

The inference helpers suggest choices using English and Chinese keywords.
They do not implement those choices. Languages default to Japanese for
kana, Chinese for Han text, or English otherwise. Music defaults to ambient.

## Art and audio contracts

The package re-exports the knowledge package's art and audio briefs without
changing their signatures. `bin/brief.ts` is copied byte-for-byte from
`packages/knowledge/scripts/brief.ts` during prepack.

- `presetLook(slug)` reads a preset before a game has art.
- `readLook(dir)` reads the project's configured look file.
- `artBrief(look, kind, subject)` describes an asset's art requirements.
- `audioBrief({ music, mood, genre, premise? })` describes music and effects.
- `readAudioChoices(dir)` reads the `create` block.
- `rigParts(template)` describes the auto-rig's layered-art contract.

UI packs, look presets, locale resources, and design tokens remain available
separately. The cook resolves the look's preset and UI pack; locale fonts
still need installation and loading as the `screens` guide describes (`og api show screens`).

The release archive carries this package in `create/`. It ships one tracked
`template/` and the catalogues and presets used by the briefs, but no
`template-2d/` or `archetypes/`. `bun run test` checks source and packed
creation against the same skeleton contract.
