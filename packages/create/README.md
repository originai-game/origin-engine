# @origingame/create

Scaffolds an Origin Engine game from the project template (`templates/game`
of the engine repository): `project.ogt`, `src/`, `assets/`, `tests/`,
`package.json` and `tsconfig.json`, with the game's name and id filled in.

```sh
bun packages/create/bin/create.ts my-game --name "My Game"   # in the engine repository
cd my-game && bun install && og cook && og run .
```

`create [dir] [--name "Name"] [--id slug] [--force]`: the id defaults to the
name in lowercase-hyphen form (or the directory's name); a directory that is
not empty is refused unless `--force` is given.

The published package carries the template under `template/`, copied by the
`prepack` script (`scripts/pack-template.ts`); build output (`cooked/`,
`dist/`, `node_modules/`) is never copied. The template's `gitattributes`
(media through Git LFS) is written as the project's `.gitattributes`: under
its own name it would route the engine repository's plain-blob media through
LFS too. `bun run test` scaffolds into a
temporary directory and checks the result.

Note on the name: npm maps `npm create @origingame/game` to the package
`@origingame/create-game` and `npm create @origingame` to `@origingame/create`.
This package is `@origingame/create` (design §9); publishing it also as
`@origingame/create-game`, or documenting `npm create @origingame`, is the
publisher's choice.
