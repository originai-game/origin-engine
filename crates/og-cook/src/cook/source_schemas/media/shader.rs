//! WGSL remains a string. Only its parameter and pipeline contracts are data.
use super::*;

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Param {
    Float {
        name: String,
        default: Option<f32>,
    },
    Vec2 {
        name: String,
        default: Option<[f32; 2]>,
    },
    Vec3 {
        name: String,
        default: Option<[f32; 3]>,
    },
    Vec4 {
        name: String,
        default: Option<[f32; 4]>,
    },
    /// Linear RGB; not a display-space hex colour.
    Color {
        name: String,
        default: Option<[f32; 3]>,
    },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(tag = "kind", rename_all = "kebab-case", deny_unknown_fields)]
pub enum Mode {
    Hooks,
    Sprite,
    Full { lit: bool },
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Shader {
    /// WGSL hooks or vertex/fragment entries; use input.text for external WGSL.
    pub code: String,
    pub mode: Option<Mode>,
    /// Names, reserved identifiers and backend limits are checked by the domain.
    #[serde(default)]
    #[schemars(length(max = og_render_data::MAX_SHADER_PARAMS))]
    pub params: Vec<Param>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct Post {
    /// WGSL defining fn post(f: PostInput, p: Params) -> vec4<f32>.
    pub code: String,
    #[serde(default)]
    #[schemars(length(max = og_render_data::MAX_SHADER_PARAMS))]
    pub params: Vec<Param>,
}

#[derive(Debug, Deserialize, JsonSchema)]
#[schemars(crate = "og_core::reflect::source::schemars")]
#[serde(deny_unknown_fields)]
pub struct UiMaterial {
    /// WGSL defining fn material(m: MaterialInput, p: Params) -> vec4<f32>.
    pub code: String,
    #[serde(default)]
    #[schemars(length(max = og_draw::ui::UI_MATERIAL_PARAMS))]
    pub params: Vec<Param>,
}

schema!(Shader, Post, UiMaterial);
