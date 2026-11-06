# Saltwind Isle — the open-world archetype (3D)

Morning breaks on an isle in an endless sea. Five landmarks stand on it:
a watchtower, a windmill, a chapel and, across a deep strait on the north
isle, an old well and a lighthouse. The explorer sets out from the camp on
the south beach; coming within sight of a landmark discovers it (its name
on the compass and the map), and reaching it claims it: its flag rises
and its brazier catches. The strait is too deep to wade. Fell trees for
wood and break boulders for stone (hold Gather), carry them in the bag and
build the bridge at the strait's landing. Every landmark claimed before
the third nightfall lights the isle and wins the run; the third nightfall
ends it. Deep water is the soft fail: the tide carries the explorer back
to the camp and half the bag with it. Then the summary, and again.

盐风岛：晨光照在无边大海中的一座岛上。岛上有五处地标：瞭望塔、风车、礼拜堂，以及深海峡对岸北岛上的古井和灯塔。探险者从南岸的营地出发；走到能看见地标的距离就会发现它（罗盘和地图上出现它的名字），走到地标跟前就占领它：旗帜升起，火盆燃起。海峡太深，无法涉水。按住采集砍树得木材、敲碎巨石得石料，装进背包，在海峡岸边建起桥。在第三次夜幕降临前占领全部地标就点亮了全岛，赢下这一局；第三次入夜则本局结束。深水是温和的失败：潮水把探险者冲回营地，还卷走背包里一半的东西。随后是总结，再来一次。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It is the open-world slice beside Ember Vale
(`templates/archetypes/third-person`): the same terrain, atmosphere and
character stack, but a larger map that is explored rather than defended,
with gathering, an inventory, a build that opens a new area, quests, a
map and a compass, and a run of three days and two nights.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.6:run;0.8:warp:tree;1:key_down:KeyE;2.4:key_up:KeyE;2.5:screenshot:felled.png;2.6:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships (behind "tap to play")
```

## What it demonstrates, and the skills it exercises

- **A larger terrain in an endless sea** (og-terrain, og-render §water):
  `assets/terrain/isle.ogt` is a 256 m heightmap (the main island's
  highland, the watchtower's hill, the windmill's meadow, the camp's
  beach; the north isle and the lighthouse's crag across a strait 20 m
  wide) splatted from a control map with grass, the paths, rock and sand,
  with its heightfield collider. `og.Water` is `endless`, feeling the
  terrain as its `seabed`: the waves shoal and break into foam on every
  beach, the shallows are green and the strait dark.
- **The atmosphere and a day-night cycle** (og-render): `og.Sky`, height
  fog tinted by `og.DayNight`, a shadowed sun, the moon, bloom, AO, a
  vignette and a light teal-orange LUT. The run's clock drives the hour:
  110 s of daylight from 7:00 to 19:30, 22 s of night to the next 7:00,
  three days. At night the explorers' lanterns and the claimed braziers
  light the dark, and a cool moonlight fill (`og.Environment`'s hemisphere
  light, faded in after 19:00 and out by 7:00) keeps the explorers' and
  the trees' sides readable where the night sky's own ambient is nearly
  black and the moon lights only what faces up.
- **A character on og.CharacterController** (og-physics): a capsule with a
  slope limit, step height, ground snap and coyote time; the move walks
  relative to the seat's view (WASD or the arrows, the left stick, the
  touch stick); Shift, a full stick or Run runs; Space, A or Jump jumps;
  the shallows slow to `Tuning.wade` and splash.
- **Three views that really play** (`src/view.ts`, the same file in every
  3D starter): third person (`og.FollowCamera` behind the explorer, the
  look turning it, swinging behind a walking explorer, kept above the
  ground and the sea by `Eye`), first person (`og.FirstPersonCamera` at
  the eyes, the explorer's own body on a layer its camera skips, its
  shadow still falling) and isometric (`og.FollowCamera` at the look's
  fixed angle, orthographic; the right stick turns the explorer).
- **Skeletal animation and IK** (og-anim): the strider plays an
  `og.AnimationGraph` (`src/anim.ts`: idle, walk and run by speed, an air
  state); gathering raises an `og.IkChain` (FABRIK, three bones) on the
  right arm towards the tree, the boulder or the bridge's landing.
- **Gathering, the bag and a build** (`src/work.ts`): every explorer
  holding Gather at a tree or a boulder adds its time (two together work
  twice as fast); a tree falls into two wood and a stump, a boulder breaks
  into two stone and rubble, each with chips and a sound, and both grow
  back at the next dawn. Six wood and four stone build the bridge: its
  planks rise into place one after another and its deck becomes solid.
- **Quests, an inventory, a map and a compass** (the `rpg` kit and og.ui):
  `assets/story/lights.ogt` (claim the five) and `bridge.ogt` (gather the
  wood and the stone, build the span) are `og.story` quests, world state
  reset with the run, shown by `rpg.questTracker` with the distance to
  the next landmark; the bag is `rpg.inventoryGrid`; `og.ui.minimapView`
  draws `assets/maps/isle.png` with the camp, every discovered landmark,
  the bridge's site and each explorer; `og.ui.compass` along the top shows
  the bearings, the next landmark and the bridge with their distances.
- **Discovery feedback** (og-feel): a toast and a chime when a landmark is
  first seen; on a claim the flag rising, the brazier's fire, smoke, a
  burst of sparks and its light swelling in, a camera punch, a toast and
  the quest's count; a shake when the bridge stands; the dawn and night
  stingers with their toasts, and a warning as the last day ends.
- **Screens on og.ui with tokens and MSDF text** (og-ui, og-taste): the
  engine's cartoon pack as `assets/styles/saltwind`; the title over an
  orbit of the isle in the afternoon and the pause as the game's own
  screen documents (`assets/ui/main-menu.ogt`, `pause.ogt`), the HUD
  (the day and hour, the landmarks, the bag top-left; Pause, the map, its
  zone and the quests top-right; the compass top-centre; the prompt at the
  foot; on a touch screen a stick, Gather, Jump and Run), toasts, a
  settings dialog and the summary built from widgets.
- **Sound** (og-audio): a voice per event (pitch-varied steps, splashes,
  axe and pick strokes; the pickup, the claim, the discovery, the build,
  the collapse, the dawn and night stingers), the surf, the UI's sounds
  from `og sfx` presets and the isle's loop on the music bus.
- **Tune**: every `Tuning` field is tunable.
- **Languages** (og-i18n): every string is a key of `assets/strings/en.ogt`
  and `zh-CN.ogt`; Chinese is drawn in the style's fallback font.
- **The platform** (og-platform-sdk): settings through
  `loadSettings`/`saveSettings`; the record (`{runs, wins, best}`) read on
  the first tick and written at the revision read, one call at a time; a
  write refused as stale is read again, merged and written at the new
  revision; a `run.over` telemetry event.

## Budget: what a phone draws at 60 fps

The isle is sized and dressed for a phone, not a desktop:

- **One terrain of 257 × 257 samples** (256 m at a sample a metre, the
  density of Ember Vale's 129 × 129), drawn through the terrain's LOD, so
  what is drawn stays bounded as the map grows; **four layers** (grass,
  path, rock, sand) from one 512² control map, with **no displacement**
  and **no `og.Grass`**.
- **One `og.Water`** of 96 segments over the 256 m square, endless past
  it.
- **Shadows on the sun alone**: the lanterns and braziers are point lights
  without shadows.
- **Low-poly props**: 74 trees, 40 woods, 26 boulders and the buildings
  are KayKit models sharing one 1 k texture atlas; the cook warns about
  a few that exceed its per-prop face budget, which is harmless at this
  count.
- Fog closes the far shore, so the far plane (600 m) is rarely reached.

Not yet measured on a phone: Ember Vale's README states no figure, and
this archetype's frame time on a real phone waits for the compatibility
phase (Android on the Pixel, iPhone).

## Seats: alone, two on one device, two over a room

Every explorer is a seat (`src/seats.ts`, the same file in every starter;
`src/creation.ts` holds the creation choices and the controls): the game
reads `og.PlayerInput` alone, so the isle plays

- **alone**: seat 0 from the keys (both sets), any pad, the mouse while the
  pointer is locked, and on a touch screen the game's own HUD (the stick
  bottom-left; Gather, Jump and Run bottom-right; a drag on the right half
  turning the view);
- **local-multiplayer**: a second explorer (tinted) on the arrows, Right
  Shift, Enter and `/` (or pad 2) beside seat 0 on WASD, Left Shift, Space
  and E (or pad 1); the screen splits in halves (an `og.CameraView` per
  seat), each half with its own compass and prompt; on a phone each seat
  has its own stick and buttons in its own bottom corner and drags the
  view in its own half;
- **online-lockstep**: two peers over a room, each camera with its own
  explorer (`og.Lockstep.localPlayer`), every peer running the whole isle.

It is co-op: the bag, the landmarks, the bridge and the clock are shared;
two at one tree or the landing work it together. Menus act through the
seats too: Set out and Again press Jump, Leave the isle and Title press
Menu, Pause and Resume press Pause, so a room acts on one tick on every
peer. `bot:<n>` drives this device's seats from a script (the room test);
`warp_seat:<seat>:...` moves any seat's explorer.

## Rules

- **Moving.** Walk 3.4 m/s, run 6.8 m/s, wade at half either. Water over
  1 m deep is out of the explorer's depth: the tide carries it back to the
  camp and half the wood and half the stone (rounded down) with it.
- **Discovering and claiming.** Within 42 m a landmark is discovered;
  within 8 m of its centre it is claimed.
- **Gathering.** Within 2.6 m of a tree, holding Gather fells it in 1.1 s
  (two wood); a boulder breaks in 1.4 s (two stone). Letting go loses the
  progress. What is gathered grows back at the next dawn.
- **The bridge.** At the strait's landing with 6 wood and 4 stone, holding
  Gather builds it in 2.4 s; with less, the prompt says what it needs.
- **The end.** All five claimed: every light burns on the isle (a win; the
  best time is kept). The third nightfall (330 s of daylight and 44 s of
  night played): the third night fell. 1.5 s later the summary; Again
  starts a run from nothing, Title goes back. Only the record carries over.

Everything is deterministic: the clock is the fixed tick, the character
controller, the animation and og.story are in snapshots, and
`tests/replay.og.ts` replays a run (moving, a jump, a tree felled, the
bridge built, a claim) to the same hash, or with a room plays two peers to
one hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| walking, running | the blend from idle through walk to run by speed, a footstep a stride (pitch-varied) | the `Hero` system in `src/hero.ts`, `src/anim.ts` |
| wading | slower, a splash at each stride; out of depth, the collapse sound, the toast and the walk back from camp | `Hero` |
| a jump | the air state cross-fades in, a jump sound; a landing in the shallows splashes | `Hero` |
| gathering | the explorer stands and the hand reaches out (IK), a stroke a third of a second with chips and a sound; the tree falls into a stump, the boulder into rubble, a pickup | `Work` in `src/work.ts` |
| the bridge | the planks rise one after another, a build sound, a shake, a toast | `Work` |
| a landmark seen | a chime, a toast, its name on the compass and the map | `Work`, `Ui` |
| a landmark claimed | the flag rises, fire, smoke and a burst of sparks, the light swelling in, a camera punch, a toast and the quest's count | `claim` in `src/work.ts` |
| dusk, night, dawn | the sun lowers and the fog warms, then the dark, the lanterns, the night stinger; dawn's stinger and the woods back | `clock` in `src/ui.ts`, `setHour` in `src/world.ts` |
| the end | the summary after a beat | `clock`, `summary` in `src/ui.ts` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase and ending, pause and settings, the run's time, day and hour, the claimed and discovered landmarks, the bag and the totals, the bridge and its building time, the counts (washes, jumps), seat 0's target, gathering, wading and speed (for the facts), the sound requests, the explorers' and the terrain's handles |
| `Tuning` | resource | the knobs below |
| `Hero` | each explorer's capsule | its seat, body, reach, lantern and splash, last tick's buttons, facing, its target and node, gathering, wading, speed, air time, steps |
| `SeatLook` | each explorer | its view's yaw and pitch (`src/view.ts`) |
| `Site` | the five landmarks | index, claimed, when, discovered |
| `Node` | every tree and boulder | index, kind, felled, progress, the tick last worked, radius, its model and stump |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

The settings (volumes, language, reduced motion) and the record are this
device's own (`src/device.ts`), never in the world.

## Systems (in tick order)

| System | Job |
|---|---|
| `Seats` | each seat this device plays: its keys, pad, mouse-look and touch HUD to `og.PlayerInput` |
| `Device` | this device's platform answers (the settings, the record): it writes no shared state, so a lockstep session lets it hear them |
| `ViewRig` | each seat's look turned by its input, the cameras placed for the view |
| `Eye` | keeps each third-person camera above the ground and the sea |
| `og.systems.cameras` | the camera's controllers, punch and shake |
| `Ui` | the screens, the seats' choices (start, pause, leave), the clock and its ending, the HUD, toasts, the buses |
| `Hero` | each explorer from its input: the controller, the graph, the reach, steps, splashes, jumps, the tide, the lantern |
| `Work` | gathering (summed over the explorers), regrowth at dawn, the bridge, discovery and claiming, the flags and braziers |
| `Voices` | plays each requested sound once |

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `walkSpeed`, `runSpeed` | 3.4, 6.8 m/s | the explorer |
| `wade` | 0.5 | the speed in the shallows, as a fraction |
| `jumpSpeed` | 5.4 m/s | the jump |
| `daySeconds`, `nightSeconds` | 110 s, 22 s | the day and the night; three days end the run |
| `reach` | 2.6 m | how near a tree or a boulder an explorer gathers |
| `chopSeconds`, `breakSeconds`, `buildSeconds` | 1.1, 1.4, 2.4 s | how long gathering and the bridge take |
| `claimRadius`, `sight` | 8 m, 42 m | how near a landmark claims it, and discovers it |

## What a Maker changes first

1. **The isle**: `tools/make-assets.py` (the hills, the strait, the paths,
   where the landmarks, the bridge, the trees and the boulders stand), then
   `python3 tools/make-assets.py && og cook .`; it writes `src/layout.ts`,
   the heightmap, the control map and the minimap together.
2. **The pressure**: `daySeconds`, `nightSeconds` and `DAYS`, the
   bridge's `BRIDGE_COST` and the yields in `src/components.ts`, the claim
   and sight radii.
3. **The landmarks**: `BUILDINGS` in `src/world.ts` (another KayKit model,
   its scale and collider), their names in the strings.
4. **The look**: the post stack, fog and sky in `buildIsle`, the water's
   colours, the style's palette in `assets/styles/saltwind/style.ogt`.
5. **The quests**: `assets/story/*.ogt`; the tracker shows what they say.
6. **Words**: `assets/strings/*.ogt`.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `run` | starts a run now, as Set out does |
| `warp:<site>` / `warp:tree` / `warp:rock` / `warp:bridge` / `warp:camp` / `warp:<x>:<z>` | puts seat 0's explorer 13 m short of a landmark (0 to 4) facing it, by the tree or boulder nearest the camp, at the bridge's landing, at the camp, or on the ground at x, z |
| `warp_seat:<seat>:<where>` | the same for any seat |
| `give:<wood>:<stone>` | puts wood and stone in the bag |
| `claim:<site>` | claims a landmark now (the fifth wins) |
| `clock:<seconds>` | jumps the run's clock (the third nightfall at 374 s) |
| `bot:<n>` | a scripted player drives this device's seats (0 hands them back) |
| `stale_record` | another device writes the record, so this one's next write is refused as stale and merged |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| core actions | `touch_move_to:Play`, `Pause`, `Stick`, `Gather`, `Jump`, `Run`, `Again`, `Menu`, `TouchSeat1/Stick`; WASD, the arrows, Shift, Space, E, `/`, Escape; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the ending, the day,
hour and time, the landmarks claimed and seen, the bag, the bridge,
washes, seat 0's target, gathering, wading, grounded and speed, jumps,
the record, the language and the seats. The playtests read the same
values from the `Game` resource with `get:Game`.

## Playtests (`og test .`)

Every playtest reads the scaffold's choices (`src/creation.ts`) and plays
every view and player count.

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Set out by tap; day 1 at 7:00; walking 3.4 and running 6.8 m/s; the prompt at a tree, E held fells it into the bag (two wood, in the inventory); the chapel discovered (its toast), then claimed by walking in (the HUD, the toast and the quest count it) |
| `tests/bridge.og.ts` | at the landing the prompt says what the bridge needs; the deep strait washes the explorer back with half the bag; six wood and four stone and E held build it, paid from the bag, the deck risen; the explorer walks across dry onto the north isle |
| `tests/win.og.ts` | five landmarks claimed: the isle lit, the record merged after a stale write with a `run.over` event, the summary, then Again from nothing |
| `tests/lose.og.ts` | nightfall and its toast, the next dawn regrowing the woods, the third nightfall ending the run, its summary, Title with the isle reset |
| `tests/pause.og.ts` | Escape holds the run (time scale 0, the clock stopped, keys move nothing); settings switch to 简体中文 and move the music; Done saves through og.platform; Resume |
| `tests/phone.og.ts` | 1170 × 2532 (390 × 844 at 3×): the HUD inside the safe area, touch-sized controls, the stick walks, a drag turns the view, Jump, Run, Gather held fells a tree |
| `tests/seats.og.ts` | the keys, a pad and the touch HUD reach seat 0's (and with two on one device seat 1's) `og.PlayerInput` and move only that seat's explorer along its own view |
| `tests/replay.og.ts` | a run with movement, a jump, a tree, the bridge and a claim, run twice, ends on one hash; online, two peers in a room hash the same at tick 600 |
| `tests/frames.og.ts` | the title, a landmark claimed and the first night by it match `references/<tier>/<view>-<seats>-*.png` (`phone-…` in portrait; `OG_FX_CAPTURE=1 og test .` records them) |

## Assets

`tools/make-assets.py` regenerates the isle's heightmap, control map and
minimap, `src/layout.ts`, the item icons, the sounds and music and the
`og sfx` presets byte for byte; the character, the clip library, the
textures and the fire are the engine's own CC0 assets; the trees, rocks,
tent, flags and buildings are KayKit's CC0 Medieval Hexagon Pack.
`LICENSES.md` lists every file; the style's are in
`assets/styles/saltwind/LICENSES.md`.

## Known limits (engine gaps, not design)

- `og.DayNight` advances on unscaled ticks, so a pause would not hold it;
  the game holds it (speed 0) and sets the hour itself (`setHour` in
  `src/world.ts`).
- The style's Fredoka font lacks "★", so the summary marks a new best in
  words.
- Under a look, the cook conforms a style's `atlas.png` as the style's
  (its layout kept) but not its `atlas@2x.png` and `atlas@3x.png`, which
  it shrinks as mesh textures (to 64 × 64 under faceted-low-poly), so a
  phone's HUD drew blocky. Their import settings say `conform ui` until
  the cook names them too; the minimap and the item icons say it as HUD
  images.
