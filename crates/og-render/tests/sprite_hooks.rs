//! Sprites drawn with a material shader (`sprite_hooks.rs`): the same
//! hooks a mesh runs cut a sprite where `surface_color`'s alpha is 0 (a
//! dissolve), tint it, and a `mode sprite` shader reads the texture
//! elsewhere with `sprite_texel`; a parameter set after the material was
//! made moves the cut; a sprite whose material has no shader draws plain.

use glam::{Vec2, Vec3};
use og_render::{
    Camera, ColorSpace, Material, MaterialShader, Mips, ParamKind, RenderDesc, RenderScene,
    ShaderMode, ShaderParam, Sprite, SpriteShading, TextureData, render_to_rgba,
};

const W: u32 = 64;
const H: u32 = 32;

/// A quad filling the frame's width over the default sky, its left half
/// red and its right half green in the texture.
fn scene(shader: Option<MaterialShader>, cut: Option<f32>) -> RenderScene {
    let mut scene = RenderScene::new();
    scene.frame.camera = Camera::orthographic_2d(Vec2::ZERO, 1.0);
    let mut rgba = Vec::new();
    for _y in 0..8 {
        for x in 0..8 {
            rgba.extend_from_slice(if x < 4 {
                &[255, 0, 0, 255]
            } else {
                &[0, 255, 0, 255]
            });
        }
    }
    let texture = scene.add_texture(TextureData {
        width: 8,
        height: 8,
        rgba,
        color_space: ColorSpace::Linear,
        mips: Mips::None,
    });
    scene.set_texture_filter(texture, og_render::TextureFilter::Nearest);
    scene.add_sprite(Sprite {
        texture: Some(texture),
        position: Vec3::ZERO,
        size: Vec2::new(2.0, 1.0),
        anchor: Vec2::splat(0.5),
        ..Default::default()
    });
    let material = scene.add_material(Material::default());
    if let Some(shader) = shader {
        let s = scene.add_shader(shader);
        assert!(scene.set_material_shader(material, Some(s)));
        if let Some(cut) = cut {
            assert!(scene.set_shader_param(material, "cut", &[cut]));
        }
    }
    scene.sprite.sprite_shading.push(SpriteShading {
        sprite: 0,
        material,
    });
    scene
}

fn dissolve(mode: ShaderMode, code: &str) -> MaterialShader {
    MaterialShader {
        name: "dissolve".into(),
        code: code.into(),
        params: vec![ShaderParam {
            name: "cut".into(),
            kind: ParamKind::Float,
            default: [0.75, 0.0, 0.0, 0.0],
        }],
        mode,
    }
}

/// The frame's pixel at (x, y), rgb.
fn at(scene: &RenderScene, points: &[(u32, u32)]) -> Vec<[u8; 3]> {
    let desc = RenderDesc {
        width: W,
        height: H,
        ..Default::default()
    };
    let img = render_to_rgba(&desc, scene).expect(
        "a GPU adapter: a GPU test, run where there is one (OG_RENDERER=none takes it away)",
    );
    points
        .iter()
        .map(|&(x, y)| {
            let i = ((y * W + x) * 4) as usize;
            [img.pixels[i], img.pixels[i + 1], img.pixels[i + 2]]
        })
        .collect()
}

// Columns at 1/8, 5/8 and 7/8 of the quad (the frame is the quad's width).
const POINTS: [(u32, u32); 3] = [(8, 16), (40, 16), (56, 16)];

fn red(c: [u8; 3]) -> bool {
    c[0] > 150 && c[1] < 60
}
fn green(c: [u8; 3]) -> bool {
    c[1] > 150 && c[0] < 60
}
/// The sky behind the quad: a light grey, neither red nor green.
fn sky(c: [u8; 3]) -> bool {
    c.iter().all(|&v| v > 120) && c.iter().max().unwrap() - c.iter().min().unwrap() < 40
}

#[test]
#[ignore = "requires GPU"]
fn a_material_shader_dissolves_tints_and_distorts_a_sprite() {
    // No shader on the material: plain.
    let plain = at(&scene(None, None), &POINTS);
    assert!(
        red(plain[0]) && green(plain[1]) && green(plain[2]),
        "{plain:?}"
    );

    // A hooks shader (as a mesh would run it) cuts at uv.x > cut.
    let hooks = "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {\n  return vec4<f32>(c.rgb, select(c.a, 0.0, h.uv.x > p.cut));\n}\n";
    let cut = at(
        &scene(Some(dissolve(ShaderMode::Hooks, hooks)), None),
        &POINTS,
    );
    assert!(red(cut[0]) && green(cut[1]) && sky(cut[2]), "{cut:?}");

    // The parameter set after: the cut moves to the middle.
    let half = at(
        &scene(Some(dissolve(ShaderMode::Hooks, hooks)), Some(0.5)),
        &POINTS,
    );
    assert!(red(half[0]) && sky(half[1]) && sky(half[2]), "{half:?}");

    // A sprite shader reads the texel half the quad to its left: the right
    // half shows red, the left half (reading outside the texture, clamped)
    // red too; the emissive hook adds blue everywhere it draws.
    let shift = "fn surface_color(h: Hook, p: Params, c: vec4<f32>) -> vec4<f32> {\n  return sprite_texel(h.uv - vec2<f32>(0.5, 0.0));\n}\nfn surface_emissive(h: Hook, p: Params, e: vec3<f32>) -> vec3<f32> {\n  return e + vec3<f32>(0.0, 0.0, 1.0);\n}\n";
    let shifted = at(
        &scene(Some(dissolve(ShaderMode::Sprite, shift)), None),
        &POINTS,
    );
    for c in &shifted {
        assert!(red(*c) && c[2] > 150, "{shifted:?}");
    }
}
