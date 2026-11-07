//! Timeline and path text parsing and canonical writing.

use super::*;

impl Timeline {
    /// Parses timeline text. `read` loads a caption file a track names
    /// (`captions "narration.srt"`); the cooked text has none.
    pub fn parse(text: &str, read: &mut Read<'_>) -> Result<Timeline, String> {
        let doc = Document::parse(text).map_err(|e| e.to_string())?;
        if doc.ty != TIMELINE_TYPE {
            return Err(format!("a timeline is `type timeline`, not `{}`", doc.ty));
        }
        if doc.version != 1 {
            return Err(format!(
                "timeline version {} (this engine reads 1)",
                doc.version
            ));
        }
        let mut t = Timeline {
            name: String::new(),
            duration: 0.0,
            paths: Vec::new(),
            tracks: Vec::new(),
        };
        let mut duration = None;
        for e in &doc.body.entries {
            match e.key.as_str() {
                "name" => t.name = string(e)?,
                "duration" => {
                    let d = one_number(e)?;
                    if d <= 0.0 {
                        return Err(at(e, "duration is seconds, more than 0"));
                    }
                    duration = Some(d);
                }
                "path" => {
                    let (name, desc) = path_entry(e)?;
                    if t.paths.iter().any(|(n, _)| *n == name) {
                        return Err(at(e, format!("path '{name}' is declared twice")));
                    }
                    t.paths.push((name, desc));
                }
                "track" => t.tracks.push(track(e, read)?),
                _ => {
                    return Err(unknown(
                        e,
                        "timeline entry",
                        "name, duration, path or track",
                    ));
                }
            }
        }
        let last = t.tracks.iter().map(track_end).fold(0.0f32, f32::max);
        t.duration = match duration {
            Some(d) => d,
            None if last > 0.0 => last,
            None => return Err("a timeline needs `duration` (seconds) or a key after 0".into()),
        };
        // Keys past the end never play.
        for tr in &t.tracks {
            let first_late = tr.times().into_iter().find(|&k| k > t.duration);
            if let Some(k) = first_late {
                return Err(format!(
                    "the {} track has a key at {k} s, after the timeline's duration {} s",
                    tr.kind(),
                    t.duration
                ));
            }
        }
        // Rails name a path of the file or one the world will have; the
        // runtime refuses an unknown one when it plays.
        Ok(t)
    }

    /// A path the file declares.
    pub fn path(&self, name: &str) -> Option<&PathDesc> {
        self.paths.iter().find(|(n, _)| n == name).map(|(_, d)| d)
    }

    /// The canonical text: every caption file imported as keys, every
    /// rotation as a quaternion, every key with its resolved values.
    pub fn to_text(&self) -> String {
        let mut doc = Document::new(TIMELINE_TYPE, 1);
        let b = &mut doc.body;
        if !self.name.is_empty() {
            b.push(Entry::new("name").with(Value::String(self.name.clone())));
        }
        b.push(Entry::new("duration").with(num(self.duration)));
        for (name, p) in &self.paths {
            b.push(write_path(name, p));
        }
        for tr in &self.tracks {
            b.push(write_track(tr));
        }
        doc.to_text()
            .expect("timeline text is made of words, numbers and strings")
    }
}

/// A `path "<name>" { kind, closed, tension, point... }` entry, as a
/// timeline or a scene declares one; the points are checked against the kind.
pub fn path_entry(e: &Entry) -> Result<(String, PathDesc), String> {
    let name = match e.values.as_slice() {
        [Value::String(s)] if !s.is_empty() => s.clone(),
        _ => return Err(at(e, "a path is `path \"<name>\" {`")),
    };
    let block = e
        .block
        .as_ref()
        .ok_or_else(|| at(e, "a path opens a block: `path \"name\" {`"))?;
    let desc = path_desc(block)?;
    crate::Path::new(desc.clone()).map_err(|m| at(e, format!("path '{name}': {m}")))?;
    Ok((name, desc))
}

fn write_path(name: &str, p: &PathDesc) -> Entry {
    let mut pb = Block::new();
    pb.push(Entry::new("kind").with(Value::Word(p.kind.name().into())));
    pb.push(Entry::new("closed").with(boolean(p.closed)));
    pb.push(Entry::new("tension").with(num(p.tension)));
    for q in &p.points {
        pb.push(v3(Entry::new("point"), *q));
    }
    Entry::new("path")
        .with(Value::String(name.to_string()))
        .with_block(pb)
}

/// The `.ogt` type of a scene's paths section.
pub const PATHS_TYPE: &str = "paths";

/// Named paths as canonical text (`type paths`: a scene blob's `paths`).
pub fn paths_to_text(paths: &[(String, PathDesc)]) -> String {
    let mut doc = Document::new(PATHS_TYPE, 1);
    for (name, p) in paths {
        doc.body.push(write_path(name, p));
    }
    doc.to_text()
        .expect("paths text is made of words and numbers")
}

/// Reads [`paths_to_text`]'s text back.
pub fn parse_paths(text: &str) -> Result<Vec<(String, PathDesc)>, String> {
    let doc = Document::parse(text).map_err(|e| e.to_string())?;
    if doc.ty != PATHS_TYPE {
        return Err(format!("expected `type paths`, not `{}`", doc.ty));
    }
    doc.body
        .entries
        .iter()
        .map(|e| {
            if e.key != "path" {
                return Err(unknown(e, "paths entry", "path"));
            }
            path_entry(e)
        })
        .collect()
}

fn num(v: f32) -> Value {
    Value::Number(Number::from_f32(v).unwrap_or_else(|_| Number::from(0)))
}

fn boolean(v: bool) -> Value {
    Value::Word(if v { "true" } else { "false" }.into())
}

fn v3(e: Entry, v: Vec3) -> Entry {
    e.with(num(v.x)).with(num(v.y)).with(num(v.z))
}

fn track_end(t: &Track) -> f32 {
    let max = |it: &mut dyn Iterator<Item = f32>| it.fold(0.0f32, f32::max);
    match t {
        Track::Camera(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Transform(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Animation(c) => max(&mut c.keys.iter().map(|k| k.time)),
        Track::Audio(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Music(c) => max(&mut c.iter().map(|k| k.end.unwrap_or(k.time))),
        Track::Caption(c) => max(&mut c.keys.iter().map(|k| k.end)),
        Track::Event(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Transition(c) => max(&mut c.iter().map(|k| k.time + k.duration)),
        Track::Marker(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Fields(f) => max(&mut f.keys.iter().map(|k| k.time)),
        Track::Timeline(c) => max(&mut c.iter().map(|k| k.time)),
        Track::Shot(c) => max(&mut c.iter().map(|k| k.time)),
    }
}

fn path_desc(block: &Block) -> Result<PathDesc, String> {
    let mut d = PathDesc::default();
    for e in &block.entries {
        no_block(e)?;
        match e.key.as_str() {
            "kind" => {
                let w = word(e)?;
                d.kind = PathKind::from_name(w).ok_or_else(|| {
                    at(
                        e,
                        format!("path kind '{w}' is not catmullRom, bezier or linear"),
                    )
                })?;
            }
            "closed" => d.closed = flag(e)?,
            "tension" => d.tension = one_number(e)?,
            "point" => d.points.push(vec3(e)?),
            _ => return Err(unknown(e, "path entry", "kind, closed, tension or point")),
        }
    }
    Ok(d)
}

// ── writing ────────────────────────────────────────────────────────────────

fn key_entry(time: f32, block: Block) -> Entry {
    Entry::new("key").with(num(time)).with_block(block)
}

fn ease_entry(b: &mut Block, e: Ease) {
    if e != Ease::Linear {
        let value = match e {
            Ease::CubicBezier(_) | Ease::Steps { .. } => Value::String(e.to_text()),
            _ => Value::Word(e.name().into()),
        };
        b.push(Entry::new("ease").with(value));
    }
}

fn write_track(tr: &Track) -> Entry {
    let mut b = Block::new();
    let mut head = Entry::new("track").with(Value::Word(tr.kind().into()));
    match tr {
        Track::Camera(c) => {
            match &c.mode {
                CameraMode::Keys => {}
                CameraMode::Rail(p) => b.push(Entry::new("rail").with(Value::String(p.clone()))),
                CameraMode::Follow(p) => {
                    b.push(Entry::new("follow").with(Value::String(p.clone())))
                }
            }
            for k in &c.keys {
                let mut kb = Block::new();
                if let Some(p) = k.position {
                    kb.push(v3(Entry::new("position"), p));
                }
                if let Some(a) = k.at {
                    kb.push(Entry::new("at").with(num(a)));
                }
                if let Some(t) = k.target {
                    kb.push(v3(Entry::new("target"), t));
                }
                if let Some(f) = k.fov {
                    kb.push(Entry::new("fov").with(num(f)));
                }
                if let Some(r) = k.roll {
                    kb.push(Entry::new("roll").with(num(r)));
                }
                if let Some(f) = k.focus {
                    kb.push(Entry::new("focus").with(num(f)));
                }
                if let Some(e) = &k.focus_on {
                    kb.push(Entry::new("focusOn").with(Value::String(e.clone())));
                }
                if let Some(r) = k.focus_range {
                    kb.push(Entry::new("focusRange").with(num(r)));
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Transform(t) => {
            head = head.with(Value::String(t.entity.clone()));
            for k in &t.keys {
                let mut kb = Block::new();
                if let Some(p) = k.position {
                    kb.push(v3(Entry::new("position"), p));
                }
                if let Some(q) = k.quaternion {
                    kb.push(
                        Entry::new("quaternion")
                            .with(num(q.x))
                            .with(num(q.y))
                            .with(num(q.z))
                            .with(num(q.w)),
                    );
                }
                if let Some(s) = k.scale {
                    kb.push(v3(Entry::new("scale"), s));
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Animation(a) => {
            head = head.with(Value::String(a.entity.clone()));
            if !a.model.is_empty() {
                b.push(Entry::new("model").with(Value::String(a.model.clone())));
            }
            for k in &a.keys {
                let mut kb = Block::new();
                kb.push(Entry::new("clip").with(Value::String(k.clip.clone())));
                kb.push(Entry::new("speed").with(num(k.speed)));
                kb.push(Entry::new("blendIn").with(num(k.blend_in)));
                kb.push(Entry::new("loop").with(boolean(k.looping)));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Shot(ks) => {
            for k in ks {
                let mut kb = Block::new();
                if k.camera.is_empty() {
                    kb.push(Entry::new("none"));
                } else {
                    kb.push(Entry::new("camera").with(Value::String(k.camera.clone())));
                }
                if k.blend >= 0.0 {
                    kb.push(Entry::new("blend").with(num(k.blend)));
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Audio(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("sound").with(Value::String(k.sound.clone())));
                kb.push(Entry::new("volume").with(num(k.volume)));
                kb.push(Entry::new("playbackRate").with(num(k.pitch)));
                kb.push(Entry::new("bus").with(Value::Word(bus_name(k.bus).into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Music(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("sound").with(Value::String(k.sound.clone())));
                kb.push(Entry::new("volume").with(num(k.volume)));
                kb.push(Entry::new("fadeIn").with(num(k.fade_in)));
                kb.push(Entry::new("fadeOut").with(num(k.fade_out)));
                if let Some(end) = k.end {
                    kb.push(Entry::new("end").with(num(end)));
                }
                kb.push(Entry::new("loop").with(boolean(k.looping)));
                kb.push(Entry::new("bus").with(Value::Word(bus_name(k.bus).into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Caption(c) => {
            b.push(Entry::new("style").with(Value::Word(c.style.clone())));
            b.push(Entry::new("anchor").with(Value::Word(c.anchor.clone())));
            for k in &c.keys {
                let mut kb = Block::new();
                kb.push(Entry::new("text").with(Value::String(k.text.clone())));
                kb.push(Entry::new("end").with(num(k.end)));
                if let Some(s) = &k.style {
                    kb.push(Entry::new("style").with(Value::Word(s.clone())));
                }
                if let Some(a) = &k.anchor {
                    kb.push(Entry::new("anchor").with(Value::Word(a.clone())));
                }
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Event(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("do").with(Value::String(k.action.clone())));
                if k.retroactive {
                    kb.push(Entry::new("retroactive").with(boolean(true)));
                }
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Transition(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("kind").with(Value::Word(k.kind.name().into())));
                kb.push(Entry::new("mode").with(Value::Word(k.mode.name().into())));
                kb.push(Entry::new("duration").with(num(k.duration)));
                kb.push(v3(Entry::new("color"), k.color));
                kb.push(Entry::new("from").with(Value::Word(SIDES[k.from as usize].into())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Marker(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("name").with(Value::String(k.name.clone())));
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Fields(f) => {
            if !f.entity.is_empty() {
                head = head.with(Value::String(f.entity.clone()));
            }
            let property = f.kind == FieldsKind::Property;
            if property && let Some(field) = f.fields().first() {
                b.push(Entry::new("field").with(Value::String((*field).to_string())));
            }
            if f.index != 0 {
                b.push(Entry::new("index").with(num(f.index as f32)));
            }
            for k in &f.keys {
                let mut kb = Block::new();
                for (name, v) in &k.values {
                    let mut e = Entry::new(if property { "value" } else { name.as_str() });
                    match v {
                        FieldValue::Numbers(ns) => {
                            for n in ns {
                                e = e.with(num(*n));
                            }
                        }
                        FieldValue::Word(w) => e = e.with(Value::Word(w.clone())),
                    }
                    kb.push(e);
                }
                ease_entry(&mut kb, k.ease);
                b.push(key_entry(k.time, kb));
            }
        }
        Track::Timeline(ks) => {
            for k in ks {
                let mut kb = Block::new();
                kb.push(Entry::new("play").with(Value::String(k.play.clone())));
                kb.push(Entry::new("speed").with(num(k.speed)));
                kb.push(Entry::new("loop").with(boolean(k.looping)));
                b.push(key_entry(k.time, kb));
            }
        }
    }
    head.with_block(b)
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEXT: &str = r#"type timeline
version 1
---
name "Intro"
duration 8
path "flyby" {
  point -6 3 9
  point 0 2.5 7
  point 5 2 5
}
track camera {
  rail "flyby"
  key 0 {
    at 0
    target 0 1 0
    fov 50
  }
  key 6 {
    at 1
    ease power2.inOut
  }
}
track transform "crate" {
  key 3 {
    rotation 0 90 0
    position 1 2 3
  }
  key 1 {
    position 0 0 0
  }
}
track caption {
  captions "narration.srt" 0.5
  key 7 {
    text "Press \"Start\""
  }
}
track event {
  key 2 {
    do "teleport:crate:0,4,0"
    retroactive true
  }
}
track transition {
  key 7.5 {
    mode out
    duration 0.5
  }
}
track marker {
  key 3 {
    name "drop"
  }
}
"#;

    fn read(file: &str) -> Result<String, String> {
        assert_eq!(file, "narration.srt");
        Ok("1\n00:00:01,000 --> 00:00:02,500\nHello.\n\n2\n00:00:03,000 --> 00:00:04,000\nAgain.\n".into())
    }

    #[test]
    fn parses_sorts_imports_and_round_trips_canonically() {
        let t = Timeline::parse(TEXT, &mut read).unwrap();
        assert_eq!(t.name, "Intro");
        assert_eq!(t.duration, 8.0);
        let Track::Transform(tr) = &t.tracks[1] else {
            panic!()
        };
        assert_eq!(tr.keys[0].time, 1.0, "keys are sorted by time");
        let q = tr.keys[1].quaternion.unwrap();
        assert!(
            (q.y - std::f32::consts::FRAC_1_SQRT_2).abs() < 1e-6,
            "{q:?}"
        );
        let Track::Caption(c) = &t.tracks[2] else {
            panic!()
        };
        let cues: Vec<_> = c
            .keys
            .iter()
            .map(|k| (k.time, k.end, k.text.as_str()))
            .collect();
        assert_eq!(
            cues,
            vec![
                (1.5, 3.0, "Hello."),
                (3.5, 4.5, "Again."),
                (7.0, 10.0, "Press \"Start\"")
            ]
        );
        let text = t.to_text();
        let back = Timeline::parse(&text, &mut |_| Err("no files".into())).unwrap();
        assert_eq!(
            back, t,
            "the canonical text reads back to the same timeline:\n{text}"
        );
        assert_eq!(back.to_text(), text);
    }

    #[test]
    fn parameterized_easing_survives_timeline_text() {
        for curve in ["cubic-bezier(0.17, -0.3, 0.71, 1.4)", "steps(7, jump-both)"] {
            let text = TEXT.replace("ease power2.inOut", &format!("ease \"{curve}\""));
            let timeline = Timeline::parse(&text, &mut read).unwrap();
            let canonical = timeline.to_text();
            assert!(canonical.contains(&format!("ease \"{curve}\"")));
            assert_eq!(Timeline::parse(&canonical, &mut read).unwrap(), timeline);
        }
        for curve in [
            "cubic-bezier(-0.1, 0, 1, 1)",
            "cubic-bezier(0, 1, 0)",
            "steps(0)",
            "steps(4, sideways)",
            "steps(1, jump-none)",
            "linear",
            "ease-in",
        ] {
            let text = TEXT.replace("ease power2.inOut", &format!("ease \"{curve}\""));
            assert!(
                Timeline::parse(&text, &mut read)
                    .unwrap_err()
                    .contains(curve)
            );
        }
        for keyword in ["linear", "ease-in"] {
            assert!(Ease::from_css(keyword).is_some());
            let text = TEXT.replace("ease power2.inOut", &format!("ease {keyword}"));
            assert!(Timeline::parse(&text, &mut read).is_err());
        }
    }

    #[test]
    fn refusals_name_the_line_and_the_fix() {
        let cases = [
            (
                "track camera {\n  key 0 {\n    position 0 0\n  }\n}\n",
                "line 6: `position` takes three numbers",
            ),
            ("track transform {\n}\n", "names its entity"),
            (
                "track camera {\n  key 1 {\n    at 0.5\n  }\n}\n",
                "add `rail",
            ),
            (
                "duration 1\ntrack marker {\n  key 2 {\n    name \"x\"\n  }\n}\n",
                "after the timeline's duration",
            ),
            (
                "track event {\n  key 1 {\n    do \"Teleport\"\n  }\n}\n",
                "command-channel action",
            ),
            (
                "track audio {\n  key 1 {\n    volume 1\n  }\n}\n",
                "names its sound",
            ),
            (
                "path \"p\" {\n  kind bezier\n  point 0 0 0\n  point 1 0 0\n}\n",
                "path 'p': an open bezier",
            ),
            ("track wobble {\n}\n", "track kind 'wobble'"),
        ];
        for (body, want) in cases {
            let text = format!("type timeline\nversion 1\n---\n{body}");
            let e = Timeline::parse(&text, &mut |_| Err("none".into())).unwrap_err();
            assert!(e.contains(want), "{body}: {e}");
        }
    }

    /// Every R10 track and camera key reads, writes canonically and reads
    /// back the same; an unknown kind is refused naming it and this engine.
    #[test]
    fn r10_tracks_round_trip_and_an_unknown_kind_names_the_engine() {
        let text = r#"type timeline
version 1
---
duration 4
track camera {
  key 0 {
    position 0 2 6
    target 0 1 0
    roll -10
    focusOn "hero"
    focusRange 1.5
  }
  key 2 {
    roll 15
    focus 4
    ease power1.inOut
  }
}
track property "lamp" {
  field "og.PointLight.distance"
  key 0 {
    value 2
  }
  key 3 {
    value 9
    ease power2.out
  }
}
track light "lamp" {
  key 1 {
    color 1 0.5 0.2
    intensity 3
  }
}
track visibility "crate" {
  key 2 {
    visible false
  }
}
track post {
  key 0 {
    bloomIntensity 0.4
  }
}
track blendshape "face" {
  key 0 {
    w0 0
  }
  key 1 {
    w0 1
  }
}
track material "lamp" {
  index 2
  key 0 {
    emissive 1 0.4 0
    emissiveIntensity 0
  }
}
track timeline {
  key 1 {
    play "timelines/door.ogt"
    speed 2
    loop true
  }
}
"#;
        let mut no = |_: &str| -> Result<String, String> { Err("none".into()) };
        let t = Timeline::parse(text, &mut no).unwrap();
        assert_eq!(
            t.tracks.iter().map(Track::kind).collect::<Vec<_>>(),
            [
                "camera",
                "property",
                "light",
                "visibility",
                "post",
                "blendshape",
                "material",
                "timeline"
            ]
        );
        let again = Timeline::parse(&t.to_text(), &mut no).unwrap();
        assert_eq!(again, t);
        let Track::Camera(c) = &t.tracks[0] else {
            panic!()
        };
        assert_eq!(
            (
                c.keys[0].roll,
                c.keys[0].focus_on.as_deref(),
                c.keys[1].focus
            ),
            (Some(-10.0), Some("hero"), Some(4.0))
        );
        let e = Timeline::parse(
            "type timeline\nversion 1\n---\nduration 1\ntrack wobble {\n}\n",
            &mut no,
        )
        .unwrap_err();
        assert!(
            e.contains("'wobble'") && e.contains(concat!("og ", env!("CARGO_PKG_VERSION"))),
            "{e}"
        );
        let e = Timeline::parse("type timeline\nversion 1\n---\nduration 1\ntrack visibility \"a\" {\n  key 0 {\n    visible 2\n  }\n}\n", &mut no).unwrap_err();
        assert!(e.contains("visible takes true or false"), "{e}");
    }
}
