# @origingame/create-game

`og create` (also `og new`) writes one empty project from `template/`:
`project.ogt`, `src/main.ts`, `tsconfig.json`, `package.json`, `tests/`,
and `assets/`. The entry exports `og.game({})`. There is no 2D or 3D
variant, sample scene, gameplay, or installed art.

```sh
og create my-game --name "My Game" --view side-2d --look pixel --no-prompt
```

The agent chooses the form, genre, view, look and variant, UI style,
platform, input, players, languages, and music. These choices are constraints and hints in
`project.ogt`'s `create` block. Creation also writes `assets/looks/<id>.ogt`
with the chosen preset, variant and UI pack, named by the manifest's top-level
`look`. It generates no gameplay or art, and changes no publish settings.
The agent builds the game with the engine API and skills after creation.
The block's `form` is also the game's profile, the one field it follows:
`form "2d"` is a 2D game (rapier2d physics, sprites in the art brief, the
cook's 2D conformance, the 2D publish module); `2.5d`, `3d` and `film` are
3D (`profileOf(form)`). Each requested language installs its
script's font pack under `assets/fonts/<pack>/` with its licence (docs/ui.md
§3.4), which og.i18n chains with no generated code; the languages still need
authored string tables. `og check` warns about a language with no font for
its script.

## Options and return value

`create [dir] [--name "Name"] [--id slug] [--premise "one line"] [--force]`
accepts `--form`, `--genre`, `--view`, `--look`, `--variant`,
`--ui`, `--platform`, `--input`, `--players`, `--lang en,zh`, and `--music`.

`--form` is `2d`, `2.5d`, `3d` or `film`. For a game, the view is one of
its form's (`2d`: side-2d, top-down-2d; `2.5d`: isometric-2.5d; `3d`:
third-person-3d, first-person-3d), defaulting to its genre's first view of
that form; a form and view that disagree are refused. `film` is an
interactive film (`og api show flow`): creation writes the same skeleton
plus `assets/story/film.yarn` (three flow nodes, two endings),
`assets/segments/opening.ogt` and `ending.ogt`, a placeholder
`assets/clips/opening.ogt` stream, the picture's `assets/ui/film-view.tsx`
and an entry that loads and starts the film paused and shows it with the
built-in Subtitles and FlowChoices (from `templates/film`, packed
as `film/`; the clip is pinned in `templates/og-assets.json`, so a checkout
runs `bun scripts/assets.ts ensure templates` first). `--genre
interactive-film` chooses form film too. A film's genre is
`interactive-film` and it has no `view` or `players`; another genre, a view
or players is refused. The look must draw the form (its catalogue `views`; every look
styles a film), and `--variant` must be one of the look's. Mood is not a
creation choice: the look draws in its preset's or variant's own mood, and
`--mood` is refused (write `mood "<word>"` in the look file to change it).

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
`creationSchema(lookSlugs)`, `creationLooks(styles?)`, `parseCreationInput(value)`,
`inferCreationChoices(premise, input?, defaults?)`,
`resolveCreationChoices(premise, input?, catalogue?)`, and
`creationBlock(choices)` expose the selection contract.

`creationSchema` returns JSON Schema 2020-12 with stable enum IDs. Its
`x-labels` map supplies English and Simplified Chinese labels and
descriptions for enum choices. Look entries supply Chinese copy; English
look names and descriptions remain in the catalogue.

`creationLooks()` lists the looks a new game may pick: the catalogue's
twelve, each with a preset. Pass it to `creationSchema`. A name that is no
look fails with the list. A look's UI pack (default, cartoon, fantasy or
scifi) is the default UI choice. `variant` names one of the look's variants
(docs/art.md §2.11) and is written into the look file.

The inference helpers suggest choices using English and Chinese keywords.
They do not implement those choices. Languages default to Japanese for
kana, Chinese for Han text, or English otherwise. Music defaults to ambient.

## Creation document

`creationDocument(previews?, catalogue?)` returns everything a home page
needs to offer creation, and `og create --creation [--previews file.json]`
prints it as `JSON.stringify(doc, null, 2)` plus a newline. A release
publishes it, every preview filled, as `<version>/creation.json`. It is a
contract with the platform: a field is added, never renamed or removed,
without a breaking changelog entry.

| Field | Contents |
|---|---|
| `schemaVersion` | 1 |
| `order`, `more` | the home page's questions in order (`form`, `genre`, `look`, `variant`, `ui`), and those behind "more settings" |
| `schema` | `creationSchema(looks)`: JSON Schema 2020-12 of a selection, with `form`, `variant`, and `allOf` rules (a film's genre is interactive-film and it has no view or players; a game's view is its form's) |
| `options.<field>` | each option's `id`, `label` and `description` (`en`, `zh-CN`); views carry their `form`; looks carry `variants`; looks, variants and `ui` options carry `preview` |
| `compat` | `formViews`, `genreViews` (preferred first), `genreDefaultView[genre][form]`, `formFields[form]` (the fields that apply), and `looks[slug]`: `forms`, `views`, recommended `ui`, `variants` |
| `defaults` | the default `form` and `byForm[form]`: what creation picks with no premise |

A `preview` is `{ path }`, relative to the version root, and the release
adds `url`, `sha256`, `size` and `engine` (the version that rendered it).
`previewSlots(catalogue?)` lists every slot as `{ path, kind, look?,
variant?, pack? }`: `previews/<look>.webp`,
`previews/<look>.<variant>.webp` and `previews/ui/<pack>.webp`. Given previews, `creationDocument` throws on an
unfilled slot or a path that is no slot. Both read the packed
`looks/catalogue.json` by default, so a release imports the version's own
copy from `create/bin/create.ts`.

## Art and audio contracts

The package re-exports the knowledge package's art and audio briefs without
changing their signatures. `bin/knowledge.ts` imports
`packages/knowledge/scripts/brief.ts` itself in a checkout, and the engine
package's `knowledge/scripts/brief.ts` beside the packed package (the
release archive's `engine/`, or `@origingame/engine` in `node_modules`), so
nothing is copied or generated first.

- `presetLook(slug)` reads a preset before a game has art.
- `readLook(dir)` reads the project's configured look file.
- `artBrief(look, kind, subject)` describes an asset's art requirements.
- `audioBrief({ music, mood, genre, premise? })` describes music and effects.
- `readAudioChoices(dir)` reads the `create` block, and the look's mood.
- `rigParts(template)` describes the auto-rig's layered-art contract.

UI packs, look presets, locale resources, and design tokens remain available
separately. The cook resolves the look's preset and UI pack; locale fonts
still need installation and loading as the `screens` guide describes (`og api show screens`).

The release archive carries this package in `create/`. It ships one tracked
`template/` and the catalogue and previews creation reads, but no
`template-2d/` or `archetypes/`. `bun run test` checks source and packed
creation against the same skeleton contract.
