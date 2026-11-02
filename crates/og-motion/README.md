# og-motion

Motion generation for humanoids (`docs/design.md` §2.7, planned): a
pose-conditioned motion model, a 17.6M-parameter student distilled from
NVIDIA MotionBricks (SOMA v6, 189M) in the og-motion-train repository. Given
each character's four most recent poses and one to four target poses, it
returns the motion between them: 6 to 16 tokens of four frames at 30 Hz,
with per-frame foot contacts. Its poses are presentation (§2.4).

```rust
let bundle = og_motion::Bundle::from_safetensors(&bytes)?;
let model = og_motion::Model::new(&bundle)?;                 // CPU (the `reference` feature)
let (generation, alignments) = model.generate_requests(&requests);
let pose = og_motion::decode_frame(generation.frame(0, 10), &alignments[0]);

let gpu = og_motion::gpu::Gpu::new(device, queue, &bundle)?; // wgpu compute
let student = gpu.student(&og_motion::glue::student_inputs(&statics, &inputs)).await?;

// The intent API: goals per character, generated in batches, chained.
let mut motion = og_motion::intent::Motion::default();
let hero = motion.add(recent_four_frames);
motion.push(hero, Goal::placed(&library_pose, [x, z], heading));
motion.generate_gpu(&mut gpu, 30).await?;                    // characters with < 30 frames queued
let frame = motion.pop(hero);                                // a world pose and its foot contacts

// On a game's rig: SOMA as og-anim's skeleton, feet planted, retargeted.
let skeleton = og_motion::soma::Skeleton::from_bundle(&bundle)?;
let mut local: Vec<_> = frames.iter().map(|p| skeleton.local(p)).collect();
og_motion::soma::plant_feet(&skeleton.asset(), &mut local, &contacts);
let clip = og_motion::soma::clip("walk", &skeleton.asset(), &local);
let on_rig = og_anim::retarget(&skeleton.asset(), &HumanoidMap::detect(&skeleton.asset()), &rig, &rig_map, &clip, &Default::default())?;
```

## API map

| Module | Entry points |
|---|---|
| `bundle` | `Bundle::from_safetensors` (F32 or F16 tensors, `config` metadata), `get`, `parameters`, `Config` |
| `contract` | `Pose`, `Request`, `encode` (world poses → the seven inputs and an `Alignment`), `decode_frame`, `valid_frames`, the joint names and feature ranges |
| `glue` | `Statics`, `student_inputs`, `decoder_inputs`, `finish`: everything around the two networks, shared by both paths |
| `cpu` | `Cpu`: the student and the decoder in f32 loops, the reference path (the `reference` feature: tests and tools only; the engine runs the model on the GPU only) |
| `library` | `POSES` (idle, walk, run, jump, crouch, wave, cheer), `Library`: the poses og.motion's goals name, carried by the bundle |
| `gpu` | `Gpu` (`request`, `request_on`, `new`, `set_blocking`, async `student`, `decoder`, `generate`), `shader_sources` |
| `intent` | `Goal` (`placed`), `Motion` (`add`, `push`, `batch`, `accept`, `generate`, `generate_gpu`, `pop`), `Frame` |
| `soma` | `Skeleton` (`from_bundle`, `asset`, `local`), `plant_feet`, `Planter` (frame by frame, anchors in the world), `clip` |
| (root) | `Model` (CPU end to end), `Generation` |

## The bundle

One safetensors file written by og-motion-train's `scripts/export.py`
(`og-motion/1`): `student.*` and `decoder.*` in PyTorch state-dict names,
the teacher's frozen `codebook` [5,16,32], motion statistics `stats.mean`
and `stats.std` [370] and T-pose offsets [30,3,3]. 70.6 MB in f32, 35 MB in
f16 (the closed loop measures no difference). The codebook, statistics and
offsets are NVIDIA's: the bundle's `attribution` metadata ("Licensed by
NVIDIA Corporation under the NVIDIA Open Model Agreement.") travels with
any redistribution. No weights are in this repository.

## Checks

- `tests/reference.rs`: `encode` and the CPU path against og-motion-train's
  numpy specification (`ogm/reference.py`) on `tests/fixtures/`, a random
  32-wide bundle and three synthetic characters with different token counts
  (written by `scripts/make_fixture.py`; no trained, captured or NVIDIA
  number in it). Tokens and counts equal, motion within 1e-3.
- `tests/shaders.rs`: every kernel through naga to MSL, SPIR-V and HLSL (any
  machine).
- `tests/intent.rs`: goals chained through generations from each
  character's last four frames, goals placed in the world, and a heel kept
  where it touched down and let go over `RELEASE_FRAMES`.
- `tests/gpu.rs`: the GPU path against the CPU path (needs a GPU adapter).
- `examples/bench.rs`: times a trained bundle and checks it against a case
  file. On the trained run2b and run3 bundles, the CPU and GPU paths agree
  with `ogm/reference.py` exactly in tokens and counts and within 9e-5 in
  motion. run3 (the student retrained with og-motion-train's fixed count
  mask) is the bundle to ship: in og-motion-train's closed loop it reaches
  the targets as run2b does (9.16 cm against the teacher's 8.68) and chooses
  token counts as the teacher does; its weights in f16 change nothing
  measurable.

## Licensing of the weights

- The test fixtures (`tests/fixtures/`) hold no trained, captured or NVIDIA
  number: og-motion-train's `scripts/make_fixture.py` writes them from a
  fixed seed (random weights, codebook, statistics and offsets; synthetic
  poses), and a rerun gives the same bytes.
- The trained student and decoder are distilled from NVIDIA MotionBricks'
  outputs, on target poses from og-motion-train's own motion library (no
  Bones sample motion, whose licence is evaluation-only). The NVIDIA Open
  Model Agreement has no clause on training with outputs ("NVIDIA does not
  claim ownership to any outputs"), and the owner reports NVIDIA's
  permission (2026-11-02). **Open for the owner:** keep that permission in
  writing before a game ships the weights; the bundle carries the
  attribution "Licensed by NVIDIA Corporation under the NVIDIA Open Model
  Agreement." either way.

## State

`examples/bench.rs` with the run2b bundle (2026-11-02, release, warm, quiet
machines; the student is about 1.5 GFLOP a character), every token equal
to the reference on every backend:

| GPU | 1 character | 8 | 32 | 256 (per character) |
|---|---|---|---|---|
| RTX 5060, Vulkan (studio-win) | 9.5 ms | 35.4 ms | 49.6 ms | 151 ms (0.59) |
| RTX 5060, DX12 | 6.0 ms | 17.4 ms | 29.6 ms | 133 ms (0.52) |
| M4 Pro 16-core, Metal (macrunner) | 17.1 ms | 45.1 ms | 63.0 ms | 218 ms (0.85) |

The nine-step demo course (445 frames) takes 194 ms on the RTX 5060 and
175 ms on the M4 Pro. Before this round the M4 Pro took 7.3 ms a character
at 256 and the RTX 5060 0.88: wgpu's forced loop bounding (on by default
for every shader module) cost the linear kernel ten times its speed on
Metal. The modules are now made with `create_shader_module_trusted`
without it, keeping bounds checks (every kernel's loops are bounded by the
uniforms it checks); a `Plan` per batch size holds the buffers, staging and
bind groups, so a call only writes its inputs; layer norm runs a workgroup
per row and attention one per query. One character is latency bound: each
of the student's dependent dispatches costs about 0.1 ms on Metal whatever
its size (measured by truncating the pass), so fewer, fused kernels are
what would cut it. `Gpu::request` on DX12 took 26–30 s with wgpu's FXC
fallback; with static DXC, as og-render's, 1.4 s (Vulkan 0.23 s).

The model's joint positions and its rotations on the rest bone lengths
disagree, by 5 cm on average over joints and frames and up to 0.2–0.3 m at
the hands and feet, the teacher's as much as the student's (og-motion-train's
closed loop, `fk_gap_cm`): `soma` shows the rotations, as MotionBricks' own
runtime does, and plants the feet. The runtime's API is og-runtime's `og.motion` (`crates/og-api/api/motion.ogt`):
goals kept by the simulation, the model on og-render's device, its weights
(`og-motion-train`'s `weights/engine/model.safetensors`, f16 with the pose
library, 35.4 MB) shipped in `@origingame/engine` and cooked as
`cooked/motion/model.ogb` for a project that uses og.motion. The engine is
GPU only (docs/design.md §2.6): no CPU path stands in. Open: kernel fusion
for one character, f16 storage on the GPU.
