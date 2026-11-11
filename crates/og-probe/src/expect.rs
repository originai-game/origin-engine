//! Expectations: assertions a script or a probe program makes about the
//! world, the frame and the UI as it runs (`expect:` actions, the `expect`
//! program operation; docs/design.md §3.4). Each is evaluated at the tick it
//! runs and recorded with that tick, what was expected and what was
//! observed; a host reports them (`og run --json`'s `expectations`) and
//! fails the run on any failure.
//!
//! ```text
//! expect:entity:<id>:within:x,y,w,h            its visible box (describe) inside this CSS-pixel box
//! expect:entity:<id>:within:x0,y0,z0,x1,y1,z1  its Transform.position inside this world box
//! expect:event:<Name>[:count]                  events of that type sent since the script began: at least one, or exactly count
//! expect:ui:text:[=]<text>                     a UI element shows the text (a substring, exactly with `=`)
//! expect:region:x,y,w,h:color:#rrggbb[:tol]    the region's mean colour (CSS pixels) within tol per channel (0-255, default 16)
//! expect:frame:<ref.png>:rmse:<=v              the frame against a reference PNG: RGB root-mean-square difference (0-255) at most v
//! expect:state:<path><op><value>               a fact of state().facts by dotted path: == != < <= > >=; value as JSON, else text
//! expect:entity:<id>:<path><op><value>         a field of the entity as dump_entity shows it (`render.lod==2`, `og.Transform.position.y>0`)
//! expect:timings:<path><op><value>             state().timings by dotted path (`triangles<=1500000`)
//! ```
//!
//! A frame expectation without a renderer is `unchecked` (never a failure),
//! as an unchecked screenshot is; so are an entity's `render.*` fields and
//! the renderer's timings (`triangles`, `render`, `passes`).

use crate::frame::{Rgba, decode_png};
use crate::hooks::find_by_text;
use crate::probe::Probe;
use og_ecs::World;
use serde_json::{Value, json};

/// How an expectation came out.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    Pass,
    Fail,
    /// It needs a frame and the host has no renderer.
    Unchecked,
}

impl Status {
    pub fn name(self) -> &'static str {
        match self {
            Status::Pass => "pass",
            Status::Fail => "fail",
            Status::Unchecked => "unchecked",
        }
    }
}

/// One evaluated expectation (IDL `probe.ExpectResult`).
#[derive(Clone, Debug, PartialEq)]
pub struct ExpectResult {
    /// The tick it was evaluated at.
    pub tick: u64,
    /// The script step's time, when a script ran it.
    pub at: Option<f64>,
    /// As written, after `expect:`.
    pub expect: String,
    /// `entity`, `event`, `ui`, `region`, `frame` or `state`.
    pub kind: String,
    pub status: Status,
    pub expected: Value,
    pub observed: Value,
    /// Why it failed (or was unchecked), naming what to look at.
    pub reason: String,
}

impl ExpectResult {
    pub fn to_json(&self) -> Value {
        json!({
            "tick": self.tick,
            "at": self.at,
            "expect": self.expect,
            "kind": self.kind,
            "status": self.status.name(),
            "expected": self.expected,
            "observed": self.observed,
            "reason": self.reason,
        })
    }

    /// The log line: `expect <text>: pass` or `... FAIL (observed ...): reason`.
    pub fn line(&self) -> String {
        match self.status {
            Status::Pass => format!("expect {}: pass (observed {})", self.expect, self.observed),
            Status::Fail => format!(
                "expect {}: FAIL at tick {}: expected {}, observed {}; {}",
                self.expect, self.tick, self.expected, self.observed, self.reason
            ),
            Status::Unchecked => format!("expect {}: unchecked ({})", self.expect, self.reason),
        }
    }
}

/// A comparison of `state`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum Op {
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

impl Op {
    fn text(self) -> &'static str {
        match self {
            Op::Eq => "==",
            Op::Ne => "!=",
            Op::Lt => "<",
            Op::Le => "<=",
            Op::Gt => ">",
            Op::Ge => ">=",
        }
    }
}

/// What a comparison compares against: a value as written, or `$<path>`,
/// another path of the same source read when it is evaluated
/// (`state:coins<=$total`).
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Rhs {
    Value(Value),
    Path(String),
}

impl From<Value> for Rhs {
    fn from(v: Value) -> Rhs {
        Rhs::Value(v)
    }
}

impl std::fmt::Display for Rhs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Rhs::Value(v) => write!(f, "{v}"),
            Rhs::Path(p) => write!(f, "${p}"),
        }
    }
}

/// A parsed expectation.
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum Expect {
    Within {
        id: String,
        bounds: Vec<f32>,
    },
    Event {
        name: String,
        count: Option<u64>,
    },
    UiText {
        query: String,
    },
    Region {
        rect: [f32; 4],
        color: [u8; 3],
        tolerance: f32,
    },
    Frame {
        reference: String,
        rmse: f64,
    },
    State {
        path: String,
        op: Op,
        value: Rhs,
    },
    /// A field of an entity's dump (`dump_entity`).
    Field {
        id: String,
        path: String,
        op: Op,
        value: Rhs,
    },
    /// The probe's timings (`state().timings`).
    Timings {
        path: String,
        op: Op,
        value: Rhs,
    },
}

const USAGE: &str = "write `expect:entity:<id>:within:x,y,w,h`, `expect:entity:<id>:<path>==<value>`, `expect:event:<Name>[:count]`, `expect:ui:text:<text>`, `expect:region:x,y,w,h:color:#rrggbb[:tolerance]`, `expect:frame:<ref.png>:rmse:<=v`, `expect:state:<path>==<value>` or `expect:timings:<path>==<value>`, where a value may be `$<path>`, another path of the same source";

/// `<path><op><value>`: the dotted path, the comparison and the value (JSON,
/// else text; `$<path>` another path of the same source). `form` names the
/// expectation in the error.
fn comparison(text: &str, form: &str) -> Result<(String, Op, Rhs), String> {
    let ops = [
        ("==", Op::Eq),
        ("!=", Op::Ne),
        ("<=", Op::Le),
        (">=", Op::Ge),
        ("<", Op::Lt),
        (">", Op::Gt),
    ];
    let (at, len, op) = ops
        .iter()
        .filter_map(|(t, op)| text.find(t).map(|i| (i, t.len(), *op)))
        .min_by_key(|(i, len, _)| (*i, usize::MAX - len))
        .ok_or_else(|| format!("'{text}' has no comparison (== != < <= > >=)"))?;
    let path = text[..at].trim().to_owned();
    let raw = text[at + len..].trim();
    if path.is_empty() || raw.is_empty() {
        return Err(format!("write `expect:{form}<path>{}<value>`", op.text()));
    }
    let value = match raw.strip_prefix('$') {
        Some(other) if !other.trim().is_empty() => Rhs::Path(other.trim().to_owned()),
        _ => {
            Rhs::Value(serde_json::from_str(raw).unwrap_or_else(|_| Value::String(raw.to_owned())))
        }
    };
    Ok((path, op, value))
}

fn numbers(text: &str) -> Result<Vec<f32>, String> {
    text.split(',')
        .map(|t| {
            t.trim()
                .parse::<f32>()
                .ok()
                .filter(|v| v.is_finite())
                .ok_or_else(|| format!("'{}' is not a number", t.trim()))
        })
        .collect()
}

fn hex_color(text: &str) -> Result<[u8; 3], String> {
    let h = text.trim().trim_start_matches('#');
    if h.len() != 6 {
        return Err(format!("'{text}' is not a colour; write #rrggbb"));
    }
    let byte = |i: usize| {
        u8::from_str_radix(&h[i..i + 2], 16).map_err(|_| format!("'{text}' is not a colour"))
    };
    Ok([byte(0)?, byte(2)?, byte(4)?])
}

/// Parses an expectation from its arguments (the step's, after `expect`).
pub(crate) fn parse(args: &[String]) -> Result<Expect, String> {
    let kind = args.first().map(String::as_str).unwrap_or("");
    let rest = |from: usize| args.get(from..).map(|a| a.join(":")).unwrap_or_default();
    match kind {
        "entity" => {
            let id = args
                .get(1)
                .filter(|s| !s.is_empty())
                .ok_or("expect:entity needs the entity's id")?;
            if args.get(2).map(String::as_str) != Some("within") {
                let (path, op, value) = comparison(&rest(2), &format!("entity:{id}:"))
                    .map_err(|e| format!("expect:entity:{id}: {e}; {USAGE}"))?;
                return Ok(Expect::Field {
                    id: id.clone(),
                    path,
                    op,
                    value,
                });
            }
            let bounds = numbers(&rest(3))?;
            if bounds.len() != 4 && bounds.len() != 6 {
                return Err(
                    "within takes x,y,w,h (CSS pixels) or x0,y0,z0,x1,y1,z1 (world units)".into(),
                );
            }
            Ok(Expect::Within {
                id: id.clone(),
                bounds,
            })
        }
        "event" => {
            let name = args
                .get(1)
                .filter(|s| !s.is_empty())
                .ok_or("expect:event needs the event's type name")?
                .clone();
            let count = match args.get(2) {
                None => None,
                Some(c) => Some(
                    c.trim()
                        .parse::<u64>()
                        .map_err(|_| format!("'{c}' is not a count of events"))?,
                ),
            };
            Ok(Expect::Event { name, count })
        }
        "ui" => {
            if args.get(1).map(String::as_str) != Some("text") {
                return Err(format!("expect:ui: {USAGE}"));
            }
            let query = rest(2);
            if query.is_empty() {
                return Err("expect:ui:text needs the text".into());
            }
            Ok(Expect::UiText { query })
        }
        "region" => {
            let r = numbers(args.get(1).map_or("", String::as_str))?;
            let [x, y, w, h] = r[..] else {
                return Err("region takes x,y,w,h in CSS pixels".into());
            };
            if args.get(2).map(String::as_str) != Some("color") {
                return Err(format!("expect:region: {USAGE}"));
            }
            let color = hex_color(args.get(3).map_or("", String::as_str))?;
            let tolerance = match args.get(4) {
                None => 16.0,
                Some(t) => t
                    .trim()
                    .parse::<f32>()
                    .ok()
                    .filter(|t| t.is_finite() && *t >= 0.0)
                    .ok_or_else(|| format!("'{t}' is not a tolerance (0 to 255)"))?,
            };
            if w <= 0.0 || h <= 0.0 {
                return Err("the region needs a width and a height above 0".into());
            }
            Ok(Expect::Region {
                rect: [x, y, w, h],
                color,
                tolerance,
            })
        }
        "frame" => {
            let reference = args
                .get(1)
                .filter(|s| !s.is_empty())
                .ok_or("expect:frame needs the reference PNG")?
                .clone();
            let tail = rest(2);
            let v = tail
                .trim()
                .trim_start_matches("rmse")
                .trim_start_matches(':')
                .trim()
                .trim_start_matches("<=")
                .trim_start_matches(':')
                .trim();
            let rmse = v
                .parse::<f64>()
                .ok()
                .filter(|v| v.is_finite() && *v >= 0.0)
                .ok_or_else(|| format!("'{tail}' is not `rmse:<=v`; {USAGE}"))?;
            Ok(Expect::Frame { reference, rmse })
        }
        "state" => {
            let (path, op, value) = comparison(&rest(1), "state:")?;
            Ok(Expect::State { path, op, value })
        }
        "timings" => {
            let (path, op, value) = comparison(&rest(1), "timings:")?;
            Ok(Expect::Timings { path, op, value })
        }
        _ => Err(format!("'{kind}' is not an expectation; {USAGE}")),
    }
}

/// Records `value <op> expected` at `path` of `of` (named `what`).
fn check_path(
    r: &mut ExpectResult,
    of: &Value,
    what: &str,
    prefix: &str,
    path: &str,
    op: Op,
    expected: &Rhs,
) {
    let rhs = match expected {
        Rhs::Value(v) => Some(v),
        Rhs::Path(p) => at_path(of, p),
    };
    check_value(r, at_path(of, path), rhs, what, prefix, path, op, expected);
}

/// Records `at <op> rhs`: `rhs` is the expected value, read from `expected`'s
/// path when it names one (None: the source has no such path).
#[allow(clippy::too_many_arguments)]
fn check_value(
    r: &mut ExpectResult,
    at: Option<&Value>,
    rhs: Option<&Value>,
    what: &str,
    prefix: &str,
    path: &str,
    op: Op,
    expected: &Rhs,
) {
    r.expected = match (expected, rhs) {
        (Rhs::Path(p), Some(v)) => json!(format!("{} {v} (${p})", op.text())),
        _ => json!(format!("{} {expected}", op.text())),
    };
    let Some(rhs) = rhs else {
        r.observed = at.cloned().unwrap_or(Value::Null);
        r.status = Status::Fail;
        r.reason = format!(
            "{what} has no '{}'",
            expected.to_string().trim_start_matches('$')
        );
        return;
    };
    match at {
        None => {
            r.observed = Value::Null;
            r.status = Status::Fail;
            r.reason = format!("{what} has no '{path}'");
        }
        Some(v) => {
            r.observed = v.clone();
            if !compare(v, op, rhs) {
                r.status = Status::Fail;
                r.reason = match expected {
                    Rhs::Path(p) => {
                        format!(
                            "{prefix}{path} {} {prefix}{p} ({rhs}) does not hold",
                            op.text()
                        )
                    }
                    Rhs::Value(_) => format!("{prefix}{path} {} {rhs} does not hold", op.text()),
                };
            }
        }
    }
}

/// A field of an entity's dump by dotted path, where a component's name
/// may itself hold dots (`og.Transform.position.y`): the longest key that
/// matches is taken at each level.
fn dump_path<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    if path.is_empty() {
        return Some(value);
    }
    let Value::Object(m) = value else {
        return at_path(value, path);
    };
    let parts: Vec<&str> = path.split('.').collect();
    for n in (1..=parts.len()).rev() {
        let key = parts[..n].join(".");
        if let Some(v) = m.get(&key) {
            return dump_path(v, &parts[n..].join("."));
        }
    }
    None
}

/// A fact by dotted path (`a.b.0`).
fn at_path<'v>(value: &'v Value, path: &str) -> Option<&'v Value> {
    path.split('.').try_fold(value, |v, key| match v {
        Value::Object(m) => m.get(key),
        // A vector's lane by name, as the API writes it (`position.y`): dumps hold vectors as arrays.
        Value::Array(a) => a.get(match key {
            "x" | "r" => 0,
            "y" | "g" => 1,
            "z" | "b" => 2,
            "w" => 3,
            _ => key.parse::<usize>().ok()?,
        }),
        _ => None,
    })
}

fn compare(observed: &Value, op: Op, expected: &Value) -> bool {
    if let (Some(a), Some(b)) = (observed.as_f64(), expected.as_f64()) {
        return match op {
            Op::Eq => a == b,
            Op::Ne => a != b,
            Op::Lt => a < b,
            Op::Le => a <= b,
            Op::Gt => a > b,
            Op::Ge => a >= b,
        };
    }
    match op {
        Op::Eq => observed == expected,
        Op::Ne => observed != expected,
        _ => match (observed.as_str(), expected.as_str()) {
            (Some(a), Some(b)) => match op {
                Op::Lt => a < b,
                Op::Le => a <= b,
                Op::Gt => a > b,
                _ => a >= b,
            },
            _ => false,
        },
    }
}

/// The RGB root-mean-square difference of two images of one size, 0 to 255.
pub(crate) fn rmse(a: &Rgba, b: &Rgba) -> Option<f64> {
    if a.width != b.width || a.height != b.height {
        return None;
    }
    let mut sum = 0f64;
    for (p, q) in a
        .pixels
        .as_chunks::<4>()
        .0
        .iter()
        .zip(b.pixels.as_chunks::<4>().0)
    {
        for c in 0..3 {
            let d = f64::from(p[c]) - f64::from(q[c]);
            sum += d * d;
        }
    }
    let n = f64::from(a.width) * f64::from(a.height) * 3.0;
    Some((sum / n).sqrt())
}

fn round(v: f64, digits: i32) -> f64 {
    let k = 10f64.powi(digits);
    (v * k).round() / k
}

impl Probe {
    /// Evaluates one expectation now (`expect` member; an `expect:` step).
    /// `Err` is a malformed expectation (nothing was evaluated).
    pub fn expect(&mut self, world: &mut World, what: &str) -> Result<ExpectResult, String> {
        let args: Vec<String> = what.split(':').map(str::to_owned).collect();
        self.expect_args(world, &args, None)
    }

    pub(crate) fn expect_args(
        &mut self,
        world: &mut World,
        args: &[String],
        at: Option<f64>,
    ) -> Result<ExpectResult, String> {
        let parsed = parse(args)?;
        let tick = world.time().tick;
        let text = args.join(":");
        let kind = args.first().cloned().unwrap_or_default();
        let mut r = ExpectResult {
            tick,
            at,
            expect: text,
            kind,
            status: Status::Pass,
            expected: Value::Null,
            observed: Value::Null,
            reason: String::new(),
        };
        self.evaluate(world, parsed, &mut r);
        Ok(self.checks.record(&mut self.out, &mut self.timeline, r))
    }

    /// Evaluates `parsed` on the world as it is into `r`, which opens as a
    /// pass: its expected and observed values, and the status and reason
    /// when it fails or cannot be checked.
    fn evaluate(&mut self, world: &mut World, parsed: Expect, r: &mut ExpectResult) {
        match parsed {
            Expect::Within { id, bounds } if bounds.len() == 4 => {
                r.expected = json!({"x": bounds[0], "y": bounds[1], "width": bounds[2], "height": bounds[3]});
                let Some(e) = self.names.resolve(world, &id) else {
                    fail(r, format!("no live entity '{id}'"));
                    return;
                };
                let seen = self.host.seen_now(world);
                let Some(seen) = seen else {
                    r.status = Status::Unchecked;
                    r.reason = "this host has no scene view to describe the frame".into();
                    return;
                };
                match seen.entities.iter().find(|s| s.entity == e) {
                    None => fail(r, format!("'{id}' shows no pixel in the frame")),
                    Some(s) => {
                        let [x, y, w, h] = s.rect;
                        r.observed = json!({"x": round(x.into(), 2), "y": round(y.into(), 2), "width": round(w.into(), 2), "height": round(h.into(), 2)});
                        // Half a CSS pixel of slack: the id pass is a quarter of the frame.
                        let slack = 0.5;
                        let inside = x >= bounds[0] - slack
                            && y >= bounds[1] - slack
                            && x + w <= bounds[0] + bounds[2] + slack
                            && y + h <= bounds[1] + bounds[3] + slack;
                        if !inside {
                            fail(r, format!("'{id}'s visible box leaves the expected box"));
                        }
                    }
                }
            }
            Expect::Within { id, bounds } => {
                r.expected = json!({"min": &bounds[..3], "max": &bounds[3..]});
                let e = match self.names.resolve(world, &id) {
                    Some(e) => e,
                    None => {
                        fail(r, format!("no live entity '{id}'"));
                        return;
                    }
                };
                match self.position_of(world, e) {
                    Err(reason) => fail(r, reason),
                    Ok(p) => {
                        r.observed = json!([
                            round(p[0].into(), 4),
                            round(p[1].into(), 4),
                            round(p[2].into(), 4)
                        ]);
                        let inside = (0..3).all(|i| p[i] >= bounds[i] && p[i] <= bounds[i + 3]);
                        if !inside {
                            fail(r, format!("'{id}'s Transform.position is outside the box"));
                        }
                    }
                }
            }
            Expect::Event { name, count } => {
                r.expected = count.map_or(json!(">= 1"), |c| json!(c));
                if world.id_by_name(&name).is_err() {
                    fail(r, format!("no event type '{name}' is registered"));
                    return;
                }
                let seen = self.checks.events_seen(&name);
                r.observed = json!(seen);
                let ok = match count {
                    Some(c) => seen == c,
                    None => seen >= 1,
                };
                if !ok {
                    fail(
                        r,
                        format!("{seen} {name} event(s) were sent since the script began"),
                    );
                }
            }
            Expect::UiText { query } => {
                r.expected = json!(query);
                let elements = self.host.elements(world);
                match find_by_text(&elements, &query) {
                    Some(e) => r.observed = json!({"id": e.id, "text": e.text}),
                    None => {
                        let texts: Vec<&str> = elements
                            .iter()
                            .filter(|e| !e.text.is_empty())
                            .map(|e| e.text.as_str())
                            .collect();
                        r.observed = json!(texts);
                        fail(
                            r,
                            format!(
                                "no UI element shows it ({} elements; `dump_ui` lists them)",
                                elements.len()
                            ),
                        );
                    }
                }
            }
            Expect::Region {
                rect,
                color,
                tolerance,
            } => self.host.expect_region(world, rect, color, tolerance, r),
            Expect::Frame {
                reference,
                rmse: limit,
            } => self.host.expect_reference(world, reference, limit, r),
            Expect::State { path, op, value } => {
                let facts = self.host.facts(world);
                check_path(r, &facts, "state().facts", "facts.", &path, op, &value);
            }
            Expect::Timings { path, op, value } => {
                // What the renderer measures, without one: unchecked.
                if !self.host.has_renderer()
                    && ["triangles", "render", "passes"]
                        .contains(&path.split('.').next().unwrap_or(""))
                {
                    r.status = Status::Unchecked;
                    r.reason = "no renderer, timings.".to_string() + &path + " unchecked";
                    return;
                }
                let state = self.state(world);
                check_path(
                    r,
                    &state["timings"],
                    "state().timings",
                    "timings.",
                    &path,
                    op,
                    &value,
                );
            }
            Expect::Field {
                id,
                path,
                op,
                value,
            } => {
                let Some(e) = self.names.resolve(world, &id) else {
                    fail(r, format!("no live entity '{id}'"));
                    return;
                };
                // The renderer's observation (render.lod), without one: unchecked.
                if !self.host.has_renderer() && path.split('.').next() == Some("render") {
                    r.status = Status::Unchecked;
                    r.reason = format!("no renderer, {path} unchecked");
                    return;
                }
                let mut dump = self.dump_entity(world, e);
                // `og.Transform.position` as well as `components.og.Transform...`.
                let found = at_path(&dump, &path).is_some();
                if !found && let Some(components) = dump["components"].as_object().cloned() {
                    for (name, fields) in components {
                        dump[name] = fields;
                    }
                }
                let at = dump_path(&dump, &path);
                let rhs = match &value {
                    Rhs::Value(v) => Some(v),
                    Rhs::Path(p) => dump_path(&dump, p),
                };
                check_value(
                    r,
                    at,
                    rhs,
                    &format!("dump_entity of '{id}'"),
                    "",
                    &path,
                    op,
                    &value,
                );
            }
        }
    }

    /// Every `invariant:` step, after tick `ran`: a failure is recorded as a
    /// violation at `ran` (the tick whose systems produced the state, as
    /// og.assert and the default invariants number it) with the observed value.
    #[cfg(not(feature = "publish"))]
    pub(crate) fn check_invariants(&mut self, world: &mut og_ecs::World, ran: u64) {
        if self.checks.invariants.is_empty() {
            return;
        }
        let invariants = std::mem::take(&mut self.checks.invariants);
        self.checks.quiet = true;
        for args in &invariants {
            let Ok(r) = self.expect_args(world, args, None) else {
                continue;
            };
            self.checks.invariant_checked(ran, r);
        }
        self.checks.quiet = false;
        self.checks.invariants = invariants;
    }

    /// Every expectation evaluated since the last call, in order.
    pub fn take_expectations(&mut self) -> Vec<ExpectResult> {
        std::mem::take(&mut self.checks.expectations)
    }
}

impl crate::probe::state::Host {
    /// `expect:region`: the mean colour of a rect of the frame (CSS px).
    fn expect_region(
        &mut self,
        world: &World,
        rect: [f32; 4],
        color: [u8; 3],
        tolerance: f32,
        r: &mut ExpectResult,
    ) {
        r.expected = json!({"color": format!("#{:02x}{:02x}{:02x}", color[0], color[1], color[2]), "tolerance": tolerance});
        let Some(image) = self.expect_frame(world, r) else {
            return;
        };
        let scale = self.css_scale(world, &image);
        let x0 = ((rect[0] * scale).floor().max(0.0) as u32).min(image.width);
        let y0 = ((rect[1] * scale).floor().max(0.0) as u32).min(image.height);
        let x1 = (((rect[0] + rect[2]) * scale).ceil().max(0.0) as u32).min(image.width);
        let y1 = (((rect[1] + rect[3]) * scale).ceil().max(0.0) as u32).min(image.height);
        if x1 <= x0 || y1 <= y0 {
            fail(r, "the region is outside the frame".into());
            return;
        }
        let mut sum = [0f64; 3];
        for y in y0..y1 {
            for x in x0..x1 {
                let o = ((y * image.width + x) * 4) as usize;
                for (c, total) in sum.iter_mut().enumerate() {
                    *total += f64::from(image.pixels[o + c]);
                }
            }
        }
        let n = f64::from((x1 - x0) * (y1 - y0));
        let mean = sum.map(|s| (s / n).round() as u8);
        r.observed = json!({"color": format!("#{:02x}{:02x}{:02x}", mean[0], mean[1], mean[2])});
        let off = (0..3)
            .map(|c| (f32::from(mean[c]) - f32::from(color[c])).abs())
            .fold(0.0, f32::max);
        if off > tolerance {
            fail(
                r,
                format!("the region's mean colour is {off} off in a channel"),
            );
        }
    }

    /// `expect:frame`: the frame against a reference file, by RMSE.
    fn expect_reference(
        &mut self,
        world: &World,
        reference: String,
        limit: f64,
        r: &mut ExpectResult,
    ) {
        r.expected = json!({"reference": reference, "rmse": format!("<= {limit}")});
        let reference_png = match self.reader.as_ref() {
            Some(read) => read(&reference),
            None => None,
        };
        let Some(bytes) = reference_png else {
            fail(
                r,
                format!("cannot read the reference '{reference}' (a path in the project)"),
            );
            return;
        };
        let wanted = match decode_png(&bytes) {
            Ok(img) => img,
            Err(e) => {
                fail(r, format!("{reference}: {e}"));
                return;
            }
        };
        let Some(image) = self.frame_rgba_sized(world, r, wanted.width, wanted.height) else {
            return;
        };
        match rmse(&image, &wanted) {
            None => fail(r, "the frame and the reference differ in size".into()),
            Some(v) => {
                r.observed = json!({"rmse": round(v, 3)});
                if v > limit {
                    fail(
                        r,
                        format!("the frame is {} RMSE from {reference}", round(v, 3)),
                    );
                }
            }
        }
    }

    /// The frame at the target's size, or `None` with the result marked
    /// unchecked (no renderer) or failed (no frame).
    fn expect_frame(&mut self, world: &World, r: &mut ExpectResult) -> Option<Rgba> {
        self.frame_rgba_sized(world, r, 0, 0)
    }

    fn frame_rgba_sized(
        &mut self,
        world: &World,
        r: &mut ExpectResult,
        width: u32,
        height: u32,
    ) -> Option<Rgba> {
        let Some(capture) = self.capture.as_mut().filter(|c| c.has_renderer()) else {
            r.status = Status::Unchecked;
            r.reason = "no renderer, frame unchecked".into();
            return None;
        };
        let Some(frame) = capture.capture(world, width, height) else {
            r.status = Status::Fail;
            r.reason = match capture.capture_error() {
                Some(why) => format!("the renderer produced no frame: {why}"),
                None => "the renderer produced no frame".into(),
            };
            return None;
        };
        match decode_png(&frame.png) {
            Ok(img) => Some(img),
            Err(e) => {
                r.status = Status::Fail;
                r.reason = format!("the frame does not decode: {e}");
                None
            }
        }
    }

    /// Frame pixels per CSS pixel.
    fn css_scale(&self, world: &World, image: &Rgba) -> f32 {
        self.surface_scale(world, image.width)
    }
}

impl crate::probe::state::Checks {
    fn record(
        &mut self,
        out: &mut crate::probe::state::Out,
        timeline: &mut crate::probe::timeline::Recorder,
        r: ExpectResult,
    ) -> ExpectResult {
        // An invariant's evaluation, every tick: its result is the caller's alone.
        if self.quiet {
            return r;
        }
        out.note(r.tick, r.status == Status::Fail, r.line());
        timeline.expect(&r);
        self.expectations.push(r.clone());
        r
    }

    /// Events of `name` counted since the script that asked began.
    fn events_seen(&self, name: &str) -> u64 {
        let total = self.events.get(name).map_or(0, |w| w.total);
        total - self.event_base.get(name).copied().unwrap_or(0).min(total)
    }

    /// Starts counting the events the expectations among `steps` name, from now.
    pub(crate) fn watch_events<'a>(&mut self, names: impl Iterator<Item = &'a str>) {
        for name in names {
            let w = self.events.entry(name.to_owned()).or_default();
            self.event_base.insert(name.to_owned(), w.total);
        }
    }

    /// Counts the watched events sent since the last count (`before`: by
    /// the host between ticks; after: by the tick's systems).
    pub(crate) fn count_events(&mut self, world: &World, before: bool) {
        for (name, w) in &mut self.events {
            let Ok(id) = world.id_by_name(name) else {
                continue;
            };
            let n = world.pending_events_bytes(id).map_or(0, |p| p.len() as u64);
            if before {
                w.total += n.saturating_sub(w.after);
            } else {
                w.total += n;
                w.after = n;
            }
        }
    }
}

fn fail(r: &mut ExpectResult, reason: String) {
    r.status = Status::Fail;
    r.reason = reason;
}

/// One watched event type's count.
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Watch {
    pub total: u64,
    /// Pending after the last step (already counted).
    pub after: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A component name with dots is one key of an entity's dump.
    #[test]
    fn dump_paths_take_dotted_component_names() {
        let dump = json!({"render": {"lod": 2}, "og.Transform": {"position": {"y": 1.5}}});
        assert_eq!(dump_path(&dump, "render.lod"), Some(&json!(2)));
        assert_eq!(
            dump_path(&dump, "og.Transform.position.y"),
            Some(&json!(1.5))
        );
        assert_eq!(dump_path(&dump, "og.Mesh.x"), None);
        let arrays = json!({"og.Transform": {"position": [1.0, 2.5, 3.0]}});
        assert_eq!(
            dump_path(&arrays, "og.Transform.position.y"),
            Some(&json!(2.5))
        );
        assert_eq!(
            dump_path(&arrays, "og.Transform.position.2"),
            Some(&json!(3.0))
        );
    }

    fn args(s: &str) -> Vec<String> {
        s.split(':').map(str::to_owned).collect()
    }

    #[test]
    fn parses_every_form() {
        assert_eq!(
            parse(&args("entity:crate:within:0,0,100,50")).unwrap(),
            Expect::Within {
                id: "crate".into(),
                bounds: vec![0.0, 0.0, 100.0, 50.0]
            }
        );
        assert!(matches!(
            parse(&args("entity:crate:within:-1,0.4,-1,1,0.6,1")).unwrap(),
            Expect::Within { bounds, .. } if bounds.len() == 6
        ));
        assert_eq!(
            parse(&args("event:Hit:2")).unwrap(),
            Expect::Event {
                name: "Hit".into(),
                count: Some(2)
            }
        );
        assert_eq!(
            parse(&args("ui:text:=Score: 3")).unwrap(),
            Expect::UiText {
                query: "=Score: 3".into()
            }
        );
        assert_eq!(
            parse(&args("region:10,10,4,4:color:#ff8000:8")).unwrap(),
            Expect::Region {
                rect: [10.0, 10.0, 4.0, 4.0],
                color: [255, 128, 0],
                tolerance: 8.0
            }
        );
        assert_eq!(
            parse(&args("frame:refs/a.png:rmse:<=2.5")).unwrap(),
            Expect::Frame {
                reference: "refs/a.png".into(),
                rmse: 2.5
            }
        );
        assert_eq!(
            parse(&args("state:guard==\"patrol\"")).unwrap(),
            Expect::State {
                path: "guard".into(),
                op: Op::Eq,
                value: json!("patrol").into()
            }
        );
        assert_eq!(
            parse(&args("state:crateY<=0.6")).unwrap(),
            Expect::State {
                path: "crateY".into(),
                op: Op::Le,
                value: json!(0.6).into()
            }
        );
        assert_eq!(
            parse(&args("entity:tree-1990:render.lod==2")).unwrap(),
            Expect::Field {
                id: "tree-1990".into(),
                path: "render.lod".into(),
                op: Op::Eq,
                value: json!(2).into()
            }
        );
        assert_eq!(
            parse(&args("timings:triangles<=1500000")).unwrap(),
            Expect::Timings {
                path: "triangles".into(),
                op: Op::Le,
                value: json!(1500000).into()
            }
        );
        assert_eq!(
            parse(&args("state:coins<=$total")).unwrap(),
            Expect::State {
                path: "coins".into(),
                op: Op::Le,
                value: Rhs::Path("total".into())
            }
        );
        assert!(parse(&args("entity:crate:nothing")).is_err());
        assert!(parse(&args("state:started")).is_err());
        assert!(parse(&args("region:1,2:color:#fff")).is_err());
        assert!(parse(&args("nothing")).is_err());
    }

    /// `$total` is the facts' total read at evaluation, not the text "total".
    #[test]
    fn a_dollar_path_compares_against_another_field() {
        let eval = |facts: Value, what: &str| {
            let Expect::State { path, op, value } = parse(&args(what)).unwrap() else {
                panic!("{what} is a state expectation");
            };
            let mut r = ExpectResult {
                tick: 1,
                at: None,
                expect: what.into(),
                kind: "state".into(),
                status: Status::Pass,
                expected: Value::Null,
                observed: Value::Null,
                reason: String::new(),
            };
            check_path(&mut r, &facts, "state().facts", "facts.", &path, op, &value);
            r
        };
        assert_eq!(
            eval(json!({"coins": 1, "total": 2}), "state:coins<=$total").status,
            Status::Pass
        );
        let r = eval(json!({"coins": 3, "total": 2}), "state:coins<=$total");
        assert_eq!(r.status, Status::Fail, "{r:?}");
        assert_eq!(r.observed, json!(3));
        assert!(r.reason.contains("facts.total (2)"), "{}", r.reason);
        let r = eval(json!({"coins": 1}), "state:coins<=$total");
        assert_eq!(r.status, Status::Fail);
        assert!(r.reason.contains("has no 'total'"), "{}", r.reason);
    }

    #[test]
    fn compares_numbers_and_values() {
        assert!(compare(&json!(1.5), Op::Gt, &json!(1)));
        assert!(compare(&json!(true), Op::Eq, &json!(true)));
        assert!(!compare(&json!("search"), Op::Eq, &json!("patrol")));
        assert!(compare(&json!(null), Op::Ne, &json!(0)));
        assert_eq!(
            at_path(&json!({"a": {"b": [3, 4]}}), "a.b.1"),
            Some(&json!(4))
        );
    }
}
