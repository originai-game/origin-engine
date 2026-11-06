# Asset licences

| File | Licence | Origin |
|---|---|---|
| `assets/sprites/units.png`, `assets/sprites/units.ogt`, `assets/sprites/*.png.ogt` | CC0 1.0 | self-made: the Warden, the Hunter and the five foes side-on in two idle frames each, and each as a table token seen from above, drawn texel by texel by `tools/make-assets.py` (standard library only, deterministic), with the flipbook and import settings |
| `assets/sprites/portraits/*.png`, `*.png.ogt` | CC0 1.0 | self-made: each unit's head for the unit frames and the turn order, cut from its frame by the same script |
| `assets/sprites/battlefield.png`, `assets/sprites/table.png` | CC0 1.0 | self-made: the ash plain at dusk (side-2d) and the war table on its plank floor (top-down-2d), same script |
| `assets/cards/*.png`, `*.png.ogt` | CC0 1.0 | self-made: the fifteen cards' art, same script |
| `assets/audio/*.wav` | CC0 1.0 | self-made: synthesised tones and noise (the card, hits, block, hurt, poison, buffs, the draw, the turns, a fall, the win, the loss) and the battle's music loop, same script |
| `assets/audio/*.ogt` | CC0 1.0 | `og sfx` presets (click, select, back, error, levelup) that `og cook` synthesises, written by the same script |
| `assets/ui/*.ogt` | MIT | the engine's screen documents `main-menu` and `pause` (`og ui eject`), edited for this game |
| `assets/looks/ashen.ogt` | CC0 1.0 | self-made: the game's look, the engine's `16bit-pixel` extended with this game's UI style |
| `assets/strings/*.ogt` | CC0 1.0 | self-made: the English and Chinese string tables |
| `assets/styles/ashen/*` | CC0 1.0 art, SIL OFL 1.1 fonts | the engine's pixel style pack (`templates/styles/pixel`) with this game's palette (`assets/styles/ashen/LICENSES.md`) |
| `references/*/*.png` | CC0 1.0 | frames of this game recorded by `tests/frames.og.ts` |

The unit frames and the turn order are the engine's rpg kit
(`@origingame/engine/extensions/rpg`, MIT, the engine's licence); the hand
and the rewards are og.ui's own `cardHand` and `rewardChoice`.
