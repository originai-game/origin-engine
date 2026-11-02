# Kit art (docs/ui.md §15.5.3)

Painted art for the kit's examples, so its frames show real art instead of
line-icon placeholders. Generated on 2026-11-02 by the UI art review with
the OriginGame platform's image tool (origin MCP `generate_image`, model
gpt-image-2.5-sunburst) for this repository: original work for the project,
no third-party art, released CC0 1.0 with the rest of the kit's self-made
assets. One direction for all of it (semi-realistic painterly fantasy, one
warm key light from the upper left), so it reads under every style.

Sheets were cut into cells (`items/`, `boons/`: 256 px; `cards/`,
`portraits/`: 512 px) and saved as JPEG; scenes keep their aspect.

| Folder | Kind (docs/art.md §2.2) | Files |
|---|---|---|
| `items/` | itemIcon | iron-sword, oak-shield, crown, hunters-bow, healing-draught, mana-vial, ember-ring, iron-key, sapphire-amulet, knight-helm, gauntlets, boots, ashbringer, arcane-staff, mage-robe, elven-bow |
| `boons/` | abilityIcon, currency, talentIcon | thunder-strike, titan-heart, crown-of-ages, gold, crystal, mana, bomb, apple, flame-blade, poison, curse, frost, star, rune-shield, broken-chain, padlock |
| `cards/` | cardArt | strike, defend, bash, anger, inflame, wound |
| `portraits/` | bust, unitPortrait | aria, kael, mira (gacha featured), cultist, red-louse (enemies), aria-vell (the paperdoll's character) |
| `scenes/` | keyArt, fullBody, menuBackground | gacha-moonlit-tide (16:9, subject in the right 55 %, text side dark), paperdoll-aria-vell (2:3), title (16:9, menu side dark) |

## Added 2026-11-02 (the kit-content stream)

Backdrops for the 16 built-in styles (`src/main.ts` `STYLE_BACKDROPS`) and
the shop's merchant, so each style's screens sit on their own scene:

| File | Kind | Origin |
|---|---|---|
| `portraits/brann.jpg` | bust (the shop's merchant) | origin MCP `generate_image` (gpt-image-2.5-sunburst), same direction as the set above, 512 px |
| `scenes/scifi.jpg`, `cyber.jpg`, `tactical.jpg`, `broadcast.jpg`, `kawaii.jpg`, `jade.jpg`, `wasteland.jpg`, `midcore.jpg`, `foundry.jpg` | menuBackground (16:9, the left 40 % calm for the menu) | origin MCP `generate_image` (gpt-image-2.5-sunburst), one prompt per style direction, cut to 1280×720 |
| `scenes/gothic-halls.jpg`, `storybook-vale.jpg`, `doodle-town.jpg`, `golden-valley.jpg`, `manuscript.jpg` | menuBackground | copies of the art samples' far layers (`examples/art-paint-2d/styles/<gothic-contour, decorative-storybook, detailed-doodle, classical-painting-collage, illuminated-manuscript>/art/backdrop.jpg`), made for this repository with Painter (`tests/evidence/art/samples/NOTES.md`) |
| `scenes/pixel-dusk.png` | menuBackground | `examples/art-pixel-side/styles/detailed-pixel/art/backdrop.png`, scaled ×3 by nearest neighbour |
