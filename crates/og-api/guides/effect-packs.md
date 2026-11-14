<!-- Generated from the vfx packs' extension.json, assets.json and preview/*.cost.json (packages/engine/extensions/, packages/vfx-packs/) by `og api gen`. Do not edit: `og api check` fails on any difference. -->
# The effect packs

Ready-made effects a game installs with `og vfx add <pack>`: converted Effekseer samples (`type vfx`, 3D) and pixel-art recipes (`type vfx2d`, 2D), each listed with its tags, keywords and effects. A bundled pack ships with the engine; a downloadable one comes with each release and `og vfx add` fetches it. Once installed, an effect spawns as `vfx/<pack>/<effect>` (`og api show vfx`; the `pixel-vfx` guide for `type vfx2d`); the pack format is docs/vfx-packs.md.

The last four columns are what `og vfx preview` measured, on the device each pack names: Size is the box the effect reaches in metres (x × y × z, y up), Anchor `ground` means it stands on its origin and `centre` that it plays about it (a ring, a shield: raise it to stand on the ground), GPU ms its peak frame time over an empty frame, Draws the most draw calls of one of its frames. An empty cell is not measured yet.

## 1. effekseer-starter

A starter set of Effekseer sample effects from nine contributors (fire, explosions, hits, slashes, heals, buffs, level-ups, projectiles, impacts, portals, weather, ambient), converted to type vfx. 38 effects, bundled with the engine: `og vfx add effekseer-starter`.

- Style: stylised, 3d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, weather
- Elements: dark, earth, fire, holy, ice, lightning, nature, water
- Keywords: 3d, anime, combat, effekseer, magic, particles, starter

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-starter/Attack_Impact.ogt` | Attack Impact | hit-spark |  | 0.567 |  |  | 0.32 | 24 |
| `vfx/effekseer-starter/Blow2.ogt` | Spark Hit | hit-spark |  | 0.533 |  |  | 0.63 | 19 |
| `vfx/effekseer-starter/Blow4.ogt` | Bright Explosion | explosion |  | 0.845 |  |  | 2.38 | 21 |
| `vfx/effekseer-starter/Blow8.ogt` | Star Hit | hit-spark |  | 3.017 |  |  | 2.78 | 25 |
| `vfx/effekseer-starter/Breakdown.ogt` | Breakdown Burst | explosion |  | 0.517 |  |  | 0.37 | 23 |
| `vfx/effekseer-starter/Claw3.ogt` | Claw | slash |  | 2.733 |  |  | 4.07 | 63 |
| `vfx/effekseer-starter/Cure1.ogt` | Star Cure | heal | holy | 1.275 |  |  | 1.25 | 24 |
| `vfx/effekseer-starter/Cure7.ogt` | Light Cure | heal | holy | 2.6 |  |  | 0.66 | 40 |
| `vfx/effekseer-starter/FeatherBomb.ogt` | Smoke Bomb | explosion |  | 13.006 |  |  | 13.07 | 55 |
| `vfx/effekseer-starter/Fire7.ogt` | Fire Pillar | fire | fire | 2.998 |  |  | 8.63 | 30 |
| `vfx/effekseer-starter/FireBall.ogt` | Fireball | magic-projectile | fire | 5.517 |  |  | 6.44 | 49 |
| `vfx/effekseer-starter/HealPotion2.ogt` | Potion | heal |  | 1.63 |  |  | 0.41 | 22 |
| `vfx/effekseer-starter/HolySandstorm.ogt` | Sandstorm | weather | earth | 6.733 |  |  | 6.77 | 27 |
| `vfx/effekseer-starter/Icicle2.ogt` | Icicles | magic-projectile | ice | 0.167 |  |  | 0.06 | 19 |
| `vfx/effekseer-starter/Laser03.ogt` | Green Laser | magic-projectile |  | loops |  |  | 0.59 | 20 |
| `vfx/effekseer-starter/Light4.ogt` | Level Up Arrows | level-up | holy | 2.092 |  |  | 3.79 | 78 |
| `vfx/effekseer-starter/Magic2_fire.ogt` | Fire Pentagram | portal | fire | 0.483 |  |  | 0.10 | 16 |
| `vfx/effekseer-starter/MagicCold.ogt` | Frost | magic-impact | ice | 2.15 |  |  | 0.39 | 17 |
| `vfx/effekseer-starter/MagicDark.ogt` | Dark Curse | magic-impact | dark | 2.05 |  |  | 0.93 | 19 |
| `vfx/effekseer-starter/MagicFire2.ogt` | Fire Breath | fire | fire | 1.91 |  |  | 0.26 | 15 |
| `vfx/effekseer-starter/MagicHeal2.ogt` | Healing Circle | heal | holy | 4 |  |  | 0.94 | 19 |
| `vfx/effekseer-starter/MagicWater.ogt` | Bubble Burst | magic-impact | water | 2.09 |  |  | 2.24 | 23 |
| `vfx/effekseer-starter/PowerUp.ogt` | Power Up | buff |  | 1.75 |  |  | 0.29 | 18 |
| `vfx/effekseer-starter/Spear2.ogt` | Light Spear | magic-projectile |  | 0.983 |  |  | 0.23 | 16 |
| `vfx/effekseer-starter/Sword5.ogt` | Blue Slash | slash |  | 0.6 |  |  | 4.22 | 17 |
| `vfx/effekseer-starter/Sword7.ogt` | Cyan Cross Slash | slash |  | 0.583 |  |  | 0.24 | 18 |
| `vfx/effekseer-starter/Sword8.ogt` | Red Arc Slash | slash |  | 1.417 |  |  | 0.60 | 21 |
| `vfx/effekseer-starter/Thunder1.ogt` | Golden Rise | level-up | lightning | 2.15 |  |  | 0.58 | 19 |
| `vfx/effekseer-starter/Undine4.ogt` | Water Burst | magic-impact | water | 1.667 |  |  | 0.48 | 18 |
| `vfx/effekseer-starter/boss_death.ogt` | Boss Death | explosion |  | loops |  |  | 0.25 | 17 |
| `vfx/effekseer-starter/electric_dust.ogt` | Floating Motes | ambient | lightning | 3.4 |  |  | 0.35 | 16 |
| `vfx/effekseer-starter/hit_eff.ogt` | Hit Flash | hit-spark |  | 0.622 |  |  | 0.34 | 24 |
| `vfx/effekseer-starter/hozyo2.ogt` | Power Rings | buff |  | 1.217 |  |  | 0.41 | 30 |
| `vfx/effekseer-starter/hozyo4.ogt` | Attack Up | buff | fire | loops |  |  | 4.94 | 24 |
| `vfx/effekseer-starter/magic_circle.ogt` | Magic Circle | portal |  | 16.683 |  |  | 0.46 | 26 |
| `vfx/effekseer-starter/meteo1.ogt` | Meteor Strike | explosion | fire | 0.858 |  |  | 0.37 | 23 |
| `vfx/effekseer-starter/sakura1.ogt` | Sakura Petals | ambient | nature | 13.8 |  |  | 1.52 | 52 |
| `vfx/effekseer-starter/smallTexturesSquareWave_orbital.ogt` | Arcane Ring | portal |  | 66.667 |  |  | 0.07 | 17 |

## 2. pixel-scenario

The Pixel VFX Forge library: 42 pixel-art effects (combat, magic, elements, feedback, ambient) as type vfx2d recipes. 42 effects, bundled with the engine: `og vfx add pixel-scenario`.

- Style: pixel, 2d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, smoke, ui, weather
- Elements: arcane, dark, earth, fire, holy, ice, lightning, nature, water, wind
- Keywords: 2d, flipbook, forge, pixel, pixel art, scenario, sprite

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/pixel-scenario/arcane-magic-circle.ogt` | Eldritch Magic Circle | magic-impact | arcane | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/big-fire-explosion.ogt` | Cinder Explosion | explosion | fire | 1.2 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/blue-energy-beam.ogt` | Aether Beam | magic-projectile | arcane | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/campfire.ogt` | Ember Flame | fire | fire | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/charge-up-energy.ogt` | Pulse Charge | buff | arcane | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/cherry-blossom-petals.ogt` | Thorn Ambient | ambient | nature | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/crescent-sword-slash.ogt` | Razor Slash | slash |  | 0.667 |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/critical-hit-damage-999.ogt` | Ember Damage Number | ui |  | 1.067 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/dark-shadow-aura.ogt` | Night Aura | buff | dark | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/dash-speed-lines.ogt` | Cyclone Dash | slash | wind | 0.556 |  |  | 1.42 | 13 |
| `vfx/pixel-scenario/dimensional-portal.ogt` | Astral Portal | portal | arcane | loops |  |  | 0.02 | 13 |
| `vfx/pixel-scenario/dragon-fire-breath.ogt` | Cinder Breath | fire | fire | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/electric-impact-hit.ogt` | Static Impact | hit-spark | lightning | 0.556 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/embers-rising.ogt` | Blaze Ambient | ambient | fire | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/falling-leaves.ogt` | Bloom Ambient | ambient | nature | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/feral-claw-scratch-blood.ogt` | Sanguine Claw | slash |  | 0.667 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/fire-tornado.ogt` | Blaze Tornado | fire | fire | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/fireball-projectile.ogt` | Ember Projectile | magic-projectile | fire | loops |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/fireflies.ogt` | Mystic Ambient | ambient | fire | loops |  |  | 0.02 | 13 |
| `vfx/pixel-scenario/fireworks-celebration.ogt` | Arcane Fireworks | ui | fire | 2.133 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/frost-nova.ogt` | Frost Nova | magic-impact | ice | 1.067 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/gold-coins-loot-drop.ogt` | Lucky Loot | ui |  | 1.6 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/ground-slam-quake.ogt` | Stone Ground Slam | magic-impact | earth | 1.2 |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/healing-with-rising-crosses.ogt` | Bloom Heal | heal | holy | 1.333 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/hex-shield-barrier.ogt` | Flux Shield | buff | arcane | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/holy-pillar-of-light.ogt` | Divine Pillar | magic-impact | holy | 1.333 |  |  | 0.02 | 13 |
| `vfx/pixel-scenario/ice-spikes-erupting.ogt` | Frozen Spikes | magic-impact | ice | 1.2 |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/lava-eruption-pillar.ogt` | Volcanic Pillar | magic-impact | fire | 1.333 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/level-up.ogt` | Gilded Level Up | level-up |  | 1.6 |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/lightning-strike-from-the-sky.ogt` | Storm Strike | magic-impact | lightning | 0.889 |  |  | 0.00 | 13 |
| `vfx/pixel-scenario/meteor-strike.ogt` | Ember Meteor | explosion | fire | 1.733 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/plasma-orb.ogt` | Ion Orb | magic-projectile | lightning | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/poison-gas-cloud.ogt` | Blight Cloud | smoke | nature | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/purple-black-hole-vortex.ogt` | Shadow Vortex | portal | dark | loops |  |  | 0.03 | 13 |
| `vfx/pixel-scenario/rain.ogt` | Aqua Ambient | weather | water | loops |  |  | 0.05 | 13 |
| `vfx/pixel-scenario/smoke-poof.ogt` | Ashen Poof | smoke |  | 0.933 |  |  | 0.12 | 13 |
| `vfx/pixel-scenario/snow-falling.ogt` | Glacial Ambient | weather | ice | loops |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/sparkle-twinkle.ogt` | Sacred Sparkle | ui |  | loops |  |  | 0.04 | 13 |
| `vfx/pixel-scenario/spinning-wind-slash.ogt` | Cyclone Slash | slash | wind | 0.667 |  |  | 0.01 | 13 |
| `vfx/pixel-scenario/teleport-blink.ogt` | Mystic Teleport | portal | arcane | 1.333 |  |  | 0.06 | 13 |
| `vfx/pixel-scenario/triple-slash-combo-with-blue-energy.ogt` | Flux Slash | slash | arcane | 1.167 |  |  | 0.02 | 13 |
| `vfx/pixel-scenario/water-splash.ogt` | Aqua Splash | magic-impact | water | 1.067 |  |  | 0.02 | 13 |

## 3. effekseer-andrewfm01

AndrewFM01's Effekseer sample effects (15), converted to type vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-andrewfm01`.

- Style: stylised, 3d
- Categories: beam, explosion, fire, hit-spark, magic-impact, magic-projectile, portal, smoke, ui
- Elements: fire, lightning
- Keywords: 3d, andrewfm01, anime, effekseer, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-andrewfm01/actionLines_halo.ogt` | Action Lines Halo | ui |  | 1.673 |  |  | 1.31 | 18 |
| `vfx/effekseer-andrewfm01/actionLines_halo2.ogt` | Action Lines Halo 2 | ui |  | 1.683 |  |  | 1.83 | 17 |
| `vfx/effekseer-andrewfm01/actionLines_radial.ogt` | Action Lines Radial | ui |  | 1.673 |  |  | 4.18 | 20 |
| `vfx/effekseer-andrewfm01/actionLines_radial2.ogt` | Action Lines Radial 2 | ui |  | 1.7 |  |  | 6.79 | 20 |
| `vfx/effekseer-andrewfm01/actionLines_shockwave.ogt` | Action Lines Shockwave | ui |  | 1.667 |  |  | 0.15 | 20 |
| `vfx/effekseer-andrewfm01/actionLines_strip.ogt` | Action Lines Strip | ui |  | 1.667 |  |  | 0.08 | 19 |
| `vfx/effekseer-andrewfm01/blue_laser.ogt` | Blue Laser | beam |  | loops |  |  | 0.34 | 18 |
| `vfx/effekseer-andrewfm01/boss_death.ogt` | Boss Death | explosion |  | loops |  |  | 0.30 | 17 |
| `vfx/effekseer-andrewfm01/drill.ogt` | Drill | magic-projectile |  | loops |  |  | 2.39 | 24 |
| `vfx/effekseer-andrewfm01/electric_dust.ogt` | Electric Dust | smoke | lightning | 3.4 |  |  | 0.32 | 16 |
| `vfx/effekseer-andrewfm01/fire.ogt` | Fire | fire | fire | 1 |  |  | 1.48 | 24 |
| `vfx/effekseer-andrewfm01/hit_eff.ogt` | Hit Eff | hit-spark |  | 0.622 |  |  | 0.42 | 24 |
| `vfx/effekseer-andrewfm01/magic_circle.ogt` | Magic Circle | portal |  | 16.683 |  |  | 0.47 | 26 |
| `vfx/effekseer-andrewfm01/smallTexturesRibbon.ogt` | Small Textures Ribbon | magic-impact |  | 1.017 |  |  | 0.09 | 20 |
| `vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital.ogt` | Small Textures Square Wave Orbital | magic-impact |  | 66.667 |  |  | 0.08 | 17 |

## 4. effekseer-effekseer01

Effekseer01's Effekseer sample effects (15), converted to type vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-effekseer01`.

- Style: stylised, 3d
- Categories: basic, beam
- Elements: 
- Keywords: 3d, anime, effekseer, effekseer01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-effekseer01/Homing_Laser01.ogt` | Homing Laser 01 | beam |  | 4.267 |  |  | 0.14 | 17 |
| `vfx/effekseer-effekseer01/Laser01.ogt` | Laser 01 | beam |  | 1.333 |  |  | 1.12 | 19 |
| `vfx/effekseer-effekseer01/Laser02.ogt` | Laser 02 | beam |  | 3.733 |  |  | 1.73 | 19 |
| `vfx/effekseer-effekseer01/Laser03.ogt` | Laser 03 | beam |  | loops |  |  | 0.69 | 20 |
| `vfx/effekseer-effekseer01/Simple_Distortion.ogt` | Simple Distortion | basic |  | 2.917 |  |  | 0.44 | 24 |
| `vfx/effekseer-effekseer01/Simple_GeneratingPosition1.ogt` | Simple Generating Position 1 | basic |  | 0.168 |  |  | 0.30 | 15 |
| `vfx/effekseer-effekseer01/Simple_Model_UV.ogt` | Simple Model UV | basic |  | 3.5 |  |  | 0.38 | 16 |
| `vfx/effekseer-effekseer01/Simple_Ribbon_Parent.ogt` | Simple Ribbon Parent | basic |  | 1.817 |  |  | 0.13 | 16 |
| `vfx/effekseer-effekseer01/Simple_Ribbon_Sword.ogt` | Simple Ribbon Sword | basic |  | 0.4 |  |  | 0.04 | 15 |
| `vfx/effekseer-effekseer01/Simple_Ring_Shape1.ogt` | Simple Ring Shape 1 | basic |  | 9.5 |  |  | 0.14 | 15 |
| `vfx/effekseer-effekseer01/Simple_Ring_Shape2.ogt` | Simple Ring Shape 2 | basic |  | 7 |  |  | 0.24 | 15 |
| `vfx/effekseer-effekseer01/Simple_Sprite_BillBoard.ogt` | Simple Sprite Bill Board | basic |  | 0.665 |  |  | 1.17 | 15 |
| `vfx/effekseer-effekseer01/Simple_Sprite_Fixed.ogt` | Simple Sprite Fixed | basic |  | 1.633 |  |  | 0.81 | 15 |
| `vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis.ogt` | Simple Sprite Fixed YAxis | basic |  | 2.817 |  |  | 1.07 | 15 |
| `vfx/effekseer-effekseer01/Simple_Track1.ogt` | Simple Track 1 | basic |  | 3.333 |  |  | 0.51 | 15 |

## 5. effekseer-hato01

HATO01's Effekseer sample effects (4), converted to type vfx. 4 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-hato01`.

- Style: stylised, 3d
- Categories: ambient
- Elements: nature
- Keywords: 3d, anime, effekseer, hato01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-hato01/aoi-chou.ogt` | Aoi Chou | ambient | nature | loops |  |  | 0.31 | 20 |
| `vfx/effekseer-hato01/iwai-ume.ogt` | Iwai Ume | ambient | nature | 20.3 |  |  | 0.45 | 18 |
| `vfx/effekseer-hato01/kiroi-chou.ogt` | Kiroi Chou | ambient | nature | 33.375 |  |  | 0.37 | 22 |
| `vfx/effekseer-hato01/sakura1.ogt` | Sakura 1 | ambient | nature | 13.8 |  |  | 1.55 | 52 |

## 6. effekseer-magicalxspiral

MAGICAL X SPIRAL's Effekseer sample effects (101), converted to type vfx. 101 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-magicalxspiral`.

- Style: stylised, 3d
- Categories: explosion, fire, magic-impact, magic-projectile, portal, shield, slash, smoke, weather
- Elements: earth, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, magicalxspiral, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-magicalxspiral/A_Gohlem1.ogt` | A Gohlem 1 | magic-impact | earth | 0.983 |  |  | 0.34 | 30 |
| `vfx/effekseer-magicalxspiral/A_Salamander1.ogt` | A Salamander 1 | magic-impact | fire | 0.983 |  |  | 0.19 | 19 |
| `vfx/effekseer-magicalxspiral/A_Salamander2.ogt` | A Salamander 2 | magic-impact | fire | 0.767 |  |  | 0.45 | 28 |
| `vfx/effekseer-magicalxspiral/A_Salamander4.ogt` | A Salamander 4 | magic-impact | fire | 1.167 |  |  | 0.60 | 24 |
| `vfx/effekseer-magicalxspiral/A_Sylph1.ogt` | A Sylph 1 | magic-impact | wind | 0.983 |  |  | 0.15 | 20 |
| `vfx/effekseer-magicalxspiral/A_Sylph2.ogt` | A Sylph 2 | magic-impact | wind | 0.575 |  |  | 0.26 | 17 |
| `vfx/effekseer-magicalxspiral/A_Undine1.ogt` | A Undine 1 | magic-impact | water | 0.983 |  |  | 1.13 | 19 |
| `vfx/effekseer-magicalxspiral/A_Undine2.ogt` | A Undine 2 | magic-impact | water | 0.567 |  |  | 0.45 | 26 |
| `vfx/effekseer-magicalxspiral/AquaPoint.ogt` | Aqua Point | magic-impact | water | 2.317 |  |  | 0.27 | 21 |
| `vfx/effekseer-magicalxspiral/Attack1.ogt` | Attack 1 | slash |  | 0.983 |  |  | 0.13 | 18 |
| `vfx/effekseer-magicalxspiral/Attack2.ogt` | Attack 2 | slash |  | 0.983 |  |  | 0.14 | 18 |
| `vfx/effekseer-magicalxspiral/Attack5.ogt` | Attack 5 | slash |  | 0.9 |  |  | 0.12 | 18 |
| `vfx/effekseer-magicalxspiral/Attack6.ogt` | Attack 6 | slash |  | 0.9 |  |  | 0.10 | 18 |
| `vfx/effekseer-magicalxspiral/Attack_Impact.ogt` | Attack Impact | magic-impact |  | 0.567 |  |  | 0.28 | 23 |
| `vfx/effekseer-magicalxspiral/Breakdown.ogt` | Breakdown | explosion |  | 0.517 |  |  | 0.37 | 23 |
| `vfx/effekseer-magicalxspiral/FirePot1.ogt` | Fire Pot 1 | fire | fire | 1 |  |  | 0.23 | 20 |
| `vfx/effekseer-magicalxspiral/FirePot2.ogt` | Fire Pot 2 | fire | fire | 1 |  |  | 0.17 | 20 |
| `vfx/effekseer-magicalxspiral/Gohlem1.ogt` | Gohlem 1 | magic-impact | earth | 0.817 |  |  | 0.37 | 27 |
| `vfx/effekseer-magicalxspiral/Gohlem11.ogt` | Gohlem 11 | magic-impact | earth | 0.858 |  |  | 0.34 | 17 |
| `vfx/effekseer-magicalxspiral/Gohlem5.ogt` | Gohlem 5 | magic-impact | earth | 0.983 |  |  | 1.43 | 34 |
| `vfx/effekseer-magicalxspiral/Gohlem7.ogt` | Gohlem 7 | magic-impact | earth | 2.917 |  |  | 0.30 | 24 |
| `vfx/effekseer-magicalxspiral/Gohlem_Shield1.ogt` | Gohlem Shield 1 | shield | earth | 0.883 |  |  | 0.29 | 20 |
| `vfx/effekseer-magicalxspiral/Gohlem_Shield2.ogt` | Gohlem Shield 2 | shield | earth | 1.833 |  |  | 0.14 | 15 |
| `vfx/effekseer-magicalxspiral/Holy1.ogt` | Holy 1 | magic-impact | holy | 0.45 |  |  | 0.17 | 18 |
| `vfx/effekseer-magicalxspiral/Holy2.ogt` | Holy 2 | magic-impact | holy | 0.583 |  |  | 0.12 | 18 |
| `vfx/effekseer-magicalxspiral/Icicle1.ogt` | Icicle 1 | magic-projectile | ice | 0.4 |  |  | 0.20 | 16 |
| `vfx/effekseer-magicalxspiral/Icicle2.ogt` | Icicle 2 | magic-projectile | ice | 0.167 |  |  | 0.08 | 19 |
| `vfx/effekseer-magicalxspiral/Icicle3.ogt` | Icicle 3 | magic-projectile | ice | 0.167 |  |  | 0.24 | 19 |
| `vfx/effekseer-magicalxspiral/IcicleBroken.ogt` | Icicle Broken | explosion | ice | 0.517 |  |  | 0.43 | 27 |
| `vfx/effekseer-magicalxspiral/IcicleLocus.ogt` | Icicle Locus | magic-projectile | ice | 0.4 |  |  | 0.15 | 16 |
| `vfx/effekseer-magicalxspiral/Impact.ogt` | Impact | magic-impact |  | 0.7 |  |  | 0.24 | 25 |
| `vfx/effekseer-magicalxspiral/Lance3.ogt` | Lance 3 | magic-projectile |  | 2.15 |  |  | 0.56 | 19 |
| `vfx/effekseer-magicalxspiral/Lance4.ogt` | Lance 4 | magic-projectile |  | 1.15 |  |  | 0.44 | 26 |
| `vfx/effekseer-magicalxspiral/Lance5.ogt` | Lance 5 | magic-projectile |  | 1.15 |  |  | 0.48 | 30 |
| `vfx/effekseer-magicalxspiral/Magic1_earth.ogt` | Magic 1 Earth | magic-impact | earth | 1.667 |  |  | 0.54 | 21 |
| `vfx/effekseer-magicalxspiral/Magic1_fire.ogt` | Magic 1 Fire | fire | fire | 1.667 |  |  | 0.56 | 21 |
| `vfx/effekseer-magicalxspiral/Magic1_holy.ogt` | Magic 1 Holy | magic-impact | holy | 1.667 |  |  | 0.60 | 21 |
| `vfx/effekseer-magicalxspiral/Magic1_water.ogt` | Magic 1 Water | magic-impact | water | 1.667 |  |  | 0.61 | 21 |
| `vfx/effekseer-magicalxspiral/Magic1_wind.ogt` | Magic 1 Wind | magic-impact | wind | 1.667 |  |  | 0.54 | 21 |
| `vfx/effekseer-magicalxspiral/Magic2_earth.ogt` | Magic 2 Earth | magic-impact | earth | 0.483 |  |  | 0.10 | 16 |
| `vfx/effekseer-magicalxspiral/Magic2_fire.ogt` | Magic 2 Fire | fire | fire | 0.483 |  |  | 0.18 | 16 |
| `vfx/effekseer-magicalxspiral/Magic2_holy.ogt` | Magic 2 Holy | magic-impact | holy | 0.483 |  |  | 0.19 | 16 |
| `vfx/effekseer-magicalxspiral/Magic2_water.ogt` | Magic 2 Water | magic-impact | water | 0.483 |  |  | 0.12 | 16 |
| `vfx/effekseer-magicalxspiral/Magic2_wind.ogt` | Magic 2 Wind | magic-impact | wind | 0.483 |  |  | 0.12 | 16 |
| `vfx/effekseer-magicalxspiral/MagicArea.ogt` | Magic Area | portal |  | 2.917 |  |  | 0.39 | 22 |
| `vfx/effekseer-magicalxspiral/Rock1.ogt` | Rock 1 | magic-impact | earth | 1.667 |  |  | 0.33 | 18 |
| `vfx/effekseer-magicalxspiral/Rock2.ogt` | Rock 2 | magic-impact | earth | 0.817 |  |  | 0.28 | 27 |
| `vfx/effekseer-magicalxspiral/Salamander1.ogt` | Salamander 1 | magic-impact | fire | 0.45 |  |  | 0.16 | 18 |
| `vfx/effekseer-magicalxspiral/Salamander11.ogt` | Salamander 11 | magic-impact | fire | 1.317 |  |  | 1.18 | 39 |
| `vfx/effekseer-magicalxspiral/Salamander12.ogt` | Salamander 12 | magic-impact | fire | 0.883 |  |  | 0.97 | 28 |
| `vfx/effekseer-magicalxspiral/Salamander14.ogt` | Salamander 14 | magic-impact | fire | 0.833 |  |  | 0.32 | 19 |
| `vfx/effekseer-magicalxspiral/Salamander15.ogt` | Salamander 15 | magic-impact | fire | 2.7 |  |  | 0.18 | 26 |
| `vfx/effekseer-magicalxspiral/Salamander2.ogt` | Salamander 2 | magic-impact | fire | 0.583 |  |  | 0.07 | 18 |
| `vfx/effekseer-magicalxspiral/Salamander3.ogt` | Salamander 3 | magic-impact | fire | 1.317 |  |  | 0.72 | 30 |
| `vfx/effekseer-magicalxspiral/Salamander4.ogt` | Salamander 4 | magic-impact | fire | 1.317 |  |  | 0.58 | 30 |
| `vfx/effekseer-magicalxspiral/Salamander5.ogt` | Salamander 5 | magic-impact | fire | 1.317 |  |  | 0.95 | 30 |
| `vfx/effekseer-magicalxspiral/Salamander6.ogt` | Salamander 6 | magic-impact | fire | 1.233 |  |  | 0.44 | 22 |
| `vfx/effekseer-magicalxspiral/Salamander7.ogt` | Salamander 7 | magic-impact | fire | 0.717 |  |  | 0.32 | 18 |
| `vfx/effekseer-magicalxspiral/Snowman1.ogt` | Snowman 1 | weather | ice | 0.417 |  |  | 0.04 | 16 |
| `vfx/effekseer-magicalxspiral/StairBroken.ogt` | Stair Broken | explosion | earth | 0.633 |  |  | 0.36 | 27 |
| `vfx/effekseer-magicalxspiral/StairBroken3.ogt` | Stair Broken 3 | explosion | earth | 1.317 |  |  | 0.27 | 15 |
| `vfx/effekseer-magicalxspiral/Sylph1.ogt` | Sylph 1 | magic-impact | wind | 0.383 |  |  | 0.07 | 18 |
| `vfx/effekseer-magicalxspiral/Sylph10.ogt` | Sylph 10 | magic-impact | wind | 1.55 |  |  | 0.22 | 21 |
| `vfx/effekseer-magicalxspiral/Sylph11.ogt` | Sylph 11 | magic-impact | wind | 0.883 |  |  | 0.62 | 25 |
| `vfx/effekseer-magicalxspiral/Sylph13.ogt` | Sylph 13 | magic-impact | wind | 2.517 |  |  | 0.29 | 23 |
| `vfx/effekseer-magicalxspiral/Sylph15.ogt` | Sylph 15 | magic-impact | wind | 0.817 |  |  | 0.74 | 27 |
| `vfx/effekseer-magicalxspiral/Sylph2.ogt` | Sylph 2 | magic-impact | wind | 0.575 |  |  | 0.25 | 17 |
| `vfx/effekseer-magicalxspiral/Sylph3.ogt` | Sylph 3 | magic-impact | wind | 0.45 |  |  | 0.16 | 18 |
| `vfx/effekseer-magicalxspiral/Sylph4.ogt` | Sylph 4 | magic-impact | wind | 0.583 |  |  | 0.09 | 18 |
| `vfx/effekseer-magicalxspiral/Sylph5.ogt` | Sylph 5 | magic-impact | wind | 2.7 |  |  | 0.17 | 22 |
| `vfx/effekseer-magicalxspiral/Sylph6.ogt` | Sylph 6 | magic-impact | wind | 0.367 |  |  | 0.02 | 15 |
| `vfx/effekseer-magicalxspiral/Sylph7.ogt` | Sylph 7 | magic-impact | wind | 1.667 |  |  | 1.00 | 26 |
| `vfx/effekseer-magicalxspiral/Sylph8.ogt` | Sylph 8 | magic-impact | wind | 1.167 |  |  | 0.13 | 20 |
| `vfx/effekseer-magicalxspiral/Thunder1.ogt` | Thunder 1 | magic-impact | lightning | 2.15 |  |  | 0.51 | 19 |
| `vfx/effekseer-magicalxspiral/Thunder2.ogt` | Thunder 2 | magic-impact | lightning | 0.817 |  |  | 0.65 | 30 |
| `vfx/effekseer-magicalxspiral/Thunder3.ogt` | Thunder 3 | magic-impact | lightning | 0.817 |  |  | 0.32 | 17 |
| `vfx/effekseer-magicalxspiral/Topwater1.ogt` | Topwater 1 | magic-impact | water | 0.65 |  |  | 0.49 | 17 |
| `vfx/effekseer-magicalxspiral/Undine1.ogt` | Undine 1 | magic-impact | water | 1 |  |  | 0.62 | 30 |
| `vfx/effekseer-magicalxspiral/Undine11.ogt` | Undine 11 | magic-impact | water | 10.083 |  |  | 0.04 | 17 |
| `vfx/effekseer-magicalxspiral/Undine13.ogt` | Undine 13 | magic-impact | water | 3.917 |  |  | 0.14 | 18 |
| `vfx/effekseer-magicalxspiral/Undine2.ogt` | Undine 2 | magic-impact | water | 0.45 |  |  | 0.24 | 18 |
| `vfx/effekseer-magicalxspiral/Undine3.ogt` | Undine 3 | magic-impact | water | 0.583 |  |  | 0.09 | 18 |
| `vfx/effekseer-magicalxspiral/Undine4.ogt` | Undine 4 | magic-impact | water | 1.667 |  |  | 0.51 | 18 |
| `vfx/effekseer-magicalxspiral/Undine5.ogt` | Undine 5 | magic-impact | water | 0.667 |  |  | 0.59 | 26 |
| `vfx/effekseer-magicalxspiral/Undine6.ogt` | Undine 6 | magic-impact | water | 1.833 |  |  | 0.32 | 17 |
| `vfx/effekseer-magicalxspiral/Undine8.ogt` | Undine 8 | magic-impact | water | 1.667 |  |  | 1.15 | 26 |
| `vfx/effekseer-magicalxspiral/WHead1.ogt` | WHead 1 | magic-impact | water | 0.417 |  |  | 0.07 | 16 |
| `vfx/effekseer-magicalxspiral/WHead2.ogt` | WHead 2 | magic-impact | water | 0.567 |  |  | 0.19 | 16 |
| `vfx/effekseer-magicalxspiral/WHead3.ogt` | WHead 3 | magic-impact | water | 0.567 |  |  | 0.29 | 25 |
| `vfx/effekseer-magicalxspiral/Water_Impact.ogt` | Water Impact | magic-impact | water | 0.65 |  |  | 0.32 | 25 |
| `vfx/effekseer-magicalxspiral/Wind_Impact.ogt` | Wind Impact | magic-impact | wind | 0.4 |  |  | 0.19 | 21 |
| `vfx/effekseer-magicalxspiral/foot_smoke1.ogt` | Foot Smoke 1 | smoke |  | 0.25 |  |  | 0.15 | 17 |
| `vfx/effekseer-magicalxspiral/foot_smoke2.ogt` | Foot Smoke 2 | smoke |  | 1.233 |  |  | 0.19 | 16 |
| `vfx/effekseer-magicalxspiral/foot_smoke3.ogt` | Foot Smoke 3 | smoke |  | 0.9 |  |  | 0.17 | 15 |
| `vfx/effekseer-magicalxspiral/foot_smoke4.ogt` | Foot Smoke 4 | smoke |  | 0.9 |  |  | 0.15 | 15 |
| `vfx/effekseer-magicalxspiral/foot_smoke5.ogt` | Foot Smoke 5 | smoke |  | 0.9 |  |  | 0.57 | 15 |
| `vfx/effekseer-magicalxspiral/foot_smoke6.ogt` | Foot Smoke 6 | smoke |  | 0.567 |  |  | 0.18 | 15 |
| `vfx/effekseer-magicalxspiral/foot_smoke7.ogt` | Foot Smoke 7 | smoke |  | 0.9 |  |  | 0.10 | 15 |
| `vfx/effekseer-magicalxspiral/foot_smoke8.ogt` | Foot Smoke 8 | smoke |  | 0.9 |  |  | 0.45 | 15 |
| `vfx/effekseer-magicalxspiral/meteo1.ogt` | Meteo 1 | magic-projectile | fire | 0.858 |  |  | 0.54 | 23 |
| `vfx/effekseer-magicalxspiral/salamander13.ogt` | Salamander 13 | magic-impact | fire | 1.683 |  |  | 0.34 | 24 |

## 7. effekseer-nextsoft01

NextSoft01's Effekseer sample effects (20), converted to type vfx. 20 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-nextsoft01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, magic-impact, magic-projectile, shield, slash, weather
- Elements: dark, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, nextsoft01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-nextsoft01/HealPotion1.ogt` | Heal Potion 1 | heal | holy | 1.267 |  |  | 0.23 | 17 |
| `vfx/effekseer-nextsoft01/HealPotion2.ogt` | Heal Potion 2 | heal | holy | 1.63 |  |  | 0.50 | 22 |
| `vfx/effekseer-nextsoft01/MagicCold.ogt` | Magic Cold | magic-impact | ice | 2.15 |  |  | 0.40 | 17 |
| `vfx/effekseer-nextsoft01/MagicDark.ogt` | Magic Dark | magic-impact | dark | 2.05 |  |  | 0.97 | 19 |
| `vfx/effekseer-nextsoft01/MagicFire1.ogt` | Magic Fire 1 | fire | fire | 2.329 |  |  | 0.91 | 15 |
| `vfx/effekseer-nextsoft01/MagicFire2.ogt` | Magic Fire 2 | fire | fire | 1.91 |  |  | 0.29 | 15 |
| `vfx/effekseer-nextsoft01/MagicHeal1.ogt` | Magic Heal 1 | heal | holy | 1.967 |  |  | 0.73 | 17 |
| `vfx/effekseer-nextsoft01/MagicHeal2.ogt` | Magic Heal 2 | heal | holy | 4 |  |  | 0.96 | 19 |
| `vfx/effekseer-nextsoft01/MagicLight.ogt` | Magic Light | magic-impact | holy | 1.317 |  |  | 0.38 | 18 |
| `vfx/effekseer-nextsoft01/MagicMeteo.ogt` | Magic Meteo | magic-projectile | fire | 2.417 |  |  | 0.52 | 21 |
| `vfx/effekseer-nextsoft01/MagicShield.ogt` | Magic Shield | shield |  | 0.183 |  |  | 0.04 | 21 |
| `vfx/effekseer-nextsoft01/MagicSword.ogt` | Magic Sword | slash |  | 1.733 |  |  | 0.59 | 22 |
| `vfx/effekseer-nextsoft01/MagicThunder.ogt` | Magic Thunder | magic-impact | lightning | 1.05 |  |  | 1.28 | 25 |
| `vfx/effekseer-nextsoft01/MagicTornade.ogt` | Magic Tornade | weather | wind | 0.817 |  |  | 0.47 | 15 |
| `vfx/effekseer-nextsoft01/MagicWater.ogt` | Magic Water | magic-impact | water | 2.09 |  |  | 2.17 | 23 |
| `vfx/effekseer-nextsoft01/PowerUp.ogt` | Power Up | buff |  | 1.75 |  |  | 0.30 | 18 |
| `vfx/effekseer-nextsoft01/Spear1.ogt` | Spear 1 | magic-projectile |  | 0.367 |  |  | 0.55 | 17 |
| `vfx/effekseer-nextsoft01/Spear2.ogt` | Spear 2 | magic-projectile |  | 0.983 |  |  | 0.23 | 16 |
| `vfx/effekseer-nextsoft01/Sword1.ogt` | Sword 1 | slash |  | 0.333 |  |  | 0.19 | 17 |
| `vfx/effekseer-nextsoft01/Sword2.ogt` | Sword 2 | slash |  | 1.1 |  |  | 1.33 | 19 |

## 8. effekseer-pierre01

Pierre vol.1's Effekseer sample effects (11), converted to type vfx. 11 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre01`.

- Style: stylised, 3d
- Categories: ambient, buff, fire, magic-impact, magic-projectile, weather
- Elements: fire, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-pierre01/Atmosphere.ogt` | Atmosphere | ambient |  | loops |  |  | 0.81 | 27 |
| `vfx/effekseer-pierre01/Background.ogt` | Background | ambient |  | 19.333 |  |  | 0.33 | 24 |
| `vfx/effekseer-pierre01/Flame.ogt` | Flame | fire | fire | 5.283 |  |  | 6.39 | 62 |
| `vfx/effekseer-pierre01/Herald.ogt` | Herald | buff |  | 2.083 |  |  | 1.81 | 81 |
| `vfx/effekseer-pierre01/HolySandstorm.ogt` | Holy Sandstorm | weather | wind | 6.733 |  |  | 7.59 | 27 |
| `vfx/effekseer-pierre01/LightningStrike.ogt` | Lightning Strike | magic-impact | lightning | 2.783 |  |  | 6.37 | 55 |
| `vfx/effekseer-pierre01/MissileBoost.ogt` | Missile Boost | magic-projectile |  | loops |  |  | 5.82 | 26 |
| `vfx/effekseer-pierre01/MonsterRoar.ogt` | Monster Roar | magic-impact |  | 2.417 |  |  | 1.99 | 35 |
| `vfx/effekseer-pierre01/PhantasmMeteor.ogt` | Phantasm Meteor | magic-projectile | fire | 4.15 |  |  | 8.23 | 62 |
| `vfx/effekseer-pierre01/PhantasmMeteor_Single.ogt` | Phantasm Meteor Single | magic-projectile | fire | 3.612 |  |  | 5.55 | 46 |
| `vfx/effekseer-pierre01/SonicBoom.ogt` | Sonic Boom | magic-projectile |  | 1.833 |  |  | 2.31 | 31 |

## 9. effekseer-pierre02

Pierre vol.2's Effekseer sample effects (12), converted to type vfx. 12 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre02`.

- Style: stylised, 3d
- Categories: explosion, heal, magic-impact, magic-projectile, portal, slash, smoke
- Elements: dark, fire, holy, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre02

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-pierre02/Benediction.ogt` | Benediction | heal | holy | 5.933 |  |  | 9.71 | 47 |
| `vfx/effekseer-pierre02/BloodLance.ogt` | Blood Lance | magic-projectile | dark | loops |  |  | 3.91 | 45 |
| `vfx/effekseer-pierre02/CosmicMist.ogt` | Cosmic Mist | smoke | dark | 7.267 |  |  | 16.08 | 56 |
| `vfx/effekseer-pierre02/FeatherBomb.ogt` | Feather Bomb | explosion |  | 13.006 |  |  | 12.71 | 55 |
| `vfx/effekseer-pierre02/FireBall.ogt` | Fire Ball | magic-projectile | fire | 5.517 |  |  | 6.56 | 49 |
| `vfx/effekseer-pierre02/GateOfCalve.ogt` | Gate Of Calve | portal |  | 14.083 |  |  | 2.62 | 35 |
| `vfx/effekseer-pierre02/LossOfBlood.ogt` | Loss Of Blood | magic-impact | dark | 1.85 |  |  | 36.30 | 47 |
| `vfx/effekseer-pierre02/Sword_Ember.ogt` | Sword Ember | slash | fire | loops |  |  | 1.04 | 26 |
| `vfx/effekseer-pierre02/Sword_Lightning.ogt` | Sword Lightning | slash | lightning | loops |  |  | 1.47 | 26 |
| `vfx/effekseer-pierre02/Sword_Moonlight.ogt` | Sword Moonlight | slash | holy | loops |  |  | 1.01 | 26 |
| `vfx/effekseer-pierre02/Sword_Sacredlight.ogt` | Sword Sacredlight | slash | holy | loops |  |  | 1.84 | 24 |
| `vfx/effekseer-pierre02/Sword_Storm.ogt` | Sword Storm | slash | wind | loops |  |  | 3.63 | 28 |

## 10. effekseer-suzuki01

Suzuki01's Effekseer sample effects (3), converted to type vfx. 3 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-suzuki01`.

- Style: stylised, 3d
- Categories: buff, slash, weather
- Elements: ice
- Keywords: 3d, anime, effekseer, particles, suzuki01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-suzuki01/aura.ogt` | Aura | buff |  | loops |  |  | 1.29 | 29 |
| `vfx/effekseer-suzuki01/snowstorm11.ogt` | Snowstorm 11 | weather | ice | 8.917 |  |  | 4.00 | 24 |
| `vfx/effekseer-suzuki01/sword.ogt` | Sword | slash |  | loops |  |  | 1.33 | 30 |

## 11. effekseer-tktk01

tktk vol.1's Effekseer sample effects (71), converted to type vfx. 71 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, earth, fire, holy, wind
- Keywords: 3d, anime, effekseer, particles, tktk01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-tktk01/Arrow1.ogt` | Arrow 1 | magic-projectile |  | 0.454 |  |  | 0.30 | 29 |
| `vfx/effekseer-tktk01/Arrow2.ogt` | Arrow 2 | magic-projectile |  | 2.092 |  |  | 1.39 | 50 |
| `vfx/effekseer-tktk01/Blow1.ogt` | Blow 1 | hit-spark |  | 0.775 |  |  | 5.87 | 17 |
| `vfx/effekseer-tktk01/Blow10.ogt` | Blow 10 | hit-spark |  | 0.867 |  |  | 4.46 | 21 |
| `vfx/effekseer-tktk01/Blow11.ogt` | Blow 11 | hit-spark |  | 0.65 |  |  | 3.95 | 30 |
| `vfx/effekseer-tktk01/Blow12.ogt` | Blow 12 | hit-spark |  | 1.816 |  |  | 2.92 | 29 |
| `vfx/effekseer-tktk01/Blow13.ogt` | Blow 13 | hit-spark |  | 0.832 |  |  | 3.35 | 33 |
| `vfx/effekseer-tktk01/Blow14.ogt` | Blow 14 | hit-spark |  | 1.4 |  |  | 3.99 | 30 |
| `vfx/effekseer-tktk01/Blow15.ogt` | Blow 15 | hit-spark |  | 0.908 |  |  | 5.74 | 41 |
| `vfx/effekseer-tktk01/Blow2.ogt` | Blow 2 | hit-spark |  | 0.533 |  |  | 1.27 | 19 |
| `vfx/effekseer-tktk01/Blow3.ogt` | Blow 3 | hit-spark |  | 0.583 |  |  | 1.82 | 23 |
| `vfx/effekseer-tktk01/Blow4.ogt` | Blow 4 | hit-spark |  | 0.845 |  |  | 2.86 | 21 |
| `vfx/effekseer-tktk01/Blow5.ogt` | Blow 5 | hit-spark |  | 1.775 |  |  | 1.52 | 25 |
| `vfx/effekseer-tktk01/Blow6.ogt` | Blow 6 | hit-spark |  | 2.367 |  |  | 1.28 | 16 |
| `vfx/effekseer-tktk01/Blow7.ogt` | Blow 7 | hit-spark |  | 1.678 |  |  | 1.20 | 21 |
| `vfx/effekseer-tktk01/Blow8.ogt` | Blow 8 | hit-spark |  | 3.017 |  |  | 3.16 | 25 |
| `vfx/effekseer-tktk01/Blow9.ogt` | Blow 9 | hit-spark |  | 1.667 |  |  | 3.81 | 29 |
| `vfx/effekseer-tktk01/Cure1.ogt` | Cure 1 | heal | holy | 1.275 |  |  | 1.25 | 24 |
| `vfx/effekseer-tktk01/Cure2.ogt` | Cure 2 | heal | holy | 1.779 |  |  | 1.55 | 26 |
| `vfx/effekseer-tktk01/Cure3.ogt` | Cure 3 | heal | holy | 1.252 |  |  | 3.69 | 18 |
| `vfx/effekseer-tktk01/Cure4.ogt` | Cure 4 | heal | holy | 0.967 |  |  | 0.68 | 16 |
| `vfx/effekseer-tktk01/Cure5.ogt` | Cure 5 | heal | holy | 1.433 |  |  | 5.35 | 23 |
| `vfx/effekseer-tktk01/Cure6.ogt` | Cure 6 | heal | holy | 1.251 |  |  | 2.26 | 18 |
| `vfx/effekseer-tktk01/Cure7.ogt` | Cure 7 | heal | holy | 2.6 |  |  | 1.01 | 40 |
| `vfx/effekseer-tktk01/Dark1.ogt` | Dark 1 | magic-impact | dark | 1.617 |  |  | 6.79 | 18 |
| `vfx/effekseer-tktk01/Dark2.ogt` | Dark 2 | magic-impact | dark | 1.7 |  |  | 1.37 | 24 |
| `vfx/effekseer-tktk01/Dark3.ogt` | Dark 3 | magic-impact | dark | 3.333 |  |  | 2.54 | 20 |
| `vfx/effekseer-tktk01/Fire1.ogt` | Fire 1 | fire | fire | loops |  |  | 4.50 | 20 |
| `vfx/effekseer-tktk01/Fire2.ogt` | Fire 2 | fire | fire | 3.013 |  |  | 4.00 | 19 |
| `vfx/effekseer-tktk01/Fire3.ogt` | Fire 3 | fire | fire | loops |  |  | 3.87 | 23 |
| `vfx/effekseer-tktk01/Fire4.ogt` | Fire 4 | fire | fire | 2.425 |  |  | 6.32 | 23 |
| `vfx/effekseer-tktk01/Fire5.ogt` | Fire 5 | fire | fire | 1.442 |  |  | 2.66 | 21 |
| `vfx/effekseer-tktk01/Fire6.ogt` | Fire 6 | fire | fire | 1.251 |  |  | 3.50 | 31 |
| `vfx/effekseer-tktk01/Fire7.ogt` | Fire 7 | fire | fire | loops |  |  | 3.31 | 26 |
| `vfx/effekseer-tktk01/Fire8.ogt` | Fire 8 | fire | fire | 1.925 |  |  | 4.80 | 22 |
| `vfx/effekseer-tktk01/Light1.ogt` | Light 1 | magic-impact | holy | 2.667 |  |  | 0.89 | 26 |
| `vfx/effekseer-tktk01/Light2.ogt` | Light 2 | magic-impact | holy | 0.55 |  |  | 3.54 | 33 |
| `vfx/effekseer-tktk01/Light3.ogt` | Light 3 | magic-impact | holy | 1 |  |  | 0.53 | 20 |
| `vfx/effekseer-tktk01/Light4.ogt` | Light 4 | magic-impact | holy | 2.092 |  |  | 3.56 | 78 |
| `vfx/effekseer-tktk01/Magic1.ogt` | Magic 1 | magic-impact |  | 1 |  |  | 4.65 | 29 |
| `vfx/effekseer-tktk01/Magic2.ogt` | Magic 2 | magic-impact |  | 2.471 |  |  | 4.00 | 34 |
| `vfx/effekseer-tktk01/Soil1.ogt` | Soil 1 | magic-impact | earth | loops |  |  | 4.50 | 29 |
| `vfx/effekseer-tktk01/Sword1.ogt` | Sword 1 | slash |  | 0.8 |  |  | 0.07 | 23 |
| `vfx/effekseer-tktk01/Sword10.ogt` | Sword 10 | slash |  | 0.837 |  |  | 0.26 | 17 |
| `vfx/effekseer-tktk01/Sword11.ogt` | Sword 11 | slash |  | 1.283 |  |  | 5.84 | 20 |
| `vfx/effekseer-tktk01/Sword12.ogt` | Sword 12 | slash |  | 0.797 |  |  | 1.80 | 27 |
| `vfx/effekseer-tktk01/Sword13.ogt` | Sword 13 | slash |  | 2.592 |  |  | 3.82 | 26 |
| `vfx/effekseer-tktk01/Sword14.ogt` | Sword 14 | slash |  | 2.302 |  |  | 1.76 | 22 |
| `vfx/effekseer-tktk01/Sword15.ogt` | Sword 15 | slash |  | 2.275 |  |  | 3.76 | 22 |
| `vfx/effekseer-tktk01/Sword16.ogt` | Sword 16 | slash |  | loops |  |  | 4.98 | 71 |
| `vfx/effekseer-tktk01/Sword17.ogt` | Sword 17 | slash |  | 1.068 |  |  | 5.47 | 25 |
| `vfx/effekseer-tktk01/Sword18.ogt` | Sword 18 | slash |  | 2.933 |  |  | 4.64 | 31 |
| `vfx/effekseer-tktk01/Sword19.ogt` | Sword 19 | slash |  | 2.49 |  |  | 5.38 | 52 |
| `vfx/effekseer-tktk01/Sword2.ogt` | Sword 2 | slash |  | 0.698 |  |  | 7.62 | 21 |
| `vfx/effekseer-tktk01/Sword20.ogt` | Sword 20 | slash |  | 2.867 |  |  | 3.81 | 23 |
| `vfx/effekseer-tktk01/Sword21.ogt` | Sword 21 | slash |  | 3.333 |  |  | 3.19 | 30 |
| `vfx/effekseer-tktk01/Sword22.ogt` | Sword 22 | slash |  | 1.983 |  |  | 4.35 | 28 |
| `vfx/effekseer-tktk01/Sword3.ogt` | Sword 3 | slash |  | 1.032 |  |  | 1.28 | 23 |
| `vfx/effekseer-tktk01/Sword4.ogt` | Sword 4 | slash |  | 1.333 |  |  | 1.18 | 22 |
| `vfx/effekseer-tktk01/Sword5.ogt` | Sword 5 | slash |  | 0.6 |  |  | 2.74 | 17 |
| `vfx/effekseer-tktk01/Sword6.ogt` | Sword 6 | slash |  | 0.682 |  |  | 1.23 | 22 |
| `vfx/effekseer-tktk01/Sword7.ogt` | Sword 7 | slash |  | 0.583 |  |  | 0.93 | 18 |
| `vfx/effekseer-tktk01/Sword8.ogt` | Sword 8 | slash |  | 1.417 |  |  | 1.14 | 21 |
| `vfx/effekseer-tktk01/Sword9.ogt` | Sword 9 | slash |  | 0.867 |  |  | 1.04 | 21 |
| `vfx/effekseer-tktk01/Wind1.ogt` | Wind 1 | magic-impact | wind | 2.967 |  |  | 3.70 | 18 |
| `vfx/effekseer-tktk01/Wind2.ogt` | Wind 2 | magic-impact | wind | 0.673 |  |  | 1.28 | 26 |
| `vfx/effekseer-tktk01/hozyo1.ogt` | Hozyo 1 | buff |  | 1.291 |  |  | 4.84 | 28 |
| `vfx/effekseer-tktk01/hozyo2.ogt` | Hozyo 2 | buff |  | 1.217 |  |  | 1.09 | 30 |
| `vfx/effekseer-tktk01/hozyo3.ogt` | Hozyo 3 | buff |  | 0.8 |  |  | 1.06 | 26 |
| `vfx/effekseer-tktk01/hozyo4.ogt` | Hozyo 4 | buff |  | loops |  |  | 4.68 | 24 |
| `vfx/effekseer-tktk01/hozyo5.ogt` | Hozyo 5 | buff |  | loops |  |  | 4.63 | 25 |

## 12. effekseer-tktk02

tktk vol.2's Effekseer sample effects (33), converted to type vfx. 33 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk02`.

- Style: stylised, 3d
- Categories: fire, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, fire, holy
- Keywords: 3d, anime, effekseer, particles, tktk02

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/effekseer-tktk02/Blow1.ogt` | Blow 1 | hit-spark |  | 1.167 |  |  | 1.61 | 36 |
| `vfx/effekseer-tktk02/Blow2.ogt` | Blow 2 | hit-spark |  | 2.383 |  |  | 2.29 | 40 |
| `vfx/effekseer-tktk02/Blow3.ogt` | Blow 3 | hit-spark |  | 2.4 |  |  | 4.85 | 29 |
| `vfx/effekseer-tktk02/Blow4.ogt` | Blow 4 | hit-spark |  | 1 |  |  | 3.04 | 21 |
| `vfx/effekseer-tktk02/Blow5.ogt` | Blow 5 | hit-spark |  | 1.332 |  |  | 5.44 | 60 |
| `vfx/effekseer-tktk02/Blow6.ogt` | Blow 6 | hit-spark |  | 3.6 |  |  | 5.65 | 78 |
| `vfx/effekseer-tktk02/Claw1.ogt` | Claw 1 | slash |  | loops |  |  | 3.39 | 37 |
| `vfx/effekseer-tktk02/Claw2.ogt` | Claw 2 | slash |  | loops |  |  | 2.65 | 46 |
| `vfx/effekseer-tktk02/Claw3.ogt` | Claw 3 | slash |  | 2.733 |  |  | 4.44 | 63 |
| `vfx/effekseer-tktk02/Claw4.ogt` | Claw 4 | slash |  | loops |  |  | 7.20 | 80 |
| `vfx/effekseer-tktk02/Dark1.ogt` | Dark 1 | magic-impact | dark | 1.165 |  |  | 4.95 | 31 |
| `vfx/effekseer-tktk02/Dark2.ogt` | Dark 2 | magic-impact | dark | 3.033 |  |  | 3.80 | 53 |
| `vfx/effekseer-tktk02/Fire1.ogt` | Fire 1 | fire | fire | 2.967 |  |  | 2.60 | 24 |
| `vfx/effekseer-tktk02/Fire2.ogt` | Fire 2 | fire | fire | 3.317 |  |  | 3.85 | 29 |
| `vfx/effekseer-tktk02/Fire3.ogt` | Fire 3 | fire | fire | 1.3 |  |  | 4.86 | 31 |
| `vfx/effekseer-tktk02/Fire4.ogt` | Fire 4 | fire | fire | loops |  |  | 4.24 | 37 |
| `vfx/effekseer-tktk02/Fire5.ogt` | Fire 5 | fire | fire | 2.533 |  |  | 6.47 | 66 |
| `vfx/effekseer-tktk02/Fire6.ogt` | Fire 6 | fire | fire | 1.371 |  |  | 7.19 | 41 |
| `vfx/effekseer-tktk02/Fire7.ogt` | Fire 7 | fire | fire | 2.998 |  |  | 8.90 | 30 |
| `vfx/effekseer-tktk02/Fire8.ogt` | Fire 8 | fire | fire | loops |  |  | 3.92 | 23 |
| `vfx/effekseer-tktk02/Gun1.ogt` | Gun 1 | magic-projectile |  | 2.036 |  |  | 4.14 | 39 |
| `vfx/effekseer-tktk02/Gun2.ogt` | Gun 2 | magic-projectile |  | 2.968 |  |  | 3.02 | 48 |
| `vfx/effekseer-tktk02/Gun3.ogt` | Gun 3 | magic-projectile |  | 2.169 |  |  | 1.72 | 28 |
| `vfx/effekseer-tktk02/Gun4.ogt` | Gun 4 | magic-projectile |  | loops |  |  | 3.28 | 65 |
| `vfx/effekseer-tktk02/Gun5.ogt` | Gun 5 | magic-projectile |  | 1.733 |  |  | 4.08 | 35 |
| `vfx/effekseer-tktk02/Gun6.ogt` | Gun 6 | magic-projectile |  | 2.217 |  |  | 2.31 | 39 |
| `vfx/effekseer-tktk02/Gun7.ogt` | Gun 7 | magic-projectile |  | 3.2 |  |  | 13.91 | 54 |
| `vfx/effekseer-tktk02/Gun8.ogt` | Gun 8 | magic-projectile |  | 2.1 |  |  | 0.52 | 28 |
| `vfx/effekseer-tktk02/Gun9.ogt` | Gun 9 | magic-projectile |  | 3.067 |  |  | 1.91 | 48 |
| `vfx/effekseer-tktk02/Light1.ogt` | Light 1 | magic-impact | holy | 1.082 |  |  | 3.57 | 41 |
| `vfx/effekseer-tktk02/Light2.ogt` | Light 2 | magic-impact | holy | 2.6 |  |  | 3.18 | 50 |
| `vfx/effekseer-tktk02/Light3.ogt` | Light 3 | magic-impact | holy | loops |  |  | 6.40 | 93 |
| `vfx/effekseer-tktk02/Light4.ogt` | Light 4 | magic-impact | holy | 2.777 |  |  | 3.69 | 78 |

## 13. Ambient effects

Ambient effects (ambient): 12 in `effekseer-hato01`, `effekseer-pierre01`, `effekseer-starter`, `pixel-scenario`.

- effekseer-hato01: `vfx/effekseer-hato01/aoi-chou.ogt` (Aoi Chou), `vfx/effekseer-hato01/iwai-ume.ogt` (Iwai Ume), `vfx/effekseer-hato01/kiroi-chou.ogt` (Kiroi Chou), `vfx/effekseer-hato01/sakura1.ogt` (Sakura 1)
- effekseer-pierre01: `vfx/effekseer-pierre01/Atmosphere.ogt` (Atmosphere), `vfx/effekseer-pierre01/Background.ogt` (Background)
- effekseer-starter: `vfx/effekseer-starter/electric_dust.ogt` (Floating Motes), `vfx/effekseer-starter/sakura1.ogt` (Sakura Petals)
- pixel-scenario: `vfx/pixel-scenario/cherry-blossom-petals.ogt` (Thorn Ambient), `vfx/pixel-scenario/embers-rising.ogt` (Blaze Ambient), `vfx/pixel-scenario/falling-leaves.ogt` (Bloom Ambient), `vfx/pixel-scenario/fireflies.ogt` (Mystic Ambient)

## 14. Basic effects

Basic effects (basic): 11 in `effekseer-effekseer01`.

- effekseer-effekseer01: `vfx/effekseer-effekseer01/Simple_Distortion.ogt` (Simple Distortion), `vfx/effekseer-effekseer01/Simple_GeneratingPosition1.ogt` (Simple Generating Position 1), `vfx/effekseer-effekseer01/Simple_Model_UV.ogt` (Simple Model UV), `vfx/effekseer-effekseer01/Simple_Ribbon_Parent.ogt` (Simple Ribbon Parent), `vfx/effekseer-effekseer01/Simple_Ribbon_Sword.ogt` (Simple Ribbon Sword), `vfx/effekseer-effekseer01/Simple_Ring_Shape1.ogt` (Simple Ring Shape 1), `vfx/effekseer-effekseer01/Simple_Ring_Shape2.ogt` (Simple Ring Shape 2), `vfx/effekseer-effekseer01/Simple_Sprite_BillBoard.ogt` (Simple Sprite Bill Board), `vfx/effekseer-effekseer01/Simple_Sprite_Fixed.ogt` (Simple Sprite Fixed), `vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis.ogt` (Simple Sprite Fixed YAxis), `vfx/effekseer-effekseer01/Simple_Track1.ogt` (Simple Track 1)

## 15. Beam effects

Beam effects (beam): 5 in `effekseer-andrewfm01`, `effekseer-effekseer01`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/blue_laser.ogt` (Blue Laser)
- effekseer-effekseer01: `vfx/effekseer-effekseer01/Homing_Laser01.ogt` (Homing Laser 01), `vfx/effekseer-effekseer01/Laser01.ogt` (Laser 01), `vfx/effekseer-effekseer01/Laser02.ogt` (Laser 02), `vfx/effekseer-effekseer01/Laser03.ogt` (Laser 03)

## 16. Buff effects

Buff effects (buff): 14 in `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `pixel-scenario`.

- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/PowerUp.ogt` (Power Up)
- effekseer-pierre01: `vfx/effekseer-pierre01/Herald.ogt` (Herald)
- effekseer-starter: `vfx/effekseer-starter/PowerUp.ogt` (Power Up), `vfx/effekseer-starter/hozyo2.ogt` (Power Rings), `vfx/effekseer-starter/hozyo4.ogt` (Attack Up)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/aura.ogt` (Aura)
- effekseer-tktk01: `vfx/effekseer-tktk01/hozyo1.ogt` (Hozyo 1), `vfx/effekseer-tktk01/hozyo2.ogt` (Hozyo 2), `vfx/effekseer-tktk01/hozyo3.ogt` (Hozyo 3), `vfx/effekseer-tktk01/hozyo4.ogt` (Hozyo 4), `vfx/effekseer-tktk01/hozyo5.ogt` (Hozyo 5)
- pixel-scenario: `vfx/pixel-scenario/charge-up-energy.ogt` (Pulse Charge), `vfx/pixel-scenario/dark-shadow-aura.ogt` (Night Aura), `vfx/pixel-scenario/hex-shield-barrier.ogt` (Flux Shield)

## 17. Explosion effects

Explosion effects (explosion): 13 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/boss_death.ogt` (Boss Death)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Breakdown.ogt` (Breakdown), `vfx/effekseer-magicalxspiral/IcicleBroken.ogt` (Icicle Broken), `vfx/effekseer-magicalxspiral/StairBroken.ogt` (Stair Broken), `vfx/effekseer-magicalxspiral/StairBroken3.ogt` (Stair Broken 3)
- effekseer-pierre02: `vfx/effekseer-pierre02/FeatherBomb.ogt` (Feather Bomb)
- effekseer-starter: `vfx/effekseer-starter/Blow4.ogt` (Bright Explosion), `vfx/effekseer-starter/Breakdown.ogt` (Breakdown Burst), `vfx/effekseer-starter/FeatherBomb.ogt` (Smoke Bomb), `vfx/effekseer-starter/boss_death.ogt` (Boss Death), `vfx/effekseer-starter/meteo1.ogt` (Meteor Strike)
- pixel-scenario: `vfx/pixel-scenario/big-fire-explosion.ogt` (Cinder Explosion), `vfx/pixel-scenario/meteor-strike.ogt` (Ember Meteor)

## 18. Fire effects

Fire effects (fire): 29 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/fire.ogt` (Fire)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/FirePot1.ogt` (Fire Pot 1), `vfx/effekseer-magicalxspiral/FirePot2.ogt` (Fire Pot 2), `vfx/effekseer-magicalxspiral/Magic1_fire.ogt` (Magic 1 Fire), `vfx/effekseer-magicalxspiral/Magic2_fire.ogt` (Magic 2 Fire)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicFire1.ogt` (Magic Fire 1), `vfx/effekseer-nextsoft01/MagicFire2.ogt` (Magic Fire 2)
- effekseer-pierre01: `vfx/effekseer-pierre01/Flame.ogt` (Flame)
- effekseer-starter: `vfx/effekseer-starter/Fire7.ogt` (Fire Pillar), `vfx/effekseer-starter/MagicFire2.ogt` (Fire Breath)
- effekseer-tktk01: `vfx/effekseer-tktk01/Fire1.ogt` (Fire 1), `vfx/effekseer-tktk01/Fire2.ogt` (Fire 2), `vfx/effekseer-tktk01/Fire3.ogt` (Fire 3), `vfx/effekseer-tktk01/Fire4.ogt` (Fire 4), `vfx/effekseer-tktk01/Fire5.ogt` (Fire 5), `vfx/effekseer-tktk01/Fire6.ogt` (Fire 6), `vfx/effekseer-tktk01/Fire7.ogt` (Fire 7), `vfx/effekseer-tktk01/Fire8.ogt` (Fire 8)
- effekseer-tktk02: `vfx/effekseer-tktk02/Fire1.ogt` (Fire 1), `vfx/effekseer-tktk02/Fire2.ogt` (Fire 2), `vfx/effekseer-tktk02/Fire3.ogt` (Fire 3), `vfx/effekseer-tktk02/Fire4.ogt` (Fire 4), `vfx/effekseer-tktk02/Fire5.ogt` (Fire 5), `vfx/effekseer-tktk02/Fire6.ogt` (Fire 6), `vfx/effekseer-tktk02/Fire7.ogt` (Fire 7), `vfx/effekseer-tktk02/Fire8.ogt` (Fire 8)
- pixel-scenario: `vfx/pixel-scenario/campfire.ogt` (Ember Flame), `vfx/pixel-scenario/dragon-fire-breath.ogt` (Cinder Breath), `vfx/pixel-scenario/fire-tornado.ogt` (Blaze Tornado)

## 19. Heal effects

Heal effects (heal): 17 in `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `pixel-scenario`.

- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/HealPotion1.ogt` (Heal Potion 1), `vfx/effekseer-nextsoft01/HealPotion2.ogt` (Heal Potion 2), `vfx/effekseer-nextsoft01/MagicHeal1.ogt` (Magic Heal 1), `vfx/effekseer-nextsoft01/MagicHeal2.ogt` (Magic Heal 2)
- effekseer-pierre02: `vfx/effekseer-pierre02/Benediction.ogt` (Benediction)
- effekseer-starter: `vfx/effekseer-starter/Cure1.ogt` (Star Cure), `vfx/effekseer-starter/Cure7.ogt` (Light Cure), `vfx/effekseer-starter/HealPotion2.ogt` (Potion), `vfx/effekseer-starter/MagicHeal2.ogt` (Healing Circle)
- effekseer-tktk01: `vfx/effekseer-tktk01/Cure1.ogt` (Cure 1), `vfx/effekseer-tktk01/Cure2.ogt` (Cure 2), `vfx/effekseer-tktk01/Cure3.ogt` (Cure 3), `vfx/effekseer-tktk01/Cure4.ogt` (Cure 4), `vfx/effekseer-tktk01/Cure5.ogt` (Cure 5), `vfx/effekseer-tktk01/Cure6.ogt` (Cure 6), `vfx/effekseer-tktk01/Cure7.ogt` (Cure 7)
- pixel-scenario: `vfx/pixel-scenario/healing-with-rising-crosses.ogt` (Bloom Heal)

## 20. Hit spark effects

Hit spark effects (hit-spark): 27 in `effekseer-andrewfm01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/hit_eff.ogt` (Hit Eff)
- effekseer-starter: `vfx/effekseer-starter/Attack_Impact.ogt` (Attack Impact), `vfx/effekseer-starter/Blow2.ogt` (Spark Hit), `vfx/effekseer-starter/Blow8.ogt` (Star Hit), `vfx/effekseer-starter/hit_eff.ogt` (Hit Flash)
- effekseer-tktk01: `vfx/effekseer-tktk01/Blow1.ogt` (Blow 1), `vfx/effekseer-tktk01/Blow10.ogt` (Blow 10), `vfx/effekseer-tktk01/Blow11.ogt` (Blow 11), `vfx/effekseer-tktk01/Blow12.ogt` (Blow 12), `vfx/effekseer-tktk01/Blow13.ogt` (Blow 13), `vfx/effekseer-tktk01/Blow14.ogt` (Blow 14), `vfx/effekseer-tktk01/Blow15.ogt` (Blow 15), `vfx/effekseer-tktk01/Blow2.ogt` (Blow 2), `vfx/effekseer-tktk01/Blow3.ogt` (Blow 3), `vfx/effekseer-tktk01/Blow4.ogt` (Blow 4), `vfx/effekseer-tktk01/Blow5.ogt` (Blow 5), `vfx/effekseer-tktk01/Blow6.ogt` (Blow 6), `vfx/effekseer-tktk01/Blow7.ogt` (Blow 7), `vfx/effekseer-tktk01/Blow8.ogt` (Blow 8), `vfx/effekseer-tktk01/Blow9.ogt` (Blow 9)
- effekseer-tktk02: `vfx/effekseer-tktk02/Blow1.ogt` (Blow 1), `vfx/effekseer-tktk02/Blow2.ogt` (Blow 2), `vfx/effekseer-tktk02/Blow3.ogt` (Blow 3), `vfx/effekseer-tktk02/Blow4.ogt` (Blow 4), `vfx/effekseer-tktk02/Blow5.ogt` (Blow 5), `vfx/effekseer-tktk02/Blow6.ogt` (Blow 6)
- pixel-scenario: `vfx/pixel-scenario/electric-impact-hit.ogt` (Static Impact)

## 21. Level up effects

Level up effects (level-up): 3 in `effekseer-starter`, `pixel-scenario`.

- effekseer-starter: `vfx/effekseer-starter/Light4.ogt` (Level Up Arrows), `vfx/effekseer-starter/Thunder1.ogt` (Golden Rise)
- pixel-scenario: `vfx/pixel-scenario/level-up.ogt` (Gilded Level Up)

## 22. Magic impact effects

Magic impact effects (magic-impact): 109 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/smallTexturesRibbon.ogt` (Small Textures Ribbon), `vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital.ogt` (Small Textures Square Wave Orbital)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/A_Gohlem1.ogt` (A Gohlem 1), `vfx/effekseer-magicalxspiral/A_Salamander1.ogt` (A Salamander 1), `vfx/effekseer-magicalxspiral/A_Salamander2.ogt` (A Salamander 2), `vfx/effekseer-magicalxspiral/A_Salamander4.ogt` (A Salamander 4), `vfx/effekseer-magicalxspiral/A_Sylph1.ogt` (A Sylph 1), `vfx/effekseer-magicalxspiral/A_Sylph2.ogt` (A Sylph 2), `vfx/effekseer-magicalxspiral/A_Undine1.ogt` (A Undine 1), `vfx/effekseer-magicalxspiral/A_Undine2.ogt` (A Undine 2), `vfx/effekseer-magicalxspiral/AquaPoint.ogt` (Aqua Point), `vfx/effekseer-magicalxspiral/Attack_Impact.ogt` (Attack Impact), `vfx/effekseer-magicalxspiral/Gohlem1.ogt` (Gohlem 1), `vfx/effekseer-magicalxspiral/Gohlem11.ogt` (Gohlem 11), `vfx/effekseer-magicalxspiral/Gohlem5.ogt` (Gohlem 5), `vfx/effekseer-magicalxspiral/Gohlem7.ogt` (Gohlem 7), `vfx/effekseer-magicalxspiral/Holy1.ogt` (Holy 1), `vfx/effekseer-magicalxspiral/Holy2.ogt` (Holy 2), `vfx/effekseer-magicalxspiral/Impact.ogt` (Impact), `vfx/effekseer-magicalxspiral/Magic1_earth.ogt` (Magic 1 Earth), `vfx/effekseer-magicalxspiral/Magic1_holy.ogt` (Magic 1 Holy), `vfx/effekseer-magicalxspiral/Magic1_water.ogt` (Magic 1 Water), `vfx/effekseer-magicalxspiral/Magic1_wind.ogt` (Magic 1 Wind), `vfx/effekseer-magicalxspiral/Magic2_earth.ogt` (Magic 2 Earth), `vfx/effekseer-magicalxspiral/Magic2_holy.ogt` (Magic 2 Holy), `vfx/effekseer-magicalxspiral/Magic2_water.ogt` (Magic 2 Water), `vfx/effekseer-magicalxspiral/Magic2_wind.ogt` (Magic 2 Wind), `vfx/effekseer-magicalxspiral/Rock1.ogt` (Rock 1), `vfx/effekseer-magicalxspiral/Rock2.ogt` (Rock 2), `vfx/effekseer-magicalxspiral/Salamander1.ogt` (Salamander 1), `vfx/effekseer-magicalxspiral/Salamander11.ogt` (Salamander 11), `vfx/effekseer-magicalxspiral/Salamander12.ogt` (Salamander 12), `vfx/effekseer-magicalxspiral/Salamander14.ogt` (Salamander 14), `vfx/effekseer-magicalxspiral/Salamander15.ogt` (Salamander 15), `vfx/effekseer-magicalxspiral/Salamander2.ogt` (Salamander 2), `vfx/effekseer-magicalxspiral/Salamander3.ogt` (Salamander 3), `vfx/effekseer-magicalxspiral/Salamander4.ogt` (Salamander 4), `vfx/effekseer-magicalxspiral/Salamander5.ogt` (Salamander 5), `vfx/effekseer-magicalxspiral/Salamander6.ogt` (Salamander 6), `vfx/effekseer-magicalxspiral/Salamander7.ogt` (Salamander 7), `vfx/effekseer-magicalxspiral/Sylph1.ogt` (Sylph 1), `vfx/effekseer-magicalxspiral/Sylph10.ogt` (Sylph 10), `vfx/effekseer-magicalxspiral/Sylph11.ogt` (Sylph 11), `vfx/effekseer-magicalxspiral/Sylph13.ogt` (Sylph 13), `vfx/effekseer-magicalxspiral/Sylph15.ogt` (Sylph 15), `vfx/effekseer-magicalxspiral/Sylph2.ogt` (Sylph 2), `vfx/effekseer-magicalxspiral/Sylph3.ogt` (Sylph 3), `vfx/effekseer-magicalxspiral/Sylph4.ogt` (Sylph 4), `vfx/effekseer-magicalxspiral/Sylph5.ogt` (Sylph 5), `vfx/effekseer-magicalxspiral/Sylph6.ogt` (Sylph 6), `vfx/effekseer-magicalxspiral/Sylph7.ogt` (Sylph 7), `vfx/effekseer-magicalxspiral/Sylph8.ogt` (Sylph 8), `vfx/effekseer-magicalxspiral/Thunder1.ogt` (Thunder 1), `vfx/effekseer-magicalxspiral/Thunder2.ogt` (Thunder 2), `vfx/effekseer-magicalxspiral/Thunder3.ogt` (Thunder 3), `vfx/effekseer-magicalxspiral/Topwater1.ogt` (Topwater 1), `vfx/effekseer-magicalxspiral/Undine1.ogt` (Undine 1), `vfx/effekseer-magicalxspiral/Undine11.ogt` (Undine 11), `vfx/effekseer-magicalxspiral/Undine13.ogt` (Undine 13), `vfx/effekseer-magicalxspiral/Undine2.ogt` (Undine 2), `vfx/effekseer-magicalxspiral/Undine3.ogt` (Undine 3), `vfx/effekseer-magicalxspiral/Undine4.ogt` (Undine 4), `vfx/effekseer-magicalxspiral/Undine5.ogt` (Undine 5), `vfx/effekseer-magicalxspiral/Undine6.ogt` (Undine 6), `vfx/effekseer-magicalxspiral/Undine8.ogt` (Undine 8), `vfx/effekseer-magicalxspiral/WHead1.ogt` (WHead 1), `vfx/effekseer-magicalxspiral/WHead2.ogt` (WHead 2), `vfx/effekseer-magicalxspiral/WHead3.ogt` (WHead 3), `vfx/effekseer-magicalxspiral/Water_Impact.ogt` (Water Impact), `vfx/effekseer-magicalxspiral/Wind_Impact.ogt` (Wind Impact), `vfx/effekseer-magicalxspiral/salamander13.ogt` (Salamander 13)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicCold.ogt` (Magic Cold), `vfx/effekseer-nextsoft01/MagicDark.ogt` (Magic Dark), `vfx/effekseer-nextsoft01/MagicLight.ogt` (Magic Light), `vfx/effekseer-nextsoft01/MagicThunder.ogt` (Magic Thunder), `vfx/effekseer-nextsoft01/MagicWater.ogt` (Magic Water)
- effekseer-pierre01: `vfx/effekseer-pierre01/LightningStrike.ogt` (Lightning Strike), `vfx/effekseer-pierre01/MonsterRoar.ogt` (Monster Roar)
- effekseer-pierre02: `vfx/effekseer-pierre02/LossOfBlood.ogt` (Loss Of Blood)
- effekseer-starter: `vfx/effekseer-starter/MagicCold.ogt` (Frost), `vfx/effekseer-starter/MagicDark.ogt` (Dark Curse), `vfx/effekseer-starter/MagicWater.ogt` (Bubble Burst), `vfx/effekseer-starter/Undine4.ogt` (Water Burst)
- effekseer-tktk01: `vfx/effekseer-tktk01/Dark1.ogt` (Dark 1), `vfx/effekseer-tktk01/Dark2.ogt` (Dark 2), `vfx/effekseer-tktk01/Dark3.ogt` (Dark 3), `vfx/effekseer-tktk01/Light1.ogt` (Light 1), `vfx/effekseer-tktk01/Light2.ogt` (Light 2), `vfx/effekseer-tktk01/Light3.ogt` (Light 3), `vfx/effekseer-tktk01/Light4.ogt` (Light 4), `vfx/effekseer-tktk01/Magic1.ogt` (Magic 1), `vfx/effekseer-tktk01/Magic2.ogt` (Magic 2), `vfx/effekseer-tktk01/Soil1.ogt` (Soil 1), `vfx/effekseer-tktk01/Wind1.ogt` (Wind 1), `vfx/effekseer-tktk01/Wind2.ogt` (Wind 2)
- effekseer-tktk02: `vfx/effekseer-tktk02/Dark1.ogt` (Dark 1), `vfx/effekseer-tktk02/Dark2.ogt` (Dark 2), `vfx/effekseer-tktk02/Light1.ogt` (Light 1), `vfx/effekseer-tktk02/Light2.ogt` (Light 2), `vfx/effekseer-tktk02/Light3.ogt` (Light 3), `vfx/effekseer-tktk02/Light4.ogt` (Light 4)
- pixel-scenario: `vfx/pixel-scenario/arcane-magic-circle.ogt` (Eldritch Magic Circle), `vfx/pixel-scenario/frost-nova.ogt` (Frost Nova), `vfx/pixel-scenario/ground-slam-quake.ogt` (Stone Ground Slam), `vfx/pixel-scenario/holy-pillar-of-light.ogt` (Divine Pillar), `vfx/pixel-scenario/ice-spikes-erupting.ogt` (Frozen Spikes), `vfx/pixel-scenario/lava-eruption-pillar.ogt` (Volcanic Pillar), `vfx/pixel-scenario/lightning-strike-from-the-sky.ogt` (Storm Strike), `vfx/pixel-scenario/water-splash.ogt` (Aqua Splash)

## 23. Magic projectile effects

Magic projectile effects (magic-projectile): 36 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/drill.ogt` (Drill)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Icicle1.ogt` (Icicle 1), `vfx/effekseer-magicalxspiral/Icicle2.ogt` (Icicle 2), `vfx/effekseer-magicalxspiral/Icicle3.ogt` (Icicle 3), `vfx/effekseer-magicalxspiral/IcicleLocus.ogt` (Icicle Locus), `vfx/effekseer-magicalxspiral/Lance3.ogt` (Lance 3), `vfx/effekseer-magicalxspiral/Lance4.ogt` (Lance 4), `vfx/effekseer-magicalxspiral/Lance5.ogt` (Lance 5), `vfx/effekseer-magicalxspiral/meteo1.ogt` (Meteo 1)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicMeteo.ogt` (Magic Meteo), `vfx/effekseer-nextsoft01/Spear1.ogt` (Spear 1), `vfx/effekseer-nextsoft01/Spear2.ogt` (Spear 2)
- effekseer-pierre01: `vfx/effekseer-pierre01/MissileBoost.ogt` (Missile Boost), `vfx/effekseer-pierre01/PhantasmMeteor.ogt` (Phantasm Meteor), `vfx/effekseer-pierre01/PhantasmMeteor_Single.ogt` (Phantasm Meteor Single), `vfx/effekseer-pierre01/SonicBoom.ogt` (Sonic Boom)
- effekseer-pierre02: `vfx/effekseer-pierre02/BloodLance.ogt` (Blood Lance), `vfx/effekseer-pierre02/FireBall.ogt` (Fire Ball)
- effekseer-starter: `vfx/effekseer-starter/FireBall.ogt` (Fireball), `vfx/effekseer-starter/Icicle2.ogt` (Icicles), `vfx/effekseer-starter/Laser03.ogt` (Green Laser), `vfx/effekseer-starter/Spear2.ogt` (Light Spear)
- effekseer-tktk01: `vfx/effekseer-tktk01/Arrow1.ogt` (Arrow 1), `vfx/effekseer-tktk01/Arrow2.ogt` (Arrow 2)
- effekseer-tktk02: `vfx/effekseer-tktk02/Gun1.ogt` (Gun 1), `vfx/effekseer-tktk02/Gun2.ogt` (Gun 2), `vfx/effekseer-tktk02/Gun3.ogt` (Gun 3), `vfx/effekseer-tktk02/Gun4.ogt` (Gun 4), `vfx/effekseer-tktk02/Gun5.ogt` (Gun 5), `vfx/effekseer-tktk02/Gun6.ogt` (Gun 6), `vfx/effekseer-tktk02/Gun7.ogt` (Gun 7), `vfx/effekseer-tktk02/Gun8.ogt` (Gun 8), `vfx/effekseer-tktk02/Gun9.ogt` (Gun 9)
- pixel-scenario: `vfx/pixel-scenario/blue-energy-beam.ogt` (Aether Beam), `vfx/pixel-scenario/fireball-projectile.ogt` (Ember Projectile), `vfx/pixel-scenario/plasma-orb.ogt` (Ion Orb)

## 24. Portal effects

Portal effects (portal): 9 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/magic_circle.ogt` (Magic Circle)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/MagicArea.ogt` (Magic Area)
- effekseer-pierre02: `vfx/effekseer-pierre02/GateOfCalve.ogt` (Gate Of Calve)
- effekseer-starter: `vfx/effekseer-starter/Magic2_fire.ogt` (Fire Pentagram), `vfx/effekseer-starter/magic_circle.ogt` (Magic Circle), `vfx/effekseer-starter/smallTexturesSquareWave_orbital.ogt` (Arcane Ring)
- pixel-scenario: `vfx/pixel-scenario/dimensional-portal.ogt` (Astral Portal), `vfx/pixel-scenario/purple-black-hole-vortex.ogt` (Shadow Vortex), `vfx/pixel-scenario/teleport-blink.ogt` (Mystic Teleport)

## 25. Shield effects

Shield effects (shield): 3 in `effekseer-magicalxspiral`, `effekseer-nextsoft01`.

- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Gohlem_Shield1.ogt` (Gohlem Shield 1), `vfx/effekseer-magicalxspiral/Gohlem_Shield2.ogt` (Gohlem Shield 2)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicShield.ogt` (Magic Shield)

## 26. Slash effects

Slash effects (slash): 48 in `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Attack1.ogt` (Attack 1), `vfx/effekseer-magicalxspiral/Attack2.ogt` (Attack 2), `vfx/effekseer-magicalxspiral/Attack5.ogt` (Attack 5), `vfx/effekseer-magicalxspiral/Attack6.ogt` (Attack 6)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicSword.ogt` (Magic Sword), `vfx/effekseer-nextsoft01/Sword1.ogt` (Sword 1), `vfx/effekseer-nextsoft01/Sword2.ogt` (Sword 2)
- effekseer-pierre02: `vfx/effekseer-pierre02/Sword_Ember.ogt` (Sword Ember), `vfx/effekseer-pierre02/Sword_Lightning.ogt` (Sword Lightning), `vfx/effekseer-pierre02/Sword_Moonlight.ogt` (Sword Moonlight), `vfx/effekseer-pierre02/Sword_Sacredlight.ogt` (Sword Sacredlight), `vfx/effekseer-pierre02/Sword_Storm.ogt` (Sword Storm)
- effekseer-starter: `vfx/effekseer-starter/Claw3.ogt` (Claw), `vfx/effekseer-starter/Sword5.ogt` (Blue Slash), `vfx/effekseer-starter/Sword7.ogt` (Cyan Cross Slash), `vfx/effekseer-starter/Sword8.ogt` (Red Arc Slash)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/sword.ogt` (Sword)
- effekseer-tktk01: `vfx/effekseer-tktk01/Sword1.ogt` (Sword 1), `vfx/effekseer-tktk01/Sword10.ogt` (Sword 10), `vfx/effekseer-tktk01/Sword11.ogt` (Sword 11), `vfx/effekseer-tktk01/Sword12.ogt` (Sword 12), `vfx/effekseer-tktk01/Sword13.ogt` (Sword 13), `vfx/effekseer-tktk01/Sword14.ogt` (Sword 14), `vfx/effekseer-tktk01/Sword15.ogt` (Sword 15), `vfx/effekseer-tktk01/Sword16.ogt` (Sword 16), `vfx/effekseer-tktk01/Sword17.ogt` (Sword 17), `vfx/effekseer-tktk01/Sword18.ogt` (Sword 18), `vfx/effekseer-tktk01/Sword19.ogt` (Sword 19), `vfx/effekseer-tktk01/Sword2.ogt` (Sword 2), `vfx/effekseer-tktk01/Sword20.ogt` (Sword 20), `vfx/effekseer-tktk01/Sword21.ogt` (Sword 21), `vfx/effekseer-tktk01/Sword22.ogt` (Sword 22), `vfx/effekseer-tktk01/Sword3.ogt` (Sword 3), `vfx/effekseer-tktk01/Sword4.ogt` (Sword 4), `vfx/effekseer-tktk01/Sword5.ogt` (Sword 5), `vfx/effekseer-tktk01/Sword6.ogt` (Sword 6), `vfx/effekseer-tktk01/Sword7.ogt` (Sword 7), `vfx/effekseer-tktk01/Sword8.ogt` (Sword 8), `vfx/effekseer-tktk01/Sword9.ogt` (Sword 9)
- effekseer-tktk02: `vfx/effekseer-tktk02/Claw1.ogt` (Claw 1), `vfx/effekseer-tktk02/Claw2.ogt` (Claw 2), `vfx/effekseer-tktk02/Claw3.ogt` (Claw 3), `vfx/effekseer-tktk02/Claw4.ogt` (Claw 4)
- pixel-scenario: `vfx/pixel-scenario/crescent-sword-slash.ogt` (Razor Slash), `vfx/pixel-scenario/dash-speed-lines.ogt` (Cyclone Dash), `vfx/pixel-scenario/feral-claw-scratch-blood.ogt` (Sanguine Claw), `vfx/pixel-scenario/spinning-wind-slash.ogt` (Cyclone Slash), `vfx/pixel-scenario/triple-slash-combo-with-blue-energy.ogt` (Flux Slash)

## 27. Smoke effects

Smoke effects (smoke): 12 in `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/electric_dust.ogt` (Electric Dust)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/foot_smoke1.ogt` (Foot Smoke 1), `vfx/effekseer-magicalxspiral/foot_smoke2.ogt` (Foot Smoke 2), `vfx/effekseer-magicalxspiral/foot_smoke3.ogt` (Foot Smoke 3), `vfx/effekseer-magicalxspiral/foot_smoke4.ogt` (Foot Smoke 4), `vfx/effekseer-magicalxspiral/foot_smoke5.ogt` (Foot Smoke 5), `vfx/effekseer-magicalxspiral/foot_smoke6.ogt` (Foot Smoke 6), `vfx/effekseer-magicalxspiral/foot_smoke7.ogt` (Foot Smoke 7), `vfx/effekseer-magicalxspiral/foot_smoke8.ogt` (Foot Smoke 8)
- effekseer-pierre02: `vfx/effekseer-pierre02/CosmicMist.ogt` (Cosmic Mist)
- pixel-scenario: `vfx/pixel-scenario/poison-gas-cloud.ogt` (Blight Cloud), `vfx/pixel-scenario/smoke-poof.ogt` (Ashen Poof)

## 28. Ui effects

Ui effects (ui): 10 in `effekseer-andrewfm01`, `pixel-scenario`.

- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/actionLines_halo.ogt` (Action Lines Halo), `vfx/effekseer-andrewfm01/actionLines_halo2.ogt` (Action Lines Halo 2), `vfx/effekseer-andrewfm01/actionLines_radial.ogt` (Action Lines Radial), `vfx/effekseer-andrewfm01/actionLines_radial2.ogt` (Action Lines Radial 2), `vfx/effekseer-andrewfm01/actionLines_shockwave.ogt` (Action Lines Shockwave), `vfx/effekseer-andrewfm01/actionLines_strip.ogt` (Action Lines Strip)
- pixel-scenario: `vfx/pixel-scenario/critical-hit-damage-999.ogt` (Ember Damage Number), `vfx/pixel-scenario/fireworks-celebration.ogt` (Arcane Fireworks), `vfx/pixel-scenario/gold-coins-loot-drop.ogt` (Lucky Loot), `vfx/pixel-scenario/sparkle-twinkle.ogt` (Sacred Sparkle)

## 29. Weather effects

Weather effects (weather): 7 in `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `pixel-scenario`.

- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Snowman1.ogt` (Snowman 1)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicTornade.ogt` (Magic Tornade)
- effekseer-pierre01: `vfx/effekseer-pierre01/HolySandstorm.ogt` (Holy Sandstorm)
- effekseer-starter: `vfx/effekseer-starter/HolySandstorm.ogt` (Sandstorm)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/snowstorm11.ogt` (Snowstorm 11)
- pixel-scenario: `vfx/pixel-scenario/rain.ogt` (Aqua Ambient), `vfx/pixel-scenario/snow-falling.ogt` (Glacial Ambient)
