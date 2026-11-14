//! The engine's stores (docs/ui.md §2): what every mounted document reads
//! without its mount binding it. `$ui` is the device, the surface and the
//! action bar; `$captions` the lines the engine is saying now.

use super::*;
use crate::doc::{CAPTIONS_STORE, UI_STORE};
use crate::reactive::session::Command;

impl Ui {
    /// Binds the engine's stores into a mount's inputs, each under its
    /// name unless the mount binds its own.
    pub(crate) fn bind_engine_stores(&mut self, inputs: &mut BTreeMap<String, SourceId>) {
        if !inputs.contains_key("ui") {
            self.publish_ui_store();
            inputs.insert("ui".into(), UI_STORE.clone());
        }
        if !inputs.contains_key("captions") {
            self.publish_captions();
            inputs.insert("captions".into(), CAPTIONS_STORE.clone());
        }
    }

    /// Declares `$captions`, empty, unless it is there.
    fn publish_captions(&mut self) {
        let sources = &self.docs.state().memory.reactive.sources;
        if sources.keyed(&CAPTIONS_STORE).is_ok() {
            return;
        }
        let command = Command::Collection {
            source: CAPTIONS_STORE.clone(),
            key: "id".into(),
            items: Vec::new(),
        };
        if let Err(error) = self.docs.execute(command) {
            self.out
                .warnings_mut()
                .push(format!("$captions store: {error}"));
        }
    }

    /// The lines the engine is saying now, in order, each `(id, text)`
    /// with an id no other has: `$captions` becomes them. A line that
    /// stays keeps its row, so only what changed is built again.
    pub fn set_captions(&mut self, lines: &[(String, String)]) {
        self.publish_captions();
        let source = CAPTIONS_STORE.clone();
        let Ok(held) = self.docs.state().memory.reactive.sources.keyed(&source) else {
            return;
        };
        let text = |key: &str| held.row(key).and_then(|row| row.get("text")).cloned();
        if held.keys().len() == lines.len()
            && lines
                .iter()
                .zip(held.keys())
                .all(|((id, said), key)| id == key && text(key) == Some(json!(said)))
        {
            return;
        }
        let mut commands = Vec::new();
        for key in held.keys() {
            if !lines.iter().any(|(id, _)| id == key) {
                commands.push(Command::Remove {
                    source: source.clone(),
                    key: key.clone(),
                });
            }
        }
        // From the last line back, each goes before the one after it.
        let mut before = None;
        for (id, said) in lines.iter().rev() {
            commands.push(match text(id) {
                Some(_) => Command::Move {
                    source: source.clone(),
                    key: id.clone(),
                    before: before.clone(),
                },
                None => Command::Insert {
                    source: source.clone(),
                    row: [
                        ("id".to_owned(), json!(id)),
                        ("text".to_owned(), json!(said)),
                    ]
                    .into_iter()
                    .collect(),
                    before: before.clone(),
                },
            });
            if text(id).is_some_and(|held| held != json!(said)) {
                commands.push(Command::Patch {
                    source: source.clone(),
                    key: id.clone(),
                    fields: [("text".to_owned(), json!(said))].into_iter().collect(),
                });
            }
            before = Some(id.clone());
        }
        for command in commands {
            if let Err(error) = self.docs.execute(command) {
                self.out
                    .warnings_mut()
                    .push(format!("$captions store: {error}"));
                return;
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
}
