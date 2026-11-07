//! An Effekseer project in `assets/` cooks as a composed effect: its vfx
//! under the project's own path, the emitters and ring meshes it generates
//! beside it, the textures it names read relative to it, and what it could
//! not carry said as the cook's warnings (samples: Effekseer's CC0 effects,
//! credited in `third_party/effekseer/`).

use og_cook::runtime::Bundle;
use og_cook::{CookOptions, cook_project};
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/effekseer")
            .join(name),
    )
    .unwrap()
}

#[test]
fn an_effekseer_project_cooks_as_a_vfx_with_what_it_needs() {
    let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-cook-effekseer");
    let _ = std::fs::remove_dir_all(&root);
    for (to, from) in [
        ("fx/sprite.efkproj", "Simple_Sprite_BillBoard.efkproj"),
        ("fx/ring.efkproj", "Simple_Ring_Shape1.efkproj"),
        ("fx/Texture/Particle01.png", "Texture/Particle01.png"),
    ] {
        let p = root.join("assets").join(to);
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(p, fixture(from)).unwrap();
    }
    let report = cook_project(&root, &CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    for id in [
        "fx/sprite.efkproj",
        "fx/sprite.efkproj#n0.emitter.ogt",
        "fx/ring.efkproj",
        "fx/ring.efkproj#n0.ring.glb",
        "fx/ring.efkproj#n0.effect.ogt",
        "fx/Texture/Particle01.png",
    ] {
        assert!(bundle.output(id).is_ok(), "{id} is not in the bundle");
    }
    let id = "fx/sprite.efkproj";
    let bytes = std::fs::read(root.join("cooked").join(&bundle.output(id).unwrap().file)).unwrap();
    let vfx = og_cook::vfx::parse_vfx(&bundle.vfx_text(id, &bytes).unwrap()).unwrap();
    assert_eq!(
        vfx.references().emitters,
        ["fx/sprite.efkproj#n0.emitter.ogt"]
    );
    // The sprite's random colour is a warning of the cook, named by file and node.
    assert!(
        report
            .warnings
            .iter()
            .any(|w| w.starts_with("assets/fx/sprite.efkproj: warning: Effekseer import: node 0")),
        "{:?}",
        report.warnings
    );
}
