//! Import of glTF 2.0 (`.glb`, and `.gltf` with embedded or external
//! buffers), Wavefront OBJ and FBX 7 ([`crate::tools::import_fbx`]) into parts: a [`Solid`] each, in its own
//! coordinates, with the transform that places it.
//!
//! Units and axes: the kernel works in millimetres with Z up. glTF is
//! metres with Y up by its specification, and OBJ is read the same way
//! unless [`Import`] says otherwise; both are turned into kernel space
//! (glTF −Z becomes kernel +Y, +Y becomes +Z, as the exporter does in
//! reverse). Transforms are conjugated by the same axis change exactly
//! (only signs and swaps), so no rounding enters there.
//!
//! Names: each part's faces carry the part's name (the glTF node's name,
//! else its mesh's, else `node<i>`; the OBJ `o` or `g` name, else
//! `object`), made unique with `.1`, `.2`… Each glTF primitive and each
//! OBJ material run is its own surface.
//!
//! Every polygon is triangulated on the way in (glTF strips and fans,
//! OBJ faces of more than three corners). Triangles stay in file order and
//! positions are not welded; repair welds. The first UV set (glTF
//! `TEXCOORD_0`, OBJ `vt`, FBX's first UV layer) goes into
//! [`Solid::uvs`], per triangle corner, when every triangle of the part
//! has one; a part where only some do has none, and the import says so
//! in [`Imported::skipped`]. Vertex colours (glTF `COLOR_0`, OBJ
//! `v x y z r g b`) go into [`Solid::colors`], linear RGBA per corner.
//! The part's first material is kept as a [`Material`]
//! ([`ImportedPart::material`]): its factors as the file means them
//! (glTF's defaults where it leaves one unset, listed in
//! [`ImportedPart::defaults`]) and its maps (base colour, metallic-
//! roughness, normal, emissive, occlusion) as their files' bytes.

use std::fmt;

use crate::math::{DVec3, Frame};
use crate::mesh::{Skin, Solid};

use crate::tools::export::Material;
use crate::tools::image::ImageInfo;
use crate::tools::place::{Axis, AxisTurn, turned};

mod space;
pub use space::*;

/// How to read a file. The zero value reads every format by its own
/// convention.
///
/// `up` and `front` (OBJ and glTF) name the file's own axes that are the
/// object's up and the way it faces; they become the kernel's +Z and −Y
/// by one of the 24 exact axis turns (`crate::tools::place`), through
/// [`import`]. Give one and the
/// other follows by the smallest turn. Without either, the file is read
/// Y up and facing +Z, as glTF says (`z_up`: Z up and facing −Y). FBX
/// says its axes in its `GlobalSettings`, and takes none of these.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Import {
    /// OBJ only: millimetres per file unit; 0 reads metres (1000), as
    /// glTF is.
    pub unit_mm: f64,
    /// OBJ and glTF: the file is Z up already (no axis change); the same
    /// as `up` +z and `front` −y, and not given with them.
    pub z_up: bool,
    /// OBJ and glTF: the file's axis that is up.
    pub up: Option<Axis>,
    /// OBJ and glTF: the file's axis the object faces.
    pub front: Option<Axis>,
}

impl Import {
    /// The turn from the file's axes to the kernel's, or why the options
    /// disagree.
    pub fn axes(&self) -> Result<AxisTurn, ImportError> {
        let given = self.up.is_some() || self.front.is_some();
        if self.z_up && given {
            return Err(ImportError::Unsupported(
                "z_up together with up or front: z_up is up +z, front -y; give one or the other"
                    .into(),
            ));
        }
        if self.z_up {
            return Ok(AxisTurn::IDENTITY);
        }
        let (up, front) = if given {
            (self.up, self.front)
        } else {
            (Some(Axis::PosY), None)
        };
        AxisTurn::new(up, front).map_err(ImportError::Unsupported)
    }

    /// The same reading as a plain read and a turn after it: these
    /// options without `z_up`, `up` and `front`, and the kernel axes, of
    /// the file read so, that the options make up and front (for
    /// [`crate::tools::Place`]'s `up` and `front`; `None` when they turn
    /// nothing). For OBJ and glTF; FBX takes no axes.
    pub fn as_turn(&self) -> Result<(Import, Option<Axis>, Option<Axis>), ImportError> {
        let plain = Import {
            unit_mm: self.unit_mm,
            ..Default::default()
        };
        if !self.turns() {
            return Ok((plain, None, None));
        }
        let y_up = AxisTurn::new(Some(Axis::PosY), None).map_err(ImportError::Unsupported)?;
        let back = y_up.then(&self.axes()?.inverse());
        Ok((
            plain,
            Some(back.image(Axis::PosZ)),
            Some(back.image(Axis::NegY)),
        ))
    }

    /// Whether the options ask for other axes than the format's own.
    pub fn turns(&self) -> bool {
        self.z_up || self.up.is_some() || self.front.is_some()
    }
}

/// One placed piece of an imported model.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportedPart {
    /// Unique among the parts; every face carries it as a name.
    pub name: String,
    /// The geometry in the part's own coordinates, kernel units.
    pub solid: Solid,
    /// Places the part in the model (kernel units).
    pub transform: Affine,
    /// The first material the part uses (its name "" when it has none):
    /// factors as the file means them, and the maps whose bytes could be
    /// read (glTF's five; OBJ `map_Kd`, `map_Ke`, `norm`; FBX
    /// `DiffuseColor`, `EmissiveColor`, `NormalMap`), sampled at the
    /// solid's [`Solid::uvs`]. glTF's factors are linear
    /// ([`ColorSpace::Linear`]); MTL's and FBX's colours sRGB.
    pub material: Material,
    /// The glTF material factors the file leaves unset, which take the
    /// specification's defaults (`metallicFactor`: 1, a metal), by their
    /// glTF names; empty for OBJ and FBX.
    pub defaults: Vec<String>,
    /// The file's animations of this part's skin, each sampled at its key
    /// times (`crate::mesh::SkinClip`): glTF `animations` on the skin's
    /// joints and the nodes above them. Empty for a part without a skin,
    /// and for OBJ.
    pub clips: Vec<crate::mesh::SkinClip>,
}

/// What an import read.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Imported {
    /// The model's pieces, in file order (glTF: the scene's nodes, depth
    /// first).
    pub parts: Vec<ImportedPart>,
    /// The textures the file references, with their sizes.
    pub images: Vec<ImageInfo>,
    /// Polygons of more than three corners that were triangulated.
    pub polygons_triangulated: usize,
    /// Things read past and not imported (points, lines, skins, morph
    /// targets), one line each.
    pub skipped: Vec<String>,
    /// The uniform scales taken out of skinned parts' bones, one per
    /// skinned part that had one.
    pub skin_scales: Vec<SkinScale>,
    /// glTF skins no mesh uses (an animation file that carries the
    /// skeleton alone), each with its clips, read with glTF's axes
    /// (`up` and `front` do not turn them). A skin that cannot be read is
    /// a line in [`Imported::skipped`] instead.
    pub skeletons: Vec<ImportedSkeleton>,
}

/// A skeleton with no mesh, and the clips that move it.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct ImportedSkeleton {
    /// The skin's name in the file, else `skinK`.
    pub name: String,
    /// The bones, with no vertices weighted.
    pub skin: Skin,
    /// The file's animations of it, as [`ImportedPart::clips`].
    pub clips: Vec<crate::mesh::SkinClip>,
}

/// A uniform scale taken out of a skinned part's bones: their bind
/// transforms were rigid up to it (an armature scaled 0.01 for
/// centimetres, say). The bones' rest frames are their bind transforms in
/// the world (from the file's inverse bind matrices) with it divided out;
/// the vertices are placed where the file's bind pose puts them in the
/// world; the inverse bind matrices are recomputed from the rest frames.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SkinScale {
    /// The part.
    pub part: String,
    /// The scale every bone's world transform carried.
    pub scale: f64,
    /// How far, in mm, the file's inverse bind matrices, with the scale
    /// factored back in, put a vertex from where the recomputed ones do:
    /// they agree to this.
    pub bind_error: f64,
    /// The largest departure of a bone's world transform from a uniform
    /// scale (relative axis lengths and cosines; exporters' rounding,
    /// below [`UNEVEN`]), made exact in its rest frame.
    pub uneven: f64,
    /// How far, in mm, the file's own bone nodes stand from the bind pose
    /// at the vertices they weight: 0 to rounding when the file is saved
    /// at its bind pose, more where it is saved posed. The bind pose (the
    /// inverse bind matrices) is what is kept.
    pub node_offset: f64,
}

/// Departures from a uniform scale up to this (relative: axis lengths,
/// and the cosines between axes) are exporters' rounding, and taken as
/// uniform; beyond it a scale is uneven or sheared.
pub const UNEVEN: f64 = 1e-4;

/// Why a file could not be imported.
#[derive(Clone, Debug, PartialEq)]
pub enum ImportError {
    /// Not a file of the format, or broken (what and where).
    Format(String),
    /// A feature the importer does not read, such as a required glTF
    /// extension.
    Unsupported(String),
    /// An external buffer could not be had.
    External {
        /// The URI the file names.
        uri: String,
        /// What the resolver said.
        why: String,
    },
    /// A position is NaN or infinite (which part).
    NotFinite(String),
}

impl fmt::Display for ImportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ImportError::Format(w) => write!(f, "not a readable file: {w}"),
            ImportError::Unsupported(w) => write!(f, "not supported: {w}"),
            ImportError::External { uri, why } => write!(f, "cannot read {uri:?}: {why}"),
            ImportError::NotFinite(p) => write!(f, "part {p:?} has a position that is not finite"),
        }
    }
}

impl std::error::Error for ImportError {}

/// Fetches a file a model names by relative URI (percent-decoded), for
/// external glTF buffers and images and OBJ material libraries.
pub type Resolve<'a> = &'a dyn Fn(&str) -> Result<Vec<u8>, String>;

/// A resolver that has no files: external references fail.
pub fn no_files(uri: &str) -> Result<Vec<u8>, String> {
    Err(format!("no file system to read {uri:?} from"))
}

/// A file that is not what its format says.
pub fn fmt_err(what: impl Into<String>) -> ImportError {
    ImportError::Format(what.into())
}
