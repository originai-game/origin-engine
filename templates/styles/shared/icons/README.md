# The engine's game icon set

`game.ts` is the icon set `<Icon name>` draws from when the game has no icon of
that name (the default pack's `iconSet`, docs/ui.md §3.3); names it lacks fall
back to the engine's Lucide line icons (`back`, `close`, `chevronLeft`, `gear` …).
Every icon sits on a 24-unit grid and is cooked as a distance field at 48 px
an em, crisp from 16 to 64 px. `templates/tools/make-game-icons.ts` writes this
directory; edit the tool, not the files.

## Hero icons (tones)

Our own designs (CC0), drawn as stickers: an `icon-outline` rim and an
`icon-shadow` drop under the body, which is painted in theme colours, then
`icon-shade` and `icon-gloss` shapes and `icon-mark` marks over it. A pack
restyles all of them through those colours (CONVENTIONS.md §The default
pack's names); a pack whose own `iconSet` names an icon replaces it.

| Name | Body colours |
|---|---|
| `coin` | `icon-gold` |
| `gem` | `icon-purple` |
| `star` | `icon-gold` |
| `heart` | `icon-red` |
| `shield` | `icon-blue` |
| `energy` | `icon-gold` |
| `ammo` | `icon-gold`, `icon-orange` |
| `lock` | `icon-gold`, `icon-steel` |
| `key` | `icon-gold` |
| `chest` | `icon-gold`, `icon-wood` |
| `gift` | `icon-gold`, `icon-red` |
| `crown` | `icon-gold`, `icon-red` |
| `potion` | `icon-glass`, `icon-red`, `icon-wood` |
| `ticket` | `icon-orange` |
| `xp` | `icon-green` |
| `alert` | `icon-red` |
| `new` | `icon-red` |
| `rarity` | `icon-blue` |
| `trophy` | `icon-gold`, `icon-wood` |
| `hourglass` | `icon-glass`, `icon-gold`, `icon-wood` |

## Ink icons

Filled game-icons.net silhouettes (CC BY 3.0; authors in
`third_party/game-icons/ICONS.md`, credited on the Credits screen), drawn in the
icon's ink (`text-*`, or `fill-*`).

- **Combat and stats**: `attack`, `sword`, `axe`, `bow`, `staff`, `magic`, `armor`, `speed`, `crit`, `accuracy`, `strength`, `intellect`, `luck`, `levelUp`, `bomb`, `skull`, `flame`, `barrier`
- **Navigation and menus**: `home`, `shop`, `bag`, `hero`, `team`, `friends`, `quest`, `map`, `mail`, `chat`, `leaderboard`, `calendar`, `event`, `castle`, `banner`, `compass`, `book`, `search`, `cards`, `dice`, `rocket`, `inventory`, `skillTree`, `rank`
- **Settings and system**: `sound`, `soundOff`, `music`, `bell`, `vibrate`, `language`, `gamepad`, `save`, `exit`, `power`, `info`, `account`, `phone`, `unlock`, `settings`, `refresh`
- **Social**: `like`, `dislike`, `handshake`, `share`, `megaphone`, `trade`
- **Shop**: `cart`, `sale`, `wallet`, `crate`, `piggyBank`, `coins`, `cash`, `bank`, `video`, `vip`, `pass`, `ad`
- **Rewards**: `chestOpen`, `medal`, `laurels`, `ribbon`, `lootBag`, `missions`, `target`, `sparkles`, `claimAll`, `stamina`
- **Status effects**: `burn`, `freeze`, `poison`, `bleed`, `stun`, `sleep`, `silence`, `slow`, `haste`, `regen`, `curse`, `blind`, `shock`, `ghost`, `root`, `weak`, `charm`
