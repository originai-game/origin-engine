//! Which stream file `og.load.video` reads (G3-2): the main stream, unless
//! the host's AV1 decoder is the web's rav1d-wasm fallback. Then a browser
//! whose WebCodecs decodes HEVC or H.264 reads that rendition, the first of
//! its codecs the cook made; otherwise a stream larger than 720p reads the
//! 720p rendition the cook made beside it; never two of them. A stream that
//! fits has no 720p rendition and the fallback reads the main one.
//!
//! Ways this could fail: a rendition read beside the main stream (the
//! browser pays for two downloads); a rendition asked for on a host with a
//! full AV1 decoder; a codec the browser decodes but the cook did not make
//! (the video fails to load instead of falling back); the cook's order
//! winning over the browser's; VideoInfo.codec saying av1 for H.264 units.

use og_runtime::{
    Dispatch, Fetch, Hooks, VideoDecoder, og_engine_create, og_game_game, og_host_begin_bundle,
    og_load_video, set_hooks,
};
use serde_json::{Value, json};
use std::cell::RefCell;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::OnceLock;

/// A project with a 1440x810 stream and a 640x360 one (each with the
/// default HEVC rendition) and a 640x360 one with both renditions, cooked
/// once (None without an ffmpeg with AV1, libx264 and libx265).
fn project() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| {
        og_cook::cook::video::av1_encoder()?;
        let has = og_cook::cook::video::has_encoder;
        (has("libx264") && has("libx265")).then_some(())?;
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
        for (name, size, line) in [
            ("wide", "1440 810", ""),
            ("small", "640 360", ""),
            ("both", "640 360", "renditions hevc h264\n"),
        ] {
            std::fs::write(
                clips.join(format!("{name}.ogt")),
                format!("type video\nversion 1\n---\nsource \"count.mp4\"\nstream true\nfps 10\nmaxSeconds 1\nsize {size}\naudio false\n{line}"),
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
/// reports `decoder` as its AV1 decoder and `codecs` as the others its
/// WebCodecs decodes.
struct Host {
    decoder: &'static str,
    codecs: &'static [&'static str],
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
        let codecs = self.codecs.iter().map(|c| c.to_string()).collect();
        match self.decoder {
            "rav1d-wasm" => VideoDecoder {
                name: "rav1d-wasm".into(),
                hardware: false,
                max_width: 1280,
                max_height: 720,
                codecs,
            },
            "webcodecs" => VideoDecoder {
                name: "webcodecs".into(),
                hardware: true,
                max_width: 3840,
                max_height: 2160,
                codecs,
            },
            _ => VideoDecoder::default(),
        }
    }
}

/// The stream files `og.load.video` of `clip` read under `decoder` and
/// `codecs`, and the codec its VideoInfo names.
fn streams_read(
    root: &Path,
    decoder: &'static str,
    codecs: &'static [&'static str],
    clip: &str,
) -> (Vec<String>, String) {
    let reads = Rc::new(RefCell::new(Vec::new()));
    set_hooks(Rc::new(Host {
        decoder,
        codecs,
        reads: reads.clone(),
    }));
    og_host_begin_bundle();
    let options = json!({ "root": root.to_str().unwrap(), "deferStart": true }).to_string();
    let created: Value =
        serde_json::from_str(&og_engine_create(og_game_game(Some("{}".into())), options)).unwrap();
    let world = created["value"].as_u64().unwrap() as u32;
    let loaded: Value = serde_json::from_str(&og_runtime::host_impl::answer(og_load_video(
        world,
        format!("clips/{clip}.ogt"),
    )))
    .unwrap();
    assert_eq!(loaded["ok"], json!(true), "{loaded}");
    let files = reads
        .borrow()
        .iter()
        .filter(|p| p.contains(".stream"))
        .map(|p| p.rsplit('/').next().unwrap().to_string())
        .collect();
    (
        files,
        loaded["value"]["codec"].as_str().unwrap().to_string(),
    )
}

#[test]
fn the_fallback_reads_only_the_720p_rendition_and_only_of_a_larger_stream() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with an AV1 encoder");
        return;
    };
    let read = |decoder, codecs, clip| streams_read(root, decoder, codecs, clip);
    let one = |file: &str, codec: &str| (vec![file.to_string()], codec.to_string());
    assert_eq!(
        read("rav1d", &[], "wide"),
        one("wide.ogt.stream.ogb", "av1")
    );
    assert_eq!(
        read("rav1d-wasm", &[], "wide"),
        one("wide.ogt.stream720.ogb", "av1")
    );
    assert_eq!(
        read("rav1d-wasm", &[], "small"),
        one("small.ogt.stream.ogb", "av1")
    );
}

#[test]
fn a_browser_without_av1_reads_the_hevc_or_h264_rendition_it_decodes_and_only_that() {
    let Some(root) = project() else {
        eprintln!("skipped: no ffmpeg with AV1, libx264 and libx265");
        return;
    };
    let read = |decoder, codecs, clip| streams_read(root, decoder, codecs, clip);
    let one = |file: &str, codec: &str| (vec![file.to_string()], codec.to_string());
    // In place of the 720p AV1: the full-size HEVC.
    assert_eq!(
        read("rav1d-wasm", &["hevc", "h264"], "wide"),
        one("wide.ogt.streamhevc.ogb", "hevc")
    );
    // The browser's order, not the cook's.
    assert_eq!(
        read("rav1d-wasm", &["h264", "hevc"], "both"),
        one("both.ogt.streamh264.ogb", "h264")
    );
    // H.264 alone, which the default cook did not make: the AV1 fallback.
    assert_eq!(
        read("rav1d-wasm", &["h264"], "wide"),
        one("wide.ogt.stream720.ogb", "av1")
    );
    // A browser that decodes AV1 plays it, whatever else it decodes.
    assert_eq!(
        read("webcodecs", &["hevc", "h264"], "both"),
        one("both.ogt.stream.ogb", "av1")
    );
}
