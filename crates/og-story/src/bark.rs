//! Barks: ambient lines an entity says on its own, outside a conversation
//! (a guard's yawn, a merchant's call, a shout when it spots the player).
//! A dialogue declares them (`bark <id> { topic ... }`); the game asks for a
//! topic (og.story.bark) or lets an entity bark every so often
//! (og.story.ambient). Each request is settled at the og.story system's
//! next run: of the barks of its topic whose `when` holds (it may read the
//! world: `self.Health.hp`, `Game.alarm`) and whose cooldowns are over, the
//! highest `priority` is said, ties drawn by `weight` on the story's
//! stream. A bark being said is interrupted only by a higher priority.

use crate::asset::{Bark, Dialogue};
use crate::expr::Val;
use crate::story::{ConvScope, Event, Library, Read, Story};
use std::sync::Arc;

/// A bark asked for.
#[derive(Clone, Debug, PartialEq)]
pub struct Request {
    pub entity: u64,
    pub topic: String,
    /// Only this dialogue's barks (empty: any loaded dialogue's).
    pub dialogue: String,
}

/// An entity that barks on its own every `every` ticks (plus up to
/// `jitter` drawn on the story's stream).
#[derive(Clone, Debug, PartialEq)]
pub struct Ambient {
    pub topic: String,
    pub dialogue: String,
    pub every: u32,
    pub jitter: u32,
    /// The tick it next asks.
    pub next: u32,
}

/// A bark being said.
#[derive(Clone, Debug, PartialEq)]
pub struct Said {
    pub dialogue: String,
    pub bark: String,
    /// Its line's index in the bark.
    pub line: u32,
    pub key: String,
    pub args: Vec<(String, Val)>,
    pub speaker: String,
    pub voice: String,
    pub seq: u32,
    pub start: u32,
    /// The tick it stops showing.
    pub until: u32,
    pub priority: i32,
    pub voice_ticks: u32,
    pub voice_start: Option<u32>,
}

impl Said {
    /// Ticks of its voice left at `now` (as [`crate::Shown::voice_left`]).
    pub fn voice_left(&self, now: u32) -> u32 {
        match self.voice_start {
            _ if self.voice.is_empty() => 0,
            None => self.voice_ticks,
            Some(s) => s.saturating_add(self.voice_ticks).saturating_sub(now),
        }
    }
}

fn key_of(d: &Dialogue, b: &Bark) -> String {
    format!("{}.{}", d.name, b.id)
}

impl Story {
    /// Asks `entity` to bark a line of `topic` at the og.story system's
    /// next run. Refused for a topic no loaded dialogue (or `dialogue`) has.
    pub fn bark(
        &mut self,
        lib: &Library,
        entity: u64,
        topic: &str,
        dialogue: &str,
    ) -> Result<(), String> {
        let known = lib
            .dialogues
            .values()
            .filter(|d| dialogue.is_empty() || d.name == dialogue)
            .any(|d| d.barks.iter().any(|b| b.topic == topic));
        if !known {
            return Err(if dialogue.is_empty() {
                format!("no loaded dialogue has a bark of topic '{topic}'")
            } else {
                format!(
                    "the dialogue '{dialogue}' has no bark of topic '{topic}' (or is not loaded)"
                )
            });
        }
        self.bark_requests.push(crate::bark::Request {
            entity,
            topic: topic.to_string(),
            dialogue: dialogue.to_string(),
        });
        Ok(())
    }

    /// Makes `entity` bark `topic` every `every` ticks, the first time
    /// after `every` ticks; `every` 0 stops it.
    pub fn ambient(&mut self, entity: u64, topic: &str, dialogue: &str, every: u32, jitter: u32) {
        if every == 0 {
            self.ambient.remove(&entity);
            return;
        }
        let next = self.tick.saturating_add(every);
        self.ambient.insert(
            entity,
            Ambient {
                topic: topic.to_string(),
                dialogue: dialogue.to_string(),
                every,
                jitter,
                next,
            },
        );
    }

    /// The barks of one og.story run: ambient entities that are due ask,
    /// barks whose time is over stop, and every request is settled in the
    /// order asked. `read` reads the world for `when` and arguments, and
    /// `alive` says whether an entity still lives.
    pub fn run_barks(
        &mut self,
        lib: &Library,
        tick: u32,
        read: Read<'_>,
        alive: &dyn Fn(u64) -> bool,
    ) {
        self.tick = tick;
        self.ambient.retain(|e, _| alive(*e));
        let due: Vec<u64> = self
            .ambient
            .iter()
            .filter(|(_, a)| a.next <= tick)
            .map(|(e, _)| *e)
            .collect();
        for e in due {
            let a = self.ambient[&e].clone();
            let extra = if a.jitter > 0 {
                self.rng.integer(0, i64::from(a.jitter)) as u32
            } else {
                0
            };
            if let Some(slot) = self.ambient.get_mut(&e) {
                slot.next = tick.saturating_add(a.every).saturating_add(extra);
            }
            self.bark_requests.push(Request {
                entity: e,
                topic: a.topic,
                dialogue: a.dialogue,
            });
        }
        self.barks.retain(|e, s| s.until > tick && alive(*e));
        for r in std::mem::take(&mut self.bark_requests) {
            if alive(r.entity) {
                self.settle_bark(lib, tick, read, &r);
            }
        }
    }

    fn settle_bark(&mut self, lib: &Library, tick: u32, read: Read<'_>, r: &Request) {
        let busy = self.barks.get(&r.entity).map(|s| s.priority);
        let mut found: Vec<(Arc<Dialogue>, usize)> = Vec::new();
        for d in lib.dialogues.values() {
            if !r.dialogue.is_empty() && d.name != r.dialogue {
                continue;
            }
            for (i, b) in d.barks.iter().enumerate() {
                if b.topic != r.topic || busy.is_some_and(|p| p >= b.priority) {
                    continue;
                }
                let id = key_of(d, b);
                if b.once && self.bark_once.contains(&id) {
                    continue;
                }
                let since = |t: Option<&u32>| t.map_or(u32::MAX, |t| tick.saturating_sub(*t));
                if since(self.bark_last_by.get(&(id.clone(), r.entity))) < b.cooldown
                    || since(self.bark_last.get(&id)) < b.shared
                {
                    continue;
                }
                let holds = b.when.as_ref().is_none_or(|x| {
                    let scope = ConvScope {
                        story: self,
                        lib,
                        dialogue: Some(&d.name),
                        locals: None,
                        world: Some((read, Some(r.entity))),
                    };
                    x.eval(&scope).is_ok_and(|v| v.truthy())
                });
                if holds {
                    found.push((d.clone(), i));
                }
            }
        }
        let Some(top) = found.iter().map(|(d, i)| d.barks[*i].priority).max() else {
            return;
        };
        found.retain(|(d, i)| d.barks[*i].priority == top);
        let pick = if found.len() > 1 {
            let w: Vec<f64> = found.iter().map(|(d, i)| d.barks[*i].weight).collect();
            og_core::procgen::weighted(&mut self.rng, &w).unwrap_or(0)
        } else {
            0
        };
        let (d, i) = found.swap_remove(pick);
        let b = &d.barks[i];
        let id = key_of(&d, b);
        let n = b.lines.len() as u32;
        let line = if n <= 1 {
            0
        } else if b.random {
            self.rng.integer(0, i64::from(n) - 1) as u32
        } else {
            let c = self.bark_cycle.entry(id.clone()).or_insert(0);
            let l = *c % n;
            *c = (l + 1) % n;
            l
        };
        let l = &b.lines[line as usize];
        let args = {
            let scope = ConvScope {
                story: self,
                lib,
                dialogue: Some(&d.name),
                locals: None,
                world: Some((read, Some(r.entity))),
            };
            l.text
                .args
                .iter()
                .map(|(k, x)| (k.clone(), x.eval(&scope).unwrap_or(Val::Num(0.0))))
                .collect()
        };
        self.seq = self.seq.wrapping_add(1);
        self.bark_last.insert(id.clone(), tick);
        self.bark_last_by.insert((id.clone(), r.entity), tick);
        if b.once {
            self.bark_once.insert(id);
        }
        let speaker = if b.speaker.is_empty() {
            crate::NARRATOR.to_string()
        } else {
            b.speaker.clone()
        };
        self.barks.insert(
            r.entity,
            Said {
                dialogue: d.name.clone(),
                bark: b.id.clone(),
                line,
                key: l.text.key.clone(),
                args,
                speaker,
                voice: l.voice.clone(),
                seq: self.seq,
                start: tick,
                until: tick.saturating_add(b.duration),
                priority: b.priority,
                voice_ticks: lib.voice_ticks.get(&l.voice).copied().unwrap_or(0),
                voice_start: None,
            },
        );
        self.events.push(Event::Bark {
            entity: r.entity,
            bark: b.id.clone(),
            key: l.text.key.clone(),
            seq: self.seq,
        });
        for s in &b.steps {
            self.act(lib, &d, 0, s);
        }
    }
}
