# Lamp Dive — the roguelike archetype (2D)

A lamp-bearer goes two floors down a tomb, one step a turn. Every dive is
new: the rooms, the corridors, the foes and the finds come from the dive's
seed. The lamp lights only so far and only what a straight line reaches;
what it lit before stays dimly remembered. Rats and skeletons hunt the
lamp; the moss-faced guardian waits by the relic on the second floor, and
its every third strike is charged. When the lamp goes out the dive is over
(no second chances): the summary, then a new dive from nothing.

提灯下潜：提灯人一回合一步，向下走两层墓穴。每次下潜都是新的：房间、走廊、敌人和物品都由这次下潜的种子生成。灯只照得到一定距离，也只照得到直线看得见的地方；照过的地方会暗暗记住。老鼠和骷髅会追着灯光而来；苔面守卫在第二层的遗物旁等着，每第三击都是蓄力重击。灯一灭，这次下潜就结束了（没有第二次机会）：先看总结，再从头开始新的下潜。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It rebuilds the platform's retired Discover
roguelike prototype on the engine: its premise (a short dive, one life,
procedural floors, a run summary) and none of its code or art.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.3:dive:7;0.5:key_down:ArrowUp;0.53:key_up:ArrowUp;1:screenshot:dive.png;1.1:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships
```

## What it demonstrates, and the skills it exercises

- **Procedural floors from og.Rng** (og-core §2, §6): a floor's plan
  (`src/dive.ts`: rooms placed without touching, L-shaped corridors, the
  start, the stairs or the relic in the room farthest by walking distance,
  foes and finds in the others) is a pure function of an `og.Rng` stream
  forked with the dive's seed and the floor (`fork("dive-<seed>-floor-<n>")`),
  so the three systems that need it compute the same plan and a dive
  replays from its seed. A new dive draws its seed from the world's
  `og.Rng`.
- **Turns on a tilemap** (og-game §8): `assets/maps/dive.ogt` is all stone;
  each floor is carved at run time with `og.tilemap.setTile` on its ground,
  walls (an auto-tiled `bitmask16` terrain), decor and fog layers. A key
  press is one turn: a step tweened over 0.09 s, a strike, a wait or a
  draught; a press while the last action plays out waits for its turn.
- **Fog of war**: the `fog` layer lies over everything; when the lamp-bearer
  moves, every cell within the lamp's reach that a line from it reaches
  without crossing stone is cleared, cells lit before are dimmed, cells
  never lit stay dark. Foes and finds show only on lit cells.
- **Monsters on behaviour trees** (og-ai §3, §4): `assets/ai/foe.ogt` (rats,
  skeletons) and `assets/ai/guardian.ogt` run on `og.AiAgent` with
  `og.Perception` occluded by the walls' colliders (`og.Tilemap`'s
  `colliderLayers`); the trees check `Game.foesTurn`, `sees`,
  `targetDistance` and `sinceSeen`, and act through the game's commands
  (`src/foes.ts`): strike, hunt along an `og.nav.findPath` path on a grid
  rebuilt per floor, roam, brace. Each foe acts once a turn.
- **Items, permadeath and a run summary**: potions, a blade, a ward, lamp
  oil, coins and the relic; the lamp out ends the dive, the summary
  (`og.ui.modal`) says how deep, how long, what fell, what was found and
  what put the lamp out; Dive again starts from nothing.
- **Feel** (og-feel §3–§7): a strike is a lunge, a white flash, a squash,
  the number rising, a 4-tick hit stop (6 on a high roll), trauma and a
  kick toward the foe; a fall is a stretch to nothing; a hurt is a red
  flash, a squash, the red number, a hit stop and a kick; the guardian
  glows red and shows "!" the turn before a charged strike (twice the hit
  stop and trauma); its fall and the lamp going out are heavier still.
- **Screens on og.ui with a skin** (og-ui §2–§6, og-taste §1–§7): the
  engine's pixel pack recoloured as `assets/skins/lampdive` (the tomb's
  violet panels, the lamp's amber as the one accent); the title over the
  lamp-lit first room, the HUD as capsules in the corners (health meter,
  strike, armour, gold, the floor, Pause), the draught button (it shakes
  with the error sound when refused), Wait and an `og.ui.dpad` on a touch
  screen, `og.ui.pauseMenu`, a settings dialog and the summary built from
  widgets, a dissolve between screens (a fade with reduced motion).
- **Sound** (og-audio §1–§5): a voice per event (pitch-varied steps, hits,
  pickups), the UI's sounds from `og sfx` presets, a 66 BPM tomb loop on
  the music bus ducked under the effects and a reverb on the effects bus
  (`og.AudioBusFx`).
- **Languages** (og-i18n §1–§5): every string is a key of
  `assets/strings/en.ogt` and `zh-CN.ogt`; the Chinese font is Noto Sans SC
  subset to exactly the table's characters.
- **The platform** (og-platform-sdk §1–§4): settings through
  `loadSettings`/`saveSettings`; the dive record (`{dives, relics}`) read on
  tick 0 and written at the revision read; a write refused as stale
  (another device dived meanwhile) is read again and merged (the stored
  counts plus what this device added) and written at the new revision; a
  `dive.over` event.

`og polish .` (2026-10-19, og and the native addon built release from
this checkout): taste 99, feel 100, in
`tests/evidence/P2/archetypes/roguelike/` of the engine repository. With
the dev-profile og the gate builds, on a machine at load 30 to 50, the
same playtests' frame time p95 was 20 to 90 ms (median step 2 ms) and
feel 77–80: the p95 follows the machine's load, not the game
(`tests/evidence/P2/archetypes/frame-time.md`).

## Rules

- **A turn.** An arrow, WASD, the d-pad, the HUD's d-pad or a tap on the
  next cell steps there; into a foe it strikes it; into stone it bumps (no
  turn). Space (the pad's A, the HUD's Wait) waits; Q or 1 (the pad's X,
  the HUD's Drink) drinks a draught (refused at full health or with the
  flask empty, no turn). After every action but a bump or a refusal, each
  foe takes one action. A held direction repeats after 14 ticks, then
  every 7.
- **Strikes.** The lamp-bearer's strike is its attack (3) plus a die's 0
  or 1. A foe's strike is its attack less the armour, at least 1.

  | Foe | Health | Strike | Mind |
  |---|---|---|---|
  | Tomb rat | 5 | 1 | strikes beside the lamp, hunts it for 6 s after seeing it, else roams every turn |
  | Skeleton | 9 | 2 | the same mind, roams every other turn |
  | Moss-faced guardian | 18 | 3, charged 6 every third turn | strikes beside the lamp, closes in from 4 cells, else holds its ground; a charge not struck with is spent |

- **Finds.** A draught (+8 health), a blade (+1 strike), a ward (+1
  armour), lamp oil (+1 reach), coins (5–15), the relic (the win).
- **The floors.** Floor 1's stairs, in the room farthest from the start,
  go down (the foes left behind take no turn). Floor 2's relic stands in
  its farthest room, the guardian before it.
- **The end.** The lamp out (health 0) or the relic taken ends the dive;
  50 ticks later the summary. Nothing carries over to the next dive but the
  record of dives and relics.

Everything is deterministic: the clock is the fixed tick, the plans come
from `og.Rng` streams, og.ai and og.nav are deterministic and in
snapshots, and `tests/replay.og.ts` replays a dive to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a step | a 0.09 s quadOut tween, a pitch-varied footstep, the fog clearing | `Hero`, `Fog` |
| a bump | a squash against the stone, a dull thud, no turn | `Hero` |
| a strike | a lunge and back, the foe flashes white and squashes, the number in white (gold and bigger on a high roll), a 4–6-tick hit stop, trauma 0.15 (0.3 on the guardian), a kick | `Hero`, `Foes` |
| a foe felled | it stretches to nothing and fades, a chirp and a crunch (the guardian: a roar, a 12-tick hit stop, trauma 0.6) | `Foes` |
| a hurt | the foe lunges, the lamp-bearer flashes red and squashes, `-n` in red, a 3-tick hit stop (8 charged), trauma 0.25 (0.5), a kick | `strike` in `src/foes.ts` |
| a charge coming | the guardian tinted red, "!" rising over it, a double beep | `turnTaken` |
| a find | it rises and fades, its name in gold, a chime (an arpeggio for gear, a sip for a draught) | `Loot` |
| the stairs | `Floor 2` in gold, a punch, a falling rumble | `Hero` |
| the lamp out | a 16-tick hit stop, trauma 0.7, a falling tone; the summary after a beat | `strike` in `src/foes.ts`, `Ui` |
| the relic | a punch, a fanfare, the summary with a crown and a burst | `Loot`, `summary` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, the seed, the floor, the lamp-bearer's numbers and cell, the counts (turns, kills, hits, hurts, finds), the foes' turn, the queued action, the record, sound requests, the map, grid and mind handles, the lit and seen cells |
| `Tuning` | resource | the knobs below |
| `Prefs` | resource | the volumes, the language, reduced motion, the settings request |
| `Foes` | resource | where each foe stands (the Hero system reads it to strike what it walks into) |
| `Hero` | the lamp-bearer | steps, bumps |
| `Foe` | the 10 pooled foes | slot, kind, health, strike, cell, turns, the charge, the fall |
| `Loot` | the 12 pooled finds | slot, kind, coins, cell, the pickup |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `og.systems.cameras` | the camera follows the lamp-bearer (`og.Follow2D`) |
| `Ui` | the screens, starting and ending dives, the camera's height for the surface, the buses, the platform's answers |
| `Hero` | carves a floor, draws the lamp's glow, the lamp-bearer's turn (step, strike, bump, wait, draught, stairs) |
| `Foes` | places a floor's foes, lands the lamp-bearer's strike, the falls, which foes the lamp shows, the charge's tint, the board; takes the foes' turn down once og.ai has thought with it |
| `Loot` | places a floor's finds, picks them up, shows the lit ones |
| `Fog` | what the lamp lights |
| `Voices` | plays each requested sound once |

og.ai runs before them (the foes' decisions, whose commands run before the
next tick).

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `heroHp`, `heroAttack`, `potionHeal` | 20, 3, 8 | the lamp-bearer |
| `lamp` | 4.5 cells | how far the lamp lights |
| `ratHp`, `ratAttack` | 5, 1 | the rat |
| `skeletonHp`, `skeletonAttack` | 9, 2 | the skeleton |
| `guardianHp`, `guardianAttack` | 18, 3 | the guardian (charged: twice) |
| `sight` | 6 m | how far foes see |
| `stepTime` | 0.09 s | a step's tween; actions wait for it |
| `hitStop` | 4 ticks | a strike's hit stop |

## What a Maker changes first

1. **The floors**: `planFloor` in `src/dive.ts` (room sizes and count, what
   goes in which room, how many foes on floor 2); more floors are `FLOORS`
   in `src/ui.ts` and a branch in `planFloor`.
2. **The foes**: their numbers in `Tuning`, their minds in `assets/ai/`; a
   new foe is a clip in `tools/make-assets.py`, a `FoeKind`, a `stats` case
   and a mind.
3. **The finds**: `LootKind`, the `finds` lists in `planFloor`, a `take`
   case.
4. **The look**: `assets/skins/lampdive/skin.ogt`'s palette and the art in
   `tools/make-assets.py`, then `python3 tools/make-assets.py && og cook .`.
5. **Words**: `assets/strings/*.ogt`; a new Chinese string needs the
   script run again to subset the font.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `dive[:<seed>]` | starts a dive, as Dive does; with a seed, that dive |
| `descend` | goes down a floor now, as the stairs do |
| `warp:<x>:<y>` | puts the lamp-bearer on a cell (column, row from the top) |
| `hp:<n>` | sets the lamp-bearer's health |
| `give:<potion\|blade\|ward\|oil>` | a find, as picking it up gives it |
| `stale_record` | another device writes the dive record, so this one's next write is refused as stale and merged |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `strike`, `hunt`, `roam`, `brace` `:<foe>` | the foes' behaviour commands (og.ai sends them) |
| core actions | `touch_move_to:play`, `Pause`, `Drink`, `Wait`, `Pad/Up`…, `Again`, `Menu`; arrows, WASD, `Space`, `Q`, `Escape`; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the seed, the floor and
the deepest, health, strike, armour, the lamp, draughts, gold, `kills`,
`hits`, `hurts`, `finds`, `turns`, the lamp-bearer's cell, the lit and seen
cells, the win, what put the lamp out, the record and the language. The
playtests read the same values from the `Game` resource with `get:Game`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Dive by tap on a fresh seed; seed 7 walked with the arrows: a step a turn, a bump with none, the lamp lighting new cells, rats struck by walking into them, Q drinking |
| `tests/foes.og.ts` | a rat that sees the lamp hunts it and strikes beside it; a strike lands, the second fells the rat and it leaves the board |
| `tests/lose.og.ts` | the lamp out by a rat; the summary after a beat with how and where; Dive again starts from nothing (new seed, the blade gone) |
| `tests/win.og.ts` | the stairs to floor 2 (the foes left behind take no turn), the guardian fought and felled, the relic, the record merged after a stale write, the summary, Title |
| `tests/pause.og.ts` | Escape holds the dive (time scale 0, keys take no turn); settings switch to 简体中文 and move the music; Done saves through og.platform; Resume |
| `tests/pad.og.ts` | a gamepad alone starts, steps, holds a direction, waits, drinks and is refused, pauses and resumes |
| `tests/phone.og.ts` | 390 × 844 at 3×: the HUD inside the safe area, touch-sized buttons, the d-pad steps and Wait waits |
| `tests/frames.og.ts` | the title, the lamp in the corridor and the summary match `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a dive on the seed Dive picks, walked and fought, run twice, ends on one hash |

Each one fails when the behaviour it names is broken (one was broken on
purpose and restored: `tests/evidence/P2/archetypes/break-and-restore.md`
in the engine repository).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (two runs
give the same bytes): the tileset, the map, the sprite sheet and the glow,
the sounds and the music, the `og sfx` preset files of the UI's sounds and
the footstep, and the font subset. `LICENSES.md` lists them; the skin's
are in `assets/skins/lampdive/LICENSES.md`.

## Known limits (engine gaps, not design)

- og.ui's `settingsScreen` and `resultsScreen` carry English labels of their
  own, so this game builds its settings dialog and its summary from widgets
  to keep every string in its tables.
- A handle column (`og.AiAgent.behaviour`, `og.Flipbook.sheet`, …) is two
  `u32` per row (the handle's low and high halves), so a system writing one
  through its column view writes `[i * 2]` and `[i * 2 + 1]`
  (`Foes` in `src/systems.ts`); writing `column[i] = handle` lands in
  another row's halves.
- The fog is a tilemap layer the Fog system rewrites cell by cell through
  `og.tilemap.getTile`/`setTile` (one call each), since a system has no
  bulk read or write of a layer.
