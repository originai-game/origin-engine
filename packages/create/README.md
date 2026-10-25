# @origingame/create-game

Scaffolds an Origin Engine game from a project template of the engine
repository, the 3D template `templates/game` or the 2D starter
`templates/game-2d`: `project.ogt`, `src/`, `assets/`, `tests/`,
`package.json` and `tsconfig.json`, with the game's name and id filled in.

```sh
bun packages/create/bin/create.ts my-game --name "My Game"   # in the engine repository
cd my-game && bun install && og cook && og run .
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

The published package carries the 3D template under `template/`, the 2D
starter under `template-2d/` and the style catalogue under `looks/`, copied by the `prepack` script
(`scripts/pack-template.ts`); build output (`cooked/`,
`dist/`, `node_modules/`) is never copied. The template's `gitattributes`
(media through Git LFS) is written as the project's `.gitattributes`: under
its own name it would route the engine repository's plain-blob media through
LFS too. `bun run test` scaffolds into a
temporary directory and checks the result.

Note on the name: npm maps `npm create @origingame/game` to the package
`@origingame/create-game` and `npm create @origingame` to `@origingame/create-game`.
This package is `@origingame/create-game` (design §9); publishing it also as
`@origingame/create-game`, or documenting `npm create @origingame`, is the
publisher's choice.
