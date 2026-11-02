# R10: cinematics and cameras

## Batch 1: timeline tracks

`cinematics/` is one 3D room for `timelines/cinematic.ogt` (8 s). Every
track kind of batch 1 drives something you can see:

- **light** `lamp`: its og.PointLight goes from 0 to 40 (warm) by 2.5 s.
- **material** `lamp`: the sphere's emissive goes from 0 to 8 at the same
  time. This is presentation state reapplied from the timeline's time at
  extraction, so a restore mid-timeline draws the same material.
- **property** `fill` `og.DirectionalLight.intensity`, and the resource
  `og.RenderSettings.environmentIntensity`: the room goes from day to night
  by 3 s.
- **visibility** `crate`: hidden at 3.5 s.
- **post**: the vignette comes in from 0 to 0.7 by 6 s.
- **camera**: a move with `roll` (-10° at 4 s, back to 0 by 7 s), focus on
  the `hero` pillar (`focusOn`) and then a focus distance of 7.5 m (the back
  wall) from 5 s.
- **timeline**: the nested `timelines/door.ogt` starts at 4.5 s and slides
  the door open on the emissive yard.

The run is deterministic: on the Mac satellite (metal) it ends on
`tick 480 hash 995b0fea85875e57 entities 12` on every run.

Commands (the Mac satellite, a native dev build):

```sh
og run tests/evidence/r10/cinematics --headless --size 1280x720 --out /tmp/r10 \
  --commands "0:timeline:play:cinematic;0.1:screenshot:a;3:screenshot:b;4:screenshot:c;7.9:screenshot:d;8:quit"
og render tests/evidence/r10/cinematics --commands "0:timeline:play:cinematic" \
  --fps 30 --from 0 --to 8 --size 1280x720 --out cinematic.mp4
```

Frames in the showcase archive (feature R10):

- `a-start`: the room as set up: day light, the lamp dark, the pillar
  sharp and the background blurred (focus on `hero`).
- `b-lamp`: the light, material and property tracks have dimmed the room
  and lit the lamp. The crate is still there, and the camera has begun to
  roll.
- `c-roll`: the camera is rolled, and the crate is gone (visibility track).
- `d-focus-door`: the roll is back to 0, focus has moved to the back wall
  (the pillar blurs), the vignette is in, and the nested door timeline has
  opened on the yard.
- `cinematic.mp4`: the whole 8 s (240 frames in 2.7 s on metal).
