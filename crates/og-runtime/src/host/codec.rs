//! Reflected layouts as the hosts see them: which column kind each field has,
//! and the conversion between a value's JSON (what crosses the boundary) and
//! its packed bytes (what `og-ecs` stores).
//!
//! A list field (`list(Asset(), 4)`, the IDL's `List.Post.8`) has its
//! element's kind and a list type (`u64[4]`): its JSON is an array of the
//! element's values, refused past the capacity, never truncated.

use og_core::reflect::{FieldSchema, FieldType, Schema};
use serde_json::{Map, Value, json};

mod field;
mod layout;
mod rows;

use field::*;
pub use rows::*;

/// A column kind of the IDL (`reflect`, `world`, `render` modules).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    F32,
    F64,
    U32,
    I32,
    Bool,
    Vector2,
    Vector3,
    Quaternion,
    Matrix4,
    Color,
    Entity,
    Geometry,
    Material,
    Asset,
    Sound,
    /// Fixed-capacity UTF-8 text: a `str(N)` field (`strN`), or an event's
    /// `String` (a `str256`).
    Str,
    /// An enum: a u32 index into the field's variants.
    Enum,
    /// Shared timing tag and parameter payload.
    Ease,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Kind> {
        Some(match name {
            "f32" => Kind::F32,
            "f64" => Kind::F64,
            "u32" => Kind::U32,
            "i32" => Kind::I32,
            "bool" => Kind::Bool,
            "Vector2" => Kind::Vector2,
            "Vector3" => Kind::Vector3,
            "Quaternion" => Kind::Quaternion,
            "Matrix4" => Kind::Matrix4,
            "Color" => Kind::Color,
            "Entity" => Kind::Entity,
            "Geometry" => Kind::Geometry,
            "Material" => Kind::Material,
            "Asset" => Kind::Asset,
            "Sound" => Kind::Sound,
            // The handles og.load answers beside Asset: fonts, UI styles, looks, shaders, rigs.
            "Font" | "Style" | "Look" | "Shader" | "Rig" => Kind::Asset,
            // The UI's draw assets: vectors and UI materials.
            "VectorArt" | "UiMaterial" => Kind::Asset,
            // The animation module's asset handles: u64 like every handle.
            "SkeletonAsset" | "ClipAsset" | "GraphAsset" => Kind::Asset,
            // The effect assets and textures in records (Decal atlas, LUT, emitter).
            "Texture" | "Lut" | "ParticleEmitter" | "EnvironmentMap" | "Palette"
            | "TextureArray" => Kind::Asset,
            // The E4 handles: flipbook sheets, tilemaps, tweens.
            "FlipbookSheet" | "TilemapAsset" | "Tween" | "Rig2D" => Kind::Asset,
            // Footage (og.load.video).
            "Video" => Kind::Asset,
            // The E6 handles: navigation (grids, navmeshes, flow fields) and behaviours.
            "Navigation" | "Behaviour" => Kind::Asset,
            // Phase 2: terrains.
            "TerrainData" => Kind::Asset,
            // Depth S2: paths and timelines.
            "Path" | "TimelineAsset" => Kind::Asset,
            // 3D VFX: composed effects.
            "VfxAsset" => Kind::Asset,
            // Depth S8: render targets and custom post passes.
            "RenderTarget" | "Post" => Kind::Asset,
            "String" | "str" => Kind::Str,
            "Ease" => Kind::Ease,
            _ => return None,
        })
    }

    /// The JSON keys of a struct-valued kind, in lane order.
    pub(crate) fn keys(self) -> &'static [&'static str] {
        match self {
            Kind::Vector2 => &["x", "y"],
            Kind::Vector3 => &["x", "y", "z"],
            Kind::Quaternion => &["x", "y", "z", "w"],
            Kind::Color => &["r", "g", "b"],
            _ => &[],
        }
    }

    /// The lane [`Kind::keys`] gives `key` (a vector's `y` is 1): every key
    /// is one byte, compared here as one (a string compare per lane was a
    /// tenth of reading a sprite).
    #[inline]
    fn lane_of(self, key: &str) -> Option<usize> {
        let &[key] = key.as_bytes() else {
            return None;
        };
        self.keys().iter().position(|k| k.as_bytes() == [key])
    }

    /// The og-core field type this kind is stored as: a `bool` is one byte
    /// (a `Uint8Array` column), a `Matrix4` sixteen f32, handles a `u64`, a
    /// `str(N)` field its `capacity` bytes (an event's text without one a
    /// `str256`), an enum a `u32` index; with `list`, a list of at most that
    /// many of a number or a handle.
    pub fn field_type(
        self,
        viewed: bool,
        capacity: Option<usize>,
        list: Option<usize>,
    ) -> Result<FieldType, String> {
        if let Some(n) = list {
            let element = match self {
                Kind::F32 => og_core::Scalar::F32,
                Kind::F64 => og_core::Scalar::F64,
                Kind::U32 => og_core::Scalar::U32,
                Kind::I32 => og_core::Scalar::I32,
                k if k.is_handle() => og_core::Scalar::U64,
                _ => {
                    return Err(
                        "a list holds f32, f64, u32, i32 or a handle (Asset(), Entity(), Post(), ...)"
                            .into(),
                    );
                }
            };
            if capacity.is_some() {
                return Err("a list has a capacity in elements, not bytes".into());
            }
            return FieldType::list(element, n).map_err(|e| e.reason().to_owned());
        }
        if let Some(n) = capacity {
            if self != Kind::Str {
                return Err("only a str field has a capacity".into());
            }
            return FieldType::string(n).map_err(|e| e.reason().to_owned());
        }
        Ok(match self {
            Kind::F32 => FieldType::F32,
            Kind::F64 => FieldType::F64,
            Kind::U32 => FieldType::U32,
            Kind::I32 => FieldType::I32,
            Kind::Vector2 => FieldType::Vec2,
            Kind::Vector3 | Kind::Color => FieldType::Vec3,
            Kind::Quaternion => FieldType::Quat,
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound => {
                FieldType::U64
            }
            Kind::Bool => FieldType::Bool,
            Kind::Matrix4 => FieldType::Mat4,
            Kind::Enum => FieldType::Enum,
            Kind::Ease => FieldType::Ease,
            Kind::Str if !viewed => FieldType::Str(256),
            Kind::Str => {
                return Err("a string field declares its capacity in bytes: str(32)".into());
            }
        })
    }

    /// Whether the kind is a 64-bit handle (an entity or an asset).
    pub fn is_handle(self) -> bool {
        matches!(
            self,
            Kind::Entity | Kind::Geometry | Kind::Material | Kind::Asset | Kind::Sound
        )
    }

    /// The kind a field of a registered og-core schema is read as (a list's
    /// is its element's).
    pub fn of(ty: FieldType) -> Kind {
        match ty {
            FieldType::List(element, _) => Kind::of(element.field_type()),
            FieldType::F32 => Kind::F32,
            FieldType::F64 => Kind::F64,
            FieldType::I32 => Kind::I32,
            FieldType::U32 => Kind::U32,
            FieldType::I64 | FieldType::U64 => Kind::Entity,
            FieldType::Vec2 => Kind::Vector2,
            FieldType::Vec3 => Kind::Vector3,
            FieldType::Vec4 | FieldType::Quat => Kind::Quaternion,
            FieldType::Bool => Kind::Bool,
            FieldType::Mat4 => Kind::Matrix4,
            FieldType::Enum => Kind::Enum,
            FieldType::Ease => Kind::Ease,
            FieldType::Str(_) => Kind::Str,
        }
    }
}

/// One field of a host layout.
#[derive(Clone, Debug)]
pub struct FieldLayout {
    pub name: String,
    pub kind: Kind,
    pub ty: FieldType,
    pub default: Value,
    pub readonly: bool,
    pub tunable: bool,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub step: Option<f64>,
    pub unit: Option<String>,
    pub label: Option<String>,
    pub doc: String,
    /// An enum field's variant names, in index order.
    pub variants: Vec<String>,
    /// Where its coordinates lie, for og.WorldOrigin's shift (R12): the
    /// IDL's `worldspace` attribute, or a game record's `worldspace` option.
    pub worldspace: Option<String>,
}

/// The IDL's `worldspace` attribute of an engine record's field, for a
/// layout read back from a module's derived schema.
pub(crate) fn idl_worldspace(schema: &str, field: &str) -> Option<String> {
    let decl = og_api::host::RECORDS.iter().find(|d| d.schema == schema)?;
    let f = decl.fields.iter().find(|f| f.name == field)?;
    (!f.worldspace.is_empty()).then(|| f.worldspace.to_string())
}

/// What a record is registered as.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RecordKind {
    Component,
    Resource,
    Event,
}

impl RecordKind {
    pub fn as_str(self) -> &'static str {
        match self {
            RecordKind::Component => "component",
            RecordKind::Resource => "resource",
            RecordKind::Event => "event",
        }
    }
}

/// A component, resource or event layout declared by name.
#[derive(Clone, Debug)]
pub struct Layout {
    pub schema: String,
    pub kind: RecordKind,
    /// Shared: a layout is cloned on every record call (`running_record`),
    /// and copying each field's strings was a tenth of og.world.get.
    pub fields: std::sync::Arc<Vec<FieldLayout>>,
    /// `(field, byte offset)` by field name, built on the first
    /// [`Layout::slot`] (the fields never change after construction).
    pub(crate) by_name: std::sync::Arc<std::sync::OnceLock<Vec<(usize, usize)>>>,
    /// The last bytes [`Layout::decode_kept`] decoded and their JSON,
    /// shared by the layout's clones.
    kept: std::sync::Arc<std::sync::Mutex<Kept>>,
    /// The rows [`crate::present::fx::kept_rows`] last read, their bytes and their
    /// JSON in the order it answered them, shared by the layout's clones.
    pub(crate) kept_rows: std::sync::Arc<std::sync::Mutex<KeptRows>>,
}

/// A record's packed bytes and their JSON ([`Layout::decode_kept`]).
type Kept = Option<(Vec<u8>, std::sync::Arc<Value>)>;

/// Rows' packed bytes and their JSON, in a read's order
/// ([`crate::present::fx::kept_rows`]).
pub(crate) type KeptRows = Vec<(Vec<u8>, std::sync::Arc<Value>)>;

/// The variants of an IDL enum column (`AudioBus`), from the declared
/// columns: a game's field may be of an enum column no engine record uses.
fn enum_variants(column: &str) -> Option<Vec<String>> {
    og_api::host::COLUMNS
        .iter()
        .find(|c| c.name == column && !c.variants.is_empty())
        .map(|c| c.variants.iter().map(|v| v.to_string()).collect())
}

fn number(v: &Value, what: &str) -> Result<f64, String> {
    v.as_f64()
        .ok_or_else(|| format!("{what} is a number, got {v}"))
}

#[cfg(test)]
mod tests;
