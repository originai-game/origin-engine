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

## 1. Choose a style from the premise

Write the premise in one line, then answer four questions:

| Question | Pulls toward |
|---|---|
| **Genre**: what does the player do most? | action and arcade → bold, high contrast, fast motion; puzzle and idle → soft, readable, gentle motion; strategy and RPG → dense, ornamented, calm |
| **Mood**: how should a session feel? | tense → dark ground, one hot accent; cosy → warm light ground, rounded; eerie → desaturated, thin type |
| **Era or world**: when and where is it set? | 8- and 16-bit nostalgia → pixel; myth, castles, quests → fantasy; space, hacking, mechs → sci-fi; city nights, synthwave, rhythm → neon; everyday, notebooks, mysteries → paper |
| **Audience**: who plays, on what? | children and casual phone players → cartoon (big targets, bright, bouncy); core desktop players → any, but denser |

Then pick a style from the catalogue (§12): its entry names the look
preset that sets the frame's treatment, the palette and asset specs, and
the skin pack that matches it. Then pick **one** starting skin. The six packs in the engine's
`templates/ui-kit/assets/skins/` are starting points, not the only looks:

| Pack | Premises it fits | Signature |
|---|---|---|
| `pixel` | platformers, roguelikes, retro shooters, jam games | square frames, 2 px bevels, hard shadows, a bitmap face |
| `cartoon` | casual, match-3, party, kids, idle | thick outlines, cream panels, squash on press, pops |
| `fantasy` | RPGs, card battlers, quest and dungeon games | parchment, gold corners and flourishes, a serif display |
| `scifi` | space, tactics, hacking, mech | hairlines, cut corners, brackets, scanlines, mono numerals |
| `paper` | narrative, puzzle, cosy, mystery, word games | off-white sheets, ink outlines, soft shadows, one red |
| `neon` | arcade, rhythm, racing, synthwave | dark glass, glowing tubes, italic display |

```ts
const skin = og.load.skin(world, "skins/fantasy/skin.ogt");
if (skin.ok) og.ui.configure(world, { skin: skin.value });
```

Make it the game's own by changing the skin file, never by passing colours
to individual calls: copy the pack directory into the game's
`assets/skins/<game>/`, then change `palette` (start with `accent`, `bg`,
`surface`, `fg`), `ramp` and `fonts` (any OFL font; keep
`fallback "fonts/NotoSansSC-Regular-subset.ttf"` for Chinese), and redraw
the atlas if the frames should change. The art in the art direction of the
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
  alone (body text at least 4.5:1; the packs' test enforces it for theirs).
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
  pale), an overcast grey (low contrast, `sunSize` 0), a warm dusk
  (`og.DayNight` at 17–18 h). A day-night cycle wants `speed` slow enough
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

Name it in the game (`og.game({ look: "looks/harbour.ogt", … })`) and it
is applied before `start` runs; switch looks at a tick with
`og.load.look` and `og.render.applyLook` (a flashback, a photo mode, an
area's mood). A look replaces each resource it names whole and leaves the
others alone, so an area look that names only `Fog` and `Sky` keeps the
game's grade. A copy of a preset under the preset's own name
(`assets/looks/<slug>.ogt` with `extends "<slug>"`) overrides it for every
look of the game that extends that slug. Colours may be written as sRGB
`"#rrggbb"`; the cook stores them linear. The cook refuses a field the
resource does not have, a value out of range and an asset not in the
bundle, naming the line.

**Expand the art bible.** Fill the catalogue's `artBibleTemplate` with the
chosen entry and the project's premise, camera and platform, and keep the
result with the game. It is the brief for every asset request after it:
asking for one asset never redesigns the style. Then hold the game to the
style's three `checks` on real frames (`og run --headless --commands` with
screenshots of the title, play and results), as §10 holds the UI.

| Slug | Family | Style | Support | Skin |
|---|---|---|---|---|
| `expressionist-painting` | Paint and line | Expressionist paint | content | `paper` |
| `bold-comic-illustration` | Paint and line | Bold comic illustration | partial | `fantasy` |
| `watercolor-wash` | Paint and line | Watercolour wash | planned | `paper` |
| `pixel-3d-hybrid` | Pixel and retro | Pixel sprites in 3D sets | partial | `fantasy` |
| `faceted-low-poly` | 3D rendering | Faceted low poly | full | `paper` |
| `sculpted-clay` | 3D rendering | Sculpted clay | partial | `cartoon` |
| `8bit-pixel` | Pixel and retro | Early console pixels | partial | `pixel` |
| `voxel-worlds` | 3D rendering | Voxel worlds | partial | `pixel` |
| `illuminated-manuscript` | Paint and line | Manuscript page | content | `paper` |
| `rubber-hose-animation` | Paint and line | Rubber-hose cartoon | content | `cartoon` |
| `pencil-hatching` | Paint and line | Pencil and hatching | planned | `paper` |
| `gothic-contour` | Paint and line | Gothic ornament | content | `fantasy` |
| `lush-painted-worlds` | Paint and line | Lush opaque painting | content | `fantasy` |
| `decorative-storybook` | Paint and line | Storybook ornament | content | `cartoon` |
| `classical-painting-collage` | Paint and line | Old-master collage | content | `paper` |
| `detailed-doodle` | Paint and line | Fine-line doodle worlds | content | `paper` |
| `black-figure-pottery` | Paint and line | Black-figure pottery | content | `fantasy` |
| `heavy-ink-shadows` | Paint and line | Heavy ink and shadow | planned | `fantasy` |
| `monochrome-pixel` | Pixel and retro | One-ink pixels | partial | `pixel` |
| `16bit-pixel` | Pixel and retro | Late console pixels | partial | `pixel` |
| `detailed-pixel` | Pixel and retro | Dense illustrative pixels | partial | `fantasy` |
| `luminous-pixel` | Pixel and retro | Glowing pixels on dark ground | partial | `neon` |
| `topdown-tile-pixel` | Pixel and retro | Top-down tile worlds | partial | `fantasy` |
| `low-resolution-3d` | 3D rendering | Low-resolution 3D | planned | `scifi` |
| `coarse-pixel` | Pixel and retro | Minimal low-count pixels | partial | `pixel` |
| `anime-cel-3d` | 3D rendering | Anime cel-shaded 3D | partial | `neon` |
| `inked-comic-3d` | 3D rendering | Inked comic 3D | partial | `scifi` |
| `paper-craft` | Shape and material | Paper craft | partial | `paper` |
| `miniature-diorama` | 3D rendering | Miniature diorama | full | `cartoon` |
| `painterly-3d` | 3D rendering | Painterly 3D | planned | `fantasy` |
| `cinematic-realism` | 3D rendering | Cinematic realism | full | `scifi` |
| `muted-cinematic-3d` | 3D rendering | Muted theatrical light | full | `scifi` |
| `layered-silhouette` | Shape and material | Layered silhouettes | partial | `neon` |
| `minimal-geometric` | Shape and material | Minimal geometry | full | `neon` |
| `neon-vector` | Shape and material | Neon vector lines | full | `neon` |
| `monochrome-dither-3d` | 3D rendering | Monochrome dithered 3D | planned | `pixel` |
| `luminous-layered-painting` | 3D rendering | Luminous layered depths | partial | `fantasy` |

Use the catalogue as a vocabulary, never as a list of games to copy: every
design, character and composition stays original (the platform's rule),
and the descriptors deliberately name no game or studio.
