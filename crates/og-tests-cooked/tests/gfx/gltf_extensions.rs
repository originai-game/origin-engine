//! The glTF extension scenes (roadmap G1-3, `og_tests_cooked::gltf_scenes`):
//! each cooks, carries what its extension says into the cooked model and
//! the scene, and matches its reference in og-tests' references/. The
//! references were not recorded with the scenes (they were written on an
//! orb with no GPU): until macrunner records them (`OG_GFX_CAPTURE=<scene>`)
//! each test fails at its reference, after its cooked checks pass.
//!
//! Before G1-3 the cook dropped every one of these extensions (and refused
//! `EXT_mesh_gpu_instancing`, which the instancing file requires): no
//! transform, no lights, one box instead of seven, no variants.

use super::*;
use og_tests_cooked::gltf_scenes;

/// `gltf-uv-transform`: both materials carry their base colour transform
/// (the emissive slot's identity keeps the cook from baking it), as glTF
/// wrote it, and the frame matches its reference.
#[test]
fn gltf_uv_transform_is_carried_and_matches_its_reference() {
    let (_, model) = gltf_scenes::uv_transform_scene().expect("the scene cooks");
    // The two quads' materials (the default material follows them).
    let carried: Vec<_> = model.materials[..2]
        .iter()
        .map(|m| {
            let t = m.uv_transforms[0];
            (t.offset.to_array(), t.scale.to_array(), t.rotation)
        })
        .collect();
    assert_eq!(
        carried,
        [
            ([0.1, 0.0], [3.0, 2.0], 0.35),
            ([0.5, 0.5], [0.5, 0.5], 0.0)
        ],
        "each quad's base colour transform, in glTF order"
    );
    assert!(
        model
            .materials
            .iter()
            .all(|m| m.uv_transforms[3].is_identity()),
        "the emissive slots keep no transform"
    );
    check("gltf-uv-transform");
}

/// `gltf-punctual-lights`: the point and the spot draw as local lights
/// (the spot along its node's -Z, straight down) and the directional one
/// as the sun, and the frame matches its reference.
#[test]
fn gltf_punctual_lights_light_the_floor_and_match_their_reference() {
    let (scene, model) = gltf_scenes::lights_scene().expect("the scene cooks");
    assert_eq!(model.lights.len(), 3, "point, spot and directional");
    assert_eq!(scene.lights.len(), 2, "the local lights");
    match scene.lights[1].kind {
        og_render::LightKind::Spot {
            direction,
            angle_degrees,
            ..
        } => {
            assert!(
                direction.abs_diff_eq(glam::Vec3::NEG_Y, 1e-4),
                "{direction}"
            );
            assert!((angle_degrees - 0.45f32.to_degrees()).abs() < 1e-3);
        }
        other => panic!("the second light is the spot: {other:?}"),
    }
    assert!(
        (scene.light.intensity - 0.5).abs() < 1e-6,
        "the sun is the glTF's directional light"
    );
    let s = check("gltf-punctual-lights").stats;
    assert_eq!(s.lights, 2, "both local lights drawn");
}

/// `gltf-instancing`: one mesh drawn at its seven instances (not seven
/// copies), and the frame matches its reference.
#[test]
fn gltf_instancing_draws_seven_instances_of_one_mesh() {
    let (_, model, parts) = gltf_scenes::instancing_scene().expect("the scene cooks");
    assert_eq!(model.primitives.len(), 1, "one mesh, not copies");
    assert_eq!(parts.parts.len(), gltf_scenes::INSTANCES);
    assert!(
        parts.parts.iter().all(|p| p.0 == parts.parts[0].0),
        "every instance draws the one mesh"
    );
    let s = check("gltf-instancing").stats;
    assert_eq!(
        s.instances as usize,
        gltf_scenes::INSTANCES + 1,
        "the boxes and the floor"
    );
}

/// `gltf-variants-0` and `-1`: variant 0 maps the body red and keeps the
/// trim's own material, variant 1 maps both, and each frame matches its
/// reference.
#[test]
fn gltf_variants_switch_materials_and_match_their_references() {
    let (_, model) = gltf_scenes::variants_scene(0).expect("the scene cooks");
    assert_eq!(model.variants, ["red", "blue"]);
    let name = |v| {
        (0..model.primitives.len())
            .map(|p| model.materials[model.primitive_material(p, v)].name.clone())
            .collect::<Vec<_>>()
    };
    assert_eq!(name(None), ["body", "trim"]);
    assert_eq!(name(Some(0)), ["red", "trim"]);
    assert_eq!(name(Some(1)), ["blue", "gold"]);
    check("gltf-variants-0");
    check("gltf-variants-1");
}
