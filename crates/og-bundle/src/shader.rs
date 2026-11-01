//! Material shaders as text (`.ogt` of `type shader`, the E5 round): the
//! hooks of og-render's lit shader (`og_render::MaterialShader`) and the
//! uniforms they read. The cook reads the WGSL (inline or from the file
//! `source` names), compiles the lit shader with it through naga for every
//! back end (WGSL validation, SPIR-V for Vulkan, MSL for Metal, each with
//! effects on and off) and refuses the
//! asset with naga's message when any of them fails. The blob keeps the
//! canonical file with the code inlined, which the runtime parses again.
//!
//! ```text
//! type shader
//! version 1
//! ---
//! source "shaders/dissolve.wgsl"   # the hooks, an asset path (or: code "<wgsl>")
//! param amount float 0             # <name> float|vec2|vec3|vec4|color <default...>
//! param edgeColor color 4 1.6 0.4  # color: a vec3 (linear rgb)
//! ```
//!
//! `mode full [lit|unlit]` (the S8 depth stream) makes it a full shader:
//! the code defines `vertex` and `fragment` entries over og-render's
//! binding contract (`custom.rs`) instead of the hooks, lit by the frame's
//! lights (`lit`, the default) or drawn as returned (`unlit`); its shadow
//! casters' module is compiled too. `mode hooks` (the
//! default) is the hooks.

use og_core::ogt::{Document, Entry, Number, Value};
use og_render::{MaterialShader, ParamKind, ShaderMode, ShaderParam};

/// The `.ogt` type of a shader file.
pub const SHADER_TYPE: &str = "shader";

/// The `.ogt` type of a custom post pass (og-render `post_custom.rs`): the
/// same `code`/`source` and `param` lines as a shader, the code defining
/// `fn post(f: PostInput, p: Params) -> vec4<f32>`.
pub const POST_TYPE: &str = "post";

/// Parses a post pass file (`read` as for [`parse_shader`]).
pub fn parse_post(
    name: &str,
    text: &str,
    read: &mut dyn FnMut(&str) -> Result<String, String>,
) -> Result<og_render::PostShader, String> {
    let (code, params, _) = parse_entries(text, read, POST_TYPE)?;
    let post = og_render::PostShader {
        name: name.to_string(),
        code: code.ok_or("a post pass needs `code` or `source` (its WGSL `post` function)")?,
        params,
    };
    post.check()?;
    Ok(post)
}

/// The canonical file of a post pass, its code inlined.
pub fn post_text(post: &og_render::PostShader) -> String {
    entries_text(POST_TYPE, &post.code, &post.params, ShaderMode::Hooks)
}

/// The `.ogt` type of a custom UI material (`docs/ui.md` §5.4, og-render's
/// `UiMaterialShader`): the same `code`/`source` and `param` lines (at most
/// four), the code defining
/// `fn material(m: MaterialInput, p: Params) -> vec4<f32>`, the colour a
/// UI layer composites with.
///
/// ```text
/// type ui-material
/// version 1
/// ---
/// source "ui/duotone.wgsl"
/// param dark color 0.05 0.02 0.1
/// param light color 1 0.8 0.4
/// ```
pub const UI_MATERIAL_TYPE: &str = "ui-material";

/// Parses a UI material file (`read` as for [`parse_shader`]).
pub fn parse_ui_material(
    name: &str,
    text: &str,
    read: &mut dyn FnMut(&str) -> Result<String, String>,
) -> Result<og_render::UiMaterialShader, String> {
    let (code, params, _) = parse_entries(text, read, UI_MATERIAL_TYPE)?;
    let material = og_render::UiMaterialShader {
        name: name.to_string(),
        code: code
            .ok_or("a UI material needs `code` or `source` (its WGSL `material` function)")?,
        params,
    };
    material.check()?;
    Ok(material)
}

/// The canonical file of a UI material, its code inlined.
pub fn ui_material_text(material: &og_render::UiMaterialShader) -> String {
    entries_text(
        UI_MATERIAL_TYPE,
        &material.code,
        &material.params,
        ShaderMode::Hooks,
    )
}

/// The WGSL file a shader names with `source` (the cook reads it).
pub fn shader_source(text: &str) -> Option<String> {
    Document::parse(text).ok().and_then(|d| {
        d.body
            .get("source")
            .and_then(|e| e.values.first())
            .and_then(Value::as_str)
            .map(str::to_string)
    })
}

/// Parses a shader file; `read` returns the text of the file `source`
/// names (the runtime passes one that refuses: cooked files carry `code`).
pub fn parse_shader(
    name: &str,
    text: &str,
    read: &mut dyn FnMut(&str) -> Result<String, String>,
) -> Result<MaterialShader, String> {
    let (code, params, mode) = parse_entries(text, read, SHADER_TYPE)?;
    let shader = MaterialShader {
        name: name.to_string(),
        code: code.ok_or("a shader needs `code` or `source` (the WGSL hooks or full entries)")?,
        params,
        mode,
    };
    shader.check()?;
    Ok(shader)
}

type Entries = (Option<String>, Vec<ShaderParam>, ShaderMode);

/// The code, parameters and mode of a shader or post file of type `ty`.
fn parse_entries(
    text: &str,
    read: &mut dyn FnMut(&str) -> Result<String, String>,
    ty: &str,
) -> Result<Entries, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != ty {
        return Err(format!("a {ty} is `type {ty}`, not `type {}`", doc.ty));
    }
    if doc.version != 1 {
        return Err(format!(
            "{ty} version {} (this engine reads 1)",
            doc.version
        ));
    }
    let mut code = None;
    let mut params = Vec::new();
    let mut mode = ShaderMode::Hooks;
    for e in &doc.body.entries {
        let line = e.line;
        match e.key.as_str() {
            "mode" if ty == SHADER_TYPE => {
                let word = |i: usize| e.values.get(i).and_then(Value::as_word);
                mode = match (word(0), word(1), e.values.len()) {
                    (Some("hooks"), None, 1) => ShaderMode::Hooks,
                    (Some("full"), None, 1) | (Some("full"), Some("lit"), 2) => {
                        ShaderMode::Full { lit: true }
                    }
                    (Some("full"), Some("unlit"), 2) => ShaderMode::Full { lit: false },
                    _ => {
                        return Err(format!(
                            "line {line}: `mode hooks` or `mode full [lit|unlit]`"
                        ));
                    }
                };
            }
            "code" | "source" => {
                if code.is_some() {
                    return Err(format!("line {line}: one `code` or `source` per shader"));
                }
                let s = e
                    .values
                    .first()
                    .and_then(Value::as_str)
                    .ok_or_else(|| format!("line {line}: `{}` takes a string", e.key))?;
                code = Some(if e.key == "code" {
                    s.to_string()
                } else {
                    read(s).map_err(|m| format!("line {line}: source \"{s}\": {m}"))?
                });
            }
            "param" => {
                let word = |i: usize| e.values.get(i).and_then(Value::as_word);
                let (Some(pname), Some(kind)) = (word(0), word(1)) else {
                    return Err(format!(
                        "line {line}: `param <name> float|vec2|vec3|vec4|color <default...>`"
                    ));
                };
                let kind = match kind {
                    "float" => ParamKind::Float,
                    "vec2" => ParamKind::Vec2,
                    "vec3" | "color" => ParamKind::Vec3,
                    "vec4" => ParamKind::Vec4,
                    other => {
                        return Err(format!(
                            "line {line}: parameter type `{other}` (float, vec2, vec3, vec4 or color)"
                        ));
                    }
                };
                let nums: Vec<f32> = e.values[2..]
                    .iter()
                    .map(|v| v.as_number().map(Number::as_f32).filter(|f| f.is_finite()))
                    .collect::<Option<_>>()
                    .ok_or_else(|| format!("line {line}: a parameter's default is numbers"))?;
                if !(nums.is_empty() || nums.len() == kind.count()) {
                    return Err(format!(
                        "line {line}: `{pname}` takes {} number(s) as its default",
                        kind.count()
                    ));
                }
                let mut default = [0.0; 4];
                default[..nums.len()].copy_from_slice(&nums);
                params.push(ShaderParam {
                    name: pname.to_string(),
                    kind,
                    default,
                });
            }
            other => return Err(format!("line {line}: unknown {ty} entry `{other}`")),
        }
    }
    Ok((code, params, mode))
}

/// The canonical file of `shader`, its code inlined (what a cooked blob holds).
pub fn shader_text(shader: &MaterialShader) -> String {
    entries_text(SHADER_TYPE, &shader.code, &shader.params, shader.mode)
}

fn entries_text(ty: &str, code: &str, params: &[ShaderParam], mode: ShaderMode) -> String {
    let mut doc = Document::new(ty, 1);
    if let ShaderMode::Full { lit } = mode {
        doc.body.entries.push(
            Entry::new("mode")
                .with(Value::Word("full".into()))
                .with(Value::Word(if lit { "lit" } else { "unlit" }.into())),
        );
    }
    doc.body
        .entries
        .push(Entry::new("code").with(Value::String(code.to_string())));
    for p in params {
        let mut e = Entry::new("param")
            .with(Value::Word(p.name.clone()))
            .with(Value::Word(
                match p.kind {
                    ParamKind::Float => "float",
                    ParamKind::Vec2 => "vec2",
                    ParamKind::Vec3 => "vec3",
                    ParamKind::Vec4 => "vec4",
                }
                .into(),
            ));
        for v in &p.default[..p.kind.count()] {
            e = e.with(Value::Number(
                Number::parse(&format!("{v:?}")).expect("a finite float prints as a number"),
            ));
        }
        doc.body.entries.push(e);
    }
    doc.to_text().expect("a shader document is well formed")
}
