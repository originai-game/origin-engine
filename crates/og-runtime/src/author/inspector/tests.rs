use super::*;

fn file(source: &str, text: &str, lines: Vec<usize>) -> og_probe::SavedFile {
    og_probe::SavedFile {
        source: source.into(),
        written: String::new(),
        text: text.into(),
        lines,
    }
}

#[test]
fn a_save_that_wrote_no_file_hands_its_changed_texts_to_the_next_frame_once() {
    let ins = &mut Inspector::new(60);
    // An unchanged scene is not handed on; a second save of the same
    // source replaces the first's text.
    assert!(hand_over(
        ins,
        &[
            file("assets/scenes/a.ogt", "one", vec![3]),
            file("assets/scenes/b.ogt", "same", vec![])
        ]
    ));
    assert!(hand_over(
        ins,
        &[file("assets/scenes/a.ogt", "two", vec![3, 4])]
    ));
    assert_eq!(
        saved_of(ins),
        Some(serde_json::json!({ "scenes": [{ "source": "assets/scenes/a.ogt", "text": "two" }] }))
    );
    assert_eq!(saved_of(ins), None, "taken once");
    assert!(!hand_over(
        ins,
        &[file("assets/scenes/b.ogt", "same", vec![])]
    ));
    assert_eq!(saved_of(ins), None);
}
