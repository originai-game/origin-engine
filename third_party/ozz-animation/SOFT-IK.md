# Soft two-bone reach

`og_anim::soft_ik_distance` adapts the target-softening step in ozz-animation's
`src/animation/runtime/ik_two_bone_job.cc`, `SoftenTarget`, revision
`744eb9d99f606eda849acb0b1204f7a3dc20bca1`:
https://github.com/guillaumeblanc/ozz-animation/tree/744eb9d99f606eda849acb0b1204f7a3dc20bca1

Ozz is MIT licensed; its licence is retained beside this note. Its source cites
Andy Nicholas's Softimage article (http://www.softimageblog.com/archives/108,
unavailable when checked 2026-11-04) and Ryan Juckett's analytic two-bone IK.
The scalar engine implementation uses the original exponential softened-reach
law, not ozz's SIMD rational approximation:

- The onset is the sum of bone lengths minus the softness distance.
- Below onset, preserve the requested distance exactly.
- Above onset, approach full reach exponentially, with unit slope at onset:
  `onset + softness * (1 - exp(-(distance - onset) / softness))`.
- Zero softness disables the adjustment. Use og-core's deterministic
  exponential, never the platform libm in engine code.

The mathematical law is also described in Nicky Liu's
https://www.nickyliu.com/2025/02/08/soft-ik-again/ and the Andy Nicholas adaptation
on Epic's community site:
https://dev.epicgames.com/community/learning/tutorials/y4Ll/unreal-engine-control-rig-5-3-soft-ik
No Unreal code or Toolchefs' LGPL solver source is included.

This function does not stretch bones or move a simulation root. A softened
target is intentionally not the original target. The flight experiment first
tries to accommodate the 97% onset (ozz sample default) with its existing
bounded presentation-pelvis allowance. If it cannot, it reports the original
contact error rather than declaring the softened target a successful plant.
The onset is not an approved new contact tolerance or expanded pelvis budget.
