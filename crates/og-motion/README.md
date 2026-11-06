# og-motion

Motion generation for humanoids (`docs/design.md` §2.7): a
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
| `gpu` | `Gpu` (`request`, `request_on`, `new`, `set_blocking`, `set_frame_budget` (a generation in slices of about that much GPU time, one per poll, cut by each dispatch's estimated work and the rate timestamp queries measure), `rate`, async `student`, `decoder`, `generate`), `shader_sources` |
| `intent` | `Goal` (`placed`), `Motion` (`add`, `push`, `batch`, `accept`, `generate`, `generate_gpu`, `pop`), `Frame` |
| `soma` | `Skeleton` (`from_bundle`, `asset`, `local`), `plant_feet`, `Planter` (frame by frame, anchors in the world), `clip` |
| (root) | `Model` (CPU end to end), `Generation` |

## The bundle

One safetensors file written by og-motion-train's `scripts/export.py`
(`og-motion/1`): `student.*` and `decoder.*` in PyTorch state-dict names,
the teacher's frozen `codebook` [5,16,32], motion statistics `stats.mean`
and `stats.std` [370] and T-pose offsets [30,3,3]. 70.6 MB in f32, 35 MB in
f16. The codebook, statistics and
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
  file.

## Licensing of the weights

- The test fixtures (`tests/fixtures/`) hold no trained, captured or NVIDIA
  number: og-motion-train's `scripts/make_fixture.py` writes them from a
  fixed seed (random weights, codebook, statistics and offsets; synthetic
  poses), and a rerun gives the same bytes.
- The trained student and decoder are distilled from NVIDIA MotionBricks'
  outputs, on target poses from og-motion-train's own motion library (no
  Bones sample motion, whose licence is evaluation-only). The NVIDIA Open
  Model Agreement has no clause on training with outputs ("NVIDIA does not
  claim ownership to any outputs"), and NVIDIA's permission is reported by
  the owner; it is to be kept in writing before a game ships the weights.
  The bundle carries the attribution either way.

## Runtime

The runtime's API is og-runtime's `og.motion` (`crates/og-api/api/motion.ogt`):
goals kept by the simulation, the model on og-render's device, its weights
(`og-motion-train`'s `weights/engine/model.safetensors`, f16 with the pose
library, 35.4 MB) shipped in `@origingame/engine` and cooked as
`cooked/motion/model.ogb` for a project that uses og.motion. The engine is
GPU only (docs/design.md §2.6): no CPU path stands in.

On the GPU, the shader modules are made with `create_shader_module_trusted`
without wgpu's forced loop bounding, keeping bounds checks (every kernel's
loops are bounded by the uniforms it checks); a `Plan` per batch size holds
the buffers, staging and bind groups, so a call only writes its inputs;
layer norm runs a workgroup per row and attention one per query. On DX12
the shader compiler is the static DXC, as og-render's. One character is
latency bound by the student's dependent dispatches.

The model's joint positions and its rotations on the rest bone lengths
disagree, most at the hands and feet, the teacher's as much as the
student's: `soma` shows the rotations, as MotionBricks' own runtime does,
and plants the feet.
