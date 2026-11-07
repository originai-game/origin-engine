//! Public story views localize before producing shared scalar text spans.
use og_runtime::*;
use serde_json::{Value, json};
use std::rc::Rc;

struct Disk;
impl Hooks for Disk {
    fn dispatch(&self, _: &Dispatch) -> Result<String, String> {
        Ok(String::new())
    }
    fn read_asset(&self, path: &str) -> Fetch<Vec<u8>> {
        match std::fs::read(path) {
            Ok(bytes) => Fetch::Ready(bytes),
            Err(error) => Fetch::Failed(error.to_string()),
        }
    }
}

fn value(text: String) -> Value {
    let result: Value = serde_json::from_str(&text).unwrap();
    assert_eq!(result["ok"], true, "{text}");
    result["value"].clone()
}

#[test]
fn locale_switch_rebuilds_plain_text_and_scalar_spans_without_advancing() {
    let root = std::path::PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("yarn-localized-spans");
    std::fs::create_dir_all(root.join("assets")).unwrap();
    let translated = "Мэй: 🦊 [wave speed=2 active=true]Привет 🌙[/wave] [plural value={n} one=яблоко few=яблока many=яблок other=яблока/]";
    std::fs::write(
        root.join("assets/ru.ogt"),
        format!(
            "type strings\nversion 1\n---\nlocale \"ru\"\nstring \"greeting\" {}\n",
            serde_json::to_string(translated).unwrap(),
        ),
    )
    .unwrap();
    let report = og_cook::cook_project(&root, &og_cook::CookOptions::default()).unwrap();
    assert!(report.ok(), "{:?}", report.errors());
    set_hooks(Rc::new(Disk));
    og_host_begin_bundle();
    let game = og_game_game(Some(json!({"seed": 19}).to_string()));
    let engine = value(og_engine_create(game, json!({"root": root}).to_string()))
        .as_u64()
        .unwrap() as u32;
    let script = format!(
        r#"#og-profile:{}
#og-name:localized
title: Start
---
<<declare $n = 2>>
<<declare $charm = 0>>
Mae: 🦊 [wave speed=2 active={{true}}]Hi 🌙[/wave] [plural value={{$n}} one=apple other=apples/] {{0.1}} \{{literal\}} #line:greeting
-> Try #line:try #check:passive:charm:dice(6) #reason:blocked #reason-arg:n=dice(1000)
    <<stop>>
===
"#,
        og_story::yarn::PROFILE
    );
    value(og_story_parse(engine, script));
    let conv = value(og_story_start(
        engine,
        Some(json!({"dialogue": "localized"}).to_string()),
    ))
    .as_u64()
    .unwrap() as u32;
    let read = || serde_json::from_str::<Value>(&og_story_view(engine, conv)).unwrap();
    let english = read();
    assert_eq!(english["text"], "Mae: 🦊 Hi 🌙 apples 0.1 {literal}");
    let spans: Vec<og_ui::TextSpan> = serde_json::from_value(english["spans"].clone()).unwrap();
    assert_eq!(
        (spans[0].name.as_str(), spans[0].position, spans[0].length),
        ("character", 0, 5)
    );
    assert_eq!(
        (spans[1].name.as_str(), spans[1].position, spans[1].length),
        ("wave", 7, 4)
    );
    assert_eq!(
        spans[1].properties["active"],
        og_ui::TextSpanValue::Bool(true)
    );
    assert_eq!(english["choices"][0]["enabled"], false);
    // Other conversations consume the shared RNG, but cannot change an
    // already offered check target or its localized reason arguments.
    for _ in 0..3 {
        value(og_story_start(
            engine,
            Some(json!({"dialogue": "localized"}).to_string()),
        ));
        assert_eq!(read(), english);
    }
    value(og_i18n_load(engine, "ru.ogt".into()));
    let locale: Value = serde_json::from_str(&og_i18n_set_locale(engine, "ru".into())).unwrap();
    assert_eq!(locale["active"], "ru");
    let russian = read();
    assert_eq!(russian["text"], "Мэй: 🦊 Привет 🌙 яблока");
    assert_eq!(russian["seq"], english["seq"]);
    assert_eq!(russian["spans"][1]["position"], 7);
    assert_eq!(russian["spans"][1]["length"], 8);
    let locale: Value = serde_json::from_str(&og_i18n_set_locale(engine, "en".into())).unwrap();
    assert_eq!(locale["active"], "en");
    assert_eq!(read(), english);
    assert!(og_engine_destroy(engine));
}
