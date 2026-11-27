# realistic-starter

Seven realistic effects for 3D games, written by hand for Origin Engine as
`file.vfx` declarations with their emitters (`file.particles`) and one effect
material (`file.effect`):

| Effect | What | Spawn it |
|---|---|---|
| `muzzle-flash` | a gunshot's flash, burning plume, powder sparks and smoke | at the muzzle, turned with the gun: it fires along its -z |
| `bullet-impact` | sparks, chips, a dust puff and the mark of a bullet on a hard surface | at the hit, its +y along the surface's normal |
| `explosion` | a fireball, a blast ring, fragments, debris, ground dust, rising smoke, a scorch, light and a shake | on the ground |
| `dust-burst` | the dust, rubble and grit where something breaks or falls | on the ground under what broke |
| `water-splash` | droplets, a spray column, mist and spreading rings | on the water's surface |
| `torch-fire` | looping upright flames, embers, smoke and a flickering light | at the top of the torch |
| `ambient-dust` | looping dust motes in a 4 × 2 × 4 m volume | at the middle of the room |

`og vfx add realistic-starter` copies them into
`assets/vfx/realistic-starter/`; each one-shot sends its `impact` at its
first frame. Smoke and dust are lit and drift with the wind (og.Wind);
sparks, chips and droplets bounce off or die on the drawn depth.

The textures under `textures/` are made from the Kenney Particle Pack
(https://kenney.nl/assets/particle-pack, CC0 1.0): smoke, fire, flame,
muzzle-flash, dirt, scorch and light images, converted to
greyscale with alpha and packed into sheets (`smoke.png` 2 × 2,
`fire.png` 2 × 1, `flame.png` 2 × 2, `muzzle.png` 2 × 2, `dirt.png`
3 × 1). Credit: Kenney (www.kenney.nl).
