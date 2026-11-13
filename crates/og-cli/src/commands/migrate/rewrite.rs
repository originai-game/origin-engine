//! Expression and statement rewriting for three.js migration.

use super::*;

impl Ctx {
    /// The three.js class or constant named at `i` (`THREE.Mesh`, or `Mesh` imported by name) and where it ends.
    fn three_name(&self, s: &str, i: usize) -> Option<(String, usize)> {
        let head = ident_at(s, i)?;
        let end = i + head.len();
        if Some(head) == self.alias.as_deref() && s.as_bytes().get(end) == Some(&b'.') {
            let name = ident_at(&s[end + 1..], 0)?;
            return Some((name.to_string(), end + 1 + name.len()));
        }
        self.named.contains(head).then(|| (head.to_string(), end))
    }

    /// `new <three class>(args)` as the whole of `s`: the class and its raw arguments.
    fn new_call(&self, s: &str) -> Option<(String, Vec<String>)> {
        let s = s.trim();
        let rest = s.strip_prefix("new")?;
        if !rest.starts_with(char::is_whitespace) {
            return None;
        }
        let at = s.len() - rest.trim_start().len();
        let (name, end) = self.three_name(s, at)?;
        let tail = s[end..].trim_start();
        if tail.is_empty() {
            return Some((name, Vec::new()));
        }
        let open = s.len() - tail.len();
        (close(s, open) == Some(s.len() - 1)).then(|| (name, split_args(&s[open + 1..s.len() - 1])))
    }

    /// Every three.js value inside `s` rewritten: constructors of values,
    /// geometries and materials, constants, MathUtils, texture loads.
    pub(super) fn expr(&mut self, s: &str, notes: &mut Vec<Note>) -> String {
        let b = s.as_bytes();
        let mut out = String::with_capacity(s.len());
        let mut i = 0;
        let mut copied = 0;
        while i < b.len() {
            if let Some(j) = skip_literal(b, i) {
                i = j;
                continue;
            }
            // `new X(...)`
            if s[i..].starts_with("new") && ident_at(s, i) == Some("new") {
                let at = i + 3 + s[i + 3..].len() - s[i + 3..].trim_start().len();
                if let Some((name, end)) = self.three_name(s, at) {
                    let tail = &s[end..];
                    let open = end + tail.len() - tail.trim_start().len();
                    let shut = if b.get(open) == Some(&b'(') {
                        close(s, open)
                    } else {
                        None
                    };
                    let args = shut.map_or_else(Vec::new, |q| split_args(&s[open + 1..q]));
                    let after = shut.map_or(end, |q| q + 1);
                    // new THREE.TextureLoader().load("…")
                    if name == "TextureLoader" && s[after..].starts_with(".load(") {
                        let lo = after + ".load".len();
                        if let Some(lq) = close(s, lo) {
                            if let Some(r) = self.texture_load(&s[lo + 1..lq], notes) {
                                out.push_str(&s[copied..i]);
                                out.push_str(&r);
                                i = lq + 1;
                                copied = i;
                            } else {
                                i = end;
                            }
                            continue;
                        }
                    }
                    if let Some(r) = self.value(&name, &args, notes) {
                        out.push_str(&s[copied..i]);
                        out.push_str(&r);
                        i = after;
                        copied = i;
                        continue;
                    }
                    if statement_class(&name) {
                        notes.push(Note { kind: Kind::Manual, text: format!("a {name} that is not a declaration of its own: declare it `const x = new {name}(…)` on its own line, or spawn it by hand"), section: S_GRAPH });
                    } else if let Some((what, section)) = manual_class(&name) {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!("{name}: {what}"),
                            section,
                        });
                    } else if !matches!(name.as_str(), "TubeGeometry" | "ExtrudeGeometry" | "Color")
                    {
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!("no og mapping for {name}"),
                            section: S_MODEL,
                        });
                    }
                    i = end;
                    continue;
                }
            }
            if let Some(word) = ident_at(s, i) {
                // loader.load("…") on a TextureLoader variable.
                if matches!(self.vars.get(word), Some(Var::TextureLoader))
                    && s[i + word.len()..].starts_with(".load(")
                {
                    let lo = i + word.len() + ".load".len();
                    if let Some(lq) = close(s, lo)
                        && let Some(r) = self.texture_load(&s[lo + 1..lq], notes)
                    {
                        out.push_str(&s[copied..i]);
                        out.push_str(&r);
                        i = lq + 1;
                        copied = i;
                        continue;
                    }
                }
                if word == "Math" && s[i..].starts_with("Math.random(") {
                    notes.push(Note { kind: Kind::Manual, text: "Math.random() is og.Rng (world.resource(og.Rng)), seeded so a replay repeats".into(), section: S_TICK });
                }
                if let Some((name, end)) = self.three_name(s, i) {
                    let replaced = match name.as_str() {
                        "FrontSide" => Some((end, "\"front\"".to_string())),
                        "BackSide" => Some((end, "\"back\"".to_string())),
                        "DoubleSide" => Some((end, "\"double\"".to_string())),
                        "MathUtils" => [
                            (".degToRad(", " * Math.PI / 180)"),
                            (".radToDeg(", " * 180 / Math.PI)"),
                        ]
                        .iter()
                        .find_map(|(m, tail)| {
                            let open = end + m.len() - 1;
                            s[end..].starts_with(m).then_some(())?;
                            let q = close(s, open)?;
                            let inner = self.expr(&s[open + 1..q], notes);
                            Some((q + 1, format!("(({inner}){tail}")))
                        }),
                        _ => None,
                    };
                    if let Some((to, text)) = replaced {
                        out.push_str(&s[copied..i]);
                        out.push_str(&text);
                        i = to;
                        copied = i;
                        continue;
                    }
                    if name != "MathUtils" {
                        let what = manual_class(&name).map_or_else(
                            || (format!("no og mapping for {name}"), S_MODEL),
                            |(w, sec)| (format!("{name}: {w}"), sec),
                        );
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: what.0,
                            section: what.1,
                        });
                    }
                    i = end;
                    continue;
                }
                i += word.len();
                continue;
            }
            i += 1;
        }
        out.push_str(&s[copied..]);
        out
    }

    fn texture_load(&mut self, args: &str, notes: &mut Vec<Note>) -> Option<String> {
        let a = split_args(args);
        match (a.len(), a.first().and_then(|x| asset_id(x))) {
            (1, Some(id)) => Some(format!("must(og.load.texture(world, {id}))")),
            _ => {
                notes.push(Note { kind: Kind::Manual, text: "a texture load with a callback or a computed URL: og.load.texture(world, \"<asset id>\") answers the handle at once".into(), section: S_LOAD });
                None
            }
        }
    }

    /// A colour argument: a literal through color.fromHex, else as written with a note.
    fn color_arg(&mut self, v: &str, notes: &mut Vec<Note>) -> String {
        let v = self.expr(v, notes);
        color_literal(&v).unwrap_or_else(|| {
            notes.push(Note { kind: Kind::Manual, text: format!("the colour `{}` is not a hex literal: give og a linear {{ r, g, b }}, or color.fromHex of its sRGB hex", short(&v)), section: S_MATERIAL });
            v
        })
    }

    /// A value constructor rewritten, or None when `name` is not one.
    fn value(&mut self, name: &str, raw: &[String], notes: &mut Vec<Note>) -> Option<String> {
        let mut args = |c: &mut Self| raw.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        Some(match name {
            "Vector2" => {
                let a = args(self);
                format!(
                    "{{ x: {}, y: {} }}",
                    a.first().map_or("0", String::as_str),
                    a.get(1).map_or("0", String::as_str)
                )
            }
            "Vector3" => vec3(&args(self)),
            "Quaternion" => {
                let a = args(self);
                let at = |i: usize, d: &'static str| a.get(i).map_or(d, String::as_str).to_string();
                format!(
                    "{{ x: {}, y: {}, z: {}, w: {} }}",
                    at(0, "0"),
                    at(1, "0"),
                    at(2, "0"),
                    at(3, "1")
                )
            }
            "Color" => match raw.len() {
                0 => "{ r: 1, g: 1, b: 1 }".into(),
                3 => {
                    let a = args(self);
                    format!("{{ r: {}, g: {}, b: {} }}", a[0], a[1], a[2])
                }
                _ => {
                    let mut n = Vec::new();
                    let v = self.color_arg(&raw[0], &mut n);
                    if !n.is_empty() {
                        notes.extend(n);
                        return None;
                    }
                    v
                }
            },
            "Shape" => format!("new og.Shape({})", args(self).join(", ")),
            "Path" => format!("new og.Path2({})", args(self).join(", ")),
            "ExtrudeGeometry" => {
                let a = args(self);
                let shapes = a.first().cloned().unwrap_or_else(|| "[]".into());
                let desc = match a.get(1).map(|o| (o, object(o))) {
                    None => format!("{{ shapes: {shapes} }}"),
                    Some((_, Some(mut pairs))) => {
                        if let Some(p) = pairs.iter().position(|(k, _)| k == "UVGenerator") {
                            pairs.remove(p);
                            notes.push(Note { kind: Kind::Manual, text: "ExtrudeGeometry's UVGenerator is not carried over; it was dropped".into(), section: S_GEOMETRY });
                        }
                        pairs.insert(0, ("shapes".into(), shapes));
                        emit_object(&pairs)
                    }
                    Some((o, None)) => format!("{{ shapes: {shapes}, ...{o} }}"),
                };
                format!("must(og.render.extrudeGeometry(world, {desc}))")
            }
            "TubeGeometry" => {
                let curve = raw.first().and_then(|c| self.new_call(c));
                let Some((cname, cargs)) =
                    curve.filter(|(n, a)| n == "CatmullRomCurve3" && !a.is_empty())
                else {
                    notes.push(Note { kind: Kind::Manual, text: "a tube along a curve other than new CatmullRomCurve3(points): write the curve as an og.path and pass it as `path`".into(), section: S_GEOMETRY });
                    return None;
                };
                debug_assert_eq!(cname, "CatmullRomCurve3");
                let mut pairs = vec![("points".to_string(), self.expr(&cargs[0], notes))];
                if let Some(closed) = cargs.get(1) {
                    pairs.push(("closed".into(), self.expr(closed, notes)));
                }
                for (k, v) in TUBE_PARAMS.iter().zip(raw.iter().skip(1)) {
                    if v != "undefined"
                        && !(*k == "closed" && pairs.iter().any(|(p, _)| p == "closed"))
                    {
                        pairs.push((k.to_string(), self.expr(v, notes)));
                    }
                }
                format!(
                    "must(og.render.tubeGeometry(world, {}))",
                    emit_object(&pairs)
                )
            }
            _ => {
                if let Some(g) = GEOMETRIES.iter().find(|g| g.three == name) {
                    let a = args(self);
                    let pairs: Vec<(String, String)> = g
                        .params
                        .iter()
                        .zip(&a)
                        .filter(|(_, v)| v.as_str() != "undefined")
                        .map(|(k, v)| (k.to_string(), v.clone()))
                        .collect();
                    if a.len() > g.params.len() {
                        let gone = &g.dropped[..(a.len() - g.params.len()).min(g.dropped.len())];
                        notes.push(Note {
                            kind: Kind::Manual,
                            text: format!(
                                "{name}'s {} not in {}; the argument was dropped",
                                gone.join(", ") + if gone.len() == 1 { " is" } else { " are" },
                                g.desc
                            ),
                            section: S_GEOMETRY,
                        });
                    }
                    return Some(format!(
                        "must(og.render.{}(world, {}))",
                        g.og,
                        emit_object(&pairs)
                    ));
                }
                let (_, extra) = MATERIALS.iter().find(|(m, _)| *m == name)?;
                return Some(self.material(name, extra, raw.first().map(String::as_str), notes));
            }
        })
    }

    fn material(
        &mut self,
        name: &str,
        extra: &str,
        params: Option<&str>,
        notes: &mut Vec<Note>,
    ) -> String {
        let mut pairs: Vec<(String, String)> = Vec::new();
        if let Some((k, v)) = extra.split_once(": ") {
            pairs.push((k.into(), v.into()));
        }
        let Some(params) = params else {
            return format!(
                "must(og.render.standardMaterial(world, {}))",
                emit_object(&pairs)
            );
        };
        let Some(entries) = object(params) else {
            notes.push(Note { kind: Kind::Manual, text: format!("{name}'s parameters are passed as a value: check its keys against StandardMaterialDesc and its colours (linear)"), section: S_MATERIAL });
            let p = self.expr(params, notes);
            pairs.insert(0, (format!("...{p}"), format!("...{p}")));
            return format!(
                "must(og.render.standardMaterial(world, {}))",
                emit_object(&pairs)
            );
        };
        for (k, v) in entries {
            if k == "vertexColors" {
                notes.push(Note { kind: Kind::Removed, text: "vertexColors: the geometry's colors multiply the colour whenever it has them".into(), section: S_MATERIAL });
                continue;
            }
            if !STANDARD_FIELDS.contains(&k.as_str()) {
                notes.push(Note {
                    kind: Kind::Manual,
                    text: format!(
                        "{name}'s `{k}` has no field in StandardMaterialDesc; it was dropped"
                    ),
                    section: S_MATERIAL,
                });
                continue;
            }
            let v = if COLOR_FIELDS.contains(&k.as_str()) {
                self.color_arg(&v, notes)
            } else {
                self.expr(&v, notes)
            };
            pairs.retain(|(p, _)| *p != k);
            pairs.push((k, v));
        }
        format!(
            "must(og.render.standardMaterial(world, {}))",
            emit_object(&pairs)
        )
    }
}

impl Ctx {
    /// The statement handlers: a three.js import, a declaration, a statement on a known variable.
    pub(super) fn statement(&mut self, t: &str, notes: &mut Vec<Note>) -> Option<Done> {
        let body = t.trim_end_matches(';').trim_end();
        if three_import(t) {
            let clause = t["import".len()..t.rfind("from").unwrap_or(t.len())].trim();
            if let Some(alias) = clause.strip_prefix("* as ") {
                self.alias = Some(alias.trim().to_string());
            } else if let (Some(a), Some(z)) = (clause.find('{'), clause.rfind('}')) {
                for n in clause[a + 1..z].split(',') {
                    let n = n.trim().rsplit(" as ").next().unwrap_or("").trim();
                    if is_ident(n) {
                        self.named.insert(n.to_string());
                    }
                }
            }
            return Some(Done::Removed(
                "three.js's import: og comes from @origingame/engine".into(),
                S_MODEL,
            ));
        }
        if let Some(d) = self.declaration(body, notes) {
            return Some(d);
        }
        if body.contains("appendChild(")
            && self.vars.iter().any(|(n, v)| {
                matches!(v, Var::Renderer) && body.contains(&format!("{n}.domElement"))
            })
        {
            return Some(Done::Removed("the engine owns the canvas".into(), S_LIGHT));
        }
        if body.ends_with(".dispose()") && is_ident(body.split('.').next().unwrap_or("")) {
            return Some(Done::Removed(
                "no dispose: the world owns geometries, materials and textures".into(),
                S_GAPS,
            ));
        }
        let head = ident_at(body, 0)?;
        let rest = body[head.len()..].strip_prefix('.')?;
        let var = self.vars.get(head)?.clone();
        Some(self.member(head, &var, rest, notes))
    }

    fn declaration(&mut self, body: &str, notes: &mut Vec<Note>) -> Option<Done> {
        let kw = ["const ", "let ", "var "]
            .into_iter()
            .find(|k| body.starts_with(k))?;
        let after = &body[kw.len()..];
        let name = ident_at(after, 0)?;
        let eq = after.find('=')?;
        let lhs = after[..eq].trim();
        if !(lhs == name || lhs.starts_with(&format!("{name}:"))) {
            return None;
        }
        let (class, raw) = self.new_call(&after[eq + 1..])?;
        let kw = kw.trim_end();
        let spawn = |n: &str| format!("{kw} {n} = must(world.spawn({{ name: \"{n}\" }}));");
        let mut args = |c: &mut Self| raw.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        let done = match class.as_str() {
            "Scene" => {
                self.vars.insert(name.into(), Var::Scene);
                Done::Removed(
                    "the world is the scene: every entity spawned is in it".into(),
                    S_GRAPH,
                )
            }
            "WebGLRenderer" | "WebGPURenderer" => {
                self.vars.insert(name.into(), Var::Renderer);
                Done::Removed(
                    "the engine draws every frame; antialiasing is GameDesc.render".into(),
                    S_LIGHT,
                )
            }
            "TextureLoader" => {
                self.vars.insert(name.into(), Var::TextureLoader);
                Done::Removed(
                    "no loader object: og.load.texture loads a cooked texture by its asset id"
                        .into(),
                    S_LOAD,
                )
            }
            "Group" | "Object3D" => {
                self.vars.insert(name.into(), Var::Object);
                Done::Lines(vec![spawn(name)], S_GRAPH)
            }
            "Mesh" => {
                let a = args(self);
                self.vars.insert(name.into(), Var::Mesh);
                let mut fields = Vec::new();
                for (k, v) in ["geometry", "material"].iter().zip(&a) {
                    fields.push((k.to_string(), v.clone()));
                }
                Done::Lines(
                    vec![
                        spawn(name),
                        format!("world.insert({name}, og.Mesh, {});", emit_object(&fields)),
                    ],
                    S_GRAPH,
                )
            }
            "PerspectiveCamera" | "OrthographicCamera" => {
                let a = args(self);
                self.vars.insert(name.into(), Var::Camera);
                self.camera = Some(name.into());
                let mut lines = vec![format!("{kw} {name} = must(world.resource(og.Camera));")];
                let set = |k: &str, v: Option<&String>, lines: &mut Vec<String>| {
                    if let Some(v) = v.filter(|v| v.as_str() != "undefined") {
                        lines.push(format!("{name}.{k} = {v};"));
                    }
                };
                if class == "PerspectiveCamera" {
                    set("fov", a.first(), &mut lines);
                    set("near", a.get(2), &mut lines);
                    set("far", a.get(3), &mut lines);
                } else {
                    lines.push(format!("{name}.orthographic = true;"));
                    if let (Some(top), Some(bottom)) = (a.get(2), a.get(3)) {
                        lines.push(format!("{name}.height = ({top}) - ({bottom});"));
                    }
                    set("near", a.get(4), &mut lines);
                    set("far", a.get(5), &mut lines);
                }
                Done::Lines(lines, S_LIGHT)
            }
            _ => {
                if let Some(light) = LIGHTS.iter().find(|l| l.three == class) {
                    self.vars.insert(name.into(), Var::Light(light));
                    let mut fields = Vec::new();
                    for (k, v) in light.params.iter().zip(&raw) {
                        if v == "undefined" {
                            continue;
                        }
                        match k {
                            Some("color") => fields.push(("color".to_string(), self.color_arg(v, notes))),
                            Some(k) => fields.push((k.to_string(), self.expr(v, notes))),
                            None => notes.push(Note { kind: Kind::Manual, text: format!("{class}'s decay is not carried over (og's falloff is fixed); it was dropped"), section: S_LIGHT }),
                        }
                    }
                    Done::Lines(
                        vec![
                            spawn(name),
                            format!(
                                "world.insert({name}, og.{}, {});",
                                light.component,
                                emit_object(&fields)
                            ),
                        ],
                        S_LIGHT,
                    )
                } else if let Some((what, section)) = manual_class(&class) {
                    self.vars.insert(name.into(), Var::Manual(what, section));
                    Done::Manual(format!("{class}: {what}"), section)
                } else {
                    return None; // a value constructor: the expression pass rewrites it
                }
            }
        };
        Some(done)
    }

    /// A statement `name.rest` on a variable the file declared.
    fn member(&mut self, name: &str, var: &Var, rest: &str, notes: &mut Vec<Note>) -> Done {
        let mut ex =
            |c: &mut Self, v: &[String]| v.iter().map(|a| c.expr(a, notes)).collect::<Vec<_>>();
        let transform = |field: &str| format!("world.set({name}, og.Transform, {{ {field} }});");
        match var {
            Var::Manual(what, section) => Done::Manual(what.to_string(), *section),
            Var::TextureLoader => Done::Manual(
                "no loader object: og.load.texture(world, \"<asset id>\")".into(),
                S_LOAD,
            ),
            Var::Scene => {
                if let Some(a) = whole_call(rest, "add") {
                    if a.iter().all(|c| {
                        matches!(
                            self.vars.get(c.as_str()),
                            Some(Var::Object | Var::Mesh | Var::Light(_) | Var::Camera)
                        )
                    }) {
                        return Done::Removed(
                            "the world holds every entity: nothing is added to a scene".into(),
                            S_GRAPH,
                        );
                    }
                    let before = notes.len();
                    for c in &a {
                        self.expr(c, notes);
                    }
                    if notes.len() == before {
                        notes.push(Note { kind: Kind::Manual, text: format!("`{}` is not an entity this file spawns: spawn it, or instantiate the loaded model", short(&a.join(", "))), section: S_GRAPH });
                    }
                    return Done::Lines(Vec::new(), 0);
                }
                if let Some(a) = whole_call(rest, "remove") {
                    let a = ex(self, &a);
                    return Done::Lines(
                        a.iter().map(|c| format!("world.despawn({c});")).collect(),
                        S_GRAPH,
                    );
                }
                if let Some(("background", v)) = assignment(rest) {
                    let c = self.color_arg(v, notes);
                    let cam = self
                        .camera
                        .clone()
                        .unwrap_or_else(|| "must(world.resource(og.Camera))".into());
                    return Done::Lines(vec![format!("{cam}.background = {c};")], S_LIGHT);
                }
                match assignment(rest) {
                    Some(("fog", _)) => Done::Manual("og.Fog's color, start and end (density for FogExp2), with enabled true".into(), S_LIGHT),
                    Some(("environment", _)) => Done::Manual("og.Environment's map, from og.load.environment of a type environment asset".into(), S_MATERIAL),
                    _ => Done::Manual(format!("no og mapping for the scene's `{}`", short(rest)), S_GRAPH),
                }
            }
            Var::Renderer => {
                let head = rest.split(['(', '.', ' ', '=']).next().unwrap_or("");
                match head {
                    "setAnimationLoop" => Done::Manual("the render loop: its body goes in systems on the tick, and the engine draws every frame".into(), S_TICK),
                    "toneMapping" | "toneMappingExposure" => Done::Manual("RenderSettings.toneMapping (\"aces\", \"agx\", \"neutral\", \"none\") and exposure".into(), S_LIGHT),
                    "setClearColor" => Done::Manual("the clear colour is og.Camera's background (linear: color.fromHex of the hex)".into(), S_LIGHT),
                    "render" | "setSize" | "setPixelRatio" | "shadowMap" | "outputColorSpace" | "outputEncoding" | "physicallyCorrectLights" | "useLegacyLights" => {
                        Done::Removed("the engine owns the renderer, its size and its shadows".into(), S_LIGHT)
                    }
                    _ => Done::Manual(format!("no og mapping for the renderer's `{}`", short(rest)), S_LIGHT),
                }
            }
            Var::Camera => {
                if let Some(a) = whole_call(rest, "position.set") {
                    return Done::Lines(
                        vec![format!("{name}.position = {};", vec3(&ex(self, &a)))],
                        S_LIGHT,
                    );
                }
                if let Some(a) = whole_call(rest, "lookAt") {
                    let a = ex(self, &a);
                    let target = if a.len() == 3 { vec3(&a) } else { a.join(", ") };
                    return Done::Lines(vec![format!("{name}.lookAt({target});")], S_LIGHT);
                }
                if let Some(a) = whole_call(rest, "rotation.set") {
                    return Done::Lines(
                        vec![format!(
                            "{name}.quaternion = quat.fromEuler({});",
                            euler(&ex(self, &a))
                        )],
                        S_OBJECT,
                    );
                }
                if whole_call(rest, "updateProjectionMatrix").is_some()
                    || matches!(assignment(rest), Some(("aspect", _)))
                {
                    return Done::Removed(
                        "the engine sizes the projection to the play surface".into(),
                        S_LIGHT,
                    );
                }
                if matches!(assignment(rest), Some(("fov" | "near" | "far" | "zoom", _))) {
                    return Done::Lines(Vec::new(), 0); // same name on og.Camera: left as written
                }
                Done::Manual(
                    format!("no og mapping for the camera's `{}`", short(rest)),
                    S_LIGHT,
                )
            }
            Var::Object | Var::Mesh | Var::Light(_) => {
                let aimed = matches!(var, Var::Light(l) if l.aimed);
                let position = |p: String| {
                    if aimed {
                        format!(
                            "position: {p}, quaternion: og.transform.lookAt({p}, {{ x: 0, y: 0, z: 0 }})"
                        )
                    } else {
                        format!("position: {p}")
                    }
                };
                if let Some(a) = whole_call(rest, "position.set") {
                    return Done::Lines(vec![transform(&position(vec3(&ex(self, &a))))], S_OBJECT);
                }
                if let Some(a) = whole_call(rest, "position.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&position(ex(self, &a).remove(0)))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "quaternion.set").filter(|a| a.len() == 4) {
                    let a = ex(self, &a);
                    return Done::Lines(
                        vec![transform(&format!(
                            "quaternion: {{ x: {}, y: {}, z: {}, w: {} }}",
                            a[0], a[1], a[2], a[3]
                        ))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "quaternion.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&format!("quaternion: {}", ex(self, &a)[0]))],
                        S_OBJECT,
                    );
                }
                if let Some(a) =
                    whole_call(rest, "rotation.set").filter(|a| (3..=4).contains(&a.len()))
                {
                    return Done::Lines(
                        vec![transform(&format!(
                            "quaternion: quat.fromEuler({})",
                            euler(&ex(self, &a))
                        ))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.set").filter(|a| a.len() == 3) {
                    return Done::Lines(
                        vec![transform(&format!("scale: {}", vec3(&ex(self, &a))))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.setScalar")
                    .filter(|a| a.len() == 1 && (is_ident(&a[0]) || a[0].parse::<f64>().is_ok()))
                {
                    let s = &a[0];
                    return Done::Lines(
                        vec![transform(&format!("scale: {{ x: {s}, y: {s}, z: {s} }}"))],
                        S_OBJECT,
                    );
                }
                if let Some(a) = whole_call(rest, "scale.copy").filter(|a| a.len() == 1) {
                    return Done::Lines(
                        vec![transform(&format!("scale: {}", ex(self, &a)[0]))],
                        S_OBJECT,
                    );
                }
                if let Some(children) = whole_call(rest, "add") {
                    let mut lines = Vec::new();
                    for c in &children {
                        if !matches!(
                            self.vars.get(c.as_str()),
                            Some(Var::Object | Var::Mesh | Var::Light(_))
                        ) {
                            return Done::Manual(
                                format!(
                                    "`{c}` is not an entity this file spawns: give it og.Parent {{ entity: {name} }} once it is one"
                                ),
                                S_GRAPH,
                            );
                        }
                        lines.push(format!(
                            "world.insert({c}, og.Parent, {{ entity: {name} }});"
                        ));
                    }
                    return Done::Lines(lines, S_GRAPH);
                }
                if let Some(children) = whole_call(rest, "remove") {
                    return Done::Lines(
                        children
                            .iter()
                            .map(|c| format!("world.remove({c}, og.Parent);"))
                            .collect(),
                        S_GRAPH,
                    );
                }
                if let Var::Light(light) = var
                    && let Some(a) = whole_call(rest, "color.set")
                        .or_else(|| whole_call(rest, "color.setHex"))
                        .filter(|a| a.len() == 1)
                {
                    let c = self.color_arg(&a[0], notes);
                    return Done::Lines(
                        vec![format!(
                            "world.set({name}, og.{}, {{ color: {c} }});",
                            light.component
                        )],
                        S_LIGHT,
                    );
                }
                if let Some((field, value)) = assignment(rest) {
                    let component = match var {
                        Var::Mesh if MESH_FIELDS.contains(&field) => Some("Mesh"),
                        Var::Light(l)
                            if LIGHT_FIELDS.contains(&field)
                                && (field == "castShadow" || l.params.contains(&Some(field))) =>
                        {
                            Some(l.component)
                        }
                        _ => None,
                    };
                    if let Some(component) = component {
                        let v = self.expr(value, notes);
                        return Done::Lines(
                            vec![format!(
                                "world.set({name}, og.{component}, {{ {field}: {v} }});"
                            )],
                            S_OBJECT,
                        );
                    }
                    if field == "name" {
                        return Done::Manual(
                            "an entity is named at its spawn (SpawnDesc.name), once".into(),
                            S_OBJECT,
                        );
                    }
                }
                let what = if rest.starts_with("position.")
                    || rest.starts_with("rotation.")
                    || rest.starts_with("scale.")
                    || rest.starts_with("quaternion.")
                {
                    "og.Transform is read and written whole: world.get, change the value, world.set (or a system over the column)"
                } else if rest.starts_with("lookAt(") {
                    "og.transform.lookAt(eye, target) points -Z at the target (a camera or a light); an object's +Z: og.transform.lookAt(target, eye)"
                } else if rest.starts_with("shadow.") || rest.starts_with("target.") {
                    "a light's shadow is its castShadow and RenderOptions.shadowMapSize; aim it with og.transform.lookAt"
                } else if rest.starts_with("traverse(") {
                    "a system's query over the components of what it visits"
                } else {
                    "no og mapping for this member of an Object3D"
                };
                Done::Manual(what.into(), S_OBJECT)
            }
        }
    }
}
