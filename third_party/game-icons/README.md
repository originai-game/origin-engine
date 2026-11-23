# game-icons.net

Silhouettes from [game-icons.net](https://game-icons.net)
(github.com/game-icons/icons), Creative Commons Attribution 3.0 Unported
(`LICENSE`): "Icons made by {author}, from game-icons.net".

They make up the ink icons of the engine's game icon set,
`templates/styles/shared/icons/game.ogt`: `ICONS.md` names each one, its
source and its author. `templates/tools/make-game-icons.py` fetches them at a
pinned revision, scales their paths from the 512-unit to the 24-unit grid and
drops the site's background square; the shapes are otherwise unchanged.

The attribution travels with every game that draws them: the engine's credits
(`engineCredits` in `@origingame/engine/credits`, which the Credits screen
lists after the game's own sections) name game-icons.net and each author.
