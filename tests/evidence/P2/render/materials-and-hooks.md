# Materials (owed row 31) and hooks beyond the lit pass (owed row 23)

## What changed

- Normals go through the inverse transpose of the model's upper 3×3 (the
  cofactor matrix, flipped for a mirroring transform), static and skinned.
- `Vertex::tangent` (`xyz` along +u, `w` the bitangent's sign) in glTF's
  frame, bitangent up the image. Sources: the file (the glTF bridge reads
  `TANGENT`), the cook's MikkTSpace (now fed v flipped, so it agrees with
  exporters; the cook settings string names the frame, so cooks rerun), or
  `MeshData::generate_tangents` at upload (Lengyel). The frame from screen
  derivatives is removed: WGSL's framebuffer y runs down, which had turned
  both tangent axes round, so every normal map read inverted (red and green
  both). `tests/materials.rs` `a_normal_maps_green_points_up_the_image` and
  `mirrored_uvs_keep_the_map_the_right_way_round` pin the convention.
- Clearcoat (`clearcoat`, `clearcoatRoughness`): Filament's clear coat lobe
  for the lights and the image-based light. No maps (no texture slot left
  in WebGL2's skinned lit pipeline).
- Material shaders on skinned meshes; the shadow passes and the depth
  prepass run the material's hooks (`vertex_offset`, and `surface_color`'s
  alpha against `alphaTest`), with the scene clock in the shadow uniform.
- `og.render.setMaterialShader` gives an existing material (a loaded
  model's) a shader, or takes it away.

## Frames

- `materials.png`: og-tests' `materials` reference (metal): metalness row
  (front), roughness row, then emissive, bumps on a sphere and a box,
  clearcoat glossy and rough over rough red; then the alpha-tested grille,
  glass, a double-sided shell, unlit, the squashed ellipsoid.
- `skinned-hooks.png`: og-tests' `skinned-hooks`: the skinning scene's
  cylinders rippled, banded and lifted 0.4 m by a material shader, their
  shadows and ambient occlusion following.
- Re-recorded og-tests metal references, all of which still passed within
  og-tests' tolerance before re-recording:

| Scene | Against the old reference | Why |
|---|---|---|
| `shader-hooks` | RMSE 0.0444, 4 % outliers | the shadows now follow the grass's sway and the dissolve's cuts (`shader-hooks-before-after.png`, the bottom half of the frame) |
| `gltf-reference` | RMSE 0.0039, 66 outliers | the trickster's normal maps in the vertex tangent frame, the right way round (`gltf-reference-before-after.png`) |
| `showcase`, `overlay`, `post-grade`, `dof-low`, `dof-high`, `stylised`, `skinning` | RMSE 0.0008–0.0042 | the same normal maps; the walker |
| `spheres-20k`, `particles`, `particles-cpu` | RMSE 0.003–0.006, up to 855 isolated pixels | no visible change: the lit shader recompiled (Metal's fast math reorders its arithmetic), so a few silhouette and highlight pixels of the tiny spheres moved |

- fx-showcase: `shaders` re-recorded (`fx-showcase-shaders-before-after.png`:
  before, after: the grass's and the cut cube's shadows follow the shader);
  every other station passes at RMSE ≤ 0.00075 (the trickster's and the
  walker's models cook without normal maps).

## Tests

`crates/og-tests/tests/materials.rs` (8) and `hooks.rs` (3), each broken
once (`break-and-restore.md`): a non-uniformly scaled sphere shades as the
ellipsoid with its true normals (0 pixels apart); generated tangents shade
as the sphere's analytic ones (0 pixels over 4 apart, RMSE 0.00015); a
coarse sphere's tilted map is as smooth as a fine one's; mirrored uvs flip
the bitangent; green tilts up the image; a glossy coat adds a highlight
(115 to 232) sharper than a rough one (178); `generate_tangents` on a quad;
a static and a skinned mesh moved by `vertex_offset` draw, shadow and
occlude exactly as the mesh moved by its transform (0 pixels apart, with
shadows, GTAO and outlines on); the skinned cylinders take the shader's
bands.
