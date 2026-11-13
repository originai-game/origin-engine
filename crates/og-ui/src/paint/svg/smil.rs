//! SMIL animation (SVG 1.1 §19, SMIL 3.0 timing): `<animate>`, `<set>`,
//! `<animateTransform>` and `<animateMotion>`, sampled at a time in
//! seconds. Presentation only: the caller reads the clock and applies the
//! values it returns; nothing here keeps state.

use super::*;

/// The tags this module reads.
pub const TAGS: &[&str] = &["animate", "set", "animateTransform", "animateMotion"];

/// The attributes an animation element takes.
const ATTRS: &[&str] = &[
    "id",
    "attributeName",
    "attributeType",
    "begin",
    "dur",
    "repeatCount",
    "repeatDur",
    "fill",
    "restart",
    "values",
    "from",
    "to",
    "by",
    "keyTimes",
    "keySplines",
    "calcMode",
    "additive",
    "accumulate",
    "type",
    "path",
    "rotate",
];

/// When an interval begins: an offset from the clock's start, or another
/// animation's begin or end (`id.begin+1s`).
#[derive(Clone, Debug, PartialEq)]
pub enum Begin {
    At(f32),
    Sync { id: String, end: bool, offset: f32 },
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Calc {
    Discrete,
    Linear,
    Paced,
    Spline,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Rotate {
    Auto,
    AutoReverse,
    Angle(f32),
}

/// What an animation drives.
#[derive(Clone, Debug)]
pub enum Target {
    /// An attribute or presentation property by its SVG name.
    Attr(String),
    /// `transform`, by `<animateTransform type>`.
    Transform(&'static str),
    /// The element's position along a path (`<animateMotion>`), as a
    /// transform before its own.
    Motion { path: Option<Path>, rotate: Rotate },
}

/// One value: numbers in a template (`"M 0 0 L 10 5"`, `"3px"`), a
/// color, or text that does not interpolate.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Nums { nums: Vec<f32>, text: Vec<String> },
    Color([f32; 4]),
    Text(String),
}

/// A value as the animation names it: its own, the underlying value
/// (a to-animation's start), or the underlying value plus one (`by`).
#[derive(Clone, Debug, PartialEq)]
enum Item {
    Base,
    Own(Value),
    BasePlus(Value),
}

/// A parsed animation element.
#[derive(Clone, Debug)]
pub struct Animation {
    pub target: Target,
    pub begins: Vec<Begin>,
    /// The simple duration; `None` is indefinite.
    pub dur: Option<f32>,
    repeat_count: Option<f32>,
    repeat_dur: Option<f32>,
    /// Count or duration was `indefinite`.
    repeat_forever: bool,
    pub freeze: bool,
    items: Vec<Item>,
    key_times: Option<Vec<f32>>,
    key_splines: Vec<[f32; 4]>,
    calc: Calc,
    additive: bool,
    accumulate: bool,
}

/// A SMIL clock value in seconds: `2s`, `500ms`, `1.5`, `0.5min`, `1h`,
/// `00:01.5` or `01:00:00`.
pub fn clock(v: &str) -> Result<f32, String> {
    let v = v.trim();
    let bad = || format!("\"{v}\" is not a clock value (2s, 500ms, 00:01.5)");
    let num = |s: &str| s.trim().parse::<f32>().ok().filter(|n| n.is_finite());
    if v.contains(':') {
        let parts: Vec<_> = v.split(':').collect();
        if parts.len() > 3 {
            return Err(bad());
        }
        let mut total = 0.0;
        for p in parts {
            total = total * 60.0 + num(p).ok_or_else(bad)?;
        }
        return Ok(total);
    }
    for (unit, k) in [("ms", 0.001), ("min", 60.0), ("h", 3600.0), ("s", 1.0)] {
        if let Some(n) = v.strip_suffix(unit) {
            return num(n).map(|n| n * k).ok_or_else(bad);
        }
    }
    num(v).ok_or_else(bad)
}

fn begin(v: &str) -> Result<Option<Begin>, String> {
    let v = v.trim();
    if v == "indefinite" {
        return Ok(None);
    }
    if let Ok(t) = clock(v) {
        return Ok(Some(Begin::At(t)));
    }
    // `id.begin`, `id.end`, with an optional `+clock` or `-clock`.
    let (base, offset) = match v.find(['+', '-']) {
        Some(i) => {
            let o = clock(&v[i + 1..])?;
            (&v[..i], if &v[i..=i] == "-" { -o } else { o })
        }
        None => (v, 0.0),
    };
    let base = base.trim();
    let (id, end) = if let Some(id) = base.strip_suffix(".begin") {
        (id, false)
    } else if let Some(id) = base.strip_suffix(".end") {
        (id, true)
    } else {
        return Err(format!(
            "begin \"{v}\" is a clock value, a list of them, or id.begin / id.end with an offset; an event begin (click, mouseover) is not supported: start motion from input with og.tween or a CSS state"
        ));
    };
    if id.is_empty() || id.contains(char::is_whitespace) {
        return Err(format!(
            "begin \"{v}\": no animation id before .{}",
            if end { "end" } else { "begin" }
        ));
    }
    Ok(Some(Begin::Sync {
        id: id.to_string(),
        end,
        offset,
    }))
}

/// Splits `s` into its numbers and the text around them.
fn numbers_in(s: &str) -> (Vec<f32>, Vec<String>) {
    let b = s.as_bytes();
    let mut nums = Vec::new();
    let mut text = vec![String::new()];
    let mut i = 0;
    while i < b.len() {
        let c = b[i] as char;
        let starts = c.is_ascii_digit()
            || (c == '.' && b.get(i + 1).is_some_and(u8::is_ascii_digit))
            || ((c == '-' || c == '+')
                && b.get(i + 1)
                    .is_some_and(|&n| n.is_ascii_digit() || n == b'.'));
        // A letter before a digit is a name (`h1`, `#a0b`): not a number.
        let in_word = i > 0 && (b[i - 1] as char).is_ascii_alphabetic() && c != '-' && c != '+';
        if starts && !in_word {
            let mut j = i + 1;
            let mut dot = c == '.';
            let mut exp = false;
            while j < b.len() {
                let d = b[j] as char;
                if d.is_ascii_digit() {
                    j += 1;
                } else if d == '.' && !dot && !exp {
                    dot = true;
                    j += 1;
                } else if (d == 'e' || d == 'E')
                    && !exp
                    && b.get(j + 1)
                        .is_some_and(|&n| n.is_ascii_digit() || n == b'-' || n == b'+')
                {
                    exp = true;
                    j += 2;
                } else {
                    break;
                }
            }
            if let Ok(n) = s[i..j].parse::<f32>() {
                nums.push(n);
                text.push(String::new());
                i = j;
                continue;
            }
        }
        text.last_mut().unwrap().push(c);
        i += 1;
    }
    (nums, text)
}

impl Value {
    /// Parses one value of attribute `attr`.
    pub fn parse(attr: &str, v: &str) -> Value {
        let v = v.trim();
        if matches!(attr, "fill" | "stroke" | "stop-color" | "color")
            && let Ok(c) = color(v)
        {
            return Value::Color(c);
        }
        let (nums, text) = numbers_in(v);
        if nums.is_empty() {
            Value::Text(v.to_string())
        } else {
            Value::Nums { nums, text }
        }
    }

    fn nums(&self) -> Option<&[f32]> {
        match self {
            Value::Nums { nums, .. } => Some(nums),
            Value::Color(c) => Some(c),
            Value::Text(_) => None,
        }
    }

    /// Whether `self` and `other` interpolate: numbers in the same
    /// template, or two colors.
    fn pairs(&self, other: &Value) -> bool {
        let squash = |t: &[String]| -> Vec<String> {
            t.iter()
                .map(|s| {
                    s.chars()
                        .filter(|c| !c.is_whitespace() && *c != ',')
                        .collect()
                })
                .collect()
        };
        match (self, other) {
            (Value::Nums { nums: a, text: ta }, Value::Nums { nums: b, text: tb }) => {
                a.len() == b.len() && squash(ta) == squash(tb)
            }
            (Value::Color(_), Value::Color(_)) => true,
            _ => false,
        }
    }

    /// `self + k * (other - self)` when they pair, else `self`.
    fn lerp(&self, other: &Value, k: f32) -> Value {
        if !self.pairs(other) {
            return self.clone();
        }
        let (a, b) = (self.nums().unwrap(), other.nums().unwrap());
        let n: Vec<f32> = a.iter().zip(b).map(|(a, b)| a + (b - a) * k).collect();
        self.with(n)
    }

    /// `self + k * other` when they pair (addition, accumulation).
    fn plus(&self, other: &Value, k: f32) -> Value {
        if !self.pairs(other) {
            return self.clone();
        }
        let (a, b) = (self.nums().unwrap(), other.nums().unwrap());
        let n: Vec<f32> = a.iter().zip(b).map(|(a, b)| a + b * k).collect();
        self.with(n)
    }

    fn with(&self, n: Vec<f32>) -> Value {
        match self {
            Value::Nums { text, .. } => Value::Nums {
                nums: n,
                text: text.clone(),
            },
            Value::Color(_) => Value::Color([n[0], n[1], n[2], n[3]]),
            t => t.clone(),
        }
    }

    /// The distance between two values (for paced timing).
    fn distance(&self, other: &Value) -> f32 {
        if !self.pairs(other) {
            return 0.0;
        }
        let (a, b) = (self.nums().unwrap(), other.nums().unwrap());
        a.iter()
            .zip(b)
            .map(|(a, b)| (b - a) * (b - a))
            .sum::<f32>()
            .sqrt()
    }

    /// The value as attribute text.
    pub fn text(&self) -> String {
        match self {
            Value::Nums { nums, text } => {
                let mut s = text[0].clone();
                for (n, t) in nums.iter().zip(&text[1..]) {
                    s.push_str(&format!("{n}"));
                    s.push_str(t);
                }
                s
            }
            Value::Color(c) => {
                let b = |v: f32| (v.clamp(0.0, 1.0) * 255.0).round() as u8;
                format!(
                    "rgba({}, {}, {}, {})",
                    b(c[0]),
                    b(c[1]),
                    b(c[2]),
                    c[3].clamp(0.0, 1.0)
                )
            }
            Value::Text(t) => t.clone(),
        }
    }
}

/// `<animateTransform type>`'s parameters, padded with the type's
/// defaults (`scale(s)` is `scale(s s)`).
fn transform_args(kind: &str, v: &Value) -> Value {
    let Value::Nums { nums, .. } = v else {
        return v.clone();
    };
    let mut n = nums.clone();
    let want = match kind {
        "translate" | "scale" => 2,
        "rotate" => 3,
        _ => 1,
    };
    while n.len() < want {
        n.push(if kind == "scale" { n[0] } else { 0.0 });
    }
    n.truncate(want);
    let mut text = vec![String::new()];
    text.extend((1..n.len()).map(|_| " ".to_string()));
    text.push(String::new());
    Value::Nums { nums: n, text }
}

/// The point and direction (radians) at `at` of `path`'s length.
fn along(path: &Path, at: f32) -> Option<(Vec2, f32)> {
    let lines = path.flatten(0.05);
    let total: f32 = lines
        .iter()
        .flat_map(|(p, _)| p.windows(2).map(|w| (w[1] - w[0]).length()))
        .sum();
    let mut left = at.clamp(0.0, 1.0) * total;
    let mut last = None;
    for (pts, _) in &lines {
        for w in pts.windows(2) {
            let d = w[1] - w[0];
            let len = d.length();
            if len <= 0.0 {
                continue;
            }
            last = Some((w[1], d.y.atan2(d.x)));
            if left <= len {
                return Some((w[0] + d * (left / len), d.y.atan2(d.x)));
            }
            left -= len;
        }
    }
    last.or_else(|| lines.first().map(|(p, _)| (p[0], 0.0)))
}

impl Animation {
    /// Parses animation element `tag` from its literal attributes; an
    /// `<animateMotion>`'s `<mpath>` path is `mpath`.
    pub fn parse(
        tag: &str,
        attrs: &[(String, String)],
        mpath: Option<Path>,
    ) -> Result<Animation, String> {
        for (k, _) in attrs {
            if !ATTRS.contains(&k.as_str()) {
                return Err(match k.as_str() {
                    "href" | "xlink:href" => {
                        format!("<{tag} {k}>: write the animation inside the element it animates")
                    }
                    "end" | "min" | "max" => {
                        format!("<{tag} {k}>: an interval ends by dur, repeatCount and repeatDur")
                    }
                    "keyPoints" => format!(
                        "<{tag} keyPoints>: the motion is paced along its path; time it with keyTimes on values, or with og.tween"
                    ),
                    other => format!("<{tag}> takes no attribute {other}"),
                });
            }
        }
        let get = |k: &str| attrs.iter().find(|(a, _)| a == k).map(|(_, v)| v.as_str());
        let target = match tag {
            "animateTransform" => {
                if get("attributeName").is_some_and(|a| a != "transform") {
                    return Err("<animateTransform> animates attributeName=\"transform\"".into());
                }
                Target::Transform(match get("type").unwrap_or("translate").trim() {
                    "translate" => "translate",
                    "scale" => "scale",
                    "rotate" => "rotate",
                    "skewX" => "skewX",
                    "skewY" => "skewY",
                    t => {
                        return Err(format!(
                            "<animateTransform type=\"{t}\"> is translate, scale, rotate, skewX or skewY"
                        ));
                    }
                })
            }
            "animateMotion" => Target::Motion {
                path: match (mpath, get("path")) {
                    (Some(p), _) => Some(p),
                    (None, Some(d)) => {
                        Some(Path::parse(d).map_err(|e| format!("<animateMotion path>: {e}"))?)
                    }
                    (None, None) => None,
                },
                rotate: match get("rotate").map(str::trim) {
                    None => Rotate::Angle(0.0),
                    Some("auto") => Rotate::Auto,
                    Some("auto-reverse") => Rotate::AutoReverse,
                    Some(a) => Rotate::Angle(a.parse().map_err(|_| {
                        format!("<animateMotion rotate=\"{a}\"> is auto, auto-reverse or an angle")
                    })?),
                },
            },
            _ => {
                let name = get("attributeName").ok_or_else(|| {
                    format!("<{tag}> names the attribute it animates: attributeName")
                })?;
                if name == "transform" {
                    return Err(format!(
                        "<{tag} attributeName=\"transform\">: animate a transform with <animateTransform>"
                    ));
                }
                Target::Attr(name.trim().to_string())
            }
        };
        let attr = match &target {
            Target::Attr(a) => a.as_str(),
            _ => "transform",
        };
        let value = |v: &str| {
            let v = Value::parse(attr, v);
            match &target {
                Target::Transform(kind) => transform_args(kind, &v),
                _ => v,
            }
        };
        let mut items: Vec<Item> = match (get("values"), get("from"), get("to"), get("by")) {
            (Some(vs), ..) => vs
                .split(';')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|v| Item::Own(value(v)))
                .collect(),
            (None, from, Some(to), _) => vec![
                from.map_or(Item::Base, |f| Item::Own(value(f))),
                Item::Own(value(to)),
            ],
            (None, Some(from), None, Some(by)) => {
                let f = value(from);
                let b = value(by);
                vec![Item::Own(f.clone()), Item::Own(f.plus(&b, 1.0))]
            }
            (None, None, None, Some(by)) => vec![Item::Base, Item::BasePlus(value(by))],
            _ => Vec::new(),
        };
        if tag == "set" {
            items = match get("to") {
                Some(to) => vec![Item::Own(value(to))],
                None => return Err("<set> takes to=\"…\"".into()),
            };
        }
        let motion = matches!(target, Target::Motion { path: Some(_), .. });
        if items.is_empty() && !motion {
            return Err(format!("<{tag}> takes values, or from/to/by"));
        }
        let mut begins = Vec::new();
        for b in get("begin")
            .unwrap_or("0s")
            .split(';')
            .filter(|b| !b.trim().is_empty())
        {
            begins.extend(begin(b)?);
        }
        let dur = match get("dur").map(str::trim) {
            None | Some("indefinite") | Some("media") => None,
            Some(d) => {
                let d = clock(d).map_err(|e| format!("dur: {e}"))?;
                if d <= 0.0 {
                    return Err("dur is above 0".into());
                }
                Some(d)
            }
        };
        let mut repeat_forever = false;
        let repeat_count =
            match get("repeatCount").map(str::trim) {
                None => None,
                Some("indefinite") => {
                    repeat_forever = true;
                    None
                }
                Some(n) => Some(n.parse::<f32>().ok().filter(|n| *n > 0.0).ok_or_else(|| {
                    format!("repeatCount=\"{n}\" is a count above 0 or indefinite")
                })?),
            };
        let repeat_dur = match get("repeatDur").map(str::trim) {
            None => None,
            Some("indefinite") => {
                repeat_forever = true;
                None
            }
            Some(d) => Some(clock(d).map_err(|e| format!("repeatDur: {e}"))?),
        };
        let freeze = match get("fill").map(str::trim) {
            None | Some("remove") => false,
            Some("freeze") => true,
            Some(f) => return Err(format!("fill=\"{f}\" on an animation is freeze or remove")),
        };
        if let Some(r) = get("restart").map(str::trim)
            && r != "always"
        {
            return Err(format!(
                "restart=\"{r}\": an animation restarts at each of its begins (always)"
            ));
        }
        let calc = match get("calcMode").map(str::trim) {
            None if tag == "set" => Calc::Discrete,
            None if tag == "animateMotion" => Calc::Paced,
            None => Calc::Linear,
            Some("discrete") => Calc::Discrete,
            Some("linear") => Calc::Linear,
            Some("paced") => Calc::Paced,
            Some("spline") => Calc::Spline,
            Some(c) => {
                return Err(format!(
                    "calcMode=\"{c}\" is discrete, linear, paced or spline"
                ));
            }
        };
        let key_times = match get("keyTimes") {
            None => None,
            Some(k) => {
                let t: Vec<f32> = k
                    .split(';')
                    .map(str::trim)
                    .filter(|v| !v.is_empty())
                    .map(|v| {
                        v.parse::<f32>()
                            .map_err(|_| format!("keyTimes: \"{v}\" is not a number"))
                    })
                    .collect::<Result<_, _>>()?;
                if t.len() != items.len() {
                    return Err(format!(
                        "keyTimes has {} times for {} values",
                        t.len(),
                        items.len()
                    ));
                }
                if t.windows(2).any(|w| w[1] < w[0]) || t.first().is_some_and(|f| *f != 0.0) {
                    return Err("keyTimes rise from 0".into());
                }
                if calc != Calc::Discrete && t.last().is_some_and(|l| *l != 1.0) {
                    return Err("keyTimes end at 1 (except with calcMode=\"discrete\")".into());
                }
                Some(t)
            }
        };
        let key_splines = match get("keySplines") {
            None => Vec::new(),
            Some(k) => k
                .split(';')
                .map(str::trim)
                .filter(|v| !v.is_empty())
                .map(|s| {
                    let n = numbers(s)?;
                    match n[..] {
                        [a, b, c, d] if [a, b, c, d].iter().all(|v| (0.0..=1.0).contains(v)) => {
                            Ok([a, b, c, d])
                        }
                        _ => Err(format!("keySplines: \"{s}\" is four numbers from 0 to 1")),
                    }
                })
                .collect::<Result<_, _>>()?,
        };
        if calc == Calc::Spline && key_splines.len() + 1 != items.len().max(2) {
            return Err(format!(
                "calcMode=\"spline\" takes one keySplines entry per interval: {} for {} values",
                items.len().max(2) - 1,
                items.len()
            ));
        }
        let additive = match get("additive").map(str::trim) {
            None | Some("replace") => false,
            Some("sum") => true,
            Some(a) => return Err(format!("additive=\"{a}\" is replace or sum")),
        };
        let accumulate = match get("accumulate").map(str::trim) {
            None | Some("none") => false,
            Some("sum") => true,
            Some(a) => return Err(format!("accumulate=\"{a}\" is none or sum")),
        };
        Ok(Animation {
            target,
            begins,
            dur,
            repeat_count,
            repeat_dur,
            repeat_forever,
            freeze,
            items,
            key_times,
            key_splines,
            calc,
            additive,
            accumulate,
        })
    }

    /// The active duration from a begin; `None` is indefinite.
    pub fn active(&self) -> Option<f32> {
        if self.repeat_forever {
            return None;
        }
        let by_count = match (self.repeat_count, self.dur) {
            (Some(n), Some(d)) => Some(n * d),
            (Some(_), None) => None,
            (None, d) if self.repeat_dur.is_none() => d,
            (None, _) => None,
        };
        match (by_count, self.repeat_dur) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        }
    }

    /// The begin of the interval at time `t`: the latest begin at or
    /// before it, from `begins` resolved to seconds.
    pub fn current(resolved: &[f32], t: f32) -> Option<f32> {
        resolved
            .iter()
            .copied()
            .filter(|b| *b <= t)
            .fold(None, |m: Option<f32>, b| Some(m.map_or(b, |m| m.max(b))))
    }

    /// Where the animation is at `local` seconds into its interval: its
    /// iteration and progress, or `None` when it has no effect (ended
    /// without `fill="freeze"`). Also whether it is still running.
    pub fn progress(&self, local: f32) -> (Option<(f32, f32)>, bool) {
        let active = self.active();
        let Some(d) = self.dur else {
            // An indefinite simple duration holds its first value.
            return (Some((0.0, 0.0)), false);
        };
        match active {
            Some(a) if local >= a => {
                if !self.freeze {
                    return (None, false);
                }
                let n = a / d;
                let (iter, p) = if n.fract() == 0.0 && n > 0.0 {
                    (n - 1.0, 1.0)
                } else {
                    (n.floor(), n.fract())
                };
                (Some((iter, p)), false)
            }
            _ => {
                let iter = (local / d).floor();
                (Some((iter, (local - iter * d) / d)), true)
            }
        }
    }

    fn values(&self, base: &Value) -> Vec<Value> {
        self.items
            .iter()
            .map(|i| match i {
                Item::Base => base.clone(),
                Item::Own(v) => v.clone(),
                Item::BasePlus(v) => base.plus(v, 1.0),
            })
            .collect()
    }

    /// The value at progress `p` of `values`.
    fn at(&self, values: &[Value], p: f32) -> Value {
        let n = values.len();
        if n == 1 {
            return values[0].clone();
        }
        if self.calc == Calc::Discrete {
            let times = self
                .key_times
                .clone()
                .unwrap_or_else(|| (0..n).map(|i| i as f32 / n as f32).collect());
            let i = times.iter().rposition(|t| *t <= p).unwrap_or(0);
            return values[i].clone();
        }
        let times = if self.calc == Calc::Paced {
            let d: Vec<f32> = values.windows(2).map(|w| w[0].distance(&w[1])).collect();
            let total: f32 = d.iter().sum();
            let mut acc = 0.0;
            let mut t = vec![0.0];
            for (i, x) in d.iter().enumerate() {
                acc += x;
                t.push(if total > 0.0 {
                    acc / total
                } else {
                    (i + 1) as f32 / (n - 1) as f32
                });
            }
            t
        } else {
            self.key_times
                .clone()
                .unwrap_or_else(|| (0..n).map(|i| i as f32 / (n - 1) as f32).collect())
        };
        if p >= 1.0 {
            return values[n - 1].clone();
        }
        let i = times[..n - 1].iter().rposition(|t| *t <= p).unwrap_or(0);
        let span = times[i + 1] - times[i];
        let mut s = if span > 0.0 {
            (p - times[i]) / span
        } else {
            1.0
        };
        if self.calc == Calc::Spline {
            s = crate::motion::cubic_bezier(self.key_splines[i], s);
        }
        values[i].lerp(&values[i + 1], s)
    }

    /// Samples an attribute or transform animation over `base` (the
    /// underlying value as text) at `(iteration, progress)`; the result
    /// replaces `base`.
    pub fn sample(&self, base: &str, at: (f32, f32)) -> String {
        let (iter, p) = at;
        match &self.target {
            Target::Attr(name) => {
                let base_v = Value::parse(name, base);
                let values = self.values(&base_v);
                let mut v = self.at(&values, p);
                if self.accumulate && iter > 0.0 {
                    v = v.plus(values.last().unwrap(), iter);
                }
                if self.additive && !matches!(self.items.first(), Some(Item::Base)) {
                    v = base_v.plus(&v, 1.0);
                }
                v.text()
            }
            Target::Transform(kind) => {
                let zero = transform_args(
                    kind,
                    &Value::parse("transform", if *kind == "scale" { "1" } else { "0" }),
                );
                let values = self.values(&zero);
                let mut v = self.at(&values, p);
                if self.accumulate && iter > 0.0 {
                    v = v.plus(values.last().unwrap(), iter);
                }
                let own = format!("{kind}({})", v.text());
                if self.additive && !base.trim().is_empty() {
                    format!("{base} {own}")
                } else {
                    own
                }
            }
            Target::Motion { path, rotate } => {
                let (at, angle) = match path {
                    // Paced along the path's length.
                    Some(path) => along(path, p).unwrap_or((Vec2::ZERO, 0.0)),
                    None => {
                        let values = self.values(&Value::parse("motion", "0 0"));
                        let here = self.at(&values, p);
                        let ahead = self.at(&values, (p + 1e-3).min(1.0));
                        let behind = self.at(&values, (p - 1e-3).max(0.0));
                        let xy = |v: &Value| {
                            let n = v.nums().unwrap_or(&[]);
                            Vec2::new(
                                n.first().copied().unwrap_or(0.0),
                                n.get(1).copied().unwrap_or(0.0),
                            )
                        };
                        let d = xy(&ahead) - xy(&behind);
                        (xy(&here), d.y.atan2(d.x))
                    }
                };
                let deg = match rotate {
                    Rotate::Auto => angle.to_degrees(),
                    Rotate::AutoReverse => angle.to_degrees() + 180.0,
                    Rotate::Angle(a) => *a,
                };
                let own = format!("translate({} {}) rotate({deg})", at.x, at.y);
                if base.trim().is_empty() {
                    own
                } else {
                    format!("{own} {base}")
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn anim(tag: &str, attrs: &[(&str, &str)]) -> Animation {
        let attrs: Vec<_> = attrs
            .iter()
            .map(|(a, b)| (a.to_string(), b.to_string()))
            .collect();
        Animation::parse(tag, &attrs, None).unwrap()
    }

    #[test]
    fn smil_timing_values_and_splines() {
        // keyTimes put the middle value early; a spline eases the first
        // interval; a repeat count of 1.5 freezes halfway.
        let a = anim(
            "animate",
            &[
                ("attributeName", "r"),
                ("values", "0; 10; 20"),
                ("keyTimes", "0; 0.25; 1"),
                ("dur", "2s"),
                ("repeatCount", "1.5"),
                ("fill", "freeze"),
            ],
        );
        assert_eq!(a.active(), Some(3.0));
        let at = |t: f32| a.sample("5", a.progress(t).0.unwrap());
        assert_eq!(at(0.25), "5"); // p 0.125: halfway to 10
        assert_eq!(at(1.25), "15"); // p 0.625: halfway from 10 to 20
        // Frozen at p 0.5 of the second iteration: a third of 10 → 20.
        let frozen: f32 = at(3.5).parse().unwrap();
        assert!((frozen - 13.333).abs() < 0.01, "{frozen}");
        let s = anim(
            "animate",
            &[
                ("attributeName", "x"),
                ("from", "0"),
                ("to", "100"),
                ("dur", "1s"),
                ("calcMode", "spline"),
                ("keySplines", "0.42 0 1 1"),
            ],
        );
        let v: f32 = s.sample("0", s.progress(0.5).0.unwrap()).parse().unwrap();
        assert!(v < 40.0, "ease-in is behind linear: {v}");
        assert_eq!(s.progress(1.5).0, None, "removed after its interval");
        // by-animation adds to the underlying value; colors interpolate.
        let by = anim(
            "animate",
            &[("attributeName", "cx"), ("by", "10"), ("dur", "1s")],
        );
        assert_eq!(by.sample("40", (0.0, 0.5)), "45");
        let c = anim(
            "animate",
            &[
                ("attributeName", "fill"),
                ("values", "#000;#fff"),
                ("dur", "1s"),
            ],
        );
        assert_eq!(c.sample("red", (0.0, 0.5)), "rgba(128, 128, 128, 1)");
        // Discrete text, and numbers in a path template.
        let d = anim(
            "animate",
            &[
                ("attributeName", "d"),
                ("values", "M0 0 L10 0;M0 0 L10 20"),
                ("dur", "1s"),
            ],
        );
        assert_eq!(d.sample("", (0.0, 0.5)), "M0 0 L10 10");
    }

    #[test]
    fn smil_transforms_motion_and_begins() {
        let r = anim(
            "animateTransform",
            &[
                ("attributeName", "transform"),
                ("type", "rotate"),
                ("from", "0 20 20"),
                ("to", "360 20 20"),
                ("dur", "2s"),
                ("repeatCount", "indefinite"),
                ("additive", "sum"),
            ],
        );
        assert_eq!(r.active(), None);
        assert_eq!(
            r.sample("translate(5 0)", r.progress(2.5).0.unwrap()),
            "translate(5 0) rotate(90 20 20)"
        );
        let m = anim(
            "animateMotion",
            &[("path", "M0 0 H10 V10"), ("dur", "2s"), ("rotate", "auto")],
        );
        // Paced: halfway is the corner, the path turning from +x to +y.
        let at = m.sample("", (0.0, 0.75));
        let n = numbers(
            at.replace(['(', ')'], " ")
                .replace("translate", "")
                .replace("rotate", "")
                .as_str(),
        )
        .unwrap();
        assert!(
            (n[0] - 10.0).abs() < 1e-4 && (n[1] - 5.0).abs() < 1e-4 && (n[2] - 90.0).abs() < 1e-3,
            "{at}"
        );
        assert_eq!(
            begin("a.end-0.5s").unwrap(),
            Some(Begin::Sync {
                id: "a".into(),
                end: true,
                offset: -0.5
            })
        );
        assert_eq!(clock("01:02.5").unwrap(), 62.5);
        assert!(begin("click").is_err());
        assert_eq!(Animation::current(&[0.0, 2.0, 5.0], 3.0), Some(2.0));
    }
}
