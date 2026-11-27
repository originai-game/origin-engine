//! Material shader validation at cook time (checked TS declarations):
//! the lit shader with a shader's hooks, a post pass and a UI material
//! compiled through naga for every back end, so a broken one is refused
//! with naga's message before it ships. The binary shader payload is
//! og-bundle's (`og_bundle::shader`), which never builds naga.

use super::{
    BlobKind, CookError, Cx, Exec, Job, KindDef, Out, PlanState, Planned, Source, format, stamp,
};
use crate::cook::source_schemas::media::shader as input;
use crate::format::Area;
use og_render_data::{MaterialShader, ParamKind, ShaderMode, ShaderParam};

fn params(input: Vec<input::Param>) -> Vec<ShaderParam> {
    use input::Param::*;
    input
        .into_iter()
        .map(|p| {
            let (name, kind, default) = match p {
                Float { name, default } => (
                    name,
                    ParamKind::Float,
                    [default.unwrap_or(0.0), 0.0, 0.0, 0.0],
                ),
                Vec2 { name, default } => {
                    let [x, y] = default.unwrap_or_default();
                    (name, ParamKind::Vec2, [x, y, 0.0, 0.0])
                }
                Vec3 { name, default } | Color { name, default } => {
                    let [x, y, z] = default.unwrap_or_default();
                    (name, ParamKind::Vec3, [x, y, z, 0.0])
                }
                Vec4 { name, default } => (name, ParamKind::Vec4, default.unwrap_or_default()),
            };
            ShaderParam {
                name,
                kind,
                default,
            }
        })
        .collect()
}

/// The same typed shader construction for cook and og check's capability facts.
pub fn material_shader(name: &str, input: input::Shader) -> Result<MaterialShader, String> {
    let shader = MaterialShader {
        name: name.into(),
        code: input.code,
        params: params(input.params),
        mode: match input.mode {
            None | Some(input::Mode::Hooks) => ShaderMode::Hooks,
            Some(input::Mode::Sprite) => ShaderMode::Sprite,
            Some(input::Mode::Full { lit }) => ShaderMode::Full { lit },
        },
    };
    shader.check()?;
    Ok(shader)
}

/// Compiles a post pass for every back end (naga: WGSL, SPIR-V, MSL, HLSL). The
/// engine also compiles it on the device and skips it, reported, when that
/// fails.
pub fn validate_post(post: &og_render_data::PostShader) -> Result<Vec<String>, String> {
    use naga::back::hlsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let src = og_render_data::post_module_source(post);
    let first = src
        .find(post.code.as_str())
        .map_or(0, |at| src[..at].lines().count());
    let lines = post.code.lines().count();
    let module = naga::front::wgsl::parse_str(&src).map_err(|e| {
        format!(
            "WGSL: {}",
            user_lines(&e.emit_to_string(&src), first, lines)
        )
    })?;
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|e| {
            format!(
                "validation: {}",
                user_lines(&e.emit_to_string(&src), first, lines)
            )
        })?;
    naga::back::spv::write_vec(&module, &info, &Default::default(), None)
        .map_err(|e| format!("SPIR-V: {e}"))?;
    let msl = naga::back::msl::Options {
        lang_version: (2, 4),
        ..Default::default()
    };
    naga::back::msl::write_string(&module, &info, &msl, &Default::default())
        .map_err(|e| format!("MSL: {e}"))?;
    let hlsl_opts = hlsl::Options {
        shader_model: hlsl::ShaderModel::V6_0,
        ..Default::default()
    };
    let mut out = String::new();
    hlsl::Writer::new(&mut out, &hlsl_opts, &hlsl::PipelineOptions::default())
        .write(&module, &info, None)
        .map_err(|e| format!("HLSL: {e}"))?;
    Ok(vec!["SPIR-V, MSL, HLSL".to_string()])
}

/// Compiles a UI material's composite module for every back end the
/// engine draws with (naga: WGSL validation, SPIR-V for Vulkan, MSL for
/// Metal, HLSL for DX12). The engine also compiles it on the device and
/// composites the plain layer, reported, when that fails.
pub fn validate_ui_material(
    material: &og_render_data::UiMaterialShader,
) -> Result<Vec<String>, String> {
    use naga::back::hlsl;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let src = og_render_data::ui_material_module_source(material);
    let first = src
        .find(material.code.as_str())
        .map_or(0, |at| src[..at].lines().count());
    let lines = material.code.lines().count();
    let module = naga::front::wgsl::parse_str(&src).map_err(|e| {
        format!(
            "WGSL: {}",
            user_lines(&e.emit_to_string(&src), first, lines)
        )
    })?;
    let info = Validator::new(ValidationFlags::all(), Capabilities::all())
        .validate(&module)
        .map_err(|e| {
            format!(
                "validation: {}",
                user_lines(&e.emit_to_string(&src), first, lines)
            )
        })?;
    naga::back::spv::write_vec(&module, &info, &Default::default(), None)
        .map_err(|e| format!("SPIR-V: {e}"))?;
    // The instance index the UI pass reads needs MSL 2.4 (what wgpu asks of Metal).
    let msl = naga::back::msl::Options {
        lang_version: (2, 4),
        ..Default::default()
    };
    naga::back::msl::write_string(&module, &info, &msl, &Default::default())
        .map_err(|e| format!("MSL: {e}"))?;
    let options = hlsl::Options {
        shader_model: hlsl::ShaderModel::V6_0,
        ..Default::default()
    };
    let mut out = String::new();
    hlsl::Writer::new(&mut out, &options, &hlsl::PipelineOptions::default())
        .write(&module, &info, None)
        .map_err(|e| format!("HLSL: {e}"))?;
    Ok(vec!["SPIR-V, MSL, HLSL".to_string()])
}

/// Compiles the lit shader with `shader`'s hooks for every back end the
/// engine draws with. Returns what was compiled, or naga's message for the
/// first that failed.
pub fn validate_shader(shader: &MaterialShader) -> Result<Vec<String>, String> {
    use naga::back::hlsl;
    use naga::back::pipeline_constants::process_overrides;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    // A full shader's casters run its code too (custom.rs).
    type Source = fn(&MaterialShader) -> String;
    // A sprite shader runs only in the sprite module. A hooks shader is
    // checked as a mesh draws it: on a sprite, one calling a lit-only
    // helper draws plain (og-render sprite_hooks.rs).
    let mut modules: Vec<Source> = match shader.mode {
        ShaderMode::Sprite => vec![og_render_data::sprite_module_source],
        _ => vec![og_render_data::lit_module_source],
    };
    if matches!(shader.mode, ShaderMode::Full { .. }) {
        modules.push(og_render_data::shadow_module_source);
    }
    let hlsl_opts = hlsl::Options {
        shader_model: hlsl::ShaderModel::V6_0,
        ..Default::default()
    };
    for source in modules {
        let src = source(shader);
        let first = src
            .find(shader.code.as_str())
            .map_or(0, |at| src[..at].lines().count());
        let lines = shader.code.lines().count();
        let module = naga::front::wgsl::parse_str(&src).map_err(|e| {
            format!(
                "WGSL: {}",
                user_lines(&e.emit_to_string(&src), first, lines)
            )
        })?;
        let info = Validator::new(ValidationFlags::all(), Capabilities::all())
            .validate(&module)
            .map_err(|e| {
                format!(
                    "validation: {}",
                    user_lines(&e.emit_to_string(&src), first, lines)
                )
            })?;
        for effects in [1.0, 0.0] {
            let mut constants = naga::back::PipelineConstants::default();
            // The lit modules' (a shadow module has none).
            if module
                .overrides
                .iter()
                .any(|(_, o)| o.name.as_deref() == Some("EFFECTS"))
            {
                constants.insert("EFFECTS".to_string(), effects);
            }
            let (module, info) = process_overrides(&module, &info, None, &constants)
                .map_err(|e| format!("overrides: {e}"))?;
            naga::back::spv::write_vec(&module, &info, &Default::default(), None)
                .map_err(|e| format!("SPIR-V: {e}"))?;
            // MSL 2.4, what wgpu asks of Metal: the order-independent
            // transparency's entry points write the arena (group 0 binding
            // 12) from the fragment stage, which MSL before 1.2 refuses.
            let msl = naga::back::msl::Options {
                lang_version: (2, 4),
                ..Default::default()
            };
            naga::back::msl::write_string(&module, &info, &msl, &Default::default())
                .map_err(|e| format!("MSL: {e}"))?;
            let mut out = String::new();
            hlsl::Writer::new(&mut out, &hlsl_opts, &hlsl::PipelineOptions::default())
                .write(&module, &info, None)
                .map_err(|e| format!("HLSL: {e}"))?;
        }
    }
    Ok(vec!["SPIR-V, MSL, HLSL".to_string()])
}

/// naga's report with `wgsl:<line>` locations inside the hooks (lines
/// `first + 1 ..= first + lines` of the composed shader) given as lines of
/// the hooks code, `hooks:<line>`.
fn user_lines(report: &str, first: usize, lines: usize) -> String {
    let mut out = String::with_capacity(report.len());
    let mut rest = report;
    while let Some(at) = rest.find("wgsl:") {
        out.push_str(&rest[..at]);
        let after = &rest[at + 5..];
        let digits = after.chars().take_while(char::is_ascii_digit).count();
        match after[..digits].parse::<usize>() {
            Ok(line) if line > first && line <= first + lines => {
                out.push_str(&format!("hooks:{}", line - first));
            }
            _ => out.push_str(&rest[at..at + 5 + digits]),
        }
        rest = &after[digits..];
    }
    out.push_str(rest);
    out
}

pub(crate) static SHADER: KindDef = KindDef {
    name: "shader",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_shader,
};

fn plan_shader<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Shader], &[b"typed shader", &hash]);
    outs.push((
        s.rel.clone(),
        format::Shader::NAME,
        st,
        Box::new(ShaderJob(s)),
    ));
}

/// A shader's one output: its checked WGSL and typed uniforms.
struct ShaderJob<'a>(&'a Source);

impl Job for ShaderJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let ShaderJob(s) = self;
        let shader = material_shader(&s.rel, s.input().map_err(CookError)?).map_err(CookError)?;
        let compiled = validate_shader(&shader)
            .map_err(|e| CookError(format!("the shader does not compile: {e}")))?;
        let mut w = og_blob::BlobWriter::new(
            format::Shader::type_hash(),
            format::Shader::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::ShaderMeta {
                code_bytes: shader.code.len() as u32,
                mode: match shader.mode {
                    ShaderMode::Hooks => 0,
                    ShaderMode::Full { lit: true } => 1,
                    ShaderMode::Full { lit: false } => 2,
                    ShaderMode::Sprite => 3,
                },
            }],
            format::row_hash::<format::ShaderMeta>(),
        );
        crate::shader::write(&mut w, &shader.code, &shader.params);
        Ok((
            w.finish(),
            vec![format!(
                "{} parameter(s); compiled for {}",
                shader.params.len(),
                compiled.join(", ")
            )],
        ))
    }
}

pub(crate) static POST: KindDef = KindDef {
    name: "post",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_post,
};

fn plan_post<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Shader], &[b"typed post", &hash]);
    outs.push((s.rel.clone(), format::Post::NAME, st, Box::new(PostJob(s))));
}

/// A post pass's one output: its checked WGSL and typed uniforms.
struct PostJob<'a>(&'a Source);

impl Job for PostJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let PostJob(s) = self;
        let input: input::Post = s.input().map_err(CookError)?;
        let post = og_render_data::PostShader {
            name: s.rel.clone(),
            code: input.code,
            params: params(input.params),
        };
        post.check().map_err(CookError)?;
        let compiled = validate_post(&post)
            .map_err(|e| CookError(format!("the post pass does not compile: {e}")))?;
        let mut w = og_blob::BlobWriter::new(
            format::Post::type_hash(),
            format::Post::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::PostMeta {
                code_bytes: post.code.len() as u32,
            }],
            format::row_hash::<format::PostMeta>(),
        );
        crate::shader::write(&mut w, &post.code, &post.params);
        Ok((
            w.finish(),
            vec![format!(
                "{} parameter(s); compiled for {}",
                post.params.len(),
                compiled.join(", ")
            )],
        ))
    }
}

pub(crate) static UI_MATERIAL: KindDef = KindDef {
    name: "ui-material",
    extensions: &[],
    types: &[],
    by_content: true,
    streams: false,
    plan: plan_ui_material,
};

fn plan_ui_material<'a>(
    _cx: &Cx<'a>,
    _state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    let st = stamp(&[Area::Shader], &[b"typed ui-material", &hash]);
    outs.push((
        s.rel.clone(),
        format::UiMaterial::NAME,
        st,
        Box::new(UiMaterialJob(s)),
    ));
}

/// A UI material's one output: its checked WGSL and typed uniforms.
struct UiMaterialJob<'a>(&'a Source);

impl Job for UiMaterialJob<'_> {
    fn cook(&self, p: &Planned, _: &Exec) -> Result<(Vec<u8>, Vec<String>), CookError> {
        let UiMaterialJob(s) = self;
        let input: input::UiMaterial = s.input().map_err(CookError)?;
        let material = og_render_data::UiMaterialShader {
            name: s.rel.clone(),
            code: input.code,
            params: params(input.params),
        };
        material.check().map_err(CookError)?;
        let compiled = validate_ui_material(&material)
            .map_err(|e| CookError(format!("the UI material does not compile: {e}")))?;
        let mut w = og_blob::BlobWriter::new(
            format::UiMaterial::type_hash(),
            format::UiMaterial::schema_version(),
            p.stamp,
        );
        w.table(
            "meta",
            &[format::UiMaterialMeta {
                code_bytes: material.code.len() as u32,
            }],
            format::row_hash::<format::UiMaterialMeta>(),
        );
        crate::shader::write(&mut w, &material.code, &material.params);
        Ok((
            w.finish(),
            vec![format!(
                "{} parameter(s); compiled for {}",
                material.params.len(),
                compiled.join(", ")
            )],
        ))
    }
}

/// The kinds this module cooks.
pub(crate) static KINDS: &[&KindDef] = &[&SHADER, &POST, &UI_MATERIAL];
