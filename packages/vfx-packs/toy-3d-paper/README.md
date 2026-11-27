# Paper 3D effects

Install with `og vfx add toy-3d-paper`; spawn `base/vfx/toy-3d-paper/<name>`
with og.vfx.spawn. Paper-craft effects for toy-3d's paper variant: thin cut card in soft, slightly warm colours: cloud cutouts for smoke and dust that stand facing the viewer, card squares and punched dots for debris, flat stars, hearts and flames that tumble as they fall.

Every effect is solid matter, the way a toy world is built: puffs, pieces,
drops and flame shapes drawn as small lit mesh particles (`file.particles`
with a mesh `render`), with ring meshes and point lights in each
`file.vfx` (`content/<name>.ts`). Sizes are in metres for a figure about
1 m tall. The same names, timing and placement are in every toy pack
(`toy-3d`, `toy-3d-paper`), so a game
changes look by changing the pack. Effects that play once send their
impact at the time in `assets.json`. No gameplay, scoring, damage or UI
logic, and no sound.

| Effect | Plays | Placement |
|---|---|---|
| explosion | once | on the ground |
| explosion-small | once | on the ground |
| poof | once | centred on what vanishes or appears |
| hit | once | at the point struck |
| hit-small | once | at the point struck |
| hit-heavy | once | at the point struck |
| stomp | once | on the ground under the foot |
| stun | loops | above the head |
| muzzle | once | at the muzzle, +y along the barrel |
| bullet | loops | on the pellet, moved by the game |
| rocket | loops | at the rocket's tail, -y behind it |
| magic-bolt | loops | on the bolt, moved by the game |
| laser | once | from the emitter along +y, 4 m |
| coin | once | where the coin was |
| gem | once | where the gem was |
| star-collect | once | where the star was |
| heart | once | where the heart was |
| powerup | once | on the ground at the figure's feet |
| chest-open | once | in the chest's mouth |
| checkpoint | once | on the ground at the flag |
| level-up | once | on the ground at the figure's feet |
| reward-burst | once | where the reward is shown |
| magic | once | at the point struck |
| charge | loops | where the power gathers |
| magic-circle | once | on the ground |
| portal | loops | its foot on the ground, facing +z |
| teleport | once | on the ground at the figure's feet |
| shield | loops | centred on the figure |
| heal | once | on the ground at the figure's feet |
| buff | once | on the ground at the figure's feet |
| debuff | once | above the figure's head |
| burning | loops | on the figure, up to 1 m |
| poison | loops | on the figure |
| frozen | loops | on the figure |
| shocked | loops | on the figure |
| dust | once | on the ground |
| run-dust | loops | at the foot, moved with the figure |
| jump-dust | once | on the ground under the jump |
| land-dust | once | on the ground under the landing |
| double-jump | once | under the figure, in the air |
| dash | once | behind the figure, +z the way it dashes |
| skid | loops | at the sliding foot or wheel, moved by the game |
| chimney-smoke | loops | at the chimney's top |
| steam | loops | at the vent, +y the way it blows |
| splash | once | on the water's surface |
| splash-small | once | on the water's surface |
| ripple | loops | on the water's surface |
| bubbles | loops | under water, rising |
| drip | loops | the drop falls 1.5 m to the origin |
| fountain | loops | at the spout, +y up |
| wake | loops | at the stern on the water, moved by the game |
| campfire | loops | on the ground |
| torch | loops | on the torch head |
| fuse | loops | at the fuse's end |
| exhaust | loops | at the exhaust pipe, +y out of it |
| boost | once | at the exhaust pipe, +y out of it |
| drift-sparks | loops | at the rear wheel on the ground, moved by the game |
| tire-smoke | loops | at the wheel on the ground, moved by the game |
| wheel-dust | loops | behind the wheel on the ground, moved by the game |
| afterburner | loops | at the nozzle, +y out of it |
| break-block | once | centred on the block |
| break-crate | once | centred on the crate |
| break-rock | once | centred on the rock |
| break-ice | once | centred on the ice |
| build-place | once | on the ground under the placed piece |
| build-remove | once | centred on the removed piece |
| confetti | once | on the ground, +y up |
| confetti-rain | loops | 3 m above the area it covers |
| fireworks | once | on the ground: it bursts 6 m up |
| twinkle | loops | on what sparkles |
| leaves | loops | 3 m above the area it covers |
| fireflies | loops | over the area, 0.2 to 1.5 m up |

All files are original MIT work, Origin Engine, written by
`scripts/vfx-toy.ts`: the effects and emitters from one table of recipes,
the models (`models/*.glb`) from its own geometry (extrusions of star,
heart, coin, gem and drop outlines; lathed drops and flames; a
superellipsoid block; flat card cutouts), with no external images, models or
code.
