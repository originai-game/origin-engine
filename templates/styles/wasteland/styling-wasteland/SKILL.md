---
name: styling-wasteland
description: "Applies and edits the wasteland UI style. Use when installing this pack, changing its dressings or creating a related game-owned style."
---

# Styling with wasteland

Run `og ui list --json` to find the pack and its installed path. Install
with `og style add wasteland` in the game, then set `ui "wasteland"` in its
look. The editable copy is `assets/styles/wasteland/`; load that look normally.

Edit `style.ogt` directly: palette roles set colour, `typography` sets
text roles, and `dress` selectors paint named component parts and states.
Do not regenerate the pack. `choices.json` only supplies the starting
choices for `og style new`; it does not overwrite authored style data.
Keep fonts, atlases, import settings and licences with the pack. Asset paths
are relative to the style unless they deliberately share `@styles/` assets.

To change a results screen, use `og ui eject results` (or choose a screen
from `og ui eject --list`), edit the document under `assets/ui/`, and
load it with `og.load.document`. Its props, slots and event declarations
are the contract; keep game data in the viewmodel, not in the style.

For a new pack, run `og style new my-style --look "post-apocalyptic survival"`.
Then edit its data rather than the generator. A reusable extension needs
`extension.json` (format 1, kind style, its own name/version/description
and skill path), `style.ogt`, this short authoring skill and the assets it
references. Use `og style add ./path/to/my-style` to install a local pack.

Run `og cook` after edits. Review the actual game at desktop and phone
sizes and 150% text: hierarchy, materials, states, art and typography.
No contrast threshold is an acceptance condition. In the engine repository,
only the UI landing runner renders; an author supplies the affected pages.
