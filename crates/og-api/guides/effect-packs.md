<!-- Generated from the vfx packs' extension.json, assets.json and preview/*.cost.json (packages/engine/extensions/, packages/vfx-packs/) by `og api gen`. Do not edit: `og api check` fails on any difference. -->
# The effect packs

Ready-made effects a game installs with `og vfx add <pack>`: converted Effekseer samples (`type vfx`, 3D) and checked pixel-art recipes (`file.vfx2d`, 2D), each listed with its tags, keywords and effects. A bundled pack ships with the engine; a downloadable one comes with each release and `og vfx add` fetches it. Pixel effects install under content/ and load by `base/vfx/<pack>/<effect>`; 3D effects retain their asset paths (`og api show vfx`; the `pixel-vfx` guide for `file.vfx2d`). The pack format is docs/vfx-packs.md.

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

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/pixel-scenario/content/arcane-magic-circle.ts` | Eldritch Magic Circle | magic-impact | arcane | loops |  |  |  |  |
| `vfx/pixel-scenario/content/big-fire-explosion.ts` | Cinder Explosion | explosion | fire | 1.2 |  |  |  |  |
| `vfx/pixel-scenario/content/blue-energy-beam.ts` | Aether Beam | magic-projectile | arcane | loops |  |  |  |  |
| `vfx/pixel-scenario/content/campfire.ts` | Ember Flame | fire | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/charge-up-energy.ts` | Pulse Charge | buff | arcane | loops |  |  |  |  |
| `vfx/pixel-scenario/content/cherry-blossom-petals.ts` | Thorn Ambient | ambient | nature | loops |  |  |  |  |
| `vfx/pixel-scenario/content/crescent-sword-slash.ts` | Razor Slash | slash |  | 0.667 |  |  |  |  |
| `vfx/pixel-scenario/content/critical-hit-damage-999.ts` | Ember Damage Number | ui |  | 1.067 |  |  |  |  |
| `vfx/pixel-scenario/content/dark-shadow-aura.ts` | Night Aura | buff | dark | loops |  |  |  |  |
| `vfx/pixel-scenario/content/dash-speed-lines.ts` | Cyclone Dash | slash | wind | 0.556 |  |  |  |  |
| `vfx/pixel-scenario/content/dimensional-portal.ts` | Astral Portal | portal | arcane | loops |  |  |  |  |
| `vfx/pixel-scenario/content/dragon-fire-breath.ts` | Cinder Breath | fire | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/electric-impact-hit.ts` | Static Impact | hit-spark | lightning | 0.556 |  |  |  |  |
| `vfx/pixel-scenario/content/embers-rising.ts` | Blaze Ambient | ambient | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/falling-leaves.ts` | Bloom Ambient | ambient | nature | loops |  |  |  |  |
| `vfx/pixel-scenario/content/feral-claw-scratch-blood.ts` | Sanguine Claw | slash |  | 0.667 |  |  |  |  |
| `vfx/pixel-scenario/content/fire-tornado.ts` | Blaze Tornado | fire | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/fireball-projectile.ts` | Ember Projectile | magic-projectile | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/fireflies.ts` | Mystic Ambient | ambient | fire | loops |  |  |  |  |
| `vfx/pixel-scenario/content/fireworks-celebration.ts` | Arcane Fireworks | ui | fire | 2.133 |  |  |  |  |
| `vfx/pixel-scenario/content/frost-nova.ts` | Frost Nova | magic-impact | ice | 1.067 |  |  |  |  |
| `vfx/pixel-scenario/content/gold-coins-loot-drop.ts` | Lucky Loot | ui |  | 1.6 |  |  |  |  |
| `vfx/pixel-scenario/content/ground-slam-quake.ts` | Stone Ground Slam | magic-impact | earth | 1.2 |  |  |  |  |
| `vfx/pixel-scenario/content/healing-with-rising-crosses.ts` | Bloom Heal | heal | holy | 1.333 |  |  |  |  |
| `vfx/pixel-scenario/content/hex-shield-barrier.ts` | Flux Shield | buff | arcane | loops |  |  |  |  |
| `vfx/pixel-scenario/content/holy-pillar-of-light.ts` | Divine Pillar | magic-impact | holy | 1.333 |  |  |  |  |
| `vfx/pixel-scenario/content/ice-spikes-erupting.ts` | Frozen Spikes | magic-impact | ice | 1.2 |  |  |  |  |
| `vfx/pixel-scenario/content/lava-eruption-pillar.ts` | Volcanic Pillar | magic-impact | fire | 1.333 |  |  |  |  |
| `vfx/pixel-scenario/content/level-up.ts` | Gilded Level Up | level-up |  | 1.6 |  |  |  |  |
| `vfx/pixel-scenario/content/lightning-strike-from-the-sky.ts` | Storm Strike | magic-impact | lightning | 0.889 |  |  |  |  |
| `vfx/pixel-scenario/content/meteor-strike.ts` | Ember Meteor | explosion | fire | 1.733 |  |  |  |  |
| `vfx/pixel-scenario/content/plasma-orb.ts` | Ion Orb | magic-projectile | lightning | loops |  |  |  |  |
| `vfx/pixel-scenario/content/poison-gas-cloud.ts` | Blight Cloud | smoke | nature | loops |  |  |  |  |
| `vfx/pixel-scenario/content/purple-black-hole-vortex.ts` | Shadow Vortex | portal | dark | loops |  |  |  |  |
| `vfx/pixel-scenario/content/rain.ts` | Aqua Ambient | weather | water | loops |  |  |  |  |
| `vfx/pixel-scenario/content/smoke-poof.ts` | Ashen Poof | smoke |  | 0.933 |  |  |  |  |
| `vfx/pixel-scenario/content/snow-falling.ts` | Glacial Ambient | weather | ice | loops |  |  |  |  |
| `vfx/pixel-scenario/content/sparkle-twinkle.ts` | Sacred Sparkle | ui |  | loops |  |  |  |  |
| `vfx/pixel-scenario/content/spinning-wind-slash.ts` | Cyclone Slash | slash | wind | 0.667 |  |  |  |  |
| `vfx/pixel-scenario/content/teleport-blink.ts` | Mystic Teleport | portal | arcane | 1.333 |  |  |  |  |
| `vfx/pixel-scenario/content/triple-slash-combo-with-blue-energy.ts` | Flux Slash | slash | arcane | 1.167 |  |  |  |  |
| `vfx/pixel-scenario/content/water-splash.ts` | Aqua Splash | magic-impact | water | 1.067 |  |  |  |  |

## 3. realistic-starter

A starter set of realistic effects for 3D games: a muzzle flash, a bullet impact, an explosion, the dust of something breaking, a water splash, a torch's fire and dust in the air, built on the Kenney Particle Pack's textures. 7 effects, bundled with the engine: `og vfx add realistic-starter`.

- Style: realistic, 3d
- Categories: ambient, debris, explosion, fire, gunfire, hit-spark, splash
- Elements: earth, fire, water
- Keywords: 3d, debris, dust, gunfire, particles, realistic, shooter, smoke, starter

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/realistic-starter/ambient-dust.ogt` | Ambient Dust | ambient |  | loops |  |  |  |  |
| `vfx/realistic-starter/bullet-impact.ogt` | Bullet Impact | hit-spark |  | 1.8 |  |  |  |  |
| `vfx/realistic-starter/dust-burst.ogt` | Dust Burst | debris | earth | 3.6 |  |  |  |  |
| `vfx/realistic-starter/explosion.ogt` | Explosion | explosion | fire | 4 |  |  |  |  |
| `vfx/realistic-starter/muzzle-flash.ogt` | Muzzle Flash | gunfire | fire | 1.2 |  |  |  |  |
| `vfx/realistic-starter/torch-fire.ogt` | Torch Fire | fire | fire | loops |  |  |  |  |
| `vfx/realistic-starter/water-splash.ogt` | Water Splash | splash | water | 1.6 |  |  |  |  |

## 4. cartoon-2d

Flat cartoon effects for cartoon-2d: clean shapes in two to four flat tones with a 2 px dark warm contour, drawn smooth. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/cartoon-2d/content/block.ts` | Block | shield |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d/content/break-crate.ts` | Crate Break | destruction | nature | 1 |  |  |  |  |
| `vfx/cartoon-2d/content/break-ice.ts` | Ice Shatter | destruction | ice | 0.875 |  |  |  |  |
| `vfx/cartoon-2d/content/break-rock.ts` | Rock Break | destruction | earth | 1 |  |  |  |  |
| `vfx/cartoon-2d/content/bubbles.ts` | Bubbles | ambient | water | loops |  |  |  |  |
| `vfx/cartoon-2d/content/buff.ts` | Buff | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d/content/bullet-hit.ts` | Pellet Pop | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d/content/bullet.ts` | Pellet | projectile |  | loops |  |  |  |  |
| `vfx/cartoon-2d/content/burning.ts` | On Fire | status | fire | loops |  |  |  |  |
| `vfx/cartoon-2d/content/campfire.ts` | Campfire | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d/content/charge.ts` | Charge Ball | magic-impact | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d/content/checkpoint.ts` | Checkpoint | marker |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d/content/chest-open.ts` | Treasure | pickup |  | 1.2 |  |  |  |  |
| `vfx/cartoon-2d/content/chimney-smoke.ts` | Chimney Smoke | smoke |  | loops |  |  |  |  |
| `vfx/cartoon-2d/content/coin.ts` | Coin Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/confetti-rain.ts` | Confetti Rain | ui |  | loops |  |  |  |  |
| `vfx/cartoon-2d/content/confetti.ts` | Confetti Pop | ui |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d/content/dash.ts` | Dash Lines | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d/content/debuff.ts` | Debuff | status | dark | 1.067 |  |  |  |  |
| `vfx/cartoon-2d/content/double-jump.ts` | Air Jump | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d/content/explosion-small.ts` | Pop Boom | explosion | fire | 0.875 |  |  |  |  |
| `vfx/cartoon-2d/content/explosion.ts` | Boom | explosion | fire | 1.2 |  |  |  |  |
| `vfx/cartoon-2d/content/fireflies.ts` | Fireflies | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d/content/fireworks.ts` | Firework | ui | fire | 1.467 |  |  |  |  |
| `vfx/cartoon-2d/content/frozen.ts` | Frozen | status | ice | 0.933 |  |  |  |  |
| `vfx/cartoon-2d/content/fuse.ts` | Fuse Sparks | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d/content/gem.ts` | Gem Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/heal.ts` | Heal | heal | nature | 1.2 |  |  |  |  |
| `vfx/cartoon-2d/content/heart.ts` | Heart Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/hit-heavy.ts` | Smash Hit | hit-spark |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/hit-small.ts` | Tap Hit | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d/content/hit.ts` | Pow Hit | hit-spark |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d/content/jump-dust.ts` | Jump Dust | movement | earth | 0.556 |  |  |  |  |
| `vfx/cartoon-2d/content/land-dust.ts` | Land Dust | movement | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/laser.ts` | Ray Beam | beam | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d/content/leaves.ts` | Falling Leaves | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d/content/magic-bolt.ts` | Magic Bolt | magic-projectile | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d/content/magic-circle.ts` | Spell Circle | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d/content/magic.ts` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  |  |  |
| `vfx/cartoon-2d/content/muzzle.ts` | Pop Muzzle | gunfire | fire | 0.3 |  |  |  |  |
| `vfx/cartoon-2d/content/poison.ts` | Poisoned | status | poison | loops |  |  |  |  |
| `vfx/cartoon-2d/content/poof.ts` | Poof | smoke |  | 0.875 |  |  |  |  |
| `vfx/cartoon-2d/content/portal.ts` | Portal | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d/content/powerup.ts` | Power Up | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d/content/ripple.ts` | Ripples | splash | water | loops |  |  |  |  |
| `vfx/cartoon-2d/content/run-dust.ts` | Run Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d/content/shield.ts` | Shield Bubble | shield | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d/content/shocked.ts` | Zapped | status | lightning | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/skid.ts` | Skid Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d/content/slash.ts` | Swoosh Slash | slash |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/splash-small.ts` | Plop | splash | water | 0.75 |  |  |  |  |
| `vfx/cartoon-2d/content/splash.ts` | Splash | splash | water | 0.875 |  |  |  |  |
| `vfx/cartoon-2d/content/star-collect.ts` | Star Grab | pickup |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d/content/stomp.ts` | Stomp | hit-spark | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d/content/stun.ts` | Dizzy Stars | status |  | loops |  |  |  |  |
| `vfx/cartoon-2d/content/teleport.ts` | Teleport | movement | arcane | 1.2 |  |  |  |  |
| `vfx/cartoon-2d/content/torch.ts` | Torch Flame | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d/content/twinkle.ts` | Twinkle | ambient |  | loops |  |  |  |  |
| `vfx/cartoon-2d/content/whirlwind.ts` | Whirlwind | magic-impact | wind | loops |  |  |  |  |

## 5. cartoon-2d-rubber-hose

Rubber-hose effects for cartoon-2d's rubber-hose variant: muted primaries in a faded warm grade inked with a 3 px black line. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-rubber-hose`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, rubber-hose

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/cartoon-2d-rubber-hose/content/block.ts` | Block | shield |  | 0.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/break-crate.ts` | Crate Break | destruction | nature | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/break-ice.ts` | Ice Shatter | destruction | ice | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/break-rock.ts` | Rock Break | destruction | earth | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/bubbles.ts` | Bubbles | ambient | water | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/buff.ts` | Buff | buff |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/bullet-hit.ts` | Pellet Pop | hit-spark |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/bullet.ts` | Pellet | projectile |  | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/burning.ts` | On Fire | status | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/campfire.ts` | Campfire | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/charge.ts` | Charge Ball | magic-impact | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/checkpoint.ts` | Checkpoint | marker |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/chest-open.ts` | Treasure | pickup |  | 1.5 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/chimney-smoke.ts` | Chimney Smoke | smoke |  | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/coin.ts` | Coin Grab | pickup |  | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/confetti-rain.ts` | Confetti Rain | ui |  | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/confetti.ts` | Confetti Pop | ui |  | 1.667 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/dash.ts` | Dash Lines | movement | wind | 0.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/debuff.ts` | Debuff | status | dark | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/double-jump.ts` | Air Jump | movement | wind | 0.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/explosion-small.ts` | Pop Boom | explosion | fire | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/explosion.ts` | Boom | explosion | fire | 1.5 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/fireflies.ts` | Fireflies | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/fireworks.ts` | Firework | ui | fire | 1.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/frozen.ts` | Frozen | status | ice | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/fuse.ts` | Fuse Sparks | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/gem.ts` | Gem Grab | pickup |  | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/heal.ts` | Heal | heal | nature | 1.5 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/heart.ts` | Heart Grab | pickup |  | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/hit-heavy.ts` | Smash Hit | hit-spark |  | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/hit-small.ts` | Tap Hit | hit-spark |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/hit.ts` | Pow Hit | hit-spark |  | 0.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/jump-dust.ts` | Jump Dust | movement | earth | 0.833 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/land-dust.ts` | Land Dust | movement | earth | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/laser.ts` | Ray Beam | beam | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/leaves.ts` | Falling Leaves | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/magic-bolt.ts` | Magic Bolt | magic-projectile | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/magic-circle.ts` | Spell Circle | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/magic.ts` | Sparkle Cast | magic-impact | arcane | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/muzzle.ts` | Pop Muzzle | gunfire | fire | 0.5 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/poison.ts` | Poisoned | status | poison | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/poof.ts` | Poof | smoke |  | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/portal.ts` | Portal | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/powerup.ts` | Power Up | buff |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/ripple.ts` | Ripples | splash | water | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/run-dust.ts` | Run Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/shield.ts` | Shield Bubble | shield | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/shocked.ts` | Zapped | status | lightning | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/skid.ts` | Skid Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/slash.ts` | Swoosh Slash | slash |  | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/splash-small.ts` | Plop | splash | water | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/splash.ts` | Splash | splash | water | 1.167 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/star-collect.ts` | Star Grab | pickup |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/stomp.ts` | Stomp | hit-spark | earth | 1 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/stun.ts` | Dizzy Stars | status |  | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/teleport.ts` | Teleport | movement | arcane | 1.5 |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/torch.ts` | Torch Flame | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/twinkle.ts` | Twinkle | ambient |  | loops |  |  |  |  |
| `vfx/cartoon-2d-rubber-hose/content/whirlwind.ts` | Whirlwind | magic-impact | wind | loops |  |  |  |  |

## 6. cartoon-2d-storybook

Storybook effects for cartoon-2d's storybook variant: soft saturated shapes with a thin coloured line a shade darker than the fill. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-storybook`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, storybook

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/cartoon-2d-storybook/content/block.ts` | Block | shield |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/break-crate.ts` | Crate Break | destruction | nature | 1 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/break-ice.ts` | Ice Shatter | destruction | ice | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/break-rock.ts` | Rock Break | destruction | earth | 1 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/bubbles.ts` | Bubbles | ambient | water | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/buff.ts` | Buff | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/bullet-hit.ts` | Pellet Pop | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/bullet.ts` | Pellet | projectile |  | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/burning.ts` | On Fire | status | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/campfire.ts` | Campfire | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/charge.ts` | Charge Ball | magic-impact | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/checkpoint.ts` | Checkpoint | marker |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/chest-open.ts` | Treasure | pickup |  | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/chimney-smoke.ts` | Chimney Smoke | smoke |  | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/coin.ts` | Coin Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/confetti-rain.ts` | Confetti Rain | ui |  | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/confetti.ts` | Confetti Pop | ui |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/dash.ts` | Dash Lines | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/debuff.ts` | Debuff | status | dark | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/double-jump.ts` | Air Jump | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/explosion-small.ts` | Pop Boom | explosion | fire | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/explosion.ts` | Boom | explosion | fire | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/fireflies.ts` | Fireflies | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/fireworks.ts` | Firework | ui | fire | 1.467 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/frozen.ts` | Frozen | status | ice | 0.933 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/fuse.ts` | Fuse Sparks | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/gem.ts` | Gem Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/heal.ts` | Heal | heal | nature | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/heart.ts` | Heart Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/hit-heavy.ts` | Smash Hit | hit-spark |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/hit-small.ts` | Tap Hit | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/hit.ts` | Pow Hit | hit-spark |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/jump-dust.ts` | Jump Dust | movement | earth | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/land-dust.ts` | Land Dust | movement | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/laser.ts` | Ray Beam | beam | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/leaves.ts` | Falling Leaves | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/magic-bolt.ts` | Magic Bolt | magic-projectile | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/magic-circle.ts` | Spell Circle | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/magic.ts` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/muzzle.ts` | Pop Muzzle | gunfire | fire | 0.3 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/poison.ts` | Poisoned | status | poison | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/poof.ts` | Poof | smoke |  | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/portal.ts` | Portal | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/powerup.ts` | Power Up | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/ripple.ts` | Ripples | splash | water | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/run-dust.ts` | Run Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/shield.ts` | Shield Bubble | shield | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/shocked.ts` | Zapped | status | lightning | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/skid.ts` | Skid Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/slash.ts` | Swoosh Slash | slash |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/splash-small.ts` | Plop | splash | water | 0.75 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/splash.ts` | Splash | splash | water | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/star-collect.ts` | Star Grab | pickup |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/stomp.ts` | Stomp | hit-spark | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/stun.ts` | Dizzy Stars | status |  | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/teleport.ts` | Teleport | movement | arcane | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/torch.ts` | Torch Flame | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/twinkle.ts` | Twinkle | ambient |  | loops |  |  |  |  |
| `vfx/cartoon-2d-storybook/content/whirlwind.ts` | Whirlwind | magic-impact | wind | loops |  |  |  |  |

## 7. cartoon-2d-watercolor

Watercolour effects for cartoon-2d's watercolor variant: light translucent-looking washes with no contour, a darker pigment rim instead. 59 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cartoon-2d-watercolor`.

- Style: stylised, 2d
- Categories: ambient, beam, buff, destruction, explosion, fire, gunfire, heal, hit-spark, magic-impact, magic-projectile, marker, movement, pickup, projectile, shield, slash, smoke, splash, status, ui
- Elements: arcane, dark, earth, fire, ice, lightning, nature, poison, water, wind
- Keywords: 2d, cartoon, cartoon-2d, flat, watercolor

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/cartoon-2d-watercolor/content/block.ts` | Block | shield |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/break-crate.ts` | Crate Break | destruction | nature | 1 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/break-ice.ts` | Ice Shatter | destruction | ice | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/break-rock.ts` | Rock Break | destruction | earth | 1 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/bubbles.ts` | Bubbles | ambient | water | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/buff.ts` | Buff | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/bullet-hit.ts` | Pellet Pop | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/bullet.ts` | Pellet | projectile |  | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/burning.ts` | On Fire | status | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/campfire.ts` | Campfire | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/charge.ts` | Charge Ball | magic-impact | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/checkpoint.ts` | Checkpoint | marker |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/chest-open.ts` | Treasure | pickup |  | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/chimney-smoke.ts` | Chimney Smoke | smoke |  | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/coin.ts` | Coin Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/confetti-rain.ts` | Confetti Rain | ui |  | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/confetti.ts` | Confetti Pop | ui |  | 1.333 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/dash.ts` | Dash Lines | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/debuff.ts` | Debuff | status | dark | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/double-jump.ts` | Air Jump | movement | wind | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/explosion-small.ts` | Pop Boom | explosion | fire | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/explosion.ts` | Boom | explosion | fire | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/fireflies.ts` | Fireflies | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/fireworks.ts` | Firework | ui | fire | 1.467 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/frozen.ts` | Frozen | status | ice | 0.933 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/fuse.ts` | Fuse Sparks | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/gem.ts` | Gem Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/heal.ts` | Heal | heal | nature | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/heart.ts` | Heart Grab | pickup |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/hit-heavy.ts` | Smash Hit | hit-spark |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/hit-small.ts` | Tap Hit | hit-spark |  | 0.444 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/hit.ts` | Pow Hit | hit-spark |  | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/jump-dust.ts` | Jump Dust | movement | earth | 0.556 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/land-dust.ts` | Land Dust | movement | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/laser.ts` | Ray Beam | beam | lightning | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/leaves.ts` | Falling Leaves | ambient | nature | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/magic-bolt.ts` | Magic Bolt | magic-projectile | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/magic-circle.ts` | Spell Circle | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/magic.ts` | Sparkle Cast | magic-impact | arcane | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/muzzle.ts` | Pop Muzzle | gunfire | fire | 0.3 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/poison.ts` | Poisoned | status | poison | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/poof.ts` | Poof | smoke |  | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/portal.ts` | Portal | magic-impact | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/powerup.ts` | Power Up | buff |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/ripple.ts` | Ripples | splash | water | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/run-dust.ts` | Run Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/shield.ts` | Shield Bubble | shield | arcane | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/shocked.ts` | Zapped | status | lightning | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/skid.ts` | Skid Dust | movement | earth | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/slash.ts` | Swoosh Slash | slash |  | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/splash-small.ts` | Plop | splash | water | 0.75 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/splash.ts` | Splash | splash | water | 0.875 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/star-collect.ts` | Star Grab | pickup |  | 1.067 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/stomp.ts` | Stomp | hit-spark | earth | 0.667 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/stun.ts` | Dizzy Stars | status |  | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/teleport.ts` | Teleport | movement | arcane | 1.2 |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/torch.ts` | Torch Flame | fire | fire | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/twinkle.ts` | Twinkle | ambient |  | loops |  |  |  |  |
| `vfx/cartoon-2d-watercolor/content/whirlwind.ts` | Whirlwind | magic-impact | wind | loops |  |  |  |  |

## 8. cinematic-realism

Restrained cinematic effects: simulated fire and smoke, ballistic sparks, spray, motes and exhaust. Designed for cinematic-realism and muted scenes. 89 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add cinematic-realism`.

- Style: realistic, 3d
- Categories: ambient, beam, buff, decal, destruction, explosion, fire, gunfire, heal, hit-spark, level-up, magic-impact, magic-projectile, marker, movement, portal, projectile, shield, slash, smoke, splash, status, ui, vehicle
- Elements: arcane, dark, earth, fire, ice, lightning, nature, water, wind
- Keywords: ballistics, cinematic-realism, decals, exhaust, muted, particles, realistic, smoke, sound, vehicles

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/cinematic-realism/afterburner.ogt` | Afterburner | vehicle | fire | loops | 0.8 × 3.2 × 0.8 | ground | 0.12 | 15 |
| `vfx/cinematic-realism/barrier.ogt` | Barrier Dome | shield | arcane | loops | 2.6 × 2.6 × 2.6 | ground | 0.06 | 13 |
| `vfx/cinematic-realism/beacon.ogt` | Marker Beacon | marker |  | loops | 0.8 × 9.1 × 0.8 | ground | 0.07 | 14 |
| `vfx/cinematic-realism/bolt.ogt` | Energy Bolt | projectile | lightning | loops | 0.4 × 1.1 × 0.4 | centre | 0.08 | 14 |
| `vfx/cinematic-realism/boost.ogt` | Booster Burst | vehicle | fire | 1.2 | 0.8 × 3.5 × 0.7 | ground | 0.12 | 14 |
| `vfx/cinematic-realism/break-glass.ogt` | Pane Break | destruction |  | 2.5 | 3.8 × 1.5 × 3.5 | ground | 0.07 | 14 |
| `vfx/cinematic-realism/break-stone.ogt` | Masonry Break | destruction | earth | 3.5 | 5.3 × 1.8 × 3.4 | ground | 0.14 | 16 |
| `vfx/cinematic-realism/break-wood.ogt` | Crate Break | destruction | nature | 3 | 5.2 × 1.4 × 3.7 | centre | 0.51 | 16 |
| `vfx/cinematic-realism/buff.ogt` | Rising Strands | buff |  | 1.6 | 0.9 × 1.8 × 0.9 | ground | 0.08 | 14 |
| `vfx/cinematic-realism/burning.ogt` | Burning | status | fire | loops | 0.8 × 2.7 × 0.6 | ground | 0.79 | 16 |
| `vfx/cinematic-realism/campfire.ogt` | Campfire | fire | fire | loops | 0.8 × 1.7 × 0.8 | ground | 0.23 | 19 |
| `vfx/cinematic-realism/car-exhaust.ogt` | Road Exhaust | vehicle |  | loops | 0.2 × 0.7 × 0.3 | ground | 0.18 | 14 |
| `vfx/cinematic-realism/celebration.ogt` | Satin Confetti | ui |  | 3 | 3.0 × 1.2 × 2.9 | ground | 0.07 | 12 |
| `vfx/cinematic-realism/charge.ogt` | Gathering Charge | magic-impact | arcane | loops | 1.6 × 1.5 × 1.6 | centre | 0.09 | 13 |
| `vfx/cinematic-realism/chest-open.ogt` | Opened Chest | marker |  | 1.8 | 0.8 × 1.1 × 0.8 | ground | 0.17 | 17 |
| `vfx/cinematic-realism/damage-smoke.ogt` | Damage Smoke | vehicle | fire | loops | 1.3 × 2.0 × 1.7 | ground | 0.18 | 14 |
| `vfx/cinematic-realism/dash.ogt` | Dash | movement |  | 1 | 0.8 × 3.0 × 0.6 | ground | 0.18 | 16 |
| `vfx/cinematic-realism/debuff.ogt` | Sinking Strands | status | dark | 1.6 | 1.0 × 1.8 × 1.1 | ground | 0.15 | 16 |
| `vfx/cinematic-realism/dissolve.ogt` | Ash Dissolve | status |  | 3.2 | 0.9 × 2.1 × 0.8 | ground | 0.16 | 17 |
| `vfx/cinematic-realism/downwash.ogt` | Rotor Downwash | vehicle | wind | loops | 9.9 × 5.0 × 8.7 | ground | 0.25 | 15 |
| `vfx/cinematic-realism/drip.ogt` | Drip | splash | water | loops | 0.1 × 0.0 × 0.1 | ground | 0.16 | 16 |
| `vfx/cinematic-realism/dust.ogt` | Rolling Dust | smoke | earth | 2.6 | 1.8 × 1.5 × 2.1 | ground | 0.15 | 15 |
| `vfx/cinematic-realism/electric-sparks.ogt` | Arcing Fault | ambient | lightning | loops | 0.9 × 0.5 × 1.3 | ground | 0.08 | 14 |
| `vfx/cinematic-realism/embers.ogt` | Drifting Embers | ambient | fire | loops | 4.4 × 0.9 × 3.9 | ground | 0.04 | 11 |
| `vfx/cinematic-realism/exhaust.ogt` | Ion Exhaust | vehicle |  | loops | 0.4 × 1.3 × 0.4 | centre | 0.08 | 14 |
| `vfx/cinematic-realism/explosion-air.ogt` | Airburst | explosion | fire | 5 | 7.3 × 7.8 × 7.1 | centre | 0.15 | 21 |
| `vfx/cinematic-realism/explosion-energy.ogt` | Energy Blast | explosion | lightning | 6 | 6.4 × 3.0 × 6.4 | ground | 0.61 | 16 |
| `vfx/cinematic-realism/explosion-large.ogt` | Large Blast | explosion | fire | 18 | 23.0 × 12.6 × 34.8 | ground | 1.03 | 21 |
| `vfx/cinematic-realism/explosion-small.ogt` | Grenade Blast | explosion | fire | 12 | 5.8 × 2.2 × 8.1 | ground | 1.07 | 20 |
| `vfx/cinematic-realism/explosion-water.ogt` | Depth Blast | explosion | water | 5 | 13.8 × 9.5 × 13.9 | ground | 0.70 | 24 |
| `vfx/cinematic-realism/explosion.ogt` | Cinematic Explosion | explosion | fire | 14 | 5.9 × 4.7 × 8.1 | ground | 0.99 | 21 |
| `vfx/cinematic-realism/fire-large.ogt` | Large Fire | fire | fire | loops | 3.1 × 5.9 × 2.9 | ground | 1.02 | 24 |
| `vfx/cinematic-realism/fireflies.ogt` | Fireflies | ambient | nature | loops | 3.0 × 0.7 × 2.9 | ground | 0.04 | 11 |
| `vfx/cinematic-realism/fireworks.ogt` | Firework | ui | fire | 3.2 | 12.1 × 14.8 × 11.4 | ground | 0.53 | 15 |
| `vfx/cinematic-realism/flamethrower.ogt` | Flame Stream | fire | fire | loops | 1.8 × 5.8 × 1.7 | ground | 0.23 | 17 |
| `vfx/cinematic-realism/flashbang.ogt` | Flash Charge | explosion |  | 2.5 | 4.6 × 4.2 × 3.8 | centre | 0.15 | 16 |
| `vfx/cinematic-realism/footstep-dust.ogt` | Footstep Dust | movement | earth | 1 | 0.5 × 0.3 × 0.6 | ground | 0.14 | 14 |
| `vfx/cinematic-realism/footstep-water.ogt` | Wading Step | splash | water | 1 | 0.5 × 0.2 × 0.6 | ground | 0.14 | 18 |
| `vfx/cinematic-realism/fountain.ogt` | Fountain Jet | splash | water | loops | 1.0 × 2.2 × 0.9 | ground | 0.14 | 16 |
| `vfx/cinematic-realism/frozen.ogt` | Frozen | status | ice | loops | 1.4 × 1.6 × 1.4 | ground | 0.16 | 16 |
| `vfx/cinematic-realism/grenade-trail.ogt` | Grenade Smoke Thread | projectile |  | loops | 0.4 × 0.4 × 0.4 | centre | 0.31 | 14 |
| `vfx/cinematic-realism/ground-fog.ogt` | Ground Fog | ambient |  | loops | 6.6 × 2.1 × 0.0 | ground | 0.22 | 14 |
| `vfx/cinematic-realism/heal.ogt` | Mending Light | heal | nature | 2 | 1.5 × 1.7 × 1.5 | ground | 0.08 | 14 |
| `vfx/cinematic-realism/impact-concrete.ogt` | Concrete Impact | hit-spark | earth | 10 | 1.0 × 0.6 × 1.9 | ground | 0.68 | 16 |
| `vfx/cinematic-realism/impact-dirt.ogt` | Earth Impact | hit-spark | earth | 10 | 1.3 × 1.2 × 1.0 | ground | 0.71 | 15 |
| `vfx/cinematic-realism/impact-flesh.ogt` | Cloth Impact | hit-spark |  | 1.2 | 0.7 × 0.5 × 0.6 | ground | 0.14 | 15 |
| `vfx/cinematic-realism/impact-glass.ogt` | Glass Impact | hit-spark |  | 10 | 2.3 × 0.9 × 4.7 | ground | 0.71 | 13 |
| `vfx/cinematic-realism/impact-water.ogt` | Water Impact | hit-spark | water | 1.2 | 0.2 × 0.4 × 0.4 | ground | 0.14 | 18 |
| `vfx/cinematic-realism/impact-wood.ogt` | Timber Impact | hit-spark | nature | 10 | 1.1 × 0.5 × 2.5 | ground | 0.60 | 15 |
| `vfx/cinematic-realism/impact.ogt` | Steel Impact | hit-spark |  | 10 | 1.6 × 0.8 × 2.2 | ground | 0.69 | 18 |
| `vfx/cinematic-realism/landing.ogt` | Landing Dust | movement | earth | 1.4 | 1.3 × 1.0 × 1.7 | ground | 0.15 | 15 |
| `vfx/cinematic-realism/laser.ogt` | Laser Beam | beam | lightning | loops | 0.1 × 10.0 × 0.1 | ground | 0.04 | 13 |
| `vfx/cinematic-realism/leaves.ogt` | Falling Leaves | ambient | nature | loops | 4.0 × 0.9 × 4.0 | ground | 0.14 | 14 |
| `vfx/cinematic-realism/level-up.ogt` | Level Up | level-up |  | 2.2 | 3.1 × 5.9 × 3.1 | ground | 0.08 | 17 |
| `vfx/cinematic-realism/loot-glow.ogt` | Rare Drop Glow | marker |  | loops | 0.5 × 1.4 × 0.5 | ground | 0.06 | 14 |
| `vfx/cinematic-realism/magic-bolt.ogt` | Arcane Bolt | magic-projectile | arcane | loops | 0.5 × 0.5 × 0.5 | centre | 0.09 | 14 |
| `vfx/cinematic-realism/magic-circle.ogt` | Drawn Circle | magic-impact | arcane | 4 | 2.0 × 2.4 × 2.0 | ground | 0.97 | 13 |
| `vfx/cinematic-realism/magic.ogt` | Aether Wisps | magic-impact |  | 1.8 | 1.2 × 1.0 × 1.2 | centre | 0.09 | 14 |
| `vfx/cinematic-realism/mark-bullet.ogt` | Bullet Hole | decal |  | 30 | 0.1 × 0.0 × 0.1 | ground | 0.03 | 10 |
| `vfx/cinematic-realism/mark-burn.ogt` | Burn Mark | decal | fire | 30 | 1.0 × 0.0 × 0.9 | ground | 0.05 | 10 |
| `vfx/cinematic-realism/mark-wet.ogt` | Wet Patch | decal | water | 30 | 0.6 × 0.0 × 0.5 | ground | 0.57 | 10 |
| `vfx/cinematic-realism/muzzle-auto.ogt` | Automatic Fire | gunfire | fire | loops | 0.2 × 0.2 × 0.2 | ground | 0.17 | 17 |
| `vfx/cinematic-realism/muzzle-heavy.ogt` | Heavy Muzzle Blast | gunfire | fire | 2.2 | 0.9 × 1.7 × 1.1 | ground | 0.17 | 17 |
| `vfx/cinematic-realism/muzzle-suppressed.ogt` | Suppressed Shot | gunfire |  | 1.2 | 0.2 × 0.7 × 0.2 | ground | 0.14 | 16 |
| `vfx/cinematic-realism/muzzle.ogt` | Powder Flash | gunfire | fire | 1.4 | 0.4 × 0.5 × 0.4 | ground | 0.15 | 17 |
| `vfx/cinematic-realism/pickup.ogt` | Amber Motes | buff |  | 1.2 | 0.8 × 0.8 × 0.7 | ground | 0.08 | 14 |
| `vfx/cinematic-realism/poison.ogt` | Poisoned | status | nature | loops | 1.1 × 2.0 × 1.1 | ground | 0.17 | 16 |
| `vfx/cinematic-realism/portal.ogt` | Portal | portal | arcane | loops | 2.4 × 2.4 × 2.4 | ground | 0.15 | 17 |
| `vfx/cinematic-realism/reward-burst.ogt` | Reward Fountain | ui |  | 2.2 | 2.7 × 1.3 × 3.5 | ground | 0.07 | 15 |
| `vfx/cinematic-realism/rocket.ogt` | Rocket Motor | projectile | fire | loops | 1.3 × 2.0 × 1.2 | centre | 1.09 | 17 |
| `vfx/cinematic-realism/shell.ogt` | Spent Casing | gunfire |  | 2.2 | 0.7 × 0.2 × 0.1 | ground | 0.14 | 15 |
| `vfx/cinematic-realism/shield-hit.ogt` | Shield Ripple | shield | lightning | 0.8 | 1.3 × 0.9 × 1.3 | centre | 0.08 | 15 |
| `vfx/cinematic-realism/shocked.ogt` | Shocked | status | lightning | loops | 0.6 × 0.9 × 0.6 | ground | 0.04 | 12 |
| `vfx/cinematic-realism/skid.ogt` | Skid Mark | decal |  | 20 | 1.4 × 0.0 × 0.2 | ground | 0.53 | 10 |
| `vfx/cinematic-realism/slash.ogt` | Blade Arc | slash |  | 0.4 | 0.9 × 0.5 × 1.3 | centre | 0.11 | 13 |
| `vfx/cinematic-realism/smoke-grenade.ogt` | Smoke Screen | smoke |  | 18 | 2.6 × 2.5 × 2.7 | ground | 1.09 | 16 |
| `vfx/cinematic-realism/smoke-plume.ogt` | Wreck Smoke Column | smoke |  | loops | 3.0 × 3.3 × 3.2 | centre | 0.16 | 14 |
| `vfx/cinematic-realism/splash-small.ogt` | Small Splash | splash | water | 1.2 | 0.5 × 0.4 × 0.7 | ground | 0.14 | 22 |
| `vfx/cinematic-realism/splash.ogt` | Ballistic Spray | splash | water | 1.6 | 1.8 × 0.7 × 1.9 | ground | 0.15 | 26 |
| `vfx/cinematic-realism/steam.ogt` | Steam Leak | smoke | water | loops | 0.9 × 2.3 × 0.9 | ground | 0.18 | 14 |
| `vfx/cinematic-realism/sunbeam-dust.ogt` | Lit Dust | ambient |  | loops | 2.0 × 2.9 × 2.0 | ground | 0.04 | 11 |
| `vfx/cinematic-realism/teleport.ogt` | Arrival Flash | portal |  | 1.4 | 1.8 × 2.9 × 1.8 | ground | 0.12 | 16 |
| `vfx/cinematic-realism/thruster.ogt` | Attitude Jet | vehicle |  | 0.6 | 0.3 × 1.3 × 0.4 | ground | 0.04 | 13 |
| `vfx/cinematic-realism/tire-smoke.ogt` | Tyre Smoke | vehicle |  | loops | 1.7 × 1.4 × 1.6 | ground | 0.24 | 14 |
| `vfx/cinematic-realism/tracer.ogt` | Tracer Round | projectile | fire | loops | 0.1 × 2.9 × 0.1 | centre | 0.04 | 13 |
| `vfx/cinematic-realism/wake.ogt` | Hull Wake | vehicle | water | loops | 2.3 × 0.8 × 2.0 | ground | 0.19 | 18 |
| `vfx/cinematic-realism/warp.ogt` | Jump Flash | vehicle |  | 1.6 | 1.7 × 27.9 × 1.8 | centre | 0.09 | 15 |
| `vfx/cinematic-realism/waterfall-mist.ogt` | Waterfall Mist | splash | water | loops | 4.0 × 2.2 × 4.0 | ground | 0.19 | 18 |
| `vfx/cinematic-realism/wheel-dust.ogt` | Wheel Dust | vehicle | earth | loops | 1.7 × 1.3 × 2.1 | ground | 0.24 | 15 |

## 9. effekseer-andrewfm01

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

## 10. effekseer-effekseer01

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

## 11. effekseer-hato01

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

## 12. effekseer-magicalxspiral

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

## 13. effekseer-nextsoft01

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

## 14. effekseer-pierre01

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

## 15. effekseer-pierre02

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

## 16. effekseer-suzuki01

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

## 17. effekseer-tktk01

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

## 18. effekseer-tktk02

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

## 19. geometric

Texture-free hard geometry for minimal-geometric and faceted-low-poly: nine visual mechanisms, no bloom or soft sprites. 9 effects, downloadable: a release's `vfx/<pack>.tar.gz`: `og vfx add geometric`.

- Style: stylised, 3d
- Categories: ambient, buff, explosion, gunfire, hit-spark, magic-impact, smoke, splash, ui
- Elements: earth, water
- Keywords: faceted-low-poly, geometric, minimal-geometric, texture-free

Measured on NVIDIA L4 (Vulkan 580.82.07, native dev build, 960x540).

| Effect | Title | Category | Element | Seconds | Size (m) | Anchor | GPU ms | Draws |
|---|---|---|---|---|---|---|---|---|
| `vfx/geometric/celebration.ogt` | Coral Confetti | ui |  | 2 | 2.2 × 1.6 × 3.5 | ground | 0.04 | 11 |
| `vfx/geometric/dust.ogt` | Slate Dust | smoke | earth | 1.5 | 0.4 × 0.4 × 0.6 | ground | 0.04 | 11 |
| `vfx/geometric/explosion.ogt` | Facet Burst | explosion |  | 1.2 | 3.1 × 1.9 × 2.7 | centre | 0.08 | 14 |
| `vfx/geometric/impact.ogt` | Angular Impact | hit-spark |  | 0.5 | 0.9 × 0.7 × 1.2 | ground | 0.08 | 14 |
| `vfx/geometric/magic.ogt` | Open Triangles | magic-impact |  | 1.2 | 1.2 × 1.2 × 1.2 | ground | 0.05 | 13 |
| `vfx/geometric/muzzle.ogt` | Geometric Pulse | gunfire |  | 0.12 | 0.1 × 0.2 × 0.1 | centre | 0.03 | 11 |
| `vfx/geometric/pickup.ogt` | Rising Cubes | buff |  | 1 | 0.3 × 0.5 × 0.2 | centre | 0.03 | 11 |
| `vfx/geometric/splash.ogt` | Facet Splash | splash | water | 1.2 | 1.3 × 1.2 × 1.3 | centre | 0.04 | 11 |
| `vfx/geometric/trail.ogt` | Facet Trail | ambient |  | loops | 0.0 × 0.1 × 0.0 | ground | 0.04 | 11 |

## 20. Ambient effects

Ambient effects (ambient): 36 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-hato01`, `effekseer-pierre01`, `effekseer-starter`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/bubbles.ts` (Bubbles), `vfx/cartoon-2d/content/fireflies.ts` (Fireflies), `vfx/cartoon-2d/content/leaves.ts` (Falling Leaves), `vfx/cartoon-2d/content/twinkle.ts` (Twinkle)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/bubbles.ts` (Bubbles), `vfx/cartoon-2d-rubber-hose/content/fireflies.ts` (Fireflies), `vfx/cartoon-2d-rubber-hose/content/leaves.ts` (Falling Leaves), `vfx/cartoon-2d-rubber-hose/content/twinkle.ts` (Twinkle)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/bubbles.ts` (Bubbles), `vfx/cartoon-2d-storybook/content/fireflies.ts` (Fireflies), `vfx/cartoon-2d-storybook/content/leaves.ts` (Falling Leaves), `vfx/cartoon-2d-storybook/content/twinkle.ts` (Twinkle)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/bubbles.ts` (Bubbles), `vfx/cartoon-2d-watercolor/content/fireflies.ts` (Fireflies), `vfx/cartoon-2d-watercolor/content/leaves.ts` (Falling Leaves), `vfx/cartoon-2d-watercolor/content/twinkle.ts` (Twinkle)
- cinematic-realism: `vfx/cinematic-realism/electric-sparks.ogt` (Arcing Fault), `vfx/cinematic-realism/embers.ogt` (Drifting Embers), `vfx/cinematic-realism/fireflies.ogt` (Fireflies), `vfx/cinematic-realism/ground-fog.ogt` (Ground Fog), `vfx/cinematic-realism/leaves.ogt` (Falling Leaves), `vfx/cinematic-realism/sunbeam-dust.ogt` (Lit Dust)
- effekseer-hato01: `vfx/effekseer-hato01/aoi-chou.ogt` (Aoi Chou), `vfx/effekseer-hato01/iwai-ume.ogt` (Iwai Ume), `vfx/effekseer-hato01/kiroi-chou.ogt` (Kiroi Chou), `vfx/effekseer-hato01/sakura1.ogt` (Sakura 1)
- effekseer-pierre01: `vfx/effekseer-pierre01/Atmosphere.ogt` (Atmosphere), `vfx/effekseer-pierre01/Background.ogt` (Background)
- effekseer-starter: `vfx/effekseer-starter/electric_dust.ogt` (Floating Motes), `vfx/effekseer-starter/sakura1.ogt` (Sakura Petals)
- geometric: `vfx/geometric/trail.ogt` (Facet Trail)
- pixel-scenario: `vfx/pixel-scenario/content/cherry-blossom-petals.ts` (Thorn Ambient), `vfx/pixel-scenario/content/embers-rising.ts` (Blaze Ambient), `vfx/pixel-scenario/content/falling-leaves.ts` (Bloom Ambient), `vfx/pixel-scenario/content/fireflies.ts` (Mystic Ambient)
- realistic-starter: `vfx/realistic-starter/ambient-dust.ogt` (Ambient Dust)

## 21. Basic effects

Basic effects (basic): 11 in `effekseer-effekseer01`.

- effekseer-effekseer01: `vfx/effekseer-effekseer01/Simple_Distortion.ogt` (Simple Distortion), `vfx/effekseer-effekseer01/Simple_GeneratingPosition1.ogt` (Simple Generating Position 1), `vfx/effekseer-effekseer01/Simple_Model_UV.ogt` (Simple Model UV), `vfx/effekseer-effekseer01/Simple_Ribbon_Parent.ogt` (Simple Ribbon Parent), `vfx/effekseer-effekseer01/Simple_Ribbon_Sword.ogt` (Simple Ribbon Sword), `vfx/effekseer-effekseer01/Simple_Ring_Shape1.ogt` (Simple Ring Shape 1), `vfx/effekseer-effekseer01/Simple_Ring_Shape2.ogt` (Simple Ring Shape 2), `vfx/effekseer-effekseer01/Simple_Sprite_BillBoard.ogt` (Simple Sprite Bill Board), `vfx/effekseer-effekseer01/Simple_Sprite_Fixed.ogt` (Simple Sprite Fixed), `vfx/effekseer-effekseer01/Simple_Sprite_FixedYAxis.ogt` (Simple Sprite Fixed YAxis), `vfx/effekseer-effekseer01/Simple_Track1.ogt` (Simple Track 1)

## 22. Beam effects

Beam effects (beam): 10 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-effekseer01`.

- cartoon-2d: `vfx/cartoon-2d/content/laser.ts` (Ray Beam)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/laser.ts` (Ray Beam)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/laser.ts` (Ray Beam)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/laser.ts` (Ray Beam)
- cinematic-realism: `vfx/cinematic-realism/laser.ogt` (Laser Beam)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/blue_laser.ogt` (Blue Laser)
- effekseer-effekseer01: `vfx/effekseer-effekseer01/Homing_Laser01.ogt` (Homing Laser 01), `vfx/effekseer-effekseer01/Laser01.ogt` (Laser 01), `vfx/effekseer-effekseer01/Laser02.ogt` (Laser 02), `vfx/effekseer-effekseer01/Laser03.ogt` (Laser 03)

## 23. Buff effects

Buff effects (buff): 25 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `geometric`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/buff.ts` (Buff), `vfx/cartoon-2d/content/powerup.ts` (Power Up)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/buff.ts` (Buff), `vfx/cartoon-2d-rubber-hose/content/powerup.ts` (Power Up)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/buff.ts` (Buff), `vfx/cartoon-2d-storybook/content/powerup.ts` (Power Up)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/buff.ts` (Buff), `vfx/cartoon-2d-watercolor/content/powerup.ts` (Power Up)
- cinematic-realism: `vfx/cinematic-realism/buff.ogt` (Rising Strands), `vfx/cinematic-realism/pickup.ogt` (Amber Motes)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/PowerUp.ogt` (Power Up)
- effekseer-pierre01: `vfx/effekseer-pierre01/Herald.ogt` (Herald)
- effekseer-starter: `vfx/effekseer-starter/PowerUp.ogt` (Power Up), `vfx/effekseer-starter/hozyo2.ogt` (Power Rings), `vfx/effekseer-starter/hozyo4.ogt` (Attack Up)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/aura.ogt` (Aura)
- effekseer-tktk01: `vfx/effekseer-tktk01/hozyo1.ogt` (Hozyo 1), `vfx/effekseer-tktk01/hozyo2.ogt` (Hozyo 2), `vfx/effekseer-tktk01/hozyo3.ogt` (Hozyo 3), `vfx/effekseer-tktk01/hozyo4.ogt` (Hozyo 4), `vfx/effekseer-tktk01/hozyo5.ogt` (Hozyo 5)
- geometric: `vfx/geometric/pickup.ogt` (Rising Cubes)
- pixel-scenario: `vfx/pixel-scenario/content/charge-up-energy.ts` (Pulse Charge), `vfx/pixel-scenario/content/dark-shadow-aura.ts` (Night Aura), `vfx/pixel-scenario/content/hex-shield-barrier.ts` (Flux Shield)

## 24. Debris effects

Debris effects (debris): 1 in `realistic-starter`.

- realistic-starter: `vfx/realistic-starter/dust-burst.ogt` (Dust Burst)

## 25. Decal effects

Decal effects (decal): 4 in `cinematic-realism`.

- cinematic-realism: `vfx/cinematic-realism/mark-bullet.ogt` (Bullet Hole), `vfx/cinematic-realism/mark-burn.ogt` (Burn Mark), `vfx/cinematic-realism/mark-wet.ogt` (Wet Patch), `vfx/cinematic-realism/skid.ogt` (Skid Mark)

## 26. Destruction effects

Destruction effects (destruction): 15 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `vfx/cartoon-2d/content/break-crate.ts` (Crate Break), `vfx/cartoon-2d/content/break-ice.ts` (Ice Shatter), `vfx/cartoon-2d/content/break-rock.ts` (Rock Break)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/break-crate.ts` (Crate Break), `vfx/cartoon-2d-rubber-hose/content/break-ice.ts` (Ice Shatter), `vfx/cartoon-2d-rubber-hose/content/break-rock.ts` (Rock Break)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/break-crate.ts` (Crate Break), `vfx/cartoon-2d-storybook/content/break-ice.ts` (Ice Shatter), `vfx/cartoon-2d-storybook/content/break-rock.ts` (Rock Break)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/break-crate.ts` (Crate Break), `vfx/cartoon-2d-watercolor/content/break-ice.ts` (Ice Shatter), `vfx/cartoon-2d-watercolor/content/break-rock.ts` (Rock Break)
- cinematic-realism: `vfx/cinematic-realism/break-glass.ogt` (Pane Break), `vfx/cinematic-realism/break-stone.ogt` (Masonry Break), `vfx/cinematic-realism/break-wood.ogt` (Crate Break)

## 27. Explosion effects

Explosion effects (explosion): 30 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/explosion-small.ts` (Pop Boom), `vfx/cartoon-2d/content/explosion.ts` (Boom)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/explosion-small.ts` (Pop Boom), `vfx/cartoon-2d-rubber-hose/content/explosion.ts` (Boom)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/explosion-small.ts` (Pop Boom), `vfx/cartoon-2d-storybook/content/explosion.ts` (Boom)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/explosion-small.ts` (Pop Boom), `vfx/cartoon-2d-watercolor/content/explosion.ts` (Boom)
- cinematic-realism: `vfx/cinematic-realism/explosion-air.ogt` (Airburst), `vfx/cinematic-realism/explosion-energy.ogt` (Energy Blast), `vfx/cinematic-realism/explosion-large.ogt` (Large Blast), `vfx/cinematic-realism/explosion-small.ogt` (Grenade Blast), `vfx/cinematic-realism/explosion-water.ogt` (Depth Blast), `vfx/cinematic-realism/explosion.ogt` (Cinematic Explosion), `vfx/cinematic-realism/flashbang.ogt` (Flash Charge)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/boss_death.ogt` (Boss Death)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Breakdown.ogt` (Breakdown), `vfx/effekseer-magicalxspiral/IcicleBroken.ogt` (Icicle Broken), `vfx/effekseer-magicalxspiral/StairBroken.ogt` (Stair Broken), `vfx/effekseer-magicalxspiral/StairBroken3.ogt` (Stair Broken 3)
- effekseer-pierre02: `vfx/effekseer-pierre02/FeatherBomb.ogt` (Feather Bomb)
- effekseer-starter: `vfx/effekseer-starter/Blow4.ogt` (Bright Explosion), `vfx/effekseer-starter/Breakdown.ogt` (Breakdown Burst), `vfx/effekseer-starter/FeatherBomb.ogt` (Smoke Bomb), `vfx/effekseer-starter/boss_death.ogt` (Boss Death), `vfx/effekseer-starter/meteo1.ogt` (Meteor Strike)
- geometric: `vfx/geometric/explosion.ogt` (Facet Burst)
- pixel-scenario: `vfx/pixel-scenario/content/big-fire-explosion.ts` (Cinder Explosion), `vfx/pixel-scenario/content/meteor-strike.ts` (Ember Meteor)
- realistic-starter: `vfx/realistic-starter/explosion.ogt` (Explosion)

## 28. Fire effects

Fire effects (fire): 45 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/campfire.ts` (Campfire), `vfx/cartoon-2d/content/fuse.ts` (Fuse Sparks), `vfx/cartoon-2d/content/torch.ts` (Torch Flame)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/campfire.ts` (Campfire), `vfx/cartoon-2d-rubber-hose/content/fuse.ts` (Fuse Sparks), `vfx/cartoon-2d-rubber-hose/content/torch.ts` (Torch Flame)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/campfire.ts` (Campfire), `vfx/cartoon-2d-storybook/content/fuse.ts` (Fuse Sparks), `vfx/cartoon-2d-storybook/content/torch.ts` (Torch Flame)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/campfire.ts` (Campfire), `vfx/cartoon-2d-watercolor/content/fuse.ts` (Fuse Sparks), `vfx/cartoon-2d-watercolor/content/torch.ts` (Torch Flame)
- cinematic-realism: `vfx/cinematic-realism/campfire.ogt` (Campfire), `vfx/cinematic-realism/fire-large.ogt` (Large Fire), `vfx/cinematic-realism/flamethrower.ogt` (Flame Stream)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/fire.ogt` (Fire)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/FirePot1.ogt` (Fire Pot 1), `vfx/effekseer-magicalxspiral/FirePot2.ogt` (Fire Pot 2), `vfx/effekseer-magicalxspiral/Magic1_fire.ogt` (Magic 1 Fire), `vfx/effekseer-magicalxspiral/Magic2_fire.ogt` (Magic 2 Fire)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicFire1.ogt` (Magic Fire 1), `vfx/effekseer-nextsoft01/MagicFire2.ogt` (Magic Fire 2)
- effekseer-pierre01: `vfx/effekseer-pierre01/Flame.ogt` (Flame)
- effekseer-starter: `vfx/effekseer-starter/Fire7.ogt` (Fire Pillar), `vfx/effekseer-starter/MagicFire2.ogt` (Fire Breath)
- effekseer-tktk01: `vfx/effekseer-tktk01/Fire1.ogt` (Fire 1), `vfx/effekseer-tktk01/Fire2.ogt` (Fire 2), `vfx/effekseer-tktk01/Fire3.ogt` (Fire 3), `vfx/effekseer-tktk01/Fire4.ogt` (Fire 4), `vfx/effekseer-tktk01/Fire5.ogt` (Fire 5), `vfx/effekseer-tktk01/Fire6.ogt` (Fire 6), `vfx/effekseer-tktk01/Fire7.ogt` (Fire 7), `vfx/effekseer-tktk01/Fire8.ogt` (Fire 8)
- effekseer-tktk02: `vfx/effekseer-tktk02/Fire1.ogt` (Fire 1), `vfx/effekseer-tktk02/Fire2.ogt` (Fire 2), `vfx/effekseer-tktk02/Fire3.ogt` (Fire 3), `vfx/effekseer-tktk02/Fire4.ogt` (Fire 4), `vfx/effekseer-tktk02/Fire5.ogt` (Fire 5), `vfx/effekseer-tktk02/Fire6.ogt` (Fire 6), `vfx/effekseer-tktk02/Fire7.ogt` (Fire 7), `vfx/effekseer-tktk02/Fire8.ogt` (Fire 8)
- pixel-scenario: `vfx/pixel-scenario/content/campfire.ts` (Ember Flame), `vfx/pixel-scenario/content/dragon-fire-breath.ts` (Cinder Breath), `vfx/pixel-scenario/content/fire-tornado.ts` (Blaze Tornado)
- realistic-starter: `vfx/realistic-starter/torch-fire.ogt` (Torch Fire)

## 29. Gunfire effects

Gunfire effects (gunfire): 11 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `geometric`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/muzzle.ts` (Pop Muzzle)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/muzzle.ts` (Pop Muzzle)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/muzzle.ts` (Pop Muzzle)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/muzzle.ts` (Pop Muzzle)
- cinematic-realism: `vfx/cinematic-realism/muzzle-auto.ogt` (Automatic Fire), `vfx/cinematic-realism/muzzle-heavy.ogt` (Heavy Muzzle Blast), `vfx/cinematic-realism/muzzle-suppressed.ogt` (Suppressed Shot), `vfx/cinematic-realism/muzzle.ogt` (Powder Flash), `vfx/cinematic-realism/shell.ogt` (Spent Casing)
- geometric: `vfx/geometric/muzzle.ogt` (Geometric Pulse)
- realistic-starter: `vfx/realistic-starter/muzzle-flash.ogt` (Muzzle Flash)

## 30. Heal effects

Heal effects (heal): 22 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/heal.ts` (Heal)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/heal.ts` (Heal)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/heal.ts` (Heal)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/heal.ts` (Heal)
- cinematic-realism: `vfx/cinematic-realism/heal.ogt` (Mending Light)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/HealPotion1.ogt` (Heal Potion 1), `vfx/effekseer-nextsoft01/HealPotion2.ogt` (Heal Potion 2), `vfx/effekseer-nextsoft01/MagicHeal1.ogt` (Magic Heal 1), `vfx/effekseer-nextsoft01/MagicHeal2.ogt` (Magic Heal 2)
- effekseer-pierre02: `vfx/effekseer-pierre02/Benediction.ogt` (Benediction)
- effekseer-starter: `vfx/effekseer-starter/Cure1.ogt` (Star Cure), `vfx/effekseer-starter/Cure7.ogt` (Light Cure), `vfx/effekseer-starter/HealPotion2.ogt` (Potion), `vfx/effekseer-starter/MagicHeal2.ogt` (Healing Circle)
- effekseer-tktk01: `vfx/effekseer-tktk01/Cure1.ogt` (Cure 1), `vfx/effekseer-tktk01/Cure2.ogt` (Cure 2), `vfx/effekseer-tktk01/Cure3.ogt` (Cure 3), `vfx/effekseer-tktk01/Cure4.ogt` (Cure 4), `vfx/effekseer-tktk01/Cure5.ogt` (Cure 5), `vfx/effekseer-tktk01/Cure6.ogt` (Cure 6), `vfx/effekseer-tktk01/Cure7.ogt` (Cure 7)
- pixel-scenario: `vfx/pixel-scenario/content/healing-with-rising-crosses.ts` (Bloom Heal)

## 31. Hit spark effects

Hit spark effects (hit-spark): 56 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `geometric`, `pixel-scenario`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/bullet-hit.ts` (Pellet Pop), `vfx/cartoon-2d/content/hit-heavy.ts` (Smash Hit), `vfx/cartoon-2d/content/hit-small.ts` (Tap Hit), `vfx/cartoon-2d/content/hit.ts` (Pow Hit), `vfx/cartoon-2d/content/stomp.ts` (Stomp)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/bullet-hit.ts` (Pellet Pop), `vfx/cartoon-2d-rubber-hose/content/hit-heavy.ts` (Smash Hit), `vfx/cartoon-2d-rubber-hose/content/hit-small.ts` (Tap Hit), `vfx/cartoon-2d-rubber-hose/content/hit.ts` (Pow Hit), `vfx/cartoon-2d-rubber-hose/content/stomp.ts` (Stomp)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/bullet-hit.ts` (Pellet Pop), `vfx/cartoon-2d-storybook/content/hit-heavy.ts` (Smash Hit), `vfx/cartoon-2d-storybook/content/hit-small.ts` (Tap Hit), `vfx/cartoon-2d-storybook/content/hit.ts` (Pow Hit), `vfx/cartoon-2d-storybook/content/stomp.ts` (Stomp)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/bullet-hit.ts` (Pellet Pop), `vfx/cartoon-2d-watercolor/content/hit-heavy.ts` (Smash Hit), `vfx/cartoon-2d-watercolor/content/hit-small.ts` (Tap Hit), `vfx/cartoon-2d-watercolor/content/hit.ts` (Pow Hit), `vfx/cartoon-2d-watercolor/content/stomp.ts` (Stomp)
- cinematic-realism: `vfx/cinematic-realism/impact-concrete.ogt` (Concrete Impact), `vfx/cinematic-realism/impact-dirt.ogt` (Earth Impact), `vfx/cinematic-realism/impact-flesh.ogt` (Cloth Impact), `vfx/cinematic-realism/impact-glass.ogt` (Glass Impact), `vfx/cinematic-realism/impact-water.ogt` (Water Impact), `vfx/cinematic-realism/impact-wood.ogt` (Timber Impact), `vfx/cinematic-realism/impact.ogt` (Steel Impact)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/hit_eff.ogt` (Hit Eff)
- effekseer-starter: `vfx/effekseer-starter/Attack_Impact.ogt` (Attack Impact), `vfx/effekseer-starter/Blow2.ogt` (Spark Hit), `vfx/effekseer-starter/Blow8.ogt` (Star Hit), `vfx/effekseer-starter/hit_eff.ogt` (Hit Flash)
- effekseer-tktk01: `vfx/effekseer-tktk01/Blow1.ogt` (Blow 1), `vfx/effekseer-tktk01/Blow10.ogt` (Blow 10), `vfx/effekseer-tktk01/Blow11.ogt` (Blow 11), `vfx/effekseer-tktk01/Blow12.ogt` (Blow 12), `vfx/effekseer-tktk01/Blow13.ogt` (Blow 13), `vfx/effekseer-tktk01/Blow14.ogt` (Blow 14), `vfx/effekseer-tktk01/Blow15.ogt` (Blow 15), `vfx/effekseer-tktk01/Blow2.ogt` (Blow 2), `vfx/effekseer-tktk01/Blow3.ogt` (Blow 3), `vfx/effekseer-tktk01/Blow4.ogt` (Blow 4), `vfx/effekseer-tktk01/Blow5.ogt` (Blow 5), `vfx/effekseer-tktk01/Blow6.ogt` (Blow 6), `vfx/effekseer-tktk01/Blow7.ogt` (Blow 7), `vfx/effekseer-tktk01/Blow8.ogt` (Blow 8), `vfx/effekseer-tktk01/Blow9.ogt` (Blow 9)
- effekseer-tktk02: `vfx/effekseer-tktk02/Blow1.ogt` (Blow 1), `vfx/effekseer-tktk02/Blow2.ogt` (Blow 2), `vfx/effekseer-tktk02/Blow3.ogt` (Blow 3), `vfx/effekseer-tktk02/Blow4.ogt` (Blow 4), `vfx/effekseer-tktk02/Blow5.ogt` (Blow 5), `vfx/effekseer-tktk02/Blow6.ogt` (Blow 6)
- geometric: `vfx/geometric/impact.ogt` (Angular Impact)
- pixel-scenario: `vfx/pixel-scenario/content/electric-impact-hit.ts` (Static Impact)
- realistic-starter: `vfx/realistic-starter/bullet-impact.ogt` (Bullet Impact)

## 32. Level up effects

Level up effects (level-up): 4 in `cinematic-realism`, `effekseer-starter`, `pixel-scenario`.

- cinematic-realism: `vfx/cinematic-realism/level-up.ogt` (Level Up)
- effekseer-starter: `vfx/effekseer-starter/Light4.ogt` (Level Up Arrows), `vfx/effekseer-starter/Thunder1.ogt` (Golden Rise)
- pixel-scenario: `vfx/pixel-scenario/content/level-up.ts` (Gilded Level Up)

## 33. Magic impact effects

Magic impact effects (magic-impact): 133 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `geometric`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/charge.ts` (Charge Ball), `vfx/cartoon-2d/content/magic-circle.ts` (Spell Circle), `vfx/cartoon-2d/content/magic.ts` (Sparkle Cast), `vfx/cartoon-2d/content/portal.ts` (Portal), `vfx/cartoon-2d/content/whirlwind.ts` (Whirlwind)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/charge.ts` (Charge Ball), `vfx/cartoon-2d-rubber-hose/content/magic-circle.ts` (Spell Circle), `vfx/cartoon-2d-rubber-hose/content/magic.ts` (Sparkle Cast), `vfx/cartoon-2d-rubber-hose/content/portal.ts` (Portal), `vfx/cartoon-2d-rubber-hose/content/whirlwind.ts` (Whirlwind)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/charge.ts` (Charge Ball), `vfx/cartoon-2d-storybook/content/magic-circle.ts` (Spell Circle), `vfx/cartoon-2d-storybook/content/magic.ts` (Sparkle Cast), `vfx/cartoon-2d-storybook/content/portal.ts` (Portal), `vfx/cartoon-2d-storybook/content/whirlwind.ts` (Whirlwind)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/charge.ts` (Charge Ball), `vfx/cartoon-2d-watercolor/content/magic-circle.ts` (Spell Circle), `vfx/cartoon-2d-watercolor/content/magic.ts` (Sparkle Cast), `vfx/cartoon-2d-watercolor/content/portal.ts` (Portal), `vfx/cartoon-2d-watercolor/content/whirlwind.ts` (Whirlwind)
- cinematic-realism: `vfx/cinematic-realism/charge.ogt` (Gathering Charge), `vfx/cinematic-realism/magic-circle.ogt` (Drawn Circle), `vfx/cinematic-realism/magic.ogt` (Aether Wisps)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/smallTexturesRibbon.ogt` (Small Textures Ribbon), `vfx/effekseer-andrewfm01/smallTexturesSquareWave_orbital.ogt` (Small Textures Square Wave Orbital)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/A_Gohlem1.ogt` (A Gohlem 1), `vfx/effekseer-magicalxspiral/A_Salamander1.ogt` (A Salamander 1), `vfx/effekseer-magicalxspiral/A_Salamander2.ogt` (A Salamander 2), `vfx/effekseer-magicalxspiral/A_Salamander4.ogt` (A Salamander 4), `vfx/effekseer-magicalxspiral/A_Sylph1.ogt` (A Sylph 1), `vfx/effekseer-magicalxspiral/A_Sylph2.ogt` (A Sylph 2), `vfx/effekseer-magicalxspiral/A_Undine1.ogt` (A Undine 1), `vfx/effekseer-magicalxspiral/A_Undine2.ogt` (A Undine 2), `vfx/effekseer-magicalxspiral/AquaPoint.ogt` (Aqua Point), `vfx/effekseer-magicalxspiral/Attack_Impact.ogt` (Attack Impact), `vfx/effekseer-magicalxspiral/Gohlem1.ogt` (Gohlem 1), `vfx/effekseer-magicalxspiral/Gohlem11.ogt` (Gohlem 11), `vfx/effekseer-magicalxspiral/Gohlem5.ogt` (Gohlem 5), `vfx/effekseer-magicalxspiral/Gohlem7.ogt` (Gohlem 7), `vfx/effekseer-magicalxspiral/Holy1.ogt` (Holy 1), `vfx/effekseer-magicalxspiral/Holy2.ogt` (Holy 2), `vfx/effekseer-magicalxspiral/Impact.ogt` (Impact), `vfx/effekseer-magicalxspiral/Magic1_earth.ogt` (Magic 1 Earth), `vfx/effekseer-magicalxspiral/Magic1_holy.ogt` (Magic 1 Holy), `vfx/effekseer-magicalxspiral/Magic1_water.ogt` (Magic 1 Water), `vfx/effekseer-magicalxspiral/Magic1_wind.ogt` (Magic 1 Wind), `vfx/effekseer-magicalxspiral/Magic2_earth.ogt` (Magic 2 Earth), `vfx/effekseer-magicalxspiral/Magic2_holy.ogt` (Magic 2 Holy), `vfx/effekseer-magicalxspiral/Magic2_water.ogt` (Magic 2 Water), `vfx/effekseer-magicalxspiral/Magic2_wind.ogt` (Magic 2 Wind), `vfx/effekseer-magicalxspiral/Rock1.ogt` (Rock 1), `vfx/effekseer-magicalxspiral/Rock2.ogt` (Rock 2), `vfx/effekseer-magicalxspiral/Salamander1.ogt` (Salamander 1), `vfx/effekseer-magicalxspiral/Salamander11.ogt` (Salamander 11), `vfx/effekseer-magicalxspiral/Salamander12.ogt` (Salamander 12), `vfx/effekseer-magicalxspiral/Salamander14.ogt` (Salamander 14), `vfx/effekseer-magicalxspiral/Salamander15.ogt` (Salamander 15), `vfx/effekseer-magicalxspiral/Salamander2.ogt` (Salamander 2), `vfx/effekseer-magicalxspiral/Salamander3.ogt` (Salamander 3), `vfx/effekseer-magicalxspiral/Salamander4.ogt` (Salamander 4), `vfx/effekseer-magicalxspiral/Salamander5.ogt` (Salamander 5), `vfx/effekseer-magicalxspiral/Salamander6.ogt` (Salamander 6), `vfx/effekseer-magicalxspiral/Salamander7.ogt` (Salamander 7), `vfx/effekseer-magicalxspiral/Sylph1.ogt` (Sylph 1), `vfx/effekseer-magicalxspiral/Sylph10.ogt` (Sylph 10), `vfx/effekseer-magicalxspiral/Sylph11.ogt` (Sylph 11), `vfx/effekseer-magicalxspiral/Sylph13.ogt` (Sylph 13), `vfx/effekseer-magicalxspiral/Sylph15.ogt` (Sylph 15), `vfx/effekseer-magicalxspiral/Sylph2.ogt` (Sylph 2), `vfx/effekseer-magicalxspiral/Sylph3.ogt` (Sylph 3), `vfx/effekseer-magicalxspiral/Sylph4.ogt` (Sylph 4), `vfx/effekseer-magicalxspiral/Sylph5.ogt` (Sylph 5), `vfx/effekseer-magicalxspiral/Sylph6.ogt` (Sylph 6), `vfx/effekseer-magicalxspiral/Sylph7.ogt` (Sylph 7), `vfx/effekseer-magicalxspiral/Sylph8.ogt` (Sylph 8), `vfx/effekseer-magicalxspiral/Thunder1.ogt` (Thunder 1), `vfx/effekseer-magicalxspiral/Thunder2.ogt` (Thunder 2), `vfx/effekseer-magicalxspiral/Thunder3.ogt` (Thunder 3), `vfx/effekseer-magicalxspiral/Topwater1.ogt` (Topwater 1), `vfx/effekseer-magicalxspiral/Undine1.ogt` (Undine 1), `vfx/effekseer-magicalxspiral/Undine11.ogt` (Undine 11), `vfx/effekseer-magicalxspiral/Undine13.ogt` (Undine 13), `vfx/effekseer-magicalxspiral/Undine2.ogt` (Undine 2), `vfx/effekseer-magicalxspiral/Undine3.ogt` (Undine 3), `vfx/effekseer-magicalxspiral/Undine4.ogt` (Undine 4), `vfx/effekseer-magicalxspiral/Undine5.ogt` (Undine 5), `vfx/effekseer-magicalxspiral/Undine6.ogt` (Undine 6), `vfx/effekseer-magicalxspiral/Undine8.ogt` (Undine 8), `vfx/effekseer-magicalxspiral/WHead1.ogt` (WHead 1), `vfx/effekseer-magicalxspiral/WHead2.ogt` (WHead 2), `vfx/effekseer-magicalxspiral/WHead3.ogt` (WHead 3), `vfx/effekseer-magicalxspiral/Water_Impact.ogt` (Water Impact), `vfx/effekseer-magicalxspiral/Wind_Impact.ogt` (Wind Impact), `vfx/effekseer-magicalxspiral/salamander13.ogt` (Salamander 13)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicCold.ogt` (Magic Cold), `vfx/effekseer-nextsoft01/MagicDark.ogt` (Magic Dark), `vfx/effekseer-nextsoft01/MagicLight.ogt` (Magic Light), `vfx/effekseer-nextsoft01/MagicThunder.ogt` (Magic Thunder), `vfx/effekseer-nextsoft01/MagicWater.ogt` (Magic Water)
- effekseer-pierre01: `vfx/effekseer-pierre01/LightningStrike.ogt` (Lightning Strike), `vfx/effekseer-pierre01/MonsterRoar.ogt` (Monster Roar)
- effekseer-pierre02: `vfx/effekseer-pierre02/LossOfBlood.ogt` (Loss Of Blood)
- effekseer-starter: `vfx/effekseer-starter/MagicCold.ogt` (Frost), `vfx/effekseer-starter/MagicDark.ogt` (Dark Curse), `vfx/effekseer-starter/MagicWater.ogt` (Bubble Burst), `vfx/effekseer-starter/Undine4.ogt` (Water Burst)
- effekseer-tktk01: `vfx/effekseer-tktk01/Dark1.ogt` (Dark 1), `vfx/effekseer-tktk01/Dark2.ogt` (Dark 2), `vfx/effekseer-tktk01/Dark3.ogt` (Dark 3), `vfx/effekseer-tktk01/Light1.ogt` (Light 1), `vfx/effekseer-tktk01/Light2.ogt` (Light 2), `vfx/effekseer-tktk01/Light3.ogt` (Light 3), `vfx/effekseer-tktk01/Light4.ogt` (Light 4), `vfx/effekseer-tktk01/Magic1.ogt` (Magic 1), `vfx/effekseer-tktk01/Magic2.ogt` (Magic 2), `vfx/effekseer-tktk01/Soil1.ogt` (Soil 1), `vfx/effekseer-tktk01/Wind1.ogt` (Wind 1), `vfx/effekseer-tktk01/Wind2.ogt` (Wind 2)
- effekseer-tktk02: `vfx/effekseer-tktk02/Dark1.ogt` (Dark 1), `vfx/effekseer-tktk02/Dark2.ogt` (Dark 2), `vfx/effekseer-tktk02/Light1.ogt` (Light 1), `vfx/effekseer-tktk02/Light2.ogt` (Light 2), `vfx/effekseer-tktk02/Light3.ogt` (Light 3), `vfx/effekseer-tktk02/Light4.ogt` (Light 4)
- geometric: `vfx/geometric/magic.ogt` (Open Triangles)
- pixel-scenario: `vfx/pixel-scenario/content/arcane-magic-circle.ts` (Eldritch Magic Circle), `vfx/pixel-scenario/content/frost-nova.ts` (Frost Nova), `vfx/pixel-scenario/content/ground-slam-quake.ts` (Stone Ground Slam), `vfx/pixel-scenario/content/holy-pillar-of-light.ts` (Divine Pillar), `vfx/pixel-scenario/content/ice-spikes-erupting.ts` (Frozen Spikes), `vfx/pixel-scenario/content/lava-eruption-pillar.ts` (Volcanic Pillar), `vfx/pixel-scenario/content/lightning-strike-from-the-sky.ts` (Storm Strike), `vfx/pixel-scenario/content/water-splash.ts` (Aqua Splash)

## 34. Magic projectile effects

Magic projectile effects (magic-projectile): 41 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/magic-bolt.ts` (Magic Bolt)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/magic-bolt.ts` (Magic Bolt)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/magic-bolt.ts` (Magic Bolt)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/magic-bolt.ts` (Magic Bolt)
- cinematic-realism: `vfx/cinematic-realism/magic-bolt.ogt` (Arcane Bolt)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/drill.ogt` (Drill)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Icicle1.ogt` (Icicle 1), `vfx/effekseer-magicalxspiral/Icicle2.ogt` (Icicle 2), `vfx/effekseer-magicalxspiral/Icicle3.ogt` (Icicle 3), `vfx/effekseer-magicalxspiral/IcicleLocus.ogt` (Icicle Locus), `vfx/effekseer-magicalxspiral/Lance3.ogt` (Lance 3), `vfx/effekseer-magicalxspiral/Lance4.ogt` (Lance 4), `vfx/effekseer-magicalxspiral/Lance5.ogt` (Lance 5), `vfx/effekseer-magicalxspiral/meteo1.ogt` (Meteo 1)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicMeteo.ogt` (Magic Meteo), `vfx/effekseer-nextsoft01/Spear1.ogt` (Spear 1), `vfx/effekseer-nextsoft01/Spear2.ogt` (Spear 2)
- effekseer-pierre01: `vfx/effekseer-pierre01/MissileBoost.ogt` (Missile Boost), `vfx/effekseer-pierre01/PhantasmMeteor.ogt` (Phantasm Meteor), `vfx/effekseer-pierre01/PhantasmMeteor_Single.ogt` (Phantasm Meteor Single), `vfx/effekseer-pierre01/SonicBoom.ogt` (Sonic Boom)
- effekseer-pierre02: `vfx/effekseer-pierre02/BloodLance.ogt` (Blood Lance), `vfx/effekseer-pierre02/FireBall.ogt` (Fire Ball)
- effekseer-starter: `vfx/effekseer-starter/FireBall.ogt` (Fireball), `vfx/effekseer-starter/Icicle2.ogt` (Icicles), `vfx/effekseer-starter/Laser03.ogt` (Green Laser), `vfx/effekseer-starter/Spear2.ogt` (Light Spear)
- effekseer-tktk01: `vfx/effekseer-tktk01/Arrow1.ogt` (Arrow 1), `vfx/effekseer-tktk01/Arrow2.ogt` (Arrow 2)
- effekseer-tktk02: `vfx/effekseer-tktk02/Gun1.ogt` (Gun 1), `vfx/effekseer-tktk02/Gun2.ogt` (Gun 2), `vfx/effekseer-tktk02/Gun3.ogt` (Gun 3), `vfx/effekseer-tktk02/Gun4.ogt` (Gun 4), `vfx/effekseer-tktk02/Gun5.ogt` (Gun 5), `vfx/effekseer-tktk02/Gun6.ogt` (Gun 6), `vfx/effekseer-tktk02/Gun7.ogt` (Gun 7), `vfx/effekseer-tktk02/Gun8.ogt` (Gun 8), `vfx/effekseer-tktk02/Gun9.ogt` (Gun 9)
- pixel-scenario: `vfx/pixel-scenario/content/blue-energy-beam.ts` (Aether Beam), `vfx/pixel-scenario/content/fireball-projectile.ts` (Ember Projectile), `vfx/pixel-scenario/content/plasma-orb.ts` (Ion Orb)

## 35. Marker effects

Marker effects (marker): 7 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `vfx/cartoon-2d/content/checkpoint.ts` (Checkpoint)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/checkpoint.ts` (Checkpoint)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/checkpoint.ts` (Checkpoint)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/checkpoint.ts` (Checkpoint)
- cinematic-realism: `vfx/cinematic-realism/beacon.ogt` (Marker Beacon), `vfx/cinematic-realism/chest-open.ogt` (Opened Chest), `vfx/cinematic-realism/loot-glow.ogt` (Rare Drop Glow)

## 36. Movement effects

Movement effects (movement): 31 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `vfx/cartoon-2d/content/dash.ts` (Dash Lines), `vfx/cartoon-2d/content/double-jump.ts` (Air Jump), `vfx/cartoon-2d/content/jump-dust.ts` (Jump Dust), `vfx/cartoon-2d/content/land-dust.ts` (Land Dust), `vfx/cartoon-2d/content/run-dust.ts` (Run Dust), `vfx/cartoon-2d/content/skid.ts` (Skid Dust), `vfx/cartoon-2d/content/teleport.ts` (Teleport)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/dash.ts` (Dash Lines), `vfx/cartoon-2d-rubber-hose/content/double-jump.ts` (Air Jump), `vfx/cartoon-2d-rubber-hose/content/jump-dust.ts` (Jump Dust), `vfx/cartoon-2d-rubber-hose/content/land-dust.ts` (Land Dust), `vfx/cartoon-2d-rubber-hose/content/run-dust.ts` (Run Dust), `vfx/cartoon-2d-rubber-hose/content/skid.ts` (Skid Dust), `vfx/cartoon-2d-rubber-hose/content/teleport.ts` (Teleport)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/dash.ts` (Dash Lines), `vfx/cartoon-2d-storybook/content/double-jump.ts` (Air Jump), `vfx/cartoon-2d-storybook/content/jump-dust.ts` (Jump Dust), `vfx/cartoon-2d-storybook/content/land-dust.ts` (Land Dust), `vfx/cartoon-2d-storybook/content/run-dust.ts` (Run Dust), `vfx/cartoon-2d-storybook/content/skid.ts` (Skid Dust), `vfx/cartoon-2d-storybook/content/teleport.ts` (Teleport)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/dash.ts` (Dash Lines), `vfx/cartoon-2d-watercolor/content/double-jump.ts` (Air Jump), `vfx/cartoon-2d-watercolor/content/jump-dust.ts` (Jump Dust), `vfx/cartoon-2d-watercolor/content/land-dust.ts` (Land Dust), `vfx/cartoon-2d-watercolor/content/run-dust.ts` (Run Dust), `vfx/cartoon-2d-watercolor/content/skid.ts` (Skid Dust), `vfx/cartoon-2d-watercolor/content/teleport.ts` (Teleport)
- cinematic-realism: `vfx/cinematic-realism/dash.ogt` (Dash), `vfx/cinematic-realism/footstep-dust.ogt` (Footstep Dust), `vfx/cinematic-realism/landing.ogt` (Landing Dust)

## 37. Pickup effects

Pickup effects (pickup): 20 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`.

- cartoon-2d: `vfx/cartoon-2d/content/chest-open.ts` (Treasure), `vfx/cartoon-2d/content/coin.ts` (Coin Grab), `vfx/cartoon-2d/content/gem.ts` (Gem Grab), `vfx/cartoon-2d/content/heart.ts` (Heart Grab), `vfx/cartoon-2d/content/star-collect.ts` (Star Grab)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/chest-open.ts` (Treasure), `vfx/cartoon-2d-rubber-hose/content/coin.ts` (Coin Grab), `vfx/cartoon-2d-rubber-hose/content/gem.ts` (Gem Grab), `vfx/cartoon-2d-rubber-hose/content/heart.ts` (Heart Grab), `vfx/cartoon-2d-rubber-hose/content/star-collect.ts` (Star Grab)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/chest-open.ts` (Treasure), `vfx/cartoon-2d-storybook/content/coin.ts` (Coin Grab), `vfx/cartoon-2d-storybook/content/gem.ts` (Gem Grab), `vfx/cartoon-2d-storybook/content/heart.ts` (Heart Grab), `vfx/cartoon-2d-storybook/content/star-collect.ts` (Star Grab)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/chest-open.ts` (Treasure), `vfx/cartoon-2d-watercolor/content/coin.ts` (Coin Grab), `vfx/cartoon-2d-watercolor/content/gem.ts` (Gem Grab), `vfx/cartoon-2d-watercolor/content/heart.ts` (Heart Grab), `vfx/cartoon-2d-watercolor/content/star-collect.ts` (Star Grab)

## 38. Portal effects

Portal effects (portal): 11 in `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `effekseer-starter`, `pixel-scenario`.

- cinematic-realism: `vfx/cinematic-realism/portal.ogt` (Portal), `vfx/cinematic-realism/teleport.ogt` (Arrival Flash)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/magic_circle.ogt` (Magic Circle)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/MagicArea.ogt` (Magic Area)
- effekseer-pierre02: `vfx/effekseer-pierre02/GateOfCalve.ogt` (Gate Of Calve)
- effekseer-starter: `vfx/effekseer-starter/Magic2_fire.ogt` (Fire Pentagram), `vfx/effekseer-starter/magic_circle.ogt` (Magic Circle), `vfx/effekseer-starter/smallTexturesSquareWave_orbital.ogt` (Arcane Ring)
- pixel-scenario: `vfx/pixel-scenario/content/dimensional-portal.ts` (Astral Portal), `vfx/pixel-scenario/content/purple-black-hole-vortex.ts` (Shadow Vortex), `vfx/pixel-scenario/content/teleport-blink.ts` (Mystic Teleport)

## 39. Projectile effects

Projectile effects (projectile): 8 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `vfx/cartoon-2d/content/bullet.ts` (Pellet)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/bullet.ts` (Pellet)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/bullet.ts` (Pellet)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/bullet.ts` (Pellet)
- cinematic-realism: `vfx/cinematic-realism/bolt.ogt` (Energy Bolt), `vfx/cinematic-realism/grenade-trail.ogt` (Grenade Smoke Thread), `vfx/cinematic-realism/rocket.ogt` (Rocket Motor), `vfx/cinematic-realism/tracer.ogt` (Tracer Round)

## 40. Shield effects

Shield effects (shield): 13 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`.

- cartoon-2d: `vfx/cartoon-2d/content/block.ts` (Block), `vfx/cartoon-2d/content/shield.ts` (Shield Bubble)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/block.ts` (Block), `vfx/cartoon-2d-rubber-hose/content/shield.ts` (Shield Bubble)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/block.ts` (Block), `vfx/cartoon-2d-storybook/content/shield.ts` (Shield Bubble)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/block.ts` (Block), `vfx/cartoon-2d-watercolor/content/shield.ts` (Shield Bubble)
- cinematic-realism: `vfx/cinematic-realism/barrier.ogt` (Barrier Dome), `vfx/cinematic-realism/shield-hit.ogt` (Shield Ripple)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Gohlem_Shield1.ogt` (Gohlem Shield 1), `vfx/effekseer-magicalxspiral/Gohlem_Shield2.ogt` (Gohlem Shield 2)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicShield.ogt` (Magic Shield)

## 41. Slash effects

Slash effects (slash): 53 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre02`, `effekseer-starter`, `effekseer-suzuki01`, `effekseer-tktk01`, `effekseer-tktk02`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/slash.ts` (Swoosh Slash)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/slash.ts` (Swoosh Slash)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/slash.ts` (Swoosh Slash)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/slash.ts` (Swoosh Slash)
- cinematic-realism: `vfx/cinematic-realism/slash.ogt` (Blade Arc)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Attack1.ogt` (Attack 1), `vfx/effekseer-magicalxspiral/Attack2.ogt` (Attack 2), `vfx/effekseer-magicalxspiral/Attack5.ogt` (Attack 5), `vfx/effekseer-magicalxspiral/Attack6.ogt` (Attack 6)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicSword.ogt` (Magic Sword), `vfx/effekseer-nextsoft01/Sword1.ogt` (Sword 1), `vfx/effekseer-nextsoft01/Sword2.ogt` (Sword 2)
- effekseer-pierre02: `vfx/effekseer-pierre02/Sword_Ember.ogt` (Sword Ember), `vfx/effekseer-pierre02/Sword_Lightning.ogt` (Sword Lightning), `vfx/effekseer-pierre02/Sword_Moonlight.ogt` (Sword Moonlight), `vfx/effekseer-pierre02/Sword_Sacredlight.ogt` (Sword Sacredlight), `vfx/effekseer-pierre02/Sword_Storm.ogt` (Sword Storm)
- effekseer-starter: `vfx/effekseer-starter/Claw3.ogt` (Claw), `vfx/effekseer-starter/Sword5.ogt` (Blue Slash), `vfx/effekseer-starter/Sword7.ogt` (Cyan Cross Slash), `vfx/effekseer-starter/Sword8.ogt` (Red Arc Slash)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/sword.ogt` (Sword)
- effekseer-tktk01: `vfx/effekseer-tktk01/Sword1.ogt` (Sword 1), `vfx/effekseer-tktk01/Sword10.ogt` (Sword 10), `vfx/effekseer-tktk01/Sword11.ogt` (Sword 11), `vfx/effekseer-tktk01/Sword12.ogt` (Sword 12), `vfx/effekseer-tktk01/Sword13.ogt` (Sword 13), `vfx/effekseer-tktk01/Sword14.ogt` (Sword 14), `vfx/effekseer-tktk01/Sword15.ogt` (Sword 15), `vfx/effekseer-tktk01/Sword16.ogt` (Sword 16), `vfx/effekseer-tktk01/Sword17.ogt` (Sword 17), `vfx/effekseer-tktk01/Sword18.ogt` (Sword 18), `vfx/effekseer-tktk01/Sword19.ogt` (Sword 19), `vfx/effekseer-tktk01/Sword2.ogt` (Sword 2), `vfx/effekseer-tktk01/Sword20.ogt` (Sword 20), `vfx/effekseer-tktk01/Sword21.ogt` (Sword 21), `vfx/effekseer-tktk01/Sword22.ogt` (Sword 22), `vfx/effekseer-tktk01/Sword3.ogt` (Sword 3), `vfx/effekseer-tktk01/Sword4.ogt` (Sword 4), `vfx/effekseer-tktk01/Sword5.ogt` (Sword 5), `vfx/effekseer-tktk01/Sword6.ogt` (Sword 6), `vfx/effekseer-tktk01/Sword7.ogt` (Sword 7), `vfx/effekseer-tktk01/Sword8.ogt` (Sword 8), `vfx/effekseer-tktk01/Sword9.ogt` (Sword 9)
- effekseer-tktk02: `vfx/effekseer-tktk02/Claw1.ogt` (Claw 1), `vfx/effekseer-tktk02/Claw2.ogt` (Claw 2), `vfx/effekseer-tktk02/Claw3.ogt` (Claw 3), `vfx/effekseer-tktk02/Claw4.ogt` (Claw 4)
- pixel-scenario: `vfx/pixel-scenario/content/crescent-sword-slash.ts` (Razor Slash), `vfx/pixel-scenario/content/dash-speed-lines.ts` (Cyclone Dash), `vfx/pixel-scenario/content/feral-claw-scratch-blood.ts` (Sanguine Claw), `vfx/pixel-scenario/content/spinning-wind-slash.ts` (Cyclone Slash), `vfx/pixel-scenario/content/triple-slash-combo-with-blue-energy.ts` (Flux Slash)

## 42. Smoke effects

Smoke effects (smoke): 25 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `effekseer-magicalxspiral`, `effekseer-pierre02`, `geometric`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/chimney-smoke.ts` (Chimney Smoke), `vfx/cartoon-2d/content/poof.ts` (Poof)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/chimney-smoke.ts` (Chimney Smoke), `vfx/cartoon-2d-rubber-hose/content/poof.ts` (Poof)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/chimney-smoke.ts` (Chimney Smoke), `vfx/cartoon-2d-storybook/content/poof.ts` (Poof)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/chimney-smoke.ts` (Chimney Smoke), `vfx/cartoon-2d-watercolor/content/poof.ts` (Poof)
- cinematic-realism: `vfx/cinematic-realism/dust.ogt` (Rolling Dust), `vfx/cinematic-realism/smoke-grenade.ogt` (Smoke Screen), `vfx/cinematic-realism/smoke-plume.ogt` (Wreck Smoke Column), `vfx/cinematic-realism/steam.ogt` (Steam Leak)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/electric_dust.ogt` (Electric Dust)
- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/foot_smoke1.ogt` (Foot Smoke 1), `vfx/effekseer-magicalxspiral/foot_smoke2.ogt` (Foot Smoke 2), `vfx/effekseer-magicalxspiral/foot_smoke3.ogt` (Foot Smoke 3), `vfx/effekseer-magicalxspiral/foot_smoke4.ogt` (Foot Smoke 4), `vfx/effekseer-magicalxspiral/foot_smoke5.ogt` (Foot Smoke 5), `vfx/effekseer-magicalxspiral/foot_smoke6.ogt` (Foot Smoke 6), `vfx/effekseer-magicalxspiral/foot_smoke7.ogt` (Foot Smoke 7), `vfx/effekseer-magicalxspiral/foot_smoke8.ogt` (Foot Smoke 8)
- effekseer-pierre02: `vfx/effekseer-pierre02/CosmicMist.ogt` (Cosmic Mist)
- geometric: `vfx/geometric/dust.ogt` (Slate Dust)
- pixel-scenario: `vfx/pixel-scenario/content/poison-gas-cloud.ts` (Blight Cloud), `vfx/pixel-scenario/content/smoke-poof.ts` (Ashen Poof)

## 43. Splash effects

Splash effects (splash): 20 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `geometric`, `realistic-starter`.

- cartoon-2d: `vfx/cartoon-2d/content/ripple.ts` (Ripples), `vfx/cartoon-2d/content/splash-small.ts` (Plop), `vfx/cartoon-2d/content/splash.ts` (Splash)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/ripple.ts` (Ripples), `vfx/cartoon-2d-rubber-hose/content/splash-small.ts` (Plop), `vfx/cartoon-2d-rubber-hose/content/splash.ts` (Splash)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/ripple.ts` (Ripples), `vfx/cartoon-2d-storybook/content/splash-small.ts` (Plop), `vfx/cartoon-2d-storybook/content/splash.ts` (Splash)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/ripple.ts` (Ripples), `vfx/cartoon-2d-watercolor/content/splash-small.ts` (Plop), `vfx/cartoon-2d-watercolor/content/splash.ts` (Splash)
- cinematic-realism: `vfx/cinematic-realism/drip.ogt` (Drip), `vfx/cinematic-realism/footstep-water.ogt` (Wading Step), `vfx/cinematic-realism/fountain.ogt` (Fountain Jet), `vfx/cinematic-realism/splash-small.ogt` (Small Splash), `vfx/cinematic-realism/splash.ogt` (Ballistic Spray), `vfx/cinematic-realism/waterfall-mist.ogt` (Waterfall Mist)
- geometric: `vfx/geometric/splash.ogt` (Facet Splash)
- realistic-starter: `vfx/realistic-starter/water-splash.ogt` (Water Splash)

## 44. Status effects

Status effects (status): 30 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`.

- cartoon-2d: `vfx/cartoon-2d/content/burning.ts` (On Fire), `vfx/cartoon-2d/content/debuff.ts` (Debuff), `vfx/cartoon-2d/content/frozen.ts` (Frozen), `vfx/cartoon-2d/content/poison.ts` (Poisoned), `vfx/cartoon-2d/content/shocked.ts` (Zapped), `vfx/cartoon-2d/content/stun.ts` (Dizzy Stars)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/burning.ts` (On Fire), `vfx/cartoon-2d-rubber-hose/content/debuff.ts` (Debuff), `vfx/cartoon-2d-rubber-hose/content/frozen.ts` (Frozen), `vfx/cartoon-2d-rubber-hose/content/poison.ts` (Poisoned), `vfx/cartoon-2d-rubber-hose/content/shocked.ts` (Zapped), `vfx/cartoon-2d-rubber-hose/content/stun.ts` (Dizzy Stars)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/burning.ts` (On Fire), `vfx/cartoon-2d-storybook/content/debuff.ts` (Debuff), `vfx/cartoon-2d-storybook/content/frozen.ts` (Frozen), `vfx/cartoon-2d-storybook/content/poison.ts` (Poisoned), `vfx/cartoon-2d-storybook/content/shocked.ts` (Zapped), `vfx/cartoon-2d-storybook/content/stun.ts` (Dizzy Stars)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/burning.ts` (On Fire), `vfx/cartoon-2d-watercolor/content/debuff.ts` (Debuff), `vfx/cartoon-2d-watercolor/content/frozen.ts` (Frozen), `vfx/cartoon-2d-watercolor/content/poison.ts` (Poisoned), `vfx/cartoon-2d-watercolor/content/shocked.ts` (Zapped), `vfx/cartoon-2d-watercolor/content/stun.ts` (Dizzy Stars)
- cinematic-realism: `vfx/cinematic-realism/burning.ogt` (Burning), `vfx/cinematic-realism/debuff.ogt` (Sinking Strands), `vfx/cinematic-realism/dissolve.ogt` (Ash Dissolve), `vfx/cinematic-realism/frozen.ogt` (Frozen), `vfx/cinematic-realism/poison.ogt` (Poisoned), `vfx/cinematic-realism/shocked.ogt` (Shocked)

## 45. Ui effects

Ui effects (ui): 26 in `cartoon-2d`, `cartoon-2d-rubber-hose`, `cartoon-2d-storybook`, `cartoon-2d-watercolor`, `cinematic-realism`, `effekseer-andrewfm01`, `geometric`, `pixel-scenario`.

- cartoon-2d: `vfx/cartoon-2d/content/confetti-rain.ts` (Confetti Rain), `vfx/cartoon-2d/content/confetti.ts` (Confetti Pop), `vfx/cartoon-2d/content/fireworks.ts` (Firework)
- cartoon-2d-rubber-hose: `vfx/cartoon-2d-rubber-hose/content/confetti-rain.ts` (Confetti Rain), `vfx/cartoon-2d-rubber-hose/content/confetti.ts` (Confetti Pop), `vfx/cartoon-2d-rubber-hose/content/fireworks.ts` (Firework)
- cartoon-2d-storybook: `vfx/cartoon-2d-storybook/content/confetti-rain.ts` (Confetti Rain), `vfx/cartoon-2d-storybook/content/confetti.ts` (Confetti Pop), `vfx/cartoon-2d-storybook/content/fireworks.ts` (Firework)
- cartoon-2d-watercolor: `vfx/cartoon-2d-watercolor/content/confetti-rain.ts` (Confetti Rain), `vfx/cartoon-2d-watercolor/content/confetti.ts` (Confetti Pop), `vfx/cartoon-2d-watercolor/content/fireworks.ts` (Firework)
- cinematic-realism: `vfx/cinematic-realism/celebration.ogt` (Satin Confetti), `vfx/cinematic-realism/fireworks.ogt` (Firework), `vfx/cinematic-realism/reward-burst.ogt` (Reward Fountain)
- effekseer-andrewfm01: `vfx/effekseer-andrewfm01/actionLines_halo.ogt` (Action Lines Halo), `vfx/effekseer-andrewfm01/actionLines_halo2.ogt` (Action Lines Halo 2), `vfx/effekseer-andrewfm01/actionLines_radial.ogt` (Action Lines Radial), `vfx/effekseer-andrewfm01/actionLines_radial2.ogt` (Action Lines Radial 2), `vfx/effekseer-andrewfm01/actionLines_shockwave.ogt` (Action Lines Shockwave), `vfx/effekseer-andrewfm01/actionLines_strip.ogt` (Action Lines Strip)
- geometric: `vfx/geometric/celebration.ogt` (Coral Confetti)
- pixel-scenario: `vfx/pixel-scenario/content/critical-hit-damage-999.ts` (Ember Damage Number), `vfx/pixel-scenario/content/fireworks-celebration.ts` (Arcane Fireworks), `vfx/pixel-scenario/content/gold-coins-loot-drop.ts` (Lucky Loot), `vfx/pixel-scenario/content/sparkle-twinkle.ts` (Sacred Sparkle)

## 46. Vehicle effects

Vehicle effects (vehicle): 11 in `cinematic-realism`.

- cinematic-realism: `vfx/cinematic-realism/afterburner.ogt` (Afterburner), `vfx/cinematic-realism/boost.ogt` (Booster Burst), `vfx/cinematic-realism/car-exhaust.ogt` (Road Exhaust), `vfx/cinematic-realism/damage-smoke.ogt` (Damage Smoke), `vfx/cinematic-realism/downwash.ogt` (Rotor Downwash), `vfx/cinematic-realism/exhaust.ogt` (Ion Exhaust), `vfx/cinematic-realism/thruster.ogt` (Attitude Jet), `vfx/cinematic-realism/tire-smoke.ogt` (Tyre Smoke), `vfx/cinematic-realism/wake.ogt` (Hull Wake), `vfx/cinematic-realism/warp.ogt` (Jump Flash), `vfx/cinematic-realism/wheel-dust.ogt` (Wheel Dust)

## 47. Weather effects

Weather effects (weather): 7 in `effekseer-magicalxspiral`, `effekseer-nextsoft01`, `effekseer-pierre01`, `effekseer-starter`, `effekseer-suzuki01`, `pixel-scenario`.

- effekseer-magicalxspiral: `vfx/effekseer-magicalxspiral/Snowman1.ogt` (Snowman 1)
- effekseer-nextsoft01: `vfx/effekseer-nextsoft01/MagicTornade.ogt` (Magic Tornade)
- effekseer-pierre01: `vfx/effekseer-pierre01/HolySandstorm.ogt` (Holy Sandstorm)
- effekseer-starter: `vfx/effekseer-starter/HolySandstorm.ogt` (Sandstorm)
- effekseer-suzuki01: `vfx/effekseer-suzuki01/snowstorm11.ogt` (Snowstorm 11)
- pixel-scenario: `vfx/pixel-scenario/content/rain.ts` (Aqua Ambient), `vfx/pixel-scenario/content/snow-falling.ts` (Glacial Ambient)
