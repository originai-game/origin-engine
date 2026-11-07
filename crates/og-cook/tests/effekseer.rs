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

/// The triangles of every primitive of a cooked model, summed.
fn triangles(root: &Path, id: &str) -> usize {
    let bundle = Bundle::open(&root.join("cooked")).unwrap();
    let (model, _) = bundle.load_model_with_textures(id).expect("decodes");
    model.primitives.iter().map(|p| p.lods[0].count / 3).sum()
}

#[test]
fn an_effects_meshes_are_held_to_no_face_budget() {
    // A ring is a mesh element's: an effect's mesh, not a prop, so a look's
    // propFaces neither simplifies it nor warns of it, cooked from the
    // project in assets/ or from what `og vfx import` wrote.
    let cook = |name: &str, faces: u32| {
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join("og-cook-effekseer-budget")
            .join(name);
        let _ = std::fs::remove_dir_all(&root);
        let p = root.join("assets/fx/ring.efkproj");
        std::fs::create_dir_all(p.parent().unwrap()).unwrap();
        std::fs::write(&p, fixture("Simple_Ring_Shape1.efkproj")).unwrap();
        let file = root.join("ring.efkproj");
        std::fs::write(&file, fixture("Simple_Ring_Shape1.efkproj")).unwrap();
        let out = root.join("assets/imported");
        let i = og_cook::cook::efk::import(&file, &out, Some(&root.join("assets"))).unwrap();
        assert!(
            i.written
                .iter()
                .any(|w| w.ends_with("ring/n0.ring.glb.ogt")),
            "{:?}",
            i.written
        );
        std::fs::write(
            root.join("project.ogt"),
            "type project\nversion 1\n---\nlook \"looks/main.ogt\"\n",
        )
        .unwrap();
        std::fs::create_dir_all(root.join("assets/looks")).unwrap();
        std::fs::write(
            root.join("assets/looks/main.ogt"),
            format!(
                "type look\nversion 1\n---\nArtShape {{\n  propFaces {faces}\n  normals flat\n}}\n"
            ),
        )
        .unwrap();
        let report = cook_project(&root, &CookOptions::default()).unwrap();
        assert!(report.ok(), "{:?}", report.errors());
        (root, report)
    };
    let (plain, _) = cook("no-budget", 0);
    let (held, report) = cook("budget", 50);
    for id in ["fx/ring.efkproj#n0.ring.glb", "imported/ring/n0.ring.glb"] {
        let made = triangles(&plain, id);
        assert!(made > 50, "{id}: {made} triangles");
        assert_eq!(triangles(&held, id), made, "{id} was simplified");
    }
    assert!(
        !report.warnings.iter().any(|w| w.contains("propFaces")),
        "{:?}",
        report.warnings
    );
}
