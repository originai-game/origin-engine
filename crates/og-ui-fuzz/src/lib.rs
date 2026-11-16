//! og-ui's untrusted inputs, driven as a remote `<iframe>` delivers them
//! (docs/ui.md §2.1 Frames). The fuzz targets (`fuzz_targets/`) call these;
//! `tests/regressions.rs` replays every input that once failed
//! (`regressions/<target>/`), so a fixed crash stays fixed without a
//! fuzzer.

use og_ui::reactive::SourceId;
use og_ui::{InputFrame, Ui, UiDesc, UiDocument, Viewport, bind};

/// A document's text: parsed, and when it parses, mounted with no stores
/// and built for two frames. A panic, an overflow or a hang is a bug; an
/// error is the answer an untrusted document gets.
pub fn document(bytes: &[u8]) {
    let text = String::from_utf8_lossy(bytes);
    let Ok(doc) = UiDocument::parse("ui/f.svelte", &text) else {
        return;
    };
    let name = doc.name.clone();
    let mut ui = Ui::new(UiDesc::default());
    ui.define_document(doc);
    let mount = SourceId::new("fuzz", "f");
    let env = &bind::NoEnv;
    if ui
        .mount_document(mount, &name, Default::default(), Default::default(), env)
        .is_err()
    {
        return;
    }
    for _ in 0..2 {
        ui.begin(InputFrame::default(), Viewport::new(320.0, 240.0));
        let _ = ui.build_retained_documents(env);
        ui.end();
    }
}
