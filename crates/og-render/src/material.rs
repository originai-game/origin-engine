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
#[reflect(name = "og.StandardMaterial", version = 4)]
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
        }
    }
}

impl StandardMaterial {
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
                || self.ior != 1.5)
    }

    /// Whether it refracts the scene behind it (drawn after the opaque
    /// scene is copied, back to front with the blended materials).
    pub fn is_transmissive(&self) -> bool {
        self.is_physical() && self.transmission > 0.0
    }

    /// The physical texture slots: clearcoat, clearcoat roughness,
    /// clearcoat normal, sheen colour, sheen roughness, transmission,
    /// thickness, iridescence, iridescence thickness, anisotropy.
    pub(crate) fn physical_textures(&self) -> [Option<TextureHandle>; 10] {
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

    /// The texture slots in binding order (map, metalness, roughness,
    /// normal, emissive, ao, gradient, dissolve), as handles.
    pub(crate) fn textures(&self) -> [Option<TextureHandle>; 8] {
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

/// `struct MaterialFactors` in pbr.wgsl.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
pub(crate) struct MaterialUniform {
    /// rgb: colour, a: opacity.
    pub base_color: [f32; 4],
    /// x: metalness, y: roughness, zw: normal scale.
    pub mr: [f32; 4],
    /// rgb: emissive x intensity, w: alpha cutoff (< 0: none).
    pub emissive: [f32; 4],
    /// x: normal map (0 none, 1 RGB, 2 two-channel), y: ao intensity (< 0: no
    /// map), z: environment intensity, w: mode bits (1 flat, 2 unlit, 4 blended).
    pub flags: [f32; 4],
    /// x: toon (0 off, 1 gradient map, 2 built-in bands), y: rim intensity,
    /// z: rim power, w: flash amount.
    pub toon: [f32; 4],
    /// rgb: rim colour, w: dissolve amount.
    pub rim: [f32; 4],
    /// rgb: flash colour, w: dissolve edge width.
    pub flash: [f32; 4],
    /// rgb: dissolve edge colour, w: noise scale (negative: the dissolve map).
    pub dissolve: [f32; 4],
    /// rgb: outline colour, w: outline width in pixels.
    pub outline: [f32; 4],
    /// x: clearcoat, y: clearcoat roughness.
    pub coat: [f32; 4],
    /// A material shader's parameters (`hooks.rs`).
    pub custom: [[f32; 4]; crate::MAX_SHADER_PARAMS],
}

/// The uniform for `m`; `two_channel_normal` when its normal map is BC5/EAC.
pub(crate) fn uniform(m: &StandardMaterial, two_channel_normal: bool) -> MaterialUniform {
    let has_normal = m.normalMap != 0;
    let cutoff = if m.alphaTest > 0.0 { m.alphaTest } else { -1.0 };
    let mode = (m.flatShading != 0) as u32
        | ((m.unlit != 0) as u32) << 1
        | (m.is_transparent() as u32) << 2;
    let scale = if has_normal { m.normalScale } else { Vec2::ONE };
    MaterialUniform {
        base_color: [m.color.x, m.color.y, m.color.z, m.opacity],
        mr: [m.metalness, m.roughness, scale.x, scale.y],
        emissive: [
            m.emissive.x * m.emissiveIntensity,
            m.emissive.y * m.emissiveIntensity,
            m.emissive.z * m.emissiveIntensity,
            cutoff,
        ],
        flags: [
            match (has_normal, two_channel_normal) {
                (false, _) => 0.0,
                (true, false) => 1.0,
                (true, true) => 2.0,
            },
            if m.aoMap != 0 { m.aoMapIntensity } else { -1.0 },
            m.envMapIntensity,
            mode as f32,
        ],
        toon: [
            match (m.toon != 0, m.gradientMap != 0) {
                (false, _) => 0.0,
                (true, true) => 1.0,
                (true, false) => 2.0,
            },
            m.rimIntensity,
            m.rimPower.max(0.1),
            m.flash.clamp(0.0, 1.0),
        ],
        rim: [
            m.rimColor.x,
            m.rimColor.y,
            m.rimColor.z,
            m.dissolve.clamp(0.0, 1.0),
        ],
        flash: [
            m.flashColor.x,
            m.flashColor.y,
            m.flashColor.z,
            m.dissolveEdgeWidth.max(0.0),
        ],
        dissolve: [
            m.dissolveEdgeColor.x,
            m.dissolveEdgeColor.y,
            m.dissolveEdgeColor.z,
            if m.dissolveMap != 0 {
                -1.0
            } else {
                m.dissolveScale.max(1e-3)
            },
        ],
        outline: [
            m.outlineColor.x,
            m.outlineColor.y,
            m.outlineColor.z,
            m.outlineWidth.clamp(0.0, 31.0),
        ],
        coat: [
            m.clearcoat.clamp(0.0, 1.0),
            m.clearcoatRoughness.clamp(0.0, 1.0),
            0.0,
            0.0,
        ],
        custom: [[0.0; 4]; crate::MAX_SHADER_PARAMS],
    }
}
