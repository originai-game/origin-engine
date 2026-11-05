# Visual bodies (GPU XPBD)

`crates/og-render/src/visual_sim.rs` and `crates/og-render/shaders/visual_body.wgsl`
independently implement, from the published papers, the techniques of:

- Miles Macklin, Matthias Müller and Nuttapong Chentanez, **XPBD:
  Position-Based Simulation of Compliant Constrained Dynamics**, Motion in
  Games 2016 (compliant distance and volume constraints).
- Miles Macklin, Kier Storey, Michelle Lu, Pierre Terdiman, Nuttapong
  Chentanez, Stefan Jeschke and Matthias Müller, **Small Steps in Physics
  Simulation**, SCA 2019 (one constraint pass per substep, many substeps).
- Tae-Yong Kim, Nuttapong Chentanez and Matthias Müller-Fischer, **Long
  Range Attachments: A Method to Simulate Inextensible Clothing in Computer
  Games**, SCA 2012 (tethers to the nearest pin).
- Matthias Müller, Bruno Heidelberger, Marcus Hennix and John Ratcliff,
  **Position Based Dynamics**, VRIPHYS 2006 / JVCIR 2007 (the volume
  constraint of closed surfaces).
- Matthias Teschner, Bruno Heidelberger, Matthias Müller, Danat Pomeranets
  and Markus Gross, **Optimized Spatial Hashing for Collision Detection of
  Deformable Objects**, VMV 2003 (the self-collision grid).

Wind is aerodynamic, per triangle against its own velocity: drag along
the relative flow and lift across it, the flat-plate split of the normal
pressure (drag ∝ |cos θ|·|v|², lift ∝ cos θ·sin θ·|v|²) that NVIDIA's
NvCloth (BSD-3-Clause) documents for its cloth wind
(<https://nvidiagameworks.github.io/NvCloth/1.1/UserGuide/Index.html>,
"Drag lift and wind": per-triangle drag and lift coefficients against the
wind velocity, which it advises varying for gusts), with the flag model of
Keckeisen, Kimmerle, Thomaszewski and Wacker (*Modelling Effects of Wind
Fields in Cloth Animations*, WSCG 2004). The model was written from that
description; no NvCloth code was copied. The air gusts in bands that
travel with it, so the pressure differs across the cloth and folds run
down it.

Distance constraints are graph-coloured (greedy edge colouring) so each
colour solves in parallel without atomics; one workgroup steps a body
through every tick of the frame.

AVBD (Giles, Diaz and Yuksel, *Augmented Vertex Block Descent*, SIGGRAPH
2025) was considered: its per-vertex block solve pays off for stiff contact
stacks, which these one-way bodies do not have, so XPBD in small steps was
chosen. No third-party source code was copied. These implementations use the
repository's MIT licence; this note does not relicense the papers.
