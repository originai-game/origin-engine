//! og.ui's world-space UI through the engine (`docs/ui.md` §1): the UI's
//! end hands og-ui the tick's og.Camera and the pose of the entity an
//! anchor (a document's `<anchor entity>`, built through og-ui's
//! `Ui::world_anchor`) sits on, so the anchor lands where og.camera.worldToScreen puts
//! that entity and follows the camera when it moves; mass elements a system
//! spawns are counted and age out with the ticks.
//!
//! Ways it could fail: the end never sets the view (the anchor is hidden,
//! `inView` false), it reads the camera of the tick before or a stale
//! default (the anchor does not move with the camera), the entity's pose is
//! not handed in (an entity anchor is hidden), or spawned instances are
//! dropped between ticks.

use og_runtime::{
    Dispatch, Fetch, Hooks, og_camera_world_to_screen, og_engine_create, og_engine_frame,
    og_engine_write_resource, og_game_game, og_host_begin_bundle, og_system_define_system,
    og_ui_define_mass, og_ui_mass_count, og_ui_spawn_mass, og_world_spawn, set_hooks, shared_ui,
};
use serde_json::{Value, json};
use std::cell::{Cell, RefCell};
use std::rc::Rc;

const BUILD: u32 = 1;

/// The tag over entity `{entity}`, and its twin scaled by distance (`5 /
/// distance`, within 0.5 to 2), whose width over the tag's is how far the
/// entity is from the eye.
fn tags(entity: u64) -> String {
    format!(
        r#"<meta name="component" content="tags">
<div id="Root">
  <anchor id="Tag" entity={{{entity}}}><p id="Name">Brenna</p></anchor>
  <anchor id="Scaled" entity={{{entity}}} scale-at="5"><p id="Twin">Brenna</p></anchor>
</div>"#
    )
}

/// Where an anchor of [`tags`] was placed: its pivot (a document's anchor
/// stands on its point by its bottom centre) and width, CSS px.
#[derive(Clone, Copy, Debug)]
struct Placed {
    x: f32,
    y: f32,
    width: f32,
}

fn placed(e: u32, id: &str) -> Placed {
    let ui = shared_ui(e).expect("the engine's UI");
    let ui = ui.lock();
    let el = ui
        .probe_elements()
        .into_iter()
        .find(|el| el.kind == "world" && el.id.ends_with(id))
        .unwrap_or_else(|| panic!("{id} placed"));
    Placed {
        x: el.x + el.width / 2.0,
        y: el.y + el.height,
        width: el.width,
    }
}

#[derive(Default)]
struct Seen {
    spawn: Cell<bool>,
    template: Cell<u32>,
    counts: RefCell<Vec<u32>>,
}

struct Host(Rc<Seen>);

impl Hooks for Host {
    fn dispatch(&self, call: &Dispatch) -> Result<String, String> {
        if call.kind == 0 && call.id == BUILD {
            let s = &self.0;
            if s.template.get() == u32::MAX {
                let result: Value =
                    serde_json::from_str(&og_runtime::host_impl::answer(og_ui_define_mass(
                        "damage".into(),
                        Some(json!({ "kind": "number", "lifetimeTicks": 10 }).to_string()),
                    )))
                    .unwrap();
                assert_eq!(result["ok"], true, "{result}");
                s.template.set(result["value"].as_u64().unwrap() as u32);
            }
            if s.spawn.replace(false) {
                let list: Vec<Value> = (0..25)
                    .map(|i| json!({ "position": { "x": i, "y": 1, "z": 0 }, "value": i }))
                    .collect();
                og_ui_spawn_mass(s.template.get(), json!(list).to_string());
            }
            s.counts
                .borrow_mut()
                .push(og_ui_mass_count(s.template.get()));
        }
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets"))
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

fn camera(e: u32, x: f64) {
    // Looking down -Z (the identity) from (x, 0, 5).
    let cam = json!({ "position": { "x": x, "y": 0, "z": 5 }, "fov": 50.0 });
    value(og_engine_write_resource(
        e,
        "og.Camera".into(),
        cam.to_string(),
    ));
}

#[test]
fn an_entity_anchor_follows_the_ticks_camera_and_mass_elements_age_out() {
    let seen = Rc::new(Seen::default());
    seen.template.set(u32::MAX);
    set_hooks(Rc::new(Host(seen.clone())));
    og_host_begin_bundle();
    let system = og_system_define_system(
        "BuildTags".into(),
        json!({ "reads": [], "writes": [] }).to_string(),
        BUILD,
    );
    let game = og_game_game(Some(json!({ "seed": 1, "systems": [system] }).to_string()));
    let e = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let at = json!({ "position": { "x": 1.0, "y": 0.5, "z": 0.0 } });
    let entity = value(og_runtime::host_impl::answer(og_world_spawn(
        e,
        Some(at.to_string()),
    )))
    .as_u64()
    .unwrap();
    // The game's tags: a document the UI's end builds every tick.
    {
        let ui = shared_ui(e).expect("the engine's UI");
        let mut ui = ui.lock();
        ui.define_document(
            og_ui::UiDocument::parse("ui/tags.svelte", &tags(entity)).expect("parses"),
        );
        ui.mount_document(
            og_ui::reactive::SourceId::new("test", "tags"),
            "tags",
            Default::default(),
            Default::default(),
            &og_ui::bind::NoEnv,
        )
        .expect("mounts");
    }
    camera(e, 0.0);
    seen.spawn.set(true);
    let step = || value(og_engine_frame(e, 1000.0 / 60.0));
    step();
    step();
    // Where og.camera.worldToScreen puts the entity's point.
    let screen = |e: u32| -> (f64, f64) {
        let s: Value = serde_json::from_str(&og_runtime::host_impl::json(
            og_camera_world_to_screen(e, json!({ "x": 1.0, "y": 0.5, "z": 0.0 }).to_string()),
        ))
        .unwrap();
        (s["x"].as_f64().unwrap(), s["y"].as_f64().unwrap())
    };
    let (sx, sy) = screen(e);
    // Where the last end placed it.
    let info = placed(e, "Tag");
    let vp = og_runtime::og_ui_viewport(e);
    let (w, h) = (vp.width as f32, vp.height as f32);
    assert!(
        info.x > 0.0 && info.x < w && info.y > 0.0 && info.y < h,
        "in view: {info:?} in {w}x{h}"
    );
    // The scaled twin is 5 / distance the tag's width.
    let distance = 5.0 * info.width / placed(e, "Scaled").width;
    assert!(
        (f64::from(distance) - 26.25f64.sqrt()).abs() < 0.05,
        "{distance} from the eye"
    );
    assert!(
        (f64::from(info.x) - sx).abs() < 0.5 && (f64::from(info.y) - sy).abs() < 0.5,
        "{info:?} against worldToScreen ({sx}, {sy})"
    );
    // The camera moves right: the anchor moves left with the entity's image.
    camera(e, 2.0);
    step();
    step();
    let (mx, _) = screen(e);
    let moved = placed(e, "Tag");
    assert!(
        mx < sx - 50.0,
        "the camera did not move the point: {sx} -> {mx}"
    );
    assert!(
        (f64::from(moved.x) - mx).abs() < 0.5,
        "{moved:?} did not follow the camera to {mx}"
    );
    // 25 numbers spawned on the first tick, alive for their 10 ticks.
    for _ in 0..10 {
        step();
    }
    let counts = seen.counts.borrow().clone();
    assert_eq!(counts[1], 25, "{counts:?}");
    assert_eq!(*counts.last().unwrap(), 0, "{counts:?}");
}
