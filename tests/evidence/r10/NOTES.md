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

## Batch 2: virtual cameras

`vcams/` is a room with a runner (a blue pillar tweened from x -4 to 4
and back every 3 s) and four og.VirtualCamera shots that a `Director`
system enables and disables at fixed ticks:

- "wide", priority 10: a high corner shot looking at the runner. It is
  live from the first tick; the first live shot cuts.
- "close", priority 20, enabled at 2 s: follows the runner from the
  front left (`smoothTime` 0.2). The brain blends to it over 1.5 s,
  `cubicInOut`.
- "side", priority 30, enabled at 4.5 s with a 1.5 s linear blend, then
  disabled at 5 s, a third of the way in. The brain turns back to "close"
  from the pose the camera showed (og.CameraBrain.from is 0, a held pose).
  In the clip the frame-to-frame change runs 545, 440, 241 (mean absolute
  difference, frames 149 to 151) across that tick: no jump.
- "chase", priority 40, enabled at 7 s with `blendTime` 0 (a cut): behind
  the runner with `collide` on (`radius` 0.25, `damping` 0.45 s). When the
  runner passes behind the red wall, the shot pulls in front of it at
  once and eases back out: its `pull` reads 0 at 7.2 s, 2.909 m at 7.7 s,
  0.935 m at 8.5 s and 0.301 m at 9 s.

The run ends on `tick 720 hash afd0317a057016a4 entities 13` both on the
Mac satellite (metal) and headless in the orb (linux-x64, no renderer).

```sh
og run tests/evidence/r10/vcams --headless --size 1280x720 --out /tmp/r10v \
  --commands "1.0:screenshot:a-wide;2.8:screenshot:b-blend;3.8:screenshot:c-close;4.95:screenshot:d-side-mid;5.4:screenshot:e-turned-back;7.2:screenshot:f-chase-clear;7.7:screenshot:g-pulled-in;8.5:screenshot:h-easing-out;12:quit"
og render tests/evidence/r10/vcams --commands "12:quit" --fps 30 --from 0 --to 12 \
  --size 1280x720 --out vcams.mp4
```

Frames in the showcase archive (feature R10): `a-wide`, `b-blend`
(halfway to "close"), `c-close`, `d-side-mid`, `e-turned-back`,
`f-chase-clear`, `g-pulled-in` (the camera in front of the wall, the
runner in view), `h-easing-out`, and `vcams.mp4`, the whole 12 s.
