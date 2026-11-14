//! Materials: [`StandardMaterial`], the reflected three.js
//! `MeshStandardMaterial` (with `unlit`, `MeshBasicMaterial`; with the
//! `MeshPhysicalMaterial` fields, the physical lobes of `physical.rs`), and
//! the GPU form both it and the glTF-shaped [`Material`] become.

use crate::scene::{AlphaMode, Material, TextureHandle};
use glam::{Vec2, Vec3};
use og_core::Reflect;

/// Which faces of a mesh are drawn: three.js `FrontSide`, `BackSide`,
/// `DoubleSide`, with the same numbers ([`StandardMaterial::side`]).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Side {
    /// Counter-clockwise faces (the default).
    #[default]
    Front = 0,
    Back = 1,
    /// Both; the back faces are shaded with the normal flipped.
    Double = 2,
}

impl Side {
    /// The side a [`StandardMaterial::side`] value names (unknown values are `Front`).
    pub fn from_u32(v: u32) -> Side {
        match v {
            1 => Side::Back,
            2 => Side::Double,
            _ => Side::Front,
        }
    }
}

/// How a texture slot's uvs are transformed before it is sampled: glTF
/// `KHR_texture_transform` (three.js Texture `offset`, `repeat`,
/// `rotation` with `center` at the origin), in glTF's order: scaled, then
/// rotated counter-clockwise by `rotation` radians, then offset. A
/// [`StandardMaterial`] has one per slot ([`StandardMaterial::uv_transforms`]).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct UvTransform {
    pub offset: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
}

impl Default for UvTransform {
    fn default() -> Self {
        UvTransform::IDENTITY
    }
}

impl UvTransform {
    pub const IDENTITY: UvTransform = UvTransform {
        offset: Vec2::ZERO,
        scale: Vec2::ONE,
        rotation: 0.0,
    };

    /// The slots of [`StandardMaterial::uv_transforms`].
    pub const MAP: usize = 0;
    pub const METALNESS_ROUGHNESS: usize = 1;
    pub const NORMAL: usize = 2;
    pub const EMISSIVE: usize = 3;
    pub const AO: usize = 4;

    pub fn is_identity(&self) -> bool {
        *self == UvTransform::IDENTITY
    }

    /// The 2 x 3 matrix, rows `[a, b, tx]` and `[c, d, ty]`:
    /// `u' = a u + b v + tx`, `v' = c u + d v + ty` (glTF's
    /// `translation * rotation * scale`).
    pub fn matrix(&self) -> [[f32; 3]; 2] {
        let (s, c) = self.rotation.sin_cos();
        [
            [c * self.scale.x, s * self.scale.y, self.offset.x],
            [-s * self.scale.x, c * self.scale.y, self.offset.y],
        ]
    }

    /// `uv` transformed.
    pub fn apply(&self, uv: [f32; 2]) -> [f32; 2] {
        let [r0, r1] = self.matrix();
        [
            r0[0] * uv[0] + r0[1] * uv[1] + r0[2],
            r1[0] * uv[0] + r1[1] * uv[1] + r1[2],
        ]
    }
}

/// A physically based metallic-roughness material with three.js
/// `MeshStandardMaterial` parameter names, reflected: its schema
/// (`og.StandardMaterial`) is what the inspector, Tune and the probe edit,
/// and its fields match `render.StandardMaterialDesc` in the IDL.
///
/// The `MeshPhysicalMaterial` fields (clearcoat maps, sheen, transmission,
/// iridescence, anisotropy, `ior`) draw through the physical lit pipelines
/// ([`StandardMaterial::is_physical`]); a material without them draws
/// exactly as a `MeshStandardMaterial`.
///
/// Texture fields are slots: `0` is none, otherwise
/// [`TextureHandle::slot`] (the texture's index plus one). Boolean fields
/// are `0` or `1`. Colours are linear. The default is three.js's: white,
/// metalness 0, roughness 1, opaque, front side.
#[allow(non_snake_case)]
#[derive(Clone, Copy, Debug, PartialEq, Reflect)]
#[reflect(name = "og.StandardMaterial", version = 11)]
pub struct StandardMaterial {
    /// The base colour (linear RGB), multiplied with `map` and vertex colours.
    #[reflect(default = [1.0, 1.0, 1.0], min = 0.0, tunable)]
    pub color: Vec3,
    /// The base colour texture (sRGB); its alpha multiplies `opacity`.
    pub map: u32,
    /// How metallic the surface is.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub metalness: f32,
    /// Metalness from the texture's blue channel (glTF packing).
    pub metalnessMap: u32,
    /// How rough the surface is.
    #[reflect(min = 0.0, max = 1.0, default = 1.0, tunable)]
    pub roughness: f32,
    /// Roughness from the texture's green channel (glTF packing; often the
    /// same texture as `metalnessMap`).
    pub roughnessMap: u32,
    /// A tangent-space normal map (RGB, or a two-channel BC5/EAC texture
    /// whose z is reconstructed).
    pub normalMap: u32,
    /// How strongly the normal map bends the normal, per tangent axis.
    #[reflect(default = [1.0, 1.0], tunable)]
    pub normalScale: Vec2,
    /// Ambient occlusion from the texture's red channel; it darkens the
    /// image-based (ambient) light only.
    pub aoMap: u32,
    #[reflect(min = 0.0, max = 1.0, default = 1.0, tunable)]
    pub aoMapIntensity: f32,
    /// Emitted light (linear RGB), multiplied with `emissiveMap`.
    #[reflect(min = 0.0, default = [0.0, 0.0, 0.0], tunable)]
    pub emissive: Vec3,
    /// The emitted colour texture (sRGB).
    pub emissiveMap: u32,
    /// A multiplier on the emitted light.
    #[reflect(min = 0.0, default = 1.0, tunable)]
    pub emissiveIntensity: f32,
    /// The opacity; below 1 only shows with `transparent`.
    #[reflect(min = 0.0, max = 1.0, default = 1.0, tunable)]
    pub opacity: f32,
    /// 1: blended over the opaque scene, sorted back to front, no depth write.
    #[reflect(min = 0.0, max = 1.0, default = 0.0)]
    pub transparent: u32,
    /// Pixels whose alpha is below this are discarded (0: none).
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub alphaTest: f32,
    /// 0 front, 1 back, 2 double ([`Side`]).
    #[reflect(min = 0.0, max = 2.0, default = 0.0)]
    pub side: u32,
    /// 1: shaded with the face normal (facets) instead of the vertex normals.
    #[reflect(min = 0.0, max = 1.0, default = 0.0)]
    pub flatShading: u32,
    /// 1: with MSAA, an alpha-tested material turns its alpha into sample coverage (alpha to coverage, three.js
    /// `alphaToCoverage`) instead of cutting at alphaTest: foliage cards, minified until their alpha averages
    /// below 1, cover that share of the pixel and the scene shows between them; a hard cut drew them as solid
    /// sheets. Without MSAA it is the cut as before.
    #[reflect(min = 0.0, max = 1.0, default = 0.0)]
    pub alphaToCoverage: u32,
    /// A multiplier on the image-based (environment) light.
    #[reflect(min = 0.0, default = 1.0, tunable)]
    pub envMapIntensity: f32,
    /// 1: no lighting at all, three.js `MeshBasicMaterial`: the colour (times
    /// `map` and vertex colours) plus the emissive term.
    #[reflect(min = 0.0, max = 1.0, default = 0.0)]
    pub unlit: u32,
    /// 1: cel shading (three.js `MeshToonMaterial`): light is quantized by
    /// `gradientMap` (or three built-in bands without one), highlights are hard.
    #[reflect(min = 0.0, max = 1.0, default = 0.0)]
    pub toon: u32,
    /// The toon ramp: a texture whose first row maps N·L (left dark, right
    /// lit) to the light received, read without filtering (hard bands).
    pub gradientMap: u32,
    /// Rim (fresnel) light added at grazing angles, unshadowed (linear RGB).
    #[reflect(min = 0.0, default = [1.0, 1.0, 1.0], tunable)]
    pub rimColor: Vec3,
    /// 0: no rim light.
    #[reflect(min = 0.0, default = 0.0, tunable)]
    pub rimIntensity: f32,
    /// How tight the rim is (higher: thinner).
    #[reflect(min = 0.1, default = 3.0, tunable)]
    pub rimPower: f32,
    /// The colour a hit flash mixes towards (linear RGB, may exceed 1).
    #[reflect(min = 0.0, default = [1.0, 1.0, 1.0], tunable)]
    pub flashColor: Vec3,
    /// How far the final colour is mixed towards `flashColor` (0 none, 1 flat).
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub flash: f32,
    /// The dissolved fraction: 0 whole, 1 gone. Pixels whose noise is below
    /// it are discarded (also from shadows and the prepass); the band just
    /// above glows with `dissolveEdgeColor`.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub dissolve: f32,
    /// The dissolve noise from the texture's red channel (by UV); without
    /// one, value noise over the object-space position.
    pub dissolveMap: u32,
    /// The glowing edge (linear RGB, usually above 1 so it blooms).
    #[reflect(min = 0.0, default = [4.0, 1.5, 0.375], tunable)]
    pub dissolveEdgeColor: Vec3,
    /// Width of the glowing band in noise units.
    #[reflect(min = 0.0, max = 1.0, default = 0.0625, tunable)]
    pub dissolveEdgeWidth: f32,
    /// Frequency of the built-in noise (cells per object-space unit).
    #[reflect(min = 0.0, default = 4.0, tunable)]
    pub dissolveScale: f32,
    /// This object's outline colour (linear RGB) in the outline pass.
    #[reflect(min = 0.0, max = 1.0, default = [0.0, 0.0, 0.0], tunable)]
    pub outlineColor: Vec3,
    /// This object's outline width in pixels (0: the scene's
    /// `PostEffects::outline`, if any). Turns the outline pass on.
    #[reflect(min = 0.0, max = 31.0, default = 0.0, tunable)]
    pub outlineWidth: f32,
    /// How much of its detail the painterly filter keeps on this material
    /// (`Stylise::paint`, `Paint::detail`): 0 painted at the full radius,
    /// 1 kept as drawn (a face, a hand, a sign's lettering).
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub paintDetail: f32,
    /// Leaf lighting: x translucency, y canopy dome-normal blend. Zero
    /// preserves ordinary surface shading; neither needs another texture.
    #[reflect(min = 0.0, max = 1.0, default = [0.0, 0.0], tunable)]
    pub foliage: Vec2,
    /// A clear lacquer over the surface (three.js `MeshPhysicalMaterial`
    /// `clearcoat`): a second specular layer, 4 % reflective, on the
    /// surface's normal before the normal map. 0: none.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub clearcoat: f32,
    /// How rough the coat is.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub clearcoatRoughness: f32,
    /// The coat's strength from the texture's red channel (times `clearcoat`).
    pub clearcoatMap: u32,
    /// The coat's roughness from the texture's green channel (times
    /// `clearcoatRoughness`).
    pub clearcoatRoughnessMap: u32,
    /// A tangent-space normal map of the coat alone (the base keeps `normalMap`).
    pub clearcoatNormalMap: u32,
    /// How strongly `clearcoatNormalMap` bends the coat's normal, per tangent axis.
    #[reflect(default = [1.0, 1.0], tunable)]
    pub clearcoatNormalScale: Vec2,
    /// A velvet-like retroreflective layer (three.js `sheen`, glTF
    /// `KHR_materials_sheen`): its strength times `sheenColor`. 0: none.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub sheen: f32,
    /// The sheen's colour (linear RGB), multiplied with `sheenColorMap`.
    #[reflect(min = 0.0, max = 1.0, default = [0.0, 0.0, 0.0], tunable)]
    pub sheenColor: Vec3,
    /// How rough the sheen is: low is a thin rim, high spreads it over the surface.
    #[reflect(min = 0.0, max = 1.0, default = 1.0, tunable)]
    pub sheenRoughness: f32,
    /// The sheen colour texture (sRGB, RGB channels).
    pub sheenColorMap: u32,
    /// The sheen's roughness from the texture's alpha channel.
    pub sheenRoughnessMap: u32,
    /// How much light passes through the surface (three.js `transmission`,
    /// glTF `KHR_materials_transmission`): glass, water, thin plastic. The
    /// opaque scene behind it is refracted, blurred by `roughness`. 0: none.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub transmission: f32,
    /// Transmission from the texture's red channel.
    pub transmissionMap: u32,
    /// The volume's thickness under the surface in object-space units
    /// (glTF `KHR_materials_volume`): how far refraction bends the view
    /// and how much `attenuationColor` absorbs. 0: a thin wall.
    #[reflect(min = 0.0, default = 0.0, tunable)]
    pub thickness: f32,
    /// Thickness from the texture's green channel.
    pub thicknessMap: u32,
    /// The distance light travels in the volume before it is tinted to
    /// `attenuationColor` (world units). 0: no absorption (three.js's Infinity).
    #[reflect(min = 0.0, default = 0.0, tunable)]
    pub attenuationDistance: f32,
    /// The colour white light becomes after `attenuationDistance` in the volume (linear RGB).
    #[reflect(min = 0.0, max = 1.0, default = [1.0, 1.0, 1.0], tunable)]
    pub attenuationColor: Vec3,
    /// The index of refraction (glTF `KHR_materials_ior`): the dielectric
    /// reflectance ((ior - 1) / (ior + 1))², 4 % at the default 1.5, and
    /// how far transmission refracts.
    #[reflect(min = 1.0, max = 2.333, default = 1.5, tunable)]
    pub ior: f32,
    /// A thin-film interference layer (three.js `iridescence`, glTF
    /// `KHR_materials_iridescence`): soap bubbles, oil, beetle shells. 0: none.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub iridescence: f32,
    /// The film's index of refraction.
    #[reflect(min = 1.0, max = 2.333, default = 1.3, tunable)]
    pub iridescenceIOR: f32,
    /// The film's thickness in nanometres: `x` where `iridescenceThicknessMap`
    /// is black, `y` where it is white (and everywhere without the map).
    #[reflect(min = 0.0, default = [100.0, 400.0], tunable)]
    pub iridescenceThicknessRange: Vec2,
    /// Iridescence from the texture's red channel.
    pub iridescenceMap: u32,
    /// The film's thickness between the range's ends from the texture's green channel.
    pub iridescenceThicknessMap: u32,
    /// Stretches the specular highlight along a direction of the surface
    /// (three.js `anisotropy`, glTF `KHR_materials_anisotropy`): brushed
    /// metal, hair, satin. 0: none, 1: fully stretched.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub anisotropy: f32,
    /// The stretch's direction: radians from the tangent (+u) towards the bitangent.
    #[reflect(default = 0.0, tunable)]
    pub anisotropyRotation: f32,
    /// The direction (red and green, -1..1 in the tangent frame, rotated by
    /// `anisotropyRotation`) and strength (blue, times `anisotropy`).
    pub anisotropyMap: u32,
    /// How much of the diffuse light spreads under the surface before it
    /// leaves: skin, wax, marble, jade, milk. Its falloff softens past the
    /// terminator by the surface's curvature, a channel that travels
    /// farther bleeding into the shadow, and with a `thickness` light
    /// from behind comes through. 0: none.
    #[reflect(min = 0.0, max = 1.0, default = 0.0, tunable)]
    pub subsurface: f32,
    /// How far light of each channel travels under the surface, in metres
    /// (the default is skin's: red farthest).
    #[reflect(min = 0.0, default = [0.0037, 0.0014, 0.0007], tunable)]
    pub subsurfaceRadius: Vec3,
    /// The base colour from layer `mapLayer` of a texture array (sRGB)
    /// instead of `map`: a texture-array slot,
    /// [`crate::TextureArrayHandle::slot`] (0: none). Variants of one
    /// material (tiles, skins) are one material whose instances pick their
    /// layer ([`crate::Instance::material_layer`]): one batch, one draw.
    /// A material with array maps samples its base colour and normal from
    /// arrays only (`map` and `normalMap` are not read); the physical lobes
    /// and material shaders do not take them.
    pub mapArray: u32,
    /// The layer of `mapArray` sampled (past the last: the last).
    #[reflect(min = 0.0, default = 0.0)]
    pub mapLayer: u32,
    /// A tangent-space normal map from layer `normalMapLayer` of a texture
    /// array (linear) instead of `normalMap`; a texture-array slot.
    pub normalMapArray: u32,
    /// The layer of `normalMapArray` sampled.
    #[reflect(min = 0.0, default = 0.0)]
    pub normalMapLayer: u32,
    /// 1 (the default): the scene's [`crate::MaterialStyle`] fills the
    /// stylised fields this material leaves at their defaults ([`styled`]).
    /// 0: drawn exactly as declared, untouched by the style (no toon, rim,
    /// outline, flat facets, shadow tint or outline distance from it).
    #[reflect(min = 0.0, max = 1.0, default = 1.0)]
    pub styled: u32,
    /// How much the weather on surfaces ([`crate::SurfaceWeather`]: wetness,
    /// streaks, puddles, snow, dust) reaches this material: 1 all of it,
    /// 0 none.
    #[reflect(min = 0.0, max = 1.0, default = 1.0, tunable)]
    pub weathering: f32,
    /// 1: drawn as the edges of its triangles, one CSS pixel wide, in the
    /// colour its surface shows along each edge (`color` times `map` and
    /// the vertex colours, plus `emissive` times `emissiveMap`) or
    /// `wireframeColor`, depth tested and unlit (three.js
    /// `wireframe`, on every backend: lines, not a polygon mode). Its instances
    /// cast no shadow. 2 (what [`crate::MaterialStyle::wireframe`] sets):
    /// those edges over its triangles, drawn as usual.
    #[reflect(min = 0.0, max = 2.0, default = 0.0)]
    pub wireframe: u32,
    /// With `wireframe`: only the mesh's feature edges, its borders and the
    /// edges whose faces meet at more than this many degrees (a vector or
    /// neon outline of its form); 0 every triangle edge.
    #[reflect(min = 0.0, max = 180.0, default = 0.0, tunable)]
    pub wireframeAngle: f32,
    /// With `wireframe`: every edge in this linear colour instead of the
    /// surface's; black (the default) is the surface's.
    pub wireframeColor: Vec3,
    /// The uv offset `map` is sampled with, as three.js Texture.offset
    /// and glTF `KHR_texture_transform`: uv' = offset + rotate(rotation) *
    /// (repeat * uv), glTF's order.
    #[reflect(default = [0.0, 0.0], tunable)]
    pub mapOffset: Vec2,
    /// The uv scale of `map` (three.js Texture.repeat, glTF's `scale`).
    #[reflect(default = [1.0, 1.0], tunable)]
    pub mapRepeat: Vec2,
    /// The uv rotation of `map` in radians, counter-clockwise in uv space.
    #[reflect(default = 0.0, tunable)]
    pub mapRotation: f32,
    /// The uv offset `metalnessMap` and `roughnessMap` (one glTF texture; a physical material's packed maps too) is sampled with, as three.js Texture.offset
    /// and glTF `KHR_texture_transform`: uv' = offset + rotate(rotation) *
    /// (repeat * uv), glTF's order.
    #[reflect(default = [0.0, 0.0], tunable)]
    pub metalnessMapOffset: Vec2,
    /// The uv scale of `metalnessMap` and `roughnessMap` (one glTF texture; a physical material's packed maps too) (three.js Texture.repeat, glTF's `scale`).
    #[reflect(default = [1.0, 1.0], tunable)]
    pub metalnessMapRepeat: Vec2,
    /// The uv rotation of `metalnessMap` and `roughnessMap` (one glTF texture; a physical material's packed maps too) in radians, counter-clockwise in uv space.
    #[reflect(default = 0.0, tunable)]
    pub metalnessMapRotation: f32,
    /// The uv offset `normalMap` is sampled with, as three.js Texture.offset
    /// and glTF `KHR_texture_transform`: uv' = offset + rotate(rotation) *
    /// (repeat * uv), glTF's order.
    #[reflect(default = [0.0, 0.0], tunable)]
    pub normalMapOffset: Vec2,
    /// The uv scale of `normalMap` (three.js Texture.repeat, glTF's `scale`).
    #[reflect(default = [1.0, 1.0], tunable)]
    pub normalMapRepeat: Vec2,
    /// The uv rotation of `normalMap` in radians, counter-clockwise in uv space.
    #[reflect(default = 0.0, tunable)]
    pub normalMapRotation: f32,
    /// The uv offset `emissiveMap` is sampled with, as three.js Texture.offset
    /// and glTF `KHR_texture_transform`: uv' = offset + rotate(rotation) *
    /// (repeat * uv), glTF's order.
    #[reflect(default = [0.0, 0.0], tunable)]
    pub emissiveMapOffset: Vec2,
    /// The uv scale of `emissiveMap` (three.js Texture.repeat, glTF's `scale`).
    #[reflect(default = [1.0, 1.0], tunable)]
    pub emissiveMapRepeat: Vec2,
    /// The uv rotation of `emissiveMap` in radians, counter-clockwise in uv space.
    #[reflect(default = 0.0, tunable)]
    pub emissiveMapRotation: f32,
    /// The uv offset `aoMap` is sampled with, as three.js Texture.offset
    /// and glTF `KHR_texture_transform`: uv' = offset + rotate(rotation) *
    /// (repeat * uv), glTF's order.
    #[reflect(default = [0.0, 0.0], tunable)]
    pub aoMapOffset: Vec2,
    /// The uv scale of `aoMap` (three.js Texture.repeat, glTF's `scale`).
    #[reflect(default = [1.0, 1.0], tunable)]
    pub aoMapRepeat: Vec2,
    /// The uv rotation of `aoMap` in radians, counter-clockwise in uv space.
    #[reflect(default = 0.0, tunable)]
    pub aoMapRotation: f32,
}

impl Default for StandardMaterial {
    fn default() -> Self {
        StandardMaterial {
            color: Vec3::ONE,
            map: 0,
            metalness: 0.0,
            metalnessMap: 0,
            roughness: 1.0,
            roughnessMap: 0,
            normalMap: 0,
            normalScale: Vec2::ONE,
            aoMap: 0,
            aoMapIntensity: 1.0,
            emissive: Vec3::ZERO,
            emissiveMap: 0,
            emissiveIntensity: 1.0,
            opacity: 1.0,
            transparent: 0,
            alphaTest: 0.0,
            side: 0,
            flatShading: 0,
            alphaToCoverage: 0,
            envMapIntensity: 1.0,
            unlit: 0,
            toon: 0,
            gradientMap: 0,
            rimColor: Vec3::ONE,
            rimIntensity: 0.0,
            rimPower: 3.0,
            flashColor: Vec3::ONE,
            flash: 0.0,
            dissolve: 0.0,
            dissolveMap: 0,
            dissolveEdgeColor: Vec3::new(4.0, 1.5, 0.375),
            dissolveEdgeWidth: 0.0625,
            dissolveScale: 4.0,
            outlineColor: Vec3::ZERO,
            outlineWidth: 0.0,
            paintDetail: 0.0,
            foliage: Vec2::ZERO,
            clearcoat: 0.0,
            clearcoatRoughness: 0.0,
            clearcoatMap: 0,
            clearcoatRoughnessMap: 0,
            clearcoatNormalMap: 0,
            clearcoatNormalScale: Vec2::ONE,
            sheen: 0.0,
            sheenColor: Vec3::ZERO,
            sheenRoughness: 1.0,
            sheenColorMap: 0,
            sheenRoughnessMap: 0,
            transmission: 0.0,
            transmissionMap: 0,
            thickness: 0.0,
            thicknessMap: 0,
            attenuationDistance: 0.0,
            attenuationColor: Vec3::ONE,
            ior: 1.5,
            iridescence: 0.0,
            iridescenceIOR: 1.3,
            iridescenceThicknessRange: Vec2::new(100.0, 400.0),
            iridescenceMap: 0,
            iridescenceThicknessMap: 0,
            anisotropy: 0.0,
            anisotropyRotation: 0.0,
            anisotropyMap: 0,
            subsurface: 0.0,
            subsurfaceRadius: SKIN_RADIUS,
            mapArray: 0,
            mapLayer: 0,
            normalMapArray: 0,
            normalMapLayer: 0,
            styled: 1,
            weathering: 1.0,
            wireframe: 0,
            wireframeAngle: 0.0,
            wireframeColor: Vec3::ZERO,
            mapOffset: Vec2::ZERO,
            mapRepeat: Vec2::ONE,
            mapRotation: 0.0,
            metalnessMapOffset: Vec2::ZERO,
            metalnessMapRepeat: Vec2::ONE,
            metalnessMapRotation: 0.0,
            normalMapOffset: Vec2::ZERO,
            normalMapRepeat: Vec2::ONE,
            normalMapRotation: 0.0,
            emissiveMapOffset: Vec2::ZERO,
            emissiveMapRepeat: Vec2::ONE,
            emissiveMapRotation: 0.0,
            aoMapOffset: Vec2::ZERO,
            aoMapRepeat: Vec2::ONE,
            aoMapRotation: 0.0,
        }
    }
}

impl StandardMaterial {
    /// The uv transforms of the texture slots, in [`UvTransform`]'s slot
    /// order: `map`, `metalnessMap` (with `roughnessMap`), `normalMap`,
    /// `emissiveMap`, `aoMap`.
    pub fn uv_transforms(&self) -> [UvTransform; 5] {
        let t = |offset: Vec2, scale: Vec2, rotation: f32| UvTransform {
            offset,
            scale,
            rotation,
        };
        [
            t(self.mapOffset, self.mapRepeat, self.mapRotation),
            t(
                self.metalnessMapOffset,
                self.metalnessMapRepeat,
                self.metalnessMapRotation,
            ),
            t(
                self.normalMapOffset,
                self.normalMapRepeat,
                self.normalMapRotation,
            ),
            t(
                self.emissiveMapOffset,
                self.emissiveMapRepeat,
                self.emissiveMapRotation,
            ),
            t(self.aoMapOffset, self.aoMapRepeat, self.aoMapRotation),
        ]
    }

    /// Sets the slots' uv transforms ([`StandardMaterial::uv_transforms`]'s order).
    pub fn set_uv_transforms(&mut self, t: [UvTransform; 5]) {
        (self.mapOffset, self.mapRepeat, self.mapRotation) =
            (t[0].offset, t[0].scale, t[0].rotation);
        (
            self.metalnessMapOffset,
            self.metalnessMapRepeat,
            self.metalnessMapRotation,
        ) = (t[1].offset, t[1].scale, t[1].rotation);
        (
            self.normalMapOffset,
            self.normalMapRepeat,
            self.normalMapRotation,
        ) = (t[2].offset, t[2].scale, t[2].rotation);
        (
            self.emissiveMapOffset,
            self.emissiveMapRepeat,
            self.emissiveMapRotation,
        ) = (t[3].offset, t[3].scale, t[3].rotation);
        (self.aoMapOffset, self.aoMapRepeat, self.aoMapRotation) =
            (t[4].offset, t[4].scale, t[4].rotation);
    }

    /// three.js `MeshBasicMaterial`: unlit, `color` times `map`.
    pub fn basic(color: Vec3) -> StandardMaterial {
        StandardMaterial {
            color,
            unlit: 1,
            ..Default::default()
        }
    }

    pub fn side(&self) -> Side {
        Side::from_u32(self.side)
    }

    pub fn is_transparent(&self) -> bool {
        self.transparent != 0
    }

    /// Whether the dissolve discards pixels (shadows and the prepass must
    /// draw it with the material's shader too).
    pub fn dissolves(&self) -> bool {
        self.dissolve > 0.0
    }

    /// Whether it draws through the physical lit pipelines: any
    /// `MeshPhysicalMaterial` lobe or map beyond the scalar clear coat
    /// (clearcoat maps, sheen, transmission, iridescence, anisotropy, an
    /// `ior` other than 1.5). `unlit` and `toon` replace the lighting
    /// model, so neither is ever physical.
    pub fn is_physical(&self) -> bool {
        self.unlit == 0
            && self.toon == 0
            && (self.physical_textures().iter().any(Option::is_some)
                || self.sheen > 0.0
                || self.transmission > 0.0
                || self.iridescence > 0.0
                || self.anisotropy > 0.0
                || self.subsurface > 0.0
                || self.ior != 1.5)
    }

    /// Whether it sets a `MeshPhysicalMaterial` lobe: a clear coat or any
    /// field [`StandardMaterial::is_physical`] draws through the physical
    /// pipelines. The scene's toon passes such a material by
    /// ([`styled`]): toon shading has none of those lobes.
    pub fn has_physical_lobes(&self) -> bool {
        self.clearcoat > 0.0
            || self.physical_textures().iter().any(Option::is_some)
            || self.sheen > 0.0
            || self.transmission > 0.0
            || self.iridescence > 0.0
            || self.anisotropy > 0.0
            || self.subsurface > 0.0
            || self.ior != 1.5
    }

    /// Whether it refracts the scene behind it (drawn after the opaque
    /// scene is copied, back to front with the blended materials).
    pub fn is_transmissive(&self) -> bool {
        self.is_physical() && self.transmission > 0.0
    }

    /// The physical texture slots: clearcoat, clearcoat roughness,
    /// clearcoat normal, sheen colour, sheen roughness, transmission,
    /// thickness, iridescence, iridescence thickness, anisotropy.
    pub fn physical_textures(&self) -> [Option<TextureHandle>; 10] {
        [
            self.clearcoatMap,
            self.clearcoatRoughnessMap,
            self.clearcoatNormalMap,
            self.sheenColorMap,
            self.sheenRoughnessMap,
            self.transmissionMap,
            self.thicknessMap,
            self.iridescenceMap,
            self.iridescenceThicknessMap,
            self.anisotropyMap,
        ]
        .map(TextureHandle::from_slot)
    }

    /// Clears every texture slot that names `slot` (a removed texture's,
    /// [`TextureHandle::slot`]); true when one did.
    pub fn forget_texture(&mut self, slot: u32) -> bool {
        let mut forgot = false;
        for s in [
            &mut self.map,
            &mut self.metalnessMap,
            &mut self.roughnessMap,
            &mut self.normalMap,
            &mut self.emissiveMap,
            &mut self.aoMap,
            &mut self.gradientMap,
            &mut self.dissolveMap,
            &mut self.clearcoatMap,
            &mut self.clearcoatRoughnessMap,
            &mut self.clearcoatNormalMap,
            &mut self.sheenColorMap,
            &mut self.sheenRoughnessMap,
            &mut self.transmissionMap,
            &mut self.thicknessMap,
            &mut self.iridescenceMap,
            &mut self.iridescenceThicknessMap,
            &mut self.anisotropyMap,
        ] {
            if *s == slot {
                *s = 0;
                forgot = true;
            }
        }
        forgot
    }

    /// Whether its lit, prepass and shadow passes draw through the
    /// array-map pipelines ([`ArrayMaps`]): it names a texture array for
    /// its base colour or normal and is not physical (a material shader
    /// also keeps the standard path: the renderer checks that).
    pub fn draws_array_maps(&self) -> bool {
        (self.mapArray != 0 || self.normalMapArray != 0) && !self.is_physical()
    }

    /// The base colour's and the normal map's texture arrays.
    pub fn array_maps(&self) -> [Option<crate::TextureArrayHandle>; 2] {
        [self.mapArray, self.normalMapArray].map(crate::TextureArrayHandle::from_slot)
    }

    /// Clears the array slots that name `slot` (a removed texture array's,
    /// [`crate::TextureArrayHandle::slot`]); true when one did.
    pub fn forget_texture_array(&mut self, slot: u32) -> bool {
        let mut forgot = false;
        for s in [&mut self.mapArray, &mut self.normalMapArray] {
            if *s == slot {
                *s = 0;
                forgot = true;
            }
        }
        forgot
    }

    /// The texture slots in binding order (map, metalness, roughness,
    /// normal, emissive, ao, gradient, dissolve), as handles.
    pub fn textures(&self) -> [Option<TextureHandle>; 8] {
        [
            self.map,
            self.metalnessMap,
            self.roughnessMap,
            self.normalMap,
            self.emissiveMap,
            self.aoMap,
            self.gradientMap,
            self.dissolveMap,
        ]
        .map(TextureHandle::from_slot)
    }
}

/// The glTF-shaped material as a standard one: base colour and alpha become
/// `color` and `opacity`, the packed metallic-roughness texture both maps,
/// `Mask` becomes `alphaTest`, `Blend` `transparent`.
impl From<&Material> for StandardMaterial {
    fn from(m: &Material) -> StandardMaterial {
        let slot = |t: Option<TextureHandle>| t.map_or(0, TextureHandle::slot);
        let (alpha_test, transparent) = match m.alpha_mode {
            AlphaMode::Opaque => (0.0, 0),
            AlphaMode::Mask { cutoff } => (cutoff, 0),
            AlphaMode::Blend => (0.0, 1),
        };
        StandardMaterial {
            color: Vec3::new(m.base_color[0], m.base_color[1], m.base_color[2]),
            map: slot(m.base_color_texture),
            metalness: m.metallic,
            metalnessMap: slot(m.metallic_roughness_texture),
            roughness: m.roughness,
            roughnessMap: slot(m.metallic_roughness_texture),
            normalMap: slot(m.normal_texture),
            normalScale: Vec2::splat(m.normal_scale),
            aoMap: 0,
            aoMapIntensity: 1.0,
            emissive: Vec3::from(m.emissive),
            emissiveMap: slot(m.emissive_texture),
            emissiveIntensity: 1.0,
            opacity: m.base_color[3],
            transparent,
            alphaTest: alpha_test,
            side: 0,
            flatShading: 0,
            envMapIntensity: 1.0,
            unlit: 0,
            ..StandardMaterial::default()
        }
    }
}

impl From<Material> for StandardMaterial {
    fn from(m: Material) -> StandardMaterial {
        StandardMaterial::from(&m)
    }
}

/// glTF's physical material extensions on one material, in glTF's terms
/// (`KHR_materials_clearcoat`, `_sheen`, `_transmission`, `_volume`,
/// `_ior`, `_iridescence`, `_anisotropy`), with textures as `T`: a scene's
/// [`TextureHandle`], or a model's texture index before it has a scene.
/// `present` says which extensions the material has
/// ([`PhysicalExtensions::CLEARCOAT`] and on); the others' fields are
/// glTF's defaults. [`PhysicalExtensions::apply`] lays them over a
/// [`StandardMaterial`] as three.js's `GLTFLoader` does. The glTF bridge
/// and the cook read them (`gltf::physical_extensions`).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PhysicalExtensions<T> {
    pub present: u32,
    pub clearcoat: f32,
    pub clearcoat_texture: Option<T>,
    pub clearcoat_roughness: f32,
    pub clearcoat_roughness_texture: Option<T>,
    pub clearcoat_normal_texture: Option<T>,
    pub clearcoat_normal_scale: f32,
    pub sheen_color: [f32; 3],
    pub sheen_color_texture: Option<T>,
    pub sheen_roughness: f32,
    pub sheen_roughness_texture: Option<T>,
    pub transmission: f32,
    pub transmission_texture: Option<T>,
    pub thickness: f32,
    pub thickness_texture: Option<T>,
    /// glTF's +infinity (no absorption) is 0, as in [`StandardMaterial`].
    pub attenuation_distance: f32,
    pub attenuation_color: [f32; 3],
    pub ior: f32,
    pub iridescence: f32,
    pub iridescence_texture: Option<T>,
    pub iridescence_ior: f32,
    /// Minimum and maximum film thickness (nm).
    pub iridescence_thickness: [f32; 2],
    pub iridescence_thickness_texture: Option<T>,
    pub anisotropy: f32,
    pub anisotropy_rotation: f32,
    pub anisotropy_texture: Option<T>,
}

/// Skin's scatter distance per channel, in metres:
/// [`StandardMaterial::subsurfaceRadius`]'s default.
pub const SKIN_RADIUS: Vec3 = Vec3::new(0.0037, 0.0014, 0.0007);

impl<T> Default for PhysicalExtensions<T> {
    fn default() -> Self {
        PhysicalExtensions {
            present: 0,
            clearcoat: 0.0,
            clearcoat_texture: None,
            clearcoat_roughness: 0.0,
            clearcoat_roughness_texture: None,
            clearcoat_normal_texture: None,
            clearcoat_normal_scale: 1.0,
            sheen_color: [0.0; 3],
            sheen_color_texture: None,
            sheen_roughness: 0.0,
            sheen_roughness_texture: None,
            transmission: 0.0,
            transmission_texture: None,
            thickness: 0.0,
            thickness_texture: None,
            attenuation_distance: 0.0,
            attenuation_color: [1.0; 3],
            ior: 1.5,
            iridescence: 0.0,
            iridescence_texture: None,
            iridescence_ior: 1.3,
            iridescence_thickness: [100.0, 400.0],
            iridescence_thickness_texture: None,
            anisotropy: 0.0,
            anisotropy_rotation: 0.0,
            anisotropy_texture: None,
        }
    }
}

impl<T> PhysicalExtensions<T> {
    /// [`PhysicalExtensions::present`] bits, one per extension.
    pub const CLEARCOAT: u32 = 1;
    pub const SHEEN: u32 = 2;
    pub const TRANSMISSION: u32 = 4;
    pub const VOLUME: u32 = 8;
    pub const IOR: u32 = 16;
    pub const IRIDESCENCE: u32 = 32;
    pub const ANISOTROPY: u32 = 64;
}

impl<T: Copy> PhysicalExtensions<T> {
    /// The same with every texture mapped by `f` (`None`: dropped).
    pub fn map<U>(&self, f: impl Fn(T) -> Option<U>) -> PhysicalExtensions<U> {
        let t = |x: Option<T>| x.and_then(&f);
        PhysicalExtensions {
            present: self.present,
            clearcoat: self.clearcoat,
            clearcoat_texture: t(self.clearcoat_texture),
            clearcoat_roughness: self.clearcoat_roughness,
            clearcoat_roughness_texture: t(self.clearcoat_roughness_texture),
            clearcoat_normal_texture: t(self.clearcoat_normal_texture),
            clearcoat_normal_scale: self.clearcoat_normal_scale,
            sheen_color: self.sheen_color,
            sheen_color_texture: t(self.sheen_color_texture),
            sheen_roughness: self.sheen_roughness,
            sheen_roughness_texture: t(self.sheen_roughness_texture),
            transmission: self.transmission,
            transmission_texture: t(self.transmission_texture),
            thickness: self.thickness,
            thickness_texture: t(self.thickness_texture),
            attenuation_distance: self.attenuation_distance,
            attenuation_color: self.attenuation_color,
            ior: self.ior,
            iridescence: self.iridescence,
            iridescence_texture: t(self.iridescence_texture),
            iridescence_ior: self.iridescence_ior,
            iridescence_thickness: self.iridescence_thickness,
            iridescence_thickness_texture: t(self.iridescence_thickness_texture),
            anisotropy: self.anisotropy,
            anisotropy_rotation: self.anisotropy_rotation,
            anisotropy_texture: t(self.anisotropy_texture),
        }
    }
}

impl PhysicalExtensions<TextureHandle> {
    /// Sets `m`'s fields from the extensions the material has (three.js's
    /// `GLTFLoader`: a material with `KHR_materials_sheen` gets `sheen` 1).
    pub fn apply(&self, m: &mut StandardMaterial) {
        let slot = |t: Option<TextureHandle>| t.map_or(0, TextureHandle::slot);
        let has = |bit: u32| self.present & bit != 0;
        if has(Self::CLEARCOAT) {
            m.clearcoat = self.clearcoat;
            m.clearcoatMap = slot(self.clearcoat_texture);
            m.clearcoatRoughness = self.clearcoat_roughness;
            m.clearcoatRoughnessMap = slot(self.clearcoat_roughness_texture);
            m.clearcoatNormalMap = slot(self.clearcoat_normal_texture);
            m.clearcoatNormalScale = Vec2::splat(self.clearcoat_normal_scale);
        }
        if has(Self::SHEEN) {
            m.sheen = 1.0;
            m.sheenColor = Vec3::from(self.sheen_color);
            m.sheenColorMap = slot(self.sheen_color_texture);
            m.sheenRoughness = self.sheen_roughness;
            m.sheenRoughnessMap = slot(self.sheen_roughness_texture);
        }
        if has(Self::TRANSMISSION) {
            m.transmission = self.transmission;
            m.transmissionMap = slot(self.transmission_texture);
        }
        if has(Self::VOLUME) {
            m.thickness = self.thickness;
            m.thicknessMap = slot(self.thickness_texture);
            m.attenuationDistance = self.attenuation_distance;
            m.attenuationColor = Vec3::from(self.attenuation_color);
        }
        if has(Self::IOR) {
            m.ior = self.ior;
        }
        if has(Self::IRIDESCENCE) {
            m.iridescence = self.iridescence;
            m.iridescenceMap = slot(self.iridescence_texture);
            m.iridescenceIOR = self.iridescence_ior;
            m.iridescenceThicknessRange = Vec2::from(self.iridescence_thickness);
            m.iridescenceThicknessMap = slot(self.iridescence_thickness_texture);
        }
        if has(Self::ANISOTROPY) {
            m.anisotropy = self.anisotropy;
            m.anisotropyRotation = self.anisotropy_rotation;
            m.anisotropyMap = slot(self.anisotropy_texture);
        }
    }
}
