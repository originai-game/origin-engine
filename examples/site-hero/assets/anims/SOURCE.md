# Traveller actions

## brace.glb (the `brace` clip, actions/brace.ogt)

Made through the Origin platform's generation gateway (origin MCP,
OG_PLATFORM_DEVELOPMENT_KEY) on 2026-11-03 with Meshy text-to-motion (prime
mode, 5 s), then applied to the traveller's Meshy rig. Generated motion; no
motion capture of a real person, no likeness.

- Prompt: "Starting from a neutral standing pose with arms at his sides, a
  man sidesteps once to widen his stance, lifts his right hand to grip his
  hood at the side of his head, and holds that pose motionless; finally the
  hand drops back to his side."
- Motion: operation traveller-brace-meshy-2026-11-03-v11, task 01a1a248-19fa-7373-8f7f-97a18ca26f04
- Applied to the rig task 01a1a0ca-a89b-718b-95eb-478d5f4fc081: operation
  traveller-brace-apply-2026-11-03-v11, task 01a1a24a-806c-752f-9d41-b5da5ce883ac,
  GLB media 8b889d0f6194bbb564f554cf144d0f62 (30 Hz, 4.97 s, 72 channels).
- Export: the skeleton, its skin and the one animation, renamed `brace`;
  the mesh, materials and textures dropped (18.2 MB to 0.2 MB).

Measured from the joints (forward kinematics over the clip): the right hand
rises 0.4–1.0 s and holds beside the head; the left arm stays down; the
hips move under 3 cm over the hold. Hold 1.4–3.6 s: the right wrist within
1.5 cm of (-0.085, 0.057, -0.023) m along the head's axes (the sidecar's
handTarget). The clip widens the stance only slightly (feet 0.28 to 0.30 m).

Thirteen variants were generated (v1–v13). The others were rejected:
v1 (the first, task 01a1a1e0-34f3-7208-90d6-621efe7fb63f) travels 0.45 m,
raises both hands and crouches; v3 has no relaxed entry; v4 holds a pose
from the first frame; v5–v6 raise both arms; v7 drifts the pelvis 10 cm in
the hold; v8 crouches; v9 starts in the hold; v10, v12 and v13 step and then
drift 25–65 cm or walk away.
