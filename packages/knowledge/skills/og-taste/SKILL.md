---
name: og-taste
description: Art direction for a game on Origin Engine — choosing a style from the premise (genre, mood, era, audience) out of a 37-style catalogue and binding it as a look (post stack, stylisation, material defaults, skin) with a UI skin, a palette and a type ramp; composition and hierarchy on the play surface, colour, typography, motion, feedback and consistency; atmosphere (fog, sky, time of day, weather, water); the anti-patterns that make a game look like a web form; and the checklist to run (with og run captures and dump_ui) before calling a game done. Use when starting a game's look, building or reviewing any screen or HUD, or when a game "works but looks generic".
---

A game is judged in its first second, before anyone reads a word. The
engine's UI is correct by default and characterless by default: the platform
tokens are grey glass that suits a settings page. This skill is how to give a
game a look of its own and keep it consistent. The mechanics are og-ui's
(skins, motion and sounds are og-ui §6) and the amounts of game feel
og-feel's; this is the taste.

## What works

Both games written from this package alone reached `og polish`'s taste 99
from a preset tuned in their own look file: Lantern Vale
(`examples/lantern-vale`) the anime cel preset with ink, a tinted shadow
side and a painted second look for the results; Hollowbell
(`examples/hollowbell`) a 16-bit pixel grid with its own 32-colour palette
through `Stylise.palette` and a Bayer dither, and the 2D glow. Swapping the
look at a tick (`og.render.applyLook`) for the results worked the first
time in both. The Kestrel Stair (`examples/kestrel-stair`, 3D
third-person) reached taste 98 with a look extending `watercolor-wash`
(AgX, bloom, a violet shadow tint, a pale fog, the synthetic sky) under
`og.DayNight`, and `luminous-layered-painting` applied at a tick for the
ending and the results.

## 1. Choose a style from the premise

Write the premise in one line, then answer four questions:

| Question | Pulls toward |
|---|---|
| **Genre**: what does the player do most? | action and arcade → bold, high contrast, fast motion; puzzle and idle → soft, readable, gentle motion; strategy and RPG → dense, ornamented, calm |
| **Mood**: how should a session feel? | tense → dark ground, one hot accent; cosy → warm light ground, rounded; eerie → desaturated, thin type |
| **Era or world**: when and where is it set? | 8- and 16-bit nostalgia → pixel; myth, castles, quests → fantasy; space, hacking, mechs → sci-fi; city nights, synthwave, rhythm → neon; everyday, notebooks, mysteries → paper |
| **Audience**: who plays, on what? | children and casual phone players → cartoon (big targets, bright, bouncy); core desktop players → any, but denser |

Then choose the **look** from the style catalogue (§12): match the four
answers against each style's `premise` (genre first, then mood, then era
and audience), shortlist two, and take the one whose three `checks` the
game can hold on every frame. Some premises and where they land:

| Premise | Look (`extends`) | Why |
|---|---|---|
| a brisk 2D platformer, 16-bit nostalgia | `16bit-pixel` | energetic and bright: a 224-line grid, eight tones, a Bayer dither |
| a cosy farming or life sim seen from above | `topdown-tile-pixel` | cosy and curious: a tile-friendly grid, flat crisp tiles |
| a calm city or diorama builder | `miniature-diorama` | tilt-shift focus, a tabletop model |
| an exploration or flight game, serene | `faceted-low-poly` | flat facets, a soft vignette, AgX light |
| a fighting or character-action game | `anime-cel-3d` | toon bands, cool-tinted shadows, outlines that thin with distance |
| survival horror, lonely and uncanny | `low-resolution-3d` | early-console 3D: snapped vertices, affine textures, fog |
| an arcade shooter or rhythm game | `neon-vector` | bright outlines that bloom on a dark scene |
| a narrative puzzle, gentle and sad | `watercolor-wash` | bleeding strokes pooling at their rims on paper |

`npm create @origingame/game -- --premise "<one line>"` does this match
for a new game and writes the result (`--look <slug>` names it instead):
`assets/looks/<game>.ogt` extending the preset, named by `GameDesc.look`,
with a README section on changing it. For a game that exists, write the
same file by hand (§12) and add `look: "looks/<game>.ogt"` to `og.game`.
The entry also names the palette rules, the asset specs and the skin pack
that matches it. Then pick **one** starting skin. The six packs in the engine's
`templates/ui-kit/assets/skins/` are starting points, not the only looks:

| Pack | Premises it fits | Signature | Start look (§12) |
|---|---|---|---|
| `pixel` | platformers, roguelikes, retro shooters, jam games | square frames, 2 px bevels, hard shadows, a bitmap face | `16bit-pixel` |
| `cartoon` | casual, match-3, party, kids, idle | thick outlines, cream panels, squash on press, pops | `bold-comic-illustration` |
| `fantasy` | RPGs, card battlers, quest and dungeon games | parchment, gold corners and flourishes, a serif display | `illuminated-manuscript` |
| `scifi` | space, tactics, hacking, mech | hairlines, cut corners, brackets, scanlines, mono numerals | `cinematic-realism` |
| `paper` | narrative, puzzle, cosy, mystery, word games | off-white sheets, ink outlines, soft shadows, one red | `pencil-hatching` |
| `neon` | arcade, rhythm, racing, synthwave | dark glass, glowing tubes, italic display | `neon-vector` |

```ts
const skin = og.load.skin(world, "skins/fantasy/skin.ogt");
if (skin.ok) og.ui.configure(world, { skin: skin.value });
```

Make it the game's own by changing the skin file, never by passing colours
to individual calls: copy the pack directory into the game's
`assets/skins/<game>/` (`cp -R .../skins/fantasy assets/skins/<game>`) and
load `skins/<game>/skin.ogt`. Nothing in it needs rewriting: the ids in a
skin file are read relative to the file first (`atlas "atlas.png"`, `body
"Newsreader-Medium.ttf"`), then from `assets/` (`skins/fantasy/atlas.png`
still works). The one asset outside the pack is its CJK `fallback`,
`fonts/NotoSansSC-Regular-subset.ttf` (2.5 MB, shared by every pack, so
not copied six times): a project made by `npm create @origingame/game`
has it at that id; otherwise point the line at the game's own CJK font,
or delete it when the game shows no Chinese, Japanese or Korean (the cook
names the line it cannot find). Then change `palette` (start with
`accent`, `bg`, `surface`, `fg`), `ramp` and `fonts` (any OFL font beside
the skin file), and redraw the atlas if the frames should change (`og skin
pack`, og-ui §6). The art in the art direction of the
world (sprites, backgrounds) and the skin must agree: a pixel-art world in a
neon UI reads as two games.

## 2. Composition and hierarchy

- **What the eye reads first**: decide the one thing per screen (the score on
  results, Play on the title, the health on a HUD) and make it the biggest,
  the brightest or the only accent-coloured thing. If two things compete,
  shrink one.
- **The play surface is the hero**: a HUD sits in the corners and edges
  (anchors `topLeft`, `topRight`, `bottom`), never across the middle. Keep
  the centre third free during play.
- **HUD weight**: at most three HUD groups visible at once; each is a capsule
  with an icon and a number, not a panel with labels. Move rarely needed
  information behind pause.
- **Safe area**: everything interactive inside `og.ui.frameViewport().safeArea`
  plus the density gutter; anchors do this for you, absolute positions do not.
- **Phone vs desktop**: on `phonePortrait` the thumb zone is the bottom third:
  primary actions go there, touch controls at least 44 CSS px apart; titles
  shrink one step (`displayMd` instead of `displayLg`). On desktop, menus sit
  centred or left, never full-width.
- **Grouping**: spacing does the grouping. Related things are `space.sm`
  apart, groups `space.lg` or more. Do not draw a box around everything.

## 3. Colour

- **Limited palette**: a ground (`bg`), one or two surfaces, one ink (`fg`)
  with a muted ink, **one accent**, and the status colours. That is the whole
  palette; a skin declares exactly these.
- **One accent**, used for the one primary action per screen, earned stars,
  the player's own row and focus. If everything is accent, nothing is.
- **Value before hue**: check the screen in greyscale. Text, the primary
  button and the player character must separate from their ground by value
  alone (body text at least 4.5:1; the packs' test enforces it for theirs:
  every label ink, every button label in every state, a text field's
  placeholder, which draws in `fgSubtle` on the field's frame, and an
  accent badge, `accentInk` on `accentSoft` over a panel (the results
  screen's "New best"), and a status badge in each of `success`, `warn`,
  `danger` and `info` (the tone's own ink on its 16 % tint), so a copied
  pack whose `fgSubtle`, `fill`, `accentInk`, `accentSoft` or a status
  colour moves must keep those pairs at 4.5:1 too. The status colours are
  also the meters', toasts' and status dots' colours: on a cream panel they
  are deep (the cartoon pack's warn is `#7b5102`, not a bright amber that
  read 1.63:1), on a dark one light (a red under a dark pack's panel is a
  salmon such as `#ff8080`); a pack whose panels are light on a dark stage
  holds them in its panels, where its badges sit;
  it also reads the primary and secondary labels from a rendered frame,
  where thin strokes may never reach their colour: a pixel font's one-pixel
  strokes fall between pixels and took half their ink, so the pixel pack
  thickens them with `textWeight body 0.5` and cooks DotGothic16 at
  `px_range 8`; a copy of the pack made before that needs both lines).
- Saturated colour on large areas tires the eye: keep saturation for small
  things (the accent, icons, effects) and keep grounds calmer.
- Status colours mean status: never use `danger` red as decoration.

## 4. Typography

- **Display vs body**: a display face for titles, the score and big numbers
  (characterful: serif, rounded, italic, pixel), a body face for everything
  read (plain, open, legible at 13–15 px). Never the display face for
  paragraphs.
- **Numerals**: scores, timers, prices and stats in the numeric role
  (`font: "mono"`), so digits do not jitter as they count; big scores in the
  display face. Count them with `og.ui.counter`, never by setting text.
- **A ramp, not sizes**: use the skin's steps (`displayLg`, `xl`, `body`,
  `meta`); three or four steps per screen at most. Each step carries its
  tracking (display sizes tighten, as the tokens do; a skin's `ramp` line
  takes it as its fourth number after the CJK leading), and it is applied
  to Latin text only.
- Headings get `heading: true` so the skin's flourish frames them.

## 5. Motion

- Every change of state moves; nothing pops in or out without a transition
  unless reduced motion is on.
- **Durations 120 to 300 ms** for UI (the skin's `durations`), up to 700 ms
  for a screen transition, longer only for rewards. **Never linear**: ease
  out on arrival, ease in on exit.
- **Anticipation and follow-through**: a press squashes before the action
  (`press` in the skin), a reward overshoots and settles (`pop`, `motion.pop`).
- **Stagger**: a panel's controls arrive one after another (`motion.stagger`
  30–60 ms), not all at once.
- **Screens change through a transition** (`og.ui.screenTransition`): fade
  for calm games, wipe or slide for brisk ones, iris for arcade and retro,
  dissolve for dreamlike ones. One kind per game.
- Respect `reducedMotion` (the settings template passes it): everything
  becomes a fade; do not add motion that ignores it.

## 6. Feedback

Every input is answered **on the same tick** (og-feel §1) by a visual and a
sound: hover (the skin's lift or highlight), press (squash, the `press`
sound), success (a toast, a burst, `success`), refusal (a disabled button
shakes and plays `error` by itself; a wrong answer or a locked door shakes
its element with `og.ui.shake` and the `error` sound; never silently ignore
a press, and never shake the camera for it). Hand the UI its sounds with
`og.ui.configure(world, { sounds: { hover, press, open, close, success, error } })`
or the skin's `sound` lines. Results land with a count-up, stars with a
`og.ui.burst`, rewards with a pop.

## 7. Consistency

- **One skin per game.** Every screen, dialog and HUD uses it.
- **One radius scale**: the skin's `radius`; never pass radii by hand.
- **One shadow direction**: the skin's shadows all fall the same way (down);
  in-world shadows should agree.
- **One icon set**: the skin's `icons`; do not mix icon styles.
- **One transition kind** for screen changes, one entrance for dialogs.

## 8. Atmosphere

Fog, sky, time of day and weather set the mood of a 3D (and 2D) scene more
cheaply than any model; use them on purpose, one look per area.

- **Fog is depth**: distance fog with `density` 0.005–0.02 per metre makes
  the far plane recede; keep the nearest 10 m clear (`start` 6–10). Height
  fog (`heightDensity` 0.03–0.1 below the ground's height) fills valleys and
  swamps. Match the fog's `color` to the sky's horizon or the scene floats.
- **One sky per area**: a clear noon (`og.Sky` zenith deep blue, horizon
  pale), an overcast grey (low contrast, `sunSize` 0), a warm dusk with
  long shadows (`og.DayNight` at 16.5–17.25 h: at the default `tilt` the
  sun is 20° to 10° up, its light turning warm from about 17 h). Later the
  sun's light on level ground sinks into the sky's and its shadows fade
  (17.5 h: under 7° up), and at 18 h it sets whatever the `tilt` (which
  only lowers the noon sun); 17.5–18.2 h is a glowing horizon without
  sun shadows. Relief shadows the play area sooner: a valley's western
  ridge puts the floor in shade by about 17 h, so check the hours on a
  frame of the real ground (turn `azimuth` to set the sun across the
  valley). A day-night cycle wants `speed` slow enough
  that a player notices it over minutes (0.02–0.05 game hours per second),
  never per encounter.
- **Weather is a beat, not wallpaper**: rain or snow for a level's mood or a
  story moment; `intensity` 0.5–1; dust for deserts; mist for mornings and
  graveyards. Keep the play area readable: if enemies vanish in the fog,
  lower the density.
- **Water reads by its edges**: foam where it meets rocks and posts
  (`foamWidth` 0.2–0.5), a shallow colour close to the bed's and a deep one
  dark and saturated; `waveStrength` 0.2–0.5 for ponds, 0.6–1 for seas.
- **2D**: a `Water2D` lip a shade lighter than the surface; metaball fluids
  in one bright colour with a lighter `rim`, drawn under the foreground.
- Prove it: captures at the area's time of day; the far third of the frame
  lower in contrast than the near third, and the HUD unchanged by the fog.

## 9. Anti-patterns: "it looks like a web form"

Each of these is a defect, not a style choice:

- grey rounded rectangles on a grey ground (the token look left as is);
- the default sans everywhere, display text at body weight;
- a centred stack of equal buttons with no primary;
- no ornament, no texture, no shadow: flat boxes floating on the game;
- no motion: screens and dialogs pop in, numbers jump;
- no sound: buttons are silent;
- the HUD as a toolbar: a full-width bar of labelled boxes across the top;
- labels instead of icons on the HUD ("Health: 72/100" instead of a heart
  and a bar);
- every colour saturated, or every button in the accent;
- a settings-page layout (left-aligned form rows) for a title screen.

## 10. The checklist before a game is done

Run each item; each is checkable from a scripted run.

1. **A skin is configured**: `og.ui.theme(world).skin` is not empty (log it
   from a command, or read it in a playtest through a `facts` field).
2. **Title, pause and results screens captured**: `og run --headless --commands
   "0.5:show:title;1.5:screenshot:title.png;…"` (with the game's own commands);
   look at each capture in greyscale: the one thing per screen reads first.
3. **One primary per screen**: in a `dump_ui` capture, at most one button per
   screen uses the primary variant (the template's first item, or the one
   you marked); every other is secondary or quiet.
4. **HUD in the corners**: on the gameplay capture, `dump_ui` shows every HUD
   element's rectangle outside the central third of the surface, inside the
   safe area.
5. **Touch targets**: run with the phone viewport (`og.ui.previewViewport`,
   `og run --size 1170x2532`) and check in `dump_ui` that every control is at
   least 44 × 44 CSS px and nothing leaves the safe area.
6. **Text legible**: body text at 13 px or more in `dump_ui` heights (a line
   of at least 16 px); the skin passes 4.5:1 (the packs' test, or the same
   check over your atlas).
7. **Motion present**: captures 100 ms apart after `show:` differ (an
   entrance), and a score `dump_ui` text read twice during the results
   differs, then settles on the final value.
8. **Screen changes transition**: a capture halfway through a screen change
   shows the cover (`og.ui.screenTransition` answers `covering` or
   `revealing`).
9. **Sound on input**: after a scripted press, `dump_world` lists an
   `og.AudioSource` on the ui bus (bus 3).
10. **Reduced motion**: with `reducedMotion: true` the captures show fades
    only, and nothing is missing.
11. **CJK**: switch the language to Chinese in settings and capture: no
    missing-glyph boxes (the skin's fallback font covers it).
12. **None of section 9** is visible in any capture.
13. **`og polish .` scores taste 90 or more** where it applies, and every
    finding left is one you read in the frames and can name a reason for
    (og-build §5).

## 11. Polish, measured: `og polish`

`og polish .` scores the measurable part of §10 (contrast, text size, HUD
occupancy, screen transitions, UI sounds, locale keys) from the frames, the
UI trees and the run's audio, with og-feel's measures beside it; og-build §5
is the one list of its measures, thresholds and fixes. The numbers cannot
judge a style: read the frames before changing anything, and fix the cause
in the skin or the template, never per call.

## 12. Style catalogue: looks

A style is chosen once per game and then held by data, not by habit: the
**look** (`assets/looks/<game>.ogt`, an `.ogt` of `type look`) binds the
frame's whole treatment (tone mapping and exposure, the post stack, the
stylisation primitives, material defaults such as toon shading and
outlines, fog and sky, the UI skin and the antialiasing) in one diffable
file. The engine ships the primitives and the presets; the taste is in
which one you pick and how you tune it.

**Pick from the catalogue.** `looks/catalogue.json` (beside this skill)
describes 37 styles in four families (pixel and retro, paint and line,
shape and material, 3D rendering). For each: a one-line descriptor, three
checks a critic can judge on a captured frame, the premises it fits
(genre, mood, era, audience, as §1 asks them), the preset it maps to, the
skin pack, palette constraints, asset specs (logical resolution, sprite
scale, face budget, outline width, texture density) and an art-bible
direction. Match the premise's four answers against `premise`, shortlist
two, and choose by the checks you can actually hold for the whole game.

**Read `look.support` before promising a style.** `full`: every primitive
the style needs draws today. `partial` and `planned`: the frame will lack
what `missing` names until those primitives land; say so to whoever asked
for the style, and either accept the gap or pick a neighbour. `content`:
the look lives in authored art (painted backdrops, frame-by-frame
animation, hand-placed pixels); the engine frames it with the look, but
the assets carry it, so budget for them.

**Write the look.** Start from the style's preset and change only what the
game needs; your file wins field by field. A style whose `look.presetReady`
is false has no preset in this engine yet: leave out `extends` and set the
blocks yourself from the entry's `primitives`.

```text
type look
version 1
---
name "Harbour at dusk"
extends "anime-cel-3d"          # the preset's slug (templates/looks/<slug>.ogt)
skin "skins/harbour/skin.ogt"   # the game's own copy of the style's skin pack
RenderSettings {
  toneMapping agx
  exposure 1.1
}
MaterialStyle {
  shadowTint "#3b3566"
  shadowTintAmount 0.6
}
Fog {
  enabled true
  color "#9aa7c7"
  density 0.012
}
```

**The pixel grid** (`Stylise.pixelHeight`, `pixelUpscale`) scales the
frame up by a whole number, so pick the height for the output you ship
at. The height sets the scale: `pixelUpscale` when height × scale fits
the output's height, else (and for 0) the largest that does. The width
is the aspect's columns, at most the output's width over the scale, so
at most the last column gives way, never the scale: 240 lines at 1280 ×
720 are 426 columns at 3x (1278 × 720, a 1-pixel border each side), at
1920 × 1080 427 columns at 4x (1708 × 960, a 106/60-pixel border). A
height that divides the output fills it: 180 (4x), 240 (3x) or 360 (2x)
at 720p; 270 (4x) or 360 (3x) at 1080p. The `16bit-pixel` preset's 224
lines draw at 3x inside a 43/24-pixel border at 720p. Size the camera to
the grid: with 16-pixel tiles, 240 lines are 15 tiles tall.

The engine ships canvases and palettes a look names without the game
owning a copy: `Stylise.paintCanvas "@canvas/paper"` (also `linen`, a
woven canvas, and `watercolor`, cold-pressed paper) and `Stylise.palette
"@palettes/handheld4"` (also `lcd2`, `dusk16`, `bright32` and `1bit`). The
cook puts the ones a look names into the bundle; a palette or canvas of
the game's own is a path under `assets/` as before.

**Fit the pixel grid to the screens.** `Stylise.pixelHeight` sets the
whole-number scale from the output's height alone, the width follows the
aspect, and what the grid does not cover is a border in
`Camera.background` (the rule is `Stylise.pixelHeight`'s doc, og-game
§15). So choose the height by the screens the game meets, not by the era
alone. The scale per output (✓ fills it, ≈ fills it but for one column, b
leaves a border):

| `pixelHeight` | 1280×720 | 1920×1080 | 2560×1440 | 3840×2160 | phone 1600×720 | phone 1688×780 | phone 1830×824 |
|---|---|---|---|---|---|---|---|
| 90 | 8 ✓ | 12 ✓ | 16 ✓ | 24 ✓ | 8 ✓ | 8 b | 9 b |
| 135 | 5 b | 8 ✓ | 10 b | 16 ✓ | 5 b | 5 b | 6 b |
| 144 | 5 ✓ | 7 b | 10 ✓ | 15 ✓ | 5 ✓ | 5 b | 5 b |
| 160 | 4 b | 6 b | 9 ≈ | 13 b | 4 b | 4 b | 5 b |
| 180 | 4 ✓ | 6 ✓ | 8 ✓ | 12 ✓ | 4 ✓ | 4 b | 4 b |
| 192 | 3 b | 5 b | 7 b | 11 b | 3 b | 4 b | 4 b |
| 216 | 3 b | 5 ✓ | 6 b | 10 ✓ | 3 b | 3 b | 3 b |
| 224 | 3 b | 4 b | 6 b | 9 b | 3 b | 3 b | 3 b |
| 240 | 3 ≈ | 4 b | 6 ≈ | 9 ≈ | 3 ≈ | 3 b | 3 b |
| 270 | 2 b | 4 ✓ | 5 b | 8 ✓ | 2 b | 2 b | 3 b |
| 360 | 2 ✓ | 3 ✓ | 4 ✓ | 6 ✓ | 2 ✓ | 2 b | 2 b |

- 180 is the height to reach for (90 or 360 for a coarser or finer grid):
  it fills every 16:9 size. 144 suits 720p and 1440p, 216 and 270 1080p
  and 4K; 240 fills 720p but leaves a 106/60-pixel border at 1080p.
- 224 (`16bit-pixel`) and 192 (`8bit-pixel`) leave a border everywhere:
  write `pixelHeight 180` (or 216) in the game's look, which wins over the
  preset field by field, unless the border is part of the look.
- A phone's output is its page's CSS size times the device pixel ratio,
  capped at 2 (`publish { dpr_cap 2 }` in project.ogt; the columns above
  are 800×360, 844×390 and 915×412 CSS pixels in landscape), so most
  phones leave a border whatever the height. Give it a colour of the
  palette (`Camera.background`), or letterbox the page to 16:9
  (`publish { fit letterbox aspect "16:9" }`).
- With 16-pixel tiles, 180 lines are 11¼ tiles tall: size the camera and
  the rooms to the grid, not the other way round.
- Prove it on frames: `og run . --headless --size 1280x720` and again at
  `1920x1080`, a screenshot each, and read the border.

With a moving 3D camera, lay the paper and the strokes on the scene
rather than over the screen: `Stylise.paintCanvasSpace surface` (grain
per metre by `paintCanvasWorldScale`), `hatchingSpace surface` and
`halftoneSpace surface` stick to the surfaces and wrap round forms, so
nothing slides over the world as the camera moves; `screen` (the
default) suits 2D, fixed cameras and a deliberate printed-sheet feel.
When things move (characters, vehicles, a turning prop), take
`hatchingSpace object` (and `halftoneSpace object`, `paintCanvasSpace
object`): the marks are laid in each object's own space, so a walking
figure carries its strokes instead of wading through them (terrain keeps
the world's). `hatchingDepth` (0 to 1) gathers the strokes along
silhouettes and creases as `inkDepth` inks them: the contour shading of
an inked page. Pixel sprites standing in a 3D set take
`Sprite.dropShadow`; on a deck or a step give them `dropShadowHeight`,
and over an og.Terrain their shadows follow the slope, and bend into a
hollow or over a ridge narrower than themselves, by themselves.

Depth bands (`Stylise.depthBands`, the layered-silhouette, black-figure
and luminous-layered presets) cut the frame by distance from the camera,
so they need a camera that looks *across* the world: a side-scroller, a
stage, a three-quarter view down a valley or a street, a high view over
a landscape, where near, middle and far ground lie at clearly different
depths. Set `depthBandsNear` where the nearest layer you want starts and
`depthBandsFar` where the farthest layer ends (the far colour begins one
band before it), measured along the view on a real frame. They do not
work top-down or in a steep overhead camera: the whole play area sits at
about one depth and falls in one band, so it goes one flat colour and
the characters vanish into it; use a tinted shadow side, fog or a
palette there instead, or the bands only for a high establishing shot
(a title, the results). `depthBandsFlatten` 1 gives flat cut-paper
layers; 0 keeps the lit scene and only tints each layer (its shading and
hues show through), so characters stay readable; 0.3–0.6 is a painted
middle ground.

Name it in the game (`og.game({ look: "looks/harbour.ogt", … })`) and it
is applied after the first scene spawns and before `start` runs (so
`start` may still change any field, and reads the look's skin from
`og.render.look(world)?.skin`); switch looks at a tick with
`og.load.look` and `og.render.applyLook` (a flashback, a photo mode, an
area's mood). A look's `MaterialStyle` reaches every lit material that
leaves the field at its default; a material that must stay as made (a
photographic hero prop in a toon set, a product shot, a gem) takes
`styled: false` in `og.render.standardMaterial` and is drawn exactly as
declared. Physical materials are safe under a toon look: lacquer
(`clearcoat`), glass and jelly (`transmission`), velvet (`sheen`) keep
their shading and take only the rim, the outline and the shadow tint, so
a toon look never switches them off; set `toon: true` on one yourself to
flatten it too. A look replaces each resource it names whole and leaves the
others alone, so an area look that names only `Fog` and `Sky` keeps the
game's grade. A copy of a preset under the preset's own name
(`assets/looks/<slug>.ogt` with `extends "<slug>"`) overrides it for every
look of the game that extends that slug. Colours may be written as sRGB
`"#rrggbb"`; the cook stores them linear. The cook refuses a field the
resource does not have, a value out of range and an asset not in the
bundle, naming the line. A look's top-level `antialias` takes `tier`,
`off`, `msaa4` or `taa` (FXAA is `GameDesc.render`'s, og-game §16), and an
`Environment` block sets an environment map (`map "env/dusk.ogt"`) as
part of the look. A look in a streaming group applies once its group is
resident. The mechanics (what each resource's fields do, presets, the
built-in canvases) are og-game §15 and §16.

**Expand the art bible.** Fill the catalogue's `artBibleTemplate` with the
chosen entry and the project's premise, camera and platform, and keep the
result with the game. It is the brief for every asset request after it:
asking for one asset never redesigns the style. Then hold the game to the
style's three `checks` on real frames (`og run --headless --commands` with
screenshots of the title, play and results), as §10 holds the UI, and
make a playtest fact of `og.render.look(world)?.name` so a later change
cannot drop the look unnoticed. The platform's own wizard does the same
in its style step (`docs/platform-asks.md` in the engine repository): the
catalogue entry chosen from the premise, the look written, and the
entry's three checks handed to the critic that reads the frames.

| Slug | Family | Style | Support | Skin |
|---|---|---|---|---|
| `expressionist-painting` | Paint and line | Expressionist paint | content | `paper` |
| `bold-comic-illustration` | Paint and line | Bold comic illustration | full | `fantasy` |
| `watercolor-wash` | Paint and line | Watercolour wash | full | `paper` |
| `pixel-3d-hybrid` | Pixel and retro | Pixel sprites in 3D sets | full | `fantasy` |
| `faceted-low-poly` | 3D rendering | Faceted low poly | full | `paper` |
| `sculpted-clay` | 3D rendering | Sculpted clay | full | `cartoon` |
| `8bit-pixel` | Pixel and retro | Early console pixels | full | `pixel` |
| `voxel-worlds` | 3D rendering | Voxel worlds | full | `pixel` |
| `illuminated-manuscript` | Paint and line | Manuscript page | content | `paper` |
| `rubber-hose-animation` | Paint and line | Rubber-hose cartoon | content | `cartoon` |
| `pencil-hatching` | Paint and line | Pencil and hatching | full | `paper` |
| `gothic-contour` | Paint and line | Gothic ornament | content | `fantasy` |
| `lush-painted-worlds` | Paint and line | Lush opaque painting | content | `fantasy` |
| `decorative-storybook` | Paint and line | Storybook ornament | content | `cartoon` |
| `classical-painting-collage` | Paint and line | Old-master collage | content | `paper` |
| `detailed-doodle` | Paint and line | Fine-line doodle worlds | content | `paper` |
| `black-figure-pottery` | Paint and line | Black-figure pottery | content | `fantasy` |
| `heavy-ink-shadows` | Paint and line | Heavy ink and shadow | full | `fantasy` |
| `monochrome-pixel` | Pixel and retro | One-ink pixels | full | `pixel` |
| `16bit-pixel` | Pixel and retro | Late console pixels | full | `pixel` |
| `detailed-pixel` | Pixel and retro | Dense illustrative pixels | full | `fantasy` |
| `luminous-pixel` | Pixel and retro | Glowing pixels on dark ground | full | `neon` |
| `topdown-tile-pixel` | Pixel and retro | Top-down tile worlds | full | `fantasy` |
| `low-resolution-3d` | 3D rendering | Low-resolution 3D | full | `scifi` |
| `coarse-pixel` | Pixel and retro | Minimal low-count pixels | full | `pixel` |
| `anime-cel-3d` | 3D rendering | Anime cel-shaded 3D | full | `neon` |
| `inked-comic-3d` | 3D rendering | Inked comic 3D | full | `scifi` |
| `paper-craft` | Shape and material | Paper craft | full | `paper` |
| `miniature-diorama` | 3D rendering | Miniature diorama | full | `cartoon` |
| `painterly-3d` | 3D rendering | Painterly 3D | full | `fantasy` |
| `cinematic-realism` | 3D rendering | Cinematic realism | full | `scifi` |
| `muted-cinematic-3d` | 3D rendering | Muted theatrical light | full | `scifi` |
| `layered-silhouette` | Shape and material | Layered silhouettes | full | `neon` |
| `minimal-geometric` | Shape and material | Minimal geometry | full | `neon` |
| `neon-vector` | Shape and material | Neon vector lines | full | `neon` |
| `monochrome-dither-3d` | 3D rendering | Monochrome dithered 3D | full | `pixel` |
| `luminous-layered-painting` | 3D rendering | Luminous layered depths | full | `fantasy` |

Use the catalogue as a vocabulary, never as a list of games to copy: every
design, character and composition stays original (the platform's rule),
and the descriptors deliberately name no game or studio.
