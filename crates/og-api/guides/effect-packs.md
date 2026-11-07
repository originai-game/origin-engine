<!-- Generated from the vfx packs' extension.json and assets.json (packages/engine/extensions/, packages/vfx-packs/) by `og api gen`. Do not edit: `og api check` fails on any difference. -->
# The effect packs

Ready-made effects a game installs with `og vfx add <pack>`: converted Effekseer samples (`type vfx`, 3D) and pixel-art recipes (`type vfx2d`, 2D), each listed with its tags, keywords and effects. A bundled pack ships with the engine; a downloadable one comes with each release and `og vfx add` fetches it. Once installed, an effect spawns as `vfx/<pack>/<effect>` (`og api show vfx`; the `pixel-vfx` guide for `type vfx2d`); the pack format is docs/vfx-packs.md.

## 1. effekseer-starter

A starter set of Effekseer sample effects from nine contributors (fire, explosions, hits, slashes, heals, buffs, level-ups, projectiles, impacts, portals, weather, ambient), converted to type vfx. 38 effects, bundled with the engine: `og vfx add effekseer-starter`.

- Style: stylised, 3d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, weather
- Elements: dark, earth, fire, holy, ice, lightning, nature, water
- Keywords: 3d, anime, combat, effekseer, magic, particles, starter

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-starter/Attack_Impact.ogt` | Attack Impact | hit-spark |  | 0.567 |
| `vfx/effekseer-starter/Blow2.ogt` | Spark Hit | hit-spark |  | 0.533 |
| `vfx/effekseer-starter/Blow4.ogt` | Bright Explosion | explosion |  | 0.845 |
| `vfx/effekseer-starter/Blow8.ogt` | Star Hit | hit-spark |  | 3.017 |
| `vfx/effekseer-starter/Breakdown.ogt` | Breakdown Burst | explosion |  | 0.517 |
| `vfx/effekseer-starter/Claw3.ogt` | Claw | slash |  | 2.733 |
| `vfx/effekseer-starter/Cure1.ogt` | Star Cure | heal | holy | 1.275 |
| `vfx/effekseer-starter/Cure7.ogt` | Light Cure | heal | holy | 2.6 |
| `vfx/effekseer-starter/FeatherBomb.ogt` | Smoke Bomb | explosion |  | 13.006 |
| `vfx/effekseer-starter/Fire7.ogt` | Fire Pillar | fire | fire | 2.998 |
| `vfx/effekseer-starter/FireBall.ogt` | Fireball | magic-projectile | fire | 5.517 |
| `vfx/effekseer-starter/HealPotion2.ogt` | Potion | heal |  | 1.63 |
| `vfx/effekseer-starter/HolySandstorm.ogt` | Sandstorm | weather | earth | 6.733 |
| `vfx/effekseer-starter/Icicle2.ogt` | Icicles | magic-projectile | ice | 0.167 |
| `vfx/effekseer-starter/Laser03.ogt` | Green Laser | magic-projectile |  | loops |
| `vfx/effekseer-starter/Light4.ogt` | Level Up Arrows | level-up | holy | 2.092 |
| `vfx/effekseer-starter/Magic2_fire.ogt` | Fire Pentagram | portal | fire | 0.483 |
| `vfx/effekseer-starter/MagicCold.ogt` | Frost | magic-impact | ice | 2.15 |
| `vfx/effekseer-starter/MagicDark.ogt` | Dark Curse | magic-impact | dark | 2.05 |
| `vfx/effekseer-starter/MagicFire2.ogt` | Fire Breath | fire | fire | 1.91 |
| `vfx/effekseer-starter/MagicHeal2.ogt` | Healing Circle | heal | holy | 4 |
| `vfx/effekseer-starter/MagicWater.ogt` | Bubble Burst | magic-impact | water | 2.09 |
| `vfx/effekseer-starter/PowerUp.ogt` | Power Up | buff |  | 1.75 |
| `vfx/effekseer-starter/Spear2.ogt` | Light Spear | magic-projectile |  | 0.983 |
| `vfx/effekseer-starter/Sword5.ogt` | Blue Slash | slash |  | 0.6 |
| `vfx/effekseer-starter/Sword7.ogt` | Cyan Cross Slash | slash |  | 0.583 |
| `vfx/effekseer-starter/Sword8.ogt` | Red Arc Slash | slash |  | 1.417 |
| `vfx/effekseer-starter/Thunder1.ogt` | Golden Rise | level-up | lightning | 2.15 |
| `vfx/effekseer-starter/Undine4.ogt` | Water Burst | magic-impact | water | 1.667 |
| `vfx/effekseer-starter/boss_death.ogt` | Boss Death | explosion |  | loops |
| `vfx/effekseer-starter/electric_dust.ogt` | Floating Motes | ambient | lightning | 3.4 |
| `vfx/effekseer-starter/hit_eff.ogt` | Hit Flash | hit-spark |  | 0.622 |
| `vfx/effekseer-starter/hozyo2.ogt` | Power Rings | buff |  | 1.217 |
| `vfx/effekseer-starter/hozyo4.ogt` | Attack Up | buff | fire | loops |
| `vfx/effekseer-starter/magic_circle.ogt` | Magic Circle | portal |  | 16.683 |
| `vfx/effekseer-starter/meteo1.ogt` | Meteor Strike | explosion | fire | 0.858 |
| `vfx/effekseer-starter/sakura1.ogt` | Sakura Petals | ambient | nature | 13.8 |
| `vfx/effekseer-starter/smallTexturesSquareWave_orbital.ogt` | Arcane Ring | portal |  | 66.667 |

## 2. pixel-scenario

The Pixel VFX Forge library: 42 pixel-art effects (combat, magic, elements, feedback, ambient) as type vfx2d recipes. 42 effects, bundled with the engine: `og vfx add pixel-scenario`.

- Style: pixel, 2d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, smoke, ui, weather
- Elements: arcane, dark, earth, fire, holy, ice, lightning, nature, water, wind
- Keywords: 2d, flipbook, forge, pixel, pixel art, scenario, sprite

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/pixel-scenario/arcane-magic-circle.ogt` | Eldritch Magic Circle | magic-impact | arcane | loops |
| `vfx/pixel-scenario/big-fire-explosion.ogt` | Cinder Explosion | explosion | fire | 1.2 |
| `vfx/pixel-scenario/blue-energy-beam.ogt` | Aether Beam | magic-projectile | arcane | loops |
| `vfx/pixel-scenario/campfire.ogt` | Ember Flame | fire | fire | loops |
| `vfx/pixel-scenario/charge-up-energy.ogt` | Pulse Charge | buff | arcane | loops |
| `vfx/pixel-scenario/cherry-blossom-petals.ogt` | Thorn Ambient | ambient | nature | loops |
| `vfx/pixel-scenario/crescent-sword-slash.ogt` | Razor Slash | slash |  | 0.667 |
| `vfx/pixel-scenario/critical-hit-damage-999.ogt` | Ember Damage Number | ui |  | 1.067 |
| `vfx/pixel-scenario/dark-shadow-aura.ogt` | Night Aura | buff | dark | loops |
| `vfx/pixel-scenario/dash-speed-lines.ogt` | Cyclone Dash | slash | wind | 0.556 |
| `vfx/pixel-scenario/dimensional-portal.ogt` | Astral Portal | portal | arcane | loops |
| `vfx/pixel-scenario/dragon-fire-breath.ogt` | Cinder Breath | fire | fire | loops |
| `vfx/pixel-scenario/electric-impact-hit.ogt` | Static Impact | hit-spark | lightning | 0.556 |
| `vfx/pixel-scenario/embers-rising.ogt` | Blaze Ambient | ambient | fire | loops |
| `vfx/pixel-scenario/falling-leaves.ogt` | Bloom Ambient | ambient | nature | loops |
| `vfx/pixel-scenario/feral-claw-scratch-blood.ogt` | Sanguine Claw | slash |  | 0.667 |
| `vfx/pixel-scenario/fire-tornado.ogt` | Blaze Tornado | fire | fire | loops |
| `vfx/pixel-scenario/fireball-projectile.ogt` | Ember Projectile | magic-projectile | fire | loops |
| `vfx/pixel-scenario/fireflies.ogt` | Mystic Ambient | ambient | fire | loops |
| `vfx/pixel-scenario/fireworks-celebration.ogt` | Arcane Fireworks | ui | fire | 2.133 |
| `vfx/pixel-scenario/frost-nova.ogt` | Frost Nova | magic-impact | ice | 1.067 |
| `vfx/pixel-scenario/gold-coins-loot-drop.ogt` | Lucky Loot | ui |  | 1.6 |
| `vfx/pixel-scenario/ground-slam-quake.ogt` | Stone Ground Slam | magic-impact | earth | 1.2 |
| `vfx/pixel-scenario/healing-with-rising-crosses.ogt` | Bloom Heal | heal | holy | 1.333 |
| `vfx/pixel-scenario/hex-shield-barrier.ogt` | Flux Shield | buff | arcane | loops |
| `vfx/pixel-scenario/holy-pillar-of-light.ogt` | Divine Pillar | magic-impact | holy | 1.333 |
| `vfx/pixel-scenario/ice-spikes-erupting.ogt` | Frozen Spikes | magic-impact | ice | 1.2 |
| `vfx/pixel-scenario/lava-eruption-pillar.ogt` | Volcanic Pillar | magic-impact | fire | 1.333 |
| `vfx/pixel-scenario/level-up.ogt` | Gilded Level Up | level-up |  | 1.6 |
| `vfx/pixel-scenario/lightning-strike-from-the-sky.ogt` | Storm Strike | magic-impact | lightning | 0.889 |
| `vfx/pixel-scenario/meteor-strike.ogt` | Ember Meteor | explosion | fire | 1.733 |
| `vfx/pixel-scenario/plasma-orb.ogt` | Ion Orb | magic-projectile | lightning | loops |
| `vfx/pixel-scenario/poison-gas-cloud.ogt` | Blight Cloud | smoke | nature | loops |
| `vfx/pixel-scenario/purple-black-hole-vortex.ogt` | Shadow Vortex | portal | dark | loops |
| `vfx/pixel-scenario/rain.ogt` | Aqua Ambient | weather | water | loops |
| `vfx/pixel-scenario/smoke-poof.ogt` | Ashen Poof | smoke |  | 0.933 |
| `vfx/pixel-scenario/snow-falling.ogt` | Glacial Ambient | weather | ice | loops |
| `vfx/pixel-scenario/sparkle-twinkle.ogt` | Sacred Sparkle | ui |  | loops |
| `vfx/pixel-scenario/spinning-wind-slash.ogt` | Cyclone Slash | slash | wind | 0.667 |
| `vfx/pixel-scenario/teleport-blink.ogt` | Mystic Teleport | portal | arcane | 1.333 |
| `vfx/pixel-scenario/triple-slash-combo-with-blue-energy.ogt` | Flux Slash | slash | arcane | 1.167 |
| `vfx/pixel-scenario/water-splash.ogt` | Aqua Splash | magic-impact | water | 1.067 |

## 3. effekseer-andrewfm01

AndrewFM01's Effekseer sample effects (15), converted to type vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-andrewfm01`.

- Style: stylised, 3d
- Categories: beam, explosion, fire, hit-spark, magic-impact, magic-projectile, portal, smoke, ui
- Elements: fire, lightning
- Keywords: 3d, andrewfm01, anime, effekseer, particles

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-andrewfm01/actionLines_halo.ogt` | Action Lines Halo | ui |  | 1.673 |
| `vfx/effekseer-andrewfm01/actionLines_halo2.ogt` | Action Lines Halo 2 | ui |  | 1.683 |
| `vfx/effekseer-andrewfm01/actionLines_radial.ogt` | Action Lines Radial | ui |  | 1.673 |
| `vfx/effekseer-andrewfm01/actionLines_radial2.ogt` | Action Lines Radial 2 | ui |  | 1.7 |
| `vfx/effekseer-andrewfm01/actionLines_shockwave.ogt` | Action Lines Shockwave | ui |  | 1.667 |
| `vfx/effekseer-andrewfm01/actionLines_strip.ogt` | Action Lines Strip | ui |  | 1.667 |
| `vfx/effekseer-andrewfm01/blue_laser.ogt` | Blue Laser | beam |  | loops |
| `vfx/effekseer-andrewfm01/boss_death.ogt` | Boss Death | explosion |  | loops |
| `vfx/effekseer-andrewfm01/drill.ogt` | Drill | magic-projectile |  | loops |
| `vfx/effekseer-andrewfm01/electric_dust.ogt` | Electric Dust | smoke | lightning | 3.4 |
| `vfx/effekseer-andrewfm01/fire.ogt` | Fire | fire | fire | 1 |
| `vfx/effekseer-andrewfm01/hit_eff.ogt` | Hit Eff | hit-spark |  | 0.622 |
| `vfx/effekseer-andrewfm01/magic_circle.ogt` | Magic Circle | portal |  | 16.683 |
| `vfx/effekseer-andrewfm01/smallTexturesRibbon.ogt` | Small Textures Ribbon | magic-impact |  | 1.017 |
| `vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital.ogt` | Small Textures Square Wave Orbital | magic-impact |  | 66.667 |

## 4. effekseer-effekseer01

Effekseer01's Effekseer sample effects (15), converted to type vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-effekseer01`.

- Style: stylised, 3d
- Categories: basic, beam
- Elements: 
- Keywords: 3d, anime, effekseer, effekseer01, particles

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-effekseer01/Homing_Laser01.ogt` | Homing Laser 01 | beam |  | 4.267 |
| `vfx/effekseer-effekseer01/Laser01.ogt` | Laser 01 | beam |  | 1.333 |
| `vfx/effekseer-effekseer01/Laser02.ogt` | Laser 02 | beam |  | 3.733 |
| `vfx/effekseer-effekseer01/Laser03.ogt` | Laser 03 | beam |  | loops |
| `vfx/effekseer-effekseer01/Simple_Distortion.ogt` | Simple Distortion | basic |  | 2.917 |
| `vfx/effekseer-effekseer01/Simple_GeneratingPosition1.ogt` | Simple Generating Position 1 | basic |  | 0.168 |
| `vfx/effekseer-effekseer01/Simple_Model_UV.ogt` | Simple Model UV | basic |  | 3.5 |
| `vfx/effekseer-effekseer01/Simple_Ribbon_Parent.ogt` | Simple Ribbon Parent | basic |  | 1.817 |
| `vfx/effekseer-effekseer01/Simple_Ribbon_Sword.ogt` | Simple Ribbon Sword | basic |  | 0.4 |
| `vfx/effekseer-effekseer01/Simple_Ring_Shape1.ogt` | Simple Ring Shape 1 | basic |  | 9.5 |
| `vfx/effekseer-effekseer01/Simple_Ring_Shape2.ogt` | Simple Ring Shape 2 | basic |  | 7 |
| `vfx/effekseer-effekseer01/Simple_Sprite_BillBoard.ogt` | Simple Sprite Bill Board | basic |  | 0.665 |
| `vfx/effekseer-effekseer01/Simple_Sprite_Fixed.ogt` | Simple Sprite Fixed | basic |  | 1.633 |
| `vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis.ogt` | Simple Sprite Fixed YAxis | basic |  | 2.817 |
| `vfx/effekseer-effekseer01/Simple_Track1.ogt` | Simple Track 1 | basic |  | 3.333 |

## 5. effekseer-hato01

HATO01's Effekseer sample effects (4), converted to type vfx. 4 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-hato01`.

- Style: stylised, 3d
- Categories: ambient
- Elements: nature
- Keywords: 3d, anime, effekseer, hato01, particles

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-hato01/aoi-chou.ogt` | Aoi Chou | ambient | nature | loops |
| `vfx/effekseer-hato01/iwai-ume.ogt` | Iwai Ume | ambient | nature | 20.3 |
| `vfx/effekseer-hato01/kiroi-chou.ogt` | Kiroi Chou | ambient | nature | 33.375 |
| `vfx/effekseer-hato01/sakura1.ogt` | Sakura 1 | ambient | nature | 13.8 |

## 6. effekseer-magicalxspiral

MAGICAL X SPIRAL's Effekseer sample effects (101), converted to type vfx. 101 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-magicalxspiral`.

- Style: stylised, 3d
- Categories: explosion, fire, magic-impact, magic-projectile, portal, shield, slash, smoke, weather
- Elements: earth, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, magicalxspiral, particles

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-magicalxspiral/A_Gohlem1.ogt` | A Gohlem 1 | magic-impact | earth | 0.983 |
| `vfx/effekseer-magicalxspiral/A_Salamander1.ogt` | A Salamander 1 | magic-impact | fire | 0.983 |
| `vfx/effekseer-magicalxspiral/A_Salamander2.ogt` | A Salamander 2 | magic-impact | fire | 0.767 |
| `vfx/effekseer-magicalxspiral/A_Salamander4.ogt` | A Salamander 4 | magic-impact | fire | 1.167 |
| `vfx/effekseer-magicalxspiral/A_Sylph1.ogt` | A Sylph 1 | magic-impact | wind | 0.983 |
| `vfx/effekseer-magicalxspiral/A_Sylph2.ogt` | A Sylph 2 | magic-impact | wind | 0.575 |
| `vfx/effekseer-magicalxspiral/A_Undine1.ogt` | A Undine 1 | magic-impact | water | 0.983 |
| `vfx/effekseer-magicalxspiral/A_Undine2.ogt` | A Undine 2 | magic-impact | water | 0.567 |
| `vfx/effekseer-magicalxspiral/AquaPoint.ogt` | Aqua Point | magic-impact | water | 2.317 |
| `vfx/effekseer-magicalxspiral/Attack1.ogt` | Attack 1 | slash |  | 0.983 |
| `vfx/effekseer-magicalxspiral/Attack2.ogt` | Attack 2 | slash |  | 0.983 |
| `vfx/effekseer-magicalxspiral/Attack5.ogt` | Attack 5 | slash |  | 0.9 |
| `vfx/effekseer-magicalxspiral/Attack6.ogt` | Attack 6 | slash |  | 0.9 |
| `vfx/effekseer-magicalxspiral/Attack_Impact.ogt` | Attack Impact | magic-impact |  | 0.567 |
| `vfx/effekseer-magicalxspiral/Breakdown.ogt` | Breakdown | explosion |  | 0.517 |
| `vfx/effekseer-magicalxspiral/FirePot1.ogt` | Fire Pot 1 | fire | fire | 1 |
| `vfx/effekseer-magicalxspiral/FirePot2.ogt` | Fire Pot 2 | fire | fire | 1 |
| `vfx/effekseer-magicalxspiral/Gohlem1.ogt` | Gohlem 1 | magic-impact | earth | 0.817 |
| `vfx/effekseer-magicalxspiral/Gohlem11.ogt` | Gohlem 11 | magic-impact | earth | 0.858 |
| `vfx/effekseer-magicalxspiral/Gohlem5.ogt` | Gohlem 5 | magic-impact | earth | 0.983 |
| `vfx/effekseer-magicalxspiral/Gohlem7.ogt` | Gohlem 7 | magic-impact | earth | 2.917 |
| `vfx/effekseer-magicalxspiral/Gohlem_Shield1.ogt` | Gohlem Shield 1 | shield | earth | 0.883 |
| `vfx/effekseer-magicalxspiral/Gohlem_Shield2.ogt` | Gohlem Shield 2 | shield | earth | 1.833 |
| `vfx/effekseer-magicalxspiral/Holy1.ogt` | Holy 1 | magic-impact | holy | 0.45 |
| `vfx/effekseer-magicalxspiral/Holy2.ogt` | Holy 2 | magic-impact | holy | 0.583 |
| `vfx/effekseer-magicalxspiral/Icicle1.ogt` | Icicle 1 | magic-projectile | ice | 0.4 |
| `vfx/effekseer-magicalxspiral/Icicle2.ogt` | Icicle 2 | magic-projectile | ice | 0.167 |
| `vfx/effekseer-magicalxspiral/Icicle3.ogt` | Icicle 3 | magic-projectile | ice | 0.167 |
| `vfx/effekseer-magicalxspiral/IcicleBroken.ogt` | Icicle Broken | explosion | ice | 0.517 |
| `vfx/effekseer-magicalxspiral/IcicleLocus.ogt` | Icicle Locus | magic-projectile | ice | 0.4 |
| `vfx/effekseer-magicalxspiral/Impact.ogt` | Impact | magic-impact |  | 0.7 |
| `vfx/effekseer-magicalxspiral/Lance3.ogt` | Lance 3 | magic-projectile |  | 2.15 |
| `vfx/effekseer-magicalxspiral/Lance4.ogt` | Lance 4 | magic-projectile |  | 1.15 |
| `vfx/effekseer-magicalxspiral/Lance5.ogt` | Lance 5 | magic-projectile |  | 1.15 |
| `vfx/effekseer-magicalxspiral/Magic1_earth.ogt` | Magic 1 Earth | magic-impact | earth | 1.667 |
| `vfx/effekseer-magicalxspiral/Magic1_fire.ogt` | Magic 1 Fire | fire | fire | 1.667 |
| `vfx/effekseer-magicalxspiral/Magic1_holy.ogt` | Magic 1 Holy | magic-impact | holy | 1.667 |
| `vfx/effekseer-magicalxspiral/Magic1_water.ogt` | Magic 1 Water | magic-impact | water | 1.667 |
| `vfx/effekseer-magicalxspiral/Magic1_wind.ogt` | Magic 1 Wind | magic-impact | wind | 1.667 |
| `vfx/effekseer-magicalxspiral/Magic2_earth.ogt` | Magic 2 Earth | magic-impact | earth | 0.483 |
| `vfx/effekseer-magicalxspiral/Magic2_fire.ogt` | Magic 2 Fire | fire | fire | 0.483 |
| `vfx/effekseer-magicalxspiral/Magic2_holy.ogt` | Magic 2 Holy | magic-impact | holy | 0.483 |
| `vfx/effekseer-magicalxspiral/Magic2_water.ogt` | Magic 2 Water | magic-impact | water | 0.483 |
| `vfx/effekseer-magicalxspiral/Magic2_wind.ogt` | Magic 2 Wind | magic-impact | wind | 0.483 |
| `vfx/effekseer-magicalxspiral/MagicArea.ogt` | Magic Area | portal |  | 2.917 |
| `vfx/effekseer-magicalxspiral/Rock1.ogt` | Rock 1 | magic-impact | earth | 1.667 |
| `vfx/effekseer-magicalxspiral/Rock2.ogt` | Rock 2 | magic-impact | earth | 0.817 |
| `vfx/effekseer-magicalxspiral/Salamander1.ogt` | Salamander 1 | magic-impact | fire | 0.45 |
| `vfx/effekseer-magicalxspiral/Salamander11.ogt` | Salamander 11 | magic-impact | fire | 1.317 |
| `vfx/effekseer-magicalxspiral/Salamander12.ogt` | Salamander 12 | magic-impact | fire | 0.883 |
| `vfx/effekseer-magicalxspiral/Salamander14.ogt` | Salamander 14 | magic-impact | fire | 0.833 |
| `vfx/effekseer-magicalxspiral/Salamander15.ogt` | Salamander 15 | magic-impact | fire | 2.7 |
| `vfx/effekseer-magicalxspiral/Salamander2.ogt` | Salamander 2 | magic-impact | fire | 0.583 |
| `vfx/effekseer-magicalxspiral/Salamander3.ogt` | Salamander 3 | magic-impact | fire | 1.317 |
| `vfx/effekseer-magicalxspiral/Salamander4.ogt` | Salamander 4 | magic-impact | fire | 1.317 |
| `vfx/effekseer-magicalxspiral/Salamander5.ogt` | Salamander 5 | magic-impact | fire | 1.317 |
| `vfx/effekseer-magicalxspiral/Salamander6.ogt` | Salamander 6 | magic-impact | fire | 1.233 |
| `vfx/effekseer-magicalxspiral/Salamander7.ogt` | Salamander 7 | magic-impact | fire | 0.717 |
| `vfx/effekseer-magicalxspiral/Snowman1.ogt` | Snowman 1 | weather | ice | 0.417 |
| `vfx/effekseer-magicalxspiral/StairBroken.ogt` | Stair Broken | explosion | earth | 0.633 |
| `vfx/effekseer-magicalxspiral/StairBroken3.ogt` | Stair Broken 3 | explosion | earth | 1.317 |
| `vfx/effekseer-magicalxspiral/Sylph1.ogt` | Sylph 1 | magic-impact | wind | 0.383 |
| `vfx/effekseer-magicalxspiral/Sylph10.ogt` | Sylph 10 | magic-impact | wind | 1.55 |
| `vfx/effekseer-magicalxspiral/Sylph11.ogt` | Sylph 11 | magic-impact | wind | 0.883 |
| `vfx/effekseer-magicalxspiral/Sylph13.ogt` | Sylph 13 | magic-impact | wind | 2.517 |
| `vfx/effekseer-magicalxspiral/Sylph15.ogt` | Sylph 15 | magic-impact | wind | 0.817 |
| `vfx/effekseer-magicalxspiral/Sylph2.ogt` | Sylph 2 | magic-impact | wind | 0.575 |
| `vfx/effekseer-magicalxspiral/Sylph3.ogt` | Sylph 3 | magic-impact | wind | 0.45 |
| `vfx/effekseer-magicalxspiral/Sylph4.ogt` | Sylph 4 | magic-impact | wind | 0.583 |
| `vfx/effekseer-magicalxspiral/Sylph5.ogt` | Sylph 5 | magic-impact | wind | 2.7 |
| `vfx/effekseer-magicalxspiral/Sylph6.ogt` | Sylph 6 | magic-impact | wind | 0.367 |
| `vfx/effekseer-magicalxspiral/Sylph7.ogt` | Sylph 7 | magic-impact | wind | 1.667 |
| `vfx/effekseer-magicalxspiral/Sylph8.ogt` | Sylph 8 | magic-impact | wind | 1.167 |
| `vfx/effekseer-magicalxspiral/Thunder1.ogt` | Thunder 1 | magic-impact | lightning | 2.15 |
| `vfx/effekseer-magicalxspiral/Thunder2.ogt` | Thunder 2 | magic-impact | lightning | 0.817 |
| `vfx/effekseer-magicalxspiral/Thunder3.ogt` | Thunder 3 | magic-impact | lightning | 0.817 |
| `vfx/effekseer-magicalxspiral/Topwater1.ogt` | Topwater 1 | magic-impact | water | 0.65 |
| `vfx/effekseer-magicalxspiral/Undine1.ogt` | Undine 1 | magic-impact | water | 1 |
| `vfx/effekseer-magicalxspiral/Undine11.ogt` | Undine 11 | magic-impact | water | 10.083 |
| `vfx/effekseer-magicalxspiral/Undine13.ogt` | Undine 13 | magic-impact | water | 3.917 |
| `vfx/effekseer-magicalxspiral/Undine2.ogt` | Undine 2 | magic-impact | water | 0.45 |
| `vfx/effekseer-magicalxspiral/Undine3.ogt` | Undine 3 | magic-impact | water | 0.583 |
| `vfx/effekseer-magicalxspiral/Undine4.ogt` | Undine 4 | magic-impact | water | 1.667 |
| `vfx/effekseer-magicalxspiral/Undine5.ogt` | Undine 5 | magic-impact | water | 0.667 |
| `vfx/effekseer-magicalxspiral/Undine6.ogt` | Undine 6 | magic-impact | water | 1.833 |
| `vfx/effekseer-magicalxspiral/Undine8.ogt` | Undine 8 | magic-impact | water | 1.667 |
| `vfx/effekseer-magicalxspiral/WHead1.ogt` | WHead 1 | magic-impact | water | 0.417 |
| `vfx/effekseer-magicalxspiral/WHead2.ogt` | WHead 2 | magic-impact | water | 0.567 |
| `vfx/effekseer-magicalxspiral/WHead3.ogt` | WHead 3 | magic-impact | water | 0.567 |
| `vfx/effekseer-magicalxspiral/Water_Impact.ogt` | Water Impact | magic-impact | water | 0.65 |
| `vfx/effekseer-magicalxspiral/Wind_Impact.ogt` | Wind Impact | magic-impact | wind | 0.4 |
| `vfx/effekseer-magicalxspiral/foot_smoke1.ogt` | Foot Smoke 1 | smoke |  | 0.25 |
| `vfx/effekseer-magicalxspiral/foot_smoke2.ogt` | Foot Smoke 2 | smoke |  | 1.233 |
| `vfx/effekseer-magicalxspiral/foot_smoke3.ogt` | Foot Smoke 3 | smoke |  | 0.9 |
| `vfx/effekseer-magicalxspiral/foot_smoke4.ogt` | Foot Smoke 4 | smoke |  | 0.9 |
| `vfx/effekseer-magicalxspiral/foot_smoke5.ogt` | Foot Smoke 5 | smoke |  | 0.9 |
| `vfx/effekseer-magicalxspiral/foot_smoke6.ogt` | Foot Smoke 6 | smoke |  | 0.567 |
| `vfx/effekseer-magicalxspiral/foot_smoke7.ogt` | Foot Smoke 7 | smoke |  | 0.9 |
| `vfx/effekseer-magicalxspiral/foot_smoke8.ogt` | Foot Smoke 8 | smoke |  | 0.9 |
| `vfx/effekseer-magicalxspiral/meteo1.ogt` | Meteo 1 | magic-projectile | fire | 0.858 |
| `vfx/effekseer-magicalxspiral/salamander13.ogt` | Salamander 13 | magic-impact | fire | 1.683 |

## 7. effekseer-nextsoft01

NextSoft01's Effekseer sample effects (20), converted to type vfx. 20 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-nextsoft01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, magic-impact, magic-projectile, shield, slash, weather
- Elements: dark, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, nextsoft01, particles

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-nextsoft01/HealPotion1.ogt` | Heal Potion 1 | heal | holy | 1.267 |
| `vfx/effekseer-nextsoft01/HealPotion2.ogt` | Heal Potion 2 | heal | holy | 1.63 |
| `vfx/effekseer-nextsoft01/MagicCold.ogt` | Magic Cold | magic-impact | ice | 2.15 |
| `vfx/effekseer-nextsoft01/MagicDark.ogt` | Magic Dark | magic-impact | dark | 2.05 |
| `vfx/effekseer-nextsoft01/MagicFire1.ogt` | Magic Fire 1 | fire | fire | 2.329 |
| `vfx/effekseer-nextsoft01/MagicFire2.ogt` | Magic Fire 2 | fire | fire | 1.91 |
| `vfx/effekseer-nextsoft01/MagicHeal1.ogt` | Magic Heal 1 | heal | holy | 1.967 |
| `vfx/effekseer-nextsoft01/MagicHeal2.ogt` | Magic Heal 2 | heal | holy | 4 |
| `vfx/effekseer-nextsoft01/MagicLight.ogt` | Magic Light | magic-impact | holy | 1.317 |
| `vfx/effekseer-nextsoft01/MagicMeteo.ogt` | Magic Meteo | magic-projectile | fire | 2.417 |
| `vfx/effekseer-nextsoft01/MagicShield.ogt` | Magic Shield | shield |  | 0.183 |
| `vfx/effekseer-nextsoft01/MagicSword.ogt` | Magic Sword | slash |  | 1.733 |
| `vfx/effekseer-nextsoft01/MagicThunder.ogt` | Magic Thunder | magic-impact | lightning | 1.05 |
| `vfx/effekseer-nextsoft01/MagicTornade.ogt` | Magic Tornade | weather | wind | 0.817 |
| `vfx/effekseer-nextsoft01/MagicWater.ogt` | Magic Water | magic-impact | water | 2.09 |
| `vfx/effekseer-nextsoft01/PowerUp.ogt` | Power Up | buff |  | 1.75 |
| `vfx/effekseer-nextsoft01/Spear1.ogt` | Spear 1 | magic-projectile |  | 0.367 |
| `vfx/effekseer-nextsoft01/Spear2.ogt` | Spear 2 | magic-projectile |  | 0.983 |
| `vfx/effekseer-nextsoft01/Sword1.ogt` | Sword 1 | slash |  | 0.333 |
| `vfx/effekseer-nextsoft01/Sword2.ogt` | Sword 2 | slash |  | 1.1 |

## 8. effekseer-pierre01

Pierre vol.1's Effekseer sample effects (11), converted to type vfx. 11 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre01`.

- Style: stylised, 3d
- Categories: ambient, buff, fire, magic-impact, magic-projectile, weather
- Elements: fire, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre01

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-pierre01/Atmosphere.ogt` | Atmosphere | ambient |  | loops |
| `vfx/effekseer-pierre01/Background.ogt` | Background | ambient |  | 19.333 |
| `vfx/effekseer-pierre01/Flame.ogt` | Flame | fire | fire | 5.283 |
| `vfx/effekseer-pierre01/Herald.ogt` | Herald | buff |  | 2.083 |
| `vfx/effekseer-pierre01/HolySandstorm.ogt` | Holy Sandstorm | weather | wind | 6.733 |
| `vfx/effekseer-pierre01/LightningStrike.ogt` | Lightning Strike | magic-impact | lightning | 2.783 |
| `vfx/effekseer-pierre01/MissileBoost.ogt` | Missile Boost | magic-projectile |  | loops |
| `vfx/effekseer-pierre01/MonsterRoar.ogt` | Monster Roar | magic-impact |  | 2.417 |
| `vfx/effekseer-pierre01/PhantasmMeteor.ogt` | Phantasm Meteor | magic-projectile | fire | 4.15 |
| `vfx/effekseer-pierre01/PhantasmMeteor_Single.ogt` | Phantasm Meteor Single | magic-projectile | fire | 3.612 |
| `vfx/effekseer-pierre01/SonicBoom.ogt` | Sonic Boom | magic-projectile |  | 1.833 |

## 9. effekseer-pierre02

Pierre vol.2's Effekseer sample effects (12), converted to type vfx. 12 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre02`.

- Style: stylised, 3d
- Categories: explosion, heal, magic-impact, magic-projectile, portal, slash, smoke
- Elements: dark, fire, holy, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre02

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-pierre02/Benediction.ogt` | Benediction | heal | holy | 5.933 |
| `vfx/effekseer-pierre02/BloodLance.ogt` | Blood Lance | magic-projectile | dark | loops |
| `vfx/effekseer-pierre02/CosmicMist.ogt` | Cosmic Mist | smoke | dark | 7.267 |
| `vfx/effekseer-pierre02/FeatherBomb.ogt` | Feather Bomb | explosion |  | 13.006 |
| `vfx/effekseer-pierre02/FireBall.ogt` | Fire Ball | magic-projectile | fire | 5.517 |
| `vfx/effekseer-pierre02/GateOfCalve.ogt` | Gate Of Calve | portal |  | 14.083 |
| `vfx/effekseer-pierre02/LossOfBlood.ogt` | Loss Of Blood | magic-impact | dark | 1.85 |
| `vfx/effekseer-pierre02/Sword_Ember.ogt` | Sword Ember | slash | fire | loops |
| `vfx/effekseer-pierre02/Sword_Lightning.ogt` | Sword Lightning | slash | lightning | loops |
| `vfx/effekseer-pierre02/Sword_Moonlight.ogt` | Sword Moonlight | slash | holy | loops |
| `vfx/effekseer-pierre02/Sword_Sacredlight.ogt` | Sword Sacredlight | slash | holy | loops |
| `vfx/effekseer-pierre02/Sword_Storm.ogt` | Sword Storm | slash | wind | loops |

## 10. effekseer-suzuki01

Suzuki01's Effekseer sample effects (3), converted to type vfx. 3 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-suzuki01`.

- Style: stylised, 3d
- Categories: buff, slash, weather
- Elements: ice
- Keywords: 3d, anime, effekseer, particles, suzuki01

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-suzuki01/aura.ogt` | Aura | buff |  | loops |
| `vfx/effekseer-suzuki01/snowstorm11.ogt` | Snowstorm 11 | weather | ice | 8.917 |
| `vfx/effekseer-suzuki01/sword.ogt` | Sword | slash |  | loops |

## 11. effekseer-tktk01

tktk vol.1's Effekseer sample effects (71), converted to type vfx. 71 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, earth, fire, holy, wind
- Keywords: 3d, anime, effekseer, particles, tktk01

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-tktk01/Arrow1.ogt` | Arrow 1 | magic-projectile |  | 0.454 |
| `vfx/effekseer-tktk01/Arrow2.ogt` | Arrow 2 | magic-projectile |  | 2.092 |
| `vfx/effekseer-tktk01/Blow1.ogt` | Blow 1 | hit-spark |  | 0.775 |
| `vfx/effekseer-tktk01/Blow10.ogt` | Blow 10 | hit-spark |  | 0.867 |
| `vfx/effekseer-tktk01/Blow11.ogt` | Blow 11 | hit-spark |  | 0.65 |
| `vfx/effekseer-tktk01/Blow12.ogt` | Blow 12 | hit-spark |  | 1.816 |
| `vfx/effekseer-tktk01/Blow13.ogt` | Blow 13 | hit-spark |  | 0.832 |
| `vfx/effekseer-tktk01/Blow14.ogt` | Blow 14 | hit-spark |  | 1.4 |
| `vfx/effekseer-tktk01/Blow15.ogt` | Blow 15 | hit-spark |  | 0.908 |
| `vfx/effekseer-tktk01/Blow2.ogt` | Blow 2 | hit-spark |  | 0.533 |
| `vfx/effekseer-tktk01/Blow3.ogt` | Blow 3 | hit-spark |  | 0.583 |
| `vfx/effekseer-tktk01/Blow4.ogt` | Blow 4 | hit-spark |  | 0.845 |
| `vfx/effekseer-tktk01/Blow5.ogt` | Blow 5 | hit-spark |  | 1.775 |
| `vfx/effekseer-tktk01/Blow6.ogt` | Blow 6 | hit-spark |  | 2.367 |
| `vfx/effekseer-tktk01/Blow7.ogt` | Blow 7 | hit-spark |  | 1.678 |
| `vfx/effekseer-tktk01/Blow8.ogt` | Blow 8 | hit-spark |  | 3.017 |
| `vfx/effekseer-tktk01/Blow9.ogt` | Blow 9 | hit-spark |  | 1.667 |
| `vfx/effekseer-tktk01/Cure1.ogt` | Cure 1 | heal | holy | 1.275 |
| `vfx/effekseer-tktk01/Cure2.ogt` | Cure 2 | heal | holy | 1.779 |
| `vfx/effekseer-tktk01/Cure3.ogt` | Cure 3 | heal | holy | 1.252 |
| `vfx/effekseer-tktk01/Cure4.ogt` | Cure 4 | heal | holy | 0.967 |
| `vfx/effekseer-tktk01/Cure5.ogt` | Cure 5 | heal | holy | 1.433 |
| `vfx/effekseer-tktk01/Cure6.ogt` | Cure 6 | heal | holy | 1.251 |
| `vfx/effekseer-tktk01/Cure7.ogt` | Cure 7 | heal | holy | 2.6 |
| `vfx/effekseer-tktk01/Dark1.ogt` | Dark 1 | magic-impact | dark | 1.617 |
| `vfx/effekseer-tktk01/Dark2.ogt` | Dark 2 | magic-impact | dark | 1.7 |
| `vfx/effekseer-tktk01/Dark3.ogt` | Dark 3 | magic-impact | dark | 3.333 |
| `vfx/effekseer-tktk01/Fire1.ogt` | Fire 1 | fire | fire | loops |
| `vfx/effekseer-tktk01/Fire2.ogt` | Fire 2 | fire | fire | 3.013 |
| `vfx/effekseer-tktk01/Fire3.ogt` | Fire 3 | fire | fire | loops |
| `vfx/effekseer-tktk01/Fire4.ogt` | Fire 4 | fire | fire | 2.425 |
| `vfx/effekseer-tktk01/Fire5.ogt` | Fire 5 | fire | fire | 1.442 |
| `vfx/effekseer-tktk01/Fire6.ogt` | Fire 6 | fire | fire | 1.251 |
| `vfx/effekseer-tktk01/Fire7.ogt` | Fire 7 | fire | fire | loops |
| `vfx/effekseer-tktk01/Fire8.ogt` | Fire 8 | fire | fire | 1.925 |
| `vfx/effekseer-tktk01/Light1.ogt` | Light 1 | magic-impact | holy | 2.667 |
| `vfx/effekseer-tktk01/Light2.ogt` | Light 2 | magic-impact | holy | 0.55 |
| `vfx/effekseer-tktk01/Light3.ogt` | Light 3 | magic-impact | holy | 1 |
| `vfx/effekseer-tktk01/Light4.ogt` | Light 4 | magic-impact | holy | 2.092 |
| `vfx/effekseer-tktk01/Magic1.ogt` | Magic 1 | magic-impact |  | 1 |
| `vfx/effekseer-tktk01/Magic2.ogt` | Magic 2 | magic-impact |  | 2.471 |
| `vfx/effekseer-tktk01/Soil1.ogt` | Soil 1 | magic-impact | earth | loops |
| `vfx/effekseer-tktk01/Sword1.ogt` | Sword 1 | slash |  | 0.8 |
| `vfx/effekseer-tktk01/Sword10.ogt` | Sword 10 | slash |  | 0.837 |
| `vfx/effekseer-tktk01/Sword11.ogt` | Sword 11 | slash |  | 1.283 |
| `vfx/effekseer-tktk01/Sword12.ogt` | Sword 12 | slash |  | 0.797 |
| `vfx/effekseer-tktk01/Sword13.ogt` | Sword 13 | slash |  | 2.592 |
| `vfx/effekseer-tktk01/Sword14.ogt` | Sword 14 | slash |  | 2.302 |
| `vfx/effekseer-tktk01/Sword15.ogt` | Sword 15 | slash |  | 2.275 |
| `vfx/effekseer-tktk01/Sword16.ogt` | Sword 16 | slash |  | loops |
| `vfx/effekseer-tktk01/Sword17.ogt` | Sword 17 | slash |  | 1.068 |
| `vfx/effekseer-tktk01/Sword18.ogt` | Sword 18 | slash |  | 2.933 |
| `vfx/effekseer-tktk01/Sword19.ogt` | Sword 19 | slash |  | 2.49 |
| `vfx/effekseer-tktk01/Sword2.ogt` | Sword 2 | slash |  | 0.698 |
| `vfx/effekseer-tktk01/Sword20.ogt` | Sword 20 | slash |  | 2.867 |
| `vfx/effekseer-tktk01/Sword21.ogt` | Sword 21 | slash |  | 3.333 |
| `vfx/effekseer-tktk01/Sword22.ogt` | Sword 22 | slash |  | 1.983 |
| `vfx/effekseer-tktk01/Sword3.ogt` | Sword 3 | slash |  | 1.032 |
| `vfx/effekseer-tktk01/Sword4.ogt` | Sword 4 | slash |  | 1.333 |
| `vfx/effekseer-tktk01/Sword5.ogt` | Sword 5 | slash |  | 0.6 |
| `vfx/effekseer-tktk01/Sword6.ogt` | Sword 6 | slash |  | 0.682 |
| `vfx/effekseer-tktk01/Sword7.ogt` | Sword 7 | slash |  | 0.583 |
| `vfx/effekseer-tktk01/Sword8.ogt` | Sword 8 | slash |  | 1.417 |
| `vfx/effekseer-tktk01/Sword9.ogt` | Sword 9 | slash |  | 0.867 |
| `vfx/effekseer-tktk01/Wind1.ogt` | Wind 1 | magic-impact | wind | 2.967 |
| `vfx/effekseer-tktk01/Wind2.ogt` | Wind 2 | magic-impact | wind | 0.673 |
| `vfx/effekseer-tktk01/hozyo1.ogt` | Hozyo 1 | buff |  | 1.291 |
| `vfx/effekseer-tktk01/hozyo2.ogt` | Hozyo 2 | buff |  | 1.217 |
| `vfx/effekseer-tktk01/hozyo3.ogt` | Hozyo 3 | buff |  | 0.8 |
| `vfx/effekseer-tktk01/hozyo4.ogt` | Hozyo 4 | buff |  | loops |
| `vfx/effekseer-tktk01/hozyo5.ogt` | Hozyo 5 | buff |  | loops |

## 12. effekseer-tktk02

tktk vol.2's Effekseer sample effects (33), converted to type vfx. 33 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk02`.

- Style: stylised, 3d
- Categories: fire, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, fire, holy
- Keywords: 3d, anime, effekseer, particles, tktk02

| Effect | Title | Category | Element | Seconds |
|---|---|---|---|---|
| `vfx/effekseer-tktk02/Blow1.ogt` | Blow 1 | hit-spark |  | 1.167 |
| `vfx/effekseer-tktk02/Blow2.ogt` | Blow 2 | hit-spark |  | 2.383 |
| `vfx/effekseer-tktk02/Blow3.ogt` | Blow 3 | hit-spark |  | 2.4 |
| `vfx/effekseer-tktk02/Blow4.ogt` | Blow 4 | hit-spark |  | 1 |
| `vfx/effekseer-tktk02/Blow5.ogt` | Blow 5 | hit-spark |  | 1.332 |
| `vfx/effekseer-tktk02/Blow6.ogt` | Blow 6 | hit-spark |  | 3.6 |
| `vfx/effekseer-tktk02/Claw1.ogt` | Claw 1 | slash |  | loops |
| `vfx/effekseer-tktk02/Claw2.ogt` | Claw 2 | slash |  | loops |
| `vfx/effekseer-tktk02/Claw3.ogt` | Claw 3 | slash |  | 2.733 |
| `vfx/effekseer-tktk02/Claw4.ogt` | Claw 4 | slash |  | loops |
| `vfx/effekseer-tktk02/Dark1.ogt` | Dark 1 | magic-impact | dark | 1.165 |
| `vfx/effekseer-tktk02/Dark2.ogt` | Dark 2 | magic-impact | dark | 3.033 |
| `vfx/effekseer-tktk02/Fire1.ogt` | Fire 1 | fire | fire | 2.967 |
| `vfx/effekseer-tktk02/Fire2.ogt` | Fire 2 | fire | fire | 3.317 |
| `vfx/effekseer-tktk02/Fire3.ogt` | Fire 3 | fire | fire | 1.3 |
| `vfx/effekseer-tktk02/Fire4.ogt` | Fire 4 | fire | fire | loops |
| `vfx/effekseer-tktk02/Fire5.ogt` | Fire 5 | fire | fire | 2.533 |
| `vfx/effekseer-tktk02/Fire6.ogt` | Fire 6 | fire | fire | 1.371 |
| `vfx/effekseer-tktk02/Fire7.ogt` | Fire 7 | fire | fire | 2.998 |
| `vfx/effekseer-tktk02/Fire8.ogt` | Fire 8 | fire | fire | loops |
| `vfx/effekseer-tktk02/Gun1.ogt` | Gun 1 | magic-projectile |  | 2.036 |
| `vfx/effekseer-tktk02/Gun2.ogt` | Gun 2 | magic-projectile |  | 2.968 |
| `vfx/effekseer-tktk02/Gun3.ogt` | Gun 3 | magic-projectile |  | 2.169 |
| `vfx/effekseer-tktk02/Gun4.ogt` | Gun 4 | magic-projectile |  | loops |
| `vfx/effekseer-tktk02/Gun5.ogt` | Gun 5 | magic-projectile |  | 1.733 |
| `vfx/effekseer-tktk02/Gun6.ogt` | Gun 6 | magic-projectile |  | 2.217 |
| `vfx/effekseer-tktk02/Gun7.ogt` | Gun 7 | magic-projectile |  | 3.2 |
| `vfx/effekseer-tktk02/Gun8.ogt` | Gun 8 | magic-projectile |  | 2.1 |
| `vfx/effekseer-tktk02/Gun9.ogt` | Gun 9 | magic-projectile |  | 3.067 |
| `vfx/effekseer-tktk02/Light1.ogt` | Light 1 | magic-impact | holy | 1.082 |
| `vfx/effekseer-tktk02/Light2.ogt` | Light 2 | magic-impact | holy | 2.6 |
| `vfx/effekseer-tktk02/Light3.ogt` | Light 3 | magic-impact | holy | loops |
| `vfx/effekseer-tktk02/Light4.ogt` | Light 4 | magic-impact | holy | 2.777 |
