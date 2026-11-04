# Phong tessellation

`crates/og-render/src/surface.rs` and `shaders/surface.wgsl` independently
implement the tangent-plane projection and barycentric interpolation in:

Tamy Boubekeur and Marc Alexa, **Phong Tessellation**, ACM Transactions on
Graphics 27(5), SIGGRAPH Asia 2008.
https://perso.telecom-paristech.fr/boubek/papers/PhongTessellation/PhongTessellation.pdf

By default each coarse triangle becomes four triangles; presentation settings
select one to eight segments per edge. The projection blend is 0.75,
as suggested by the paper, with the same edge samples on adjoining faces.
Area-weighted vertex normals are computed by gathering incident faces (CSR
adjacency), without floating-point atomics. The resulting ordinary vertex
buffer is shared by lit, depth, reflection and shadow passes. Shared edges
evaluate in canonical vertex order to avoid numerical cracks. The coarse CPU
fallback retains the original cloth finite-difference or soft-body
area-weighted normals and does not refine the geometry.

No third-party source code was copied. These independent implementations
use the repository's MIT licence. Dupuy's *Adaptive GPU Tessellation with
Compute Shaders* was consulted as an architectural reference, but its isubd
code was not incorporated: adaptive binary subdivision is unnecessary for
this fixed-resolution visual refinement. This note does not relicense the
papers or claim a licence for isubd.
