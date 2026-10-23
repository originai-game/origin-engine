//! Material shaders as text (`.ogt` of `type shader`, the E5 round): the
//! hooks of og-render's lit shader (`og_render::MaterialShader`) and the
//! uniforms they read. The cook reads the WGSL (inline or from the file
//! `source` names), compiles the lit shader with it through naga for every
//! tier and back end (WGSL validation, SPIR-V for Vulkan, MSL for Metal,
//! GLSL ES 3.0 for WebGL2, each with effects on and off) and refuses the
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
//! casters' module is compiled for every tier too. `mode hooks` (the
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

/// Compiles a post pass for every tier and back end (naga: WGSL, SPIR-V,
/// MSL, GLSL ES 3.0). The engine also compiles it on the device and skips
/// it, reported, when that fails.
#[cfg(feature = "cook")]
pub fn validate_post(post: &og_render::PostShader) -> Result<Vec<String>, String> {
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let src = og_render::post_module_source(post);
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
    let mut done = Vec::new();
    for full in [true, false] {
        let caps = if full {
            Capabilities::all()
        } else {
            Capabilities::empty()
        };
        let info = Validator::new(ValidationFlags::all(), caps)
            .validate(&module)
            .map_err(|e| {
                format!(
                    "validation: {}",
                    user_lines(&e.emit_to_string(&src), first, lines)
                )
            })?;
        if full {
            naga::back::spv::write_vec(&module, &info, &Default::default(), None)
                .map_err(|e| format!("SPIR-V: {e}"))?;
            naga::back::msl::write_string(&module, &info, &Default::default(), &Default::default())
                .map_err(|e| format!("MSL: {e}"))?;
            done.push("full tier: SPIR-V, MSL".to_string());
        } else {
            for (stage, entry) in [
                (naga::ShaderStage::Vertex, "vs"),
                (naga::ShaderStage::Fragment, "fs_custom_post"),
            ] {
                let options = naga::back::glsl::Options {
                    version: naga::back::glsl::Version::Embedded {
                        version: 300,
                        is_webgl: true,
                    },
                    ..Default::default()
                };
                let pipeline = naga::back::glsl::PipelineOptions {
                    shader_stage: stage,
                    entry_point: entry.to_string(),
                    multiview: None,
                };
                let mut out = String::new();
                naga::back::glsl::Writer::new(
                    &mut out,
                    &module,
                    &info,
                    &options,
                    &pipeline,
                    naga::proc::BoundsCheckPolicies::default(),
                )
                .and_then(|mut w| w.write().map(|_| ()))
                .map_err(|e| format!("GLSL ES 3.0 ({entry}): {e}"))?;
            }
            done.push("WebGL2: GLSL ES 3.0".to_string());
        }
    }
    Ok(done)
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

/// Compiles the lit shader with `shader`'s hooks for every tier and back
/// end the engine draws with. Returns what was compiled, or naga's
/// message for the first that failed.
#[cfg(feature = "cook")]
pub fn validate_shader(shader: &MaterialShader) -> Result<Vec<String>, String> {
    use naga::back::pipeline_constants::process_overrides;
    use naga::valid::{Capabilities, ValidationFlags, Validator};
    let mut done = Vec::new();
    // A full shader's casters run its code too (custom.rs).
    type Source = fn(&MaterialShader, bool) -> String;
    let mut modules: Vec<(&str, Source, [&str; 2])> =
        vec![("lit", og_render::lit_module_source, ["vs", "fs"])];
    if shader.mode != ShaderMode::Hooks {
        modules.push((
            "shadow",
            og_render::shadow_module_source,
            ["vs_hooked", "fs_hooked"],
        ));
    }
    for (tier, full) in [("full", true), ("webgl2", false)] {
        for &(_label, source, entries) in &modules {
            let src = source(shader, full);
            let first = src
                .find(shader.code.as_str())
                .map_or(0, |at| src[..at].lines().count());
            let lines = shader.code.lines().count();
            let module = naga::front::wgsl::parse_str(&src).map_err(|e| {
                format!(
                    "WGSL ({tier}): {}",
                    user_lines(&e.emit_to_string(&src), first, lines)
                )
            })?;
            let caps = if full {
                Capabilities::all()
            } else {
                Capabilities::empty()
            };
            let info = Validator::new(ValidationFlags::all(), caps)
                .validate(&module)
                .map_err(|e| {
                    format!(
                        "validation ({tier}): {}",
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
                    .map_err(|e| format!("overrides ({tier}): {e}"))?;
                if full {
                    naga::back::spv::write_vec(&module, &info, &Default::default(), None)
                        .map_err(|e| format!("SPIR-V: {e}"))?;
                    naga::back::msl::write_string(
                        &module,
                        &info,
                        &Default::default(),
                        &Default::default(),
                    )
                    .map_err(|e| format!("MSL: {e}"))?;
                } else {
                    for (stage, entry) in [
                        (naga::ShaderStage::Vertex, entries[0]),
                        (naga::ShaderStage::Fragment, entries[1]),
                    ] {
                        let options = naga::back::glsl::Options {
                            version: naga::back::glsl::Version::Embedded {
                                version: 300,
                                is_webgl: true,
                            },
                            ..Default::default()
                        };
                        let pipeline = naga::back::glsl::PipelineOptions {
                            shader_stage: stage,
                            entry_point: entry.to_string(),
                            multiview: None,
                        };
                        let mut out = String::new();
                        let mut w = naga::back::glsl::Writer::new(
                            &mut out,
                            &module,
                            &info,
                            &options,
                            &pipeline,
                            naga::proc::BoundsCheckPolicies::default(),
                        )
                        .map_err(|e| format!("GLSL ES 3.0 ({entry}): {e}"))?;
                        w.write()
                            .map_err(|e| format!("GLSL ES 3.0 ({entry}): {e}"))?;
                    }
                }
            }
        }
        done.push(if full {
            "full tier: SPIR-V, MSL".to_string()
        } else {
            "WebGL2: GLSL ES 3.0".to_string()
        });
    }
    Ok(done)
}

/// naga's report with `wgsl:<line>` locations inside the hooks (lines
/// `first + 1 ..= first + lines` of the composed shader) given as lines of
/// the hooks code, `hooks:<line>`.
#[cfg(feature = "cook")]
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

#[cfg(all(test, feature = "cook"))]
mod tests {
    use super::*;

    const CODE: &str = "fn fragment(f: FragmentInput, p: Params) -> SurfaceOutput {
  var s = standard_surface(f);
  s.color = vec4<f32>(p.tint * tex0(f.uv).rgb, step(p.cut, noise3(f.local * 4.0)));
  return s;
}";

    fn full(code: &str) -> String {
        format!(
            "type shader\nversion 1\n---\nmode full unlit\ncode \"{}\"\nparam tint color 1 0.5 0.2\nparam cut float 0.3\n",
            code.replace('\n', "\\n")
        )
    }

    fn refuse(_: &str) -> Result<String, String> {
        Err("no files".into())
    }

    /// A full shader parses with its mode, validates for every tier (its
    /// lit and its shadow casters' modules) and keeps its mode through the
    /// canonical text the blob holds.
    #[test]
    fn a_full_shader_cooks_for_every_tier_and_keeps_its_mode() {
        let s = parse_shader("crt", &full(CODE), &mut refuse).expect("parses");
        assert_eq!(s.mode, ShaderMode::Full { lit: false });
        let done = validate_shader(&s).expect("validates");
        assert_eq!(done.len(), 2, "{done:?}");
        let again = parse_shader("crt", &shader_text(&s), &mut refuse).expect("round trip");
        assert_eq!(again, s);
    }

    /// A post pass cooks for every tier and round-trips; a broken one is
    /// refused at its line.
    #[test]
    fn a_post_pass_cooks_for_every_tier() {
        let text = |code: &str| {
            format!(
                "type post\nversion 1\n---\ncode \"{}\"\nparam curve float 0.1\n",
                code.replace('\n', "\\n")
            )
        };
        let code = "fn post(f: PostInput, p: Params) -> vec4<f32> {\n  let d = depth(f.uv) + normal(f.uv).x;\n  return previous(f.uv * (1.0 + p.curve)) + scene_color(f.uv) * 0.0 * d;\n}";
        let post = parse_post("crt", &text(code), &mut refuse).expect("parses");
        assert_eq!(validate_post(&post).expect("validates").len(), 2);
        assert_eq!(
            parse_post("crt", &post_text(&post), &mut refuse).expect("again"),
            post
        );
        let broken = parse_post(
            "crt",
            &text(&code.replace("p.curve", "p.bend")),
            &mut refuse,
        )
        .expect("parses");
        let e = validate_post(&broken).expect_err("refused");
        assert!(e.contains("hooks:3"), "{e}");
        let e = parse_post("crt", &text("fn other() {}"), &mut refuse).expect_err("no post");
        assert!(e.contains("post"), "{e}");
    }

    /// A full shader's error is reported at its own line, and one with
    /// neither entry is refused.
    #[test]
    fn a_broken_full_shader_is_refused_at_its_line() {
        let broken = full(&CODE.replace("noise3(f.local * 4.0)", "noise3(f.nowhere)"));
        let s = parse_shader("crt", &broken, &mut refuse).expect("parses");
        let e = validate_shader(&s).expect_err("refused");
        assert!(e.contains("hooks:3"), "{e}");
        let none = full(&CODE.replace("fn fragment(", "fn other("));
        let e = parse_shader("crt", &none, &mut refuse).expect_err("no entry");
        assert!(e.contains("neither"), "{e}");
    }
}
