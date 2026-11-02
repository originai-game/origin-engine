# Motion course

og-motion's demo course (`crates/og-motion/README.md`) played by two game
rigs: one character walks, turns, runs, jumps, crouches, stands up and waves along
fixed waypoints. The motion was generated for NVIDIA's SOMA skeleton by
the distilled model, chained through nine goals with the intent API, its
feet planted from the model's contacts; `og.anim.retarget` puts it on the
strider (Mixamo names, a 1.14 m leg) and the stout (Unreal names, 0.62 m).

`assets/anims/course.glb` is generated and not committed: it is the
output of a trained bundle (og-motion-train's export) on a course whose
target poses come from its motion library.

```sh
# og-motion-train: the bundle and the course
python scripts/export.py out/run3 out/dec1/decoder.pt out/run3/export
python scripts/demo_course.py out/run3/export out/demo
# origin-engine: the course on the GPU, written as a SOMA glTF library
cargo run --release -p og-motion --example course -- \
  <og-motion-train>/out/run3/export/motion.safetensors <og-motion-train>/out/demo/course.safetensors \
  /tmp/course.json examples/motion-course/assets/anims/course.glb
og run examples/motion-course --size 1280x720 --commands "2.5:screenshot:walk.png;5.5:screenshot:run.png;8.1:screenshot:jump.png;10.5:screenshot:crouch.png;16.4:screenshot:wave.png;16.45:quit"
```

The model is "Licensed by NVIDIA Corporation under the NVIDIA Open Model
Agreement."; see LICENSES.md for the generated file.
