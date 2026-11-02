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

`examples/bench.rs` with the run2b bundle (2026-11-02, release, warm; the
student is about 1.5 GFLOP a character). Before the kernel work below, on
the runners:

| GPU | 1 character | 8 | 32 | 256 (per character) |
|---|---|---|---|---|
| RTX 5060, Vulkan (studio-win) | 15.7 ms | 30.6 ms | 48.1 ms | 226 ms (0.88) |
| RTX 5060, DX12 | 17.5 ms | 25.6 ms | 41.3 ms | 209 ms (0.82) |
| M4 Pro 16-core, Metal (macrunner, quiet) | 29.6 ms | 63.8 ms | 233 ms | 1867 ms (7.3) |

Metal ran at about 205 GFLOP/s, an M4 Pro's or a paravirtual GPU's alike:
wgpu's forced loop bounding (on by default for every shader module) cost
the linear kernel ten times its speed on Metal. The modules are now made
with `create_shader_module_trusted` without it, keeping bounds checks (the
kernels' loops are bounded by uniforms they check); with that, a `Plan`
per batch size (buffers, staging and bind groups made once, a call only
writes its inputs), and per-row layer norm and per-query attention
kernels, the Mac satellite's paravirtual GPU does one character in 17 ms,
32 in 1.6 ms a character and 256 in 0.67 ms a character (was 36 ms and 6
ms), and the demo course in 238 ms. One character is latency bound: each
of the student's dependent dispatches costs about 0.1 ms there whatever
its size (measured by truncating the pass), so fewer, fused kernels are
what would cut it. `Gpu::request` on DX12 took 26–30 s with wgpu's FXC
fallback; it now uses static DXC on Windows MSVC, as og-render. The
runners' numbers after this work are owed. Open: kernel fusion for one
character, a CPU path fast enough to stand in where there is no adapter,
f16 storage, and the intent API, retargeting and foot planting in og-anim.
