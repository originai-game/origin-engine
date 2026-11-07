use og_core::Rng;
use og_story::{Dialogue, Event, Library, Story, Val, Wait};
use std::collections::BTreeMap;
use std::sync::Arc;

fn library(body: &str) -> Library {
    let source = format!("#og-profile:yarn-3.0-og-1\n#og-name:test\n{body}");
    let dialogue = Dialogue::parse(&source).unwrap();
    let dialogue = Dialogue::decode(&dialogue.encode().unwrap()).unwrap();
    Library {
        dialogues: BTreeMap::from([("test".into(), Arc::new(dialogue))]),
        ..Default::default()
    }
}

fn start(l: &Library) -> (Story, u32) {
    let mut s = Story::new(Rng::new(19).fork("story"));
    let id = s
        .start(l, "test", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    (s, id)
}

fn restore(s: &Story) -> Story {
    let mut bytes = Vec::new();
    s.save(&mut bytes);
    Story::load(&bytes).unwrap()
}

#[test]
fn edited_lines_and_detour_stack_restore_without_repeating_commands() {
    let source = "title: Start\n---\n<<declare $gold = 0>>\n<<set $gold = $gold + 1>>\n<<reward>>\n<<detour Shop>>\nAfter #line:after\n===\ntitle: Shop\n---\nMoved #line:moved\nShop: Hello #line:hello\nShop: Goodbye #line:goodbye\n===";
    let l = library(source);
    let (mut s, id) = start(&l);
    s.advance(&l, id).unwrap();
    s.events.clear();
    let mut back = restore(&s);
    let edited = library(
        &source
            .replace("Moved #line:moved\n", "")
            .replace("Shop: Hello", "Unrelated #line:inserted\nShop: Hello")
            .replace("After #line:after", "Moved #line:moved\nAfter #line:after"),
    );
    back.resolve(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "hello");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "goodbye");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "moved");
    back.advance(&edited, id).unwrap();
    assert_eq!(back.convs[&id].history.last().unwrap().key, "after");
    assert_eq!(back.get(&edited, "gold"), Some(Val::Num(1.0)));
    assert!(!back.events.iter().any(|e| matches!(e, Event::Emit { .. })));
}

#[test]
fn settled_choice_cannot_award_twice_after_restore() {
    let l = library(
        "title: Start\n---\n<<declare $gold = 0>>\n-> Buy <<if $gold == 0>> #line:buy\n    <<set $gold = $gold + 1>>\n    <<reward>>\n    Bought #line:bought\n-> Leave #line:leave\n    <<stop>>\n===",
    );
    let (mut s, id) = start(&l);
    assert_eq!(s.convs[&id].wait, Wait::Choices);
    s.choose(&l, id, 0).unwrap();
    s.events.clear();
    let mut back = restore(&s);
    assert!(back.choose(&l, id, 0).is_err());
    assert_eq!(back.get(&l, "gold"), Some(Val::Num(1.0)));
    assert!(back.events.is_empty());
}

#[test]
fn weighted_line_groups_restore_the_story_rng() {
    let l = library(
        "title: Start\n---\nReady #line:ready\n=> A <<if true>> #line:a #weight:1\n=> B <<if true>> #line:b #weight:7\n=> Fallback #line:fallback #weight:100000\n=> Never <<if false>> #line:never #weight:100000\n<<jump Start>>\n===",
    );
    let (mut s, id) = start(&l);
    let mut back = restore(&s);
    for _ in 0..20 {
        s.advance(&l, id).unwrap();
        back.advance(&l, id).unwrap();
        assert_eq!(s.convs[&id].history.last(), back.convs[&id].history.last());
        assert!(!matches!(
            s.convs[&id].history.last().unwrap().key.as_str(),
            "fallback" | "never"
        ));
        assert_eq!(s.rng.save(), back.rng.save());
    }
}

#[test]
fn missing_line_id_and_unimplemented_features_are_errors() {
    for (body, expected) in [
        ("Hello", "#line:"),
        ("<<enum Mode>>", "unsupported Yarn feature"),
        ("-> Buy", "#line:"),
        ("Hello #line:a #line:b", "duplicate #line"),
        ("<<jump {$target}>>", "dynamic jump"),
        ("[wave]Hello[/wave] #line:a", "markup"),
    ] {
        let source =
            format!("#og-profile:yarn-3.0-og-1\n#og-name:test\ntitle: Start\n---\n{body}\n===");
        let error = Dialogue::parse(&source).unwrap_err();
        assert!(error.contains(expected), "{error}");
        assert!(error.contains("line 5"), "{error}");
    }
}

#[test]
fn branches_strings_and_nested_options_execute_only_the_selected_body() {
    let l = library(
        r#"title: Start
---
<<declare $name = "Kai $ and # literal">>
<<declare $gold = -2>>
<<if $gold > 0>>
Wrong #line:wrong
<<elseif $gold == -2 && !false>>
Hello {$name} #line:hello
-> Continue #line:continue
    -> Inner #line:inner
        <<set $gold to 7>>
        <<jump end>>
<<else>>
Wrong again #line:wrong_again
<<endif>>
===
title: end
---
Done #line:done
===
"#,
    );
    let (mut s, id) = start(&l);
    assert_eq!(
        s.convs[&id].history.last().unwrap().args,
        [("name".into(), Val::Str("Kai $ and # literal".into()))]
    );
    s.choose(&l, id, 0).unwrap();
    let mut back = restore(&s);
    back.choose(&l, id, 0).unwrap();
    assert_eq!(back.get(&l, "gold"), Some(Val::Num(7.0)));
    assert_eq!(back.convs[&id].history.last().unwrap().key, "done");
}
