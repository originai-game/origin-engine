//! The model API as JSON operations: what a binding (TypeScript over
//! wasm, the CLI) sends, one operation at a time.
//!
//! Entry: [`Session`]. [`Session::apply`] takes one operation, an object
//! `{"op": "<name>", …fields}` with camelCase fields, and answers a JSON
//! object. Operations that change the model are recorded;
//! [`Session::program`] gives them back as a program,
//! `{"name": …, "ops": [...]}`, and [`run`] replays one. Readers (frames,
//! boxes, faces, measures) are not recorded: the values they gave are
//! written into the operations that used them, so a replay needs no
//! readers and gives the same bytes.
//!
//! A program may declare parameters, `"params": {"height": 900}`, and
//! give any field of its operations as `"$height"` or an expression over
//! them (`"$height / 2 - 15"`: numbers, `+ - * /` and parentheses): [`with_params`] sets
//! them (a variant's values over the program's own), and every replay
//! takes the program's own.
//!
//! Every operation, and every object inside one (a profile, a section, a
//! view, a frame), takes only the fields listed; any other field is an
//! error naming it and the fields it takes. A failing operation answers
//! [`ModelError::At`]: its index in the program, its verb, and the part it
//! made or worked on.
//!
//! Operations, with their fields (lengths in the model's unit, angles in
//! radians, frames as `{"origin": [x, y, z], "x": […], "y": […], "z": […]}`,
//! parts by the `id` a constructor answered, turned solids' `axis` `"+Y"`
//! (the default) or `"+Z"`, `segments` a turn with 0 for the chord rule):
//!
//! | op | fields | answers |
//! |---|---|---|
//! | `units` | `unit`: `"mm"`, `"cm"`, `"m"` | |
//! | `uvs` | | |
//! | `box` | `name`, `width`, `depth`, `height`, `on`? | `id` |
//! | `cylinder` | `name`, `radius`, `radiusTop`?, `height`, `segments`?, `on`? | `id` |
//! | `cone` | `name`, `radius`, `height`, `segments`?, `on`? | `id` |
//! | `sphere` | `name`, `radius`, `axis`?, `segments`?, `on`? | `id` |
//! | `dome` | `name`, `radius`, `height`?, `axis`?, `segments`?, `on`? | `id` |
//! | `tube` | `name`, `outer`, `inner`, `height`, `segments`?, `on`? | `id` |
//! | `torus` | `name`, `radius`, `tube`, `angle`?, `axis`?, `segments`?, `tubeSegments`?, `on`? | `id` |
//! | `wedge` | `name`, `width`, `height`, `depth`, `on`? | `id` |
//! | `extrude` | `name`, `profile`, `height` or `until` (`{part, face}`), `twist`?, `taper`?, `bothSides`?, `on`? | `id` |
//! | `revolve` | `name`, `profile` (x the radius), `angle`?, `axis`?, `segments`?, `on`? | `id` |
//! | `sweep` | `name`, `profile`, `path` (a path), `up`?, `twist`?, `scaleEnd`?, `on`? | `id` |
//! | `loft` | `name`, `sections`: `[{profile, z}]`, `samples`?, `steps`?, `curve`? (`"auto"`, `"straight"`, `"catmullRom"`, `"spline"`), `on`? | `id` |
//! | `thread` | `name`, `radius`, `pitch`, `length`, `depth`, `on`? | `id` |
//! | `wire` | `name`, `path` (a path), `diameter`, `closed`?, `fillet`?, `fillets`? (points-only paths), `segments`?, `on`? | `id` |
//! | `body` | `name`, `plan`, `top`, `bottom`, `n`? (views: a number, `[[y, v]…]`, or `{points, smooth}`), `on`? | `id` |
//! | `knurl` | `name`, `diameter`, `height`, `pitch`?, `straight`?, `on`? | `id` |
//! | `gear` | `name`, `teeth`, `module`, `width`, `pressureAngle`?, `bore`?, `hub`?, `hubHeight`?, `on`? | `id` |
//! | `at`, `transform` | `id`, `frame` | |
//! | `group` | `name`, `children` | `id` |
//! | `wrap` | `name`, `parts` (ids) | `id`: the parts' convex hull, a new part; faces and edges `hull` |
//! | `assembly` | `name`, `children`, `anchors`? (`{name: frame}`) | `id` |
//! | `attach` | `to`, `name`, `children` | `id` |
//! | `copy` | `id`, `name` | `id` |
//! | `mirror` | `id`, `plane` (a frame), `name` | `id` |
//! | `cut`, `join`, `intersect` | `target`, `tool`, `check`? | `voids`, `touchingEdges`, `touching`, `facesWithoutUvs`, `crossingPairs` |
//! | `merge` | `target`, `tool` | |
//! | `delete` | `id` | |
//! | `seat` | `host`, `part` | `crossingPairs` |
//! | `scale` | `id`, `x`?, `y`?, `z`? (factors along its own axes; 0 keeps 1) | |
//! | `hole` | `id`, `diameter`, `at` (frames), `depth`?, `counterbore`? `[d, depth]`, `countersink`? `[d, angle]` | `crossingPairs` |
//! | `pocket` | `id`, `profile`, `depth`?, `at`? (frames) | `crossingPairs` |
//! | `split` | `id`, `frame`, `keep`? (`"above"`, `"below"`) | |
//! | `shell` | `id`, `thickness`, `open`? | `crossingPairs` |
//! | `dent` | `id`, `at` (points), `radius`, `depth` | `crossingPairs` |
//! | `subdivide` | `id`, `levels`? (1), `sharp`? (edge names kept sharp) | |
//! | `fillet` | `id`, `radius`, `edges`?, `segments`?, `profile`? | |
//! | `chamfer` | `id`, `distance`, `distance2`?, `angle`?, `edges`?, `segments`? | |
//! | `soften` | `id`, `radius`, `angle`?, `edges`?, `segments`? | |
//! | `material` | `ids`, `name`?, `color`?, `metalness`?, `roughness`?, `emissive`?, `opacity`?, `map`?, `mapSize`? | |
//! | `joint` | `child`, `parent`, `name`, `kind`?, `on`, `axis`?, `range`?, `mimic`? (`{joint, multiplier}`) | |
//! | `pose` | `joint`, `value`; or `clip`, `time` (the pose a clip has then) | |
//! | `clip` | `name`, `keys`: `[{time, pose: {joint: value}}]` (seconds, rising) | |
//!
//! Readers, not recorded: `frame` (`origin`, `normal`, `up`?: a frame with
//! [`Frame::facing`]'s axes, or with Y toward `up`), `offset` (`frame`,
//! `distance`), `translate` (`frame`, `x`, `y`, `z`), `turn` (`frame`,
//! `rx`, `ry`, `rz`), `compose` (`frame`, `local`), `bbox` (`ids`,
//! `sides`), `face` (`id`, `name`), `edge` (`id`, `name`: a `path`),
//! `local` (`id`, `frame`), `anchor` (`id`, `name`), `placement` (`id`),
//! `grid` (`nx`, `ny`?, `px`?, `py`?, `stagger`?, `on`?), `ring` (`n`,
//! `radius`, `start`?, `angle`?, `on`?), `along` (`path`, `pitch`?,
//! `count`?), `snap` (`ids`, `points`, `direction`?), `names` (`id`),
//! `measure`. Profiles are objects with one key: `rect`, `circle`,
//! `ellipse`, `slot`, `polygon` (`sides`, `radius` or `flats`), `poly`
//! (`points`: `[x, y]` or `{x, y, radius, bulge}`), `gear`, `union` /
//! `subtract` / `intersect` (a list, the first operated on by the rest),
//! `offset` (`profile`, `distance`), `translate` (`profile`, `x`, `y`),
//! `rotate` (`profile`, `angle`), `scale` (`profile`, `x`, `y`; 0 keeps 1),
//! `superellipse` (`rx`, `ry`, `exponent`, `segments`), `spline`
//! (`points`, closed), `path` (a 2D path, closed), `stroke` (`path`,
//! `width`, `join`, `roundCaps`, `closed`), `hull` (a list), `mirror`
//! (`profile`, `axis`: `"y"` or `"x"`), `text` (`text`, `font`, `size`,
//! `spacing`).
//!
//! A path is a list: a start point, then pieces from where the last
//! ended: a point (a line), `{"arc": {"through", "to"}}`,
//! `{"tangentArc": {"to"}}`, `{"spline": {"points"}}`, flattened by the
//! curve rule (`crate::profile::curve`, [`crate::generators::path3`]). In 3D (`sweep`,
//! `wire`) it may instead be `{"helix": {"radius", "pitch", "turns"}}`.
//!
//! Every generator also takes `uvs`? (a boolean; the `uvs` operation's
//! choice by default): the part's solid gets UVs in the model's unit
//! (`crate::generators::solids::textured`). `uvs` asks for them on every part
//! made after it, and is refused once a part without them exists. A
//! material's `map` names an image the caller supplies
//! ([`Session::texture`], [`textures_named`]); `mapSize` is the units it
//! spans. A part with UVs adds `uv` to the measures.
//!
//! A `text` profile names a font; the caller supplies its bytes under that
//! name ([`Session::font`], [`run_with`]); [`fonts_named`] lists the names
//! a program uses. No font ships with the kernel.

use std::collections::BTreeMap;

use crate::bevel::{Chamfer, Fillet, Soften};
use crate::boolean::BoolOp;
use crate::edit::{Dent, Hole, Keep, Pocket, Shell, Subdivide};
use crate::generators::body::{Body, View};
use crate::generators::loft::LoftCurve;
use crate::generators::parts::{GearDesc, Knurl};
use crate::generators::path3::{Flat, Path3, Piece};
use crate::generators::paths::{Helix, Loft, LoftAlong, Sweep, Thread};
use crate::generators::solids::{
    Axis, BoxDesc, ConeDesc, CylinderDesc, DomeDesc, Extrude, Revolve, SphereDesc, TorusDesc,
    TubeDesc,
};
use crate::generators::wire::Wire;
use crate::math::{DVec2, DVec3, Frame};
use crate::profile::Profile;
use crate::tools::{JointKind, Material, Mimic};
use serde_json::{Map, Value, json};

use crate::model::error::ModelError;
use crate::model::place::{self, Path};
use crate::model::tree::{Combine, Id, JointDesc, Model, Unit};

mod args;
mod generate;
mod outlines;
mod report;

mod ops;
pub(crate) use args::*;
pub(crate) use generate::*;
pub(crate) use ops::*;
pub(crate) use outlines::*;
pub use report::*;

/// A model and the program that built it.
#[derive(Clone, Debug, Default)]
pub struct Session {
    pub(crate) model: Model,
    pub(crate) ops: Vec<Value>,
    pub(crate) fonts: Fonts,
    pub(crate) textures: Fonts,
}

/// Fonts a caller supplies, by the name `text` profiles give them: a
/// font file's bytes (TrueType or OpenType).
pub type Fonts = BTreeMap<String, Vec<u8>>;

/// A program's parameters: `"params": {"height": 900, …}` beside its
/// `ops`, each a number an operation's field takes as `"$height"`.
pub type Params = BTreeMap<String, f64>;

/// The program with its parameters set: every string in its operations
/// that names a parameter (`"$name"`, or an expression such as
/// `"($width - 40) / 2"`, [`eval`]) becomes the number it comes to with `overrides` gives `name`, else the
/// program's own `params` value, and `params` is dropped. A `"$name"` the
/// program does not declare, an override it does not declare, or a
/// parameter that is not a finite number is an error. A program without
/// `params` and without overrides comes back as it was.
pub fn with_params(program: &str, overrides: &Params) -> Result<String, ModelError> {
    let mut v: Value =
        serde_json::from_str(program).map_err(|e| ModelError::Program(format!("not JSON: {e}")))?;
    let declared = match v.as_object_mut().and_then(|o| o.remove("params")) {
        None => Params::new(),
        Some(Value::Object(o)) => o
            .into_iter()
            .map(|(k, x)| match x.as_f64().filter(|n| n.is_finite()) {
                Some(n) => Ok((k, n)),
                None => Err(ModelError::Program(format!(
                    "params: {k:?} is not a finite number"
                ))),
            })
            .collect::<Result<_, _>>()?,
        Some(_) => {
            return Err(ModelError::Program(
                "params is an object of numbers: {\"height\": 900}".into(),
            ));
        }
    };
    if declared.is_empty() && overrides.is_empty() {
        return Ok(program.to_string());
    }
    let mut values = declared.clone();
    for (k, x) in overrides {
        if !declared.contains_key(k) {
            return Err(ModelError::Program(format!(
                "params: the program declares no {k:?} (it has {:?})",
                declared.keys().collect::<Vec<_>>()
            )));
        }
        if !x.is_finite() {
            return Err(ModelError::Program(format!("params: {k:?} is not finite")));
        }
        values.insert(k.clone(), *x);
    }
    fn walk(v: &mut Value, values: &Params) -> Result<(), ModelError> {
        match v {
            Value::String(s) if s.contains('$') => {
                let x =
                    eval(s, values).map_err(|why| ModelError::Program(format!("{s:?}: {why}")))?;
                *v = json!(x);
            }
            Value::Array(l) => l.iter_mut().try_for_each(|x| walk(x, values))?,
            Value::Object(o) => o.values_mut().try_for_each(|x| walk(x, values))?,
            _ => {}
        }
        Ok(())
    }
    if let Some(ops) = v.get_mut("ops") {
        walk(ops, &values)?;
    }
    Ok(v.to_string())
}

/// A parameter expression: numbers, `$name`, `+ - * /`, unary minus and
/// parentheses, left to right with `*` and `/` before `+` and `-`, in
/// f64 (so the same on every machine). A result that is not finite (a
/// division by 0) is an error.
fn eval(text: &str, values: &Params) -> Result<f64, String> {
    struct P<'a> {
        s: &'a [u8],
        i: usize,
        values: &'a Params,
    }
    impl P<'_> {
        fn skip(&mut self) {
            while self.s.get(self.i).is_some_and(u8::is_ascii_whitespace) {
                self.i += 1;
            }
        }
        fn peek(&mut self) -> Option<u8> {
            self.skip();
            self.s.get(self.i).copied()
        }
        fn sum(&mut self) -> Result<f64, String> {
            let mut x = self.product()?;
            while let Some(c @ (b'+' | b'-')) = self.peek() {
                self.i += 1;
                let y = self.product()?;
                x = if c == b'+' { x + y } else { x - y };
            }
            Ok(x)
        }
        fn product(&mut self) -> Result<f64, String> {
            let mut x = self.atom()?;
            while let Some(c @ (b'*' | b'/')) = self.peek() {
                self.i += 1;
                let y = self.atom()?;
                x = if c == b'*' { x * y } else { x / y };
            }
            Ok(x)
        }
        fn atom(&mut self) -> Result<f64, String> {
            match self.peek() {
                Some(b'-') => {
                    self.i += 1;
                    Ok(-self.atom()?)
                }
                Some(b'(') => {
                    self.i += 1;
                    let x = self.sum()?;
                    if self.peek() != Some(b')') {
                        return Err(format!("a ')' is missing at {}", self.i));
                    }
                    self.i += 1;
                    Ok(x)
                }
                Some(b'$') => {
                    self.i += 1;
                    let start = self.i;
                    while self
                        .s
                        .get(self.i)
                        .is_some_and(|c| c.is_ascii_alphanumeric() || *c == b'_')
                    {
                        self.i += 1;
                    }
                    let name = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
                    self.values
                        .get(name)
                        .copied()
                        .ok_or_else(|| format!("the program declares no parameter {name:?}"))
                }
                Some(c) if c.is_ascii_digit() || c == b'.' => {
                    let start = self.i;
                    while self.s.get(self.i).is_some_and(|c| {
                        c.is_ascii_digit()
                            || *c == b'.'
                            || ((*c == b'e' || *c == b'E') && self.i > start)
                            || ((*c == b'-' || *c == b'+')
                                && matches!(self.s[self.i - 1], b'e' | b'E'))
                    }) {
                        self.i += 1;
                    }
                    let n = std::str::from_utf8(&self.s[start..self.i]).unwrap_or("");
                    n.parse().map_err(|_| format!("{n:?} is not a number"))
                }
                Some(c) => Err(format!(
                    "{:?} at {} is not part of an expression (numbers, $name, + - * / and parentheses)",
                    c as char, self.i
                )),
                None => Err("the expression ends early".into()),
            }
        }
    }
    let mut p = P {
        s: text.as_bytes(),
        i: 0,
        values,
    };
    let x = p.sum()?;
    if p.peek().is_some() {
        return Err(format!("{:?} at {} is left over", &text[p.i..], p.i));
    }
    if !x.is_finite() {
        return Err(format!("comes to {x}, not a finite number"));
    }
    Ok(x)
}

/// Replays a program (`{"name": …, "ops": [...]}`) into a model.
pub fn run(program: &str) -> Result<Model, ModelError> {
    Ok(Session::replay(program)?.model)
}

/// [`run`], with the fonts its `text` profiles name ([`fonts_named`]).
pub fn run_with(program: &str, fonts: &Fonts) -> Result<Model, ModelError> {
    Ok(Session::replay_with(program, fonts)?.model)
}

/// The font names a program's `text` profiles give, sorted, each once:
/// what a caller reads (the CLI and MCP read each as a path relative to
/// the program) and passes to [`run_with`].
pub fn fonts_named(program: &str) -> Result<Vec<String>, ModelError> {
    let v: Value =
        serde_json::from_str(program).map_err(|e| ModelError::Program(format!("not JSON: {e}")))?;
    fn walk(v: &Value, out: &mut std::collections::BTreeSet<String>) {
        match v {
            Value::Object(o) => {
                if let Some(f) = o
                    .get("text")
                    .and_then(|t| t.get("font"))
                    .and_then(Value::as_str)
                {
                    out.insert(f.to_string());
                }
                o.values().for_each(|x| walk(x, out));
            }
            Value::Array(l) => l.iter().for_each(|x| walk(x, out)),
            _ => {}
        }
    }
    let mut out = std::collections::BTreeSet::new();
    walk(v.get("ops").unwrap_or(&Value::Null), &mut out);
    Ok(out.into_iter().collect())
}

/// [`run_with`], with the textures its materials' `map`s name
/// ([`textures_named`]).
pub fn run_files(program: &str, fonts: &Fonts, textures: &Fonts) -> Result<Model, ModelError> {
    Ok(Session::replay_files(program, fonts, textures)?.model)
}

/// The texture names a program's materials give (`map`), sorted, each
/// once: what a caller reads (the CLI and MCP read each as a path
/// relative to the program) and passes to [`run_files`].
pub fn textures_named(program: &str) -> Result<Vec<String>, ModelError> {
    let v: Value =
        serde_json::from_str(program).map_err(|e| ModelError::Program(format!("not JSON: {e}")))?;
    let mut out = std::collections::BTreeSet::new();
    for op in v.get("ops").and_then(Value::as_array).into_iter().flatten() {
        if op.get("op").and_then(Value::as_str) == Some("material")
            && let Some(f) = op.get("map").and_then(Value::as_str)
        {
            out.insert(f.to_string());
        }
    }
    Ok(out.into_iter().collect())
}

impl Session {
    /// An empty model named `name`.
    pub fn new(name: &str) -> Session {
        Session {
            model: Model::new(name),
            ops: Vec::new(),
            fonts: Fonts::new(),
            textures: Fonts::new(),
        }
    }

    /// Supplies a font under `name`, the name `text` profiles give it.
    /// The bytes are read when a profile uses them; the program records
    /// the name only.
    pub fn font(&mut self, name: &str, bytes: Vec<u8>) {
        self.fonts.insert(name.to_string(), bytes);
    }

    /// The fonts supplied so far.
    pub fn fonts(&self) -> &Fonts {
        &self.fonts
    }

    /// Supplies an image (PNG or JPEG) under `name`, the name a
    /// material's `map` gives it. The program records the name only.
    pub fn texture(&mut self, name: &str, bytes: Vec<u8>) {
        self.textures.insert(name.to_string(), bytes);
    }

    /// The textures supplied so far.
    pub fn textures(&self) -> &Fonts {
        &self.textures
    }

    /// [`Session::replay_with`], with textures too ([`Session::texture`]).
    pub fn replay_files(
        program: &str,
        fonts: &Fonts,
        textures: &Fonts,
    ) -> Result<Session, ModelError> {
        // Its parameters at their own values.
        let program = &with_params(program, &Params::new())?;
        let v: Value = serde_json::from_str(program)
            .map_err(|e| ModelError::Program(format!("not JSON: {e}")))?;
        let name = v.get("name").and_then(Value::as_str).unwrap_or("model");
        let ops = v
            .get("ops")
            .and_then(Value::as_array)
            .ok_or_else(|| ModelError::Program("no \"ops\" list".into()))?;
        let mut s = Session::new(name);
        s.fonts = fonts.clone();
        s.textures = textures.clone();
        for (i, op) in ops.iter().enumerate() {
            s.apply_at(op, i)?;
        }
        Ok(s)
    }

    /// Replays a program into a session that goes on recording.
    pub fn replay(program: &str) -> Result<Session, ModelError> {
        Session::replay_with(program, &Fonts::new())
    }

    /// [`Session::replay`] with fonts supplied ([`Session::font`]).
    pub fn replay_with(program: &str, fonts: &Fonts) -> Result<Session, ModelError> {
        Session::replay_files(program, fonts, &Fonts::new())
    }

    /// The model so far.
    pub fn model(&self) -> &Model {
        &self.model
    }

    /// The operations that changed the model, as a program [`run`] replays.
    pub fn program(&self) -> String {
        json!({"name": self.model.name(), "ops": self.ops}).to_string()
    }

    /// Applies one operation given as JSON text; answers JSON text.
    pub fn apply(&mut self, op: &str) -> Result<String, ModelError> {
        let v: Value =
            serde_json::from_str(op).map_err(|e| ModelError::Program(format!("not JSON: {e}")))?;
        Ok(self.apply_value(&v)?.to_string())
    }

    /// Applies one operation; answers a JSON object. An error names the
    /// operation, the index it would be recorded at, and its part
    /// ([`ModelError::At`]).
    pub fn apply_value(&mut self, op: &Value) -> Result<Value, ModelError> {
        let index = self.ops.len();
        self.apply_at(op, index)
    }

    fn apply_at(&mut self, op: &Value, index: usize) -> Result<Value, ModelError> {
        let verb = op.get("op").and_then(Value::as_str).unwrap_or("");
        let at = |error: ModelError, part: Option<String>| ModelError::At {
            index,
            op: verb.into(),
            reader: is_reader(verb),
            part,
            error: Box::new(error),
        };
        let o = op.as_object().ok_or_else(|| {
            at(
                ModelError::Program("an operation is an object".into()),
                None,
            )
        })?;
        if verb.is_empty() {
            return Err(at(
                ModelError::Program("an operation has an \"op\" string".into()),
                None,
            ));
        }
        let a = Args {
            o,
            op: verb,
            fonts: &self.fonts,
            textures: &self.textures,
        };
        let result = check_fields(o, verb).and_then(|()| dispatch(&mut self.model, &a));
        match result {
            Ok((answer, recorded)) => {
                if recorded {
                    self.ops.push(op.clone());
                }
                Ok(answer)
            }
            Err(e) => Err(at(e, self.part_of(o))),
        }
    }

    /// The part an operation made or worked on, by name.
    fn part_of(&self, o: &Map<String, Value>) -> Option<String> {
        for k in ["id", "target", "host", "child", "to", "ids"] {
            let first = match o.get(k) {
                Some(Value::Array(l)) => l.first(),
                v => v,
            };
            if let Some(name) = first.and_then(id_of).and_then(|i| self.model.label(i)) {
                return Some(name.into());
            }
        }
        o.get("name").and_then(Value::as_str).map(String::from)
    }
}

pub(crate) fn dispatch(m: &mut Model, a: &Args) -> Result<(Value, bool), ModelError> {
    {
        let none = || Value::Object(Map::new());
        let read = |v: Value| Ok((v, false));
        let id = |i: Id| json!({ "id": i.0 });
        match a.op {
            // ── readers ──
            "frame" => {
                let origin = a.vec3("origin")?;
                let normal = a.vec3("normal")?;
                let f = match a.opt("up") {
                    Some(_) => Frame::with_up(origin, normal, a.vec3("up")?),
                    None => Frame::facing(origin, normal),
                }
                .ok_or_else(|| a.bad("normal", "zero, or along up"))?;
                read(frame_json(&f))
            }
            "offset" => read(frame_json(&a.frame("frame")?.offset(a.num("distance")?))),
            "translate" => read(frame_json(&a.frame("frame")?.translate(
                a.num_or("x", 0.0)?,
                a.num_or("y", 0.0)?,
                a.num_or("z", 0.0)?,
            ))),
            "turn" => read(frame_json(&a.frame("frame")?.turn(
                a.num_or("rx", 0.0)?,
                a.num_or("ry", 0.0)?,
                a.num_or("rz", 0.0)?,
            ))),
            "compose" => read(frame_json(&a.frame("frame")?.compose(&a.frame("local")?))),
            "bbox" => read(frame_json(&m.bbox(&a.ids("ids")?, a.str("sides")?)?)),
            "face" => read(frame_json(&m.face(a.id("id")?, a.str("name")?)?)),
            "edge" => read(path_json(&m.edge(a.id("id")?, a.str("name")?)?)),
            "local" => read(frame_json(&m.local(a.id("id")?, &a.frame("frame")?)?)),
            "anchor" => read(frame_json(&m.anchor(a.id("id")?, a.str("name")?)?)),
            "placement" => read(frame_json(&m.placement(a.id("id")?)?)),
            "grid" => read(frames_json(&place::grid(
                a.u32_or("nx", 1)?,
                a.u32_or("ny", 1)?,
                a.num_or("px", 0.0)?,
                a.num_or("py", 0.0)?,
                a.bool_or("stagger", false)?,
                &a.frame_or("on")?,
            ))),
            "ring" => read(frames_json(&place::ring(
                a.u32_or("n", 0)?,
                a.num("radius")?,
                a.num_or("start", 0.0)?,
                a.num_or("angle", 0.0)?,
                &a.frame_or("on")?,
            ))),
            "along" => read(frames_json(&place::along(
                &a.path("path")?,
                a.num_or("pitch", 0.0)?,
                a.u32_or("count", 0)?,
            )?)),
            "snap" => {
                let dir = match a.opt("direction") {
                    Some(_) => Some(a.vec3("direction")?),
                    None => None,
                };
                read(frames_json(&m.snap(
                    &a.ids("ids")?,
                    &a.points("points")?,
                    dir,
                )?))
            }
            "names" => {
                let (faces, edges) = m.names(a.id("id")?)?;
                read(json!({ "faces": faces, "edges": edges }))
            }
            "measure" => match a.opt("posed") {
                None | Some(Value::Bool(false)) => read(measures_value(m)?),
                Some(Value::Bool(true)) => read(posed_value(m)?),
                Some(_) => Err(a.bad("posed", "true or false")),
            },
            // ── the model ──
            "units" => {
                let u = match a.str("unit")? {
                    "mm" => Unit::Millimetre,
                    "cm" => Unit::Centimetre,
                    "m" => Unit::Metre,
                    _ => return Err(a.bad("unit", "\"mm\", \"cm\" or \"m\"")),
                };
                m.units(u)?;
                Ok((none(), true))
            }
            "uvs" => {
                m.ask_uvs()?;
                Ok((none(), true))
            }
            "box" | "cylinder" | "cone" | "sphere" | "dome" | "tube" | "torus" | "wedge"
            | "extrude" | "revolve" | "sweep" | "loft" | "thread" | "wire" | "body" | "knurl"
            | "gear" => {
                // Everything is read and checked before the part is added,
                // so a failing operation leaves the model as it was.
                let on = match a.opt("on") {
                    Some(_) => {
                        let f = a.frame("on")?;
                        if !f.is_rigid(1e-9) {
                            return Err(a.bad("on", "a frame: finite, orthonormal, right-handed"));
                        }
                        Some(f)
                    }
                    None => None,
                };
                let uvs = match a.opt("uvs") {
                    None => m.wants_uvs(),
                    Some(_) => a.bool_or("uvs", false)?,
                };
                let solid = if a.opt("until").is_some() {
                    extrude_until(m, a, &on.unwrap_or(Frame::IDENTITY), uvs)?
                } else {
                    generate(a, uvs)?
                };
                let i = m.add(a.str("name")?, solid)?;
                if let Some(f) = on {
                    m.at(i, &f)?;
                }
                Ok((id(i), true))
            }
            "at" => {
                m.at(a.id("id")?, &a.frame("frame")?)?;
                Ok((none(), true))
            }
            "transform" => {
                m.transform(a.id("id")?, &a.frame("frame")?)?;
                Ok((none(), true))
            }
            "group" => Ok((id(m.group(a.str("name")?, &a.ids("children")?)?), true)),
            "wrap" => Ok((id(m.wrap(a.str("name")?, &a.ids("parts")?)?), true)),
            "assembly" => {
                let mut anchors = Vec::new();
                if let Some(v) = a.opt("anchors") {
                    let o = v
                        .as_object()
                        .ok_or_else(|| a.bad("anchors", "an object of frames"))?;
                    for (k, f) in o {
                        anchors.push((k.clone(), frame_of(f).map_err(|w| a.bad("anchors", &w))?));
                    }
                }
                Ok((
                    id(m.assembly(a.str("name")?, &a.ids("children")?, &anchors)?),
                    true,
                ))
            }
            "attach" => Ok((
                id(m.attach(a.id("to")?, a.str("name")?, &a.ids("children")?)?),
                true,
            )),
            "copy" => Ok((id(m.copy(a.id("id")?, a.str("name")?)?), true)),
            "mirror" => Ok((
                id(m.mirror(a.id("id")?, &a.frame("plane")?, a.str("name")?)?),
                true,
            )),
            "cut" | "join" | "intersect" => {
                let op = match a.op {
                    "cut" => BoolOp::Cut,
                    "join" => BoolOp::Join,
                    _ => BoolOp::Intersect,
                };
                let r = m.combine(
                    a.id("target")?,
                    a.id("tool")?,
                    &Combine {
                        op,
                        check: a.bool_or("check", false)?,
                    },
                )?;
                Ok((
                    json!({
                        "voids": r.voids,
                        "touchingEdges": r.touching_edges,
                        "touching": r.touching,
                        "facesWithoutUvs": r.faces_without_uvs,
                        "crossingPairs": r.crossing_pairs,
                    }),
                    true,
                ))
            }
            "merge" => {
                m.merge(a.id("target")?, a.id("tool")?)?;
                Ok((none(), true))
            }
            "delete" => {
                m.delete(a.id("id")?)?;
                Ok((none(), true))
            }
            "seat" => {
                let r = m.seat(a.id("host")?, a.id("part")?)?;
                Ok((json!({"crossingPairs": r.crossing_pairs}), true))
            }
            "scale" => {
                let f = |k: &str| -> Result<f64, ModelError> {
                    let v = a.num_or(k, 0.0)?;
                    Ok(if v == 0.0 { 1.0 } else { v })
                };
                m.scale(a.id("id")?, DVec3::new(f("x")?, f("y")?, f("z")?))?;
                Ok((none(), true))
            }
            "hole" => {
                let pair = |k: &str| -> Result<(f64, f64), ModelError> {
                    match a.opt(k) {
                        None => Ok((0.0, 0.0)),
                        Some(_) => {
                            let v = a.nums(k)?;
                            match v[..] {
                                [x, y] => Ok((x, y)),
                                _ => Err(a.bad(k, "two numbers")),
                            }
                        }
                    }
                };
                let d = Hole {
                    diameter: a.num("diameter")?,
                    at: a.frames("at")?,
                    depth: a.num_or("depth", 0.0)?,
                    counterbore: pair("counterbore")?,
                    countersink: pair("countersink")?,
                };
                let r = m.hole(a.id("id")?, &d)?;
                Ok((json!({"crossingPairs": r.crossing_pairs}), true))
            }
            "pocket" => {
                let d = Pocket {
                    profile: a.profile("profile")?,
                    depth: a.num_or("depth", 0.0)?,
                    at: match a.opt("at") {
                        Some(_) => a.frames("at")?,
                        None => Vec::new(),
                    },
                };
                let r = m.pocket(a.id("id")?, &d)?;
                Ok((json!({"crossingPairs": r.crossing_pairs}), true))
            }
            "split" => {
                let keep = match a.opt("keep").and_then(Value::as_str).unwrap_or("above") {
                    "above" => Keep::Above,
                    "below" => Keep::Below,
                    _ => return Err(a.bad("keep", "\"above\" or \"below\"")),
                };
                m.split(a.id("id")?, &a.frame("frame")?, keep)?;
                Ok((none(), true))
            }
            "shell" => {
                let d = Shell {
                    thickness: a.num("thickness")?,
                    open: a.names_or("open")?,
                };
                let r = m.shell(a.id("id")?, &d)?;
                Ok((json!({"crossingPairs": r.crossing_pairs}), true))
            }
            "dent" => {
                let d = Dent {
                    at: a.points("at")?,
                    radius: a.num("radius")?,
                    depth: a.num("depth")?,
                };
                let r = m.dent(a.id("id")?, &d)?;
                Ok((json!({"crossingPairs": r.crossing_pairs}), true))
            }
            "subdivide" => {
                let d = Subdivide {
                    levels: a.u32_or("levels", 0)?,
                    sharp: a.names_or("sharp")?,
                };
                m.subdivide(a.id("id")?, &d)?;
                Ok((none(), true))
            }
            "fillet" => {
                let d = Fillet {
                    radius: a.num("radius")?,
                    edges: a.names_or("edges")?,
                    segments: a.u32_or("segments", 0)?,
                    profile: a.num_or("profile", 0.0)?,
                    ..Default::default()
                };
                m.fillet(a.id("id")?, &d)?;
                Ok((none(), true))
            }
            "chamfer" => {
                let d = Chamfer {
                    distance: a.num("distance")?,
                    distance2: a.num_or("distance2", 0.0)?,
                    angle: a.num_or("angle", 0.0)?,
                    edges: a.names_or("edges")?,
                    segments: a.u32_or("segments", 0)?,
                    ..Default::default()
                };
                m.chamfer(a.id("id")?, &d)?;
                Ok((none(), true))
            }
            "soften" => {
                let d = Soften {
                    radius: a.num("radius")?,
                    angle: a.num_or("angle", 0.0)?,
                    edges: a.names_or("edges")?,
                    segments: a.u32_or("segments", 0)?,
                };
                m.soften(a.id("id")?, &d)?;
                Ok((none(), true))
            }
            "material" => {
                let colour = |k: &str| -> Result<Option<[f64; 3]>, ModelError> {
                    match a.opt(k) {
                        None => Ok(None),
                        Some(Value::String(s)) => {
                            hex(s).map(Some).ok_or_else(|| a.bad(k, "\"#rrggbb\""))
                        }
                        Some(_) => match a.nums(k)?[..] {
                            [r, g, b] => Ok(Some([r, g, b])),
                            _ => Err(a.bad(k, "\"#rrggbb\" or [r, g, b]")),
                        },
                    }
                };
                let opt_num = |k: &str| -> Result<Option<f64>, ModelError> {
                    match a.opt(k) {
                        None => Ok(None),
                        Some(_) => a.num(k).map(Some),
                    }
                };
                let mat = Material {
                    name: a.opt("name").and_then(Value::as_str).unwrap_or("").into(),
                    color: colour("color")?,
                    metalness: a.num_or("metalness", 0.0)?,
                    roughness: opt_num("roughness")?,
                    emissive: colour("emissive")?.unwrap_or([0.0; 3]),
                    opacity: opt_num("opacity")?,
                    map: match a.opt("map") {
                        None => None,
                        Some(_) => Some(a.texture("map")?),
                    },
                    clearcoat: a.num_or("clearcoat", 0.0)?,
                    clearcoat_roughness: a.num_or("clearcoatRoughness", 0.0)?,
                    sheen: colour("sheenColor")?.unwrap_or([0.0; 3]),
                    sheen_roughness: a.num_or("sheenRoughness", 0.0)?,
                    transmission: a.num_or("transmission", 0.0)?,
                    iridescence: a.num_or("iridescence", 0.0)?,
                    ..Default::default()
                };
                if a.opt("mapSize").is_some() && mat.map.is_none() {
                    return Err(a.bad("mapSize", "with a `map` only"));
                }
                m.material_mapped(&a.ids("ids")?, &mat, a.num_or("mapSize", 1.0)?)?;
                Ok((none(), true))
            }
            "joint" => {
                let kind = match a.opt("kind").and_then(Value::as_str).unwrap_or("revolute") {
                    "revolute" => JointKind::Revolute,
                    "continuous" => JointKind::Continuous,
                    "prismatic" => JointKind::Prismatic,
                    "fixed" => JointKind::Fixed,
                    "floating" => JointKind::Floating,
                    _ => {
                        return Err(
                            a.bad("kind", "revolute, continuous, prismatic, fixed or floating")
                        );
                    }
                };
                let range = match a.opt("range") {
                    None | Some(Value::Null) => None,
                    Some(_) => match a.nums("range")?[..] {
                        [lo, hi] => Some((lo, hi)),
                        _ => return Err(a.bad("range", "[lower, upper]")),
                    },
                };
                let mimic = match a.opt("mimic") {
                    None | Some(Value::Null) => None,
                    Some(v) => {
                        let o = v
                            .as_object()
                            .ok_or_else(|| a.bad("mimic", "{joint, multiplier}"))?;
                        only(o, "mimic", &["joint", "multiplier"])?;
                        Some(Mimic {
                            joint: o
                                .get("joint")
                                .and_then(Value::as_str)
                                .ok_or_else(|| a.bad("mimic", "{joint, multiplier}"))?
                                .into(),
                            multiplier: o.get("multiplier").and_then(Value::as_f64).unwrap_or(1.0),
                        })
                    }
                };
                let d = JointDesc {
                    name: a.str("name")?.into(),
                    kind,
                    on: a.frame("on")?,
                    axis: match a.opt("axis") {
                        Some(Value::String(s)) => {
                            place::direction(s).ok_or_else(|| a.bad("axis", "\"+X\"…"))?
                        }
                        Some(_) => a.vec3("axis")?,
                        None => DVec3::ZERO,
                    },
                    range,
                    mimic,
                };
                m.joint(a.id("child")?, a.id("parent")?, &d)?;
                Ok((none(), true))
            }
            "pose" => {
                match a.opt("clip") {
                    Some(_) => {
                        if a.opt("joint").is_some() || a.opt("value").is_some() {
                            return Err(a.bad("clip", "with time, or joint with value: not both"));
                        }
                        m.pose_at(a.str("clip")?, a.num("time")?)?;
                    }
                    None => {
                        if a.opt("time").is_some() {
                            return Err(a.bad("time", "goes with clip"));
                        }
                        m.pose(a.str("joint")?, a.num("value")?)?;
                    }
                }
                Ok((none(), true))
            }
            "clip" => {
                let keys = a
                    .get("keys")?
                    .as_array()
                    .ok_or_else(|| a.bad("keys", "a list of {time, pose}"))?;
                let mut out = Vec::with_capacity(keys.len());
                for (i, key) in keys.iter().enumerate() {
                    let what = format!("keys[{i}]");
                    let o = key
                        .as_object()
                        .ok_or_else(|| a.bad(&what, "an object {time, pose}"))?;
                    only(o, &what, &["time", "pose"])?;
                    let time = o
                        .get("time")
                        .and_then(Value::as_f64)
                        .ok_or_else(|| a.bad(&what, "a time in seconds"))?;
                    let pose = o
                        .get("pose")
                        .and_then(Value::as_object)
                        .ok_or_else(|| a.bad(&what, "a pose {joint: value}"))?;
                    let mut values = Vec::with_capacity(pose.len());
                    for (joint, v) in pose {
                        let v = v
                            .as_f64()
                            .ok_or_else(|| a.bad(&what, "joint values are numbers"))?;
                        values.push((joint.clone(), v));
                    }
                    out.push((time, values));
                }
                m.clip(a.str("name")?, &out)?;
                Ok((none(), true))
            }
            other => Err(ModelError::Program(format!(
                "no operation {other:?}; operations: {}",
                OPS.iter().map(|(o, _)| *o).collect::<Vec<_>>().join(", ")
            ))),
        }
    }
}
