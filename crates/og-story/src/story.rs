//! The story a world keeps: the story's variables, the nodes seen, the
//! lines read, the conversations under way and the quests, with the
//! story's random stream. All of it is saved ([`Story::save`]) and
//! restored ([`Story::load`]); a save names nodes, lines, quests, stages
//! and objectives by their ids, so it survives every edit of a dialogue or
//! quest that keeps those ids. A missing dialogue continuation is an error,
//! never a restart that might repeat a command or reward.

use crate::asset::{Choice, Dialogue, Node, Quest, QuestOp, Step};
use crate::expr::{Expr, Scope, Val};
use crate::yarn::saliency::{Saliency, complexity};
use og_core::Rng;
use std::cell::Cell;
use std::collections::{BTreeMap, BTreeSet};
use std::sync::Arc;

/// The steps a conversation may run without showing a line before it is
/// stopped as a loop.
pub const MAX_STEPS: u32 = 10_000;

/// The dialogues and quests an engine has loaded, by name and id.
#[derive(Clone, Debug, Default)]
pub struct Library {
    pub dialogues: BTreeMap<String, Arc<Dialogue>>,
    pub quests: BTreeMap<String, Arc<Quest>>,
    /// Each voice clip's length in ticks (the runtime fills it from the
    /// cooked sounds when a dialogue loads; 0 or absent: unknown).
    pub voice_ticks: BTreeMap<String, u32>,
}

impl Library {
    /// The starting value of a story variable, from the first dialogue or
    /// quest (by name) that declares it.
    pub fn default_of(&self, name: &str) -> Option<Val> {
        let d = self
            .dialogues
            .values()
            .flat_map(|d| d.vars.iter())
            .chain(self.quests.values().flat_map(|q| q.vars.iter()));
        d.filter(|v| v.global && v.name == name)
            .map(|v| v.value.clone())
            .next()
    }
}

/// Where a conversation waits.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Wait {
    /// On a line: `advance` moves on.
    Line,
    /// On a node's choices: `choose` picks one.
    Choices,
    Ended,
}

/// A line as shown, its arguments evaluated when it was.
#[derive(Clone, Debug, PartialEq)]
pub struct Shown {
    pub node: String,
    /// Its step in the node.
    pub step: u32,
    pub speaker: String,
    pub key: String,
    pub args: Vec<(String, Val)>,
    pub voice: String,
    pub expression: String,
    /// The camera shot it is shown in (its own, else its node's; empty:
    /// none).
    pub shot: String,
    pub seq: u32,
    /// It had been read before it was shown.
    pub seen: bool,
    /// The choice made while it was shown: its index and id.
    pub chosen: Option<(u32, String)>,
    /// The selected option's arguments as offered, never reevaluated in history.
    pub chosen_args: Vec<(String, Val)>,
    /// Its voice clip's length in ticks (Library::voice_ticks; 0: unknown
    /// or none).
    pub voice_ticks: u32,
    /// The tick its voice started (set by the runtime when it plays it).
    pub voice_start: Option<u32>,
}

impl Shown {
    /// Ticks of its voice left at `now`: all of them before it starts, 0
    /// once it has ended (or without one).
    pub fn voice_left(&self, now: u32) -> u32 {
        match self.voice_start {
            _ if self.voice.is_empty() => 0,
            None => self.voice_ticks,
            Some(s) => (s.saturating_add(self.voice_ticks)).saturating_sub(now),
        }
    }
}

/// An active check's roll.
#[derive(Clone, Debug, PartialEq)]
pub struct Roll {
    pub stat: String,
    pub target: f64,
    pub roll: u32,
    pub passed: bool,
}

/// Values evaluated once when an option is offered, not on every UI read.
#[derive(Clone, Debug, PartialEq)]
pub struct ChoiceValues {
    pub visible: bool,
    pub enabled: bool,
    pub chance: f64,
    pub args: Vec<(String, Val)>,
    pub reason_args: Vec<(String, Val)>,
    pub target: Option<f64>,
}

/// A conversation.
#[derive(Clone, Debug, PartialEq)]
pub struct Conv {
    pub dialogue: String,
    pub node: String,
    /// The step shown (a line) or next to run.
    pub pc: u32,
    pub wait: Wait,
    pub locals: BTreeMap<String, Val>,
    /// Speaker ids to the entities playing them.
    pub cast: BTreeMap<String, u64>,
    /// `node/choice` ids chosen before.
    pub visited: BTreeSet<String>,
    /// Every line shown, the last one current.
    pub history: Vec<Shown>,
    /// Ticks left and the timer's length.
    pub timer: Option<(u32, u32)>,
    pub last_roll: Option<Roll>,
    /// Stable continuation block IDs, not instruction offsets.
    pub stack: Vec<String>,
    /// Tracked source nodes per call frame, outermost first. A node-group frame
    /// holds both the group and its selected member; both exit together.
    pub active_nodes: Vec<Vec<String>>,
    pub choice_values: BTreeMap<String, ChoiceValues>,
}

/// A quest's state.
#[derive(Clone, Debug, PartialEq)]
pub struct QuestState {
    pub id: String,
    pub stage: String,
    /// `active`, `completed` or `failed`.
    pub status: &'static str,
    /// The current stage's objectives' counts.
    pub counts: BTreeMap<String, u32>,
    /// Objectives the quest failed with (left undone when it failed).
    pub failed: BTreeSet<String>,
    pub tracked: bool,
    pub is_new: bool,
    pub updated: bool,
    pub updated_tick: u32,
    /// A marker the game set (entity, position).
    pub marker: Option<(u64, [f32; 3])>,
}

/// What the story tells the world (sent as events by the og.story system).
#[derive(Clone, Debug, PartialEq)]
pub enum Event {
    Line {
        conv: u32,
        seq: u32,
    },
    Ended {
        conv: u32,
    },
    Check {
        conv: u32,
        roll: Roll,
    },
    Quest {
        quest: String,
        status: &'static str,
    },
    Emit {
        conv: u32,
        name: String,
        arg: String,
    },
    /// A bark was said (conv 0 for an `emit` of a bark's steps).
    Bark {
        entity: u64,
        bark: String,
        key: String,
        seq: u32,
    },
}

/// A world's story.
#[derive(Clone, Debug)]
pub struct Story {
    pub globals: BTreeMap<String, Val>,
    /// Nodes entered, as `dialogue.node`.
    pub seen: BTreeSet<String>,
    pub visits: BTreeMap<String, u32>,
    pub once: BTreeSet<String>,
    pub saliency: Saliency,
    pub content_views: BTreeMap<String, u32>,
    /// Lines read, as `dialogue.node.key`; kept by [`Story::reset`].
    pub read: BTreeSet<String>,
    pub seq: u32,
    pub rng: Rng,
    pub next_conv: u32,
    pub convs: BTreeMap<u32, Conv>,
    pub quests: Vec<QuestState>,
    /// Events not yet sent.
    pub events: Vec<Event>,
    /// Barks being said, by speaker entity.
    pub barks: BTreeMap<u64, crate::bark::Said>,
    /// Barks asked for, said at the og.story system's next run.
    pub bark_requests: Vec<crate::bark::Request>,
    /// Entities that bark on their own, every so often.
    pub ambient: BTreeMap<u64, crate::bark::Ambient>,
    /// When each bark (`dialogue.bark`) was last said by anyone.
    pub bark_last: BTreeMap<String, u32>,
    /// When each bark was last said by each entity.
    pub bark_last_by: BTreeMap<(String, u64), u32>,
    /// The next line of each `cycle` bark.
    pub bark_cycle: BTreeMap<String, u32>,
    /// The `once` barks said this playthrough.
    pub bark_once: BTreeSet<String>,
    /// The voices playing: `c<conversation>` or `b<entity>` to the line's
    /// seq and the entity playing it (the runtime's).
    pub voices: BTreeMap<String, (u32, u64)>,
    /// While a line's shot is og.CameraBrain's solo: the solo the story
    /// found (handed back when no line has a shot) and the one it wrote,
    /// as entity bits (the runtime's).
    pub camera: Option<(u64, u64)>,
    /// The tick the og.story system last ran (not saved: it is the world's).
    pub tick: u32,
    /// Warnings not yet logged (not saved).
    pub warnings: Vec<String>,
}

impl Story {
    /// An empty story drawing on `rng` (og.Rng's fork "story").
    pub fn new(rng: Rng) -> Story {
        Story {
            globals: BTreeMap::new(),
            seen: BTreeSet::new(),
            visits: BTreeMap::new(),
            once: BTreeSet::new(),
            saliency: Saliency::default(),
            content_views: BTreeMap::new(),
            read: BTreeSet::new(),
            seq: 0,
            rng,
            next_conv: 1,
            convs: BTreeMap::new(),
            quests: Vec::new(),
            events: Vec::new(),
            barks: BTreeMap::new(),
            bark_requests: Vec::new(),
            ambient: BTreeMap::new(),
            bark_last: BTreeMap::new(),
            bark_last_by: BTreeMap::new(),
            bark_cycle: BTreeMap::new(),
            bark_once: BTreeSet::new(),
            voices: BTreeMap::new(),
            camera: None,
            tick: 0,
            warnings: Vec::new(),
        }
    }

    /// The shot the camera should show: the current line's of the newest
    /// conversation still running (none when that line has none), with the
    /// conversation's id.
    pub fn shot(&self) -> Option<(u32, &str)> {
        self.convs
            .iter()
            .rev()
            .find(|(_, c)| c.wait != Wait::Ended)
            .and_then(|(id, c)| {
                let l = c.history.last()?;
                (!l.shot.is_empty()).then_some((*id, l.shot.as_str()))
            })
    }

    /// A new playthrough: variables, seen nodes, conversations and quests
    /// cleared; read lines, the line count and the random stream kept.
    pub fn reset(&mut self) {
        self.globals.clear();
        self.seen.clear();
        self.visits.clear();
        self.once.clear();
        self.content_views.clear();
        self.convs.clear();
        self.quests.clear();
        self.events.clear();
        self.barks.clear();
        self.bark_requests.clear();
        self.bark_last.clear();
        self.bark_last_by.clear();
        self.bark_cycle.clear();
        self.bark_once.clear();
    }

    pub(crate) fn warn(&mut self, w: String) {
        if !self.warnings.contains(&w) {
            self.warnings.push(w);
        }
    }

    /// A story variable, evaluating a smart variable when it is read.
    pub fn get(&mut self, lib: &Library, name: &str) -> Option<Val> {
        if let Some(d) = lib.dialogues.values().find(|d| d.smart.contains_key(name)) {
            return Some(self.value(lib, d, 0, &d.smart[name]));
        }
        self.globals
            .get(name)
            .cloned()
            .or_else(|| lib.default_of(name))
    }

    /// Sets a story variable; the quests settle.
    pub fn set(&mut self, lib: &Library, name: &str, v: Val) {
        if lib.dialogues.values().any(|d| d.smart.contains_key(name)) {
            self.warn(format!("cannot assign to smart variable ${name}"));
            return;
        }
        self.globals.insert(name.to_string(), v);
        self.settle(lib);
    }

    pub fn quest(&self, id: &str) -> Option<&QuestState> {
        self.quests.iter().find(|q| q.id == id)
    }

    fn quest_mut(&mut self, id: &str) -> Option<&mut QuestState> {
        self.quests.iter_mut().find(|q| q.id == id)
    }

    // ── conversations ──────────────────────────────────────────────────

    /// Starts a dialogue: its first line, or its choices, are shown now.
    pub fn start(
        &mut self,
        lib: &Library,
        dialogue: &str,
        cast: BTreeMap<String, u64>,
        vars: BTreeMap<String, Val>,
        node: Option<&str>,
    ) -> Result<u32, String> {
        let d = lib.dialogues.get(dialogue).cloned().ok_or_else(|| {
            format!("no dialogue '{dialogue}' is loaded (og.story.load it first)")
        })?;
        let at = node.unwrap_or(&d.start).to_string();
        if d.node(&at).is_none() {
            return Err(format!("the dialogue '{dialogue}' has no node '{at}'"));
        }
        let mut locals: BTreeMap<String, Val> = d
            .vars
            .iter()
            .filter(|v| !v.global)
            .map(|v| (v.name.clone(), v.value.clone()))
            .collect();
        for (k, v) in vars {
            match locals.get_mut(&k) {
                Some(slot) => *slot = v,
                None => {
                    return Err(format!(
                        "the dialogue '{dialogue}' declares no 'var {k}' to start with"
                    ));
                }
            }
        }
        for s in cast.keys() {
            if d.speaker(s).is_none() && s != crate::NARRATOR {
                return Err(format!("the dialogue '{dialogue}' has no speaker '{s}'"));
            }
        }
        let id = self.next_conv;
        self.next_conv = self.next_conv.wrapping_add(1).max(1);
        self.convs.insert(
            id,
            Conv {
                dialogue: d.name.clone(),
                node: at.clone(),
                pc: 0,
                wait: Wait::Line,
                locals,
                cast,
                visited: BTreeSet::new(),
                history: Vec::new(),
                timer: None,
                last_roll: None,
                stack: Vec::new(),
                choice_values: BTreeMap::new(),
                active_nodes: vec![if d.untracked.contains(&at) {
                    Vec::new()
                } else {
                    vec![at.clone()]
                }],
            },
        );
        self.enter(&d, id, &at);
        self.run(lib, &d, id);
        self.settle(lib);
        Ok(id)
    }

    /// Moves a conversation past its line.
    pub fn advance(&mut self, lib: &Library, id: u32) -> Result<(), String> {
        let d = self.resolve(lib, id)?;
        let c = &self.convs[&id];
        match c.wait {
            Wait::Line => {}
            Wait::Choices => return Err("the conversation waits for a choice: choose one".into()),
            Wait::Ended => return Err("the conversation has ended".into()),
        }
        self.convs.get_mut(&id).unwrap().pc += 1;
        self.run(lib, &d, id);
        self.settle(lib);
        Ok(())
    }

    /// Picks choice `index` of the node a conversation waits on.
    pub fn choose(&mut self, lib: &Library, id: u32, index: u32) -> Result<(), String> {
        let d = self.resolve(lib, id)?;
        let c = &self.convs[&id];
        if c.wait != Wait::Choices {
            return Err(if c.wait == Wait::Ended {
                "the conversation has ended".into()
            } else {
                "the conversation shows a line, not choices: advance it".into()
            });
        }
        let node = d.node(&c.node).expect("resolved");
        if index == u32::MAX {
            let next = node
                .choice_fallback
                .as_ref()
                .ok_or("this choice group has no fallthrough")?;
            self.convs.get_mut(&id).unwrap().timer = None;
            self.jump(&d, id, next);
            self.run(lib, &d, id);
            self.settle(lib);
            return Ok(());
        }
        let Some(choice) = node.choices.get(index as usize) else {
            return Err(format!(
                "the node has {} choices; there is no choice {index}",
                node.choices.len()
            ));
        };
        let (visible, enabled, _) = self.choice_state(lib, &d, id, choice);
        if !visible || !enabled {
            return Err(format!("choice {index} ('{}') is not available", choice.id));
        }
        self.take(lib, &d, id, index);
        self.settle(lib);
        Ok(())
    }

    fn take(&mut self, lib: &Library, d: &Arc<Dialogue>, id: u32, index: u32) {
        let c = self.convs.get_mut(&id).unwrap();
        let node = d.node(&c.node).expect("resolved");
        let choice = &node.choices[index as usize];
        c.visited.insert(format!("{}/{}", node.id, choice.id));
        c.timer = None;
        if let Some(last) = c.history.last_mut() {
            last.chosen = Some((index, choice.id.clone()));
            last.chosen_args = c
                .choice_values
                .get(&choice.id)
                .map_or_else(Vec::new, |v| v.args.clone());
        }
        for s in &choice.steps {
            self.act(lib, d, id, s);
        }
        let mut to = choice.goto.clone();
        if let Some(k) = &choice.check
            && k.active
        {
            let stat = self.number(lib, d, id, &Expr::Var(k.stat.clone()));
            let target = self.convs[&id]
                .choice_values
                .get(&choice.id)
                .and_then(|v| v.target)
                .unwrap_or_else(|| self.number(lib, d, id, &k.target));
            let roll = self.rng.integer(1, 20) as u32;
            let passed = f64::from(roll) + stat >= target;
            let r = Roll {
                stat: k.stat.clone(),
                target,
                roll,
                passed,
            };
            self.events.push(Event::Check {
                conv: id,
                roll: r.clone(),
            });
            self.convs.get_mut(&id).unwrap().last_roll = Some(r);
            if !passed {
                to = choice.fail.clone();
            }
        }
        match to {
            Some(t) => {
                self.jump(d, id, &t);
                self.run(lib, d, id);
            }
            None => self.end(id),
        }
    }

    /// Ends and forgets a conversation.
    pub fn stop(&mut self, id: u32) -> bool {
        self.end(id);
        self.convs.remove(&id).is_some()
    }

    /// One tick of the timers (the og.story system, before the game's
    /// systems): a timer that runs out takes its choice.
    pub fn tick(&mut self, lib: &Library, tick: u32) {
        self.tick = tick;
        let ids: Vec<u32> = self
            .convs
            .iter()
            .filter(|(_, c)| c.timer.is_some() && c.wait == Wait::Choices)
            .map(|(k, _)| *k)
            .collect();
        for id in ids {
            let Ok(d) = self.resolve(lib, id) else {
                continue;
            };
            let c = self.convs.get_mut(&id).unwrap();
            let Some((left, total)) = c.timer else {
                continue;
            };
            if left > 1 {
                c.timer = Some((left - 1, total));
                continue;
            }
            let node = d.node(&c.node).expect("resolved");
            let default = node
                .timer
                .as_ref()
                .and_then(|(_, t)| node.choices.iter().position(|ch| &ch.id == t));
            let usable = default.filter(|&i| {
                let (v, e, _) = self.choice_state(lib, &d, id, &node.choices[i]);
                v && e
            });
            match usable {
                Some(i) => self.take(lib, &d, id, i as u32),
                None => {
                    self.warn(format!(
                        "conversation {id}: the timer's choice in '{}.{}' is not available when it runs out; the conversation ends",
                        d.name, node.id
                    ));
                    self.end(id);
                }
            }
        }
        self.settle(lib);
    }

    /// Resolve stable continuation and line IDs after loading or editing.
    /// Missing continuations are errors: restarting could repeat commands.
    pub fn resolve(&mut self, lib: &Library, id: u32) -> Result<Arc<Dialogue>, String> {
        let c = self
            .convs
            .get(&id)
            .ok_or_else(|| format!("no conversation {id}"))?;
        let Some(d) = lib.dialogues.get(&c.dialogue).cloned() else {
            return Err(format!(
                "conversation {id}'s dialogue '{}' is not loaded",
                c.dialogue
            ));
        };
        if c.wait == Wait::Ended {
            return Ok(d);
        }
        let Some(node) = d.node(&c.node) else {
            return Err(format!(
                "conversation {id}: saved continuation '{}' is missing from '{}'; restore its stable id (restarting could repeat commands)",
                c.node, d.name
            ));
        };
        for continuation in &c.stack {
            if d.node(continuation).is_none() {
                return Err(format!(
                    "conversation {id}: saved detour continuation '{continuation}' is missing; restore its stable id"
                ));
            }
        }
        let shown_key = c.history.last().map(|s| s.key.clone());
        let fits = |pc: u32| matches!(node.steps.get(pc as usize), Some(Step::Say(l)) if Some(&l.text.key) == shown_key.as_ref());
        match c.wait {
            Wait::Line if !fits(c.pc) => {
                let found = (0..node.steps.len() as u32).find(|&i| fits(i));
                let c = self.convs.get_mut(&id).unwrap();
                match found {
                    Some(pc) => c.pc = pc,
                    None => {
                        return Err(format!(
                            "conversation {id}: saved line '{}' is missing; restore its #line: id",
                            shown_key.unwrap_or_default()
                        ));
                    }
                }
            }
            Wait::Choices if node.choices.is_empty() => {
                return Err(format!(
                    "conversation {id}: saved choices at '{}' are missing; restore their stable ids",
                    c.node
                ));
            }
            _ => {}
        }
        Ok(d)
    }

    fn exit_node(&mut self, id: u32) {
        let c = self.convs.get_mut(&id).unwrap();
        if let Some(nodes) = c.active_nodes.pop() {
            for node in nodes {
                let count = self
                    .visits
                    .entry(format!("{}.{node}", c.dialogue))
                    .or_default();
                *count = count.saturating_add(1);
            }
        }
    }

    fn end(&mut self, id: u32) {
        while self
            .convs
            .get(&id)
            .is_some_and(|c| !c.active_nodes.is_empty())
        {
            self.exit_node(id);
        }
        if let Some(c) = self.convs.get_mut(&id)
            && c.wait != Wait::Ended
        {
            c.wait = Wait::Ended;
            c.timer = None;
            self.events.push(Event::Ended { conv: id });
        }
    }

    fn enter(&mut self, d: &Dialogue, _id: u32, node: &str) {
        self.seen.insert(format!("{}.{node}", d.name));
    }

    /// Jumps to a continuation block.
    fn jump(&mut self, d: &Dialogue, id: u32, to: &str) {
        let c = self.convs.get_mut(&id).unwrap();
        c.node = to.to_string();
        c.pc = 0;
        self.enter(d, id, to);
    }

    /// Runs a conversation from its node and step until it shows a line,
    /// waits on choices or ends.
    fn run(&mut self, lib: &Library, d: &Arc<Dialogue>, id: u32) {
        for _ in 0..MAX_STEPS {
            let c = &self.convs[&id];
            let Some(node) = d.node(&c.node) else {
                self.end(id);
                return;
            };
            let pc = c.pc as usize;
            let Some(step) = node.steps.get(pc) else {
                if node.choices.is_empty() {
                    self.end(id);
                } else {
                    self.wait_choices(lib, d, node, id);
                }
                return;
            };
            match step {
                Step::Say(_) => {
                    self.show(lib, d, id, node, pc);
                    if node.choice_line == Some(pc) && !node.choices.is_empty() {
                        for s in &node.steps[pc + 1..] {
                            self.act(lib, d, id, s);
                        }
                        self.wait_choices(lib, d, node, id);
                    } else {
                        self.convs.get_mut(&id).unwrap().wait = Wait::Line;
                    }
                    return;
                }
                Step::If(x, t) => {
                    if self.truthy(lib, d, id, x) {
                        self.jump(d, id, t);
                    } else {
                        self.convs.get_mut(&id).unwrap().pc += 1;
                    }
                }
                Step::Goto(t) => {
                    self.jump(d, id, t);
                }
                Step::Jump(expr) | Step::DynamicDetour(expr, _) => {
                    let target = self.value(lib, d, id, expr);
                    let Val::Str(target) = target else {
                        self.warn("Yarn destination must evaluate to a string".into());
                        self.end(id);
                        return;
                    };
                    if !d.node(&target).is_some_and(|n| n.id == n.title) {
                        self.warn(format!("Yarn destination '{target}' names no source node"));
                        self.end(id);
                        return;
                    }
                    if let Step::DynamicDetour(_, continuation) = step {
                        self.convs
                            .get_mut(&id)
                            .unwrap()
                            .stack
                            .push(continuation.clone());
                    } else {
                        while !self.convs[&id].active_nodes.is_empty() {
                            self.exit_node(id);
                        }
                        self.convs.get_mut(&id).unwrap().stack.clear();
                    }
                    self.convs.get_mut(&id).unwrap().active_nodes.push(
                        if d.untracked.contains(&target) {
                            Vec::new()
                        } else {
                            vec![target.clone()]
                        },
                    );
                    self.jump(d, id, &target);
                }
                Step::End => {
                    self.end(id);
                    return;
                }
                Step::Detour(target, continuation) => {
                    self.convs.get_mut(&id).unwrap().active_nodes.push(
                        if d.untracked.contains(target) {
                            Vec::new()
                        } else {
                            vec![target.clone()]
                        },
                    );
                    self.convs
                        .get_mut(&id)
                        .unwrap()
                        .stack
                        .push(continuation.clone());
                    self.jump(d, id, target);
                }
                Step::Return => {
                    self.exit_node(id);
                    if let Some(target) = self.convs.get_mut(&id).unwrap().stack.pop() {
                        self.jump(d, id, &target);
                    } else {
                        self.end(id);
                        return;
                    }
                }
                Step::Select(options) => {
                    let mut eligible: Vec<_> = options
                        .iter()
                        .filter(|(_, cond, _)| {
                            cond.as_ref().is_none_or(|x| self.truthy(lib, d, id, x))
                        })
                        .collect();
                    let track_views = matches!(
                        self.saliency,
                        Saliency::BestLeastRecentlyViewed | Saliency::RandomBestLeastRecentlyViewed
                    );
                    if track_views {
                        let count = |target: &str| {
                            self.content_views
                                .get(&format!("{}.{target}", d.name))
                                .copied()
                                .unwrap_or(0)
                        };
                        let least = eligible.iter().map(|o| count(&o.0)).min().unwrap_or(0);
                        eligible.retain(|o| count(&o.0) == least);
                    }
                    let score = |o: &&(String, Option<Expr>, f64)| {
                        d.saliency_scores
                            .get(&o.0)
                            .copied()
                            .unwrap_or_else(|| o.1.as_ref().map_or(0, complexity))
                    };
                    match self.saliency {
                        Saliency::First => {}
                        Saliency::ConditionalFirstWeighted => {
                            if eligible.iter().any(|o| score(o) > 0) {
                                eligible.retain(|o| score(o) > 0);
                            }
                        }
                        _ => {
                            let best = eligible.iter().map(score).max().unwrap_or(0);
                            eligible.retain(|o| score(o) == best);
                        }
                    }
                    if eligible.is_empty() {
                        self.convs.get_mut(&id).unwrap().pc += 1;
                    } else {
                        let i = match self.saliency {
                            Saliency::ConditionalFirstWeighted => {
                                let weights: Vec<_> = eligible.iter().map(|o| o.2).collect();
                                og_core::procgen::weighted(&mut self.rng, &weights).unwrap_or(0)
                            }
                            Saliency::RandomBestLeastRecentlyViewed if eligible.len() > 1 => {
                                self.rng.integer(0, eligible.len() as i64 - 1) as usize
                            }
                            _ => 0,
                        };
                        if track_views {
                            let count = self
                                .content_views
                                .entry(format!("{}.{}", d.name, eligible[i].0))
                                .or_default();
                            *count = count.saturating_add(1);
                        }
                        let target = &eligible[i].0;
                        if d.group_members.get(target) == Some(&node.id)
                            && !d.untracked.contains(target)
                        {
                            self.convs
                                .get_mut(&id)
                                .unwrap()
                                .active_nodes
                                .last_mut()
                                .unwrap()
                                .push(target.clone());
                        }
                        self.jump(d, id, target);
                    }
                }
                s => {
                    self.act(lib, d, id, s);
                    self.convs.get_mut(&id).unwrap().pc += 1;
                }
            }
        }
        let c = &self.convs[&id];
        let w = format!(
            "conversation {id}: '{}.{}' ran {MAX_STEPS} steps without a line (a loop); it ends",
            d.name, c.node
        );
        self.warn(w);
        self.end(id);
    }

    fn wait_choices(&mut self, lib: &Library, d: &Dialogue, node: &Node, id: u32) {
        let mut values = BTreeMap::new();
        for choice in &node.choices {
            let scope = self.scope(lib, d, id);
            let (visible, enabled, chance, target) = Self::choice_state_in(&scope, choice);
            let args = choice
                .text
                .args
                .iter()
                .map(|(key, expr)| expr.eval(&scope).map(|value| (key.clone(), value)))
                .collect::<Result<Vec<_>, _>>();
            let reason_args = choice
                .reason
                .iter()
                .flat_map(|reason| &reason.args)
                .map(|(key, expr)| expr.eval(&scope).map(|value| (key.clone(), value)))
                .collect::<Result<Vec<_>, _>>();
            self.rng = scope.rng.get();
            let args = args.unwrap_or_else(|error| {
                self.warn(format!("dialogue '{}': {error}", d.name));
                Vec::new()
            });
            let reason_args = reason_args.unwrap_or_else(|error| {
                self.warn(format!("dialogue '{}': {error}", d.name));
                Vec::new()
            });
            values.insert(
                choice.id.clone(),
                ChoiceValues {
                    visible,
                    enabled,
                    chance,
                    args,
                    reason_args,
                    target,
                },
            );
        }
        let c = self.convs.get_mut(&id).unwrap();
        c.choice_values = values;
        c.wait = Wait::Choices;
        c.pc = node.steps.len() as u32;
        c.timer = node.timer.as_ref().map(|(t, _)| (*t, *t));
    }

    fn show(&mut self, lib: &Library, d: &Arc<Dialogue>, id: u32, node: &Node, pc: usize) {
        let Step::Say(l) = &node.steps[pc] else {
            return;
        };
        let args: Vec<(String, Val)> = l
            .text
            .args
            .iter()
            .map(|(k, x)| (k.clone(), self.value(lib, d, id, x)))
            .collect();
        let read_id = format!("{}.{}.{}", d.name, node.id, l.text.key);
        let seen = !self.read.insert(read_id);
        self.seq = self.seq.wrapping_add(1);
        let shown = Shown {
            node: node.id.clone(),
            step: pc as u32,
            speaker: l.speaker.clone(),
            key: l.text.key.clone(),
            args,
            voice: l.voice.clone(),
            expression: l.expression.clone(),
            shot: match l.shot.as_str() {
                "none" => String::new(),
                "" => node.shot.clone(),
                own => own.to_string(),
            },
            seq: self.seq,
            seen,
            chosen: None,
            chosen_args: Vec::new(),
            voice_ticks: lib.voice_ticks.get(&l.voice).copied().unwrap_or(0),
            voice_start: None,
        };
        let c = self.convs.get_mut(&id).unwrap();
        c.pc = pc as u32;
        c.history.push(shown);
        self.events.push(Event::Line {
            conv: id,
            seq: self.seq,
        });
    }

    pub(crate) fn act(&mut self, lib: &Library, d: &Arc<Dialogue>, id: u32, s: &Step) {
        match s {
            Step::MarkOnce(key) => {
                self.once.insert(format!("{}.{key}", d.name));
            }
            Step::Call(expr) => {
                self.value(lib, d, id, expr);
            }
            Step::Command(expr) => {
                let value = self.value(lib, d, id, expr);
                let command = crate::yarn::functions::string(&value);
                let (name, arg) = command.split_once(' ').unwrap_or((&command, ""));
                self.events.push(Event::Emit {
                    conv: id,
                    name: name.into(),
                    arg: arg.into(),
                });
            }
            Step::Set(name, x) => {
                let v = self.value(lib, d, id, x);
                match self.convs.get_mut(&id).and_then(|c| c.locals.get_mut(name)) {
                    Some(slot) => *slot = v,
                    None => {
                        self.globals.insert(name.clone(), v);
                    }
                }
            }
            Step::Emit(name, x) => {
                let arg = x
                    .as_ref()
                    .map(|x| self.value(lib, d, id, x).to_string())
                    .unwrap_or_default();
                self.events.push(Event::Emit {
                    conv: id,
                    name: name.clone(),
                    arg,
                });
            }
            Step::Quest(op) => {
                let op = match op {
                    QuestOp::Progress(q, o, x) => {
                        let n = self.number(lib, d, id, x);
                        self.progress(lib, q, o, n.max(0.0) as u32);
                        return;
                    }
                    other => other.clone(),
                };
                if let Err(e) = self.quest_op(lib, &op) {
                    self.warn(format!("dialogue '{}': {e}", d.name));
                }
            }
            _ => {}
        }
    }

    fn scope<'a>(&'a self, lib: &'a Library, d: &'a Dialogue, id: u32) -> ConvScope<'a> {
        ConvScope {
            story: self,
            rng: Cell::new(self.rng),
            lib,
            dialogue: Some(&d.name),
            locals: self.convs.get(&id).map(|c| &c.locals),
            world: None,
        }
    }

    fn value(&mut self, lib: &Library, d: &Dialogue, id: u32, x: &Expr) -> Val {
        let scope = self.scope(lib, d, id);
        let r = x.eval(&scope);
        self.rng = scope.rng.get();
        match r {
            Ok(v) => v,
            Err(e) => {
                self.warn(format!("dialogue '{}': {e}", d.name));
                Val::Num(0.0)
            }
        }
    }

    fn truthy(&mut self, lib: &Library, d: &Dialogue, id: u32, x: &Expr) -> bool {
        self.value(lib, d, id, x).truthy()
    }

    fn number(&mut self, lib: &Library, d: &Dialogue, id: u32, x: &Expr) -> f64 {
        match self.value(lib, d, id, x) {
            Val::Num(n) => n,
            Val::Bool(b) => f64::from(u8::from(b)),
            Val::Str(_) => 0.0,
        }
    }

    /// A choice's state: visible, enabled, and its check's chance (1 without
    /// a check).
    pub fn choice_state(
        &self,
        lib: &Library,
        d: &Dialogue,
        id: u32,
        choice: &Choice,
    ) -> (bool, bool, f64) {
        if let Some(values) = self
            .convs
            .get(&id)
            .and_then(|c| c.choice_values.get(&choice.id))
        {
            return (values.visible, values.enabled, values.chance);
        }
        let scope = self.scope(lib, d, id);
        let (visible, enabled, chance, _) = Self::choice_state_in(&scope, choice);
        (visible, enabled, chance)
    }

    pub fn choice_args(&self, id: u32, choice: &Choice) -> Vec<(String, Val)> {
        self.convs
            .get(&id)
            .and_then(|c| c.choice_values.get(&choice.id))
            .map_or_else(Vec::new, |v| v.args.clone())
    }

    fn choice_state_in(scope: &dyn Scope, choice: &Choice) -> (bool, bool, f64, Option<f64>) {
        let cond = choice
            .cond
            .as_ref()
            .is_none_or(|x| x.eval(scope).is_ok_and(|v| v.truthy()));
        let num = |x: &Expr| match x.eval(scope) {
            Ok(Val::Num(n)) => n,
            Ok(Val::Bool(b)) => f64::from(u8::from(b)),
            _ => 0.0,
        };
        let target = choice.check.as_ref().map(|k| num(&k.target));
        let (pass, chance) = match &choice.check {
            None => (true, 1.0),
            Some(k) => {
                let stat = num(&Expr::Var(k.stat.clone()));
                let target = target.expect("check target evaluated");
                if k.active {
                    // d20 + stat >= target: the rolls from target - stat up.
                    (true, ((21.0 - (target - stat)) / 20.0).clamp(0.0, 1.0))
                } else {
                    let p = stat >= target;
                    (p, if p { 1.0 } else { 0.0 })
                }
            }
        };
        let enabled = cond && pass;
        (enabled || !choice.hide, enabled, chance, target)
    }

    /// An expression's value in a conversation (`d` and `id`) or the story's
    /// scope; `None` when it cannot be evaluated.
    pub fn eval(&self, lib: &Library, d: Option<&Dialogue>, id: u32, x: &Expr) -> Option<Val> {
        let scope = ConvScope {
            story: self,
            rng: Cell::new(self.rng),
            lib,
            dialogue: d.map(|d| d.name.as_str()),
            locals: self.convs.get(&id).map(|c| &c.locals),
            world: None,
        };
        x.eval(&scope).ok()
    }

    /// A text's arguments evaluated in a conversation (or the story's scope).
    pub fn args(
        &self,
        lib: &Library,
        d: Option<&Dialogue>,
        id: u32,
        args: &[(String, Expr)],
    ) -> Vec<(String, Val)> {
        let scope = ConvScope {
            story: self,
            rng: Cell::new(self.rng),
            lib,
            dialogue: d.map(|d| d.name.as_str()),
            locals: self.convs.get(&id).map(|c| &c.locals),
            world: None,
        };
        args.iter()
            .map(|(k, x)| (k.clone(), x.eval(&scope).unwrap_or(Val::Num(0.0))))
            .collect()
    }

    // ── quests ─────────────────────────────────────────────────────────

    /// Starts a quest; false if it has started before.
    pub fn start_quest(&mut self, lib: &Library, id: &str) -> Result<bool, String> {
        let r = self.quest_op(lib, &QuestOp::Start(id.to_string()));
        self.settle(lib);
        r
    }

    /// Counts `n` toward an objective of an active quest's current stage;
    /// false when the quest is not active or the stage has no such
    /// objective.
    pub fn progress(&mut self, lib: &Library, id: &str, objective: &str, n: u32) -> bool {
        let Some(def) = lib.quests.get(id).cloned() else {
            self.warn(format!("no quest '{id}' is loaded"));
            return false;
        };
        let tick = self.tick;
        let Some(q) = self.quest_mut(id) else {
            return false;
        };
        if q.status != "active" {
            return false;
        }
        let Some(o) = def
            .stage(&q.stage)
            .and_then(|s| s.objectives.iter().find(|o| o.id == objective))
        else {
            return false;
        };
        let count = q.counts.entry(objective.to_string()).or_insert(0);
        let next = count.saturating_add(n).min(o.target);
        if next != *count {
            *count = next;
            q.updated = true;
            q.updated_tick = tick;
            let ev = Event::Quest {
                quest: id.to_string(),
                status: "active",
            };
            self.events.push(ev);
        }
        self.settle(lib);
        true
    }

    /// The event names quest objectives count (`on <Event>`) across the
    /// loaded quests, sorted.
    pub fn counted_events(lib: &Library) -> Vec<String> {
        let mut out: Vec<String> = lib
            .quests
            .values()
            .flat_map(|q| q.stages.iter())
            .flat_map(|s| s.objectives.iter())
            .filter_map(|o| o.on.clone())
            .collect();
        out.sort();
        out.dedup();
        out
    }

    /// One event `name` the world sent: every objective of an active
    /// quest's current stage that counts it (`on name`) and whose `where`
    /// holds counts one. `read` reads the event's fields (`event.kind`) and
    /// the world. True when something counted.
    pub fn count_event(&mut self, lib: &Library, name: &str, read: Read<'_>) -> bool {
        let mut hits = Vec::new();
        for q in self.quests.iter().filter(|q| q.status == "active") {
            let Some(stage) = lib.quests.get(&q.id).and_then(|d| d.stage(&q.stage)) else {
                continue;
            };
            for o in stage
                .objectives
                .iter()
                .filter(|o| o.on.as_deref() == Some(name))
            {
                let scope = ConvScope {
                    story: self,
                    rng: Cell::new(self.rng),
                    lib,
                    dialogue: None,
                    locals: None,
                    world: Some((read, None)),
                };
                if o.filter
                    .as_ref()
                    .is_none_or(|x| x.eval(&scope).is_ok_and(|v| v.truthy()))
                {
                    hits.push((q.id.clone(), o.id.clone()));
                }
            }
        }
        let mut any = false;
        for (q, o) in hits {
            any |= self.progress(lib, &q, &o, 1);
        }
        any
    }

    /// A quest operation; Ok(false) when it changes nothing.
    pub fn quest_op(&mut self, lib: &Library, op: &QuestOp) -> Result<bool, String> {
        let id = match op {
            QuestOp::Start(q)
            | QuestOp::Progress(q, ..)
            | QuestOp::Stage(q, _)
            | QuestOp::Complete(q)
            | QuestOp::Fail(q) => q.clone(),
        };
        let def = lib
            .quests
            .get(&id)
            .cloned()
            .ok_or_else(|| format!("no quest '{id}' is loaded (og.story.load it first)"))?;
        let tick = self.tick;
        match op {
            QuestOp::Start(_) => {
                if self.quest(&id).is_some() {
                    return Ok(false);
                }
                let tracked = !self
                    .quests
                    .iter()
                    .any(|q| q.tracked && q.status == "active");
                self.quests.push(QuestState {
                    id: id.clone(),
                    stage: def.stages[0].id.clone(),
                    status: "active",
                    counts: BTreeMap::new(),
                    failed: BTreeSet::new(),
                    tracked,
                    is_new: true,
                    updated: true,
                    updated_tick: tick,
                    marker: None,
                });
                self.events.push(Event::Quest {
                    quest: id,
                    status: "active",
                });
            }
            QuestOp::Progress(_, o, x) => {
                let n = match x {
                    Expr::Lit(Val::Num(n)) => *n,
                    _ => 1.0,
                };
                return Ok(self.progress(lib, &id, o, n.max(0.0) as u32));
            }
            QuestOp::Stage(_, s) => {
                if def.stage(s).is_none() {
                    return Err(format!("the quest '{id}' has no stage '{s}'"));
                }
                let Some(q) = self.quest_mut(&id) else {
                    return Ok(false);
                };
                if q.status != "active" || q.stage == *s {
                    return Ok(false);
                }
                q.stage = s.clone();
                q.counts.clear();
                q.updated = true;
                q.updated_tick = tick;
                self.events.push(Event::Quest {
                    quest: id,
                    status: "active",
                });
            }
            QuestOp::Complete(_) | QuestOp::Fail(_) => {
                let status = if matches!(op, QuestOp::Complete(_)) {
                    "completed"
                } else {
                    "failed"
                };
                let Some(q) = self.quest_mut(&id) else {
                    return Ok(false);
                };
                if q.status != "active" {
                    return Ok(false);
                }
                finish(q, &def, status, tick);
                self.events.push(Event::Quest { quest: id, status });
            }
        }
        Ok(true)
    }

    /// Re-checks every active quest's conditions to a fixpoint: failures,
    /// `when` objectives, finished stages.
    pub fn settle(&mut self, lib: &Library) {
        for _ in 0..64 {
            let mut changed = false;
            for i in 0..self.quests.len() {
                let (id, stage) = (self.quests[i].id.clone(), self.quests[i].stage.clone());
                if self.quests[i].status != "active" {
                    continue;
                }
                let Some(def) = lib.quests.get(&id).cloned() else {
                    continue;
                };
                let Some(st) = def.stage(&stage) else {
                    let first = def.stages[0].id.clone();
                    self.warn(format!(
                        "quest '{id}': the stage '{stage}' is gone (renamed or removed); it falls back to '{first}'"
                    ));
                    let q = &mut self.quests[i];
                    q.stage = first;
                    q.counts.clear();
                    changed = true;
                    continue;
                };
                let gone: Vec<String> = self.quests[i]
                    .counts
                    .keys()
                    .filter(|k| !st.objectives.iter().any(|o| &o.id == *k))
                    .cloned()
                    .collect();
                for g in gone {
                    self.warn(format!(
                        "quest '{id}': the objective '{g}' is gone from stage '{stage}' (renamed or removed); its count is dropped"
                    ));
                    self.quests[i].counts.remove(&g);
                }
                let scope = ConvScope {
                    story: self,
                    rng: Cell::new(self.rng),
                    lib,
                    dialogue: None,
                    locals: None,
                    world: None,
                };
                let holds = |x: &Expr| x.eval(&scope).is_ok_and(|v| v.truthy());
                let fail = st.fail.as_ref().is_some_and(holds);
                let whens: Vec<(String, u32)> = st
                    .objectives
                    .iter()
                    .filter(|o| o.when.as_ref().is_some_and(holds))
                    .map(|o| (o.id.clone(), o.target))
                    .collect();
                let tick = self.tick;
                let q = &mut self.quests[i];
                if fail {
                    finish(q, &def, "failed", tick);
                    self.events.push(Event::Quest {
                        quest: id,
                        status: "failed",
                    });
                    changed = true;
                    continue;
                }
                for (o, t) in whens {
                    let c = q.counts.entry(o).or_insert(0);
                    if *c < t {
                        *c = t;
                        q.updated = true;
                        q.updated_tick = tick;
                        changed = true;
                    }
                }
                let done = st
                    .objectives
                    .iter()
                    .filter(|o| !o.optional)
                    .all(|o| q.counts.get(&o.id).copied().unwrap_or(0) >= o.target);
                if done {
                    let status = match &st.next {
                        Some(n) => {
                            q.stage = n.clone();
                            q.counts.clear();
                            q.updated = true;
                            q.updated_tick = tick;
                            "active"
                        }
                        None => {
                            finish(q, &def, "completed", tick);
                            "completed"
                        }
                    };
                    self.events.push(Event::Quest { quest: id, status });
                    changed = true;
                }
            }
            if !changed {
                return;
            }
        }
    }
}

fn finish(q: &mut QuestState, def: &Quest, status: &'static str, tick: u32) {
    q.status = status;
    q.updated = true;
    q.updated_tick = tick;
    if status == "failed"
        && let Some(st) = def.stage(&q.stage)
    {
        for o in &st.objectives {
            if q.counts.get(&o.id).copied().unwrap_or(0) < o.target {
                q.failed.insert(o.id.clone());
            }
        }
    }
}

/// What an expression reads: a conversation's variables, then the story's.
pub(crate) struct ConvScope<'a> {
    pub story: &'a Story,
    pub rng: Cell<Rng>,
    pub lib: &'a Library,
    pub dialogue: Option<&'a str>,
    pub locals: Option<&'a BTreeMap<String, Val>>,
    /// The world, where it is at hand (a bark's `when`, an objective's
    /// `where`), and the entity `self.` names.
    pub world: Option<(Read<'a>, Option<u64>)>,
}

/// Reads the world for an expression: `(entity, "Health.hp")` a component
/// field of an entity, `(None, "Game.alarm")` a resource field or, while
/// an event is counted, `(None, "event.kind")` its field.
pub type Read<'a> = &'a dyn Fn(Option<u64>, &str) -> Option<Val>;

impl Scope for ConvScope<'_> {
    fn once(&self, id: &str) -> bool {
        self.dialogue
            .is_some_and(|d| self.story.once.contains(&format!("{d}.{id}")))
    }
    fn invoke(&self, name: &str, args: &[Val]) -> Result<Val, String> {
        if matches!(
            name,
            "random" | "random_range" | "random_range_float" | "dice"
        ) {
            let mut rng = self.rng.get();
            let value = match (name, args) {
                ("random", []) => rng.next_f64() as f32,
                ("random_range" | "random_range_float", [Val::Num(min), Val::Num(max)]) => {
                    // Both functions in the 3.2.2 release use an integer draw
                    // plus the original floating minimum (Dialogue.cs).
                    let (min, max) = (*min as f32, *max as f32);
                    let width = i64::from(max as i32) - i64::from(min as i32);
                    if width < 0 {
                        return Err("random range maximum precedes minimum".into());
                    }
                    rng.integer(0, width) as f32 + min
                }
                ("dice", [Val::Num(sides)]) if *sides >= 1.0 => {
                    rng.integer(1, *sides as i64) as f32
                }
                _ => return Err(format!("invalid arguments to {name}")),
            };
            self.rng.set(rng);
            return Ok(Val::Num(f64::from(value)));
        }
        if name == "has_any_content" {
            let [Val::Str(node)] = args else {
                return Err("has_any_content needs a node name".into());
            };
            let d = self
                .dialogue
                .and_then(|name| self.lib.dialogues.get(name))
                .ok_or("no dialogue for has_any_content")?;
            let node = d
                .node(node)
                .ok_or("has_any_content target does not exist")?;
            if let Some(Step::Select(candidates)) = node.steps.first() {
                for (_, condition, _) in candidates {
                    if condition
                        .as_ref()
                        .map(|e| e.eval(self))
                        .transpose()?
                        .is_none_or(|v| v.truthy())
                    {
                        return Ok(Val::Bool(true));
                    }
                }
                return Ok(Val::Bool(false));
            }
            return Ok(Val::Bool(true));
        }
        if matches!(name, "visited" | "visited_count") {
            let [Val::Str(node)] = args else {
                return Err(format!("{name} needs a node name"));
            };
            let count = self
                .dialogue
                .and_then(|d| self.story.visits.get(&format!("{d}.{node}")))
                .copied()
                .unwrap_or(0);
            return Ok(if name == "visited" {
                Val::Bool(count > 0)
            } else {
                Val::Num(f64::from(count))
            });
        }
        if let Some(d) = self.dialogue.and_then(|name| self.lib.dialogues.get(name)) {
            return crate::yarn::functions::invoke(&d.functions, name, args);
        }
        crate::yarn::functions::pure(name, args)?
            .ok_or_else(|| format!("function '{name}' is not registered"))
    }

    fn var(&self, name: &str) -> Option<Val> {
        if name.contains('.') {
            let (read, me) = self.world?;
            return match name.strip_prefix("self.") {
                Some(rest) => read(Some(me?), rest),
                None => read(None, name),
            };
        }
        if let Some(expr) = self
            .dialogue
            .and_then(|d| self.lib.dialogues.get(d))
            .and_then(|d| d.smart.get(name))
        {
            return expr.eval(self).ok();
        }
        self.locals
            .and_then(|l| l.get(name).cloned())
            .or_else(|| self.story.globals.get(name).cloned())
            .or_else(|| self.lib.default_of(name))
    }
    fn seen(&self, node: &str) -> bool {
        if node.contains('.') {
            self.story.seen.contains(node)
        } else {
            self.dialogue
                .is_some_and(|d| self.story.seen.contains(&format!("{d}.{node}")))
        }
    }
    fn quest_status(&self, quest: &str) -> &'static str {
        self.story.quest(quest).map_or("none", |q| q.status)
    }
    fn quest_stage(&self, quest: &str) -> String {
        self.story
            .quest(quest)
            .map(|q| q.stage.clone())
            .unwrap_or_default()
    }
    fn quest_count(&self, quest: &str, objective: &str) -> f64 {
        self.story
            .quest(quest)
            .and_then(|q| q.counts.get(objective))
            .map_or(0.0, |c| f64::from(*c))
    }
}
