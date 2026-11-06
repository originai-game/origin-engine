//! og-story's contract: validation that names the line and the fix, a
//! deterministic walk on a seed, saves that survive edits and warn on the
//! one breaking edit, quests, checks and timers.

use og_core::Rng;
use og_story::{Dialogue, Event, Library, Quest, Story, Val, Wait};
use std::collections::BTreeMap;
use std::sync::Arc;

const SMITH: &str = r#"type dialogue
version 1
---
name smith
start greet
speaker smith {
  name "npc.smith"
  portrait "art/smith.png"
  side right
}
var mood 0
global gold 10
global charm 3
node greet {
  say smith "smith.hello" {
    arg gold "gold"
  }
  set mood "mood + 1"
  choice "smith.buy" {
    if "gold >= 10"
    reason "smith.need_gold" {
      arg need "10 - gold"
    }
    kind trade
    set gold "gold - 10"
    goto bought
  }
  choice "smith.haggle" {
    check active charm 12
    goto bought
    fail angry
  }
  choice "smith.secret" {
    check passive charm 5
    hide
    goto bought
  }
  choice "smith.leave"
  timer 30 smith.leave
}
node bought {
  say smith "smith.thanks"
  quest progress rescue sword
  emit give_sword "mood"
}
node angry {
  pick {
    calm 1
    furious 1
  }
}
node calm {
  say smith "smith.calm"
  goto greet
}
node furious {
  say smith "smith.furious"
}
"#;

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
    // The greeting is shown with its choices; the steps after it ran.
    assert_eq!(s.convs[&c].wait, Wait::Choices);
    assert_eq!(s.convs[&c].locals["mood"], Val::Num(1.0));
    assert_eq!(
        s.convs[&c].history[0].args,
        [("gold".into(), Val::Num(10.0))]
    );
    let d = l.dialogues["smith"].clone();
    let node = d.node("greet").unwrap();
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
    assert!(s.convs[&c].visited.contains("greet/smith.buy"));
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
        "node bought {\n  say smith \"smith.thanks\"",
        "node bought {\n  set mood 2\n  say smith \"smith.thanks\"",
    );
    let l2 = lib(&moved);
    let mut r = Story::load(&a).unwrap();
    r.advance(&l2, c).unwrap();
    assert_eq!(r.convs[&c].wait, Wait::Ended);
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

/// The one breaking edit: renaming the node a save stands on. The story
/// warns, naming the id, and restarts the conversation at the dialogue's
/// start instead of landing on whatever node now has that place.
#[test]
fn a_renamed_node_warns_and_falls_back_to_the_start() {
    let l = lib(SMITH);
    let mut s = Story::new(Rng::new(9));
    let c = s
        .start(&l, "smith", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    s.choose(&l, c, 0).unwrap();
    assert_eq!(s.convs[&c].node, "bought");
    let mut a = Vec::new();
    s.save(&mut a);

    let renamed = SMITH
        .replace("node bought {", "node paid {")
        .replace("goto bought", "goto paid");
    let l2 = lib(&renamed);
    let mut r = Story::load(&a).unwrap();
    r.resolve(&l2, c).unwrap();
    assert_eq!(r.warnings.len(), 1, "{:?}", r.warnings);
    let w = &r.warnings[0];
    assert!(
        w.contains("'bought'") && w.contains("renamed or removed") && w.contains("'greet'"),
        "{w}"
    );
    assert_eq!(r.convs[&c].node, "greet");
    assert_eq!(line(&r, c), "smith.hello");
    assert_eq!(r.convs[&c].wait, Wait::Choices);

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
