# Motion course

og.motion on two game rigs: the engine's motion model animates the strider
(Mixamo names, a 1.14 m leg) and the stout (Unreal names, 0.62 m) live, on
the GPU, from goals `src/main.ts` queues: walk out, turn, run, jump,
crouch, walk back, face the camera, wave and cheer. `og.MotionAgent.drive`
moves their Transforms with fixed math in the tick; the generated pose is
drawn over it, retargeted to each rig with its feet planted, and never fed
back into the simulation (the world hash is the same with or without the
model).

`og cook` writes `cooked/motion/model.ogb` because the code uses og.motion.
In an engine checkout the weights are not in git: point the cook at them.

```sh
# og-motion-train's weights/engine/model.safetensors (scripts/export_engine.py)
OG_MOTION_MODEL=<og-motion-train>/weights/engine/model.safetensors \
  og run examples/motion-course --size 960x540 --commands "2.5:screenshot:walk.png;5.6:screenshot:jump.png;12:screenshot:wave.png;12:dump_motion:motion.json;12.05:quit"
```

`dump_motion` shows the goals the simulation keeps and the model's side:
its backend, frames generated ahead, batches and GPU time, and how far
planted feet slid.

The model is "Licensed by NVIDIA Corporation under the NVIDIA Open Model
Agreement."; see LICENSES.md.
