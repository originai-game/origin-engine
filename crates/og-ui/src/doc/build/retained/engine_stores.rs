//! The engine's stores (docs/ui.md §2): what every mounted document reads
//! without its mount binding it. `$ui` is the device, the surface and the
//! action bar; `$captions` the lines the engine is saying now; `$toasts`
//! the toasts up. The engine draws none of them: a document does.

use super::*;
use crate::doc::{CAPTIONS_STORE, NO_ROWS, TOASTS_STORE, UI_STORE};
use crate::reactive::session::Command;
use serde_json::Map;

/// The handler a document names to dismiss the toast of its row
/// (`onclick={dismissToast}` inside `{#each $toasts as t (t.id)}`): the
/// engine answers it, and the game never sees it.
pub const DISMISS_TOAST: &str = "dismissToast";

impl Ui {
    /// Binds the engine's stores into a mount's inputs, each under its
    /// name unless the mount binds its own.
    pub(crate) fn bind_engine_stores(&mut self, inputs: &mut BTreeMap<String, SourceId>) {
        if !inputs.contains_key("ui") {
            self.publish_ui_store();
            inputs.insert("ui".into(), UI_STORE.clone());
        }
        for (name, source) in [("captions", &*CAPTIONS_STORE), ("toasts", &*TOASTS_STORE)] {
            if !inputs.contains_key(name) {
                self.declare_rows(source);
                inputs.insert(name.into(), source.clone());
            }
        }
        // What an `{#each}` over a list prop left out iterates.
        self.declare_rows(&NO_ROWS);
    }

    /// Declares the collection `source`, empty, unless it is there.
    fn declare_rows(&mut self, source: &SourceId) {
        let sources = &self.docs.state().memory.reactive.sources;
        if sources.keyed(source).is_ok() {
            return;
        }
        let command = Command::Collection {
            source: source.clone(),
            key: "id".into(),
            items: Vec::new(),
        };
        if let Err(error) = self.docs.execute(command) {
            self.out
                .warnings_mut()
                .push(format!("${} store: {error}", source.id));
        }
    }

    /// Makes the collection `source` hold `rows`, in order, each with a
    /// key no other has in its collection's key field. A row that stays
    /// keeps its place in the collection and is patched only in the fields
    /// that changed, so only what changed is built again.
    pub(super) fn set_rows(&mut self, source: &SourceId, rows: &[Map<String, Value>]) {
        let Ok(held) = self.docs.state().memory.reactive.sources.keyed(source) else {
            return;
        };
        let field = held.key_field().to_owned();
        let id = |row: &Map<String, Value>| {
            row.get(&field)
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned()
        };
        if held.keys().len() == rows.len()
            && rows
                .iter()
                .zip(held.keys())
                .all(|(row, key)| id(row) == *key && held.row(key) == Some(row))
        {
            return;
        }
        let mut commands = Vec::new();
        for key in held.keys() {
            if !rows.iter().any(|row| id(row) == *key) {
                commands.push(Command::Remove {
                    source: source.clone(),
                    key: key.clone(),
                });
            }
        }
        // From the last row back, each goes before the one after it.
        let mut before = None;
        for row in rows.iter().rev() {
            let key = id(row);
            match held.row(&key) {
                Some(was) => {
                    commands.push(Command::Move {
                        source: source.clone(),
                        key: key.clone(),
                        before: before.clone(),
                    });
                    let fields: Map<String, Value> = row
                        .iter()
                        .filter(|(name, value)| was.get(*name) != Some(value))
                        .map(|(name, value)| (name.clone(), value.clone()))
                        .collect();
                    if !fields.is_empty() {
                        commands.push(Command::Patch {
                            source: source.clone(),
                            key: key.clone(),
                            fields,
                        });
                    }
                }
                None => commands.push(Command::Insert {
                    source: source.clone(),
                    row: row.clone(),
                    before: before.clone(),
                }),
            }
            before = Some(key);
        }
        for command in commands {
            if let Err(error) = self.docs.execute(command) {
                self.out
                    .warnings_mut()
                    .push(format!("${} store: {error}", source.id));
                return;
            }
        }
    }

    /// The lines the engine is saying now, in order, each `(id, text)`
    /// with an id no other has: `$captions` becomes them.
    pub fn set_captions(&mut self, lines: &[(String, String)]) {
        let rows: Vec<_> = lines
            .iter()
            .map(|(id, text)| {
                [
                    ("id".to_owned(), json!(id)),
                    ("text".to_owned(), json!(text)),
                ]
                .into_iter()
                .collect()
            })
            .collect();
        self.declare_rows(&CAPTIONS_STORE);
        self.set_rows(&CAPTIONS_STORE, &rows);
    }

    /// `$toasts` becomes the toasts up, oldest first: `{ id, text, tone,
    /// leaving }`, the tone by name and `leaving` while it goes.
    pub(crate) fn publish_toasts(&mut self) {
        let rows: Vec<_> = self
            .interaction
            .state
            .toasts
            .iter()
            .map(|t| {
                [
                    ("id".to_owned(), json!(t.id.to_string())),
                    ("text".to_owned(), json!(t.text)),
                    (
                        "tone".to_owned(),
                        json!(crate::Tone::from_index(t.tone).name()),
                    ),
                    ("leaving".to_owned(), json!(t.leaving != 0)),
                ]
                .into_iter()
                .collect()
            })
            .collect();
        self.declare_rows(&TOASTS_STORE);
        self.set_rows(&TOASTS_STORE, &rows);
    }

    /// Answers the documents' `dismissToast` events: each dismisses its
    /// row's toast, and none reaches the game.
    pub(crate) fn answer_dismiss_toast(&mut self, events: &mut Vec<DocEvent>) {
        let mut dismissed = Vec::new();
        events.retain(|e| {
            let answered = e.kind == "emit" && e.name == DISMISS_TOAST;
            if answered {
                dismissed.push(e.row_key.clone());
            }
            !answered
        });
        for key in dismissed {
            if let Ok(id) = key.parse() {
                self.dismiss_toast(id);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::UiDocument;
    use crate::bind::NoEnv;

    const SAID: &str = r#"<meta name="component" content="said">
<column>{#each $captions as c (c.id)}<text>{c.text}</text>{/each}</column>"#;

    /// Sets the captions, builds a frame and answers the texts drawn, in
    /// tree order.
    fn show(ui: &mut Ui, lines: &[(&str, &str)]) -> Vec<String> {
        let lines: Vec<_> = lines
            .iter()
            .map(|(id, text)| (id.to_string(), text.to_string()))
            .collect();
        ui.set_captions(&lines);
        ui.begin(
            crate::InputFrame::default(),
            crate::Viewport::new(640.0, 480.0),
        );
        ui.build_retained_documents(&NoEnv).unwrap();
        let warnings = ui.end().warnings.clone();
        assert!(warnings.is_empty(), "{warnings:?}");
        ui.probe_elements()
            .into_iter()
            .filter(|e| e.kind == "label")
            .map(|e| e.text)
            .collect()
    }

    /// Ways this could fail: a mount not given `$captions`; a line that
    /// stays losing its place when one joins before it; a changed text
    /// (a reloaded timeline's key) kept as it was; a line that went still
    /// drawn; a mount's own `captions` replaced by the engine's.
    #[test]
    fn every_mount_reads_the_captions_the_engine_is_saying() {
        let mut ui = Ui::default();
        ui.define_document(UiDocument::parse("said.svelte", SAID).unwrap());
        let mount = |ui: &mut Ui, id: &str, inputs: BTreeMap<String, SourceId>| {
            ui.mount_document(
                SourceId::new("test", id),
                "said",
                inputs,
                BTreeMap::new(),
                &NoEnv,
            )
            .unwrap();
        };
        mount(&mut ui, "said", BTreeMap::new());
        assert_eq!(show(&mut ui, &[]), [""; 0]);
        assert_eq!(show(&mut ui, &[("a", "one"), ("b", "two")]), ["one", "two"]);
        // One moved, one reworded, one new before a line that stays.
        assert_eq!(
            show(&mut ui, &[("b", "two!"), ("c", "three"), ("a", "one")]),
            ["two!", "three", "one"]
        );
        assert_eq!(show(&mut ui, &[("c", "three")]), ["three"]);
        // A mount that binds its own `captions` reads its own.
        let own = SourceId::new("test", "own");
        ui.docs
            .execute(Command::Collection {
                source: own.clone(),
                key: "id".into(),
                items: vec![
                    [
                        ("id".to_owned(), json!("x")),
                        ("text".to_owned(), json!("mine")),
                    ]
                    .into_iter()
                    .collect(),
                ],
            })
            .unwrap();
        ui.unmount_document(&SourceId::new("test", "said"));
        mount(&mut ui, "own", [("captions".to_owned(), own)].into());
        assert_eq!(show(&mut ui, &[("c", "three")]), ["mine"]);
    }

    const TOASTS: &str = r#"<meta name="component" content="toasts">
<column>{#each $toasts as t (t.id)}<button onclick={dismissToast}>{t.text} {t.tone} {t.leaving ? "leaving" : "up"}</button>{/each}</column>
<style>button { width: 200px; height: 30px; }</style>"#;

    /// One frame with `input`; answers the toasts drawn (their text, tone
    /// and whether they are leaving) and the events the game sees.
    fn frame(ui: &mut Ui, input: crate::InputFrame) -> (Vec<String>, Vec<DocEvent>) {
        ui.begin(input, crate::Viewport::new(640.0, 480.0));
        ui.build_retained_documents(&NoEnv).unwrap();
        let warnings = ui.end().warnings.clone();
        assert!(warnings.is_empty(), "{warnings:?}");
        let up = ui
            .probe_elements()
            .into_iter()
            .filter(|e| e.kind == "button")
            .map(|e| e.text)
            .collect();
        (up, ui.take_doc_events())
    }

    /// Ways this could fail: more than three up, or the newest leaving
    /// first; a toast not leaving on its time, or gone before its exit
    /// plays; a press on one reaching the game instead of dismissing it,
    /// or dismissing another; a restore not bringing them back.
    #[test]
    fn toasts_are_rows_a_document_shows_and_dismisses() {
        let mut ui = Ui::default();
        ui.define_document(UiDocument::parse("toasts.svelte", TOASTS).unwrap());
        ui.mount_document(
            SourceId::new("test", "toasts"),
            "toasts",
            BTreeMap::new(),
            BTreeMap::new(),
            &NoEnv,
        )
        .unwrap();
        frame(&mut ui, crate::InputFrame::default());
        for n in 0..5 {
            let tone = if n == 4 {
                crate::Tone::Danger
            } else {
                crate::Tone::Neutral
            };
            ui.toast(
                &format!("Saved{n}"),
                crate::ToastDesc {
                    tone,
                    ..Default::default()
                },
            );
        }
        // Shown the tick they are queued; the two oldest are leaving.
        let (up, _) = frame(&mut ui, crate::InputFrame::default());
        assert_eq!(
            up,
            [
                "Saved0 neutral leaving",
                "Saved1 neutral leaving",
                "Saved2 neutral up",
                "Saved3 neutral up",
                "Saved4 danger up"
            ]
        );
        for _ in 0..20 {
            frame(&mut ui, crate::InputFrame::default());
        }
        let (up, _) = frame(&mut ui, crate::InputFrame::default());
        assert_eq!(
            up,
            ["Saved2 neutral up", "Saved3 neutral up", "Saved4 danger up"]
        );
        let saved = ui.save_state();
        // A press dismisses the toast pressed, and the game sees nothing.
        let at = ui
            .probe_elements()
            .into_iter()
            .find(|e| e.kind == "button" && e.text.starts_with("Saved3"))
            .map(|e| og_core::glam::Vec2::new(e.x + e.width / 2.0, e.y + e.height / 2.0))
            .unwrap();
        let press = |pressed, released| crate::InputFrame {
            pointer: at,
            pointer_inside: true,
            down: pressed,
            pressed,
            released,
            ..crate::InputFrame::default()
        };
        frame(&mut ui, press(true, false));
        let (_, events) = frame(&mut ui, press(false, true));
        assert!(events.iter().all(|e| e.name != DISMISS_TOAST), "{events:?}");
        let (up, _) = frame(&mut ui, crate::InputFrame::default());
        assert_eq!(
            up,
            [
                "Saved2 neutral up",
                "Saved3 neutral leaving",
                "Saved4 danger up"
            ]
        );
        // The neutral ones leave after 3.2 s, the danger one after 6 s.
        for _ in 0..240 {
            frame(&mut ui, crate::InputFrame::default());
        }
        let (up, _) = frame(&mut ui, crate::InputFrame::default());
        assert_eq!(up, ["Saved4 danger up"]);
        for _ in 0..200 {
            frame(&mut ui, crate::InputFrame::default());
        }
        assert_eq!(frame(&mut ui, crate::InputFrame::default()).0, [""; 0]);
        // A restore brings back what was up.
        ui.restore_state(&saved).unwrap();
        frame(&mut ui, crate::InputFrame::default());
        let (up, _) = frame(&mut ui, crate::InputFrame::default());
        assert_eq!(
            up,
            ["Saved2 neutral up", "Saved3 neutral up", "Saved4 danger up"]
        );
    }
}
