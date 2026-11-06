# Greywatch Keep — the rpg archetype (3D)

The dead have risen in the yard of a border keep. Castellan Maren waits by
the gate; talk to her, and her branching conversation gives the quest:
put down the four risen and their captain, take the warden's seal from the
chest by the crypt door, and bring it back to her. The risen walk the
keep's navmesh and think with og.ai machines: they see you, hunt you and
wind up their blows. Strike back with the sword, drink a draught when it
goes badly, pick up what they drop, and level up on what you put down.
Handing Maren the seal wins the run; the party falling loses it. Then the
summary, and again.

灰望要塞：亡者在一座边境要塞的庭院中复苏。城堡主玛伦守在大门旁；和她交谈，她的分支对话会给出任务：消灭四个复苏者和它们的队长，从墓门前的宝箱里取出守墓人的印章，再交还给她。复苏者沿要塞的导航网格行走，由 og.ai 状态机思考：看见你、追击你、蓄力出手。用剑反击，危急时喝下药水，拾取它们掉落的东西，靠击倒的敌人升级。把印章交给玛伦即获胜；队伍全部倒下则失败。随后是总结，再来一次。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.5:run;1:quest:clear;2.2:warp:foe1;2.25:key_down:Space;2.3:key_up:Space;2.45:screenshot:blow.png;2.5:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og build . --publish         # dist/: the play page as it ships (behind "tap to play")
```

## What it demonstrates, and the skills it exercises

- **Dialogue and quests as data** (og-story): `assets/story/maren.ogt` is
  Maren's conversation (a greeting with three replies, what happened and
  who she is, the offer with its pay and a refusal she remembers, a
  reminder that counts the risen left, the hand-over); `seal.ogt` is the
  quest (stage `clear`: four risen and the captain; `fetch`: the seal from
  the chest; `report`: back to Maren). The game starts conversations,
  passes the seats' replies to `og.story.choose` and `advance`, counts
  objectives with `og.story.progress`, and reads `og.story.view`,
  `og.story.quest` and `og.story.quests` for the HUD.
- **The rpg kit** (`@origingame/engine/extensions/rpg`): the party's
  `partyFrames` (alone, one large `unitFrame`), the last struck risen's
  `unitFrame` at the top, the `questTracker`, Maren's `conversation` as a
  box with numbered replies, and over the pause the bag: the
  `inventoryGrid` of the party's things and the `questLog`.
- **Foes on a navmesh with og.ai** (og-ai): `assets/nav/keep.ogt` bakes the
  yard from the same boxes the heroes bump into (`tools/make-nav.ts` from
  `src/layout.ts`), with the gate's ward as an area the risen's
  `og.NavFilter` leaves out; each risen is an `og.NavAgent` with
  `og.Crowd`, `og.Perception` (a 150° cone, the walls and pillars hide the
  party) and a machine (`assets/ai/minion.ogt`, `captain.ogt`: idle and
  wander, hunt, strike, search) whose `alarm` and `strike` commands the
  game answers.
- **Melee with feel** (og-feel, og-anim): the knight's swing is the attack
  clip retargeted onto KayKit's Rig_Medium, its blow on the clip's `hit`
  marker (`og.AnimationEvent`) on every risen in reach and arc, with a
  little aim help; a hit stop, the risen's white flash and squash, a
  camera kick and shake, sparks, the damage number, its flinch and a step
  back. A risen's blow: a red glint gathering in its chest first, then the
  clip; on the hero a red flash, a screen flash, a shake and a knock-back.
- **Characters** (og-model, og-anim): KayKit's Knight, Barbarian, Mage,
  Skeleton Minion and Skeleton Warrior, each merged with its weapons
  (`tools/compose.py`) and given the clip library through a rig file;
  each body an `og.AnimationGraph` (idle, walk and run by speed) with an
  `og.AnimationPlayer` over it for the one-shots.
- **The keep** (og-render): KayKit Dungeon Remastered walls, arches, the
  gated door, pillars, crates, barrels, banners and torches (fire, smoke
  and a flickering light each), the far walls whole and the near ones a
  cut-away parapet so nothing hides the party; a low evening sun, fog,
  bloom, AO and a vignette.
- **Screens** (og-ui): the title over a slow orbit of the keep and the
  pause as the game's screen documents (`assets/ui/`), the HUD, the bag,
  the settings and the summary, in `assets/styles/greywatch` (the engine's
  fantasy pack); every string a key of `assets/strings/en.ogt` and
  `zh-CN.ogt`.
- **Sound** (og-audio): Kenney's swings, blows, bones and steps, `og sfx`
  presets for the interface, the draught, coins, the level-up and the
  hurt, the yard's music ducked under the effects with a stone echo.
- **The platform** (og-platform-sdk): the settings saved and restored (this
  device's own, `src/device.ts`, never in the world), the record written at
  the revision read and merged when another device wrote first (outside a
  running room only: a platform answer is one peer's alone), a `run.over`
  event.

## Views

`src/creation.ts`'s `view` (`src/view.ts`, the same file in every 3D
starter) picks the camera, and all three play the same keep:

- **isometric-2.5d** (the default): an orthographic camera at the true
  isometric angle following the hero. The stick walks the screen's way (up
  is up the screen); the look (the mouse while the pointer is locked, the
  right stick, a finger dragged across the right half) turns the hero, and
  a hero that walks without it faces where it walks.
- **third-person-3d**: a follow camera behind each hero, turned by its
  look; the stick walks where it looks.
- **first-person-3d**: the camera at each hero's eye, its own body left
  out and its blade drawn before the eye, swung through each strike.

## Seats: alone, two on one device, two over a room

Every hero is a seat (`src/seats.ts`, the same file in every starter): the
game reads `og.PlayerInput` alone, so the keep plays

- **alone**: seat 0 (the knight, Aldric) from both sets of keys, any pad,
  the mouse and the touch controls;
- **local-multiplayer**, as a party: the barbarian (Brenna) on the arrows,
  Enter, `.` and `,` (or pad 2) beside the knight on WASD, Space or J, E
  and Q (or pad 1). **The screen splits in halves**, each its hero's own
  camera of the chosen view (an `og.CameraView` per seat): two heroes
  fighting in different corners of the yard each keep their framing and
  their own look, which one shared isometric frame could not give the
  first- and third-person views. On a touch screen each seat has its own
  corner of controls and its own half to drag the look in;
- **online-lockstep**: two peers over a room, each camera on its own hero
  (`og.Lockstep.localPlayer`), every peer running the whole keep.

The party shares the bag (gold and draughts), the quest and the
experience: every risen put down gives its experience to each standing
hero. A fallen hero rises at the gate after `Tuning.respawn` while the
other stands; both down ends the run. Each risen hunts the nearest hero it
sees. Menus act through the seats: Enter the keep and Again press
`choice1`, Leave the keep and Title press `menu` (a control with no key),
Pause, Resume and the settings' Done and the bag's Close press Pause,
paused the Bag presses `choice2` and Settings `choice3` (on the title too),
and a reply tapped or clicked in the conversation presses its `choice1` to
`choice3`, so a room acts on one tick on every peer. Conversation lines show whole for the same reason (a
typewriter's pace is one device's own).

## Rules

- **Moving.** 4.2 m/s, slower in a swing's first part; the heroes cannot
  walk through the risen, the walls or the props.
- **Attack** (Space, J, the left button, pad A, the touch Attack; seat 1
  Enter, `/`): a 0.5 s swing; its blow lands on every standing risen
  within 2.1 m and 75° of where the hero faces, for 12 damage and 4 more a
  level, turning first to the nearest risen within reach. An Attack
  pressed during a swing waits a quarter of a second for the next.
- **Potion** (Q, pad Y, the touch Potion; seat 1 `,`): one of the party's
  draughts heals 55. The party sets out with two; the risen drop more.
- **Talk** (E, pad X, the touch Talk; seat 1 `.`): by Maren, her
  conversation (Talk or Attack moves a line on, 1 to 3 or a tap pick a
  reply); by the chest, it opens once the yard is clear. A hero stands
  still while it talks.
- **The risen.** A minion has 34 health, strikes for 9 within 1.35 m after
  a 0.3 s wind-up; the captain has 120, slams for 18. They see 9 m in a
  150° cone and never enter the gate's ward, the torchlit corner by Maren.
- **Loot and levels.** Each risen drops gold, and a draught 40 % of the
  time (the captain both); walking over it picks it up. 60 experience for
  level 2, 60 more for each level after, to level 5; a level adds 20
  health and heals the hero whole.
- **The end.** The seal handed to Maren: the crypt is sealed (a win; the
  best time is kept). Every hero down: the party has fallen. 1.5 s later
  the summary; Again starts a run from nothing, Title goes back.

Everything is deterministic: the fixed tick, og.ai, og.nav, og.story and
the character controller are in snapshots, and every random drop draws
from og.Rng.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a swing | the attack clip, a whoosh; the hero turns to the nearest risen in reach | `step` in `src/heroes.ts` |
| a blow lands | a 4-tick hit stop, the risen's white flash and squash, sparks, the number, a camera kick and shake, its flinch and step back, a blow and (on the captain) a clang | `blow` in `src/heroes.ts`, `strikeFoe` in `src/foes.ts` |
| a risen falls | its fall clip, a longer hit stop, a camera punch, bones; its drop bobbing and glowing; it sinks into the flagstones | `fall` in `src/foes.ts` |
| a risen strikes | a red glint gathering in its chest as it turns to its mark, then the clip, the captain's slam shaking the camera | `Foes` in `src/foes.ts` |
| the hero is struck | its red flash, a red screen flash, a shake, a short hit stop, a knock-back, the number | `hurtHero` in `src/heroes.ts` |
| a draught | green motes rising, the healing number, a rising chime | `drink` in `src/heroes.ts` |
| a level | a ring of gold sparks, the fanfare, "Level 2!" | `award` in `src/heroes.ts` |
| the quest moves on | a bell; the tracker ticks; a toast once no one is talking (a conversation that opens dismisses it) | `Story` in `src/story.ts` |
| the end | the stinger, the summary after a beat with its icon popping in and, on a win, a burst | `clock`, `summary` in `src/ui.ts` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase and ending, the record and its requests, the quest's stage (mirrored from og.story), the party's bag (gold, draughts, the seal), the conversation on screen and its talker, the last struck risen, the counts, sound requests, the heroes', Maren's and the navmesh's handles |
| `Tuning` | resource | the knobs below |
| `prefs` | `src/device.ts`, not in the world | the volumes, the language, reduced motion: this device's own, so no peer's settings enter a room's hash |
| `Hero` | each hero's capsule | its seat, body, last tick's buttons, health, level, experience, facing, its swing, hurt, spare and knock-back times, fallen and its rise, what Talk acts on, steps, swings, kills |
| `Foe` | the five risen | index, kind, health, its paces and reach (its behaviour reads them), its strike and its blow, its reel, its fall |
| `Loot` | eight pooled drops | lying or not, gold or a draught, how much, when it dropped |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |
| `SeatLook` | each hero | its look's heading and pitch (`src/view.ts`) |

The story (variables, conversations, the quest) is og.story's own state.

## Systems (in tick order)

| System | Job |
|---|---|
| `Seats` | each seat this device plays: its keys, pad, mouse-look and touch controls to `og.PlayerInput` |
| `ViewRig` | each seat's look turned by its aim; the cameras placed (`src/view.ts`) |
| `og.systems.cameras` | the follow and first-person cameras, the camera's punch and shake |
| `TitleCamera` | the title's orbit of the keep; the run's camera given back after it |
| `Ui` | the screens, the seats' choices (enter, pause, leave), the run's clock and end, the hold, the buses, the platform's answers |
| `Heroes` | each hero from its input: moving by its view, the swing and its blow, the draught, Talk and the replies, its fall and rise, the graph and the steps |
| `Foes` | the risen's sight, their strikes' wind-ups, clips and blows, their reel, their fall and sinking; the loot bobbing and picked up |
| `Story` | the conversation's end, the quest's stage with its toast, the win |
| `Keep` | Maren turning to her talker, the torches' flicker |
| `Voices` | plays each requested sound once |

og.nav, og.ai (whose `alarm` and `strike` commands run before the next
tick), physics and animation run in the engine's own stages.

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `walkSpeed` | 4.2 m/s | the heroes |
| `swing`, `damage`, `damagePerLevel`, `reach`, `arc` | 0.5 s, 12, 4, 2.1 m, 75° | the sword |
| `heal` | 55 | a draught |
| `health`, `healthPerLevel`, `levelXp` | 100, 20, 60 | the heroes' growth |
| `respawn` | 6 s | a fallen hero's rise while the other stands |
| `hitStop` | 4 ticks | a blow's hit stop |
| `minionHealth`, `captainHealth`, `minionDamage`, `captainDamage` | 34, 120, 9, 18 | the risen |
| `minionXp`, `captainXp` | 30, 90 | their experience |
| `sight` | 9 m | how far the risen see |
| `potionChance` | 0.4 | a minion's draught drop |

## What a Maker changes first

1. **The words and the branches**: `assets/story/maren.ogt` (nodes,
   replies, conditions on the quest) and `seal.ogt` (stages, objectives,
   rewards); the lines are keys of `assets/strings/*.ogt`.
2. **The yard**: `src/layout.ts` (walls, props, torches, where the risen
   rise, the gate's ward), then `bun tools/make-nav.ts && og cook .`.
3. **The fight**: `Tuning` (damage, reach, health, experience), the risen's
   minds in `assets/ai/`, their wind-ups and strikes in `src/foes.ts`.
4. **The characters**: another KayKit or CC0 humanoid through
   `tools/compose.py` and a `.glb.ogt` naming `rigs/kaykit.ogt`.
5. **The HUD and the bag**: the rpg kit's calls in `src/ui.ts`; `og ui add
   rpg` copies the kit into `assets/ui-kits/rpg/` to edit it.
6. **The title and the pause**: `assets/ui/main-menu.ogt` and `pause.ogt`.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `run` | starts a run now, as Enter the keep does |
| `warp:<maren\|chest\|gate\|foe<i>>` / `warp:<x>:<z>` | puts seat 0's hero by Maren, the chest, the gate or a risen (foe0 to foe4), facing it, or at x, z |
| `warp_seat:<seat>:<place>` | the same for any seat |
| `talk` | seat 0's hero starts Maren's conversation wherever it stands |
| `quest:<clear\|fetch\|report>` | the quest given; every risen slain; the seal in the bag too |
| `slay:<risen>` | a risen (0 to 4; 4 the captain) falls now, as to seat 0's blow |
| `hp:<seat>:<n>` | sets a hero's health (0 fells it) |
| `potions:<n>` | sets the party's draughts |
| `bot:<n>` | a scripted player drives this device's seats (the room test) |
| `stale_record` | another device writes the record, so this one's next write is refused as stale and merged |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| `alarm:<risen>`, `strike:<risen>` | the risen's behaviour commands (og.ai sends them with the risen's entity) |

`facts` reports the phase, the ending, the quest's stage, the risen slain,
standing and hunting, the bag, the conversation's node and line, the
counts, the record, the language and each hero's health, level,
experience, place and facing.

## Playtests (`og test .`)

Every playtest reads the seats, the view and the input from
`src/creation.ts`, so each scaffold checks what it plays.

| File | Proves |
|---|---|
| `tests/seats.og.ts` | alone: 1, W (the view's way), the pad, Space, the touch stick, a look drag (on the isometric view turning the hero) and the touch Attack reach seat 0's `og.PlayerInput`; two on one device: seat 1's keys and pad 2 play the barbarian alone, seat 0's the knight |
| `tests/flow.og.ts` | Enter the keep by tap; Talk by Maren, her replies branching to the offer, agreeing starts the quest (the tracker shows it), the hero stands still while she talks, her reminder counts the risen left |
| `tests/fight.og.ts` | swings land (the risen's health, the hit stop, its frame), three put a minion down (the quest counts it, the experience, a gold drop picked up), a second brings level 2, Q drinks a draught; another risen hunts and strikes |
| `tests/win.og.ts` | the seal handed over completes the quest and wins after her thanks; the record merged after a stale write, a `run.over` event, the summary, then Again from nothing |
| `tests/lose.og.ts` | a hero struck to nothing falls; alone that ends the run, with two the fallen rises while the other stands and both down ends it; Title goes back |
| `tests/pause.og.ts` | Escape holds the run; the bag shows the inventory grid and the journal; Settings switch to 简体中文 and save through og.platform; Resume |
| `tests/phone.og.ts` | 1170 × 2532: the title and HUD inside the safe area, touch-sized controls apart, the stick walks, Talk by Maren, a reply by tap, the line moved on by tap |
| `tests/frames.og.ts` | the title, a blow landing and Maren's conversation match `references/<tier>/<frame>-<view>[-local][-phone].png` (`OG_FX_CAPTURE=1 og test .` records them) |

## Assets

`LICENSES.md` lists every file. `tools/make-assets.py` regenerates the
music, the stingers and the `og sfx` presets; `tools/make-nav.ts` the
navmesh's bake file; `tools/compose.py` the characters (from the KayKit
packs' glTFs).
