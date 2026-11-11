//! `timeline`: what happened in the run since the last read, each with its
//! tick (IDL `probe.RunTimeline`): inputs through the probe, command steps,
//! screen changes, assertion events and expectations. What a recorder picks
//! its key frames by (`og render --keyframes`).

use crate::command::CommandStep;
use crate::expect::{ExpectResult, Status};
use og_ecs::World;
use serde_json::{Value, json};

/// The most events kept between two reads; more are counted as dropped.
pub const MAX_TIMELINE_EVENTS: usize = 4096;

/// The events since the last read, and the screen stack as last seen.
#[derive(Clone, Debug, Default)]
pub struct Recorder {
    events: Vec<Value>,
    dropped: u64,
    screens: Vec<String>,
}

impl Recorder {
    fn push(&mut self, tick: u64, kind: &str, what: String, detail: Value) {
        if self.events.len() < MAX_TIMELINE_EVENTS {
            self.events
                .push(json!({ "tick": tick, "kind": kind, "what": what, "detail": detail }));
        } else {
            self.dropped += 1;
        }
    }

    /// An event through the probe's `input`.
    pub(crate) fn input(&mut self, tick: u64, event: &Value, accepted: bool) {
        let mut what: Vec<String> = ["kind", "action", "key", "button"]
            .iter()
            .filter_map(|k| event.get(*k).and_then(Value::as_str))
            .map(str::to_owned)
            .collect();
        if let (Some(x), Some(y)) = (
            event.get("x").and_then(Value::as_f64),
            event.get("y").and_then(Value::as_f64),
        ) {
            what.push(format!("{},{}", x.round(), y.round()));
        }
        if !accepted {
            what.push("(refused)".into());
        }
        let mut detail = event.clone();
        if let Some(d) = detail.as_object_mut() {
            d.insert("accepted".into(), json!(accepted));
        }
        self.push(tick, "input", what.join(" "), detail);
    }

    /// A command step about to run.
    pub(crate) fn step(&mut self, tick: u64, step: &CommandStep) {
        let line = std::iter::once(step.action.as_str())
            .chain(step.args.iter().map(String::as_str))
            .collect::<Vec<_>>()
            .join(":");
        self.push(
            tick,
            "step",
            line,
            json!({ "at": step.at, "action": step.action, "args": step.args }),
        );
    }

    /// An expectation evaluated.
    pub(crate) fn expect(&mut self, r: &ExpectResult) {
        let status = match r.status {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Unchecked => "unchecked",
        };
        self.push(
            r.tick,
            "expect",
            format!("{status} {}", r.expect),
            r.to_json(),
        );
    }

    /// The screen stack after a tick: an event when it changed.
    pub(crate) fn screens(&mut self, tick: u64, now: Vec<String>) {
        if now == self.screens {
            return;
        }
        let before = std::mem::replace(&mut self.screens, now);
        let shown: Vec<&String> = self
            .screens
            .iter()
            .filter(|s| !before.contains(s))
            .collect();
        let hidden: Vec<&String> = before
            .iter()
            .filter(|s| !self.screens.contains(s))
            .collect();
        let name = |v: &[String]| {
            v.last()
                .cloned()
                .unwrap_or_else(|| "(no screen)".to_owned())
        };
        let what = format!("{} > {}", name(&before), name(&self.screens));
        let detail =
            json!({ "before": before, "after": self.screens, "shown": shown, "hidden": hidden });
        self.push(tick, "screen", what, detail);
    }

    fn take(&mut self) -> (Vec<Value>, u64) {
        (
            std::mem::take(&mut self.events),
            std::mem::take(&mut self.dropped),
        )
    }

    /// [`super::Probe::timeline`]: the events taken, with the assertions' among
    /// them by tick.
    pub(super) fn report(
        &mut self,
        assertions: &std::cell::RefCell<crate::assertions::Assertions>,
        world: &World,
    ) -> Value {
        let (mut events, dropped) = self.take();
        for e in assertions.borrow_mut().take_timeline() {
            let outcome = e["outcome"].as_str().unwrap_or("fired");
            let what = format!(
                "{outcome} {}{}",
                e["name"].as_str().unwrap_or(""),
                e.get("key")
                    .and_then(Value::as_str)
                    .map_or(String::new(), |k| format!(" [{k}]"))
            );
            events.push(json!({
                "tick": e["tick"],
                "kind": "assertion",
                "what": what,
                "detail": e,
            }));
        }
        // By tick, keeping the order within one (a stable sort).
        events.sort_by_key(|e| e["tick"].as_u64().unwrap_or(0));
        json!({
            "tick": world.time().tick,
            "screens": self.screens,
            "events": events,
            "dropped": dropped,
        })
    }
}
