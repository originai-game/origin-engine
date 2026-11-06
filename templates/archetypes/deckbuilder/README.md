# Ashen Crown — the deckbuilder archetype (2D)

A Warden (and, with two players, a Hunter beside her) climbs six steps to
the Ashen Throne with a deck of ten cards each. Every turn brings three
energy and five cards. Each card is an attack, a block, a draw, more
energy or a status (vulnerable, weak, poison). The foes show what they
will do before the heroes act. After each fight a hero takes one card of
three into the deck. At a fire a hero heals or tempers a card. The run is
won when the Ashen King falls and lost when every hero is at 0 health.

灰烬王冠：守卫（两人游戏时还有身旁的猎手）各带一副十张牌，走六步登上灰烬王座。每回合三点能量、五张手牌；牌分攻击、格挡、抽牌、能量与状态（易伤、虚弱、中毒）。敌人在英雄行动前就亮出意图。每场战斗后，每位英雄从三张牌中选一张加入牌组；篝火处可以回血或锻造一张牌。击败灰烬之王即为胜利，全员生命归零即为失败。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It is the starter of genre `roguelike-card`
(the `deckbuilder` starter in `packages/create/src/starters.ts`). The
`roguelike` archetype stays the top-down dungeon; this one takes its
project files, its pixel-art tooling (`tools/make-assets.py` with
`oglite.py`), its device settings (`src/device.ts`) and the shape of its
og-ui screens.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.5:goto:0;1.6:key_down:Space;1.63:key_up:Space;2.4:screenshot:fight.png;2.5:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships
```

## What it demonstrates, and the skills it exercises

- **A deck, piles and a hand as world state** (og-core §2, §6): a hero's
  `deck` is a list of card codes (`src/cards.ts`: an index into `CARDS`,
  plus `UPGRADED`). In a fight its `draw`, `hand` and `discard` are lists
  of indices into that deck. A shuffle is `og.Rng.permutation` and a
  foe's target is `og.Rng.integer`, both on the world's stream. So the
  first hand, every reshuffle and every reward are the same draw on every
  peer of a room, and `tests/replay.og.ts` ends two runs on one hash.
- **Intents** (Slay the Spire's rule): each foe cycles through its moves
  (`src/foes.ts`). The next move hangs over the foe as an icon and a
  number (damage after strength, weak and the target's vulnerable;
  `×n` for several hits; `+b` for block). Its tooltip says the move in
  words and, with two heroes, whom it strikes.
- **The rpg kit** (`@origingame/engine/extensions/rpg`): the turn order
  strip (heroes, then foes, the one acting marked) and a unit frame per
  hero (portrait, health, block, energy). The cards are og.ui's own
  `cardHand` (fan, hover lift, drag or tap to play, target pick, a card
  dealt in fresh), with `orbMeter` for energy and `rewardChoice` for the
  rewards, the fire and the temper.
- **Feel** (og-feel §3–§7): a played card flies from the hand to its target
  on an arc (a tween sequence on one of four pooled world sprites,
  `fly-0..3`). A hit is a lunge, a white flash, a squash, the damage
  popping up in white (gold and bigger at 12 or more), a hit stop, trauma
  and a kick. Block is a blue pop and a shimmer. Poison ticks in green on
  the foe's beat. A foe's strike is a lunge back, a red flash on the hero
  and a kick toward it. A fall stretches and fades. The boss's fall is a
  punch and the heaviest shake.
- **Screens on og.ui with a style** (og-ui §2–§6, og-taste §1–§7): the
  engine's pixel pack recoloured as `assets/styles/ashen` (dusk violets,
  one ember accent). The title and the pause are the game's screen
  documents (`assets/ui/main-menu.ogt` and `pause.ogt`). The map is a row
  of nodes with the fork as two buttons. Settings and the end summary are
  built from widgets. Screens change with a dissolve (a fade under
  reduced motion).
- **Two views, one rule set** (`src/stage.ts`): `side-2d` puts the battle
  side-on on an ash plain at dusk, heroes left and foes right on one
  ground line. `top-down-2d` puts the same battle on a felt war table
  seen from above, each unit a round token. Only the spots, the clips
  (`<unit>` or `<unit>-token`) and the camera's frame differ. The camera
  is at least 14 units tall, and taller on a portrait phone so every unit
  fits across. Under the pixel look (`assets/looks/ashen.ogt`, the
  engine's `16bit-pixel`) the Stage sets `Stylise.pixelHeight` to the rows
  that fill the surface at a whole scale, keeping 16 art pixels to a unit.
  So the frame has no border, and a phone draws the art at its own scale
  rather than crushed to 224 rows. The backdrops are drawn large enough
  (640 × 704) to cover either surface without scaling. Backdrops, card art
  and portraits are `conform ui` in their import settings: the cook keeps
  their size and applies only the palette.
- **Sound** (og-audio §1–§5): a voice per event (the card, hits, the heavy
  hit, block, hurt, poison, buffs, the draw, both turns' stings, a fall,
  the heal, the win, the loss), the UI's `og sfx` presets and a music
  loop.
- **Languages** (og-i18n §1–§5): every string is a key of
  `assets/strings/en.ogt` and `zh-CN.ogt`. That includes each card's text,
  with its numbers as `{a}` and `{b}`, drawn bold. Chinese is drawn in
  the pixel pack's CJK companion.
- **The platform** (og-platform-sdk §1–§4): settings through
  `loadSettings` and `saveSettings`, and a record of runs and wins read on
  tick 0 and written at the revision read. A room keeps neither while it
  runs.

## Rules

- **A fight.** At the start of the heroes' turn each hero up loses its
  block, gets `energy` (3) plus its Kindle, and draws `draw` (5). When the
  draw pile runs out the discard pile is shuffled in. A hand holds at
  most 10 cards. A card costs its number in energy. An unplayable card
  shakes with the error sound. End turn discards the hand. With two
  heroes, the foes act once every hero up has ended its turn. Each foe
  then acts on its beat (`foeBeat` ticks apart): its block falls, its
  poison bites (then drops by one), and it does its intent. Then the
  heroes' turn comes round again. Weak and vulnerable drop by one at the
  end of the heroes' turn.
- **Damage.** A hit is its number plus strength. Weak takes a quarter off
  (rounded down), vulnerable adds half (rounded down). Block soaks damage
  first.
- **The cards** (15 in all: a starter deck of 10 each; the rewards draw
  from the 13 that are not Strike or Defend). The numbers after the
  arrow are tempered:

  | Card | Type | Cost | Does |
  |---|---|---|---|
  | Strike | attack | 1 | 6 → 9 damage |
  | Defend | skill | 1 | 5 → 8 block |
  | Bash | attack | 2 | 8 → 10 damage, 2 → 3 vulnerable |
  | Cleave | attack | 1 | 8 → 11 damage to every foe |
  | Venom Dart | attack | 1 | 3 → 5 damage, 4 → 6 poison |
  | Toxic Cloud | skill | 2 | 6 → 9 poison to every foe |
  | Ward | skill | 2 | 10 → 14 block, the ally 5 → 7 |
  | Insight | skill | 0 | draw 2 → 3 |
  | Adrenaline | skill | 0 | 1 → 2 energy, draw 2, exhaust |
  | Flurry | attack | 1 | 3 damage 3 → 4 times |
  | Iron Hymn | power | 1 | 2 → 3 strength for the fight |
  | Heavy Blow | attack | 2 | 14 → 19 damage |
  | Rally | skill | 1 → 0 | draw 2, the ally draws 1 |
  | Sunder | attack | 1 | 7 → 9 damage, 2 → 3 weak |
  | Kindle | power | 1 → 0 | 1 more energy every turn |

  The Warden starts with 4 Strike, 4 Defend, Bash and Ward. The Hunter
  starts with 3 Strike, 4 Defend, 2 Venom Dart and Insight.
- **The foes.** Health grows by 60 % for each seat past the first.

  | Foe | Health | Moves, in turn |
  |---|---|---|
  | Cinder imp | 13 | attack 6; +2 strength; attack 4 |
  | Ash hound | 17 | attack 3 ×2; attack 8; block 6 |
  | Bog slime | 20 | attack 7; 2 weak; attack 5 and 1 vulnerable |
  | Ember knight (elite) | 46 | attack 8 and block 10; attack 14; +3 strength |
  | The Ashen King (boss) | 84 | attack 10; 5 to every hero and 1 weak; block 14 and +2 strength; attack 9 ×2 |

- **The run** (`MAP`): a fight (imp and hound), a fight (two slimes), a
  fork (a fire, or a third fight against a hound and two imps for one
  more card), the elite, a fire, the boss. After a fight each hero picks
  one card of three into its deck, or skips. A fallen hero rises with a
  quarter of its health. At a fire each hero rests (heals `restHeal`, 30 %
  of its health) or tempers one of three cards of its deck. With two
  heroes each chooses in turn.
- **The end.** The Ashen King's fall wins the run. Every hero at 0 loses
  it. The summary says how far, the fights, the cards played and the
  damage dealt. Again starts a new run. Title goes back to the title.

## Players and controls

The starter's controls (`src/creation.ts`, `src/seats.ts`): the d-pad,
aim none, `play` (Space or E, Enter or Slash, the pad's A, a tap on the
surface) and `end` (R or Backspace, the pad's Y).

- **Keys and pads.** Left and right move the card focus along the hand.
  Up and down move the target focus along the foes (a ring round the
  focused foe, in the seat's ink: ember for the Warden, blue for the
  Hunter). Play plays the focused card on the focused foe.
  End ends the turn. On the map left and right pick the path and Play
  takes it. In a reward or at a fire left and right pick and Play takes.
- **Touch.** Tap a card, then tap a foe (or drag the card onto it). A
  card without a target plays on the tap. The End turn button sits at the
  hand's right. The hand, End turn and the energy orb are sized for a
  thumb on a phone.
- **Two on one device: co-op.** Both heroes share one fight. Each has its
  own deck, hand, piles and energy. The surface shows both hands side by
  side, stacked on a portrait phone. Seat 0 plays with WASD, Space or E and
  R. Seat 1 plays with the arrows, Enter or `/` and Backspace. Pads go
  one each. A tap plays the card it lands on in that seat's hand. The
  foes wait until both have ended their turns (or the one still up). Ward
  and Rally reach the ally.
- **Online: lockstep.** Every choice goes through each seat's
  og.PlayerInput (`aimX` carries the card's deck index and the menus'
  `Ask` codes, `aimY` the foe), and every draw and target is the world's
  `og.Rng`. So each peer runs the same run from the same inputs. Each peer
  shows only its own hand, and its keys move only its own focus.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a card hovered or focused | og.ui's `cardHand` lifts and glows it | `hand` in `src/ui.ts` |
| a card played | it leaves the hand; its art flies on a 0.22 s arc (up, then down onto the target), then fades; the card's sound | `fly` in `src/run.ts` |
| a hit | the hero lunges, the foe flashes and squashes, the number pops (gold and bigger at 12 or more, the blocked part in blue), a hit stop (3 ticks longer on a heavy card), trauma | `hitFoe` |
| block | a blue `+n`, a shimmer | `gainBlock` |
| a hurt | the foe lunges, the hero flashes red, `-n` in red (blue when the block soaks it), trauma by how much was lost, a kick | `hitHero` |
| poison | a green number and flash on the foe's beat | `foeAct` |
| a foe felled | a stretch and a fade, the heaviest shake on the boss | `fell` |
| a turn | the "Your turn" or "Foes' turn" banner, its sting | `heroTurn`, `foesTurn` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, the seed, the map's step and path, the turn and its side, the foe acting and its beat, the rewards and tempers offered each seat, who has decided, the counts (fights, turns, played, dealt), the record, sound requests, the banner |
| `Tuning` | resource | the knobs below |
| `prefs` (`src/device.ts`) | this device's own, in no world | the volumes, the language, reduced motion, the settings request |
| `Hero` | a hero per seat (with its og.PlayerInput) | its seat, health, block, energy, Kindle, strength and statuses, its deck, draw, hand and discard, the fresh cards, its card and foe focus, ended, its last buttons |
| `Foe` | the three pooled foe rows | slot, kind, health, block, strength, poison, weak, vulnerable, its move and target, its fall |
| `Sfx` | one row per sound | its `Sound.*` bit |

## Systems (in tick order)

| System | Job |
|---|---|
| `Seats` | each seat's keys, pad, taps and the UI's presses to og.net.submit (`src/seats.ts`) |
| `Ui` | the screens, the HUD, the hands, the platform's answers; the cards, buttons and taps press the seat's bits and `aimX`/`aimY` |
| `Run` | every rule: the menus, the map, the turns, the cards, the foes' beats, the rewards and fires, the end |
| `Stage` | the view's spots and clips, the backdrop and the camera for the surface, the falls' fade |
| `Voices` | plays each requested sound once |

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `heroHp` | 60 | a hero's health |
| `energy` | 3 | energy a turn |
| `draw` | 5 | cards drawn a turn |
| `restHeal` | 0.3 | the fraction a fire heals |
| `foeBeat` | 42 ticks | the pause between foes acting |
| `hitStop` | 4 ticks | a hit's hit stop |

## What a Maker changes first

1. **The cards**: a row in `CARDS` (`src/cards.ts`), a case in `playCard`
   (`src/run.ts`), its words in the string table and its art in
   `tools/make-assets.py`. The starter decks and the reward pool sit
   under `CARDS`.
2. **The foes and the map**: `FOES` (health and move cycles) and `MAP`
   (the steps, the fork) in `src/foes.ts`. A new foe is a clip and a
   portrait in `tools/make-assets.py` too.
3. **The look**: `assets/styles/ashen/style.ogt`'s palette and the art in
   `tools/make-assets.py`, then `python3 tools/make-assets.py && og cook .`.
4. **The title and the pause**: `assets/ui/main-menu.ogt` and `pause.ogt`.
5. **Words**: `assets/strings/*.ogt`.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `begin` | starts a run, as Begin does |
| `goto:<step>[:<path>]` | jumps to a step of the map and takes it (0 the first fight, `2:0` the fork's fire, 3 the elite, 5 the King) |
| `hp:<n>[:<seat>]`, `energy:<n>[:<seat>]` | sets a hero's health or energy |
| `foehp:<slot>:<n>` | sets a foe's health (at least 1) |
| `give:<card>[+][:<seat>]` | puts a card (tempered with `+`) in a hero's deck, and in its hand in a fight |
| `play:<position>[:<slot>[:<seat>]]` | plays the card at a hand position on a foe slot (scripts only: a room's peers would not see it) |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `bot:<n>` | this device's seats take scripted input; 0 gives them back |

`facts` (the probe's `state()`) reports the phase, the view, the seats and
party, the step, the turn and its side, each hero (health, block, energy,
hand, pile sizes, ended), each foe (health, block, poison, vulnerable,
intent), whether the party is down, the win, the counts, the record and
the language.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Begin, the map, the first fight dealt from the seeded draw; keys move the card and foe focus, Space plays, energy is spent, the damage lands, R ends the turn, the foes act on their intents |
| `tests/rewards.og.ts` | a fight won gives three cards; one taken joins the deck; the fork's fire heals; a temper upgrades a card |
| `tests/win.og.ts` | the boss felled ends the run in a win; the summary; the record |
| `tests/lose.og.ts` | every hero at 0 loses; the summary; Again starts a new run |
| `tests/seats.og.ts` | each seat's keys, pad and taps reach its own og.PlayerInput; with two, both hands, the foes waiting for both, the bot seats |
| `tests/pause.og.ts` | Escape holds the fight; settings switch to 简体中文; the run resumes |
| `tests/phone.og.ts` | 390 × 844 at 3×: everything inside the safe area, touch-sized targets, a tapped card and a tapped foe play |
| `tests/frames.og.ts` | the title, a fight and its reward match `references/metal/` for this scaffold's view, players and platform |
| `tests/replay.og.ts` | a run with its draws and rewards, played twice, ends on one hash |

## Reference frames

`references/metal/` holds each scaffold's frames, by prefix:

| Prefix | Scaffold |
|---|---|
| `side-` | side-2d, one player, 1280 × 720 |
| `top-` | top-down-2d, one player, 1280 × 720 |
| `side-local-` | side-2d, two players on one device |
| `side-phone-` | side-2d, a phone in portrait (1170 × 2532) |

A scaffold whose prefix has no reference records its own with
`OG_FX_CAPTURE=1 og test .`.

## Assets

`tools/make-assets.py` regenerates every asset byte for byte: the units'
sheet (side-on frames and table tokens), the portraits, both backdrops
(640 × 704, so they cover a desktop or a phone at their own pixel scale),
the fifteen cards' art, the sounds, the music and the `og sfx` presets.
The string tables (`assets/strings/*.ogt`) are edited by hand, a key in
each.
`LICENSES.md` lists them; the style's are in
`assets/styles/ashen/LICENSES.md`.

## Known limits (engine gaps, not design)

- `og.ui.cardHand`, on the first frame it is drawn, lays its cards out
  across the whole viewport (its fan rectangle is not known yet). With
  two hands side by side this warns "laid out past the screen" once, and
  on a portrait phone with two players that frame puts End turn off the
  screen. The fix belongs in the engine (lay out no cards until the fan
  rectangle is known). The game does not work around it.
- Card and foe focus are the game's own state on `Hero`, not og.ui's
  focus navigation. og.ui's focus is per device, and with two seats on
  one device each hand needs its own focus.
