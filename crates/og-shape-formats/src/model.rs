//! A model's files: GLB (plain or with an atlas), OBJ, STL, URDF and USD,
//! written from the scene the kernel builds ([`Model::build`]).

use og_shape::mesh::Solid;
use og_shape::model::{Model, ModelError};
use og_shape::tools::Material;

/// What [`Model::glb_atlas`] wrote.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct AtlasExport {
    /// The GLB.
    pub glb: Vec<u8>,
    /// The packing: where each island went, and the atlas's measures.
    pub atlas: og_shape::tools::AtlasReport,
    /// Materials the packed parts share, once alike ones are merged.
    pub materials: usize,
    /// Parts with UVs left out of the atlas: their material has a map.
    pub own: Vec<String>,
}

/// The files of a [`Model`].
pub trait ModelFiles {
    /// The model as GLB, posed (`crate::gltf::write::glb`): metres, Y up, joints
    /// as `s3d_joint` extras.
    fn glb(&self) -> Result<Vec<u8>, ModelError>;

    /// The model as GLB ([`ModelFiles::glb`]) with its parts' UVs packed into
    /// one atlas (`og_shape::tools::repack`: every UV island of every part
    /// with UVs and a material without maps, at one texel density, on
    /// `d`'s grid), and parts whose materials then agree (names aside)
    /// sharing one material, named after the first. A part whose material
    /// has a map keeps its own UVs (its map was painted for them) and its
    /// material: [`AtlasExport::own`] names it. Joints, placements and
    /// names are as [`ModelFiles::glb`] writes them.
    fn glb_atlas(&self, d: &og_shape::tools::Atlas) -> Result<AtlasExport, ModelError>;

    /// The model's meshes as OBJ text, at its pose when one is set (the
    /// header then ends `posed`; [`Model::mesh_placement`]), else at rest.
    /// Parts no joint moves keep their rest positions bit for bit.
    fn obj(&self) -> Result<String, ModelError>;

    /// The model's meshes as binary STL, at its pose when one is set (the
    /// header then ends `posed`), else at rest.
    fn stl(&self) -> Result<Vec<u8>, ModelError>;

    /// The model as URDF (`crate::urdf::write::urdf`): its parts as links at
    /// rest, its joints, and each link's STL named `<prefix>_<part>.stl`.
    fn urdf(&self, prefix: &str) -> Result<crate::urdf::write::Urdf, ModelError>;

    /// [`ModelFiles::urdf`] with each closed link's `inertial` at a density
    /// and the joints' limits (`crate::urdf::write::urdf_with`).
    fn urdf_with(
        &self,
        prefix: &str,
        d: &crate::urdf::write::UrdfDesc,
    ) -> Result<crate::urdf::write::Urdf, ModelError>;

    /// The model as a USD stage (`crate::usd::write::usda`): its parts as
    /// meshes at rest with their materials, maps named
    /// `<prefix>_<part>_<kind>`.
    fn usda(&self, prefix: &str) -> Result<crate::usd::write::Usd, ModelError>;
}

impl ModelFiles for Model {
    /// The model as GLB, posed (`crate::gltf::write::glb`): metres, Y up, joints
    /// as `s3d_joint` extras.
    fn glb(&self) -> Result<Vec<u8>, ModelError> {
        Ok(crate::gltf::write::glb_clips(
            &self.build()?.scene(),
            &self.scene_clips(),
        )?)
    }

    /// The model as GLB ([`ModelFiles::glb`]) with its parts' UVs packed into
    /// one atlas (`og_shape::tools::repack`: every UV island of every part
    /// with UVs and a material without maps, at one texel density, on
    /// `d`'s grid), and parts whose materials then agree (names aside)
    /// sharing one material, named after the first. A part whose material
    /// has a map keeps its own UVs (its map was painted for them) and its
    /// material: [`AtlasExport::own`] names it. Joints, placements and
    /// names are as [`ModelFiles::glb`] writes them.
    fn glb_atlas(&self, d: &og_shape::tools::Atlas) -> Result<AtlasExport, ModelError> {
        let built = self.build()?;
        let mut scene = built.scene();
        let packs = |m: &Material, s: &Solid| s.uvs.is_some() && m.maps().is_empty();
        let mut own = Vec::new();
        let mut taken: Vec<usize> = Vec::new();
        let mut solids: Vec<Solid> = Vec::new();
        for (k, p) in scene.parts.iter().enumerate() {
            if packs(&p.material, p.solid) {
                taken.push(k);
                solids.push(p.solid.clone());
            } else if p.solid.uvs.is_some() {
                own.push(p.name.clone());
            }
        }
        if solids.is_empty() {
            return Err(ModelError::Atlas(og_shape::tools::AtlasError::NoUvs));
        }
        let atlas = og_shape::tools::repack(&mut solids, d)?;
        // Materials alike (names aside) take the first one's name.
        let bare = |m: &Material| Material {
            name: String::new(),
            ..m.clone()
        };
        let mut firsts: Vec<(Material, String)> = Vec::new();
        for &k in &taken {
            let p = &mut scene.parts[k];
            let b = bare(&p.material);
            let name = match firsts.iter().find(|(m, _)| *m == b) {
                Some((_, n)) => n.clone(),
                None => {
                    let n = if p.material.name.is_empty() {
                        p.name.clone()
                    } else {
                        p.material.name.clone()
                    };
                    firsts.push((b, n.clone()));
                    n
                }
            };
            p.material.name = name;
        }
        for (k, s) in taken.iter().zip(&solids) {
            scene.parts[*k].solid = s;
        }
        let glb = crate::gltf::write::glb_shared(&scene)?;
        Ok(AtlasExport {
            glb,
            atlas,
            materials: firsts.len(),
            own,
        })
    }

    /// The model's meshes as OBJ text, at its pose when one is set (the
    /// header then ends `posed`; [`Model::mesh_placement`]), else at rest.
    /// Parts no joint moves keep their rest positions bit for bit.
    fn obj(&self) -> Result<String, ModelError> {
        Ok(crate::obj::write::obj_at(
            &self.build()?.scene(),
            self.mesh_placement(),
        )?)
    }

    /// The model's meshes as binary STL, at its pose when one is set (the
    /// header then ends `posed`), else at rest.
    fn stl(&self) -> Result<Vec<u8>, ModelError> {
        Ok(crate::stl::write::stl_at(
            &self.build()?.scene(),
            self.mesh_placement(),
        )?)
    }

    /// The model as URDF (`crate::urdf::write::urdf`): its parts as links at
    /// rest, its joints, and each link's STL named `<prefix>_<part>.stl`.
    fn urdf(&self, prefix: &str) -> Result<crate::urdf::write::Urdf, ModelError> {
        self.urdf_with(prefix, &crate::urdf::write::UrdfDesc::default())
    }

    /// [`ModelFiles::urdf`] with each closed link's `inertial` at a density
    /// and the joints' limits (`crate::urdf::write::urdf_with`).
    fn urdf_with(
        &self,
        prefix: &str,
        d: &crate::urdf::write::UrdfDesc,
    ) -> Result<crate::urdf::write::Urdf, ModelError> {
        Ok(crate::urdf::write::urdf_with(
            &self.build()?.scene(),
            prefix,
            d,
        )?)
    }

    /// The model as a USD stage (`crate::usd::write::usda`): its parts as
    /// meshes at rest with their materials, maps named
    /// `<prefix>_<part>_<kind>`.
    fn usda(&self, prefix: &str) -> Result<crate::usd::write::Usd, ModelError> {
        Ok(crate::usd::write::usda(&self.build()?.scene(), prefix)?)
    }
}
