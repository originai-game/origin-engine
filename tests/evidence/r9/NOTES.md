# R9: characters (blendshapes, garments, GPU crowds)

## Batch 5: GPU crowds (og.CrowdAnimation)

`crowd/` is 400 template walkers on a grass plain, 20 × 20, 1.2 m apart,
each turned its own way and playing the walk on the GPU
(`og.CrowdAnimation`) at its own `offset` (0–4 s) and `speed` (0.8–1.2),
under a sun from the upper right. One more walker at the front, `solo`,
is posed by og.anim (og.Skeleton and og.AnimationPlayer) for comparison.

The walk is baked once, shared by all 400: 12 joints × 61 frames (2 s at
30 frames a second) × 64 bytes = 46,848 bytes in the palette buffer a
frame (twice under TAA, plus 64 bytes an instance for its previous
transform: 25.6 KB for 400). The palette buffer holds at most 65,536
slots (4 MiB, both sections; `MAX_JOINTS`), far under the 128 MiB
storage-binding limit; under TAA that is about 30,000 crowd members a
frame of a few such clips.

Without a renderer it runs to `tick 12 hash 58f273f047b8c153 entities
403` (OG_RENDERER=none on the orb); the crowd is presentation only.

Commands (macrunner, a native dev build):

```sh
og run tests/evidence/r9/crowd --headless --size 1280x720 --out /tmp/r9 \
  --commands "0.5:screenshot:crowd;1.0:screenshot:crowd-later;1.1:quit"
```

`crowd` should show:

- all 400 walkers standing on the plain, filling the view from the front
  rows to the far ones, lit from the right with shadows on the grass;
- their legs and arms at many different phases of the walk (out of step),
  none in the T or bind pose, none stretched or exploded (the shader
  blends two baked frames; a wrong slot would tear limbs);
- `solo` at the front centre walking like the others.

`crowd-later` (half a second on) should show the same walkers at other
phases, standing where they stood (they walk in place: no root motion).

og-tests frame (`effects::models::baked_clips_pose_a_crowd_like_their_palettes`,
an evidence frame through `save`): `crowd-baked`, 35 pink cylinders on a
white ground in 7 × 5 rows, each bent to the left by its own amount (a
3-frame clip bending 0, 0.25 and 0.5 a bone, at frames 0.37 apart). The
test also asserts a baked instance at frames 1, 1.5 and 2 draws within 40
pixels of the same pose through a palette.
