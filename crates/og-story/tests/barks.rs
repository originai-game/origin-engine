//! og-story batch 2: barks (world conditions, priority, cooldowns per
//! entity and shared, cycles, once), objectives counted from events, voice
//! timing, and the version-2 save carrying all of it.

use og_core::Rng;
use og_story::{Dialogue, Event, Library, Quest, Story, Val};
use std::collections::BTreeMap;
use std::sync::Arc;

const GUARD: &str = r#"#og-profile:yarn-3.0-og-1
#og-name:guard
title: yawn
og-topic: idle
og-cooldown: 100
---
<<declare $alert = 0>>
guard: Yawn #line:guard.yawn
guard: Stretch #line:guard.stretch
===
title: hurt
og-topic: idle
og-when: self.Health.hp < 30
og-priority: 5
og-cooldown: 50
---
guard: Hurt {self.Health.hp} #line:guard.hurt
===
title: alarm
og-topic: idle
og-when: Game.alarm
og-priority: 9
og-shared: 300
og-once: true
---
Alarm #line:guard.alarm
<<set $alert = $alert + 1>>
===
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
    assert_eq!(s.barks[&1].speaker, "guard");
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
fn voices_time_their_lines_and_saves_keep_barks() {
    let mut l = lib();
    let d = Dialogue::parse(
        "#og-profile:yarn-3.0-og-1\n#og-name:vo\ntitle: a\n---\nVoice #line:vo.a #voice:vo/a.wav\n===\n",
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
    // Version 3 (R10): the solo the story holds, and each line's shot.
    s.camera = Some((3, 9));
    s.convs.get_mut(&c).unwrap().history[0].shot = "close".into();
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
    assert_eq!(back.camera, Some((3, 9)));
    assert_eq!(back.convs[&c].history[0].shot, "close");
    // A version-1 save (batch 1) still loads.
    let mut v1 = Story::new(Rng::new(2));
    v1.globals.insert("x".into(), Val::Num(1.0));
    let mut bytes = Vec::new();
    v1.save(&mut bytes);
    // Version 4 added the story tick after the RNG seed and state.
    bytes.drain(13..17);
    // The eight empty version-2 sections and version 3's empty camera.
    let cut = bytes.len() - 8 * 4 - 1;
    let mut old = bytes[..cut].to_vec();
    old[4] = 1;
    assert_eq!(Story::load(&old).unwrap().globals["x"], Val::Num(1.0));
    // A version-2 save (G2-8): no camera byte.
    let mut v2 = bytes[..bytes.len() - 1].to_vec();
    v2[4] = 2;
    let back = Story::load(&v2).unwrap();
    assert_eq!(
        (back.globals["x"].clone(), back.camera),
        (Val::Num(1.0), None)
    );
}
