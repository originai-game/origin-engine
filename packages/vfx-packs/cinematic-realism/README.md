# Cinematic realism

89 visual mechanisms for `cinematic-realism` and its `muted` variant.
Install with `og vfx add cinematic-realism`; spawn
`vfx/cinematic-realism/<name>.ogt`. This downloadable pack adds nothing to
a game that does not install it. There are no game rules, damage, rewards
or automatic UI actions: an effect draws, lights, shakes the camera and
plays its sound, and the game decides when and where. Camera impulses are
decayed by the game's `og.Camera.traumaDecay` setting.

## Weapons and projectiles

| Effect | Placement and scale |
|---|---|
| muzzle | Origin at the barrel, +y forward; a flame tongue with side petals, then gun smoke |
| muzzle-heavy | As muzzle, for a cannon, a shotgun or an anti-materiel rifle: a metre of flame, a side fan and a cloud pushed ahead |
| muzzle-auto | Loop, one shot every 0.1 s (600 rounds a minute) with its report; despawn it when the trigger is let go |
| muzzle-suppressed | A puff of gas and no flash |
| shell | Origin at the ejection port, +y forward; one brass case thrown to +x, bouncing where it lands |
| tracer | Loop, moved by the game, +y forward; a 3 m streak behind the origin. Param `length` (1) multiplies it |
| rocket | Loop, moved by the game, +y forward; motor flame and a smoke trail left in the world |
| grenade-trail | Loop, moved by the game; a thin grey thread |
| bolt | Loop, moved by the game, +y forward; a blue-white energy bolt with a short tail |
| laser | Loop, origin at the emitter, +y along the beam. Param `length` (10) is its reach in metres; spawn `shield-hit` or `impact` at its end |
| flamethrower | Loop, origin at the nozzle, +y forward; a burning stream about 6 m long that lifts and smokes at its end |

## Impacts and marks

| Effect | Placement and scale |
|---|---|
| impact | Steel: origin on the surface, +y its normal; sparks and dust, and a sooted dent with a bare rim for 10 s |
| impact-concrete | Dust, chips and grit; the chipped pit |
| impact-dirt | A brown spray and clods; a small dark crater |
| impact-wood | Splinters and sawdust; a gouge along local +x |
| impact-glass | Shards and glints; a fracture star |
| impact-water | A few drops straight up and a ripple; origin on the water plane |
| impact-flesh | Cloth dust and fibres; no blood, no mark |
| shield-hit | Origin on an energy shield, +y its normal; two rings of charge and flecks |
| slash | Origin at the swing's pivot, the arc ahead of +x in a plane tilted 20° about x (a diagonal cut from high at -z to low at +z; rotate the effect for another), radius 1 m (scale it to the blade) |
| mark-bullet | Only the chipped pit, for 30 s |
| mark-wet | Only a 0.9 m wet splash patch, for 30 s |
| mark-burn | Only a 1.6 m burn with embers, for 30 s |
| skid | Only a tyre mark 1.4 m along local +x and 0.24 m wide, for 20 s; lay one per 1.2 m travelled |

## Explosions

| Effect | Placement and scale |
|---|---|
| explosion | Origin on the ground; a 3 m fireball, debris, dust skirt and a smoke column to 9 m; an 8 m soot mark with embers |
| explosion-small | A grenade: a 1.5 m fireball and a 3 m mark |
| explosion-large | A bomb: an 8 m fireball, a racing wall of dust, a column to 20 m and an 18 m mark |
| explosion-air | Origin in the air; a fireball, fragments drawing smoke and a hanging dark puff; no mark |
| explosion-water | Origin on the water plane; a 10 m white column, a sheet of drops, foam and long ripples |
| explosion-energy | A blue-white shell that tears open, a ground ring and a mark with cold embers |
| smoke-grenade | A screen fed for 8 s that stands about 5 m wide and thins by 18 s |
| flashbang | A white flash, magnesium sparks and a small puff |

## Vehicles

| Effect | Placement and scale |
|---|---|
| exhaust | Loop at the nozzle, +y downstream; an ion engine's hot jet and cooler wake |
| afterburner | Loop at the nozzle, +y downstream. Param `thrust` (1) thins the flame and shortens its glow |
| thruster | A cold-gas attitude jet, +y out of the nozzle, a quarter of a second |
| car-exhaust | Loop at the pipe, +y out; thin fumes |
| tire-smoke | Loop at the contact patch. Param `slip` (1) is how much |
| wheel-dust | Loop at the contact patch on a loose surface. Param `amount` (1) |
| boost | Origin at the nozzle, +y downstream; a half-second blue-to-orange burst |
| damage-smoke | Loop at the damage; black smoke. Param `fire` (0): above 0 adds flames, sparks and a flickering light |
| wake | Loop at the stern on the water plane; foam, spray and mist left behind a moving hull |
| downwash | Loop on the ground under a rotor or a landing jet; dust driven out past 5 m. Param `amount` (1) |
| warp | Origin at the craft, +y its heading; a gathering glow, then streaks and a 12 m flash ahead at 0.25 s |

## Fire, smoke and water

| Effect | Placement and scale |
|---|---|
| campfire | Loop; flames half a metre high, embers, thin smoke and a flickering light |
| fire-large | Loop; flames to 2.5 m over a 1.5 m base, a black column |
| smoke-plume | Loop; a wreck's column, 15 m and more, leaning with og.Wind |
| steam | Loop at the leak, +y out; a hard jet that billows |
| electric-sparks | Loop; two arcing faults every 2 s with sparks and a blue flash |
| embers | Loop over 4 x 4 m; embers on the air |
| dust | Origin on the ground; rolling dust, not an opaque smoke wall |
| splash | Origin on the water plane; a torn crown sheet, droplets, a central jet, foam, mist and ripples |
| splash-small | A stone's splash, about 0.3 m |
| footstep-water | A wading step: low spray, foam and a ripple |
| waterfall-mist | Loop at the foot of a fall about 2 m wide (local x): mist, spray and foam |
| drip | Loop where the drop lands; one drop every 1.5 s falls from 2.2 m above, a crown, a ripple and a wet patch where it lands |
| fountain | Loop at the nozzle, +y up; a jet 1.7 m high |

## Characters and status

| Effect | Placement and scale |
|---|---|
| footstep-dust | Origin under the foot |
| landing | Origin under the feet; a ring of dust and grit |
| dash | Origin at the chest, +y along the dash |
| teleport | Origin at the feet; a shaft, a ground ring and rising flecks at 0.15 s |
| dissolve | Origin at the feet of a 1.8 m figure going to ash over 1.2 s; hide the figure as it plays |
| heal | Origin at the feet; a green-white ring, flecks and a soft light |
| buff | Origin at the feet; two warm strands climbing round the figure |
| debuff | Two violet strands sinking round it, cold mist |
| burning | Loop; parent it to the figure (origin at the feet) |
| poison | Loop; yellow-green vapour and drops |
| frozen | Loop; sinking cold mist and ice glints |
| shocked | Loop; arcs across the body four times in 0.6 s |
| level-up | Origin at the feet; a warm shaft, two ground rings and flecks |

## Interaction and destruction

| Effect | Placement and scale |
|---|---|
| pickup | Origin at the object; amber motes and a brief warm pulse |
| beacon | Loop; a shaft of warm light 6 to 9 m high over a marker |
| loot-glow | Loop; a short shaft and two strands circling a drop |
| chest-open | Origin at the chest's mouth; rays, motes and lid dust at 0.1 s |
| break-wood | Origin at the foot of a 1 m crate; boards, splinters and dust |
| break-glass | Origin at the foot of a 1 m pane facing z; it drops and scatters |
| break-stone | Origin at the foot of the break; blocks, chips and a grey cloud |

## Magic

| Effect | Placement and scale |
|---|---|
| magic | Origin at the source; cool turbulent wisps and a lit core |
| charge | Loop at the hand or the muzzle; motes drawn in to a pulsing core |
| magic-circle | Origin on the ground; a 2.4 m drawn circle that glows for 4 s |
| magic-bolt | Loop, moved by the game; a violet bolt with a tail |
| barrier | Loop, origin at the feet; a dome 2.6 m across |
| portal | Loop, origin on the ground; an upright ring 2.2 m across facing z |

## Ambience and UI

| Effect | Placement and scale |
|---|---|
| leaves | Loop over 4 x 4 m, falling from 3 m |
| fireflies | Loop over 3 x 3 m around 0.8 m up |
| ground-fog | Loop over 6 x 6 m; fog lying low |
| sunbeam-dust | Loop in a 2 x 3 x 2 m volume; place it in a shaft of light |
| celebration | Origin at the UI anchor projected into the scene; falling satin confetti |
| fireworks | Origin on the ground; a rocket that bursts 11 m up at 0.8 s |
| reward-burst | Origin at the reward; bright discs thrown up, glints and motes |

## Marks

The effects that strike a surface project their marks down the effect's -y
onto whatever lit surface is there, with the pack's own textures
(`textures/scorch.png`, `embers.png`, `chip.png`, `gouge.png`,
`fracture.png`, `skid.png`, `sigil.png` and their normal maps, generated by
`scripts/vfx-marks.py` in the engine source). Where an effect has one, its
first param, `mark` (og.Vfx.params[0], 1 by default), is 0 for an instance
that should leave none: a hit on a character, a blast in the air. A game
that wants marks to stay places its own og.Decal with the same textures.

## Sounds

One-shot effects play a sound of the pack (`content/audio/*.ts`, `file.sfx`:
oscillator and noise layers the cook synthesises; no recording ships).
They are plain and dry so a game's own mix and reverb place them; a game
with recorded audio plays its own and sets og.AudioSettings' sfx bus or
removes the `sound` lines from its installed copy. Continuous loops are
silent: a looping bed is the game's og.AudioSource. `muzzle-auto` is the
exception, one report per cycle.

## Moving and scaling

The loops marked "moved by the game" are not projectiles: the game moves
and turns the instance, and what they leave in the world (smoke, flecks)
stays behind. og.vfx.spawn's `scale` sizes any effect; `emission` thins
its particles for a distant or a cheap one; `hue` turns the energy and
magic colours. `celebration`, `fireworks` and `reward-burst` are
world-space visuals, not UI components. The neutral colours and low light
energy leave exposure and the scene's single accent under the game's
control.

Explosion and dust use Thomas Iché / Unity Labs' Houdini simulations,
released CC0 at https://unity.com/blog/engine-platform/free-vfx-image-sequences-flipbooks.
See `third_party/unity-vfx/LICENSE` in the engine source and each file's
provenance in `assets.json`. The effect compositions are MIT, Origin Engine.
