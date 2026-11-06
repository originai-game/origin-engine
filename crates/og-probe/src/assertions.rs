//! Assertions (IDL `assert`): what the game's `og.assert.*` calls, the
//! default invariants from reflection and a playtest's `invariant:` steps
//! report, kept as one report per run beside the world, never in it (a
//! snapshot, a restore and the world's hash do not see it).
//!
//! A site is an assertion by its kind, name and place. The report holds
//! every site seen or declared (the catalog), each with whether it fired,
//! its first violation and how many it had, and an `each` site's buckets.
//! [`Assertions::take`] answers the report and starts the next one: the
//! catalog stays, so the next run lists the same sites as never fired until
//! they fire again, and nothing a run saw passes into the next (a sibling
//! fork or another playtest of a session cannot pass on an earlier run's
//! coverage).

use serde_json::{Value, json};
use std::collections::BTreeMap;

/// The most buckets one `each` site keeps; more are counted, not listed.
pub const MAX_BUCKETS: usize = 1024;
/// The most sites a report keeps; more are counted, not listed.
pub const MAX_SITES: usize = 4096;

/// IDL `assert.AssertKind`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
    Always,
    Sometimes,
    Reachable,
    Unreachable,
    Each,
}

impl Kind {
    pub fn parse(name: &str) -> Option<Kind> {
        Some(match name {
            "always" => Kind::Always,
            "sometimes" => Kind::Sometimes,
            "reachable" => Kind::Reachable,
            "unreachable" => Kind::Unreachable,
            "each" => Kind::Each,
            _ => return None,
        })
    }

    pub fn name(self) -> &'static str {
        match self {
            Kind::Always => "always",
            Kind::Sometimes => "sometimes",
            Kind::Reachable => "reachable",
            Kind::Unreachable => "unreachable",
            Kind::Each => "each",
        }
    }

    /// Whether the run must see it fire: `sometimes`, `reachable`, `each`.
    pub fn covers(self) -> bool {
        matches!(self, Kind::Sometimes | Kind::Reachable | Kind::Each)
    }
}

/// Who declared a site.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Origin {
    /// The game's `og.assert.*`.
    Game,
    /// The engine's default invariants from reflection.
    Default,
    /// A playtest's `invariant:` step.
    Playtest,
}

impl Origin {
    pub fn name(self) -> &'static str {
        match self {
            Origin::Game => "game",
            Origin::Default => "default",
            Origin::Playtest => "playtest",
        }
    }
}

/// One assertion call (IDL `assert.AssertRecord`, with its origin).
#[derive(Clone, Debug, PartialEq)]
pub struct Call {
    pub kind: Kind,
    pub origin: Origin,
    pub name: String,
    pub location: String,
    pub condition: bool,
    /// An `each`'s bucket; empty otherwise.
    pub key: String,
    pub details: Value,
    pub tick: u64,
    /// A site the build found before it ran: listed, not a call.
    pub declared: bool,
}

/// What a violation saw: the entity, the record and field, the value
/// and what was expected, beside the call's own details.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Seen {
    /// The entity's probe id (its name, or `#<index>.<generation>`).
    pub entity: Option<String>,
    pub component: Option<String>,
    pub field: Option<String>,
    pub value: Value,
    pub expected: Value,
    pub message: String,
}

/// A site's first violation.
#[derive(Clone, Debug, PartialEq)]
pub struct Violation {
    pub site: usize,
    pub tick: u64,
    pub seen: Seen,
    pub details: Value,
}

#[derive(Clone, Debug)]
struct Bucket {
    key: String,
    first: Option<u64>,
}

#[derive(Clone, Debug)]
struct Site {
    kind: Kind,
    origin: Origin,
    name: String,
    location: String,
    /// The first tick it fired (held, ran), in this report.
    first: Option<u64>,
    /// Whether it ran in this report (a declared site that never ran has not).
    ran: bool,
    failures: u64,
    last_failure: u64,
    buckets: Vec<Bucket>,
    buckets_dropped: u64,
}

/// What the default invariants cost, measured outside the tick.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Cost {
    /// The reflected fields checked.
    pub fields: u64,
    /// Ticks checked in this report.
    pub ticks: u64,
    /// Row values read in this report (a field of an entity, a resource's field).
    pub values: u64,
    pub total_ms: f64,
    pub max_ms: f64,
}

/// The report: sites, first violations, buckets and cost.
#[derive(Clone, Debug, Default)]
pub struct Assertions {
    sites: Vec<Site>,
    index: BTreeMap<(Kind, String, String), usize>,
    violations: Vec<Violation>,
    sites_dropped: u64,
    epoch: u32,
    cost: Cost,
}

impl Assertions {
    /// The report's epoch: it moves on at each [`Assertions::take`].
    pub fn epoch(&self) -> u32 {
        self.epoch
    }

    fn site(&mut self, kind: Kind, origin: Origin, name: &str, location: &str) -> Option<usize> {
        let key = (kind, name.to_owned(), location.to_owned());
        if let Some(&i) = self.index.get(&key) {
            return Some(i);
        }
        if self.sites.len() >= MAX_SITES {
            self.sites_dropped += 1;
            return None;
        }
        let i = self.sites.len();
        self.sites.push(Site {
            kind,
            origin,
            name: name.to_owned(),
            location: location.to_owned(),
            first: None,
            ran: false,
            failures: 0,
            last_failure: 0,
            buckets: Vec::new(),
            buckets_dropped: 0,
        });
        self.index.insert(key, i);
        Some(i)
    }

    /// Records one call: a first sighting, a violation or a firing.
    pub fn record(&mut self, call: &Call) {
        let Some(i) = self.site(call.kind, call.origin, &call.name, &call.location) else {
            return;
        };
        if call.declared {
            return;
        }
        let tick = call.tick;
        let site = &mut self.sites[i];
        site.ran = true;
        let fired = match call.kind {
            Kind::Always => call.condition,
            Kind::Unreachable => false,
            Kind::Sometimes => call.condition,
            Kind::Reachable => true,
            Kind::Each => {
                let at = match site.buckets.iter().position(|b| b.key == call.key) {
                    Some(at) => Some(at),
                    None if site.buckets.len() < MAX_BUCKETS => {
                        site.buckets.push(Bucket {
                            key: call.key.clone(),
                            first: None,
                        });
                        Some(site.buckets.len() - 1)
                    }
                    None => {
                        site.buckets_dropped += 1;
                        None
                    }
                };
                if let Some(b) = at.map(|at| &mut site.buckets[at])
                    && call.condition
                    && b.first.is_none()
                {
                    b.first = Some(tick);
                }
                call.condition
            }
        };
        let violated = matches!(call.kind, Kind::Always | Kind::Unreachable) && !fired;
        if fired && site.first.is_none() && call.kind != Kind::Always {
            site.first = Some(tick);
        }
        if violated {
            let seen = Seen {
                message: match call.kind {
                    Kind::Unreachable => "reached".into(),
                    _ => "did not hold".into(),
                },
                ..Seen::default()
            };
            self.violate(i, tick, seen, call.details.clone());
        }
    }

    /// A violation the engine found itself (a default invariant, a
    /// playtest's invariant): an `always` site that did not hold.
    pub fn fail(&mut self, origin: Origin, name: &str, location: &str, tick: u64, seen: Seen) {
        if let Some(i) = self.site(Kind::Always, origin, name, location) {
            self.sites[i].ran = true;
            self.violate(i, tick, seen, Value::Null);
        }
    }

    /// Lists an `always` site that runs without failing (a playtest's
    /// invariant), so the report names it.
    pub fn declare(&mut self, origin: Origin, name: &str, location: &str) {
        self.site(Kind::Always, origin, name, location);
    }

    fn violate(&mut self, i: usize, tick: u64, seen: Seen, details: Value) {
        let site = &mut self.sites[i];
        site.failures += 1;
        site.last_failure = tick;
        if site.failures == 1 {
            self.violations.push(Violation {
                site: i,
                tick,
                seen,
                details,
            });
        }
    }

    /// Adds one tick of the default invariants' cost.
    pub fn add_cost(&mut self, fields: u64, values: u64, ms: f64) {
        let c = &mut self.cost;
        c.fields = fields;
        c.ticks += 1;
        c.values += values;
        c.total_ms += ms;
        c.max_ms = c.max_ms.max(ms);
    }

    /// How many sites failed in this report.
    pub fn violated(&self) -> usize {
        self.violations.len()
    }

    /// The report (IDL `assert.AssertReport`).
    pub fn report(&self) -> Value {
        let site_json = |s: &Site| {
            let mut v = json!({
                "kind": s.kind.name(),
                "origin": s.origin.name(),
                "name": s.name,
                "location": s.location,
                "ran": s.ran,
                "fired": s.first.is_some(),
                "firstTick": s.first,
                "failures": s.failures,
            });
            if s.kind == Kind::Each {
                v["buckets"] = json!(
                    s.buckets
                        .iter()
                        .map(|b| json!({"key": b.key, "fired": b.first.is_some(), "firstTick": b.first}))
                        .collect::<Vec<_>>()
                );
                v["bucketsDropped"] = json!(s.buckets_dropped);
            }
            v
        };
        let violations: Vec<Value> = self
            .violations
            .iter()
            .map(|v| {
                let s = &self.sites[v.site];
                json!({
                    "kind": s.kind.name(),
                    "origin": s.origin.name(),
                    "name": s.name,
                    "location": s.location,
                    "tick": v.tick,
                    "entity": v.seen.entity,
                    "component": v.seen.component,
                    "field": v.seen.field,
                    "value": v.seen.value,
                    "expected": v.seen.expected,
                    "message": v.seen.message,
                    "details": v.details,
                    "failures": s.failures,
                    "lastTick": s.last_failure,
                })
            })
            .collect();
        let mut uncovered = Vec::new();
        for s in self.sites.iter().filter(|s| s.kind.covers()) {
            if s.kind == Kind::Each && !s.buckets.is_empty() {
                for b in s.buckets.iter().filter(|b| b.first.is_none()) {
                    uncovered.push(json!({"kind": "each", "name": s.name, "location": s.location, "key": b.key}));
                }
            } else if s.first.is_none() {
                uncovered.push(json!({"kind": s.kind.name(), "name": s.name, "location": s.location, "ran": s.ran}));
            }
        }
        let c = self.cost;
        json!({
            "epoch": self.epoch,
            "failures": self.sites.iter().map(|s| s.failures).sum::<u64>(),
            "violations": violations,
            "uncovered": uncovered,
            // Default invariants are listed once they fail; the others always.
            "sites": self.sites.iter().filter(|s| s.origin != Origin::Default || s.failures > 0).map(site_json).collect::<Vec<_>>(),
            "sitesDropped": self.sites_dropped,
            "defaults": {
                "fields": c.fields,
                "ticks": c.ticks,
                "values": c.values,
                "totalMs": (c.total_ms * 1000.0).round() / 1000.0,
                "meanUs": if c.ticks > 0 { (c.total_ms * 1000.0 / c.ticks as f64 * 10.0).round() / 10.0 } else { 0.0 },
                "maxUs": (c.max_ms * 10000.0).round() / 10.0,
            },
        })
    }

    /// The report, and the next one started: every site stays listed,
    /// unfired, with no failures and no buckets; the epoch moves on.
    pub fn take(&mut self) -> Value {
        let report = self.report();
        for s in &mut self.sites {
            s.first = None;
            s.ran = false;
            s.failures = 0;
            s.last_failure = 0;
            s.buckets.clear();
            s.buckets_dropped = 0;
        }
        self.violations.clear();
        self.sites_dropped = 0;
        self.cost = Cost::default();
        self.epoch = self.epoch.wrapping_add(1);
        report
    }
}

impl crate::Probe {
    /// The run's assertions, shared: the host records the game's calls
    /// into it while a system runs (the probe is busy then) and its
    /// default invariants after every tick.
    pub fn assertions(&self) -> std::rc::Rc<std::cell::RefCell<Assertions>> {
        self.assertions.clone()
    }

    /// `invariant:<expectation>`: the expectation (any form of `expect:`
    /// but `region` and `frame`, which read a frame) is evaluated after
    /// every tick from now on; the first tick it fails is a violation of
    /// the playtest's `always` site named `invariant <expectation>`.
    #[cfg(not(feature = "publish"))]
    pub(crate) fn add_invariant(&mut self, args: &[String]) -> Result<(), String> {
        let text = args.join(":");
        crate::expect::parse(args)?;
        if matches!(args.first().map(String::as_str), Some("region" | "frame")) {
            return Err(format!(
                "invariant:{text}: a region or frame expectation reads a frame, which every tick cannot afford; check it with expect: at the tick it matters"
            ));
        }
        self.assertions.borrow_mut().declare(
            Origin::Playtest,
            &format!("invariant {text}"),
            "playtest",
        );
        self.invariants.push(args.to_vec());
        Ok(())
    }

    /// Every `invariant:` step, after a tick: a failure is recorded as a
    /// violation with the tick and the observed value.
    #[cfg(not(feature = "publish"))]
    pub(crate) fn check_invariants(&mut self, world: &mut og_ecs::World) {
        if self.invariants.is_empty() {
            return;
        }
        let invariants = std::mem::take(&mut self.invariants);
        self.quiet = true;
        for args in &invariants {
            let Ok(r) = self.expect_args(world, args, None) else {
                continue;
            };
            if r.status == crate::Status::Fail {
                self.assertions.borrow_mut().fail(
                    Origin::Playtest,
                    &format!("invariant {}", r.expect),
                    "playtest",
                    r.tick,
                    Seen {
                        value: r.observed,
                        expected: r.expected,
                        message: r.reason,
                        ..Seen::default()
                    },
                );
            }
        }
        self.quiet = false;
        self.invariants = invariants;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn call(kind: Kind, name: &str, condition: bool, key: &str, tick: u64) -> Call {
        Call {
            kind,
            origin: Origin::Game,
            name: name.into(),
            location: "src/main.ts:1".into(),
            condition,
            key: key.into(),
            details: Value::Null,
            tick,
            declared: false,
        }
    }

    #[test]
    fn a_sometimes_that_never_held_is_uncovered_and_take_forgets_what_fired() {
        let mut a = Assertions::default();
        a.record(&call(Kind::Sometimes, "wins", false, "", 1));
        a.record(&call(Kind::Sometimes, "jumps", true, "", 4));
        let r = a.report();
        let names: Vec<&str> = r["uncovered"]
            .as_array()
            .unwrap()
            .iter()
            .map(|u| u["name"].as_str().unwrap())
            .collect();
        assert_eq!(names, ["wins"]);
        a.take();
        let r = a.report();
        assert_eq!(
            r["uncovered"].as_array().unwrap().len(),
            2,
            "both are unfired in the next report: {r}"
        );
        assert_eq!(r["epoch"], 1);
    }

    #[test]
    fn an_always_keeps_its_first_violation_and_counts_the_rest() {
        let mut a = Assertions::default();
        a.record(&call(Kind::Always, "hp", true, "", 1));
        a.record(&call(Kind::Always, "hp", false, "", 7));
        a.record(&call(Kind::Always, "hp", false, "", 9));
        let r = a.report();
        assert_eq!(r["violations"].as_array().unwrap().len(), 1);
        assert_eq!(r["violations"][0]["tick"], 7);
        assert_eq!(r["violations"][0]["failures"], 2);
        assert_eq!(r["violations"][0]["lastTick"], 9);
    }

    #[test]
    fn each_keeps_a_bucket_per_key_and_lists_the_unheld_ones() {
        let mut a = Assertions::default();
        a.record(&call(Kind::Each, "cleared", false, "1", 1));
        a.record(&call(Kind::Each, "cleared", true, "1", 3));
        a.record(&call(Kind::Each, "cleared", false, "2", 5));
        let r = a.report();
        let u = r["uncovered"].as_array().unwrap();
        assert_eq!(u.len(), 1);
        assert_eq!(u[0]["key"], "2");
        assert_eq!(r["sites"][0]["buckets"][0]["firstTick"], 3);
    }
}
