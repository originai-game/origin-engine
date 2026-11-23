//! The GPU's counts of the last frame are read back only for a probe call
//! that reports them. Natively the read waits for the GPU, so a host that
//! drains `assertion_events` every frame (`og run --json`, so every
//! playtest) must not pay it: that wait was a GPU sync per frame inside
//! every playtest's frameMs budget.
//!
//! Ways this could fail: every probe call reads the counts (the old
//! eager read in `answer_call`); `state` stops reading them and reports a
//! stale `timings.triangles`.

use og_render::RenderScene;
use og_runtime::{
    Dispatch, Fetch, Hooks, RenderBackend, og_engine_create, og_engine_frame,
    og_engine_probe_assertion_events, og_game_game, og_host_begin_bundle, og_runtime_probe_state,
    set_hooks, set_render_backend,
};
use serde_json::{Value, json};
use std::cell::Cell;
use std::rc::Rc;

struct NoAssets;

impl Hooks for NoAssets {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        Fetch::Failed(format!("{path}: no assets in this test"))
    }
}

/// Counts how often the runtime reads the GPU's counts back.
struct Counter(Rc<Cell<u32>>);

impl RenderBackend for Counter {
    fn tier(&self) -> String {
        "none".into()
    }
    fn size(&self) -> (u32, u32) {
        (64, 64)
    }
    fn render(&mut self, _: &RenderScene) -> Result<(), String> {
        Ok(())
    }
    fn capture(&mut self, _: &RenderScene, _: u32, _: u32) -> Option<og_render::RgbaImage> {
        None
    }
    fn read_counts(&mut self) {
        self.0.set(self.0.get() + 1);
    }
}

fn value(text: String) -> Value {
    let v: Value = serde_json::from_str(&text).expect("wire JSON");
    assert_eq!(v["ok"], json!(true), "{text}");
    v["value"].clone()
}

#[test]
fn only_a_call_that_reports_gpu_counts_reads_them_back() {
    set_hooks(Rc::new(NoAssets));
    og_host_begin_bundle();
    let game = og_game_game(Some(r#"{"seed":1}"#.into()));
    let engine = value(og_engine_create(game, "{}".into())).as_u64().unwrap() as u32;
    let reads = Rc::new(Cell::new(0));
    set_render_backend(engine, Some(Box::new(Counter(reads.clone())))).unwrap();
    for _ in 0..3 {
        value(og_engine_frame(engine, 1000.0 / 60.0));
        og_engine_probe_assertion_events(engine);
    }
    assert_eq!(
        reads.get(),
        0,
        "draining assertion events read the GPU counts back"
    );
    og_runtime_probe_state(engine);
    assert_eq!(
        reads.get(),
        1,
        "state() reports timings.triangles, so it reads them"
    );
}
