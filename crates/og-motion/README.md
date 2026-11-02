# og-motion

Motion generation for humanoids (`docs/design.md` §2.7, planned): a
pose-conditioned motion model, a 17.6M-parameter student distilled from
NVIDIA MotionBricks (SOMA v6, 189M) in the og-motion-train repository. Given
each character's four most recent poses and one to four target poses, it
returns the motion between them: 6 to 16 tokens of four frames at 30 Hz,
with per-frame foot contacts. Its poses are presentation (§2.4).

```rust
let bundle = og_motion::Bundle::from_safetensors(&bytes)?;
let model = og_motion::Model::new(&bundle)?;                 // CPU
let (generation, alignments) = model.generate_requests(&requests);
let pose = og_motion::decode_frame(generation.frame(0, 10), &alignments[0]);

let gpu = og_motion::gpu::Gpu::new(device, queue, &bundle)?; // wgpu compute
let student = gpu.student(&og_motion::glue::student_inputs(&statics, &inputs)).await?;
```

## API map

| Module | Entry points |
|---|---|
| `bundle` | `Bundle::from_safetensors` (F32 or F16 tensors, `config` metadata), `get`, `parameters`, `Config` |
| `contract` | `Pose`, `Request`, `encode` (world poses → the seven inputs and an `Alignment`), `decode_frame`, `valid_frames`, the joint names and feature ranges |
| `glue` | `Statics`, `student_inputs`, `decoder_inputs`, `finish`: everything around the two networks, shared by both paths |
| `cpu` | `Cpu`: the student and the decoder in f32 loops, the reference path |
| `gpu` | `Gpu` (`request`, `new`, async `student`, `decoder`), `shader_sources` |
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
- `tests/gpu.rs`: the GPU path against the CPU path (needs a GPU adapter).
- `examples/bench.rs`: times a trained bundle and checks it against a case
  file. On the trained run2b bundle, the CPU and GPU paths agree with
  `ogm/reference.py` exactly in tokens and counts and within 7e-5 in motion.

## State

Measured on the Mac satellite's "Apple Paravirtual device" (Metal, about
230 GFLOP/s for these kernels, not an M4 Pro's throughput): 36 ms for one
character, 6 ms a character from 32 up; the student is about 1.5 GFLOP a
character and takes 80% of it. Numbers on the runners' GPUs are owed. Open:
reusing buffers and bind groups across calls (small batches are dispatch
bound), a CPU path fast enough to stand in where there is no adapter, f16
storage, and the intent API, retargeting and foot planting in og-anim.
