//! Which stream file `og.load.video` reads (G3-2): the main stream, unless
//! the host's decoder is the web's rav1d-wasm fallback and the stream is
//! larger than 720p, when it reads the 720p rendition the cook made beside
//! it instead, never both; a stream that fits has no rendition and the
//! fallback reads the main one.
//!
//! Ways this could fail: the rendition read beside the main stream (the
//! fallback pays for both downloads); the rendition asked for on a host with
//! a full decoder; the fallback asking for a rendition a 720p stream does
//! not have (the video fails to load).

use og_runtime::{
    Dispatch, Fetch, Hooks, VideoDecoder, og_engine_create, og_game_game, og_host_begin_bundle,
    og_load_video, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// A project with a 1440x810 stream and a 640x360 one, cooked once (None
/// without an ffmpeg with an AV1 encoder).
fn project() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        og_cook::cook::video::av1_encoder()?;
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("og-runtime-video-rendition-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        let clips = root.join("assets/clips");
        std::fs::create_dir_all(&clips).unwrap();
        std::fs::copy(
            Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../packages/engine/tests/fixtures/video/assets/clips/count.mp4"),
            clips.join("count.mp4"),
        )
        .unwrap();
        for (name, size) in [("wide", "1440 810"), ("small", "640 360")] {
            std::fs::write(
                clips.join(format!("{name}.ogt")),
                format!("type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize {size}\naudio false\n"),
            )
            .unwrap();
        }
        let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        Some(root)
    })
    .as_deref()
}

/// Reads the cooked files from disk, records every path asked for, and
/// reports `decoder` as its AV1 decoder.
struct Host {
    decoder: &'static str,
    reads: Rc<RefCell<Vec<String>>>,
}

impl Hooks for Host {
    fn dispatch(&self, _call: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        self.reads.borrow_mut().push(path.to_string());
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(e) => Fetch::Failed(format!("{path}: {e}")),
        }
    }
    fn video_decoder(&self) -> VideoDecoder {
        match self.decoder {
            "rav1d-wasm" => VideoDecoder {
                name: "rav1d-wasm".into(),
                hardware: false,
                max_width: 1280,
                max_height: 720,
            },
            _ => VideoDecoder::default(),
        }
    }
}

/// The stream files `og.load.video` of `clip` read under `decoder`.
fn streams_read(root: &Path, decoder: &'static str, clip: &str) -> Vec<String> {
    let reads = Rc::new(RefCell::new(Vec::new()));
    set_hooks(Rc::new(Host {
        decoder,
        reads: reads.clone(),
    }));
    og_host_begin_bundle();
    let options = json!({ "root": root.to_str().unwrap(), "deferStart": true }).to_string();
    let created: Value =
        serde_json::from_str(&og_engine_create(og_game_game(Some("{}".into())), options)).unwrap();
    let world = created["value"].as_u64().unwrap() as u32;
    let loaded: Value =
        serde_json::from_str(&og_load_video(world, format!("clips/{clip}.ogt"))).unwrap();
    assert_eq!(loaded["ok"], json!(true), "{loaded}");
    reads
        .borrow()
        .iter()
        .filter(|p| p.contains(".stream"))
        .map(|p| p.rsplit('/').next().unwrap().to_string())
        .collect()
}

#[test]
fn the_fallback_reads_only_the_720p_rendition_and_only_of_a_larger_stream() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    assert_eq!(streams_read(root, "rav1d", "wide"), ["wide.ogt.stream.ogb"]);
    assert_eq!(
        streams_read(root, "rav1d-wasm", "wide"),
        ["wide.ogt.stream720.ogb"]
    );
    assert_eq!(
        streams_read(root, "rav1d-wasm", "small"),
        ["small.ogt.stream.ogb"]
    );
}
