//! Material shader validation at cook time (`crate::shader`'s files):
//! the lit shader with a shader's hooks, a post pass and a UI material
//! compiled through naga for every back end, so a broken one is refused
//! with naga's message before it ships. The parsing and the cooked text
//! are og-bundle's (`og_bundle::shader`), which never builds naga.

use og_render_data::{MaterialShader, ShaderMode};

/// Compiles a post pass for every back end (naga: WGSL, SPIR-V, MSL). The
/// engine also compiles it on the device and skips it, reported, when that
/// fails.
pub fn validate_post(post: &og_render_data::PostShader) -> Result<Vec<String>, String> {
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
    naga::back::msl::write_string(&module, &info, &Default::default(), &Default::default())
        .map_err(|e| format!("MSL: {e}"))?;
    Ok(vec!["SPIR-V, MSL".to_string()])
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
        }
    }
    Ok(vec!["SPIR-V, MSL".to_string()])
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::shader::{
        parse_post, parse_shader, parse_ui_material, post_text, shader_text, ui_material_text,
    };

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

    /// A full shader parses with its mode, validates for every back end (its
    /// lit and its shadow casters' modules) and keeps its mode through the
    /// canonical text the blob holds.
    #[test]
    fn a_full_shader_cooks_for_every_back_end_and_keeps_its_mode() {
        let s = parse_shader("crt", &full(CODE), &mut refuse).expect("parses");
        assert_eq!(s.mode, ShaderMode::Full { lit: false });
        let done = validate_shader(&s).expect("validates");
        assert_eq!(done, ["SPIR-V, MSL"]);
        let again = parse_shader("crt", &shader_text(&s), &mut refuse).expect("round trip");
        assert_eq!(again, s);
    }

    /// A sprite shader validates against the sprite module, where
    /// `sprite_texel` exists, keeps its mode through the canonical text, and
    /// is refused at its own line when it calls what the sprite module
    /// lacks; the same code as a hooks shader is refused by the lit module.
    #[test]
    fn a_sprite_shader_cooks_against_the_sprite_module() {
        let text = |mode: &str, code: &str| {
            format!(
                "type shader\nversion 1\n---\nmode {mode}\ncode \"{}\"\nparam amount float 0.5\n",
                code.replace('\n', "\\n")
            )
        };
        let code = "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {\n  return mix(c, sprite_texel(h.uv + vec2<f32>(p.amount * 0.01, 0.0)), 0.5);\n}";
        let s = parse_shader("shimmer", &text("sprite", code), &mut refuse).expect("parses");
        assert_eq!(s.mode, ShaderMode::Sprite);
        validate_shader(&s).expect("validates");
        let again = parse_shader("shimmer", &shader_text(&s), &mut refuse).expect("round trip");
        assert_eq!(again, s);
        let hooks = parse_shader("shimmer", &text("hooks", code), &mut refuse).expect("parses");
        let e = validate_shader(&hooks).expect_err("no sprite_texel on a mesh");
        assert!(e.contains("sprite_texel"), "{e}");
        let broken = parse_shader(
            "shimmer",
            &text("sprite", &code.replace("sprite_texel", "mesh_texel")),
            &mut refuse,
        )
        .expect("parses");
        let e = validate_shader(&broken).expect_err("refused");
        assert!(e.contains("hooks:2"), "{e}");
    }

    /// A post pass cooks for every back end and round-trips; a broken one
    /// is refused at its line.
    #[test]
    fn a_post_pass_cooks_for_every_back_end() {
        let text = |code: &str| {
            format!(
                "type post\nversion 1\n---\ncode \"{}\"\nparam curve float 0.1\n",
                code.replace('\n', "\\n")
            )
        };
        let code = "fn post(f: PostInput, p: Params) -> vec4<f32> {\n  let d = depth(f.uv) + normal(f.uv).x;\n  return previous(f.uv * (1.0 + p.curve)) + scene_color(f.uv) * 0.0 * d;\n}";
        let post = parse_post("crt", &text(code), &mut refuse).expect("parses");
        assert_eq!(validate_post(&post).expect("validates"), ["SPIR-V, MSL"]);
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

    /// A UI material cooks for Metal, Vulkan and DX12 and round-trips; a
    /// broken one is refused at its own line, one without `material` or
    /// with a fifth parameter before it compiles.
    #[test]
    fn a_ui_material_cooks_for_every_back_end_and_is_refused_at_its_line() {
        let text = |code: &str, params: &str| {
            format!(
                "type ui-material\nversion 1\n---\ncode \"{}\"\n{params}",
                code.replace('\n', "\\n")
            )
        };
        let code = "fn material(m: MaterialInput, p: Params) -> vec4<f32> {\n  let c = layer(m.uv);\n  return vec4<f32>(mix(p.dark, p.light, c.r), c.a);\n}";
        let params = "param dark color 0 0 0.1\nparam light color 1 0.8 0.4\n";
        let m = parse_ui_material("duo", &text(code, params), &mut refuse).expect("parses");
        assert_eq!(
            validate_ui_material(&m).expect("validates"),
            ["SPIR-V, MSL, HLSL"]
        );
        assert_eq!(
            parse_ui_material("duo", &ui_material_text(&m), &mut refuse).expect("again"),
            m
        );
        let broken = parse_ui_material(
            "duo",
            &text(&code.replace("p.light", "p.bright"), params),
            &mut refuse,
        )
        .expect("parses");
        let e = validate_ui_material(&broken).expect_err("refused");
        assert!(e.contains("hooks:3"), "{e}");
        let e = parse_ui_material("duo", &text("fn other() {}", ""), &mut refuse)
            .expect_err("no material");
        assert!(e.contains("material"), "{e}");
        let five: String = (0..5).map(|i| format!("param p{i} float 0\n")).collect();
        let e = parse_ui_material("duo", &text(code, &five), &mut refuse).expect_err("five");
        assert!(e.contains("at most"), "{e}");
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
