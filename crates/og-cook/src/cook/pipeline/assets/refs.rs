//! Only retained runtime fields make delivery edges. Author inputs and cook
//! stamps remain build dependencies even when their data is embedded here.
use super::*;
use crate::format;
use crate::runtime::{LoadError, invalid};
use og_anim::cooked::BundleAnim;
use og_i18n::{BundleFontPacks, BundleStrings};

fn relocations(value: &serde_json::Value, out: &mut BTreeSet<String>) {
    match value {
        serde_json::Value::Object(fields) => {
            if let Some(id) = fields.get("$ogRef").and_then(serde_json::Value::as_str) {
                out.insert(id.into());
            } else {
                fields.values().for_each(|v| relocations(v, out));
            }
        }
        serde_json::Value::Array(values) => values.iter().for_each(|v| relocations(v, out)),
        _ => {}
    }
}

pub(super) fn retained(
    bundle: &Bundle,
    output: &ManifestOutput,
) -> Result<BTreeSet<String>, LoadError> {
    let id = output.id.as_str();
    let mut out = BTreeSet::new();
    // No runtime handles in these value/embedded-data payloads. In particular
    // do not read large media chunks or baked navigation geometry to index them.
    if matches!(
        output.ty.as_str(),
        "og.Animation"
            | "og.Behaviour"
            | "og.EnvironmentMap"
            | "og.Field"
            | "og.Font"
            | "og.Fracture"
            | "og.Lightmap"
            | "og.Lut"
            | "og.MaterialShader"
            | "og.NavMesh"
            | "og.Palette"
            | "og.PostPass"
            | "og.Rig"
            | "og.SoundChunk"
            | "og.UiMaterial"
            | "og.Vector"
            | "og.VideoChunk"
    ) {
        return Ok(out);
    }
    let bytes = bundle.read(id)?;
    let exists = |id: &str| bundle.output(id).is_ok();
    // The runtime discovers these named companions, not all outputs made by
    // the same source (which can include build-only inputs or unused rigs).
    let companions: &[&str] = match output.ty.as_str() {
        "og.Model" => &["#anim", "#fracture"],
        "og.Scene" => &["#hlod"],
        "og.Texture" => &["#normal", "#sway"],
        _ => &[],
    };
    out.extend(
        companions
            .iter()
            .map(|s| format!("{id}{s}"))
            .filter(|s| exists(s)),
    );
    match output.ty.as_str() {
        "og.Model" => out.extend(bundle.model(id, &bytes)?.textures.into_iter().map(|t| t.id)),
        "og.Scene" => {
            let scene = bundle.scene(id, &bytes)?;
            out.extend(scene.prefabs);
            for entity in scene.entities {
                out.extend(entity.instance);
                out.extend(entity.subscene);
            }
            for fields in scene
                .bindings
                .values()
                .chain(std::iter::once(&scene.resource_bindings))
            {
                for value in fields.values().flat_map(|f| f.values()) {
                    relocations(value, &mut out);
                }
            }
            out.extend(bundle.manifest().lightmaps.get(id).cloned());
        }
        "og.Table" => {
            for fields in bundle.table(id, &bytes)?.bindings.values() {
                for value in fields.values() {
                    relocations(value, &mut out);
                }
            }
        }
        "og.TextureArray" => out.extend(bundle.texture_array(id, &bytes)?.layers),
        "og.Texture" => {
            if bundle.texture(id, &bytes)?.mips.is_some() {
                out.insert(crate::runtime::mips_id(id));
            }
        }
        "og.ParticleEmitter" => out.extend(
            bundle
                .emitter(id, &bytes)?
                .resources
                .iter()
                .filter_map(|r| r.asset().map(str::to_owned)),
        ),
        "og.EffectMaterial" => out.extend(
            bundle
                .effect(id, &bytes)?
                .resources
                .iter()
                .filter_map(|r| r.asset().map(str::to_owned)),
        ),
        "og.Vfx" => {
            let r = bundle.vfx(id, &bytes)?.references();
            out.extend(
                [
                    r.emitters,
                    r.effects,
                    r.models,
                    r.sounds,
                    r.flipbooks,
                    r.textures,
                ]
                .into_iter()
                .flatten(),
            );
        }
        "og.Look" => {
            let look = bundle.look(id, &bytes)?;
            for block in look.blocks {
                let record = og_records::RECORDS
                    .iter()
                    .find(|r| r.name == block.resource);
                for (name, values, _) in block.fields {
                    if record
                        .and_then(|r| r.fields.iter().find(|f| f.name == name))
                        .is_some_and(|f| {
                            matches!(
                                f.column,
                                "Texture" | "TextureArray" | "Palette" | "Lut" | "EnvironmentMap"
                            )
                        })
                    {
                        out.extend(values.iter().filter_map(|v| v.as_str().map(str::to_owned)));
                    }
                }
            }
            if let Some(ui) = look.ui {
                let directory = super::super::super::look::resolve_style(&ui, &|p| {
                    exists(&format!("{p}/pack.json"))
                })
                .map_err(|reason| LoadError::Invalid {
                    path: id.into(),
                    reason,
                })?;
                out.insert(format!("{directory}/pack.json"));
            }
        }
        "og.Terrain" => {
            let terrain = bundle.terrain(id, &bytes)?;
            out.extend(terrain.albedo_array);
            out.extend(terrain.normal_array);
            for layer in terrain.layers {
                out.extend(layer.albedo);
                out.extend(layer.normal);
            }
        }
        "og.Tileset" => {
            out.insert(bundle.tileset(id, &bytes)?.texture);
        }
        "og.Tilemap" => {
            for tileset in bundle.tilemap(id, &bytes)?.tilesets {
                out.insert(match tileset {
                    og_tilemap::TilesetRef::Asset(id) => id,
                    og_tilemap::TilesetRef::Embedded(t) => t.texture,
                });
            }
        }
        "og.Flipbook" => {
            let sheet = bundle.flipbook(id, &bytes)?;
            out.insert(if sheet.array {
                format!("{id}#array")
            } else {
                sheet.texture
            });
        }
        "og.Rig2D" => {
            out.insert(bundle.rig2d(id, &bytes)?.texture);
        }
        "og.RiveArtboard" => {
            fn assets(board: og_rive::Artboard, out: &mut BTreeSet<String>) {
                out.extend(board.images.into_iter().map(|i| i.id));
                out.extend(board.fonts.into_iter().map(|f| f.id));
                for nested in board.nested {
                    assets(nested, out);
                }
            }
            let board = og_rive::decode(&bundle.rive_artboard(id, &bytes)?).map_err(|reason| {
                LoadError::Invalid {
                    path: id.into(),
                    reason,
                }
            })?;
            assets(board, &mut out);
        }
        "og.AnimGraph" => {
            let graph = bundle.animgraph(id, &bytes)?;
            out.extend(
                graph
                    .bindings
                    .into_iter()
                    .map(|b| crate::runtime::anim_id(&b.source)),
            );
        }
        "og.MotionAction" => {
            let action = bundle.motion_action(id, &bytes)?;
            out.insert(crate::runtime::anim_id(&action.source));
            out.insert(action.rig);
        }
        "og.TimelineAsset" => out.extend(super::super::super::timeline::refs(
            &bundle.timeline(id, &bytes)?,
        )),
        "og.Strings" => out.extend(bundle.strings(id, &bytes)?.fonts),
        "og.FontPack" => out.extend(bundle.font_pack(id, &bytes)?.faces),
        "og.Dialogue" | "og.Quest" => {
            let blob = if output.ty == "og.Dialogue" {
                bundle.blob::<format::Dialogue>(id, &bytes)?
            } else {
                bundle.blob::<format::Quest>(id, &bytes)?
            };
            let mut story = og_story::asset::Asset::from_bytes(blob.raw("data")?)
                .map_err(|e| invalid(&blob, e))?;
            story
                .map_resources(|_, id| {
                    out.insert(id.clone());
                    Ok(())
                })
                .map_err(|e| invalid(&blob, e))?;
        }
        "og.UiPack" => {
            let text = bundle.pack_text(id, &bytes)?;
            let layer =
                og_ui_next::pack::Layer::parse(&text).map_err(|reason| LoadError::Invalid {
                    path: id.into(),
                    reason,
                })?;
            out.extend(layer.assets().into_iter().map(str::to_owned));
        }
        "og.Sound" => {
            for n in 1..bundle.sound(id, &bytes)?.meta.chunk_count {
                out.insert(crate::runtime::chunk_id(id, n));
            }
        }
        "og.Video" => {
            bundle.video(id, &bytes)?;
            // Quality/codec variants and localized audio are selected at
            // runtime. Pruning them would remove a supported playback path.
            out.extend(
                bundle
                    .manifest()
                    .outputs()
                    .filter(|o| {
                        o.id.strip_prefix(id).is_some_and(|s| match o.ty.as_str() {
                            "og.VideoStream" => s.starts_with("#stream"),
                            "og.Sound" => s == "#audio" || s.starts_with("#audio."),
                            "og.TextureArray" => s == "#frames",
                            "og.Texture" => s == "#poster",
                            _ => false,
                        })
                    })
                    .map(|o| o.id.clone()),
            );
        }
        "og.VideoStream" => {
            let stream = bundle.video_stream(id, &bytes)?;
            out.extend((0..stream.meta.chunks).map(|n| crate::runtime::video_chunk_id(id, n)));
        }
        "og.Hlod" => {
            let tree = bundle.hlod(id, &bytes)?;
            let scene = id.strip_suffix("#hlod").unwrap_or(id);
            out.extend(
                (0..tree.nodes.len())
                    .map(|n| crate::hlod::node_id(scene, n))
                    .filter(|id| exists(id)),
            );
        }
        ty => {
            return Err(LoadError::Invalid {
                path: id.into(),
                reason: format!("{ty}: missing retained-reference contract"),
            });
        }
    }
    out.remove("");
    out.remove(id);
    Ok(out)
}
