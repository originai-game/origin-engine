use super::*;
mod fx;
mod images;
mod media;
mod models;
mod text;
mod world;
use fx::*;
use images::*;
use media::*;
use models::*;
use text::*;
use world::*;

/// What every stage after the art contract reads.
#[derive(Clone, Copy)]
pub(crate) struct Cx<'a> {
    pub(crate) root: &'a Path,
    pub(crate) assets_dir: &'a Path,
    pub(crate) cooked_dir: &'a Path,
    pub(crate) options: &'a CookOptions,
    pub(crate) sources: &'a [Source],
    pub(crate) known: &'a BTreeSet<String>,
    pub(crate) prefab_texts: &'a BTreeMap<String, String>,
    pub(crate) group_of: &'a BTreeMap<String, String>,
    pub(crate) pack_faces: &'a BTreeSet<String>,
    pub(crate) old_outputs: &'a BTreeMap<&'a str, &'a ManifestOutput>,
    pub(crate) schemas: &'a BTreeMap<String, Schema>,
    pub(crate) budgets: &'a stats::Budgets,
    pub(crate) tex_desc: &'a str,
    pub(crate) mesh_desc: &'a str,
    pub(crate) font_desc: &'a str,
    pub(crate) budget_desc: &'a str,
    pub(crate) schema_hash: [u8; 8],
    pub(crate) style_tokens_hash: [u8; 8],
    pub(crate) story_functions: &'a BTreeMap<String, og_story::yarn::types::Function>,
    pub(crate) story_functions_stamp: &'a [u8],
    pub(crate) document_themes: &'a [og_ui::Theme],
    /// Each component name by the id of the document defining it: the
    /// game's own, else a built-in `@ui/` one (the scan's).
    pub(crate) components: &'a BTreeMap<String, String>,
}

/// What planning writes as it goes: the errors it finds, the references it
/// parses and the glTF textures already planned.
pub(crate) struct PlanState {
    pub(crate) early_errors: Vec<EarlyError>,
    pub(crate) parsed_refs: BTreeMap<String, BTreeSet<String>>,
    // glTF textures of models outside streaming groups are cooked once per
    // image and use, whichever models use them: a scene exported in many
    // glTFs (cells, tiles) repeats its images in every file, and a copy per
    // model multiplied the cooked bytes, the load's transcoding and the
    // GPU's memory (og-runtime's add_gltf shares them by these ids).
    planned_textures: std::collections::HashSet<String>,
}

/// One planned output: its id, type, stamp and work.
pub(crate) type Out<'a> = (String, &'static str, u64, Work<'a>);

// Plan every output with its stamp.
pub(crate) fn plan<'a>(
    cx: &Cx<'a>,
    early_errors: Vec<EarlyError>,
    threads: usize,
) -> (Vec<Planned<'a>>, PlanState) {
    let Cx {
        sources,
        cooked_dir,
        pack_faces,
        old_outputs,
        ..
    } = *cx;
    let mut state = PlanState {
        early_errors,
        parsed_refs: BTreeMap::new(),
        planned_textures: std::collections::HashSet::new(),
    };
    let mut plan: Vec<Planned> = Vec::new();
    for (si, s) in sources.iter().enumerate() {
        if s.error.is_some() {
            continue;
        }
        // These cookers consume only bytes/options. The other kinds resolve
        // relative references or write source-derived names/sub-asset ids;
        // their location remains part of the action, not the content hash.
        let hash = match s.kind {
            Kind::Texture
            | Kind::Sound
            | Kind::Font
            | Kind::Lut
            | Kind::Emitter
            | Kind::Document
            | Kind::Shader
            | Kind::Post
            | Kind::Tileset
            | Kind::Rig
            | Kind::MotionAction
            | Kind::Palette => stamp(&[&s.hash.to_le_bytes(), ext(&s.rel).as_bytes()]),
            Kind::System(k) if k.by_content => {
                stamp(&[&s.hash.to_le_bytes(), ext(&s.rel).as_bytes()])
            }
            _ => stamp(&[&s.hash.to_le_bytes(), s.rel.as_bytes()]),
        }
        .to_le_bytes();
        let mut outs: Vec<Out> = Vec::new();
        plan_kind(cx, &mut state, s, hash, &mut outs);
        for (id, ty, st, work) in outs {
            let file = if pack_faces.contains(&id) {
                format!("{id}.{}", og_bundle::runtime::FONT_PACK_FACE)
            } else {
                output_file(&id)
            };
            plan.push(Planned {
                source: si,
                id,
                file,
                ty,
                stamp: st,
                work: Some(work),
            });
        }
    }
    // Up to date: the manifest's stamp and file agree, and the file still
    // holds the type and schema version the loader expects today (a schema
    // change without a cooker bump would otherwise keep a blob the loader
    // refuses). Each output's file is read for it, all at once.
    let fresh = each(&plan, threads, |p| {
        let want = format::expected_header(p.ty);
        old_outputs.get(p.id.as_str()).is_some_and(|o| {
            o.stamp == p.stamp
                && o.ty == p.ty
                && std::fs::read(cooked_dir.join(&o.file)).is_ok_and(|b| {
                    b.len() as u64 == o.bytes
                        && hash_bytes(&b) == o.hash
                        && og_blob::Blob::parse(&b, &p.file).is_ok_and(|blob| {
                            let h = blob.header();
                            want == Some((h.type_hash, h.schema_version))
                        })
                })
        })
    });
    for (p, fresh) in plan.iter_mut().zip(fresh) {
        if fresh {
            p.work = None;
        }
    }
    (plan, state)
}

/// Plans a source's outputs by its kind.
fn plan_kind<'a>(
    cx: &Cx<'a>,
    state: &mut PlanState,
    s: &'a Source,
    hash: [u8; 8],
    outs: &mut Vec<Out<'a>>,
) {
    match s.kind {
        Kind::Model => plan_model(cx, state, s, hash, outs),
        Kind::Texture => plan_texture(cx, state, s, hash, outs),
        Kind::Sound if s.rel.ends_with(".ogt") => plan_sfx(cx, state, s, hash, outs),
        Kind::Sound => plan_sound(cx, state, s, hash, outs),
        Kind::Lut => plan_lut(cx, state, s, hash, outs),
        Kind::Emitter => plan_emitter(cx, state, s, hash, outs),
        Kind::Effect => plan_effect(cx, state, s, hash, outs),
        Kind::Vfx => plan_vfx(cx, state, s, hash, outs),
        Kind::Shader => plan_shader(cx, state, s, hash, outs),
        Kind::Post => plan_post(cx, state, s, hash, outs),
        Kind::UiMaterial => plan_ui_material(cx, state, s, hash, outs),
        Kind::Vector => plan_vector(cx, state, s, hash, outs),
        Kind::Rive => plan_rive(cx, state, s, hash, outs),
        Kind::Style => plan_style(cx, state, s, hash, outs),
        Kind::Document => plan_document(cx, state, s, hash, outs),
        Kind::Tileset => plan_tileset(cx, state, s, hash, outs),
        Kind::Tilemap => plan_tilemap(cx, state, s, hash, outs),
        Kind::Rig2d => plan_rig2d(cx, state, s, hash, outs),
        Kind::Flipbook => plan_flipbook(cx, state, s, hash, outs),
        Kind::FontPack => plan_font_pack(cx, state, s, hash, outs),
        Kind::Rig => plan_rig(cx, state, s, hash, outs),
        Kind::MotionAction => plan_motion_action(cx, state, s, hash, outs),
        Kind::Icons => plan_icons(cx, state, s, hash, outs),
        Kind::Terrain => plan_terrain(cx, state, s, hash, outs),
        Kind::Lightmap => plan_lightmap(cx, state, s, hash, outs),
        Kind::Timeline => plan_timeline(cx, state, s, hash, outs),
        Kind::TextureArray => plan_texture_array(cx, state, s, hash, outs),
        Kind::Video => plan_video(cx, state, s, hash, outs),
        Kind::Environment => plan_environment(cx, state, s, hash, outs),
        Kind::Look => plan_look(cx, state, s, hash, outs),
        Kind::Palette => plan_palette(cx, state, s, hash, outs),
        Kind::Font => plan_font(cx, state, s, hash, outs),
        Kind::Scene | Kind::Prefab => plan_scene(cx, state, s, hash, outs),
        Kind::System(k) => (k.plan)(cx, state, s, hash, outs),
    }
}
