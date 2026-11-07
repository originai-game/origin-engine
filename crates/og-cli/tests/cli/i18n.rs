use super::*;

fn copy(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for e in std::fs::read_dir(from).unwrap() {
        let e = e.unwrap();
        if e.path().is_dir() {
            copy(&e.path(), &to.join(e.file_name()));
        } else {
            std::fs::copy(e.path(), to.join(e.file_name())).unwrap();
        }
    }
}

/// `og i18n extract` lists every key the dialogues and quests show (the
/// speaker's name, lines, choices, a quest's title, objectives and
/// rewards) with the arguments passed; `--check` exits 1 naming the locale
/// that lacks a key and passes on complete tables; `--write` starts a base
/// table holding every key, after which the check passes.
///
/// Ways it could fail: a quest's or a speaker's key left out, a missing
/// key reported against the wrong locale, or a written stub that does not
/// parse or lacks an argument the line passes.
#[test]
fn i18n_extract_finds_story_keys_and_missing_translations() {
    let dir = scratch_root();
    let fixture = repo().join("packages/engine/tests/fixtures/story");
    copy(&fixture.join("assets"), &dir.join("assets"));

    let out = og()
        .args([
            "i18n",
            "extract",
            dir.to_str().unwrap(),
            "--json",
            "--check",
        ])
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(0), "{}", text(&out));
    let v: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    for k in [
        "npc.smith",
        "smith.hello",
        "quest.rescue",
        "quest.rescue.wolves",
        "item.gold",
    ] {
        assert!(
            v["keys"].get(k).is_some(),
            "{k} not extracted: {}",
            v["keys"]
        );
    }
    assert_eq!(
        v["keys"]["smith.hello"]["args"],
        serde_json::json!(["gold"])
    );
    assert_eq!(v["complete"], true);

    // Drop one Japanese line: only ja is incomplete, and it is named.
    let ja = dir.join("assets/strings/ja.ogt");
    let kept: String = std::fs::read_to_string(&ja)
        .unwrap()
        .lines()
        .filter(|l| !l.starts_with("string \"smith.calm\""))
        .map(|l| format!("{l}\n"))
        .collect();
    std::fs::write(&ja, kept).unwrap();
    let out = og()
        .args(["i18n", "extract", dir.to_str().unwrap(), "--check"])
        .output()
        .unwrap();
    let t = text(&out);
    assert_eq!(out.status.code(), Some(1), "{t}");
    assert!(t.contains("ja assets/strings/ja.ogt: 1 missing"), "{t}");
    assert!(t.contains("missing smith.calm"), "{t}");
    assert!(
        t.contains("en (base) assets/strings/en.ogt: complete"),
        "{t}"
    );

    // Without tables, --write starts a base table that has every key.
    std::fs::remove_dir_all(dir.join("assets/strings")).unwrap();
    let out = og()
        .args([
            "i18n",
            "extract",
            dir.to_str().unwrap(),
            "--write",
            "--check",
        ])
        .output()
        .unwrap();
    let t = text(&out);
    assert_eq!(out.status.code(), Some(0), "{t}");
    let en = std::fs::read_to_string(dir.join("assets/strings/en.ogt")).unwrap();
    assert!(
        en.contains("string \"smith.hello\" \"Steel's not cheap. You've {gold} gold.\""),
        "{en}"
    );
    let _ = std::fs::remove_dir_all(&dir);
}
