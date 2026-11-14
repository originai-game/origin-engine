use crate::Ui;
use crate::UiDocument;
use crate::doc::build::Build;
use crate::doc::build::Built;
use crate::reactive::SourceId;
use serde_json::json;
use std::sync::Arc;

fn document(styled: bool) -> UiDocument {
    let css = if styled {
        "#Root { padding: 13px; gap: 7px; } #Title { text-transform: uppercase; color: #f0a020; position: absolute; top: 230px; } #Action { height: 36px; width: 173px; } #Check, #Range { width: 219px; padding: 5px; }"
    } else {
        ""
    };
    UiDocument::parse(
        "fields.svelte",
        &format!(
            r#"
<meta name="component" content="fields">
<meta name="prop:checked" content="bool">
<meta name="prop:amount" content="number">
<column id="Root">
  <text id="Title">Mixed Case</text>
  <button id="Action" variant="primary">Apply</button>
  <label><input type="checkbox" switch id="Check" checked={{checked}}> Enabled</label>
  <input type="range" id="Range" value={{amount}} show-value="false" />
</column>
<style>{css}</style>
"#
        ),
    )
    .unwrap()
}

#[test]
fn settled_document_fields_match_fresh_styles_without_rebuilding_controls() {
    let env = crate::bind::NoEnv;
    let mut patched = Ui::default();
    let mut direct = Ui::default();
    // Both mount the document once, holding its props' values; each round
    // below replaces the document they show.
    let mount = SourceId::new("test", "fields");
    let vm = json!({"checked": true, "amount": 0.37});
    for ui in [&mut patched, &mut direct] {
        ui.set_fonts(crate::Fonts {
            body: crate::UiFont {
                handle: Some(og_draw::FontHandle::from_bits(1)),
                metrics: crate::FontMetrics::fixed(0.55),
            },
            ..Default::default()
        });
        ui.define_document(document(false));
        ui.mount_with_values(mount.clone(), "fields", &vm).unwrap();
    }
    for styled in [true, false, true] {
        let target = Arc::new(document(styled));
        for (ui, doc) in [
            (&mut patched, document(!styled)),
            (&mut direct, (*target).clone()),
        ] {
            ui.define_document(doc);
            ui.begin(
                crate::InputFrame::default(),
                crate::Viewport::new(640.0, 480.0),
            );
            ui.build_retained_documents(&env).unwrap();
        }
        // Resolve before settlement; the callback has only evaluated values
        // and the existing controls, not a second build. The elements are
        // the mount's, as it recorded them when it emitted them, each
        // resolved with the inputs its mount compiled for it.
        let replacements: Vec<_> = patched
            .doc_built()
            .iter()
            .map(|node| {
                let key = crate::id::key_of_path(&node.path);
                let host = &patched.docs.state().memory.retained;
                let cached = host.elements[&host.by_key[&key]].cached.clone();
                let build = Build {
                    doc: target.clone(),
                    mount: mount.id.clone(),
                    env: &env,
                    row_key: String::new(),
                    inherited: Arc::default(),
                    events: Vec::new(),
                    writes: Vec::new(),
                    cached: Some(&cached),
                };
                let values = patched.resolve(
                    &build,
                    &target.nodes[node.node],
                    &node.path,
                    &mut Built::default(),
                    &build.inherited,
                    &mut |_| false,
                );
                (
                    node.node,
                    values,
                    patched.docs.render_fields(key).unwrap().clone(),
                )
            })
            .collect();
        patched.watch_layout_container(patched.tree.nodes()[0].key);
        // The first measure of this subscription must run on every test frame.
        patched.unwatch_layout_container(patched.tree.nodes()[0].key);
        patched.watch_layout_container(patched.tree.nodes()[0].key);
        patched
            .end_with_layout_queries(|ui, _| {
                let state = ui.save_state();
                let events = ui.docs.state().queued.clone();
                let count = ui.tree.nodes().len();
                for (node, values, fields) in &replacements {
                    ui.apply_document_fields(&target, &target.nodes[*node], values, fields);
                }
                assert_eq!(ui.save_state(), state);
                assert_eq!(ui.docs.state().queued, events);
                assert_eq!(ui.tree.nodes().len(), count);
                Ok(replacements
                    .iter()
                    .map(|(_, _, fields)| fields.index)
                    .collect())
            })
            .unwrap();
        direct.end();
        assert_eq!(patched.elements(), direct.elements(), "styled={styled}");
        assert_eq!(
            patched.frame().items,
            direct.frame().items,
            "styled={styled}"
        );
        assert_eq!(patched.a11y(), direct.a11y(), "styled={styled}");
    }
}

/// `-og-material: "<asset>" var(--burn)`: the defined material's handle,
/// its defaults under the numbers written, and a store driving one
/// through a custom property; `<og-overlay effect="material …">` takes the same.
///
/// Ways it could fail: an asset path read as a built-in (or refused), a
/// default lost under the written numbers, a store change that does not
/// reach the parameter.
#[test]
fn a_document_material_names_an_asset_and_binds_its_parameters() {
    use crate::reactive::SourceId;
    use crate::reactive::session::Command;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    let doc = r#"<meta name="component" content="card">
<div id="Root"><div id="Card" style:--burn={$burn}>Card</div><og-overlay id="Hurt" effect='material "fx/burn.ogt" 0.25'></og-overlay></div>
<style>
#Card { width: 120px; height: 160px; background: #334455; -og-material: "fx/burn.ogt" var(--burn) 0 0 0 6; }
</style>"#;
    let parsed = crate::doc::UiDocument::parse("card.svelte", doc).unwrap();
    assert_eq!(parsed.material_assets(), ["fx/burn.ogt"]);
    let mut ui = Ui::new(UiDesc::default());
    let handle = og_draw::UiMaterialHandle(7);
    let mut defaults = [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS];
    defaults[1] = [1.0, 2.0, 3.0, 4.0];
    ui.define_material("fx/burn.ogt", handle, defaults);
    ui.define_document(parsed);
    let burn = SourceId::new("test", "burn");
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Signal {
            source: burn.clone(),
            initial: serde_json::json!(0.2),
            readable: false,
        })
        .unwrap();
    ui.mount_document(
        SourceId::new("test", "card"),
        "card",
        std::collections::BTreeMap::from([("burn".to_string(), burn.clone())]),
        std::collections::BTreeMap::new(),
        &crate::bind::NoEnv,
    )
    .unwrap();
    let layer = |ui: &mut Ui| {
        for _ in 0..2 {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
        }
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
        ui.frame()
            .items
            .iter()
            .find_map(|it| match &it.cmd {
                crate::DrawCmd::BeginLayer {
                    material: og_draw::UiMaterial::Custom(h),
                    params,
                    ..
                } if *h == handle => Some(*params),
                _ => None,
            })
            .expect("the card composites through the asset")
    };
    let p = layer(&mut ui);
    assert_eq!(p[0], [0.2, 0.0, 0.0, 0.0]);
    assert_eq!(
        p[1],
        [6.0, 2.0, 3.0, 4.0],
        "written numbers over the defaults"
    );
    ui.docs
        .state_mut()
        .memory
        .reactive
        .execute(Command::Set {
            source: burn,
            value: serde_json::json!(0.8),
            force: false,
        })
        .unwrap();
    assert_eq!(layer(&mut ui)[0][0], 0.8, "the store drives the parameter");
    assert!(
        ui.world.declared.kept_effects.values().any(|(_, m)| matches!(
            m,
            Some(crate::world::MetaDesc { kind: crate::world::MetaKind::Material { material: og_draw::UiMaterial::Custom(h), params }, .. }) if *h == handle && params[0][0] == 0.25
        )),
        "og-overlay effect=\"material\" takes the asset"
    );
}
