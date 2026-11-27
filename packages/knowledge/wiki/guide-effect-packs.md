<!-- Generated from crates/og-api/api.ogt by `og api gen`. Do not edit: `og api check` fails on any difference; change api.ogt and regenerate. -->

# The `effect-packs` guide: The effect packs

Ready-made effects a game installs with `og vfx add <pack>`: checked 3D timelines (`file.vfx`) and pixel-art recipes (`file.vfx2d`), each listed with its tags, keywords and effects. A bundled pack ships with the engine; a downloadable one comes with each release and `og vfx add` fetches it. Effects install under content/ and load by `base/vfx/<pack>/<effect>` (`og api show vfx`; the `pixel-vfx` guide for `file.vfx2d`). The pack format is docs/vfx-packs.md.

The last four columns are what `og vfx preview` measured, on the device each pack names: Size is the box the effect reaches in metres (x × y × z, y up), Anchor `ground` means it stands on its origin and `centre` that it plays about it (a ring, a shield: raise it to stand on the ground), GPU ms its peak frame time over an empty frame, Draws the most draw calls of one of its frames. An empty cell is not measured yet.

<a id="guide-effect-packs-1"></a>
### `effect-packs §1` effekseer-starter

A starter set of Effekseer sample effects from nine contributors (fire, explosions, hits, slashes, heals, buffs, level-ups, projectiles, impacts, portals, weather, ambient), converted to type vfx. 38 effects, bundled with the engine: `og vfx add effekseer-starter`.

- Style: stylised, 3d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, weather
- Elements: dark, earth, fire, holy, ice, lightning, nature, water
- Keywords: 3d, anime, combat, effekseer, magic, particles, starter

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-starter/Attack_Impact` | Attack Impact | hit-spark |  | 0.567 |  |  | 0.32 | 24 |
| `base/vfx/effekseer-starter/Blow2` | Spark Hit | hit-spark |  | 0.533 |  |  | 0.63 | 19 |
| `base/vfx/effekseer-starter/Blow4` | Bright Explosion | explosion |  | 0.845 |  |  | 2.38 | 21 |
| `base/vfx/effekseer-starter/Blow8` | Star Hit | hit-spark |  | 3.017 |  |  | 2.78 | 25 |
| `base/vfx/effekseer-starter/Breakdown` | Breakdown Burst | explosion |  | 0.517 |  |  | 0.37 | 23 |
| `base/vfx/effekseer-starter/Claw3` | Claw | slash |  | 2.733 |  |  | 4.07 | 63 |
| `base/vfx/effekseer-starter/Cure1` | Star Cure | heal | holy | 1.275 |  |  | 1.25 | 24 |
| `base/vfx/effekseer-starter/Cure7` | Light Cure | heal | holy | 2.6 |  |  | 0.66 | 40 |
| `base/vfx/effekseer-starter/FeatherBomb` | Smoke Bomb | explosion |  | 13.006 |  |  | 13.07 | 55 |
| `base/vfx/effekseer-starter/Fire7` | Fire Pillar | fire | fire | 2.998 |  |  | 8.63 | 30 |
| `base/vfx/effekseer-starter/FireBall` | Fireball | magic-projectile | fire | 5.517 |  |  | 6.44 | 49 |
| `base/vfx/effekseer-starter/HealPotion2` | Potion | heal |  | 1.63 |  |  | 0.41 | 22 |
| `base/vfx/effekseer-starter/HolySandstorm` | Sandstorm | weather | earth | 6.733 |  |  | 6.77 | 27 |
| `base/vfx/effekseer-starter/Icicle2` | Icicles | magic-projectile | ice | 0.167 |  |  | 0.06 | 19 |
| `base/vfx/effekseer-starter/Laser03` | Green Laser | magic-projectile |  | loops |  |  | 0.59 | 20 |
| `base/vfx/effekseer-starter/Light4` | Level Up Arrows | level-up | holy | 2.092 |  |  | 3.79 | 78 |
| `base/vfx/effekseer-starter/Magic2_fire` | Fire Pentagram | portal | fire | 0.483 |  |  | 0.10 | 16 |
| `base/vfx/effekseer-starter/MagicCold` | Frost | magic-impact | ice | 2.15 |  |  | 0.39 | 17 |
| `base/vfx/effekseer-starter/MagicDark` | Dark Curse | magic-impact | dark | 2.05 |  |  | 0.93 | 19 |
| `base/vfx/effekseer-starter/MagicFire2` | Fire Breath | fire | fire | 1.91 |  |  | 0.26 | 15 |
| `base/vfx/effekseer-starter/MagicHeal2` | Healing Circle | heal | holy | 4 |  |  | 0.94 | 19 |
| `base/vfx/effekseer-starter/MagicWater` | Bubble Burst | magic-impact | water | 2.09 |  |  | 2.24 | 23 |
| `base/vfx/effekseer-starter/PowerUp` | Power Up | buff |  | 1.75 |  |  | 0.29 | 18 |
| `base/vfx/effekseer-starter/Spear2` | Light Spear | magic-projectile |  | 0.983 |  |  | 0.23 | 16 |
| `base/vfx/effekseer-starter/Sword5` | Blue Slash | slash |  | 0.6 |  |  | 4.22 | 17 |
| `base/vfx/effekseer-starter/Sword7` | Cyan Cross Slash | slash |  | 0.583 |  |  | 0.24 | 18 |
| `base/vfx/effekseer-starter/Sword8` | Red Arc Slash | slash |  | 1.417 |  |  | 0.60 | 21 |
| `base/vfx/effekseer-starter/Thunder1` | Golden Rise | level-up | lightning | 2.15 |  |  | 0.58 | 19 |
| `base/vfx/effekseer-starter/Undine4` | Water Burst | magic-impact | water | 1.667 |  |  | 0.48 | 18 |
| `base/vfx/effekseer-starter/boss_death` | Boss Death | explosion |  | loops |  |  | 0.25 | 17 |
| `base/vfx/effekseer-starter/electric_dust` | Floating Motes | ambient | lightning | 3.4 |  |  | 0.35 | 16 |
| `base/vfx/effekseer-starter/hit_eff` | Hit Flash | hit-spark |  | 0.622 |  |  | 0.34 | 24 |
| `base/vfx/effekseer-starter/hozyo2` | Power Rings | buff |  | 1.217 |  |  | 0.41 | 30 |
| `base/vfx/effekseer-starter/hozyo4` | Attack Up | buff | fire | loops |  |  | 4.94 | 24 |
| `base/vfx/effekseer-starter/magic_circle` | Magic Circle | portal |  | 16.683 |  |  | 0.46 | 26 |
| `base/vfx/effekseer-starter/meteo1` | Meteor Strike | explosion | fire | 0.858 |  |  | 0.37 | 23 |
| `base/vfx/effekseer-starter/sakura1` | Sakura Petals | ambient | nature | 13.8 |  |  | 1.52 | 52 |
| `base/vfx/effekseer-starter/smallTexturesSquareWave_orbital` | Arcane Ring | portal |  | 66.667 |  |  | 0.07 | 17 |

<a id="guide-effect-packs-2"></a>
### `effect-packs §2` pixel-scenario

The Pixel VFX Forge library: 42 pixel-art effects (combat, magic, elements, feedback, ambient) as type vfx2d recipes. 42 effects, bundled with the engine: `og vfx add pixel-scenario`.

- Style: pixel, 2d
- Categories: ambient, buff, explosion, fire, heal, hit-spark, level-up, magic-impact, magic-projectile, portal, slash, smoke, ui, weather
- Elements: arcane, dark, earth, fire, holy, ice, lightning, nature, water, wind
- Keywords: 2d, flipbook, forge, pixel, pixel art, scenario, sprite

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/pixel-scenario/arcane-magic-circle` | Eldritch Magic Circle | magic-impact | arcane | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/big-fire-explosion` | Cinder Explosion | explosion | fire | 1.2 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/blue-energy-beam` | Aether Beam | magic-projectile | arcane | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/campfire` | Ember Flame | fire | fire | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/charge-up-energy` | Pulse Charge | buff | arcane | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/cherry-blossom-petals` | Thorn Ambient | ambient | nature | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/crescent-sword-slash` | Razor Slash | slash |  | 0.667 |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/critical-hit-damage-999` | Ember Damage Number | ui |  | 1.067 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/dark-shadow-aura` | Night Aura | buff | dark | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/dash-speed-lines` | Cyclone Dash | slash | wind | 0.556 |  |  | 1.42 | 13 |
| `base/vfx/pixel-scenario/dimensional-portal` | Astral Portal | portal | arcane | loops |  |  | 0.02 | 13 |
| `base/vfx/pixel-scenario/dragon-fire-breath` | Cinder Breath | fire | fire | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/electric-impact-hit` | Static Impact | hit-spark | lightning | 0.556 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/embers-rising` | Blaze Ambient | ambient | fire | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/falling-leaves` | Bloom Ambient | ambient | nature | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/feral-claw-scratch-blood` | Sanguine Claw | slash |  | 0.667 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/fire-tornado` | Blaze Tornado | fire | fire | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/fireball-projectile` | Ember Projectile | magic-projectile | fire | loops |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/fireflies` | Mystic Ambient | ambient | fire | loops |  |  | 0.02 | 13 |
| `base/vfx/pixel-scenario/fireworks-celebration` | Arcane Fireworks | ui | fire | 2.133 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/frost-nova` | Frost Nova | magic-impact | ice | 1.067 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/gold-coins-loot-drop` | Lucky Loot | ui |  | 1.6 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/ground-slam-quake` | Stone Ground Slam | magic-impact | earth | 1.2 |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/healing-with-rising-crosses` | Bloom Heal | heal | holy | 1.333 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/hex-shield-barrier` | Flux Shield | buff | arcane | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/holy-pillar-of-light` | Divine Pillar | magic-impact | holy | 1.333 |  |  | 0.02 | 13 |
| `base/vfx/pixel-scenario/ice-spikes-erupting` | Frozen Spikes | magic-impact | ice | 1.2 |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/lava-eruption-pillar` | Volcanic Pillar | magic-impact | fire | 1.333 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/level-up` | Gilded Level Up | level-up |  | 1.6 |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/lightning-strike-from-the-sky` | Storm Strike | magic-impact | lightning | 0.889 |  |  | 0.00 | 13 |
| `base/vfx/pixel-scenario/meteor-strike` | Ember Meteor | explosion | fire | 1.733 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/plasma-orb` | Ion Orb | magic-projectile | lightning | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/poison-gas-cloud` | Blight Cloud | smoke | nature | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/purple-black-hole-vortex` | Shadow Vortex | portal | dark | loops |  |  | 0.03 | 13 |
| `base/vfx/pixel-scenario/rain` | Aqua Ambient | weather | water | loops |  |  | 0.05 | 13 |
| `base/vfx/pixel-scenario/smoke-poof` | Ashen Poof | smoke |  | 0.933 |  |  | 0.12 | 13 |
| `base/vfx/pixel-scenario/snow-falling` | Glacial Ambient | weather | ice | loops |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/sparkle-twinkle` | Sacred Sparkle | ui |  | loops |  |  | 0.04 | 13 |
| `base/vfx/pixel-scenario/spinning-wind-slash` | Cyclone Slash | slash | wind | 0.667 |  |  | 0.01 | 13 |
| `base/vfx/pixel-scenario/teleport-blink` | Mystic Teleport | portal | arcane | 1.333 |  |  | 0.06 | 13 |
| `base/vfx/pixel-scenario/triple-slash-combo-with-blue-energy` | Flux Slash | slash | arcane | 1.167 |  |  | 0.02 | 13 |
| `base/vfx/pixel-scenario/water-splash` | Aqua Splash | magic-impact | water | 1.067 |  |  | 0.02 | 13 |

<a id="guide-effect-packs-3"></a>
### `effect-packs §3` realistic-starter

A starter set of realistic effects for 3D games: a muzzle flash, a bullet impact, an explosion, the dust of something breaking, a water splash, a torch's fire and dust in the air, built on the Kenney Particle Pack's textures. 7 effects, bundled with the engine: `og vfx add realistic-starter`.

- Style: realistic, 3d
- Categories: ambient, debris, explosion, fire, gunfire, hit-spark, splash
- Elements: earth, fire, water
- Keywords: 3d, debris, dust, gunfire, particles, realistic, shooter, smoke, starter

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/realistic-starter/ambient-dust` | Ambient Dust | ambient |  | loops |  |  |  |  |
| `base/vfx/realistic-starter/bullet-impact` | Bullet Impact | hit-spark |  | 1.8 |  |  |  |  |
| `base/vfx/realistic-starter/dust-burst` | Dust Burst | debris | earth | 3.6 |  |  |  |  |
| `base/vfx/realistic-starter/explosion` | Explosion | explosion | fire | 4 |  |  |  |  |
| `base/vfx/realistic-starter/muzzle-flash` | Muzzle Flash | gunfire | fire | 1.2 |  |  |  |  |
| `base/vfx/realistic-starter/torch-fire` | Torch Fire | fire | fire | loops |  |  |  |  |
| `base/vfx/realistic-starter/water-splash` | Water Splash | splash | water | 1.6 |  |  |  |  |

<a id="guide-effect-packs-4"></a>
### `effect-packs §4` cartoon-2d

Flat cartoon effects for cartoon-2d: clean shapes in two to four flat tones with a 2 px dark warm contour, drawn smooth. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/cartoon-2d/block` | Block | shield |  | 0.556 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/break-crate` | Crate Break | destruction | nature | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/break-ice` | Ice Shatter | destruction | ice | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/break-rock` | Rock Break | destruction | earth | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/bubbles` | Bubbles | ambient | water | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/buff` | Buff | buff |  | 1.067 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/bullet-hit` | Pellet Pop | hit-spark |  | 0.444 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/bullet` | Pellet | projectile |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/burning` | On Fire | status | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/campfire` | Campfire | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/charge` | Charge Ball | magic-impact | lightning | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/checkpoint` | Checkpoint | marker |  | 1.067 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/chest-open` | Treasure | pickup |  | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/chimney-smoke` | Chimney Smoke | smoke |  | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/coin` | Coin Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/confetti-rain` | Confetti Rain | ui |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/confetti` | Confetti Pop | ui |  | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/dash` | Dash Lines | movement | wind | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/debuff` | Debuff | status | dark | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/double-jump` | Air Jump | movement | wind | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/explosion-small` | Pop Boom | explosion | fire | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/explosion` | Boom | explosion | fire | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/fireflies` | Fireflies | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/fireworks` | Firework | ui | fire | 1.467 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/frozen` | Frozen | status | ice | 0.933 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/fuse` | Fuse Sparks | fire | fire | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/gem` | Gem Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/heal` | Heal | heal | nature | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/heart` | Heart Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/hit-heavy` | Smash Hit | hit-spark |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/hit-small` | Tap Hit | hit-spark |  | 0.444 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/hit` | Pow Hit | hit-spark |  | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/jump-dust` | Jump Dust | movement | earth | 0.556 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/land-dust` | Land Dust | movement | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/laser` | Ray Beam | beam | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/leaves` | Falling Leaves | ambient | nature | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/magic-bolt` | Magic Bolt | magic-projectile | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/magic-circle` | Spell Circle | magic-impact | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/magic` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/muzzle` | Pop Muzzle | gunfire | fire | 0.3 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/poison` | Poisoned | status | poison | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/poof` | Poof | smoke |  | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/portal` | Portal | magic-impact | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/powerup` | Power Up | buff |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/ripple` | Ripples | splash | water | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/run-dust` | Run Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/shield` | Shield Bubble | shield | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/shocked` | Zapped | status | lightning | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d/skid` | Skid Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/slash` | Swoosh Slash | slash |  | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/splash-small` | Plop | splash | water | 0.75 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/splash` | Splash | splash | water | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/star-collect` | Star Grab | pickup |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/stomp` | Stomp | hit-spark | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/stun` | Dizzy Stars | status |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/teleport` | Teleport | movement | arcane | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/torch` | Torch Flame | fire | fire | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/twinkle` | Twinkle | ambient |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d/whirlwind` | Whirlwind | magic-impact | wind | loops |  |  | 0.00 | 9 |

<a id="guide-effect-packs-5"></a>
### `effect-packs §5` cartoon-2d-rubber-hose

Rubber-hose effects for cartoon-2d's rubber-hose variant: muted primaries in a faded warm grade inked with a 3 px black line. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-rubber-hose`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, rubber-hose

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/cartoon-2d-rubber-hose/block` | Block | shield |  | 0.833 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/break-crate` | Crate Break | destruction | nature | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/break-ice` | Ice Shatter | destruction | ice | 1.167 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/break-rock` | Rock Break | destruction | earth | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/bubbles` | Bubbles | ambient | water | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/buff` | Buff | buff |  | 1.333 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/bullet-hit` | Pellet Pop | hit-spark |  | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/bullet` | Pellet | projectile |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/burning` | On Fire | status | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/campfire` | Campfire | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/charge` | Charge Ball | magic-impact | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/checkpoint` | Checkpoint | marker |  | 1.333 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/chest-open` | Treasure | pickup |  | 1.5 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/chimney-smoke` | Chimney Smoke | smoke |  | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/coin` | Coin Grab | pickup |  | 1 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/confetti-rain` | Confetti Rain | ui |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/confetti` | Confetti Pop | ui |  | 1.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/dash` | Dash Lines | movement | wind | 0.833 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/debuff` | Debuff | status | dark | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/double-jump` | Air Jump | movement | wind | 0.833 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/explosion-small` | Pop Boom | explosion | fire | 1.167 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/explosion` | Boom | explosion | fire | 1.5 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/fireflies` | Fireflies | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/fireworks` | Firework | ui | fire | 1.833 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/frozen` | Frozen | status | ice | 1.167 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/fuse` | Fuse Sparks | fire | fire | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/gem` | Gem Grab | pickup |  | 1 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/heal` | Heal | heal | nature | 1.5 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/heart` | Heart Grab | pickup |  | 1 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/hit-heavy` | Smash Hit | hit-spark |  | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/hit-small` | Tap Hit | hit-spark |  | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/hit` | Pow Hit | hit-spark |  | 0.833 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/jump-dust` | Jump Dust | movement | earth | 0.833 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/land-dust` | Land Dust | movement | earth | 1 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/laser` | Ray Beam | beam | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/leaves` | Falling Leaves | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/magic-bolt` | Magic Bolt | magic-projectile | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/magic-circle` | Spell Circle | magic-impact | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/magic` | Sparkle Cast | magic-impact | arcane | 1.167 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/muzzle` | Pop Muzzle | gunfire | fire | 0.5 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/poison` | Poisoned | status | poison | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/poof` | Poof | smoke |  | 1.167 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/portal` | Portal | magic-impact | arcane | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/powerup` | Power Up | buff |  | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/ripple` | Ripples | splash | water | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/run-dust` | Run Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/shield` | Shield Bubble | shield | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/shocked` | Zapped | status | lightning | 1 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/skid` | Skid Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/slash` | Swoosh Slash | slash |  | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/splash-small` | Plop | splash | water | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/splash` | Splash | splash | water | 1.167 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/star-collect` | Star Grab | pickup |  | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/stomp` | Stomp | hit-spark | earth | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/stun` | Dizzy Stars | status |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/teleport` | Teleport | movement | arcane | 1.5 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/torch` | Torch Flame | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/twinkle` | Twinkle | ambient |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-rubber-hose/whirlwind` | Whirlwind | magic-impact | wind | loops |  |  | 0.01 | 9 |

<a id="guide-effect-packs-6"></a>
### `effect-packs §6` cartoon-2d-storybook

Storybook effects for cartoon-2d's storybook variant: soft saturated shapes with a thin coloured line a shade darker than the fill. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-storybook`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, storybook

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/cartoon-2d-storybook/block` | Block | shield |  | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/break-crate` | Crate Break | destruction | nature | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/break-ice` | Ice Shatter | destruction | ice | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/break-rock` | Rock Break | destruction | earth | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/bubbles` | Bubbles | ambient | water | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/buff` | Buff | buff |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/bullet-hit` | Pellet Pop | hit-spark |  | 0.444 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/bullet` | Pellet | projectile |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/burning` | On Fire | status | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/campfire` | Campfire | fire | fire | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/charge` | Charge Ball | magic-impact | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/checkpoint` | Checkpoint | marker |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/chest-open` | Treasure | pickup |  | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/chimney-smoke` | Chimney Smoke | smoke |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/coin` | Coin Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/confetti-rain` | Confetti Rain | ui |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/confetti` | Confetti Pop | ui |  | 1.333 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/dash` | Dash Lines | movement | wind | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/debuff` | Debuff | status | dark | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/double-jump` | Air Jump | movement | wind | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/explosion-small` | Pop Boom | explosion | fire | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/explosion` | Boom | explosion | fire | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/fireflies` | Fireflies | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/fireworks` | Firework | ui | fire | 1.467 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/frozen` | Frozen | status | ice | 0.933 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/fuse` | Fuse Sparks | fire | fire | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/gem` | Gem Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/heal` | Heal | heal | nature | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/heart` | Heart Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/hit-heavy` | Smash Hit | hit-spark |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/hit-small` | Tap Hit | hit-spark |  | 0.444 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/hit` | Pow Hit | hit-spark |  | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/jump-dust` | Jump Dust | movement | earth | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/land-dust` | Land Dust | movement | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/laser` | Ray Beam | beam | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/leaves` | Falling Leaves | ambient | nature | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/magic-bolt` | Magic Bolt | magic-projectile | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/magic-circle` | Spell Circle | magic-impact | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/magic` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/muzzle` | Pop Muzzle | gunfire | fire | 0.3 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/poison` | Poisoned | status | poison | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/poof` | Poof | smoke |  | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/portal` | Portal | magic-impact | arcane | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/powerup` | Power Up | buff |  | 1.067 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/ripple` | Ripples | splash | water | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/run-dust` | Run Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/shield` | Shield Bubble | shield | arcane | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/shocked` | Zapped | status | lightning | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/skid` | Skid Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/slash` | Swoosh Slash | slash |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/splash-small` | Plop | splash | water | 0.75 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/splash` | Splash | splash | water | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/star-collect` | Star Grab | pickup |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/stomp` | Stomp | hit-spark | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/stun` | Dizzy Stars | status |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/teleport` | Teleport | movement | arcane | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-storybook/torch` | Torch Flame | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/twinkle` | Twinkle | ambient |  | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-storybook/whirlwind` | Whirlwind | magic-impact | wind | loops |  |  | 0.00 | 9 |

<a id="guide-effect-packs-7"></a>
### `effect-packs §7` cartoon-2d-watercolor

Watercolour effects for cartoon-2d's watercolor variant: light translucent-looking washes with no contour, a darker pigment rim instead. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-watercolor`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, watercolor

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/cartoon-2d-watercolor/block` | Block | shield |  | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/break-crate` | Crate Break | destruction | nature | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/break-ice` | Ice Shatter | destruction | ice | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/break-rock` | Rock Break | destruction | earth | 1 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/bubbles` | Bubbles | ambient | water | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/buff` | Buff | buff |  | 1.067 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/bullet-hit` | Pellet Pop | hit-spark |  | 0.444 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/bullet` | Pellet | projectile |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/burning` | On Fire | status | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/campfire` | Campfire | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/charge` | Charge Ball | magic-impact | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/checkpoint` | Checkpoint | marker |  | 1.067 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/chest-open` | Treasure | pickup |  | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/chimney-smoke` | Chimney Smoke | smoke |  | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/coin` | Coin Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/confetti-rain` | Confetti Rain | ui |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/confetti` | Confetti Pop | ui |  | 1.333 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/dash` | Dash Lines | movement | wind | 0.556 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/debuff` | Debuff | status | dark | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/double-jump` | Air Jump | movement | wind | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/explosion-small` | Pop Boom | explosion | fire | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/explosion` | Boom | explosion | fire | 1.2 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/fireflies` | Fireflies | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/fireworks` | Firework | ui | fire | 1.467 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/frozen` | Frozen | status | ice | 0.933 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/fuse` | Fuse Sparks | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/gem` | Gem Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/heal` | Heal | heal | nature | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/heart` | Heart Grab | pickup |  | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/hit-heavy` | Smash Hit | hit-spark |  | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/hit-small` | Tap Hit | hit-spark |  | 0.444 |  |  | 0.02 | 9 |
| `base/vfx/cartoon-2d-watercolor/hit` | Pow Hit | hit-spark |  | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/jump-dust` | Jump Dust | movement | earth | 0.556 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/land-dust` | Land Dust | movement | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/laser` | Ray Beam | beam | lightning | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/leaves` | Falling Leaves | ambient | nature | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/magic-bolt` | Magic Bolt | magic-projectile | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/magic-circle` | Spell Circle | magic-impact | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/magic` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/muzzle` | Pop Muzzle | gunfire | fire | 0.3 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/poison` | Poisoned | status | poison | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/poof` | Poof | smoke |  | 0.875 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/portal` | Portal | magic-impact | arcane | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/powerup` | Power Up | buff |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/ripple` | Ripples | splash | water | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/run-dust` | Run Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/shield` | Shield Bubble | shield | arcane | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/shocked` | Zapped | status | lightning | 0.667 |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/skid` | Skid Dust | movement | earth | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/slash` | Swoosh Slash | slash |  | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/splash-small` | Plop | splash | water | 0.75 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/splash` | Splash | splash | water | 0.875 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/star-collect` | Star Grab | pickup |  | 1.067 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/stomp` | Stomp | hit-spark | earth | 0.667 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/stun` | Dizzy Stars | status |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/teleport` | Teleport | movement | arcane | 1.2 |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/torch` | Torch Flame | fire | fire | loops |  |  | 0.00 | 9 |
| `base/vfx/cartoon-2d-watercolor/twinkle` | Twinkle | ambient |  | loops |  |  | 0.01 | 9 |
| `base/vfx/cartoon-2d-watercolor/whirlwind` | Whirlwind | magic-impact | wind | loops |  |  | 0.01 | 9 |

<a id="guide-effect-packs-8"></a>
### `effect-packs §8` cinematic-realism

Restrained cinematic effects: simulated fire and smoke, ballistic sparks, spray, motes and exhaust. Designed for cinematic-realism and muted scenes. 89 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cinematic-realism`.

- Style: realistic, 3d
- Categories: ambient, beam, buff, decal, destruction, explosion, fire, gunfire, heal, hit-spark, level-up, magic-impact, magic-projectile, marker, movement, portal, projectile, shield, slash, smoke, splash, status, ui, vehicle
- Elements: arcane, dark, earth, fire, ice, lightning, nature, water, wind
- Keywords: ballistics, cinematic-realism, decals, exhaust, muted, particles, realistic, smoke, sound, vehicles

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/cinematic-realism/afterburner` | Afterburner | vehicle | fire | loops | 0.8 × 3.2 × 0.8 | ground | 0.12 | 15 |
| `base/vfx/cinematic-realism/barrier` | Barrier Dome | shield | arcane | loops | 2.6 × 2.6 × 2.6 | ground | 0.06 | 13 |
| `base/vfx/cinematic-realism/beacon` | Marker Beacon | marker |  | loops | 0.8 × 9.1 × 0.8 | ground | 0.07 | 14 |
| `base/vfx/cinematic-realism/bolt` | Energy Bolt | projectile | lightning | loops | 0.4 × 1.1 × 0.4 | centre | 0.08 | 14 |
| `base/vfx/cinematic-realism/boost` | Booster Burst | vehicle | fire | 1.2 | 0.8 × 3.5 × 0.7 | ground | 0.12 | 14 |
| `base/vfx/cinematic-realism/break-glass` | Pane Break | destruction |  | 2.5 | 3.8 × 1.5 × 3.5 | ground | 0.07 | 14 |
| `base/vfx/cinematic-realism/break-stone` | Masonry Break | destruction | earth | 3.5 | 5.3 × 1.8 × 3.4 | ground | 0.14 | 16 |
| `base/vfx/cinematic-realism/break-wood` | Crate Break | destruction | nature | 3 | 5.2 × 1.4 × 3.7 | centre | 0.51 | 16 |
| `base/vfx/cinematic-realism/buff` | Rising Strands | buff |  | 1.6 | 0.9 × 1.8 × 0.9 | ground | 0.08 | 14 |
| `base/vfx/cinematic-realism/burning` | Burning | status | fire | loops | 0.8 × 2.7 × 0.6 | ground | 0.79 | 16 |
| `base/vfx/cinematic-realism/campfire` | Campfire | fire | fire | loops | 0.8 × 1.7 × 0.8 | ground | 0.23 | 19 |
| `base/vfx/cinematic-realism/car-exhaust` | Road Exhaust | vehicle |  | loops | 0.2 × 0.7 × 0.3 | ground | 0.18 | 14 |
| `base/vfx/cinematic-realism/celebration` | Satin Confetti | ui |  | 3 | 3.0 × 1.2 × 2.9 | ground | 0.07 | 12 |
| `base/vfx/cinematic-realism/charge` | Gathering Charge | magic-impact | arcane | loops | 1.6 × 1.5 × 1.6 | centre | 0.09 | 13 |
| `base/vfx/cinematic-realism/chest-open` | Opened Chest | marker |  | 1.8 | 0.8 × 1.1 × 0.8 | ground | 0.17 | 17 |
| `base/vfx/cinematic-realism/damage-smoke` | Damage Smoke | vehicle | fire | loops | 1.3 × 2.0 × 1.7 | ground | 0.18 | 14 |
| `base/vfx/cinematic-realism/dash` | Dash | movement |  | 1 | 0.8 × 3.0 × 0.6 | ground | 0.18 | 16 |
| `base/vfx/cinematic-realism/debuff` | Sinking Strands | status | dark | 1.6 | 1.0 × 1.8 × 1.1 | ground | 0.15 | 16 |
| `base/vfx/cinematic-realism/dissolve` | Ash Dissolve | status |  | 3.2 | 0.9 × 2.1 × 0.8 | ground | 0.16 | 17 |
| `base/vfx/cinematic-realism/downwash` | Rotor Downwash | vehicle | wind | loops | 9.9 × 5.0 × 8.7 | ground | 0.25 | 15 |
| `base/vfx/cinematic-realism/drip` | Drip | splash | water | loops | 0.1 × 0.0 × 0.1 | ground | 0.16 | 16 |
| `base/vfx/cinematic-realism/dust` | Rolling Dust | smoke | earth | 2.6 | 1.8 × 1.5 × 2.1 | ground | 0.15 | 15 |
| `base/vfx/cinematic-realism/electric-sparks` | Arcing Fault | ambient | lightning | loops | 0.9 × 0.5 × 1.3 | ground | 0.08 | 14 |
| `base/vfx/cinematic-realism/embers` | Drifting Embers | ambient | fire | loops | 4.4 × 0.9 × 3.9 | ground | 0.04 | 11 |
| `base/vfx/cinematic-realism/exhaust` | Ion Exhaust | vehicle |  | loops | 0.4 × 1.3 × 0.4 | centre | 0.08 | 14 |
| `base/vfx/cinematic-realism/explosion-air` | Airburst | explosion | fire | 5 | 7.3 × 7.8 × 7.1 | centre | 0.15 | 21 |
| `base/vfx/cinematic-realism/explosion-energy` | Energy Blast | explosion | lightning | 6 | 6.4 × 3.0 × 6.4 | ground | 0.61 | 16 |
| `base/vfx/cinematic-realism/explosion-large` | Large Blast | explosion | fire | 18 | 23.0 × 12.6 × 34.8 | ground | 1.03 | 21 |
| `base/vfx/cinematic-realism/explosion-small` | Grenade Blast | explosion | fire | 12 | 5.8 × 2.2 × 8.1 | ground | 1.07 | 20 |
| `base/vfx/cinematic-realism/explosion-water` | Depth Blast | explosion | water | 5 | 13.8 × 9.5 × 13.9 | ground | 0.70 | 24 |
| `base/vfx/cinematic-realism/explosion` | Cinematic Explosion | explosion | fire | 14 | 5.9 × 4.7 × 8.1 | ground | 0.99 | 21 |
| `base/vfx/cinematic-realism/fire-large` | Large Fire | fire | fire | loops | 3.1 × 5.9 × 2.9 | ground | 1.02 | 24 |
| `base/vfx/cinematic-realism/fireflies` | Fireflies | ambient | nature | loops | 3.0 × 0.7 × 2.9 | ground | 0.04 | 11 |
| `base/vfx/cinematic-realism/fireworks` | Firework | ui | fire | 3.2 | 12.1 × 14.8 × 11.4 | ground | 0.53 | 15 |
| `base/vfx/cinematic-realism/flamethrower` | Flame Stream | fire | fire | loops | 1.8 × 5.8 × 1.7 | ground | 0.23 | 17 |
| `base/vfx/cinematic-realism/flashbang` | Flash Charge | explosion |  | 2.5 | 4.6 × 4.2 × 3.8 | centre | 0.15 | 16 |
| `base/vfx/cinematic-realism/footstep-dust` | Footstep Dust | movement | earth | 1 | 0.5 × 0.3 × 0.6 | ground | 0.14 | 14 |
| `base/vfx/cinematic-realism/footstep-water` | Wading Step | splash | water | 1 | 0.5 × 0.2 × 0.6 | ground | 0.14 | 18 |
| `base/vfx/cinematic-realism/fountain` | Fountain Jet | splash | water | loops | 1.0 × 2.2 × 0.9 | ground | 0.14 | 16 |
| `base/vfx/cinematic-realism/frozen` | Frozen | status | ice | loops | 1.4 × 1.6 × 1.4 | ground | 0.16 | 16 |
| `base/vfx/cinematic-realism/grenade-trail` | Grenade Smoke Thread | projectile |  | loops | 0.4 × 0.4 × 0.4 | centre | 0.31 | 14 |
| `base/vfx/cinematic-realism/ground-fog` | Ground Fog | ambient |  | loops | 6.6 × 2.1 × 0.0 | ground | 0.22 | 14 |
| `base/vfx/cinematic-realism/heal` | Mending Light | heal | nature | 2 | 1.5 × 1.7 × 1.5 | ground | 0.08 | 14 |
| `base/vfx/cinematic-realism/impact-concrete` | Concrete Impact | hit-spark | earth | 10 | 1.0 × 0.6 × 1.9 | ground | 0.68 | 16 |
| `base/vfx/cinematic-realism/impact-dirt` | Earth Impact | hit-spark | earth | 10 | 1.3 × 1.2 × 1.0 | ground | 0.71 | 15 |
| `base/vfx/cinematic-realism/impact-flesh` | Cloth Impact | hit-spark |  | 1.2 | 0.7 × 0.5 × 0.6 | ground | 0.14 | 15 |
| `base/vfx/cinematic-realism/impact-glass` | Glass Impact | hit-spark |  | 10 | 2.3 × 0.9 × 4.7 | ground | 0.71 | 13 |
| `base/vfx/cinematic-realism/impact-water` | Water Impact | hit-spark | water | 1.2 | 0.2 × 0.4 × 0.4 | ground | 0.14 | 18 |
| `base/vfx/cinematic-realism/impact-wood` | Timber Impact | hit-spark | nature | 10 | 1.1 × 0.5 × 2.5 | ground | 0.60 | 15 |
| `base/vfx/cinematic-realism/impact` | Steel Impact | hit-spark |  | 10 | 1.6 × 0.8 × 2.2 | ground | 0.69 | 18 |
| `base/vfx/cinematic-realism/landing` | Landing Dust | movement | earth | 1.4 | 1.3 × 1.0 × 1.7 | ground | 0.15 | 15 |
| `base/vfx/cinematic-realism/laser` | Laser Beam | beam | lightning | loops | 0.1 × 10.0 × 0.1 | ground | 0.04 | 13 |
| `base/vfx/cinematic-realism/leaves` | Falling Leaves | ambient | nature | loops | 4.0 × 0.9 × 4.0 | ground | 0.14 | 14 |
| `base/vfx/cinematic-realism/level-up` | Level Up | level-up |  | 2.2 | 3.1 × 5.9 × 3.1 | ground | 0.08 | 17 |
| `base/vfx/cinematic-realism/loot-glow` | Rare Drop Glow | marker |  | loops | 0.5 × 1.4 × 0.5 | ground | 0.06 | 14 |
| `base/vfx/cinematic-realism/magic-bolt` | Arcane Bolt | magic-projectile | arcane | loops | 0.5 × 0.5 × 0.5 | centre | 0.09 | 14 |
| `base/vfx/cinematic-realism/magic-circle` | Drawn Circle | magic-impact | arcane | 4 | 2.0 × 2.4 × 2.0 | ground | 0.97 | 13 |
| `base/vfx/cinematic-realism/magic` | Aether Wisps | magic-impact |  | 1.8 | 1.2 × 1.0 × 1.2 | centre | 0.09 | 14 |
| `base/vfx/cinematic-realism/mark-bullet` | Bullet Hole | decal |  | 30 | 0.1 × 0.0 × 0.1 | ground | 0.03 | 10 |
| `base/vfx/cinematic-realism/mark-burn` | Burn Mark | decal | fire | 30 | 1.0 × 0.0 × 0.9 | ground | 0.05 | 10 |
| `base/vfx/cinematic-realism/mark-wet` | Wet Patch | decal | water | 30 | 0.6 × 0.0 × 0.5 | ground | 0.57 | 10 |
| `base/vfx/cinematic-realism/muzzle-auto` | Automatic Fire | gunfire | fire | loops | 0.2 × 0.2 × 0.2 | ground | 0.17 | 17 |
| `base/vfx/cinematic-realism/muzzle-heavy` | Heavy Muzzle Blast | gunfire | fire | 2.2 | 0.9 × 1.7 × 1.1 | ground | 0.17 | 17 |
| `base/vfx/cinematic-realism/muzzle-suppressed` | Suppressed Shot | gunfire |  | 1.2 | 0.2 × 0.7 × 0.2 | ground | 0.14 | 16 |
| `base/vfx/cinematic-realism/muzzle` | Powder Flash | gunfire | fire | 1.4 | 0.4 × 0.5 × 0.4 | ground | 0.15 | 17 |
| `base/vfx/cinematic-realism/pickup` | Amber Motes | buff |  | 1.2 | 0.8 × 0.8 × 0.7 | ground | 0.08 | 14 |
| `base/vfx/cinematic-realism/poison` | Poisoned | status | nature | loops | 1.1 × 2.0 × 1.1 | ground | 0.17 | 16 |
| `base/vfx/cinematic-realism/portal` | Portal | portal | arcane | loops | 2.4 × 2.4 × 2.4 | ground | 0.15 | 17 |
| `base/vfx/cinematic-realism/reward-burst` | Reward Fountain | ui |  | 2.2 | 2.7 × 1.3 × 3.5 | ground | 0.07 | 15 |
| `base/vfx/cinematic-realism/rocket` | Rocket Motor | projectile | fire | loops | 1.3 × 2.0 × 1.2 | centre | 1.09 | 17 |
| `base/vfx/cinematic-realism/shell` | Spent Casing | gunfire |  | 2.2 | 0.7 × 0.2 × 0.1 | ground | 0.14 | 15 |
| `base/vfx/cinematic-realism/shield-hit` | Shield Ripple | shield | lightning | 0.8 | 1.3 × 0.9 × 1.3 | centre | 0.08 | 15 |
| `base/vfx/cinematic-realism/shocked` | Shocked | status | lightning | loops | 0.6 × 0.9 × 0.6 | ground | 0.04 | 12 |
| `base/vfx/cinematic-realism/skid` | Skid Mark | decal |  | 20 | 1.4 × 0.0 × 0.2 | ground | 0.53 | 10 |
| `base/vfx/cinematic-realism/slash` | Blade Arc | slash |  | 0.4 | 0.9 × 0.5 × 1.3 | centre | 0.11 | 13 |
| `base/vfx/cinematic-realism/smoke-grenade` | Smoke Screen | smoke |  | 18 | 2.6 × 2.5 × 2.7 | ground | 1.09 | 16 |
| `base/vfx/cinematic-realism/smoke-plume` | Wreck Smoke Column | smoke |  | loops | 3.0 × 3.3 × 3.2 | centre | 0.16 | 14 |
| `base/vfx/cinematic-realism/splash-small` | Small Splash | splash | water | 1.2 | 0.5 × 0.4 × 0.7 | ground | 0.14 | 22 |
| `base/vfx/cinematic-realism/splash` | Ballistic Spray | splash | water | 1.6 | 1.8 × 0.7 × 1.9 | ground | 0.15 | 26 |
| `base/vfx/cinematic-realism/steam` | Steam Leak | smoke | water | loops | 0.9 × 2.3 × 0.9 | ground | 0.18 | 14 |
| `base/vfx/cinematic-realism/sunbeam-dust` | Lit Dust | ambient |  | loops | 2.0 × 2.9 × 2.0 | ground | 0.04 | 11 |
| `base/vfx/cinematic-realism/teleport` | Arrival Flash | portal |  | 1.4 | 1.8 × 2.9 × 1.8 | ground | 0.12 | 16 |
| `base/vfx/cinematic-realism/thruster` | Attitude Jet | vehicle |  | 0.6 | 0.3 × 1.3 × 0.4 | ground | 0.04 | 13 |
| `base/vfx/cinematic-realism/tire-smoke` | Tyre Smoke | vehicle |  | loops | 1.7 × 1.4 × 1.6 | ground | 0.24 | 14 |
| `base/vfx/cinematic-realism/tracer` | Tracer Round | projectile | fire | loops | 0.1 × 2.9 × 0.1 | centre | 0.04 | 13 |
| `base/vfx/cinematic-realism/wake` | Hull Wake | vehicle | water | loops | 2.3 × 0.8 × 2.0 | ground | 0.19 | 18 |
| `base/vfx/cinematic-realism/warp` | Jump Flash | vehicle |  | 1.6 | 1.7 × 27.9 × 1.8 | centre | 0.09 | 15 |
| `base/vfx/cinematic-realism/waterfall-mist` | Waterfall Mist | splash | water | loops | 4.0 × 2.2 × 4.0 | ground | 0.19 | 18 |
| `base/vfx/cinematic-realism/wheel-dust` | Wheel Dust | vehicle | earth | loops | 1.7 × 1.3 × 2.1 | ground | 0.24 | 15 |

<a id="guide-effect-packs-9"></a>
### `effect-packs §9` effekseer-andrewfm01

AndrewFM01's Effekseer sample effects (15), converted to file.vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-andrewfm01`.

- Style: stylised, 3d
- Categories: beam, explosion, fire, hit-spark, magic-impact, magic-projectile, portal, smoke, ui
- Elements: fire, lightning
- Keywords: 3d, andrewfm01, anime, effekseer, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-andrewfm01/actionLines_halo` | Action Lines Halo | ui |  | 1.673 |  |  | 1.31 | 18 |
| `base/vfx/effekseer-andrewfm01/actionLines_halo2` | Action Lines Halo 2 | ui |  | 1.683 |  |  | 1.83 | 17 |
| `base/vfx/effekseer-andrewfm01/actionLines_radial` | Action Lines Radial | ui |  | 1.673 |  |  | 4.18 | 20 |
| `base/vfx/effekseer-andrewfm01/actionLines_radial2` | Action Lines Radial 2 | ui |  | 1.7 |  |  | 6.79 | 20 |
| `base/vfx/effekseer-andrewfm01/actionLines_shockwave` | Action Lines Shockwave | ui |  | 1.667 |  |  | 0.15 | 20 |
| `base/vfx/effekseer-andrewfm01/actionLines_strip` | Action Lines Strip | ui |  | 1.667 |  |  | 0.08 | 19 |
| `base/vfx/effekseer-andrewfm01/blue_laser` | Blue Laser | beam |  | loops |  |  | 0.34 | 18 |
| `base/vfx/effekseer-andrewfm01/boss_death` | Boss Death | explosion |  | loops |  |  | 0.30 | 17 |
| `base/vfx/effekseer-andrewfm01/drill` | Drill | magic-projectile |  | loops |  |  | 2.39 | 24 |
| `base/vfx/effekseer-andrewfm01/electric_dust` | Electric Dust | smoke | lightning | 3.4 |  |  | 0.32 | 16 |
| `base/vfx/effekseer-andrewfm01/fire` | Fire | fire | fire | 1 |  |  | 1.48 | 24 |
| `base/vfx/effekseer-andrewfm01/hit_eff` | Hit Eff | hit-spark |  | 0.622 |  |  | 0.42 | 24 |
| `base/vfx/effekseer-andrewfm01/magic_circle` | Magic Circle | portal |  | 16.683 |  |  | 0.47 | 26 |
| `base/vfx/effekseer-andrewfm01/smallTexturesRibbon` | Small Textures Ribbon | magic-impact |  | 1.017 |  |  | 0.09 | 20 |
| `base/vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital` | Small Textures Square Wave Orbital | magic-impact |  | 66.667 |  |  | 0.08 | 17 |

<a id="guide-effect-packs-10"></a>
### `effect-packs §10` effekseer-effekseer01

Effekseer01's Effekseer sample effects (15), converted to file.vfx. 15 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-effekseer01`.

- Style: stylised, 3d
- Categories: basic, beam
- Elements: 
- Keywords: 3d, anime, effekseer, effekseer01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-effekseer01/Homing_Laser01` | Homing Laser 01 | beam |  | 4.267 |  |  | 0.14 | 17 |
| `base/vfx/effekseer-effekseer01/Laser01` | Laser 01 | beam |  | 1.333 |  |  | 1.12 | 19 |
| `base/vfx/effekseer-effekseer01/Laser02` | Laser 02 | beam |  | 3.733 |  |  | 1.73 | 19 |
| `base/vfx/effekseer-effekseer01/Laser03` | Laser 03 | beam |  | loops |  |  | 0.69 | 20 |
| `base/vfx/effekseer-effekseer01/Simple_Distortion` | Simple Distortion | basic |  | 2.917 |  |  | 0.44 | 24 |
| `base/vfx/effekseer-effekseer01/Simple_GeneratingPosition1` | Simple Generating Position 1 | basic |  | 0.168 |  |  | 0.30 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Model_UV` | Simple Model UV | basic |  | 3.5 |  |  | 0.38 | 16 |
| `base/vfx/effekseer-effekseer01/Simple_Ribbon_Parent` | Simple Ribbon Parent | basic |  | 1.817 |  |  | 0.13 | 16 |
| `base/vfx/effekseer-effekseer01/Simple_Ribbon_Sword` | Simple Ribbon Sword | basic |  | 0.4 |  |  | 0.04 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Ring_Shape1` | Simple Ring Shape 1 | basic |  | 9.5 |  |  | 0.14 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Ring_Shape2` | Simple Ring Shape 2 | basic |  | 7 |  |  | 0.24 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Sprite_BillBoard` | Simple Sprite Bill Board | basic |  | 0.665 |  |  | 1.17 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Sprite_Fixed` | Simple Sprite Fixed | basic |  | 1.633 |  |  | 0.81 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis` | Simple Sprite Fixed YAxis | basic |  | 2.817 |  |  | 1.07 | 15 |
| `base/vfx/effekseer-effekseer01/Simple_Track1` | Simple Track 1 | basic |  | 3.333 |  |  | 0.51 | 15 |

<a id="guide-effect-packs-11"></a>
### `effect-packs §11` effekseer-hato01

HATO01's Effekseer sample effects (4), converted to file.vfx. 4 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-hato01`.

- Style: stylised, 3d
- Categories: ambient
- Elements: nature
- Keywords: 3d, anime, effekseer, hato01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-hato01/aoi-chou` | Aoi Chou | ambient | nature | loops |  |  | 0.31 | 20 |
| `base/vfx/effekseer-hato01/iwai-ume` | Iwai Ume | ambient | nature | 20.3 |  |  | 0.45 | 18 |
| `base/vfx/effekseer-hato01/kiroi-chou` | Kiroi Chou | ambient | nature | 33.375 |  |  | 0.37 | 22 |
| `base/vfx/effekseer-hato01/sakura1` | Sakura 1 | ambient | nature | 13.8 |  |  | 1.55 | 52 |

<a id="guide-effect-packs-12"></a>
### `effect-packs §12` effekseer-magicalxspiral

MAGICAL X SPIRAL's Effekseer sample effects (101), converted to file.vfx. 101 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-magicalxspiral`.

- Style: stylised, 3d
- Categories: explosion, fire, magic-impact, magic-projectile, portal, shield, slash, smoke, weather
- Elements: earth, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, magicalxspiral, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-magicalxspiral/A_Gohlem1` | A Gohlem 1 | magic-impact | earth | 0.983 |  |  | 0.34 | 30 |
| `base/vfx/effekseer-magicalxspiral/A_Salamander1` | A Salamander 1 | magic-impact | fire | 0.983 |  |  | 0.19 | 19 |
| `base/vfx/effekseer-magicalxspiral/A_Salamander2` | A Salamander 2 | magic-impact | fire | 0.767 |  |  | 0.45 | 28 |
| `base/vfx/effekseer-magicalxspiral/A_Salamander4` | A Salamander 4 | magic-impact | fire | 1.167 |  |  | 0.60 | 24 |
| `base/vfx/effekseer-magicalxspiral/A_Sylph1` | A Sylph 1 | magic-impact | wind | 0.983 |  |  | 0.15 | 20 |
| `base/vfx/effekseer-magicalxspiral/A_Sylph2` | A Sylph 2 | magic-impact | wind | 0.575 |  |  | 0.26 | 17 |
| `base/vfx/effekseer-magicalxspiral/A_Undine1` | A Undine 1 | magic-impact | water | 0.983 |  |  | 1.13 | 19 |
| `base/vfx/effekseer-magicalxspiral/A_Undine2` | A Undine 2 | magic-impact | water | 0.567 |  |  | 0.45 | 26 |
| `base/vfx/effekseer-magicalxspiral/AquaPoint` | Aqua Point | magic-impact | water | 2.317 |  |  | 0.27 | 21 |
| `base/vfx/effekseer-magicalxspiral/Attack1` | Attack 1 | slash |  | 0.983 |  |  | 0.13 | 18 |
| `base/vfx/effekseer-magicalxspiral/Attack2` | Attack 2 | slash |  | 0.983 |  |  | 0.14 | 18 |
| `base/vfx/effekseer-magicalxspiral/Attack5` | Attack 5 | slash |  | 0.9 |  |  | 0.12 | 18 |
| `base/vfx/effekseer-magicalxspiral/Attack6` | Attack 6 | slash |  | 0.9 |  |  | 0.10 | 18 |
| `base/vfx/effekseer-magicalxspiral/Attack_Impact` | Attack Impact | magic-impact |  | 0.567 |  |  | 0.28 | 23 |
| `base/vfx/effekseer-magicalxspiral/Breakdown` | Breakdown | explosion |  | 0.517 |  |  | 0.37 | 23 |
| `base/vfx/effekseer-magicalxspiral/FirePot1` | Fire Pot 1 | fire | fire | 1 |  |  | 0.23 | 20 |
| `base/vfx/effekseer-magicalxspiral/FirePot2` | Fire Pot 2 | fire | fire | 1 |  |  | 0.17 | 20 |
| `base/vfx/effekseer-magicalxspiral/Gohlem1` | Gohlem 1 | magic-impact | earth | 0.817 |  |  | 0.37 | 27 |
| `base/vfx/effekseer-magicalxspiral/Gohlem11` | Gohlem 11 | magic-impact | earth | 0.858 |  |  | 0.34 | 17 |
| `base/vfx/effekseer-magicalxspiral/Gohlem5` | Gohlem 5 | magic-impact | earth | 0.983 |  |  | 1.43 | 34 |
| `base/vfx/effekseer-magicalxspiral/Gohlem7` | Gohlem 7 | magic-impact | earth | 2.917 |  |  | 0.30 | 24 |
| `base/vfx/effekseer-magicalxspiral/Gohlem_Shield1` | Gohlem Shield 1 | shield | earth | 0.883 |  |  | 0.29 | 20 |
| `base/vfx/effekseer-magicalxspiral/Gohlem_Shield2` | Gohlem Shield 2 | shield | earth | 1.833 |  |  | 0.14 | 15 |
| `base/vfx/effekseer-magicalxspiral/Holy1` | Holy 1 | magic-impact | holy | 0.45 |  |  | 0.17 | 18 |
| `base/vfx/effekseer-magicalxspiral/Holy2` | Holy 2 | magic-impact | holy | 0.583 |  |  | 0.12 | 18 |
| `base/vfx/effekseer-magicalxspiral/Icicle1` | Icicle 1 | magic-projectile | ice | 0.4 |  |  | 0.20 | 16 |
| `base/vfx/effekseer-magicalxspiral/Icicle2` | Icicle 2 | magic-projectile | ice | 0.167 |  |  | 0.08 | 19 |
| `base/vfx/effekseer-magicalxspiral/Icicle3` | Icicle 3 | magic-projectile | ice | 0.167 |  |  | 0.24 | 19 |
| `base/vfx/effekseer-magicalxspiral/IcicleBroken` | Icicle Broken | explosion | ice | 0.517 |  |  | 0.43 | 27 |
| `base/vfx/effekseer-magicalxspiral/IcicleLocus` | Icicle Locus | magic-projectile | ice | 0.4 |  |  | 0.15 | 16 |
| `base/vfx/effekseer-magicalxspiral/Impact` | Impact | magic-impact |  | 0.7 |  |  | 0.24 | 25 |
| `base/vfx/effekseer-magicalxspiral/Lance3` | Lance 3 | magic-projectile |  | 2.15 |  |  | 0.56 | 19 |
| `base/vfx/effekseer-magicalxspiral/Lance4` | Lance 4 | magic-projectile |  | 1.15 |  |  | 0.44 | 26 |
| `base/vfx/effekseer-magicalxspiral/Lance5` | Lance 5 | magic-projectile |  | 1.15 |  |  | 0.48 | 30 |
| `base/vfx/effekseer-magicalxspiral/Magic1_earth` | Magic 1 Earth | magic-impact | earth | 1.667 |  |  | 0.54 | 21 |
| `base/vfx/effekseer-magicalxspiral/Magic1_fire` | Magic 1 Fire | fire | fire | 1.667 |  |  | 0.56 | 21 |
| `base/vfx/effekseer-magicalxspiral/Magic1_holy` | Magic 1 Holy | magic-impact | holy | 1.667 |  |  | 0.60 | 21 |
| `base/vfx/effekseer-magicalxspiral/Magic1_water` | Magic 1 Water | magic-impact | water | 1.667 |  |  | 0.61 | 21 |
| `base/vfx/effekseer-magicalxspiral/Magic1_wind` | Magic 1 Wind | magic-impact | wind | 1.667 |  |  | 0.54 | 21 |
| `base/vfx/effekseer-magicalxspiral/Magic2_earth` | Magic 2 Earth | magic-impact | earth | 0.483 |  |  | 0.10 | 16 |
| `base/vfx/effekseer-magicalxspiral/Magic2_fire` | Magic 2 Fire | fire | fire | 0.483 |  |  | 0.18 | 16 |
| `base/vfx/effekseer-magicalxspiral/Magic2_holy` | Magic 2 Holy | magic-impact | holy | 0.483 |  |  | 0.19 | 16 |
| `base/vfx/effekseer-magicalxspiral/Magic2_water` | Magic 2 Water | magic-impact | water | 0.483 |  |  | 0.12 | 16 |
| `base/vfx/effekseer-magicalxspiral/Magic2_wind` | Magic 2 Wind | magic-impact | wind | 0.483 |  |  | 0.12 | 16 |
| `base/vfx/effekseer-magicalxspiral/MagicArea` | Magic Area | portal |  | 2.917 |  |  | 0.39 | 22 |
| `base/vfx/effekseer-magicalxspiral/Rock1` | Rock 1 | magic-impact | earth | 1.667 |  |  | 0.33 | 18 |
| `base/vfx/effekseer-magicalxspiral/Rock2` | Rock 2 | magic-impact | earth | 0.817 |  |  | 0.28 | 27 |
| `base/vfx/effekseer-magicalxspiral/Salamander1` | Salamander 1 | magic-impact | fire | 0.45 |  |  | 0.16 | 18 |
| `base/vfx/effekseer-magicalxspiral/Salamander11` | Salamander 11 | magic-impact | fire | 1.317 |  |  | 1.18 | 39 |
| `base/vfx/effekseer-magicalxspiral/Salamander12` | Salamander 12 | magic-impact | fire | 0.883 |  |  | 0.97 | 28 |
| `base/vfx/effekseer-magicalxspiral/Salamander14` | Salamander 14 | magic-impact | fire | 0.833 |  |  | 0.32 | 19 |
| `base/vfx/effekseer-magicalxspiral/Salamander15` | Salamander 15 | magic-impact | fire | 2.7 |  |  | 0.18 | 26 |
| `base/vfx/effekseer-magicalxspiral/Salamander2` | Salamander 2 | magic-impact | fire | 0.583 |  |  | 0.07 | 18 |
| `base/vfx/effekseer-magicalxspiral/Salamander3` | Salamander 3 | magic-impact | fire | 1.317 |  |  | 0.72 | 30 |
| `base/vfx/effekseer-magicalxspiral/Salamander4` | Salamander 4 | magic-impact | fire | 1.317 |  |  | 0.58 | 30 |
| `base/vfx/effekseer-magicalxspiral/Salamander5` | Salamander 5 | magic-impact | fire | 1.317 |  |  | 0.95 | 30 |
| `base/vfx/effekseer-magicalxspiral/Salamander6` | Salamander 6 | magic-impact | fire | 1.233 |  |  | 0.44 | 22 |
| `base/vfx/effekseer-magicalxspiral/Salamander7` | Salamander 7 | magic-impact | fire | 0.717 |  |  | 0.32 | 18 |
| `base/vfx/effekseer-magicalxspiral/Snowman1` | Snowman 1 | weather | ice | 0.417 |  |  | 0.04 | 16 |
| `base/vfx/effekseer-magicalxspiral/StairBroken` | Stair Broken | explosion | earth | 0.633 |  |  | 0.36 | 27 |
| `base/vfx/effekseer-magicalxspiral/StairBroken3` | Stair Broken 3 | explosion | earth | 1.317 |  |  | 0.27 | 15 |
| `base/vfx/effekseer-magicalxspiral/Sylph1` | Sylph 1 | magic-impact | wind | 0.383 |  |  | 0.07 | 18 |
| `base/vfx/effekseer-magicalxspiral/Sylph10` | Sylph 10 | magic-impact | wind | 1.55 |  |  | 0.22 | 21 |
| `base/vfx/effekseer-magicalxspiral/Sylph11` | Sylph 11 | magic-impact | wind | 0.883 |  |  | 0.62 | 25 |
| `base/vfx/effekseer-magicalxspiral/Sylph13` | Sylph 13 | magic-impact | wind | 2.517 |  |  | 0.29 | 23 |
| `base/vfx/effekseer-magicalxspiral/Sylph15` | Sylph 15 | magic-impact | wind | 0.817 |  |  | 0.74 | 27 |
| `base/vfx/effekseer-magicalxspiral/Sylph2` | Sylph 2 | magic-impact | wind | 0.575 |  |  | 0.25 | 17 |
| `base/vfx/effekseer-magicalxspiral/Sylph3` | Sylph 3 | magic-impact | wind | 0.45 |  |  | 0.16 | 18 |
| `base/vfx/effekseer-magicalxspiral/Sylph4` | Sylph 4 | magic-impact | wind | 0.583 |  |  | 0.09 | 18 |
| `base/vfx/effekseer-magicalxspiral/Sylph5` | Sylph 5 | magic-impact | wind | 2.7 |  |  | 0.17 | 22 |
| `base/vfx/effekseer-magicalxspiral/Sylph6` | Sylph 6 | magic-impact | wind | 0.367 |  |  | 0.02 | 15 |
| `base/vfx/effekseer-magicalxspiral/Sylph7` | Sylph 7 | magic-impact | wind | 1.667 |  |  | 1.00 | 26 |
| `base/vfx/effekseer-magicalxspiral/Sylph8` | Sylph 8 | magic-impact | wind | 1.167 |  |  | 0.13 | 20 |
| `base/vfx/effekseer-magicalxspiral/Thunder1` | Thunder 1 | magic-impact | lightning | 2.15 |  |  | 0.51 | 19 |
| `base/vfx/effekseer-magicalxspiral/Thunder2` | Thunder 2 | magic-impact | lightning | 0.817 |  |  | 0.65 | 30 |
| `base/vfx/effekseer-magicalxspiral/Thunder3` | Thunder 3 | magic-impact | lightning | 0.817 |  |  | 0.32 | 17 |
| `base/vfx/effekseer-magicalxspiral/Topwater1` | Topwater 1 | magic-impact | water | 0.65 |  |  | 0.49 | 17 |
| `base/vfx/effekseer-magicalxspiral/Undine1` | Undine 1 | magic-impact | water | 1 |  |  | 0.62 | 30 |
| `base/vfx/effekseer-magicalxspiral/Undine11` | Undine 11 | magic-impact | water | 10.083 |  |  | 0.04 | 17 |
| `base/vfx/effekseer-magicalxspiral/Undine13` | Undine 13 | magic-impact | water | 3.917 |  |  | 0.14 | 18 |
| `base/vfx/effekseer-magicalxspiral/Undine2` | Undine 2 | magic-impact | water | 0.45 |  |  | 0.24 | 18 |
| `base/vfx/effekseer-magicalxspiral/Undine3` | Undine 3 | magic-impact | water | 0.583 |  |  | 0.09 | 18 |
| `base/vfx/effekseer-magicalxspiral/Undine4` | Undine 4 | magic-impact | water | 1.667 |  |  | 0.51 | 18 |
| `base/vfx/effekseer-magicalxspiral/Undine5` | Undine 5 | magic-impact | water | 0.667 |  |  | 0.59 | 26 |
| `base/vfx/effekseer-magicalxspiral/Undine6` | Undine 6 | magic-impact | water | 1.833 |  |  | 0.32 | 17 |
| `base/vfx/effekseer-magicalxspiral/Undine8` | Undine 8 | magic-impact | water | 1.667 |  |  | 1.15 | 26 |
| `base/vfx/effekseer-magicalxspiral/WHead1` | WHead 1 | magic-impact | water | 0.417 |  |  | 0.07 | 16 |
| `base/vfx/effekseer-magicalxspiral/WHead2` | WHead 2 | magic-impact | water | 0.567 |  |  | 0.19 | 16 |
| `base/vfx/effekseer-magicalxspiral/WHead3` | WHead 3 | magic-impact | water | 0.567 |  |  | 0.29 | 25 |
| `base/vfx/effekseer-magicalxspiral/Water_Impact` | Water Impact | magic-impact | water | 0.65 |  |  | 0.32 | 25 |
| `base/vfx/effekseer-magicalxspiral/Wind_Impact` | Wind Impact | magic-impact | wind | 0.4 |  |  | 0.19 | 21 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke1` | Foot Smoke 1 | smoke |  | 0.25 |  |  | 0.15 | 17 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke2` | Foot Smoke 2 | smoke |  | 1.233 |  |  | 0.19 | 16 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke3` | Foot Smoke 3 | smoke |  | 0.9 |  |  | 0.17 | 15 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke4` | Foot Smoke 4 | smoke |  | 0.9 |  |  | 0.15 | 15 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke5` | Foot Smoke 5 | smoke |  | 0.9 |  |  | 0.57 | 15 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke6` | Foot Smoke 6 | smoke |  | 0.567 |  |  | 0.18 | 15 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke7` | Foot Smoke 7 | smoke |  | 0.9 |  |  | 0.10 | 15 |
| `base/vfx/effekseer-magicalxspiral/foot_smoke8` | Foot Smoke 8 | smoke |  | 0.9 |  |  | 0.45 | 15 |
| `base/vfx/effekseer-magicalxspiral/meteo1` | Meteo 1 | magic-projectile | fire | 0.858 |  |  | 0.54 | 23 |
| `base/vfx/effekseer-magicalxspiral/salamander13` | Salamander 13 | magic-impact | fire | 1.683 |  |  | 0.34 | 24 |

<a id="guide-effect-packs-13"></a>
### `effect-packs §13` effekseer-nextsoft01

NextSoft01's Effekseer sample effects (20), converted to file.vfx. 20 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-nextsoft01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, magic-impact, magic-projectile, shield, slash, weather
- Elements: dark, fire, holy, ice, lightning, water, wind
- Keywords: 3d, anime, effekseer, nextsoft01, particles

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-nextsoft01/HealPotion1` | Heal Potion 1 | heal | holy | 1.267 |  |  | 0.23 | 17 |
| `base/vfx/effekseer-nextsoft01/HealPotion2` | Heal Potion 2 | heal | holy | 1.63 |  |  | 0.50 | 22 |
| `base/vfx/effekseer-nextsoft01/MagicCold` | Magic Cold | magic-impact | ice | 2.15 |  |  | 0.40 | 17 |
| `base/vfx/effekseer-nextsoft01/MagicDark` | Magic Dark | magic-impact | dark | 2.05 |  |  | 0.97 | 19 |
| `base/vfx/effekseer-nextsoft01/MagicFire1` | Magic Fire 1 | fire | fire | 2.329 |  |  | 0.91 | 15 |
| `base/vfx/effekseer-nextsoft01/MagicFire2` | Magic Fire 2 | fire | fire | 1.91 |  |  | 0.29 | 15 |
| `base/vfx/effekseer-nextsoft01/MagicHeal1` | Magic Heal 1 | heal | holy | 1.967 |  |  | 0.73 | 17 |
| `base/vfx/effekseer-nextsoft01/MagicHeal2` | Magic Heal 2 | heal | holy | 4 |  |  | 0.96 | 19 |
| `base/vfx/effekseer-nextsoft01/MagicLight` | Magic Light | magic-impact | holy | 1.317 |  |  | 0.38 | 18 |
| `base/vfx/effekseer-nextsoft01/MagicMeteo` | Magic Meteo | magic-projectile | fire | 2.417 |  |  | 0.52 | 21 |
| `base/vfx/effekseer-nextsoft01/MagicShield` | Magic Shield | shield |  | 0.183 |  |  | 0.04 | 21 |
| `base/vfx/effekseer-nextsoft01/MagicSword` | Magic Sword | slash |  | 1.733 |  |  | 0.59 | 22 |
| `base/vfx/effekseer-nextsoft01/MagicThunder` | Magic Thunder | magic-impact | lightning | 1.05 |  |  | 1.28 | 25 |
| `base/vfx/effekseer-nextsoft01/MagicTornade` | Magic Tornade | weather | wind | 0.817 |  |  | 0.47 | 15 |
| `base/vfx/effekseer-nextsoft01/MagicWater` | Magic Water | magic-impact | water | 2.09 |  |  | 2.17 | 23 |
| `base/vfx/effekseer-nextsoft01/PowerUp` | Power Up | buff |  | 1.75 |  |  | 0.30 | 18 |
| `base/vfx/effekseer-nextsoft01/Spear1` | Spear 1 | magic-projectile |  | 0.367 |  |  | 0.55 | 17 |
| `base/vfx/effekseer-nextsoft01/Spear2` | Spear 2 | magic-projectile |  | 0.983 |  |  | 0.23 | 16 |
| `base/vfx/effekseer-nextsoft01/Sword1` | Sword 1 | slash |  | 0.333 |  |  | 0.19 | 17 |
| `base/vfx/effekseer-nextsoft01/Sword2` | Sword 2 | slash |  | 1.1 |  |  | 1.33 | 19 |

<a id="guide-effect-packs-14"></a>
### `effect-packs §14` effekseer-pierre01

Pierre vol.1's Effekseer sample effects (11), converted to file.vfx. 11 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre01`.

- Style: stylised, 3d
- Categories: ambient, buff, fire, magic-impact, magic-projectile, weather
- Elements: fire, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-pierre01/Atmosphere` | Atmosphere | ambient |  | loops |  |  | 0.81 | 27 |
| `base/vfx/effekseer-pierre01/Background` | Background | ambient |  | 19.333 |  |  | 0.33 | 24 |
| `base/vfx/effekseer-pierre01/Flame` | Flame | fire | fire | 5.283 |  |  | 6.39 | 62 |
| `base/vfx/effekseer-pierre01/Herald` | Herald | buff |  | 2.083 |  |  | 1.81 | 81 |
| `base/vfx/effekseer-pierre01/HolySandstorm` | Holy Sandstorm | weather | wind | 6.733 |  |  | 7.59 | 27 |
| `base/vfx/effekseer-pierre01/LightningStrike` | Lightning Strike | magic-impact | lightning | 2.783 |  |  | 6.37 | 55 |
| `base/vfx/effekseer-pierre01/MissileBoost` | Missile Boost | magic-projectile |  | loops |  |  | 5.82 | 26 |
| `base/vfx/effekseer-pierre01/MonsterRoar` | Monster Roar | magic-impact |  | 2.417 |  |  | 1.99 | 35 |
| `base/vfx/effekseer-pierre01/PhantasmMeteor` | Phantasm Meteor | magic-projectile | fire | 4.15 |  |  | 8.23 | 62 |
| `base/vfx/effekseer-pierre01/PhantasmMeteor_Single` | Phantasm Meteor Single | magic-projectile | fire | 3.612 |  |  | 5.55 | 46 |
| `base/vfx/effekseer-pierre01/SonicBoom` | Sonic Boom | magic-projectile |  | 1.833 |  |  | 2.31 | 31 |

<a id="guide-effect-packs-15"></a>
### `effect-packs §15` effekseer-pierre02

Pierre vol.2's Effekseer sample effects (12), converted to file.vfx. 12 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-pierre02`.

- Style: stylised, 3d
- Categories: explosion, heal, magic-impact, magic-projectile, portal, slash, smoke
- Elements: dark, fire, holy, lightning, wind
- Keywords: 3d, anime, effekseer, particles, pierre02

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-pierre02/Benediction` | Benediction | heal | holy | 5.933 |  |  | 9.71 | 47 |
| `base/vfx/effekseer-pierre02/BloodLance` | Blood Lance | magic-projectile | dark | loops |  |  | 3.91 | 45 |
| `base/vfx/effekseer-pierre02/CosmicMist` | Cosmic Mist | smoke | dark | 7.267 |  |  | 16.08 | 56 |
| `base/vfx/effekseer-pierre02/FeatherBomb` | Feather Bomb | explosion |  | 13.006 |  |  | 12.71 | 55 |
| `base/vfx/effekseer-pierre02/FireBall` | Fire Ball | magic-projectile | fire | 5.517 |  |  | 6.56 | 49 |
| `base/vfx/effekseer-pierre02/GateOfCalve` | Gate Of Calve | portal |  | 14.083 |  |  | 2.62 | 35 |
| `base/vfx/effekseer-pierre02/LossOfBlood` | Loss Of Blood | magic-impact | dark | 1.85 |  |  | 36.30 | 47 |
| `base/vfx/effekseer-pierre02/Sword_Ember` | Sword Ember | slash | fire | loops |  |  | 1.04 | 26 |
| `base/vfx/effekseer-pierre02/Sword_Lightning` | Sword Lightning | slash | lightning | loops |  |  | 1.47 | 26 |
| `base/vfx/effekseer-pierre02/Sword_Moonlight` | Sword Moonlight | slash | holy | loops |  |  | 1.01 | 26 |
| `base/vfx/effekseer-pierre02/Sword_Sacredlight` | Sword Sacredlight | slash | holy | loops |  |  | 1.84 | 24 |
| `base/vfx/effekseer-pierre02/Sword_Storm` | Sword Storm | slash | wind | loops |  |  | 3.63 | 28 |

<a id="guide-effect-packs-16"></a>
### `effect-packs §16` effekseer-suzuki01

Suzuki01's Effekseer sample effects (3), converted to file.vfx. 3 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-suzuki01`.

- Style: stylised, 3d
- Categories: buff, slash, weather
- Elements: ice
- Keywords: 3d, anime, effekseer, particles, suzuki01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-suzuki01/aura` | Aura | buff |  | loops |  |  | 1.29 | 29 |
| `base/vfx/effekseer-suzuki01/snowstorm11` | Snowstorm 11 | weather | ice | 8.917 |  |  | 4.00 | 24 |
| `base/vfx/effekseer-suzuki01/sword` | Sword | slash |  | loops |  |  | 1.33 | 30 |

<a id="guide-effect-packs-17"></a>
### `effect-packs §17` effekseer-tktk01

tktk vol.1's Effekseer sample effects (71), converted to file.vfx. 71 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk01`.

- Style: stylised, 3d
- Categories: buff, fire, heal, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, earth, fire, holy, wind
- Keywords: 3d, anime, effekseer, particles, tktk01

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-tktk01/Arrow1` | Arrow 1 | magic-projectile |  | 0.454 |  |  | 0.30 | 29 |
| `base/vfx/effekseer-tktk01/Arrow2` | Arrow 2 | magic-projectile |  | 2.092 |  |  | 1.39 | 50 |
| `base/vfx/effekseer-tktk01/Blow1` | Blow 1 | hit-spark |  | 0.775 |  |  | 5.87 | 17 |
| `base/vfx/effekseer-tktk01/Blow10` | Blow 10 | hit-spark |  | 0.867 |  |  | 4.46 | 21 |
| `base/vfx/effekseer-tktk01/Blow11` | Blow 11 | hit-spark |  | 0.65 |  |  | 3.95 | 30 |
| `base/vfx/effekseer-tktk01/Blow12` | Blow 12 | hit-spark |  | 1.816 |  |  | 2.92 | 29 |
| `base/vfx/effekseer-tktk01/Blow13` | Blow 13 | hit-spark |  | 0.832 |  |  | 3.35 | 33 |
| `base/vfx/effekseer-tktk01/Blow14` | Blow 14 | hit-spark |  | 1.4 |  |  | 3.99 | 30 |
| `base/vfx/effekseer-tktk01/Blow15` | Blow 15 | hit-spark |  | 0.908 |  |  | 5.74 | 41 |
| `base/vfx/effekseer-tktk01/Blow2` | Blow 2 | hit-spark |  | 0.533 |  |  | 1.27 | 19 |
| `base/vfx/effekseer-tktk01/Blow3` | Blow 3 | hit-spark |  | 0.583 |  |  | 1.82 | 23 |
| `base/vfx/effekseer-tktk01/Blow4` | Blow 4 | hit-spark |  | 0.845 |  |  | 2.86 | 21 |
| `base/vfx/effekseer-tktk01/Blow5` | Blow 5 | hit-spark |  | 1.775 |  |  | 1.52 | 25 |
| `base/vfx/effekseer-tktk01/Blow6` | Blow 6 | hit-spark |  | 2.367 |  |  | 1.28 | 16 |
| `base/vfx/effekseer-tktk01/Blow7` | Blow 7 | hit-spark |  | 1.678 |  |  | 1.20 | 21 |
| `base/vfx/effekseer-tktk01/Blow8` | Blow 8 | hit-spark |  | 3.017 |  |  | 3.16 | 25 |
| `base/vfx/effekseer-tktk01/Blow9` | Blow 9 | hit-spark |  | 1.667 |  |  | 3.81 | 29 |
| `base/vfx/effekseer-tktk01/Cure1` | Cure 1 | heal | holy | 1.275 |  |  | 1.25 | 24 |
| `base/vfx/effekseer-tktk01/Cure2` | Cure 2 | heal | holy | 1.779 |  |  | 1.55 | 26 |
| `base/vfx/effekseer-tktk01/Cure3` | Cure 3 | heal | holy | 1.252 |  |  | 3.69 | 18 |
| `base/vfx/effekseer-tktk01/Cure4` | Cure 4 | heal | holy | 0.967 |  |  | 0.68 | 16 |
| `base/vfx/effekseer-tktk01/Cure5` | Cure 5 | heal | holy | 1.433 |  |  | 5.35 | 23 |
| `base/vfx/effekseer-tktk01/Cure6` | Cure 6 | heal | holy | 1.251 |  |  | 2.26 | 18 |
| `base/vfx/effekseer-tktk01/Cure7` | Cure 7 | heal | holy | 2.6 |  |  | 1.01 | 40 |
| `base/vfx/effekseer-tktk01/Dark1` | Dark 1 | magic-impact | dark | 1.617 |  |  | 6.79 | 18 |
| `base/vfx/effekseer-tktk01/Dark2` | Dark 2 | magic-impact | dark | 1.7 |  |  | 1.37 | 24 |
| `base/vfx/effekseer-tktk01/Dark3` | Dark 3 | magic-impact | dark | 3.333 |  |  | 2.54 | 20 |
| `base/vfx/effekseer-tktk01/Fire1` | Fire 1 | fire | fire | loops |  |  | 4.50 | 20 |
| `base/vfx/effekseer-tktk01/Fire2` | Fire 2 | fire | fire | 3.013 |  |  | 4.00 | 19 |
| `base/vfx/effekseer-tktk01/Fire3` | Fire 3 | fire | fire | loops |  |  | 3.87 | 23 |
| `base/vfx/effekseer-tktk01/Fire4` | Fire 4 | fire | fire | 2.425 |  |  | 6.32 | 23 |
| `base/vfx/effekseer-tktk01/Fire5` | Fire 5 | fire | fire | 1.442 |  |  | 2.66 | 21 |
| `base/vfx/effekseer-tktk01/Fire6` | Fire 6 | fire | fire | 1.251 |  |  | 3.50 | 31 |
| `base/vfx/effekseer-tktk01/Fire7` | Fire 7 | fire | fire | loops |  |  | 3.31 | 26 |
| `base/vfx/effekseer-tktk01/Fire8` | Fire 8 | fire | fire | 1.925 |  |  | 4.80 | 22 |
| `base/vfx/effekseer-tktk01/Light1` | Light 1 | magic-impact | holy | 2.667 |  |  | 0.89 | 26 |
| `base/vfx/effekseer-tktk01/Light2` | Light 2 | magic-impact | holy | 0.55 |  |  | 3.54 | 33 |
| `base/vfx/effekseer-tktk01/Light3` | Light 3 | magic-impact | holy | 1 |  |  | 0.53 | 20 |
| `base/vfx/effekseer-tktk01/Light4` | Light 4 | magic-impact | holy | 2.092 |  |  | 3.56 | 78 |
| `base/vfx/effekseer-tktk01/Magic1` | Magic 1 | magic-impact |  | 1 |  |  | 4.65 | 29 |
| `base/vfx/effekseer-tktk01/Magic2` | Magic 2 | magic-impact |  | 2.471 |  |  | 4.00 | 34 |
| `base/vfx/effekseer-tktk01/Soil1` | Soil 1 | magic-impact | earth | loops |  |  | 4.50 | 29 |
| `base/vfx/effekseer-tktk01/Sword1` | Sword 1 | slash |  | 0.8 |  |  | 0.07 | 23 |
| `base/vfx/effekseer-tktk01/Sword10` | Sword 10 | slash |  | 0.837 |  |  | 0.26 | 17 |
| `base/vfx/effekseer-tktk01/Sword11` | Sword 11 | slash |  | 1.283 |  |  | 5.84 | 20 |
| `base/vfx/effekseer-tktk01/Sword12` | Sword 12 | slash |  | 0.797 |  |  | 1.80 | 27 |
| `base/vfx/effekseer-tktk01/Sword13` | Sword 13 | slash |  | 2.592 |  |  | 3.82 | 26 |
| `base/vfx/effekseer-tktk01/Sword14` | Sword 14 | slash |  | 2.302 |  |  | 1.76 | 22 |
| `base/vfx/effekseer-tktk01/Sword15` | Sword 15 | slash |  | 2.275 |  |  | 3.76 | 22 |
| `base/vfx/effekseer-tktk01/Sword16` | Sword 16 | slash |  | loops |  |  | 4.98 | 71 |
| `base/vfx/effekseer-tktk01/Sword17` | Sword 17 | slash |  | 1.068 |  |  | 5.47 | 25 |
| `base/vfx/effekseer-tktk01/Sword18` | Sword 18 | slash |  | 2.933 |  |  | 4.64 | 31 |
| `base/vfx/effekseer-tktk01/Sword19` | Sword 19 | slash |  | 2.49 |  |  | 5.38 | 52 |
| `base/vfx/effekseer-tktk01/Sword2` | Sword 2 | slash |  | 0.698 |  |  | 7.62 | 21 |
| `base/vfx/effekseer-tktk01/Sword20` | Sword 20 | slash |  | 2.867 |  |  | 3.81 | 23 |
| `base/vfx/effekseer-tktk01/Sword21` | Sword 21 | slash |  | 3.333 |  |  | 3.19 | 30 |
| `base/vfx/effekseer-tktk01/Sword22` | Sword 22 | slash |  | 1.983 |  |  | 4.35 | 28 |
| `base/vfx/effekseer-tktk01/Sword3` | Sword 3 | slash |  | 1.032 |  |  | 1.28 | 23 |
| `base/vfx/effekseer-tktk01/Sword4` | Sword 4 | slash |  | 1.333 |  |  | 1.18 | 22 |
| `base/vfx/effekseer-tktk01/Sword5` | Sword 5 | slash |  | 0.6 |  |  | 2.74 | 17 |
| `base/vfx/effekseer-tktk01/Sword6` | Sword 6 | slash |  | 0.682 |  |  | 1.23 | 22 |
| `base/vfx/effekseer-tktk01/Sword7` | Sword 7 | slash |  | 0.583 |  |  | 0.93 | 18 |
| `base/vfx/effekseer-tktk01/Sword8` | Sword 8 | slash |  | 1.417 |  |  | 1.14 | 21 |
| `base/vfx/effekseer-tktk01/Sword9` | Sword 9 | slash |  | 0.867 |  |  | 1.04 | 21 |
| `base/vfx/effekseer-tktk01/Wind1` | Wind 1 | magic-impact | wind | 2.967 |  |  | 3.70 | 18 |
| `base/vfx/effekseer-tktk01/Wind2` | Wind 2 | magic-impact | wind | 0.673 |  |  | 1.28 | 26 |
| `base/vfx/effekseer-tktk01/hozyo1` | Hozyo 1 | buff |  | 1.291 |  |  | 4.84 | 28 |
| `base/vfx/effekseer-tktk01/hozyo2` | Hozyo 2 | buff |  | 1.217 |  |  | 1.09 | 30 |
| `base/vfx/effekseer-tktk01/hozyo3` | Hozyo 3 | buff |  | 0.8 |  |  | 1.06 | 26 |
| `base/vfx/effekseer-tktk01/hozyo4` | Hozyo 4 | buff |  | loops |  |  | 4.68 | 24 |
| `base/vfx/effekseer-tktk01/hozyo5` | Hozyo 5 | buff |  | loops |  |  | 4.63 | 25 |

<a id="guide-effect-packs-18"></a>
### `effect-packs §18` effekseer-tktk02

tktk vol.2's Effekseer sample effects (33), converted to file.vfx. 33 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add effekseer-tktk02`.

- Style: stylised, 3d
- Categories: fire, hit-spark, magic-impact, magic-projectile, slash
- Elements: dark, fire, holy
- Keywords: 3d, anime, effekseer, particles, tktk02

Measured on Apple M4 Pro GPU (Metal).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/effekseer-tktk02/Blow1` | Blow 1 | hit-spark |  | 1.167 |  |  | 1.61 | 36 |
| `base/vfx/effekseer-tktk02/Blow2` | Blow 2 | hit-spark |  | 2.383 |  |  | 2.29 | 40 |
| `base/vfx/effekseer-tktk02/Blow3` | Blow 3 | hit-spark |  | 2.4 |  |  | 4.85 | 29 |
| `base/vfx/effekseer-tktk02/Blow4` | Blow 4 | hit-spark |  | 1 |  |  | 3.04 | 21 |
| `base/vfx/effekseer-tktk02/Blow5` | Blow 5 | hit-spark |  | 1.332 |  |  | 5.44 | 60 |
| `base/vfx/effekseer-tktk02/Blow6` | Blow 6 | hit-spark |  | 3.6 |  |  | 5.65 | 78 |
| `base/vfx/effekseer-tktk02/Claw1` | Claw 1 | slash |  | loops |  |  | 3.39 | 37 |
| `base/vfx/effekseer-tktk02/Claw2` | Claw 2 | slash |  | loops |  |  | 2.65 | 46 |
| `base/vfx/effekseer-tktk02/Claw3` | Claw 3 | slash |  | 2.733 |  |  | 4.44 | 63 |
| `base/vfx/effekseer-tktk02/Claw4` | Claw 4 | slash |  | loops |  |  | 7.20 | 80 |
| `base/vfx/effekseer-tktk02/Dark1` | Dark 1 | magic-impact | dark | 1.165 |  |  | 4.95 | 31 |
| `base/vfx/effekseer-tktk02/Dark2` | Dark 2 | magic-impact | dark | 3.033 |  |  | 3.80 | 53 |
| `base/vfx/effekseer-tktk02/Fire1` | Fire 1 | fire | fire | 2.967 |  |  | 2.60 | 24 |
| `base/vfx/effekseer-tktk02/Fire2` | Fire 2 | fire | fire | 3.317 |  |  | 3.85 | 29 |
| `base/vfx/effekseer-tktk02/Fire3` | Fire 3 | fire | fire | 1.3 |  |  | 4.86 | 31 |
| `base/vfx/effekseer-tktk02/Fire4` | Fire 4 | fire | fire | loops |  |  | 4.24 | 37 |
| `base/vfx/effekseer-tktk02/Fire5` | Fire 5 | fire | fire | 2.533 |  |  | 6.47 | 66 |
| `base/vfx/effekseer-tktk02/Fire6` | Fire 6 | fire | fire | 1.371 |  |  | 7.19 | 41 |
| `base/vfx/effekseer-tktk02/Fire7` | Fire 7 | fire | fire | 2.998 |  |  | 8.90 | 30 |
| `base/vfx/effekseer-tktk02/Fire8` | Fire 8 | fire | fire | loops |  |  | 3.92 | 23 |
| `base/vfx/effekseer-tktk02/Gun1` | Gun 1 | magic-projectile |  | 2.036 |  |  | 4.14 | 39 |
| `base/vfx/effekseer-tktk02/Gun2` | Gun 2 | magic-projectile |  | 2.968 |  |  | 3.02 | 48 |
| `base/vfx/effekseer-tktk02/Gun3` | Gun 3 | magic-projectile |  | 2.169 |  |  | 1.72 | 28 |
| `base/vfx/effekseer-tktk02/Gun4` | Gun 4 | magic-projectile |  | loops |  |  | 3.28 | 65 |
| `base/vfx/effekseer-tktk02/Gun5` | Gun 5 | magic-projectile |  | 1.733 |  |  | 4.08 | 35 |
| `base/vfx/effekseer-tktk02/Gun6` | Gun 6 | magic-projectile |  | 2.217 |  |  | 2.31 | 39 |
| `base/vfx/effekseer-tktk02/Gun7` | Gun 7 | magic-projectile |  | 3.2 |  |  | 13.91 | 54 |
| `base/vfx/effekseer-tktk02/Gun8` | Gun 8 | magic-projectile |  | 2.1 |  |  | 0.52 | 28 |
| `base/vfx/effekseer-tktk02/Gun9` | Gun 9 | magic-projectile |  | 3.067 |  |  | 1.91 | 48 |
| `base/vfx/effekseer-tktk02/Light1` | Light 1 | magic-impact | holy | 1.082 |  |  | 3.57 | 41 |
| `base/vfx/effekseer-tktk02/Light2` | Light 2 | magic-impact | holy | 2.6 |  |  | 3.18 | 50 |
| `base/vfx/effekseer-tktk02/Light3` | Light 3 | magic-impact | holy | loops |  |  | 6.40 | 93 |
| `base/vfx/effekseer-tktk02/Light4` | Light 4 | magic-impact | holy | 2.777 |  |  | 3.69 | 78 |

<a id="guide-effect-packs-19"></a>
### `effect-packs §19` geometric

Texture-free hard geometry for minimal-geometric and faceted-low-poly: nine visual mechanisms, no bloom or soft sprites. 9 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add geometric`.

- Style: stylised, 3d
- Categories: ambient, buff, explosion, gunfire, hit-spark, magic-impact, smoke, splash, ui
- Elements: earth, water
- Keywords: faceted-low-poly, geometric, minimal-geometric, texture-free

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `base/vfx/geometric/celebration` | Coral Confetti | ui |  | 2 | 2.2 × 1.6 × 3.5 | ground | 0.04 | 11 |
| `base/vfx/geometric/dust` | Slate Dust | smoke | earth | 1.5 | 0.4 × 0.4 × 0.6 | ground | 0.04 | 11 |
| `base/vfx/geometric/explosion` | Facet Burst | explosion |  | 1.2 | 3.1 × 1.9 × 2.7 | centre | 0.08 | 14 |
| `base/vfx/geometric/impact` | Angular Impact | hit-spark |  | 0.5 | 0.9 × 0.7 × 1.2 | ground | 0.08 | 14 |
| `base/vfx/geometric/magic` | Open Triangles | magic-impact |  | 1.2 | 1.2 × 1.2 × 1.2 | ground | 0.05 | 13 |
| `base/vfx/geometric/muzzle` | Geometric Pulse | gunfire |  | 0.12 | 0.1 × 0.2 × 0.1 | centre | 0.03 | 11 |
| `base/vfx/geometric/pickup` | Rising Cubes | buff |  | 1 | 0.3 × 0.5 × 0.2 | centre | 0.03 | 11 |
| `base/vfx/geometric/splash` | Facet Splash | splash | water | 1.2 | 1.3 × 1.2 × 1.3 | centre | 0.04 | 11 |
| `base/vfx/geometric/trail` | Facet Trail | ambient |  | loops | 0.0 × 0.1 × 0.0 | ground | 0.04 | 11 |

<a id="guide-effect-packs-20"></a>
### `effect-packs §20` Ambient effects

Ambient effects (ambient): 36 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-hato01`, `effekseer-pierre01`, `effekseer-starter`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/bubbles` (Bubbles), `base/vfx/cartoon-2d/fireflies` (Fireflies), `base/vfx/cartoon-2d/leaves` (Falling Leaves), `base/vfx/cartoon-2d/twinkle` (Twinkle)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/bubbles` (Bubbles), `base/vfx/cartoon-2d-rubber-hose/fireflies` (Fireflies), `base/vfx/cartoon-2d-rubber-hose/leaves` (Falling Leaves), `base/vfx/cartoon-2d-rubber-hose/twinkle` (Twinkle)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/bubbles` (Bubbles), `base/vfx/cartoon-2d-storybook/fireflies` (Fireflies), `base/vfx/cartoon-2d-storybook/leaves` (Falling Leaves), `base/vfx/cartoon-2d-storybook/twinkle` (Twinkle)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/bubbles` (Bubbles), `base/vfx/cartoon-2d-watercolor/fireflies` (Fireflies), `base/vfx/cartoon-2d-watercolor/leaves` (Falling Leaves), `base/vfx/cartoon-2d-watercolor/twinkle` (Twinkle)
- cinematic-realism: `base/vfx/cinematic-realism/electric-sparks` (Arcing Fault), `base/vfx/cinematic-realism/embers` (Drifting Embers), `base/vfx/cinematic-realism/fireflies` (Fireflies), `base/vfx/cinematic-realism/ground-fog` (Ground Fog), `base/vfx/cinematic-realism/leaves` (Falling Leaves), `base/vfx/cinematic-realism/sunbeam-dust` (Lit Dust)
- effekseer-hato01: `base/vfx/effekseer-hato01/aoi-chou` (Aoi Chou), `base/vfx/effekseer-hato01/iwai-ume` (Iwai Ume), `base/vfx/effekseer-hato01/kiroi-chou` (Kiroi Chou), `base/vfx/effekseer-hato01/sakura1` (Sakura 1)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/Atmosphere` (Atmosphere), `base/vfx/effekseer-pierre01/Background` (Background)
- effekseer-starter: `base/vfx/effekseer-starter/electric_dust` (Floating Motes), `base/vfx/effekseer-starter/sakura1` (Sakura Petals)
- geometric: `base/vfx/geometric/trail` (Facet Trail)
- pixel-scenario: `base/vfx/pixel-scenario/cherry-blossom-petals` (Thorn Ambient), `base/vfx/pixel-scenario/embers-rising` (Blaze Ambient), `base/vfx/pixel-scenario/falling-leaves` (Bloom Ambient), `base/vfx/pixel-scenario/fireflies` (Mystic Ambient)
- realistic-starter: `base/vfx/realistic-starter/ambient-dust` (Ambient Dust)

<a id="guide-effect-packs-21"></a>
### `effect-packs §21` Basic effects

Basic effects (basic): 11 in `effekseer-effekseer01`.

- effekseer-effekseer01: `base/vfx/effekseer-effekseer01/Simple_Distortion` (Simple Distortion), `base/vfx/effekseer-effekseer01/Simple_GeneratingPosition1` (Simple Generating Position 1), `base/vfx/effekseer-effekseer01/Simple_Model_UV` (Simple Model UV), `base/vfx/effekseer-effekseer01/Simple_Ribbon_Parent` (Simple Ribbon Parent), `base/vfx/effekseer-effekseer01/Simple_Ribbon_Sword` (Simple Ribbon Sword), `base/vfx/effekseer-effekseer01/Simple_Ring_Shape1` (Simple Ring Shape 1), `base/vfx/effekseer-effekseer01/Simple_Ring_Shape2` (Simple Ring Shape 2), `base/vfx/effekseer-effekseer01/Simple_Sprite_BillBoard` (Simple Sprite Bill Board), `base/vfx/effekseer-effekseer01/Simple_Sprite_Fixed` (Simple Sprite Fixed), `base/vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis` (Simple Sprite Fixed YAxis), `base/vfx/effekseer-effekseer01/Simple_Track1` (Simple Track 1)

<a id="guide-effect-packs-22"></a>
### `effect-packs §22` Beam effects

Beam effects (beam): 10 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-effekseer01`.

- cartoon-2d: `base/vfx/cartoon-2d/laser` (Ray Beam)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/laser` (Ray Beam)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/laser` (Ray Beam)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/laser` (Ray Beam)
- cinematic-realism: `base/vfx/cinematic-realism/laser` (Laser Beam)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/blue_laser` (Blue Laser)
- effekseer-effekseer01: `base/vfx/effekseer-effekseer01/Homing_Laser01` (Homing Laser 01), `base/vfx/effekseer-effekseer01/Laser01` (Laser 01), `base/vfx/effekseer-effekseer01/Laser02` (Laser 02), `base/vfx/effekseer-effekseer01/Laser03` (Laser 03)

<a id="guide-effect-packs-23"></a>
### `effect-packs §23` Buff effects

Buff effects (buff): 25 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `geometric`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/buff` (Buff), `base/vfx/cartoon-2d/powerup` (Power Up)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/buff` (Buff), `base/vfx/cartoon-2d-rubber-hose/powerup` (Power Up)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/buff` (Buff), `base/vfx/cartoon-2d-storybook/powerup` (Power Up)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/buff` (Buff), `base/vfx/cartoon-2d-watercolor/powerup` (Power Up)
- cinematic-realism: `base/vfx/cinematic-realism/buff` (Rising Strands), `base/vfx/cinematic-realism/pickup` (Amber Motes)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/PowerUp` (Power Up)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/Herald` (Herald)
- effekseer-starter: `base/vfx/effekseer-starter/PowerUp` (Power Up), `base/vfx/effekseer-starter/hozyo2` (Power Rings), `base/vfx/effekseer-starter/hozyo4` (Attack Up)
- effekseer-suzuki01: `base/vfx/effekseer-suzuki01/aura` (Aura)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/hozyo1` (Hozyo 1), `base/vfx/effekseer-tktk01/hozyo2` (Hozyo 2), `base/vfx/effekseer-tktk01/hozyo3` (Hozyo 3), `base/vfx/effekseer-tktk01/hozyo4` (Hozyo 4), `base/vfx/effekseer-tktk01/hozyo5` (Hozyo 5)
- geometric: `base/vfx/geometric/pickup` (Rising Cubes)
- pixel-scenario: `base/vfx/pixel-scenario/charge-up-energy` (Pulse Charge), `base/vfx/pixel-scenario/dark-shadow-aura` (Night Aura), `base/vfx/pixel-scenario/hex-shield-barrier` (Flux Shield)

<a id="guide-effect-packs-24"></a>
### `effect-packs §24` Debris effects

Debris effects (debris): 1 in `realistic-starter`.

- realistic-starter: `base/vfx/realistic-starter/dust-burst` (Dust Burst)

<a id="guide-effect-packs-25"></a>
### `effect-packs §25` Decal effects

Decal effects (decal): 4 in `cinematic-realism`.

- cinematic-realism: `base/vfx/cinematic-realism/mark-bullet` (Bullet Hole), `base/vfx/cinematic-realism/mark-burn` (Burn Mark), `base/vfx/cinematic-realism/mark-wet` (Wet Patch), `base/vfx/cinematic-realism/skid` (Skid Mark)

<a id="guide-effect-packs-26"></a>
### `effect-packs §26` Destruction effects

Destruction effects (destruction): 15 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `base/vfx/cartoon-2d/break-crate` (Crate Break), `base/vfx/cartoon-2d/break-ice` (Ice Shatter), `base/vfx/cartoon-2d/break-rock` (Rock Break)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/break-crate` (Crate Break), `base/vfx/cartoon-2d-rubber-hose/break-ice` (Ice Shatter), `base/vfx/cartoon-2d-rubber-hose/break-rock` (Rock Break)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/break-crate` (Crate Break), `base/vfx/cartoon-2d-storybook/break-ice` (Ice Shatter), `base/vfx/cartoon-2d-storybook/break-rock` (Rock Break)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/break-crate` (Crate Break), `base/vfx/cartoon-2d-watercolor/break-ice` (Ice Shatter), `base/vfx/cartoon-2d-watercolor/break-rock` (Rock Break)
- cinematic-realism: `base/vfx/cinematic-realism/break-glass` (Pane Break), `base/vfx/cinematic-realism/break-stone` (Masonry Break), `base/vfx/cinematic-realism/break-wood` (Crate Break)

<a id="guide-effect-packs-27"></a>
### `effect-packs §27` Explosion effects

Explosion effects (explosion): 30 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/explosion-small` (Pop Boom), `base/vfx/cartoon-2d/explosion` (Boom)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/explosion-small` (Pop Boom), `base/vfx/cartoon-2d-rubber-hose/explosion` (Boom)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/explosion-small` (Pop Boom), `base/vfx/cartoon-2d-storybook/explosion` (Boom)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/explosion-small` (Pop Boom), `base/vfx/cartoon-2d-watercolor/explosion` (Boom)
- cinematic-realism: `base/vfx/cinematic-realism/explosion-air` (Airburst), `base/vfx/cinematic-realism/explosion-energy` (Energy Blast), `base/vfx/cinematic-realism/explosion-large` (Large Blast), `base/vfx/cinematic-realism/explosion-small` (Grenade Blast), `base/vfx/cinematic-realism/explosion-water` (Depth Blast), `base/vfx/cinematic-realism/explosion` (Cinematic Explosion), `base/vfx/cinematic-realism/flashbang` (Flash Charge)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/boss_death` (Boss Death)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/Breakdown` (Breakdown), `base/vfx/effekseer-magicalxspiral/IcicleBroken` (Icicle Broken), `base/vfx/effekseer-magicalxspiral/StairBroken` (Stair Broken), `base/vfx/effekseer-magicalxspiral/StairBroken3` (Stair Broken 3)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/FeatherBomb` (Feather Bomb)
- effekseer-starter: `base/vfx/effekseer-starter/Blow4` (Bright Explosion), `base/vfx/effekseer-starter/Breakdown` (Breakdown Burst), `base/vfx/effekseer-starter/FeatherBomb` (Smoke Bomb), `base/vfx/effekseer-starter/boss_death` (Boss Death), `base/vfx/effekseer-starter/meteo1` (Meteor Strike)
- geometric: `base/vfx/geometric/explosion` (Facet Burst)
- pixel-scenario: `base/vfx/pixel-scenario/big-fire-explosion` (Cinder Explosion), `base/vfx/pixel-scenario/meteor-strike` (Ember Meteor)
- realistic-starter: `base/vfx/realistic-starter/explosion` (Explosion)

<a id="guide-effect-packs-28"></a>
### `effect-packs §28` Fire effects

Fire effects (fire): 45 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/campfire` (Campfire), `base/vfx/cartoon-2d/fuse` (Fuse Sparks), `base/vfx/cartoon-2d/torch` (Torch Flame)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/campfire` (Campfire), `base/vfx/cartoon-2d-rubber-hose/fuse` (Fuse Sparks), `base/vfx/cartoon-2d-rubber-hose/torch` (Torch Flame)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/campfire` (Campfire), `base/vfx/cartoon-2d-storybook/fuse` (Fuse Sparks), `base/vfx/cartoon-2d-storybook/torch` (Torch Flame)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/campfire` (Campfire), `base/vfx/cartoon-2d-watercolor/fuse` (Fuse Sparks), `base/vfx/cartoon-2d-watercolor/torch` (Torch Flame)
- cinematic-realism: `base/vfx/cinematic-realism/campfire` (Campfire), `base/vfx/cinematic-realism/fire-large` (Large Fire), `base/vfx/cinematic-realism/flamethrower` (Flame Stream)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/fire` (Fire)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/FirePot1` (Fire Pot 1), `base/vfx/effekseer-magicalxspiral/FirePot2` (Fire Pot 2), `base/vfx/effekseer-magicalxspiral/Magic1_fire` (Magic 1 Fire), `base/vfx/effekseer-magicalxspiral/Magic2_fire` (Magic 2 Fire)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicFire1` (Magic Fire 1), `base/vfx/effekseer-nextsoft01/MagicFire2` (Magic Fire 2)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/Flame` (Flame)
- effekseer-starter: `base/vfx/effekseer-starter/Fire7` (Fire Pillar), `base/vfx/effekseer-starter/MagicFire2` (Fire Breath)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Fire1` (Fire 1), `base/vfx/effekseer-tktk01/Fire2` (Fire 2), `base/vfx/effekseer-tktk01/Fire3` (Fire 3), `base/vfx/effekseer-tktk01/Fire4` (Fire 4), `base/vfx/effekseer-tktk01/Fire5` (Fire 5), `base/vfx/effekseer-tktk01/Fire6` (Fire 6), `base/vfx/effekseer-tktk01/Fire7` (Fire 7), `base/vfx/effekseer-tktk01/Fire8` (Fire 8)
- effekseer-tktk02: `base/vfx/effekseer-tktk02/Fire1` (Fire 1), `base/vfx/effekseer-tktk02/Fire2` (Fire 2), `base/vfx/effekseer-tktk02/Fire3` (Fire 3), `base/vfx/effekseer-tktk02/Fire4` (Fire 4), `base/vfx/effekseer-tktk02/Fire5` (Fire 5), `base/vfx/effekseer-tktk02/Fire6` (Fire 6), `base/vfx/effekseer-tktk02/Fire7` (Fire 7), `base/vfx/effekseer-tktk02/Fire8` (Fire 8)
- pixel-scenario: `base/vfx/pixel-scenario/campfire` (Ember Flame), `base/vfx/pixel-scenario/dragon-fire-breath` (Cinder Breath), `base/vfx/pixel-scenario/fire-tornado` (Blaze Tornado)
- realistic-starter: `base/vfx/realistic-starter/torch-fire` (Torch Fire)

<a id="guide-effect-packs-29"></a>
### `effect-packs §29` Gunfire effects

Gunfire effects (gunfire): 11 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `geometric`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/muzzle` (Pop Muzzle)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/muzzle` (Pop Muzzle)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/muzzle` (Pop Muzzle)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/muzzle` (Pop Muzzle)
- cinematic-realism: `base/vfx/cinematic-realism/muzzle-auto` (Automatic Fire), `base/vfx/cinematic-realism/muzzle-heavy` (Heavy Muzzle Blast), `base/vfx/cinematic-realism/muzzle-suppressed` (Suppressed Shot), `base/vfx/cinematic-realism/muzzle` (Powder Flash), `base/vfx/cinematic-realism/shell` (Spent Casing)
- geometric: `base/vfx/geometric/muzzle` (Geometric Pulse)
- realistic-starter: `base/vfx/realistic-starter/muzzle-flash` (Muzzle Flash)

<a id="guide-effect-packs-30"></a>
### `effect-packs §30` Heal effects

Heal effects (heal): 22 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/heal` (Heal)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/heal` (Heal)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/heal` (Heal)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/heal` (Heal)
- cinematic-realism: `base/vfx/cinematic-realism/heal` (Mending Light)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/HealPotion1` (Heal Potion 1), `base/vfx/effekseer-nextsoft01/HealPotion2` (Heal Potion 2), `base/vfx/effekseer-nextsoft01/MagicHeal1` (Magic Heal 1), `base/vfx/effekseer-nextsoft01/MagicHeal2` (Magic Heal 2)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/Benediction` (Benediction)
- effekseer-starter: `base/vfx/effekseer-starter/Cure1` (Star Cure), `base/vfx/effekseer-starter/Cure7` (Light Cure), `base/vfx/effekseer-starter/HealPotion2` (Potion), `base/vfx/effekseer-starter/MagicHeal2` (Healing Circle)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Cure1` (Cure 1), `base/vfx/effekseer-tktk01/Cure2` (Cure 2), `base/vfx/effekseer-tktk01/Cure3` (Cure 3), `base/vfx/effekseer-tktk01/Cure4` (Cure 4), `base/vfx/effekseer-tktk01/Cure5` (Cure 5), `base/vfx/effekseer-tktk01/Cure6` (Cure 6), `base/vfx/effekseer-tktk01/Cure7` (Cure 7)
- pixel-scenario: `base/vfx/pixel-scenario/healing-with-rising-crosses` (Bloom Heal)

<a id="guide-effect-packs-31"></a>
### `effect-packs §31` Hit spark effects

Hit spark effects (hit-spark): 56 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/bullet-hit` (Pellet Pop), `base/vfx/cartoon-2d/hit-heavy` (Smash Hit), `base/vfx/cartoon-2d/hit-small` (Tap Hit), `base/vfx/cartoon-2d/hit` (Pow Hit), `base/vfx/cartoon-2d/stomp` (Stomp)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/bullet-hit` (Pellet Pop), `base/vfx/cartoon-2d-rubber-hose/hit-heavy` (Smash Hit), `base/vfx/cartoon-2d-rubber-hose/hit-small` (Tap Hit), `base/vfx/cartoon-2d-rubber-hose/hit` (Pow Hit), `base/vfx/cartoon-2d-rubber-hose/stomp` (Stomp)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/bullet-hit` (Pellet Pop), `base/vfx/cartoon-2d-storybook/hit-heavy` (Smash Hit), `base/vfx/cartoon-2d-storybook/hit-small` (Tap Hit), `base/vfx/cartoon-2d-storybook/hit` (Pow Hit), `base/vfx/cartoon-2d-storybook/stomp` (Stomp)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/bullet-hit` (Pellet Pop), `base/vfx/cartoon-2d-watercolor/hit-heavy` (Smash Hit), `base/vfx/cartoon-2d-watercolor/hit-small` (Tap Hit), `base/vfx/cartoon-2d-watercolor/hit` (Pow Hit), `base/vfx/cartoon-2d-watercolor/stomp` (Stomp)
- cinematic-realism: `base/vfx/cinematic-realism/impact-concrete` (Concrete Impact), `base/vfx/cinematic-realism/impact-dirt` (Earth Impact), `base/vfx/cinematic-realism/impact-flesh` (Cloth Impact), `base/vfx/cinematic-realism/impact-glass` (Glass Impact), `base/vfx/cinematic-realism/impact-water` (Water Impact), `base/vfx/cinematic-realism/impact-wood` (Timber Impact), `base/vfx/cinematic-realism/impact` (Steel Impact)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/hit_eff` (Hit Eff)
- effekseer-starter: `base/vfx/effekseer-starter/Attack_Impact` (Attack Impact), `base/vfx/effekseer-starter/Blow2` (Spark Hit), `base/vfx/effekseer-starter/Blow8` (Star Hit), `base/vfx/effekseer-starter/hit_eff` (Hit Flash)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Blow1` (Blow 1), `base/vfx/effekseer-tktk01/Blow10` (Blow 10), `base/vfx/effekseer-tktk01/Blow11` (Blow 11), `base/vfx/effekseer-tktk01/Blow12` (Blow 12), `base/vfx/effekseer-tktk01/Blow13` (Blow 13), `base/vfx/effekseer-tktk01/Blow14` (Blow 14), `base/vfx/effekseer-tktk01/Blow15` (Blow 15), `base/vfx/effekseer-tktk01/Blow2` (Blow 2), `base/vfx/effekseer-tktk01/Blow3` (Blow 3), `base/vfx/effekseer-tktk01/Blow4` (Blow 4), `base/vfx/effekseer-tktk01/Blow5` (Blow 5), `base/vfx/effekseer-tktk01/Blow6` (Blow 6), `base/vfx/effekseer-tktk01/Blow7` (Blow 7), `base/vfx/effekseer-tktk01/Blow8` (Blow 8), `base/vfx/effekseer-tktk01/Blow9` (Blow 9)
- effekseer-tktk02: `base/vfx/effekseer-tktk02/Blow1` (Blow 1), `base/vfx/effekseer-tktk02/Blow2` (Blow 2), `base/vfx/effekseer-tktk02/Blow3` (Blow 3), `base/vfx/effekseer-tktk02/Blow4` (Blow 4), `base/vfx/effekseer-tktk02/Blow5` (Blow 5), `base/vfx/effekseer-tktk02/Blow6` (Blow 6)
- geometric: `base/vfx/geometric/impact` (Angular Impact)
- pixel-scenario: `base/vfx/pixel-scenario/electric-impact-hit` (Static Impact)
- realistic-starter: `base/vfx/realistic-starter/bullet-impact` (Bullet Impact)

<a id="guide-effect-packs-32"></a>
### `effect-packs §32` Level up effects

Level up effects (level-up): 4 in `cinematic-realism`, `effekseer-starter`, `pixel-scenario`.

- cinematic-realism: `base/vfx/cinematic-realism/level-up` (Level Up)
- effekseer-starter: `base/vfx/effekseer-starter/Light4` (Level Up Arrows), `base/vfx/effekseer-starter/Thunder1` (Golden Rise)
- pixel-scenario: `base/vfx/pixel-scenario/level-up` (Gilded Level Up)

<a id="guide-effect-packs-33"></a>
### `effect-packs §33` Magic impact effects

Magic impact effects (magic-impact): 133 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `geometric`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/charge` (Charge Ball), `base/vfx/cartoon-2d/magic-circle` (Spell Circle), `base/vfx/cartoon-2d/magic` (Sparkle Cast), `base/vfx/cartoon-2d/portal` (Portal), `base/vfx/cartoon-2d/whirlwind` (Whirlwind)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/charge` (Charge Ball), `base/vfx/cartoon-2d-rubber-hose/magic-circle` (Spell Circle), `base/vfx/cartoon-2d-rubber-hose/magic` (Sparkle Cast), `base/vfx/cartoon-2d-rubber-hose/portal` (Portal), `base/vfx/cartoon-2d-rubber-hose/whirlwind` (Whirlwind)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/charge` (Charge Ball), `base/vfx/cartoon-2d-storybook/magic-circle` (Spell Circle), `base/vfx/cartoon-2d-storybook/magic` (Sparkle Cast), `base/vfx/cartoon-2d-storybook/portal` (Portal), `base/vfx/cartoon-2d-storybook/whirlwind` (Whirlwind)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/charge` (Charge Ball), `base/vfx/cartoon-2d-watercolor/magic-circle` (Spell Circle), `base/vfx/cartoon-2d-watercolor/magic` (Sparkle Cast), `base/vfx/cartoon-2d-watercolor/portal` (Portal), `base/vfx/cartoon-2d-watercolor/whirlwind` (Whirlwind)
- cinematic-realism: `base/vfx/cinematic-realism/charge` (Gathering Charge), `base/vfx/cinematic-realism/magic-circle` (Drawn Circle), `base/vfx/cinematic-realism/magic` (Aether Wisps)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/smallTexturesRibbon` (Small Textures Ribbon), `base/vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital` (Small Textures Square Wave Orbital)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/A_Gohlem1` (A Gohlem 1), `base/vfx/effekseer-magicalxspiral/A_Salamander1` (A Salamander 1), `base/vfx/effekseer-magicalxspiral/A_Salamander2` (A Salamander 2), `base/vfx/effekseer-magicalxspiral/A_Salamander4` (A Salamander 4), `base/vfx/effekseer-magicalxspiral/A_Sylph1` (A Sylph 1), `base/vfx/effekseer-magicalxspiral/A_Sylph2` (A Sylph 2), `base/vfx/effekseer-magicalxspiral/A_Undine1` (A Undine 1), `base/vfx/effekseer-magicalxspiral/A_Undine2` (A Undine 2), `base/vfx/effekseer-magicalxspiral/AquaPoint` (Aqua Point), `base/vfx/effekseer-magicalxspiral/Attack_Impact` (Attack Impact), `base/vfx/effekseer-magicalxspiral/Gohlem1` (Gohlem 1), `base/vfx/effekseer-magicalxspiral/Gohlem11` (Gohlem 11), `base/vfx/effekseer-magicalxspiral/Gohlem5` (Gohlem 5), `base/vfx/effekseer-magicalxspiral/Gohlem7` (Gohlem 7), `base/vfx/effekseer-magicalxspiral/Holy1` (Holy 1), `base/vfx/effekseer-magicalxspiral/Holy2` (Holy 2), `base/vfx/effekseer-magicalxspiral/Impact` (Impact), `base/vfx/effekseer-magicalxspiral/Magic1_earth` (Magic 1 Earth), `base/vfx/effekseer-magicalxspiral/Magic1_holy` (Magic 1 Holy), `base/vfx/effekseer-magicalxspiral/Magic1_water` (Magic 1 Water), `base/vfx/effekseer-magicalxspiral/Magic1_wind` (Magic 1 Wind), `base/vfx/effekseer-magicalxspiral/Magic2_earth` (Magic 2 Earth), `base/vfx/effekseer-magicalxspiral/Magic2_holy` (Magic 2 Holy), `base/vfx/effekseer-magicalxspiral/Magic2_water` (Magic 2 Water), `base/vfx/effekseer-magicalxspiral/Magic2_wind` (Magic 2 Wind), `base/vfx/effekseer-magicalxspiral/Rock1` (Rock 1), `base/vfx/effekseer-magicalxspiral/Rock2` (Rock 2), `base/vfx/effekseer-magicalxspiral/Salamander1` (Salamander 1), `base/vfx/effekseer-magicalxspiral/Salamander11` (Salamander 11), `base/vfx/effekseer-magicalxspiral/Salamander12` (Salamander 12), `base/vfx/effekseer-magicalxspiral/Salamander14` (Salamander 14), `base/vfx/effekseer-magicalxspiral/Salamander15` (Salamander 15), `base/vfx/effekseer-magicalxspiral/Salamander2` (Salamander 2), `base/vfx/effekseer-magicalxspiral/Salamander3` (Salamander 3), `base/vfx/effekseer-magicalxspiral/Salamander4` (Salamander 4), `base/vfx/effekseer-magicalxspiral/Salamander5` (Salamander 5), `base/vfx/effekseer-magicalxspiral/Salamander6` (Salamander 6), `base/vfx/effekseer-magicalxspiral/Salamander7` (Salamander 7), `base/vfx/effekseer-magicalxspiral/Sylph1` (Sylph 1), `base/vfx/effekseer-magicalxspiral/Sylph10` (Sylph 10), `base/vfx/effekseer-magicalxspiral/Sylph11` (Sylph 11), `base/vfx/effekseer-magicalxspiral/Sylph13` (Sylph 13), `base/vfx/effekseer-magicalxspiral/Sylph15` (Sylph 15), `base/vfx/effekseer-magicalxspiral/Sylph2` (Sylph 2), `base/vfx/effekseer-magicalxspiral/Sylph3` (Sylph 3), `base/vfx/effekseer-magicalxspiral/Sylph4` (Sylph 4), `base/vfx/effekseer-magicalxspiral/Sylph5` (Sylph 5), `base/vfx/effekseer-magicalxspiral/Sylph6` (Sylph 6), `base/vfx/effekseer-magicalxspiral/Sylph7` (Sylph 7), `base/vfx/effekseer-magicalxspiral/Sylph8` (Sylph 8), `base/vfx/effekseer-magicalxspiral/Thunder1` (Thunder 1), `base/vfx/effekseer-magicalxspiral/Thunder2` (Thunder 2), `base/vfx/effekseer-magicalxspiral/Thunder3` (Thunder 3), `base/vfx/effekseer-magicalxspiral/Topwater1` (Topwater 1), `base/vfx/effekseer-magicalxspiral/Undine1` (Undine 1), `base/vfx/effekseer-magicalxspiral/Undine11` (Undine 11), `base/vfx/effekseer-magicalxspiral/Undine13` (Undine 13), `base/vfx/effekseer-magicalxspiral/Undine2` (Undine 2), `base/vfx/effekseer-magicalxspiral/Undine3` (Undine 3), `base/vfx/effekseer-magicalxspiral/Undine4` (Undine 4), `base/vfx/effekseer-magicalxspiral/Undine5` (Undine 5), `base/vfx/effekseer-magicalxspiral/Undine6` (Undine 6), `base/vfx/effekseer-magicalxspiral/Undine8` (Undine 8), `base/vfx/effekseer-magicalxspiral/WHead1` (WHead 1), `base/vfx/effekseer-magicalxspiral/WHead2` (WHead 2), `base/vfx/effekseer-magicalxspiral/WHead3` (WHead 3), `base/vfx/effekseer-magicalxspiral/Water_Impact` (Water Impact), `base/vfx/effekseer-magicalxspiral/Wind_Impact` (Wind Impact), `base/vfx/effekseer-magicalxspiral/salamander13` (Salamander 13)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicCold` (Magic Cold), `base/vfx/effekseer-nextsoft01/MagicDark` (Magic Dark), `base/vfx/effekseer-nextsoft01/MagicLight` (Magic Light), `base/vfx/effekseer-nextsoft01/MagicThunder` (Magic Thunder), `base/vfx/effekseer-nextsoft01/MagicWater` (Magic Water)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/LightningStrike` (Lightning Strike), `base/vfx/effekseer-pierre01/MonsterRoar` (Monster Roar)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/LossOfBlood` (Loss Of Blood)
- effekseer-starter: `base/vfx/effekseer-starter/MagicCold` (Frost), `base/vfx/effekseer-starter/MagicDark` (Dark Curse), `base/vfx/effekseer-starter/MagicWater` (Bubble Burst), `base/vfx/effekseer-starter/Undine4` (Water Burst)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Dark1` (Dark 1), `base/vfx/effekseer-tktk01/Dark2` (Dark 2), `base/vfx/effekseer-tktk01/Dark3` (Dark 3), `base/vfx/effekseer-tktk01/Light1` (Light 1), `base/vfx/effekseer-tktk01/Light2` (Light 2), `base/vfx/effekseer-tktk01/Light3` (Light 3), `base/vfx/effekseer-tktk01/Light4` (Light 4), `base/vfx/effekseer-tktk01/Magic1` (Magic 1), `base/vfx/effekseer-tktk01/Magic2` (Magic 2), `base/vfx/effekseer-tktk01/Soil1` (Soil 1), `base/vfx/effekseer-tktk01/Wind1` (Wind 1), `base/vfx/effekseer-tktk01/Wind2` (Wind 2)
- effekseer-tktk02: `base/vfx/effekseer-tktk02/Dark1` (Dark 1), `base/vfx/effekseer-tktk02/Dark2` (Dark 2), `base/vfx/effekseer-tktk02/Light1` (Light 1), `base/vfx/effekseer-tktk02/Light2` (Light 2), `base/vfx/effekseer-tktk02/Light3` (Light 3), `base/vfx/effekseer-tktk02/Light4` (Light 4)
- geometric: `base/vfx/geometric/magic` (Open Triangles)
- pixel-scenario: `base/vfx/pixel-scenario/arcane-magic-circle` (Eldritch Magic Circle), `base/vfx/pixel-scenario/frost-nova` (Frost Nova), `base/vfx/pixel-scenario/ground-slam-quake` (Stone Ground Slam), `base/vfx/pixel-scenario/holy-pillar-of-light` (Divine Pillar), `base/vfx/pixel-scenario/ice-spikes-erupting` (Frozen Spikes), `base/vfx/pixel-scenario/lava-eruption-pillar` (Volcanic Pillar), `base/vfx/pixel-scenario/lightning-strike-from-the-sky` (Storm Strike), `base/vfx/pixel-scenario/water-splash` (Aqua Splash)

<a id="guide-effect-packs-34"></a>
### `effect-packs §34` Magic projectile effects

Magic projectile effects (magic-projectile): 41 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/magic-bolt` (Magic Bolt)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/magic-bolt` (Magic Bolt)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/magic-bolt` (Magic Bolt)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/magic-bolt` (Magic Bolt)
- cinematic-realism: `base/vfx/cinematic-realism/magic-bolt` (Arcane Bolt)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/drill` (Drill)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/Icicle1` (Icicle 1), `base/vfx/effekseer-magicalxspiral/Icicle2` (Icicle 2), `base/vfx/effekseer-magicalxspiral/Icicle3` (Icicle 3), `base/vfx/effekseer-magicalxspiral/IcicleLocus` (Icicle Locus), `base/vfx/effekseer-magicalxspiral/Lance3` (Lance 3), `base/vfx/effekseer-magicalxspiral/Lance4` (Lance 4), `base/vfx/effekseer-magicalxspiral/Lance5` (Lance 5), `base/vfx/effekseer-magicalxspiral/meteo1` (Meteo 1)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicMeteo` (Magic Meteo), `base/vfx/effekseer-nextsoft01/Spear1` (Spear 1), `base/vfx/effekseer-nextsoft01/Spear2` (Spear 2)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/MissileBoost` (Missile Boost), `base/vfx/effekseer-pierre01/PhantasmMeteor` (Phantasm Meteor), `base/vfx/effekseer-pierre01/PhantasmMeteor_Single` (Phantasm Meteor Single), `base/vfx/effekseer-pierre01/SonicBoom` (Sonic Boom)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/BloodLance` (Blood Lance), `base/vfx/effekseer-pierre02/FireBall` (Fire Ball)
- effekseer-starter: `base/vfx/effekseer-starter/FireBall` (Fireball), `base/vfx/effekseer-starter/Icicle2` (Icicles), `base/vfx/effekseer-starter/Laser03` (Green Laser), `base/vfx/effekseer-starter/Spear2` (Light Spear)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Arrow1` (Arrow 1), `base/vfx/effekseer-tktk01/Arrow2` (Arrow 2)
- effekseer-tktk02: `base/vfx/effekseer-tktk02/Gun1` (Gun 1), `base/vfx/effekseer-tktk02/Gun2` (Gun 2), `base/vfx/effekseer-tktk02/Gun3` (Gun 3), `base/vfx/effekseer-tktk02/Gun4` (Gun 4), `base/vfx/effekseer-tktk02/Gun5` (Gun 5), `base/vfx/effekseer-tktk02/Gun6` (Gun 6), `base/vfx/effekseer-tktk02/Gun7` (Gun 7), `base/vfx/effekseer-tktk02/Gun8` (Gun 8), `base/vfx/effekseer-tktk02/Gun9` (Gun 9)
- pixel-scenario: `base/vfx/pixel-scenario/blue-energy-beam` (Aether Beam), `base/vfx/pixel-scenario/fireball-projectile` (Ember Projectile), `base/vfx/pixel-scenario/plasma-orb` (Ion Orb)

<a id="guide-effect-packs-35"></a>
### `effect-packs §35` Marker effects

Marker effects (marker): 7 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `base/vfx/cartoon-2d/checkpoint` (Checkpoint)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/checkpoint` (Checkpoint)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/checkpoint` (Checkpoint)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/checkpoint` (Checkpoint)
- cinematic-realism: `base/vfx/cinematic-realism/beacon` (Marker Beacon), `base/vfx/cinematic-realism/chest-open` (Opened Chest), `base/vfx/cinematic-realism/loot-glow` (Rare Drop Glow)

<a id="guide-effect-packs-36"></a>
### `effect-packs §36` Movement effects

Movement effects (movement): 31 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `base/vfx/cartoon-2d/dash` (Dash Lines), `base/vfx/cartoon-2d/double-jump` (Air Jump), `base/vfx/cartoon-2d/jump-dust` (Jump Dust), `base/vfx/cartoon-2d/land-dust` (Land Dust), `base/vfx/cartoon-2d/run-dust` (Run Dust), `base/vfx/cartoon-2d/skid` (Skid Dust), `base/vfx/cartoon-2d/teleport` (Teleport)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/dash` (Dash Lines), `base/vfx/cartoon-2d-rubber-hose/double-jump` (Air Jump), `base/vfx/cartoon-2d-rubber-hose/jump-dust` (Jump Dust), `base/vfx/cartoon-2d-rubber-hose/land-dust` (Land Dust), `base/vfx/cartoon-2d-rubber-hose/run-dust` (Run Dust), `base/vfx/cartoon-2d-rubber-hose/skid` (Skid Dust), `base/vfx/cartoon-2d-rubber-hose/teleport` (Teleport)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/dash` (Dash Lines), `base/vfx/cartoon-2d-storybook/double-jump` (Air Jump), `base/vfx/cartoon-2d-storybook/jump-dust` (Jump Dust), `base/vfx/cartoon-2d-storybook/land-dust` (Land Dust), `base/vfx/cartoon-2d-storybook/run-dust` (Run Dust), `base/vfx/cartoon-2d-storybook/skid` (Skid Dust), `base/vfx/cartoon-2d-storybook/teleport` (Teleport)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/dash` (Dash Lines), `base/vfx/cartoon-2d-watercolor/double-jump` (Air Jump), `base/vfx/cartoon-2d-watercolor/jump-dust` (Jump Dust), `base/vfx/cartoon-2d-watercolor/land-dust` (Land Dust), `base/vfx/cartoon-2d-watercolor/run-dust` (Run Dust), `base/vfx/cartoon-2d-watercolor/skid` (Skid Dust), `base/vfx/cartoon-2d-watercolor/teleport` (Teleport)
- cinematic-realism: `base/vfx/cinematic-realism/dash` (Dash), `base/vfx/cinematic-realism/footstep-dust` (Footstep Dust), `base/vfx/cinematic-realism/landing` (Landing Dust)

<a id="guide-effect-packs-37"></a>
### `effect-packs §37` Pickup effects

Pickup effects (pickup): 20 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`.

- cartoon-2d: `base/vfx/cartoon-2d/chest-open` (Treasure), `base/vfx/cartoon-2d/coin` (Coin Grab), `base/vfx/cartoon-2d/gem` (Gem Grab), `base/vfx/cartoon-2d/heart` (Heart Grab), `base/vfx/cartoon-2d/star-collect` (Star Grab)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/chest-open` (Treasure), `base/vfx/cartoon-2d-rubber-hose/coin` (Coin Grab), `base/vfx/cartoon-2d-rubber-hose/gem` (Gem Grab), `base/vfx/cartoon-2d-rubber-hose/heart` (Heart Grab), `base/vfx/cartoon-2d-rubber-hose/star-collect` (Star Grab)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/chest-open` (Treasure), `base/vfx/cartoon-2d-storybook/coin` (Coin Grab), `base/vfx/cartoon-2d-storybook/gem` (Gem Grab), `base/vfx/cartoon-2d-storybook/heart` (Heart Grab), `base/vfx/cartoon-2d-storybook/star-collect` (Star Grab)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/chest-open` (Treasure), `base/vfx/cartoon-2d-watercolor/coin` (Coin Grab), `base/vfx/cartoon-2d-watercolor/gem` (Gem Grab), `base/vfx/cartoon-2d-watercolor/heart` (Heart Grab), `base/vfx/cartoon-2d-watercolor/star-collect` (Star Grab)

<a id="guide-effect-packs-38"></a>
### `effect-packs §38` Portal effects

Portal effects (portal): 11 in `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `pixel-scenario`.

- cinematic-realism: `base/vfx/cinematic-realism/portal` (Portal), `base/vfx/cinematic-realism/teleport` (Arrival Flash)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/magic_circle` (Magic Circle)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/MagicArea` (Magic Area)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/GateOfCalve` (Gate Of Calve)
- effekseer-starter: `base/vfx/effekseer-starter/Magic2_fire` (Fire Pentagram), `base/vfx/effekseer-starter/magic_circle` (Magic Circle), `base/vfx/effekseer-starter/smallTexturesSquareWave_orbital` (Arcane Ring)
- pixel-scenario: `base/vfx/pixel-scenario/dimensional-portal` (Astral Portal), `base/vfx/pixel-scenario/purple-black-hole-vortex` (Shadow Vortex), `base/vfx/pixel-scenario/teleport-blink` (Mystic Teleport)

<a id="guide-effect-packs-39"></a>
### `effect-packs §39` Projectile effects

Projectile effects (projectile): 8 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `base/vfx/cartoon-2d/bullet` (Pellet)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/bullet` (Pellet)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/bullet` (Pellet)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/bullet` (Pellet)
- cinematic-realism: `base/vfx/cinematic-realism/bolt` (Energy Bolt), `base/vfx/cinematic-realism/grenade-trail` (Grenade Smoke Thread), `base/vfx/cinematic-realism/rocket` (Rocket Motor), `base/vfx/cinematic-realism/tracer` (Tracer Round)

<a id="guide-effect-packs-40"></a>
### `effect-packs §40` Shield effects

Shield effects (shield): 13 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`.

- cartoon-2d: `base/vfx/cartoon-2d/block` (Block), `base/vfx/cartoon-2d/shield` (Shield Bubble)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/block` (Block), `base/vfx/cartoon-2d-rubber-hose/shield` (Shield Bubble)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/block` (Block), `base/vfx/cartoon-2d-storybook/shield` (Shield Bubble)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/block` (Block), `base/vfx/cartoon-2d-watercolor/shield` (Shield Bubble)
- cinematic-realism: `base/vfx/cinematic-realism/barrier` (Barrier Dome), `base/vfx/cinematic-realism/shield-hit` (Shield Ripple)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/Gohlem_Shield1` (Gohlem Shield 1), `base/vfx/effekseer-magicalxspiral/Gohlem_Shield2` (Gohlem Shield 2)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicShield` (Magic Shield)

<a id="guide-effect-packs-41"></a>
### `effect-packs §41` Slash effects

Slash effects (slash): 53 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/slash` (Swoosh Slash)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/slash` (Swoosh Slash)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/slash` (Swoosh Slash)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/slash` (Swoosh Slash)
- cinematic-realism: `base/vfx/cinematic-realism/slash` (Blade Arc)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/Attack1` (Attack 1), `base/vfx/effekseer-magicalxspiral/Attack2` (Attack 2), `base/vfx/effekseer-magicalxspiral/Attack5` (Attack 5), `base/vfx/effekseer-magicalxspiral/Attack6` (Attack 6)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicSword` (Magic Sword), `base/vfx/effekseer-nextsoft01/Sword1` (Sword 1), `base/vfx/effekseer-nextsoft01/Sword2` (Sword 2)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/Sword_Ember` (Sword Ember), `base/vfx/effekseer-pierre02/Sword_Lightning` (Sword Lightning), `base/vfx/effekseer-pierre02/Sword_Moonlight` (Sword Moonlight), `base/vfx/effekseer-pierre02/Sword_Sacredlight` (Sword Sacredlight), `base/vfx/effekseer-pierre02/Sword_Storm` (Sword Storm)
- effekseer-starter: `base/vfx/effekseer-starter/Claw3` (Claw), `base/vfx/effekseer-starter/Sword5` (Blue Slash), `base/vfx/effekseer-starter/Sword7` (Cyan Cross Slash), `base/vfx/effekseer-starter/Sword8` (Red Arc Slash)
- effekseer-suzuki01: `base/vfx/effekseer-suzuki01/sword` (Sword)
- effekseer-tktk01: `base/vfx/effekseer-tktk01/Sword1` (Sword 1), `base/vfx/effekseer-tktk01/Sword10` (Sword 10), `base/vfx/effekseer-tktk01/Sword11` (Sword 11), `base/vfx/effekseer-tktk01/Sword12` (Sword 12), `base/vfx/effekseer-tktk01/Sword13` (Sword 13), `base/vfx/effekseer-tktk01/Sword14` (Sword 14), `base/vfx/effekseer-tktk01/Sword15` (Sword 15), `base/vfx/effekseer-tktk01/Sword16` (Sword 16), `base/vfx/effekseer-tktk01/Sword17` (Sword 17), `base/vfx/effekseer-tktk01/Sword18` (Sword 18), `base/vfx/effekseer-tktk01/Sword19` (Sword 19), `base/vfx/effekseer-tktk01/Sword2` (Sword 2), `base/vfx/effekseer-tktk01/Sword20` (Sword 20), `base/vfx/effekseer-tktk01/Sword21` (Sword 21), `base/vfx/effekseer-tktk01/Sword22` (Sword 22), `base/vfx/effekseer-tktk01/Sword3` (Sword 3), `base/vfx/effekseer-tktk01/Sword4` (Sword 4), `base/vfx/effekseer-tktk01/Sword5` (Sword 5), `base/vfx/effekseer-tktk01/Sword6` (Sword 6), `base/vfx/effekseer-tktk01/Sword7` (Sword 7), `base/vfx/effekseer-tktk01/Sword8` (Sword 8), `base/vfx/effekseer-tktk01/Sword9` (Sword 9)
- effekseer-tktk02: `base/vfx/effekseer-tktk02/Claw1` (Claw 1), `base/vfx/effekseer-tktk02/Claw2` (Claw 2), `base/vfx/effekseer-tktk02/Claw3` (Claw 3), `base/vfx/effekseer-tktk02/Claw4` (Claw 4)
- pixel-scenario: `base/vfx/pixel-scenario/crescent-sword-slash` (Razor Slash), `base/vfx/pixel-scenario/dash-speed-lines` (Cyclone Dash), `base/vfx/pixel-scenario/feral-claw-scratch-blood` (Sanguine Claw), `base/vfx/pixel-scenario/spinning-wind-slash` (Cyclone Slash), `base/vfx/pixel-scenario/triple-slash-combo-with-blue-energy` (Flux Slash)

<a id="guide-effect-packs-42"></a>
### `effect-packs §42` Smoke effects

Smoke effects (smoke): 25 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `geometric`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/chimney-smoke` (Chimney Smoke), `base/vfx/cartoon-2d/poof` (Poof)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/chimney-smoke` (Chimney Smoke), `base/vfx/cartoon-2d-rubber-hose/poof` (Poof)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/chimney-smoke` (Chimney Smoke), `base/vfx/cartoon-2d-storybook/poof` (Poof)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/chimney-smoke` (Chimney Smoke), `base/vfx/cartoon-2d-watercolor/poof` (Poof)
- cinematic-realism: `base/vfx/cinematic-realism/dust` (Rolling Dust), `base/vfx/cinematic-realism/smoke-grenade` (Smoke Screen), `base/vfx/cinematic-realism/smoke-plume` (Wreck Smoke Column), `base/vfx/cinematic-realism/steam` (Steam Leak)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/electric_dust` (Electric Dust)
- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/foot_smoke1` (Foot Smoke 1), `base/vfx/effekseer-magicalxspiral/foot_smoke2` (Foot Smoke 2), `base/vfx/effekseer-magicalxspiral/foot_smoke3` (Foot Smoke 3), `base/vfx/effekseer-magicalxspiral/foot_smoke4` (Foot Smoke 4), `base/vfx/effekseer-magicalxspiral/foot_smoke5` (Foot Smoke 5), `base/vfx/effekseer-magicalxspiral/foot_smoke6` (Foot Smoke 6), `base/vfx/effekseer-magicalxspiral/foot_smoke7` (Foot Smoke 7), `base/vfx/effekseer-magicalxspiral/foot_smoke8` (Foot Smoke 8)
- effekseer-pierre02: `base/vfx/effekseer-pierre02/CosmicMist` (Cosmic Mist)
- geometric: `base/vfx/geometric/dust` (Slate Dust)
- pixel-scenario: `base/vfx/pixel-scenario/poison-gas-cloud` (Blight Cloud), `base/vfx/pixel-scenario/smoke-poof` (Ashen Poof)

<a id="guide-effect-packs-43"></a>
### `effect-packs §43` Splash effects

Splash effects (splash): 20 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `geometric`, `realistic-starter`.

- cartoon-2d: `base/vfx/cartoon-2d/ripple` (Ripples), `base/vfx/cartoon-2d/splash-small` (Plop), `base/vfx/cartoon-2d/splash` (Splash)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/ripple` (Ripples), `base/vfx/cartoon-2d-rubber-hose/splash-small` (Plop), `base/vfx/cartoon-2d-rubber-hose/splash` (Splash)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/ripple` (Ripples), `base/vfx/cartoon-2d-storybook/splash-small` (Plop), `base/vfx/cartoon-2d-storybook/splash` (Splash)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/ripple` (Ripples), `base/vfx/cartoon-2d-watercolor/splash-small` (Plop), `base/vfx/cartoon-2d-watercolor/splash` (Splash)
- cinematic-realism: `base/vfx/cinematic-realism/drip` (Drip), `base/vfx/cinematic-realism/footstep-water` (Wading Step), `base/vfx/cinematic-realism/fountain` (Fountain Jet), `base/vfx/cinematic-realism/splash-small` (Small Splash), `base/vfx/cinematic-realism/splash` (Ballistic Spray), `base/vfx/cinematic-realism/waterfall-mist` (Waterfall Mist)
- geometric: `base/vfx/geometric/splash` (Facet Splash)
- realistic-starter: `base/vfx/realistic-starter/water-splash` (Water Splash)

<a id="guide-effect-packs-44"></a>
### `effect-packs §44` Status effects

Status effects (status): 30 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `base/vfx/cartoon-2d/burning` (On Fire), `base/vfx/cartoon-2d/debuff` (Debuff), `base/vfx/cartoon-2d/frozen` (Frozen), `base/vfx/cartoon-2d/poison` (Poisoned), `base/vfx/cartoon-2d/shocked` (Zapped), `base/vfx/cartoon-2d/stun` (Dizzy Stars)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/burning` (On Fire), `base/vfx/cartoon-2d-rubber-hose/debuff` (Debuff), `base/vfx/cartoon-2d-rubber-hose/frozen` (Frozen), `base/vfx/cartoon-2d-rubber-hose/poison` (Poisoned), `base/vfx/cartoon-2d-rubber-hose/shocked` (Zapped), `base/vfx/cartoon-2d-rubber-hose/stun` (Dizzy Stars)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/burning` (On Fire), `base/vfx/cartoon-2d-storybook/debuff` (Debuff), `base/vfx/cartoon-2d-storybook/frozen` (Frozen), `base/vfx/cartoon-2d-storybook/poison` (Poisoned), `base/vfx/cartoon-2d-storybook/shocked` (Zapped), `base/vfx/cartoon-2d-storybook/stun` (Dizzy Stars)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/burning` (On Fire), `base/vfx/cartoon-2d-watercolor/debuff` (Debuff), `base/vfx/cartoon-2d-watercolor/frozen` (Frozen), `base/vfx/cartoon-2d-watercolor/poison` (Poisoned), `base/vfx/cartoon-2d-watercolor/shocked` (Zapped), `base/vfx/cartoon-2d-watercolor/stun` (Dizzy Stars)
- cinematic-realism: `base/vfx/cinematic-realism/burning` (Burning), `base/vfx/cinematic-realism/debuff` (Sinking Strands), `base/vfx/cinematic-realism/dissolve` (Ash Dissolve), `base/vfx/cinematic-realism/frozen` (Frozen), `base/vfx/cinematic-realism/poison` (Poisoned), `base/vfx/cinematic-realism/shocked` (Shocked)

<a id="guide-effect-packs-45"></a>
### `effect-packs §45` Ui effects

Ui effects (ui): 26 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `geometric`, `pixel-scenario`.

- cartoon-2d: `base/vfx/cartoon-2d/confetti-rain` (Confetti Rain), `base/vfx/cartoon-2d/confetti` (Confetti Pop), `base/vfx/cartoon-2d/fireworks` (Firework)
- cartoon-2d-rubber-hose: `base/vfx/cartoon-2d-rubber-hose/confetti-rain` (Confetti Rain), `base/vfx/cartoon-2d-rubber-hose/confetti` (Confetti Pop), `base/vfx/cartoon-2d-rubber-hose/fireworks` (Firework)
- cartoon-2d-storybook: `base/vfx/cartoon-2d-storybook/confetti-rain` (Confetti Rain), `base/vfx/cartoon-2d-storybook/confetti` (Confetti Pop), `base/vfx/cartoon-2d-storybook/fireworks` (Firework)
- cartoon-2d-watercolor: `base/vfx/cartoon-2d-watercolor/confetti-rain` (Confetti Rain), `base/vfx/cartoon-2d-watercolor/confetti` (Confetti Pop), `base/vfx/cartoon-2d-watercolor/fireworks` (Firework)
- cinematic-realism: `base/vfx/cinematic-realism/celebration` (Satin Confetti), `base/vfx/cinematic-realism/fireworks` (Firework), `base/vfx/cinematic-realism/reward-burst` (Reward Fountain)
- effekseer-andrewfm01: `base/vfx/effekseer-andrewfm01/actionLines_halo` (Action Lines Halo), `base/vfx/effekseer-andrewfm01/actionLines_halo2` (Action Lines Halo 2), `base/vfx/effekseer-andrewfm01/actionLines_radial` (Action Lines Radial), `base/vfx/effekseer-andrewfm01/actionLines_radial2` (Action Lines Radial 2), `base/vfx/effekseer-andrewfm01/actionLines_shockwave` (Action Lines Shockwave), `base/vfx/effekseer-andrewfm01/actionLines_strip` (Action Lines Strip)
- geometric: `base/vfx/geometric/celebration` (Coral Confetti)
- pixel-scenario: `base/vfx/pixel-scenario/critical-hit-damage-999` (Ember Damage Number), `base/vfx/pixel-scenario/fireworks-celebration` (Arcane Fireworks), `base/vfx/pixel-scenario/gold-coins-loot-drop` (Lucky Loot), `base/vfx/pixel-scenario/sparkle-twinkle` (Sacred Sparkle)

<a id="guide-effect-packs-46"></a>
### `effect-packs §46` Vehicle effects

Vehicle effects (vehicle): 11 in `cinematic-realism`.

- cinematic-realism: `base/vfx/cinematic-realism/afterburner` (Afterburner), `base/vfx/cinematic-realism/boost` (Booster Burst), `base/vfx/cinematic-realism/car-exhaust` (Road Exhaust), `base/vfx/cinematic-realism/damage-smoke` (Damage Smoke), `base/vfx/cinematic-realism/downwash` (Rotor Downwash), `base/vfx/cinematic-realism/exhaust` (Ion Exhaust), `base/vfx/cinematic-realism/thruster` (Attitude Jet), `base/vfx/cinematic-realism/tire-smoke` (Tyre Smoke), `base/vfx/cinematic-realism/wake` (Hull Wake), `base/vfx/cinematic-realism/warp` (Jump Flash), `base/vfx/cinematic-realism/wheel-dust` (Wheel Dust)

<a id="guide-effect-packs-47"></a>
### `effect-packs §47` Weather effects

Weather effects (weather): 7 in `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `pixel-scenario`.

- effekseer-magicalxspiral: `base/vfx/effekseer-magicalxspiral/Snowman1` (Snowman 1)
- effekseer-nextsoft01: `base/vfx/effekseer-nextsoft01/MagicTornade` (Magic Tornade)
- effekseer-pierre01: `base/vfx/effekseer-pierre01/HolySandstorm` (Holy Sandstorm)
- effekseer-starter: `base/vfx/effekseer-starter/HolySandstorm` (Sandstorm)
- effekseer-suzuki01: `base/vfx/effekseer-suzuki01/snowstorm11` (Snowstorm 11)
- pixel-scenario: `base/vfx/pixel-scenario/rain` (Aqua Ambient), `base/vfx/pixel-scenario/snow-falling` (Glacial Ambient)
