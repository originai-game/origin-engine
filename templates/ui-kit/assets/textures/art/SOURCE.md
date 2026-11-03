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

## Round 2 (2026-11-02, the art review's notes)

| File | Kind | Origin |
|---|---|---|
| `cutouts/*.png` (20 items) | itemIcon, cut out | origin MCP `generate_image` (gpt-image-2.5-sunburst, transparent), the same direction as `items/`; where the model returned a flat magenta ground it was keyed out (ImageMagick flood fill from the corners, the edge eroded 2 px and feathered), 320 px. The kit's shop, inventory and rewards draw these on the slot's rarity ground. |
| `events/kawaii.jpg`, `cartoon.jpg`, `storybook.jpg` | banner (the title's limited-banner card) | origin MCP `generate_image`, one featured character per style in its own look, 640×360 |
| `scenes/scifi.jpg`, `cyber.jpg` | menuBackground | regenerated to the owner's redirected pack sheets (ui-art/70c6e287e1a1: a crewed launch pad at dawn; ui-art/46b39cda6040: acid yellow, signal red and cyan, no magenta) |
| `scenes/neutral.jpg` | menuBackground | self-made: a dark radial gradient with a vignette (ImageMagick), the default style's neutral ground |

## Round 3 (2026-11-02, the art review's notes)

| File | Kind | Origin |
|---|---|---|
| `scenes/midcore.jpg` | menuBackground | regenerated after the originality gate refused the first (character trade dress, real heraldry, a pasted seam): origin MCP `generate_image` (gpt-image-2.5-sunburst), a landscape with no characters, a stone fort on a bright hill behind a palisade, invented frog-on-lily-pad banners, thick-outlined stylised terrain; cut to 1280×720 |

## Round 4 (2026-11-02, the art review's notes)

| File | Kind | Origin |
|---|---|---|
| `scenes/cyber.jpg` | menuBackground | regenerated after the originality gate refused the r3 scene (a lone figure facing the city, a face billboard): origin MCP `generate_image` (gpt-image-2.5-sunburst), an empty wet rooftop, acid-yellow and cyan towers with signal-red lights, invented Latin signage only (no katakana, no faces, no people); cut to 1280×720 |

## Figures (2026-11-02, the U6d dialogue stream)

| File | Kind | Origin |
|---|---|---|
| `figures/mira.png` | fullBody, cut out (a conversation's speaker, waist up, 2:3) | Painter (transparent background), from `portraits/mira.jpg` as the identity reference, in the same direction; 640×960. The kit's dialogue and visual-novel screens stand it at the speaker's side. |
| `figures/aria.png` | fullBody, cut out (a conversation's speaker, waist up, 2:3) | origin MCP `generate_image` (gpt-image-2.5-sunburst, transparent), from `portraits/aria.jpg`'s description (silver hair, the crescent circlet and pendant, the black hooded cloak) in the same direction as `figures/mira.png`; the model returned a flat magenta ground, keyed out (ImageMagick, fuzz 22 %, the edge eroded 2 px, feathered and despilled), 640×960. The visual novel's Aria stands it. |

## The genre previews (2026-11-02, the U6d stream, r40)

Art for the lobby, character select, versus, draft, world map and song
select previews, in the same direction as the set above:

| File | Kind | Origin |
|---|---|---|
| `figures/kael.png` | fullBody, cut out (a versus side's splash, waist up, 2:3) | origin MCP `generate_image` (gpt-image-2.5-sunburst, transparent), from `portraits/kael.jpg`'s description (auburn hair, the scar, a teal jerkin over mail, a bronze pauldron, the sword over the shoulder) in the direction of `figures/mira.png`; 640×960 |
| `portraits/torvin.jpg`, `wren.jpg`, `vesper.jpg`, `maren.jpg` | bust, unitPortrait (a roster's invented fighters) | origin MCP `generate_image` (gpt-image-2.5-sunburst), one 2×2 sheet cut into cells, 512 px |
| `songs/neon-moon.jpg`, `sakura-wind.jpg`, `stormbreak.jpg`, `desert-drum.jpg`, `crystal-echo.jpg`, `phoenix.jpg` | jacket (a song's cover, square) | origin MCP `generate_image` (gpt-image-2.5-sunburst), one 3×2 sheet cut into cells, 384 px |
| `maps/world.jpg` | map (an invented continent from above) | the platform's image API (`/v1/images/generations`, gpt-image-2.5-sunburst) with the development key, the same tool as `generate_image` (whose reply was too large to carry it); cut from 3:2 to 16:9, 1536×864 |
## Starship screens (2026-11-02, holo's screens as kit screens)

The five screens the holo pack carried (ship status, turn order, ability
cards, sector map, hangar) are kit screens styled by the active pack. Their
art is new and ours; the holo concept crops (ui-art/0cccb255873a,
ui-art/11d3db9b1c45) were layout references only.

| File | Kind | Origin |
|---|---|---|
| `ships/*.png` (11: valiant, aegis, kestrel, sentinel, vanguard, carrier, module allied; dreadnought, ravager, marauder, skirmisher hostile) | unitPortrait, cut out | origin MCP `generate_image` (gpt-image-2.5-sunburst, transparent): grounded near-future spaceflight (allies white with a blue stripe facing right, hostiles graphite with red lights facing left), no text, insignia or registries; `module` returned on a flat magenta ground, keyed out (flood fill from the corners, edge eroded 2 px, feathered); trimmed, 400 px (vanguard, the hangar's hero, 640 px) |
| `space/*.jpg` (kinetic-strike, deflector-screen, squadron-launch, repair-drones, gravity-well, overcharge) | cardArt | origin MCP `generate_image`, the same direction, 512² |
| `scenes/vanguard-orbit.jpg` | keyArt (the hangar's detail) | origin MCP `generate_image`: the frigate above Earth's limb at dawn, 960 px |
| `space/star-dust.png` | field ground (the sector map's og.ui.nodeGraph `ground`) | generated by a script (a seeded scatter of white points on transparent, wrapped so the 256 px tile is seamless); CC0. Tinted in the pack's ink at 6 % by the graph. |

## The last genre previews (2026-11-02, the U6d stream, r41)

Art for the garage, ledger, RTS commands, loadout and VN gallery previews, in the same direction as the set above:

| File | Kind | Origin |
|---|---|---|
| `garage/coupe.png` | vehicle, cut out (the garage's car) | the platform's image API (`/v1/images/generations`, gpt-image-2.5-sunburst, transparent) with the development key, an invented coupe; trimmed, 1024 px wide |
| `garage/turbo.jpg`, `wheels.jpg`, `suspension.jpg`, `exhaust.jpg`, `brakes.jpg`, `wing.jpg` | itemIcon (the garage's parts) | the same API, one 3×2 sheet cut into cells, 256 px |
| `scenes/garage.jpg` | menuBackground (the garage's turntable, no car) | the same API, cut to 1280×720 |
| `loadout/raven.png` | item, cut out (the loadout's weapon) | the same API (transparent), an invented rifle; trimmed, 1024 px wide |
| `loadout/sidearm.jpg`, `frag.jpg`, `stim.jpg`, `knife.jpg`, `drone.jpg`, `shield.jpg`, `optic.jpg`, `suppressor.jpg`, `grip.jpg`, `magazine.jpg`, `stock.jpg`, `laser.jpg` | itemIcon (the loadout's slots and attachments) | the same API, two 3×2 sheets cut into cells, 256 px |
| `cg/riverboat.jpg`, `storm-ferry.jpg`, `dawn-gate.jpg` | cg (the VN gallery's event pictures) | the same API, one prompt each, cut to 1280×720 |
| `city/residential.jpg`, `commercial.jpg`, `industrial.jpg`, `tourism.jpg`, `police.jpg`, `fire.jpg`, `health.jpg`, `education.jpg`, `transit.jpg`, `parks.jpg`, `power.jpg`, `water.jpg` | itemIcon (the ledger's taxes and services, the overlay bar's lenses) | the same API, two 3×2 sheets cut into cells, 256 px |
| `scenes/city.jpg` | menuBackground (the ledger's city at dusk) | the same API, cut to 1280×720 |
| `rts/stop.jpg`, `patrol.jpg`, `build.jpg`, `rally.jpg`, `gather.jpg`, `repair.jpg`, `barracks.jpg`, `tower.jpg`, `farm.jpg`, `wall.jpg`, `forge.jpg`, `keep.jpg` | ability (the RTS command card's orders and buildings) | the same API, two 3×2 sheets cut into cells, 256 px |
| `scenes/battlefield.jpg` | menuBackground (the RTS preview's field under the console) | the same API, cut to 1280×720 |

## The last pattern previews (2026-11-03, the U6d stream, r42)

Art for the photo mode, deck builder and VN gallery previews, in the same direction as the set above:

| File | Kind | Origin |
|---|---|---|
| `photo/shot.jpg` | screenshot (photo mode's paused shot: a traveller over a valley at sunrise) | the platform's image API (`/v1/images/generations`, gpt-image-2.5-sunburst) with the development key, cut to 1280×720 |
| `photo/vivid.jpg`, `noir.jpg`, `sepia.jpg`, `cool.jpg` | screenshot (the shot through each filter, as a game's render would draw it) | made from `photo/shot.jpg` with ImageMagick (saturation and contrast, greyscale, sepia tone, a cool colourise) |
| `photo/frame-cinema.png`, `frame-polaroid.png`, `frame-film.png` | overlay on transparent ground (photo mode's frames: 2.39:1 bars, a print's border, a film strip) | drawn with ImageMagick for the kit, 1280×720 |
| `photo/thumb-cinema.jpg`, `thumb-polaroid.jpg`, `thumb-film.jpg` | thumbnail (each frame over the shot, its tile) | the shot and the frame composited with ImageMagick, 384×216 |
| `cards/fireball.jpg`, `bulwark.jpg`, `rogue.jpg`, `potion.jpg`, `storm.jpg`, `serpent.jpg` | card art (deck builder's collection, first sheet: a falling fireball, a knight behind his shield, a hooded rogue, a green potion, a sword in a storm, a serpent) | the platform's image API (`/v1/images/generations`, gpt-image-2.5-sunburst) with the development key, one 3×2 sheet cut into tiles of 512×512 |
| `cards/ice.jpg`, `horn.jpg`, `golem.jpg`, `owl.jpg`, `scroll.jpg`, `egg.jpg` | card art (second sheet: an ice crystal, a war horn, a stone golem, an owl, a burning scroll, an egg in embers) | the same API, a second 3×2 sheet cut into tiles of 512×512 |
| `cg/lantern.jpg`, `library.jpg`, `duel.jpg` | event CG (the VN gallery's pictures: two companions at a lantern festival, a reader asleep in a library, a duel at dusk) | the platform's image API, the same model and key, cut to 1280×720 |
| `cg/riverboat-night.jpg`, `duel-dusk.jpg` | event CG variant (the gallery's second takes of `riverboat.jpg` and `duel.jpg`) | made from those with ImageMagick (a night and a dusk recolour) |
