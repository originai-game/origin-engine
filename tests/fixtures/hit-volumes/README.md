# Hit volumes

og.play's mechanisms on the real stack: an og.HitVolume on a posed bone
(og.BoneAttachment on the walker's `hand.L`) swept between ticks, and
og.play.candidates with sight.

The walker turns half a radian a tick, so its hand moves farther in a tick
than its volume is thick. Two thin dummies (og.HurtVolume boxes 2 cm
thick) and a thin wall collider on layer 2 stand on the hand's circle: no
tick's pose touches them, and only the sweep along the turn between two
poses meets them. Each whole turn is a new swing, so each target is struck
once a turn and never twice in one (`dups`). Two levers (a game component,
`Lever`) stand 3 m away, one behind the wall: og.play.candidates lists
both, nearest first in entity order, the hidden one not visible.

Readers: its `tests/swing.og.ts`. og-play's unit tests hold the sweep,
the contacts and the swings' state.

Inputs: og-play, og-runtime, og-anim, og-physics, packages/engine; the
walker is cook-assets' `walker.glb` (og-assets.json).
