//! Scene geometry, materials, textures and resource handles.

use super::*;

/// One mesh vertex. Positions and normals are in the mesh's local space.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    pub uv: [f32; 2],
    /// Vertex colour, multiplied into the base colour (white when unused).
    pub color: [f32; 4],
    /// The tangent frame normal maps are read in (glTF's): `xyz` the
    /// tangent, along +u; `w` the sign of the bitangent cross(normal,
    /// tangent) × w, which points up the image (towards -v, as uvs run down
    /// it), so a map's red tilts towards +u and its green up the image. `w`
    /// 0 means none: the renderer generates it from the positions, normals
    /// and uvs ([`MeshData::generate_tangents`]).
    pub tangent: [f32; 4],
}

impl Default for Vertex {
    fn default() -> Self {
        Vertex {
            position: [0.0; 3],
            normal: [0.0, 1.0, 0.0],
            uv: [0.0; 2],
            color: [1.0; 4],
            tangent: [0.0; 4],
        }
    }
}

/// How a mesh's vertices are held on the GPU
/// ([`RenderScene::set_vertex_encoding`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum VertexEncoding {
    /// [`Vertex`] as given: 64 bytes a vertex in f32, for every pipeline.
    #[default]
    Full,
    /// 16 bytes a vertex where the mesh is drawn with a plain material (the
    /// lean pipelines; README "Instance and vertex encodings"): the
    /// position quantised to 16 bits per axis over the mesh's bounds, the
    /// normal octahedral in two 16-bit snorms, the colour in 8 bits a
    /// channel. Quantised, so its pixels are close to [`VertexEncoding::Full`]'s
    /// but not the same. The f32 vertices are uploaded too only if a
    /// material that is not plain, a scatter or a skinned instance draws
    /// the mesh. Ignored for skinned and morphed meshes.
    Compact,
}

/// Triangle-list geometry: vertices and `u32` indices (three per triangle).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshData {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl MeshData {
    /// Whether some vertex has no tangent (`tangent[3] == 0`).
    pub fn lacks_tangents(&self) -> bool {
        self.vertices.iter().any(|v| v.tangent[3] == 0.0)
    }

    /// Gives every vertex without a tangent one: the uv gradient of the
    /// triangles around it (Lengyel's method), made orthogonal to its normal,
    /// with the sign that makes the bitangent point towards -v ([`Vertex::tangent`]);
    /// a vertex whose triangles have no usable uvs gets any tangent
    /// orthogonal to its normal. Vertices that have a tangent (the cook's
    /// MikkTSpace ones) keep it.
    pub fn generate_tangents(&mut self) {
        if !self.lacks_tangents() {
            return;
        }
        let n = self.vertices.len();
        let mut along_u = vec![Vec3::ZERO; n];
        let mut along_v = vec![Vec3::ZERO; n];
        for tri in self.indices.as_chunks::<3>().0 {
            let [a, b, c] = tri.map(|i| i as usize);
            if a >= n || b >= n || c >= n {
                continue;
            }
            let (va, vb, vc) = (&self.vertices[a], &self.vertices[b], &self.vertices[c]);
            let p0 = Vec3::from(va.position);
            let e1 = Vec3::from(vb.position) - p0;
            let e2 = Vec3::from(vc.position) - p0;
            let (du1, dv1) = (vb.uv[0] - va.uv[0], vb.uv[1] - va.uv[1]);
            let (du2, dv2) = (vc.uv[0] - va.uv[0], vc.uv[1] - va.uv[1]);
            let r = du1 * dv2 - du2 * dv1;
            if r.abs() < 1e-12 {
                continue;
            }
            let su = (e1 * dv2 - e2 * dv1) / r;
            let sv = (e2 * du1 - e1 * du2) / r;
            for i in [a, b, c] {
                along_u[i] += su;
                along_v[i] += sv;
            }
        }
        for (i, v) in self.vertices.iter_mut().enumerate() {
            if v.tangent[3] != 0.0 {
                continue;
            }
            let normal = Vec3::from(v.normal).normalize_or(Vec3::Y);
            let mut t = along_u[i] - normal * normal.dot(along_u[i]);
            if t.length_squared() < 1e-20 {
                // No uv gradient: any direction across the normal.
                t = normal.any_orthonormal_vector();
            }
            let t = t.normalize();
            // The bitangent up the image: against the uv's v.
            let w = if normal.cross(t).dot(along_v[i]) > 0.0 {
                -1.0
            } else {
                1.0
            };
            v.tangent = t.extend(w).to_array();
        }
    }

    /// Axis-aligned bounds of the vertex positions (`None` for an empty mesh).
    pub fn bounds(&self) -> Option<(Vec3, Vec3)> {
        let mut it = self.vertices.iter().map(|v| Vec3::from(v.position));
        let first = it.next()?;
        Some(it.fold((first, first), |(lo, hi), p| (lo.min(p), hi.max(p))))
    }
}

/// How the texel values of a colour texture are encoded.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum ColorSpace {
    /// sRGB-encoded colour (base colour, emissive): decoded to linear when sampled.
    #[default]
    Srgb,
    /// Linear data (normal maps, metallic-roughness).
    Linear,
}

/// Whether the renderer builds a mip chain for a texture when it uploads it.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum Mips {
    /// Generate every level down to 1x1 on the GPU (box filter, linear space).
    #[default]
    Generate,
    /// Upload level 0 only.
    None,
}

/// An RGBA8 image, rows top to bottom, `width * height * 4` bytes.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct TextureData {
    pub width: u32,
    pub height: u32,
    pub rgba: Vec<u8>,
    pub color_space: ColorSpace,
    pub mips: Mips,
}

/// A texture as a host hands it over: RGBA8 (mips generated on the GPU) or
/// pre-encoded in one of [`crate::RenderInfo::texture_formats`]
/// ([`RenderScene::add_texture_source`]).
#[derive(Clone, Debug, PartialEq)]
pub enum TextureSource {
    Rgba(TextureData),
    Encoded(TextureUpload),
    /// A render target's colour or depth ([`crate::RenderScene::add_render_target`]):
    /// drawn on the GPU by a [`crate::View`], never uploaded.
    Target(crate::TargetTexture),
}

impl TextureSource {
    /// Width and height of level 0.
    pub fn size(&self) -> (u32, u32) {
        match self {
            TextureSource::Rgba(t) => (t.width, t.height),
            TextureSource::Encoded(t) => (t.desc.width, t.desc.height),
            TextureSource::Target(t) => (t.width, t.height),
        }
    }
}

/// How a material treats base-colour alpha.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum AlphaMode {
    /// Alpha is ignored.
    #[default]
    Opaque,
    /// Fragments whose alpha is below `cutoff` are discarded (glTF `MASK`).
    Mask { cutoff: f32 },
    /// Alpha blended over the opaque scene, sorted back to front (glTF
    /// `BLEND`; [`crate::StandardMaterial::transparent`]).
    Blend,
}

/// A metallic-roughness PBR material (the glTF 2.0 model). Every factor
/// multiplies its texture; a missing texture behaves as white (or as a flat
/// normal). The default is an opaque, white, non-metallic, half-rough surface.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    /// Linear RGBA base colour factor.
    pub base_color: [f32; 4],
    pub base_color_texture: Option<TextureHandle>,
    pub metallic: f32,
    pub roughness: f32,
    /// Metalness in blue, roughness in green (glTF layout).
    pub metallic_roughness_texture: Option<TextureHandle>,
    /// Tangent-space normal map; the tangent frame comes from screen derivatives.
    pub normal_texture: Option<TextureHandle>,
    pub normal_scale: f32,
    /// Linear emitted radiance (before the emissive texture).
    pub emissive: [f32; 3],
    pub emissive_texture: Option<TextureHandle>,
    pub alpha_mode: AlphaMode,
}

impl Default for Material {
    fn default() -> Self {
        Material {
            base_color: [1.0; 4],
            base_color_texture: None,
            metallic: 0.0,
            roughness: 0.5,
            metallic_roughness_texture: None,
            normal_texture: None,
            normal_scale: 1.0,
            emissive: [0.0; 3],
            emissive_texture: None,
            alpha_mode: AlphaMode::Opaque,
        }
    }
}
