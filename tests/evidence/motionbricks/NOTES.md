# MotionBricks: evaluation for motion generation (2026-11-01)

What was measured before `docs/design.md` §2.7 named motion generation as
the direction. The evaluation was run from Amp thread
T-38e88112-c769-4e7b-ab60-ee1cd4065bc7 with the runner threads
T-01a19413-ad70-76d5-9f1e-ee12fa4e84b7 (studio-win) and
T-01a194f6-7be8-728e-8039-77d38a4cc3db (macrunner). No MotionBricks
file, weight or generated clip is in this repository.

## The model

NVIDIA MotionBricks (SIGGRAPH 2026, arXiv 2604.24833,
github.com/NVlabs/motionbricks at `54fc563`), SOMA model
`MotionBricks-SOMA-RP-v1`. Given the four most recent poses and one to
four target poses at 30 Hz, it generates the motion between them: 6 to
16 tokens of four poses, 0.6 to 1.9 s per generation, chained. Its
output also carries per-frame foot contacts (heels and toes).

- ONNX opset 17 with 189 M parameters: 363 MB FP16, 723 MB FP32. The
  graph has 6,993 nodes, but most of them only reshape tensors. The
  compute is MatMul and Gemm, LayerNormalization, Softmax (attention),
  1D Conv (kernel 3 and 1), Resize, Relu and LeakyRelu, elementwise
  ops, Gather, Scatter, CumSum, ArgMax and TopK, and a little
  trigonometry. It has no Loop and no random sampling: one generation
  is one forward pass.
- Skeleton: SOMA, 30 joints, Y up, metres. Before the `soma` scheme
  (commit `8aeda5b` on studio-win's local branch
  `motionbricks-soma-scheme`, not merged), `HumanoidMap::detect`
  matched it as `mixamo` with 18 bones. That mapped the SOMA thighs
  (`LeftLeg`, `RightLeg`) to the lower legs and left the shins unmapped.
- Licences: the code is Apache 2.0. The weights are under the NVIDIA
  Open Model Agreement: commercial use is allowed and NVIDIA claims no
  ownership of outputs. The Bones sample motions are under an
  evaluation-only licence, and every target pose in the released
  runtime comes from them. Shipped motion therefore needs target poses
  of our own. Whether the outputs may be used to train another model
  (distillation) is still to be confirmed.

## Inference timing (one generation, batch N)

| Where | Runtime | N=1 | N=32 |
|---|---|---|---|
| RTX 5060, native | TensorRT 10.8, FP16 / FP32 | 3.15 / 4.22 ms | — |
| Modal A10, native | ONNX Runtime CUDA, FP16 / FP32 | 12.0 / 11.2 ms | 14.8 / 20.0 ms |
| Orb Xeon, 8 threads | ONNX Runtime CPU, FP32 / FP16 / INT8 | 32 / 32 / 22 ms | 518 / 496 / 354 ms |
| Orb Xeon, 1 thread | ONNX Runtime CPU, FP32 / INT8 | 115 / 55 ms | — |
| M4 Pro, Chrome 156 | onnxruntime-web 1.31.1 WebGPU, FP16 | 48 ms | 151 ms |
| M4 Pro, Safari 27.2 | onnxruntime-web WebGPU, FP16 | 78 ms | 289 ms |
| RTX 5060, Chrome 155 / Edge 156 | onnxruntime-web WebGPU, FP16 | 179 / 177 ms | 208 / 193 ms |
| Browser wasm, 1 thread | onnxruntime-web, FP16 | 142–343 ms | — |

- onnxruntime-web's WebGPU backend computes the wrong motion in FP32 in
  all four browsers. The root ends 0.19–0.24 m from the start instead
  of 1.98 m, while the same model on wasm is correct. FP16 is close to
  the reference (1.96–1.97 m).
- On the RTX 5060, the browser time stays almost flat from N=1 to N=32
  (175 to 200 ms). The time goes to fixed overhead: ORT left some nodes
  on the CPU, so every run copies data between the GPU and the CPU.
  That makes it about 40× slower than native.
- The first WebGPU run takes 1–14 s while shaders compile.
- Results are not reproducible across settings. On one CPU, the same
  thread count repeats bit for bit, but 1 thread and 8 threads differ by
  up to 3.5e-4. FP16 and FP32 differ. INT8 picks another motion: the
  joints move up to 1 m from the FP32 result, though the motion still
  reaches its target.

## Offline clips retargeted in the engine (studio-win, frames read)

Clips generated: idle-to-walk start, walk, run, stop, 90° and 180°
turns, a 0.5 m vault and sit-and-stand. They were exported as SOMA
glTF, cooked and retargeted onto the strider, then captured with
`og run --commands` on the GPU. The walk and run look more natural
than the template's clips. Root travel scaled by the leg ratio (1.19)
was checked numerically. Problems found:

- Feet are not planted. In contact, the ankle stays 18–63 mm above the
  ground and slides 2–4 mm per frame. `og.FootLock` planted 27 and 56
  of 180 ticks on the walk, against 110 for the native clip. The
  model's contact output is not used yet.
- Retargeting: a leaf head inherits its neck's rest correction. SOMA's
  neck leans 17° forward at rest, so the retargeted head pitches 17°
  forward.
- The loops do not close: the walk drifts 0.9° of yaw and 1.6 cm
  sideways per cycle.
