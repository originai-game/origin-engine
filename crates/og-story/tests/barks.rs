//! og-story batch 2: barks (world conditions, priority, cooldowns per
//! entity and shared, cycles, once), objectives counted from events, voice
//! timing, and the version-2 save carrying all of it.

use og_core::Rng;
use og_story::{Dialogue, Event, Library, Quest, Story, Val};
use std::collections::BTreeMap;
use std::sync::Arc;

const GUARD: &str = r#"type dialogue
version 1
---
name guard
speaker guard {
  name "npc.guard"
}
global alert 0
bark yawn {
  topic idle
  speaker guard
  say "guard.yawn"
  say "guard.stretch"
  cooldown 100
}
bark hurt {
  topic idle
  speaker guard
  when "self.Health.hp < 30"
  priority 5
  say "guard.hurt" {
    arg hp "self.Health.hp"
  }
  cooldown 50
}
bark alarm {
  topic idle
  when "Game.alarm"
  priority 9
  shared 300
  once
  say "guard.alarm"
  set alert "alert + 1"
}
"#;

const HUNT: &str = r#"type quest
version 1
---
id hunt
title "quest.hunt"
stage kill {
  objective wolves "quest.hunt.wolves" {
    target 2
    on EnemyDied
    where "event.kind == 'wolf'"
  }
  objective any "quest.hunt.any" {
    target 3
    on EnemyDied
  }
}
"#;

/// A tiny world: entity 1 and 2 have Health.hp; Game.alarm; the event
/// being counted.
struct World {
    hp: BTreeMap<u64, f64>,
    alarm: bool,
    event: Option<&'static str>,
}

impl World {
    fn read(&self) -> impl Fn(Option<u64>, &str) -> Option<Val> + '_ {
        move |e, path| match (e, path) {
            (Some(e), "Health.hp") => self.hp.get(&e).map(|h| Val::Num(*h)),
            (None, "Game.alarm") => Some(Val::Bool(self.alarm)),
            (None, "event.kind") => self.event.map(|k| Val::Str(k.into())),
            _ => None,
        }
    }
}

fn lib() -> Library {
    let mut l = Library::default();
    let d = Dialogue::parse(GUARD).unwrap();
    l.dialogues.insert(d.name.clone(), Arc::new(d));
    let q = Quest::parse(HUNT).unwrap();
    l.quests.insert(q.id.clone(), Arc::new(q));
    l
}

fn said(s: &Story, e: u64) -> Option<String> {
    s.barks.get(&e).map(|b| b.key.clone())
}

#[test]
fn bark_errors_name_the_line_and_the_fix() {
    let bad = |body: &str| {
        Dialogue::parse(&format!("type dialogue\nversion 1\n---\nname d\n{body}")).unwrap_err()
    };
    assert!(bad("bark a {\n  say \"x\"\n}").contains("needs a 'topic'"));
    assert!(bad("bark a {\n  topic idle\n}").contains("needs a line"));
    let e = bad("node n {\n  if \"Game.alarm\" n\n}");
    assert!(e.contains("line 6") && e.contains("reads the world"), "{e}");
    let e = Quest::parse(
        "type quest\nversion 1\n---\nid q\ntitle \"t\"\nstage s {\n  objective o \"k\" {\n    where \"event.kind\"\n  }\n}",
    )
    .unwrap_err();
    assert!(e.contains("add 'on <Event>'"), "{e}");
}

#[test]
fn barks_pick_by_priority_and_condition_and_keep_their_cooldowns() {
    let l = lib();
    let mut s = Story::new(Rng::new(3));
    let mut w = World {
        hp: BTreeMap::from([(1, 100.0), (2, 10.0)]),
        alarm: false,
        event: None,
    };
    let alive = |_: u64| true;
    s.bark(&l, 1, "idle", "").unwrap();
    s.bark(&l, 2, "idle", "").unwrap();
    s.run_barks(&l, 1, &w.read(), &alive);
    // 1 is healthy: the yawn. 2 is hurt: the higher priority, with its hp.
    assert_eq!(said(&s, 1).as_deref(), Some("guard.yawn"));
    assert_eq!(said(&s, 2).as_deref(), Some("guard.hurt"));
    assert_eq!(s.barks[&2].args, [("hp".into(), Val::Num(10.0))]);
    assert!(
        s.events
            .iter()
            .any(|e| matches!(e, Event::Bark { entity: 2, .. }))
    );

    // Asked again at 60: 1's yawn cools down 100 ticks, so nothing new;
    // 2's hurt cooled its 50 but is still shown at priority 5, which only
    // a higher priority interrupts.
    s.run_barks(&l, 60, &w.read(), &alive);
    let seq = s.seq;
    s.bark(&l, 1, "idle", "").unwrap();
    s.bark(&l, 2, "idle", "").unwrap();
    s.run_barks(&l, 61, &w.read(), &alive);
    assert_eq!(s.seq, seq, "cooldowns and a busy speaker say nothing");

    // The alarm (priority 9, shared, once) interrupts both... once.
    w.alarm = true;
    s.bark(&l, 1, "idle", "").unwrap();
    s.bark(&l, 2, "idle", "").unwrap();
    s.run_barks(&l, 62, &w.read(), &alive);
    assert_eq!(said(&s, 1).as_deref(), Some("guard.alarm"));
    // Shared cooldown (and once): 2, asked in the same run, gets its next best.
    assert_ne!(said(&s, 2).as_deref(), Some("guard.alarm"));
    assert_eq!(
        s.get(&l, "alert"),
        Some(Val::Num(1.0)),
        "the bark's set ran"
    );

    // The yawn cycles its lines: after its cooldown, the second.
    w.alarm = false;
    s.run_barks(&l, 400, &w.read(), &alive);
    s.bark(&l, 1, "idle", "").unwrap();
    s.run_barks(&l, 401, &w.read(), &alive);
    assert_eq!(said(&s, 1).as_deref(), Some("guard.stretch"));
    assert!(
        s.bark(&l, 1, "nope", "")
            .unwrap_err()
            .contains("topic 'nope'")
    );
}

#[test]
fn ambient_barks_come_every_so_often_on_the_seed() {
    let l = lib();
    let w = World {
        hp: BTreeMap::from([(1, 100.0)]),
        alarm: false,
        event: None,
    };
    let run = |seed: u32| {
        let mut s = Story::new(Rng::new(seed));
        s.ambient(1, "idle", "", 150, 40);
        let mut at = Vec::new();
        for t in 1..=1000 {
            let before = s.seq;
            s.run_barks(&l, t, &w.read(), &|_| true);
            if s.seq != before {
                at.push(t);
            }
        }
        at
    };
    let a = run(5);
    assert_eq!(a, run(5));
    // An ask while the last bark still shows (180 ticks) is skipped.
    assert!(a.len() >= 3, "{a:?}");
    // Never sooner than the yawn's cooldown of 100 ticks.
    assert!(a.windows(2).all(|w| w[1] - w[0] >= 100), "{a:?}");
}

#[test]
fn objectives_count_events_through_their_filters() {
    let l = lib();
    let mut s = Story::new(Rng::new(1));
    s.start_quest(&l, "hunt").unwrap();
    assert_eq!(Story::counted_events(&l), ["EnemyDied"]);
    let mut w = World {
        hp: BTreeMap::new(),
        alarm: false,
        event: Some("rat"),
    };
    assert!(s.count_event(&l, "EnemyDied", &w.read()));
    assert!(!s.count_event(&l, "PlayerJumped", &w.read()));
    let q = s.quest("hunt").unwrap();
    assert_eq!((q.counts.get("wolves"), q.counts["any"]), (None, 1));
    w.event = Some("wolf");
    s.count_event(&l, "EnemyDied", &w.read());
    s.count_event(&l, "EnemyDied", &w.read());
    // Two wolves and three kills: the last stage done, the quest complete.
    assert_eq!(s.quest("hunt").unwrap().status, "completed");
    assert!(!s.count_event(&l, "EnemyDied", &w.read()));
}

#[test]
fn voices_time_their_lines_and_saves_keep_barks() {
    let mut l = lib();
    let d = Dialogue::parse(
        "type dialogue\nversion 1\n---\nname vo\nnode a {\n  say narrator \"vo.a\" {\n    voice \"vo/a.wav\"\n  }\n}\n",
    )
    .unwrap();
    l.dialogues.insert("vo".into(), Arc::new(d));
    l.voice_ticks.insert("vo/a.wav".into(), 90);
    let mut s = Story::new(Rng::new(2));
    let c = s
        .start(&l, "vo", BTreeMap::new(), BTreeMap::new(), None)
        .unwrap();
    let line = s.convs.get_mut(&c).unwrap().history.last_mut().unwrap();
    assert_eq!(line.voice_left(10), 90, "all of it before it starts");
    line.voice_start = Some(10);
    assert_eq!(
        (
            line.voice_left(40),
            line.voice_left(100),
            line.voice_left(500)
        ),
        (60, 0, 0)
    );

    let w = World {
        hp: BTreeMap::from([(1, 5.0)]),
        alarm: false,
        event: None,
    };
    s.ambient(1, "idle", "guard", 50, 0);
    s.bark(&l, 1, "idle", "").unwrap();
    s.run_barks(&l, 3, &w.read(), &|_| true);
    s.voices.insert(format!("c{c}"), (1, 77));
    let mut a = Vec::new();
    s.save(&mut a);
    let back = Story::load(&a).unwrap();
    let mut b = Vec::new();
    back.save(&mut b);
    assert_eq!(a, b);
    assert_eq!(back.barks, s.barks);
    assert_eq!(back.ambient, s.ambient);
    assert_eq!(back.bark_last_by, s.bark_last_by);
    assert_eq!(back.voices, s.voices);
    assert_eq!(back.convs[&c].history[0].voice_start, Some(10));
    // A version-1 save (batch 1) still loads.
    let mut v1 = Story::new(Rng::new(2));
    v1.globals.insert("x".into(), Val::Num(1.0));
    let mut bytes = Vec::new();
    v1.save(&mut bytes);
    let cut = bytes.len() - 8 * 4; // the eight empty version-2 sections
    let mut old = bytes[..cut].to_vec();
    old[4] = 1;
    assert_eq!(Story::load(&old).unwrap().globals["x"], Val::Num(1.0));
}
