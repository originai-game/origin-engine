# GPU debris (og-render `debris.rs`, `shaders/debris.wgsl`)

The debris layer simulates the fine pieces of broken props as rigid
bodies on the GPU, visual only, with no readback.

## The solver

XPBD's rigid bodies, implemented from the paper (no code taken):

- Matthias Müller, Miles Macklin, Nuttapong Chentanez, Stefan Jeschke,
  Tae-Yong Kim. *Detailed Rigid Body Simulation with Extended Position
  Based Dynamics.* Computer Graphics Forum 39(8), SCA 2020, 101–112.
  <https://doi.org/10.1111/cgf.14105>,
  <https://matthias-research.github.io/pages/publications/PBDBodies.pdf>.

We follow algorithms 2 and 3 of the paper:
- Substeps, each integrating the pose.
- Contacts projected with the generalised inverse mass (eqs. 2–9), with
  static friction from the contact point's slide.
- Velocities from the moved poses (the quaternion difference for ω).
- The velocity pass for dynamic friction and restitution (section 3.6).

Each piece is one body against a static world, so it is solved per unit
mass, with a box inertia from its hull's bounds. It collides at up to 16
of its hull's vertices, picked by farthest-point sampling in og-runtime.

The visual-body solver (`third_party/visual-bodies.md`, cloth and soft
bodies) chose XPBD too. Its kernel solves one deformable body per
workgroup, so debris has its own kernel, one thread per piece.

## Scene colliders

`og-render/src/scene_colliders.rs` and `shaders/scene_colliders.wgsl` are
the packing and point-versus-collider tests of the GPU visual-body solver
(branch `gpu/visual-cloth-softbody`, written in this repository), shared
with it. They contain no third-party code.

## Licences

Engine code only, under the repository's MIT licence. The paper is cited
for its algorithm; no source was copied from it or from its authors'
Ten Minute Physics demos.
