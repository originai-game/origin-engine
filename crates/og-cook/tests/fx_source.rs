//! Checked FX inputs cross the native transport and bind to host resources.
use og_bundle::fx::cooked::{Effect, Emitter, Resource};
use og_cook::cook::source_schemas::procedural::{EffectInput, ParticlesInput, VfxInput};
use og_render_data as r;
use serde_json::json;

fn shader_params() -> Vec<r::ShaderParam> {
    vec![
        r::ShaderParam {
            name: "untouched".into(),
            kind: r::ParamKind::Float,
            default: [0.7, 0.0, 0.0, 0.0],
        },
        r::ShaderParam {
            name: "tint".into(),
            kind: r::ParamKind::Vec4,
            default: [0.0; 4],
        },
    ]
}

#[test]
fn checked_fx_fields_round_trip_and_bind_typed_resources() {
    let reference = json!({"$ogRef":"north/texture"});
    let input = EffectInput::schema().decode(json!({
        "main":{"texture":reference,"channel":"blue","uv":{"tile":[2,3],"offset":[0.2,0.7],"rotate":17}},
        "mask":{"texture":{"$ogRef":"north/mask"}},
        "erosion":{"kind":"curve","keys":[[0,0.2],[1,0.9]]},
        "color":[3,2,1,0.7],"intensity":{"kind":"random","min":0.4,"max":3},
        "shader":{"$ogRef":"base/shader"},"params":{"tint":[1,0.25,0.5,0.75]}
    })).unwrap();
    let native = Effect::capture(|links| input.build(links), &|id| {
        assert_eq!(id, "base/shader");
        Ok(shader_params())
    })
    .unwrap();
    let bytes = native.to_bytes().unwrap();
    let decoded = Effect::from_bytes(&bytes).unwrap();
    let material = decoded
        .bind(&mut og_bundle::effect::EffectLinks {
            texture: &mut |id| {
                Ok(Some(r::TextureHandle(match id {
                    "north/texture" => 17,
                    "north/mask" => 29,
                    _ => panic!("unexpected texture {id}"),
                })))
            },
            // Runtime metadata must not overwrite the defaults/overrides fixed at cook.
            shader: &mut |id| {
                assert_eq!(id, "base/shader");
                Ok(Some((r::ShaderHandle(41), Vec::new())))
            },
        })
        .unwrap();
    assert_eq!(material.main.texture, Some(r::TextureHandle(17)));
    assert_eq!(material.mask.unwrap().texture, Some(r::TextureHandle(29)));
    assert_eq!(material.main.channel, r::EffectChannel::Blue);
    assert_eq!(material.main.uv.tile, glam::Vec2::new(2.0, 3.0));
    assert_eq!(material.main.uv.offset, glam::Vec2::new(0.2, 0.7));
    assert_eq!(material.main.uv.rotation, 17.0);
    assert_eq!(material.color, [3.0, 2.0, 1.0, 0.7]);
    assert_eq!(material.intensity, r::EffectValue::Random([0.4, 3.0]));
    assert_eq!(material.erosion.unwrap().amount.eval(0.5, 0.0), 0.55);
    assert_eq!(material.shader, Some(r::ShaderHandle(41)));
    assert_eq!(material.params[0], [0.7, 0.0, 0.0, 0.0]);
    assert_eq!(material.params[1], [1.0, 0.25, 0.5, 0.75]);
    for end in 0..bytes.len() {
        assert!(Effect::from_bytes(&bytes[..end]).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Effect::from_bytes(&trailing).is_err());
    let mut bad = decoded;
    bad.data.main.texture = Some(r::TextureHandle(u32::MAX));
    assert!(bad.to_bytes().is_err());

    let input = ParticlesInput::schema().decode(json!({
        "maxParticles":257,"rate":19,"lifetime":[0.3,1.7],"lit":true,"translucency":0.37,
        "texture":reference,"render":{"kind":"mesh","mesh":{"$ogRef":"north/model"}},
        "shape":{"kind":"mesh","mesh":"sphere"},"effect":{"$ogRef":"north/material"},
        "subEmitters":[{"trigger":"death","emitter":{"$ogRef":"north/child"},"count":3,"inheritVelocity":0.7}],
        "field":{"kind":"asset","asset":{"$ogRef":"north/field"},"strength":0.7,"size":3},
        "sizeOverLife":[[0,0.4],[1,1.3]]
    })).unwrap();
    let native = Emitter::capture(|links| {
        input.build(links, &|id, strength, size| {
            assert_eq!(id.id, "north/field");
            Ok(r::VectorField::Grid {
                cells: 1,
                strength,
                size,
                vectors: vec![glam::Vec3::new(1.3, -2.7, 3.1)],
            })
        })
    })
    .unwrap();
    let bytes = native.to_bytes().unwrap();
    let decoded = Emitter::from_bytes(&bytes).unwrap();
    let emitter = decoded
        .bind(&mut og_bundle::fx::EmitterLinks {
            texture: &mut |id| {
                assert_eq!(id, "north/texture");
                Ok(Some(r::TextureHandle(47)))
            },
            array: &mut |_| unreachable!(),
            mesh: &mut |mesh| {
                Ok(Some(r::MeshHandle(match mesh {
                    og_bundle::fx::ParticleMesh::Builtin("sphere") => 53,
                    og_bundle::fx::ParticleMesh::Asset("north/model") => 67,
                    _ => panic!("unexpected mesh"),
                })))
            },
            emitter: &mut |id| {
                assert_eq!(id, "north/child");
                Ok(Some(r::EmitterHandle(71)))
            },
            effect: &mut |id| {
                assert_eq!(id, "north/material");
                Ok(Some(r::EffectMaterialHandle(83)))
            },
        })
        .unwrap();
    assert_eq!(emitter.texture, Some(r::TextureHandle(47)));
    assert_eq!(
        emitter.shape,
        r::EmitterShape::Mesh {
            mesh: r::MeshHandle(53)
        }
    );
    assert_eq!(
        emitter.render,
        r::ParticleRender::Mesh {
            mesh: r::MeshHandle(67)
        }
    );
    assert_eq!(emitter.effect, Some(r::EffectMaterialHandle(83)));
    assert_eq!(emitter.translucency, 0.37);
    assert_eq!(emitter.sub_emitters[0].emitter, r::EmitterHandle(71));
    assert_eq!(emitter.sub_emitters[0].count, 3);
    assert_eq!(emitter.sub_emitters[0].inherit_velocity, 0.7);
    assert_eq!(
        (emitter.max_particles, emitter.rate, emitter.lifetime),
        (257, 19.0, [0.3, 1.7])
    );
    assert_eq!(emitter.size_over_life.keys, [(0.0, 0.4), (1.0, 1.3)]);
    assert_eq!(
        emitter.field,
        Some(r::VectorField::Grid {
            cells: 1,
            strength: 0.7,
            size: 3.0,
            vectors: vec![glam::Vec3::new(1.3, -2.7, 3.1)]
        })
    );
    for end in 0..bytes.len() {
        assert!(Emitter::from_bytes(&bytes[..end]).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Emitter::from_bytes(&trailing).is_err());
    let mut bad = decoded;
    bad.resources[0] = Resource::Shader("wrong kind".into());
    assert!(bad.to_bytes().is_err());

    let input = ParticlesInput::schema().decode(json!({"render":{"kind":"sprite","layer":-3},"array":{"asset":{"$ogRef":"base/array"},"layers":7,"mode":{"kind":"life","cycles":2.3}}})).unwrap();
    let native = Emitter::capture(|links| input.build(links, &|_, _, _| unreachable!())).unwrap();
    let decoded = Emitter::from_bytes(&native.to_bytes().unwrap()).unwrap();
    let emitter = decoded
        .bind(&mut og_bundle::fx::EmitterLinks {
            texture: &mut |_| unreachable!(),
            array: &mut |id| {
                assert_eq!(id, "base/array");
                Ok(Some(r::TextureArrayHandle(97)))
            },
            mesh: &mut |_| unreachable!(),
            emitter: &mut |_| unreachable!(),
            effect: &mut |_| unreachable!(),
        })
        .unwrap();
    assert_eq!(emitter.array, Some(r::TextureArrayHandle(97)));
    assert_eq!(emitter.sheet.frames, 7);
    assert_eq!(emitter.sheet.mode, r::SheetMode::OverLife { cycles: 2.3 });
}

#[test]
fn checked_vfx_timeline_preserves_native_fields_and_seeded_draws() {
    use og_bundle::vfx::{Vfx, VfxInstance, VfxKind};
    let input = VfxInput::schema().decode(json!({"duration":2.3,"params":[{"name":"other","default":0.7},{"name":"power","default":0.3}],
        "events":[{"time":0.7,"name":"later"},{"time":0.3,"name":"earlier"},{"time":0.7,"name":"same-time"}],
        "elements":[
            {"node":{"kind":"emitter","asset":{"$ogRef":"north/emitter"},"seed":16777219},"at":0.2,"until":0.9,"offset":[1,2,3],"when":"power","amount":"other"},
            {"node":{"kind":"beam","to":[1,3,2],"stretch":"power"},"at":0.2,"life":0.4}
        ]})).unwrap();
    let native = input.build().unwrap();
    assert_eq!(
        native
            .events
            .iter()
            .map(|e| e.name.as_str())
            .collect::<Vec<_>>(),
        ["earlier", "later", "same-time"]
    );
    assert_eq!(native.elements[0].when, Some(1));
    assert_eq!(native.elements[0].amount, Some(0));
    assert_eq!(native.elements[1].stretch, Some(1));
    assert_eq!(native.elements[0].offset, glam::Vec3::new(1.0, 2.0, 3.0));
    assert!((native.elements[0].life - 0.7).abs() < 0.000001);
    assert!(
        matches!(&native.elements[0].kind,VfxKind::Emitter{id,seed:16777219} if id=="north/emitter")
    );
    let bytes = native.to_bytes().unwrap();
    let decoded = Vfx::from_bytes(&bytes).unwrap();
    assert_eq!(decoded, native);
    for seed in [0, 83, u32::MAX] {
        let instance = VfxInstance {
            id: 12345,
            seed,
            transform: glam::Mat4::from_translation(glam::Vec3::new(1.3, -2.7, 0.9)),
            scale: 1.7,
            tint: glam::Vec3::new(0.3, 0.7, 0.9),
            hue: 0.23,
            params: native.values(&[0.37, 0.73]),
        };
        for tick in [5, 17, 29, 59, 95] {
            assert_eq!(
                decoded.draws(&instance, 5, tick, 60.0),
                native.draws(&instance, 5, tick, 60.0)
            );
        }
    }
    for end in 0..bytes.len() {
        assert!(Vfx::from_bytes(&bytes[..end]).is_err());
    }
    let mut trailing = bytes;
    trailing.push(0);
    assert!(Vfx::from_bytes(&trailing).is_err());
    let mut bad = decoded;
    bad.elements[1].stretch = Some(2);
    assert!(bad.to_bytes().is_err());
    for element in [
        json!({"node":{"kind":"glow"},"at":0.2,"until":0.1}),
        json!({"node":{"kind":"glow"},"life":0.4,"until":0.9}),
        json!({"node":{"kind":"glow"},"life":0.4,"when":"missing"}),
    ] {
        assert!(
            VfxInput::schema()
                .decode(json!({"duration":2,"elements":[element]}))
                .unwrap()
                .build()
                .is_err()
        );
    }
}
