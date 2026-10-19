# Spark Orb — the idle archetype (2D)

A glowing orb in the night. Tap it for sparks; spend them on generators
(embers, storms, crystals, stars) that make sparks on their own and circle
the orb as motes; every tenth of a kind doubles that kind. When a run has
made enough, the orb can collapse into a supernova: the sparks and the
generators go, stardust stays, and every stardust makes everything 10 %
stronger. Close the game and come back: the orb kept going at half its
rate while you were away.

星火球：夜空里一颗发光的球。点它得到火花；用火花购买生成器（余烬、风暴、晶簇、星辰），它们会自己产出火花，并化作光点绕着星火球转；同一种每满十个，产出翻倍。一轮攒够了，星火球可以坍缩成超新星：火花和生成器都会消失，留下星尘，每一点星尘让一切产出提升 10%。关掉游戏再回来：你离开的这段时间，星火球以一半的速度继续产出。

This is a complete Origin Engine project: `og run .` plays it headless,
`og build .` makes the play page, `og test .` runs its playtests and
`og polish .` scores them. It rebuilds the platform's retired Discover
idle prototype on the engine: its premise (one big tap target, generators,
prestige, progress while away) and none of its code or art.

```sh
og cook .                    # assets/ into cooked/ (og run and og build cook too)
og run . --headless --commands "0.3:play;0.5:own:0:12;0.5:own:1:4;1.5:screenshot:orb.png;1.6:quit"
og test .                    # tests/*.og.ts through og run --headless --commands
og polish .                  # the playtests scored against og-taste and og-feel
og build . --publish         # dist/: the play page as it ships
```

## What it demonstrates, and the skills it exercises

- **An economy as pure functions** (og-core §1, §5): prices, rates,
  milestones, the tap's value, the supernova's stardust and the time away's
  credit are functions of the counts and `Tuning` in `src/numbers.ts`, so
  the systems, the HUD and the save's merge agree on every number, and
  `set:Tuning.<field>` retunes the curve live.
- **Generators and prestige**: four generators at a price growing 15 % per
  purchase, a doubling every tenth of a kind, the supernova (sparks and
  generators reset, stardust kept, +10 % to everything per stardust), and
  the run's card.
- **Progress while away through og.platform's store** (og-platform-sdk
  §1–§4): the save (`orb`) is written at the revision read every 10 s of
  play, on pause, on quitting to the title and after a supernova; on launch
  it is read back and the time away credited at half the rate (capped at
  eight hours) with a welcome back; a write refused as stale (another
  device played meanwhile) is read again and the save with more lifetime
  sparks kept and written back. The time away is the host's to give: the
  save carries `og.platform.now` when it was written, and the store
  answer's `time` (the host's clock, as host input like a key press) less
  that is how long the game was closed.
- **Big numbers in the locale's words** (og-i18n §1–§5): `big` in
  `src/numbers.ts` reads the unit size and the units from the string
  tables, so English groups by thousands (`9,999`, `12.3K`, `1.20M`,
  `4.56e15`) and Chinese by ten-thousands (`12.3万`, `1.23亿`, `4.56亿亿`);
  `{n, number}` groups the small ones. Every string is a key of
  `assets/strings/en.ogt` and `zh-CN.ogt`; the Chinese font is Noto Sans SC
  subset to exactly the table's characters.
- **A satisfying tap** (og-feel §5–§7, §9): the orb squashes and flashes,
  the sparks made rise from where the finger landed, a burst of sparks is
  thrown out, and the chime climbs a quarter tone a tap through a quick run
  of taps (twelve at most), the number turning gold and bigger from the
  eighth. A
  purchase floats the rate it added from the ring its mote joins; a
  milestone flings a ring of sparks, punches the camera and plays a
  fanfare; the supernova bursts 200 sparks, holds the world 10 ticks,
  shakes and punches it.
- **og.ui counters on the tokens** (og-ui §2–§6, og-taste §1–§7): the
  engine's neon pack recoloured as `assets/skins/sparkorb` (the orb's gold
  as the one accent on night violet); the wallet and the rate top left, the
  stardust (an `og.ui.counter`) and Pause top right, the four generators
  along the bottom (a touch-sized button each, the count of each an
  `og.ui.counter`, the price in gold when it can be paid), the supernova's
  progress (`og.ui.meter`) then its button, a card that shakes with the
  error sound when its purchase is refused and bursts at a milestone,
  `og.ui.pauseMenu`, a settings dialog, the supernova's question, its card
  and the welcome back built from widgets, an iris between screens (a fade
  with reduced motion).
- **Sound** (og-audio §1–§5): the tap's chime on the ui bus (the orb is
  the game's one big button), a purchase's bell, a milestone's arpeggio,
  the supernova's boom, the UI's sounds from `og sfx` presets, and a slow
  72 BPM loop of pads and bells on the music bus ducked under the effects.

`og polish .` (2026-10-19, og and the native addon built release from
this checkout, load 28): taste 97, feel 94 (frame time 81: the p95 follows
the machine's load, `tests/evidence/P2/archetypes/frame-time.md`; the
median step and render are 2 ms each), in
`tests/evidence/P2/archetypes/idle/` of the engine repository. Hit feedback
is n/a: nothing here takes damage.

## Rules

- **Taps.** A press on the orb, Space, Enter or the pad's A makes the
  tap's sparks: 1 (times the stardust's multiplier) plus 5 % of the rate.
- **Generators.** 1–4, a card's button, or the pad's X on the one the
  d-pad (or the arrows) chose buys the next one at its price.

  | Generator | Makes | First price |
  |---|---|---|
  | Ember | 0.5 a second | 15 |
  | Storm | 4 a second | 110 |
  | Crystal | 26 a second | 1,200 |
  | Star | 160 a second | 13,000 |

  Each one bought makes the next 15 % dearer; every tenth of a kind
  doubles what that kind makes. A purchase the sparks cannot cover is
  refused and costs nothing.
- **Supernova.** Stardust is the floor of the square root of the run's
  sparks over 100,000 (one at 100,000, two at 400,000, three at 900,000…).
  N, the pad's Y or the Supernova button asks; Collapse resets the sparks
  and the generators, keeps the stardust, and shows the run's card.
- **Away.** On launch, the save's rate times half the seconds away (at most
  eight hours) is added, with a welcome back.

Everything is deterministic: the clock is the fixed tick, the numbers are
pure functions, the stars come from an og.Rng stream forked by name, and
`tests/replay.og.ts` replays a run to the same hash.

## How it feels

| Moment | What happens | Where |
|---|---|---|
| a tap | the orb squashes 1.12 × 0.9 and flashes, `+n` rises from the tap (gold and bigger from the eighth quick tap), 12 sparks burst out, a bell climbing a quarter tone per quick tap | `Orb`, `Bursts`, `Voices` |
| a purchase | the rate it adds floats up from its ring in the generator's colour, a mote joins the ring, a bright bell | `purchase`, `Motes` |
| a refusal | the card shakes with the error sound | `hud` |
| a milestone | `Storm ×2!` over the orb, a ring of 48 sparks, a 0.06 punch, a burst from the card, an arpeggio | `purchase`, `hud` |
| the supernova | 200 sparks, a 10-tick hit stop, trauma 0.6, a 0.2 punch, a boom; an iris to the run's card | `nova`, `Ui` |
| idle | the orb breathes, sparks drift up off it, motes circle on four rings at their own speeds, stars twinkle | `Orb`, `Motes`, `fx/drift.ogt`, `fx/stars.ogt` |

## State

| Type | Kind | What it holds |
|---|---|---|
| `Game` | resource | the phase, the sparks (now, this run, ever), the generators owned, stardust and supernovas, the rate and the tap's value, the counts (taps, purchases, refusals), the chosen generator, the dialogs, the time away and its credit, the save's requests and revision, sound and burst requests |
| `Tuning` | resource | the knobs below |
| `Prefs` | resource | the volumes, the language, reduced motion, the settings request |
| `Orb` | the orb | its taps |
| `Mote` | the 48 motes | the generator and the place in its ring |
| `Burst` | the burst pools | the kind |
| `Sfx` | one row per sound | its `Sound.*` bit, its plays |

## Systems (in tick order)

| System | Job |
|---|---|
| `Ui` | the screens, the camera for the surface, the buses, the pause and the supernova's keys, the platform's answers, the save |
| `Orb` | taps (a press on the orb, Space, Enter, the pad's A), the orb's breath and squash |
| `Economy` | the generators' sparks, choosing and buying, refusals, milestones, the supernova |
| `Motes` | one mote per generator owned (twelve a kind at most) on its ring |
| `Bursts` | fires the burst asked for this tick from its pool |
| `Voices` | plays each requested sound once |

## Tunables (`Tuning`)

| Field | Default | What it changes |
|---|---|---|
| `emberRate`, `emberCost` … `starRate`, `starCost` | 0.5, 15; 4, 110; 26, 1,200; 160, 13,000 | each generator |
| `costGrowth` | 1.15 | how much dearer each one bought makes the next |
| `milestone` | 10 | every this many of a kind doubles it |
| `tapBase`, `tapShare` | 1, 0.05 | the tap: a base, plus seconds of the rate |
| `dustBonus`, `novaScale` | 0.1, 100,000 | stardust's worth, and the run sparks its first one needs |
| `offlineShare`, `offlineCap` | 0.5, 28,800 s | the time away's credit |
| `saveEvery` | 600 ticks | how often play saves |

## What a Maker changes first

1. **The curve**: the generators' numbers and `costGrowth` in `Tuning`;
   `src/numbers.ts` is the whole economy.
2. **More generators**: a name in `GENERATORS`, its strings, an icon in
   `ICONS` (src/ui.ts), a case in `base`, a mote in `tools/make-assets.py`,
   an `owned<n>` field.
3. **Prestige**: `novaGain`, `dustMultiplier`.
4. **The look**: `assets/skins/sparkorb/skin.ogt`'s palette and the art in
   `tools/make-assets.py`, then `python3 tools/make-assets.py && og cook .`.
5. **Words and numbers**: `assets/strings/*.ogt` (a locale's `num.step`
   and units make its big numbers); a new Chinese string needs the script
   run again to subset the font.

## Commands (for scripts and agents)

| Action | Does |
|---|---|
| `play` | from the title to the orb, as Play does |
| `sparks:<n>` | sets the sparks (the rise counts as made this run) |
| `own:<generator 0-3>:<n>` | sets how many of a generator are owned |
| `supernova` | collapses now, as Collapse does (refused under one stardust) |
| `relaunch:<seconds>` | closes and reopens the game that much later in this process (for tests; a real launch reads the time away from the host's clock): the save read again, the time away credited |
| `stale_save` | another device writes a save with more progress |
| `viewport:<desktop\|phone>` | lays the UI out for a phone in portrait (390 × 844 at 3×) |
| `language:<en\|zh-CN>` | switches the language |
| core actions | `touch_move_to:play`, `Shop/Gen#0/Buy`…, `Nova/Supernova`, `Collapse`, `Later`, `Again`, `Collect`, `Pause`; `Space`, `Enter`, `1`–`4`, arrows, `B`, `N`, `Escape`; `gamepad:a` … |

`facts` (the probe's `state()`) reports the phase, the sparks, the rate,
the tap's value, the generators owned, stardust, supernovas, the run's and
the lifetime's sparks, taps, purchases, refusals, saves, stale writes, the
last time away's credit and the language. The playtests read the same
values from the `Game` resource with `get:Game`.

## Playtests (`og test .`)

| File | Proves |
|---|---|
| `tests/flow.og.ts` | Play; clicks on the orb, Space and Enter make a spark each and a click beside it none; 1 and the card's button buy embers at 15 and 18; the embers make a spark a second; a Star too dear is refused |
| `tests/milestone.og.ts` | the tenth Storm doubles storms (36 → 80 a second); the wallet reads `1.23M` |
| `tests/nova.og.ts` | N asks, Not yet keeps everything, Supernova and Collapse reset the run for one stardust, the card shows the run, Ignite again runs 10 % stronger; the telemetry and the save |
| `tests/offline.og.ts` | Escape saves; `relaunch:7200` reads it back with Continue on the title and a welcome back of 64.8K sparks for two hours at half of 18 a second; a stale write is merged, the other device's progress kept |
| `tests/away.og.ts` | two processes over one kept store (`store: "keep"`): ten embers and two storms saved by Escape at the first run's host time; the second launches two hours later by its own clock, offers Continue and welcomes back two hours at half of 18 a second, with no relaunch command |
| `tests/pause.og.ts` | Escape holds the orb (time scale 0); settings switch to 简体中文 and move the music; Done saves; numbers read `1.23亿` and `每秒 +2`; Resume |
| `tests/pad.og.ts` | a gamepad alone plays, taps, chooses, buys, is refused, asks for and declines the supernova, pauses and resumes |
| `tests/phone.og.ts` | 390 × 844 at 3×: the wallet and Pause below the notch, the generators above the home bar with touch-sized buttons, taps on the orb, a card bought |
| `tests/frames.og.ts` | the title, the orb with every kind of mote and the supernova's card match `references/<tier>/` (`OG_FX_CAPTURE=1 og test .` records them) |
| `tests/replay.og.ts` | a run with taps, purchases and a milestone, run twice, ends on one hash |

Each one fails when the behaviour it names is broken (one was broken on
purpose and restored: `tests/evidence/P2/archetypes/break-and-restore.md`
in the engine repository).

## Assets

`tools/make-assets.py` regenerates every asset byte for byte (two runs
give the same bytes): the orb, the glow, the spark and the motes, the
emitters, the sounds and the music, the `og sfx` preset files of the UI's
sounds, the font subset and the skin's recoloured atlas. `LICENSES.md`
lists them; the skin's are in `assets/skins/sparkorb/LICENSES.md`.

## Known limits (engine gaps, not design)

- og.ui's `idleHud` writes `Lv` and `/s` and formats numbers in English of
  its own, and fills a 340 px panel, so this game builds its HUD from
  widgets to keep every word and number in its tables.
- og.ui's `counter` counts again from where it is on every change, so the
  sparks, which change every tick, are a label; the counters show what
  jumps (the stardust, the generators owned).
