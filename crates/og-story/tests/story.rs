//! og-story's contract: validation that names the line and the fix, a
//! deterministic walk on a seed, saves that survive edits and warn on the
//! one breaking edit, quests, checks and timers.

use og_core::Rng;
use og_story::{Dialogue, Event, Library, Quest, Story, Val, Wait};
use std::collections::BTreeMap;
use std::sync::Arc;

const SMITH: &str =
    include_str!("../../../packages/engine/tests/fixtures/story/assets/story/smith.yarn");

const RESCUE: &str = r#"type quest
version 1
---
id rescue
title "quest.rescue"
category side
global wolvesFled false
reward "item.gold" 50 {
  rarity common
}
stage hunt {
  objective wolves "quest.rescue.wolves" {
    target 3
    marker 12 0 4
  }
  objective sword "quest.rescue.sword" {
    optional
  }
  fail "wolvesFled"
  next report
}
stage report {
  objective talk "quest.rescue.talk" {
    when "seen(smith.bought)"
  }
}
"#;

fn lib(smith: &str) -> Library {
    let mut l = Library::default();
    let d = Dialogue::parse(smith).unwrap();
    l.dialogues.insert(d.name.clone(), Arc::new(d));
    let q = Quest::parse(RESCUE).unwrap();
    l.quests.insert(q.id.clone(), Arc::new(q));
    l
}

fn line(s: &Story, conv: u32) -> String {
    s.convs[&conv].history.last().unwrap().key.clone()
}

#[test]
fn lines_choices_checks_quests_and_events() {
    let l = lib(SMITH);
    let mut s = Story::new(Rng::new(1));
    s.start_quest(&l, "rescue").unwrap();
    let c = s
        .start(&l, "smith", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    assert_eq!(s.convs[&c].wait, Wait::Choices);
    assert_eq!(s.convs[&c].locals["mood"], Val::Num(1.0));
    assert_eq!(
        s.convs[&c].history[0].args,
        [("gold".into(), Val::Num(10.0))]
    );
    let d = l.dialogues["smith"].clone();
    let node = d.node(&s.convs[&c].node).unwrap();
    // buy: enabled (10 gold); secret: charm 3 < 5, hidden; haggle: 12 - 3 needs 9+, 60 %.
    assert_eq!(
        s.choice_state(&l, &d, c, &node.choices[0]),
        (true, true, 1.0)
    );
    assert_eq!(
        s.choice_state(&l, &d, c, &node.choices[1]),
        (true, true, 0.6)
    );
    assert_eq!(
        s.choice_state(&l, &d, c, &node.choices[2]),
        (false, false, 0.0)
    );
    assert!(s.choose(&l, c, 2).is_err());
    s.choose(&l, c, 0).unwrap();
    assert_eq!(s.get(&l, "gold"), Some(Val::Num(0.0)));
    assert_eq!(line(&s, c), "smith.thanks");
    assert_eq!(s.convs[&c].history[0].chosen, Some((0, "smith.buy".into())));
    assert!(
        s.convs[&c]
            .visited
            .contains(&format!("{}/smith.buy", node.id))
    );
    // The steps after a line run when it is advanced.
    assert!(!s.quest("rescue").unwrap().counts.contains_key("sword"));
    s.advance(&l, c).unwrap();
    assert_eq!(s.quest("rescue").unwrap().counts["sword"], 1);
    assert!(s.events.contains(&Event::Emit {
        conv: c,
        name: "give_sword".into(),
        arg: "1".into()
    }));
    assert_eq!(s.convs[&c].wait, Wait::Ended);
    assert!(s.events.contains(&Event::Ended { conv: c }));
    // A second conversation: buying is disabled now (no gold), seen lines are seen.
    let c2 = s
        .start(&l, "smith", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    assert!(s.convs[&c2].history[0].seen);
    assert_eq!(
        s.choice_state(&l, &d, c2, &node.choices[0]),
        (true, false, 1.0)
    );
    assert!(s.choose(&l, c2, 0).is_err());
}

#[test]
fn saves_round_trip_and_survive_edits() {
    let l = lib(SMITH);
    let mut s = Story::new(Rng::new(9).fork("story"));
    s.start_quest(&l, "rescue").unwrap();
    let c = s
        .start(&l, "smith", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    s.choose(&l, c, 0).unwrap();
    let mut a = Vec::new();
    s.save(&mut a);
    let back = Story::load(&a).unwrap();
    let mut b = Vec::new();
    back.save(&mut b);
    assert_eq!(a, b, "equal stories save equal bytes");
    for cut in [0, 4, 5, a.len() / 2, a.len() - 1] {
        assert!(Story::load(&a[..cut]).is_err(), "cut at {cut}");
    }

    // An edit that keeps the ids: a line before the shown one moves it
    // within its node, and the conversation finds it by its key.
    let moved = SMITH.replace(
        "smith: Mind the edge.",
        "Unrelated #line:unrelated\n<<set $mood = 2>>\nsmith: Mind the edge.",
    );
    let l2 = lib(&moved);
    let mut r = Story::load(&a).unwrap();
    r.advance(&l2, c).unwrap();
    assert_eq!(r.convs[&c].wait, Wait::Ended);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

/// Missing dialogue ids refuse to restart; quest migration stays unchanged.
#[test]
fn a_removed_line_refuses_to_replay_commands_and_quest_stage_migration_is_unchanged() {
    let l = lib(SMITH);
    let mut s = Story::new(Rng::new(9));
    let c = s
        .start(&l, "smith", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    s.choose(&l, c, 0).unwrap();
    assert_eq!(line(&s, c), "smith.thanks");
    let mut a = Vec::new();
    s.save(&mut a);

    let renamed = SMITH.replace("#line:smith.thanks", "#line:smith.paid");
    let l2 = lib(&renamed);
    let mut r = Story::load(&a).unwrap();
    let e = r.resolve(&l2, c).unwrap_err();
    assert!(e.contains("smith.thanks") && e.contains("stable id"), "{e}");
    assert_eq!(line(&r, c), "smith.thanks");

    // A quest stage renamed: warned by name, back to the first stage.
    let mut q = Story::new(Rng::new(1));
    q.start_quest(&l, "rescue").unwrap();
    q.progress(&l, "rescue", "wolves", 3);
    assert_eq!(q.quest("rescue").unwrap().stage, "report");
    let mut qa = Vec::new();
    q.save(&mut qa);
    let mut l3 = lib(SMITH);
    let edited = Quest::parse(
        &RESCUE
            .replace("stage report {", "stage debrief {")
            .replace("next report", "next debrief"),
    )
    .unwrap();
    l3.quests.insert("rescue".into(), Arc::new(edited));
    let mut qr = Story::load(&qa).unwrap();
    qr.settle(&l3);
    assert!(
        qr.warnings
            .iter()
            .any(|w| w.contains("'report'") && w.contains("'hunt'")),
        "{:?}",
        qr.warnings
    );
    assert_eq!(qr.quest("rescue").unwrap().stage, "hunt");
}

/// og.flow's pacing (StartDesc.hold): a node with the hold header holds
/// on entry, before its first step runs; a logic node without it runs
/// through; a held conversation saves and loads as held, and the loaded
/// one goes on the same way.
#[test]
fn a_hold_header_holds_each_node_that_has_it_and_survives_a_save() {
    let mut l = Library::default();
    let d = Dialogue::parse(FILM).unwrap();
    l.dialogues.insert(d.name.clone(), Arc::new(d));
    let mut s = Story::new(Rng::new(3).fork("story"));
    let c = s
        .start_held(
            &l,
            "film",
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            "og-flow",
        )
        .unwrap();
    let d = l.dialogues["film"].clone();
    let at = |s: &Story| {
        let block = d.node(&s.convs[&c].node).unwrap();
        (block.title.clone(), s.convs[&c].wait)
    };
    assert_eq!(at(&s), ("kitchen".into(), Wait::Held));
    assert_eq!(
        s.get(&l, "count"),
        Some(Val::Num(0.0)),
        "nothing in the node runs before it is let go"
    );
    assert!(s.choose(&l, c, 0).is_err(), "no choice while it holds");
    s.advance(&l, c).unwrap();
    assert_eq!(at(&s), ("kitchen".into(), Wait::Choices));
    assert_eq!(s.get(&l, "count"), Some(Val::Num(1.0)));
    s.choose(&l, c, 0).unwrap();
    // `branch` has no og-flow header: it runs through to hall, which holds.
    assert_eq!(at(&s), ("hall".into(), Wait::Held));
    let mut a = Vec::new();
    s.save(&mut a);
    let mut back = Story::load(&a).unwrap();
    assert_eq!(at(&back), ("hall".into(), Wait::Held));
    for st in [&mut s, &mut back] {
        st.advance(&l, c).unwrap();
        assert_eq!(st.convs[&c].wait, Wait::Ended);
        assert_eq!(st.get(&l, "count"), Some(Val::Num(11.0)));
    }
}

/// A save from before og.film became og.flow (version 9) holds its
/// conversation on `og-film`, a header its upgraded dialogue no longer
/// carries: loaded, it holds on `og-flow`, so the flow it paces stops at
/// the next node instead of running through it. A version 10 save's hold
/// is read as written. Ways this could fail: the old header kept (the
/// conversation runs through hall and ends), or a current save's header
/// rewritten too.
#[test]
fn a_save_from_before_og_flow_holds_on_the_header_its_dialogue_now_carries() {
    let mut l = Library::default();
    let d = Dialogue::parse(FILM).unwrap();
    l.dialogues.insert(d.name.clone(), Arc::new(d));
    let mut s = Story::new(Rng::new(3).fork("story"));
    // No node carries `og-film`: this conversation runs to kitchen's choices.
    let c = s
        .start_held(
            &l,
            "film",
            BTreeMap::new(),
            BTreeMap::new(),
            None,
            "og-film",
        )
        .unwrap();
    assert_eq!(s.convs[&c].wait, Wait::Choices);
    let mut now = Vec::new();
    s.save(&mut now);
    assert_eq!(now[4], og_story::save::SAVE_VERSION);
    let mut before = now.clone();
    before[4] = 9;
    let mut old = Story::load(&before).unwrap();
    assert_eq!(old.convs[&c].hold, "og-flow");
    old.choose(&l, c, 0).unwrap();
    let d = l.dialogues["film"].clone();
    assert_eq!(
        (
            d.node(&old.convs[&c].node).unwrap().title.as_str(),
            old.convs[&c].wait
        ),
        ("hall", Wait::Held)
    );
    let mut current = Story::load(&now).unwrap();
    assert_eq!(current.convs[&c].hold, "og-film");
    current.choose(&l, c, 0).unwrap();
    assert_eq!(current.convs[&c].wait, Wait::Ended);
}

const FILM: &str = r#"#og-profile:yarn-spinner-3.2.2
#og-name:film
title: kitchen
og-flow: segments/kitchen.ogt
---
<<declare $brave = false>>
<<declare $count = 0>>
<<set $count = $count + 1>>
-> Take the knife #line:kitchen.knife
    <<set $brave = true>>
    <<jump branch>>
-> Open the door #line:kitchen.door #default
    <<jump branch>>
===
title: branch
---
<<if $brave>>
    <<jump hall>>
<<else>>
    <<jump yard>>
<<endif>>
===
title: hall
og-flow: segments/hall.ogt
---
<<set $count = $count + 10>>
===
title: yard
og-flow: segments/yard.ogt
---
===
"#;
