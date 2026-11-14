//! Track keys and caption imports read from timeline text.

use super::*;

/// Sorts keys by time, keeping the file's order among equal times.
fn sorted<K>(mut keys: Vec<K>, time: impl Fn(&K) -> f32) -> Vec<K> {
    og_core::sort::sort_by(&mut keys, |a, b| time(a).total_cmp(&time(b)));
    keys
}

pub(super) fn track(e: &Entry, read: &mut Read<'_>) -> Result<Track, String> {
    let kind = e
        .values
        .first()
        .and_then(Value::as_word)
        .ok_or_else(|| at(e, "a track names its kind: `track camera {`"))?;
    let target = match e.values.get(1) {
        None => None,
        Some(Value::String(s)) if e.values.len() == 2 => Some(s.clone()),
        _ => return Err(at(e, "a track is `track <kind> [\"entity\"] {`")),
    };
    let needs_target = matches!(
        kind,
        "transform" | "animation" | "light" | "visibility" | "blendshape" | "material"
    );
    let may_target = needs_target || kind == "property";
    match (&target, needs_target) {
        _ if may_target && !needs_target => {}
        (None, true) => {
            return Err(at(
                e,
                format!("a {kind} track names its entity: `track {kind} \"crate\" {{`"),
            ));
        }
        (Some(_), false) => {
            return Err(at(e, format!("a {kind} track names no entity")));
        }
        _ => {}
    }
    let empty = Block::new();
    let block = e.block.as_ref().unwrap_or(&empty);
    let target = target.unwrap_or_default();
    Ok(match kind {
        "camera" => Track::Camera(camera(block)?),
        "transform" => Track::Transform(transform(block, target)?),
        "animation" => Track::Animation(animation(block, target)?),
        "audio" => Track::Audio(sounds(block)?),
        "music" => Track::Music(music(block)?),
        "caption" => Track::Caption(caption(block, read)?),
        "event" => Track::Event(events(block)?),
        "transition" => Track::Transition(transitions(block)?),
        "marker" => Track::Marker(markers(block)?),
        "timeline" => Track::Timeline(nested(block)?),
        "shot" => Track::Shot(shots(block)?),
        k => match FieldsKind::from_name(k) {
            Some(kind) => Track::Fields(fields(block, kind, target)?),
            None => {
                return Err(at(
                    e,
                    format!(
                        "track kind '{k}' is not one this engine (og {}) reads: {TRACK_KINDS}. A timeline written for a newer engine needs that engine",
                        env!("CARGO_PKG_VERSION")
                    ),
                ));
            }
        },
    })
}

fn only_keys<'a>(block: &'a Block, what: &str) -> Result<Vec<&'a Entry>, String> {
    for e in &block.entries {
        if e.key != "key" {
            return Err(unknown(e, &format!("{what} track entry"), "key"));
        }
    }
    Ok(keys(block).collect())
}

fn camera(block: &Block) -> Result<CameraTrack, String> {
    let mut c = CameraTrack::default();
    let mut ks = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "rail" | "follow" => {
                no_block(e)?;
                if c.mode != CameraMode::Keys {
                    return Err(at(e, "a camera track has one `rail` or one `follow`"));
                }
                let s = string(e)?;
                c.mode = if e.key == "rail" {
                    CameraMode::Rail(s)
                } else {
                    CameraMode::Follow(s)
                };
            }
            "key" => {
                let mut k = CameraKey {
                    time: key_time(e)?,
                    ..CameraKey::default()
                };
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "position" => k.position = Some(vec3(f)?),
                        "target" => k.target = Some(vec3(f)?),
                        "at" => k.at = Some(one_number(f)?),
                        "fov" => {
                            let v = one_number(f)?;
                            if !(1.0..=179.0).contains(&v) {
                                return Err(at(f, "fov is degrees, 1 to 179"));
                            }
                            k.fov = Some(v);
                        }
                        "roll" => k.roll = Some(one_number(f)?),
                        "focus" => {
                            let v = one_number(f)?;
                            if v <= 0.0 {
                                return Err(at(f, "focus is a distance in metres, more than 0"));
                            }
                            k.focus = Some(v);
                        }
                        "focusOn" => k.focus_on = Some(string(f)?),
                        "focusRange" => {
                            let v = one_number(f)?;
                            if v < 0.0 {
                                return Err(at(f, "focusRange is metres, 0 or more"));
                            }
                            k.focus_range = Some(v);
                        }
                        "ease" => k.ease = ease(f)?,
                        _ => {
                            return Err(unknown(
                                f,
                                "camera key entry",
                                "position, target, at, fov, roll, focus, focusOn, focusRange or ease",
                            ));
                        }
                    }
                }
                ks.push((e, k));
            }
            _ => return Err(unknown(e, "camera track entry", "rail, follow or key")),
        }
    }
    for (e, k) in &ks {
        match c.mode {
            CameraMode::Rail(_) if k.position.is_some() => {
                return Err(at(
                    e,
                    "a camera on a rail is placed by `at` (0..1 along the path), not `position`",
                ));
            }
            CameraMode::Keys | CameraMode::Follow(_) if k.at.is_some() => {
                return Err(at(
                    e,
                    "`at` places a camera on a rail: add `rail \"<path>\"` to the track",
                ));
            }
            _ => {}
        }
    }
    c.keys = sorted(ks.into_iter().map(|(_, k)| k).collect(), |k| k.time);
    Ok(c)
}

fn transform(block: &Block, entity: String) -> Result<TransformTrack, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "transform")? {
        let mut k = TransformKey {
            time: key_time(e)?,
            ..TransformKey::default()
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "position" => k.position = Some(vec3(f)?),
                "scale" => k.scale = Some(vec3(f)?),
                "rotation" => k.quaternion = Some(euler_degrees(vec3(f)?)),
                "quaternion" => {
                    if f.values.len() != 4 {
                        return Err(at(f, "`quaternion` takes four numbers: x y z w"));
                    }
                    let q =
                        Quat::from_xyzw(number(f, 0)?, number(f, 1)?, number(f, 2)?, number(f, 3)?);
                    if q.length_squared() < 1e-12 {
                        return Err(at(f, "a quaternion of length 0 is no rotation"));
                    }
                    k.quaternion = Some(q.normalize());
                }
                "ease" => k.ease = ease(f)?,
                _ => {
                    return Err(unknown(
                        f,
                        "transform key entry",
                        "position, rotation (degrees, XYZ), quaternion, scale or ease",
                    ));
                }
            }
        }
        ks.push(k);
    }
    Ok(TransformTrack {
        entity,
        keys: sorted(ks, |k| k.time),
    })
}

fn animation(block: &Block, entity: String) -> Result<AnimationTrack, String> {
    let mut a = AnimationTrack {
        entity,
        ..AnimationTrack::default()
    };
    let mut ks = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "model" => {
                no_block(e)?;
                a.model = string(e)?;
            }
            "key" => {
                let mut k = AnimationKey {
                    time: key_time(e)?,
                    clip: String::new(),
                    speed: 1.0,
                    blend_in: 0.2,
                    looping: true,
                };
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "clip" => k.clip = string(f)?,
                        "speed" => k.speed = one_number(f)?,
                        "blendIn" => {
                            k.blend_in = one_number(f)?;
                            if k.blend_in < 0.0 {
                                return Err(at(f, "blendIn is seconds, ≥ 0"));
                            }
                        }
                        "loop" => k.looping = flag(f)?,
                        _ => {
                            return Err(unknown(
                                f,
                                "animation key entry",
                                "clip, speed, blendIn or loop",
                            ));
                        }
                    }
                }
                if k.clip.is_empty() {
                    return Err(at(e, "an animation key names its clip: `clip \"walk\"`"));
                }
                ks.push(k);
            }
            _ => return Err(unknown(e, "animation track entry", "model or key")),
        }
    }
    a.keys = sorted(ks, |k| k.time);
    Ok(a)
}

fn bus(f: &Entry) -> Result<u32, String> {
    let w = word(f)?;
    bus_code(w).ok_or_else(|| at(f, format!("bus '{w}' is not master, music, sfx or ui")))
}

fn volume(f: &Entry) -> Result<f32, String> {
    let v = one_number(f)?;
    if v < 0.0 {
        return Err(at(f, "volume is a linear gain, ≥ 0"));
    }
    Ok(v)
}

fn sounds(block: &Block) -> Result<Vec<SoundKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "audio")? {
        let mut k = SoundKey {
            time: key_time(e)?,
            sound: String::new(),
            volume: 1.0,
            pitch: 1.0,
            bus: 2,
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "sound" => k.sound = string(f)?,
                "volume" => k.volume = volume(f)?,
                "playbackRate" => {
                    k.pitch = one_number(f)?;
                    if k.pitch <= 0.0 {
                        return Err(at(f, "playbackRate is a rate multiplier, more than 0"));
                    }
                }
                "bus" => k.bus = bus(f)?,
                _ => {
                    return Err(unknown(
                        f,
                        "audio key entry",
                        "sound, volume, playbackRate or bus",
                    ));
                }
            }
        }
        if k.sound.is_empty() {
            return Err(at(
                e,
                "an audio key names its sound: `sound \"audio/hit.wav\"`",
            ));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn music(block: &Block) -> Result<Vec<MusicKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "music")? {
        let mut k = MusicKey {
            time: key_time(e)?,
            sound: String::new(),
            volume: 1.0,
            fade_in: 0.0,
            fade_out: 0.0,
            end: None,
            looping: true,
            bus: 1,
        };
        for f in key_block(e)? {
            no_block(f)?;
            let secs = |f: &Entry| -> Result<f32, String> {
                let v = one_number(f)?;
                if v < 0.0 {
                    return Err(at(f, format!("{} is seconds, ≥ 0", f.key)));
                }
                Ok(v)
            };
            match f.key.as_str() {
                "sound" => k.sound = string(f)?,
                "volume" => k.volume = volume(f)?,
                "fadeIn" => k.fade_in = secs(f)?,
                "fadeOut" => k.fade_out = secs(f)?,
                "end" => k.end = Some(secs(f)?),
                "loop" => k.looping = flag(f)?,
                "bus" => k.bus = bus(f)?,
                _ => {
                    return Err(unknown(
                        f,
                        "music key entry",
                        "sound, volume, fadeIn, fadeOut, end, loop or bus",
                    ));
                }
            }
        }
        if k.sound.is_empty() {
            return Err(at(
                e,
                "a music key names its sound: `sound \"audio/theme.wav\"`",
            ));
        }
        if k.end.is_some_and(|end| end <= k.time) {
            return Err(at(e, "a music key's end comes after its time"));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

/// A caption as written: time, end if given, text.
type RawCaption = (f32, Option<f32>, String);

/// What `style` and `anchor`, which a caption had, are answered with.
const CAPTION_LOOK: &str = "a caption is text only: the screen that shows `$captions` (the built-in `subtitles`) and the style pack decide where and how it is drawn";

fn caption(block: &Block, read: &mut Read<'_>) -> Result<CaptionTrack, String> {
    let mut c = CaptionTrack::default();
    let mut raw: Vec<RawCaption> = Vec::new();
    for e in &block.entries {
        match e.key.as_str() {
            "style" | "anchor" => return Err(at(e, CAPTION_LOOK)),
            "captions" => {
                no_block(e)?;
                let (file, offset) = match e.values.as_slice() {
                    [Value::String(f)] => (f.clone(), 0.0),
                    [Value::String(f), Value::Number(n)] => (f.clone(), n.as_f32()),
                    _ => {
                        return Err(at(
                            e,
                            "`captions \"<file>.srt\" [offset seconds]` (or a word-timestamp .json)",
                        ));
                    }
                };
                let text = read(&file).map_err(|m| at(e, format!("captions '{file}': {m}")))?;
                let cues =
                    import(&file, &text).map_err(|m| at(e, format!("captions '{file}': {m}")))?;
                for q in cues {
                    let start = q.start + offset;
                    if start < 0.0 {
                        return Err(at(
                            e,
                            format!(
                                "captions '{file}': a cue starts before 0 with offset {offset}"
                            ),
                        ));
                    }
                    raw.push((start, Some(q.end + offset), q.text));
                }
            }
            "key" => {
                let time = key_time(e)?;
                let (mut end, mut text) = (None, None);
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "text" => text = Some(string(f)?),
                        "end" => end = Some(one_number(f)?),
                        "style" | "anchor" => return Err(at(f, CAPTION_LOOK)),
                        _ => return Err(unknown(f, "caption key entry", "text or end")),
                    }
                }
                let text = text.ok_or_else(|| at(e, "a caption key has `text \"...\"`"))?;
                if end.is_some_and(|x| x <= time) {
                    return Err(at(e, "a caption's end comes after its time"));
                }
                raw.push((time, end, text));
            }
            _ => {
                return Err(unknown(e, "caption track entry", "captions or key"));
            }
        }
    }
    let raw = sorted(raw, |r| r.0);
    for (i, (time, end, text)) in raw.iter().enumerate() {
        let next = raw.get(i + 1).map(|r| r.0);
        let end = end.unwrap_or_else(|| {
            next.unwrap_or(time + CAPTION_SECONDS)
                .min(time + CAPTION_SECONDS)
        });
        c.keys.push(CaptionKey {
            time: *time,
            end,
            text: text.clone(),
        });
    }
    Ok(c)
}

/// A caption file by its extension: SubRip, or word timestamps in JSON.
fn import(file: &str, text: &str) -> Result<Vec<Cue>, String> {
    let lower = file.to_ascii_lowercase();
    if lower.ends_with(".srt") {
        return captions::parse_srt(text);
    }
    if lower.ends_with(".json") {
        #[cfg(feature = "import")]
        return captions::parse_word_json(text, CAPTION_CHARS, CAPTION_GAP);
        #[cfg(not(feature = "import"))]
        return Err("word-timestamp JSON is imported by the cook (og cook)".into());
    }
    Err("a caption file is .srt (SubRip) or .json (word timestamps)".into())
}

fn events(block: &Block) -> Result<Vec<EventKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "event")? {
        let mut k = EventKey {
            time: key_time(e)?,
            ..EventKey::default()
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "do" => k.action = string(f)?,
                "retroactive" => k.retroactive = flag(f)?,
                _ => return Err(unknown(f, "event key entry", "do or retroactive")),
            }
        }
        let name = k.action.split(':').next().unwrap_or("");
        let mut chars = name.chars();
        let valid = matches!(chars.next(), Some(c) if c.is_ascii_lowercase() || c == '_')
            && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
        if !valid {
            return Err(at(
                e,
                "an event key runs a command-channel action: `do \"teleport:crate:0,4,0\"` (the action, then its `:`-separated arguments)",
            ));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn transitions(block: &Block) -> Result<Vec<TransitionKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "transition")? {
        let mut k = TransitionKey {
            time: key_time(e)?,
            kind: TransitionKind::Fade,
            mode: TransitionMode::Through,
            duration: 1.0,
            color: Vec3::ZERO,
            from: 0,
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "kind" => {
                    let w = word(f)?;
                    k.kind = TransitionKind::from_name(w).ok_or_else(|| {
                        at(
                            f,
                            format!(
                                "transition kind '{w}' is not {}",
                                TransitionKind::NAMES.join(", ")
                            ),
                        )
                    })?;
                }
                "mode" => {
                    let w = word(f)?;
                    k.mode = TransitionMode::from_name(w).ok_or_else(|| {
                        at(
                            f,
                            format!("transition mode '{w}' is not through, in or out"),
                        )
                    })?;
                }
                "duration" => {
                    k.duration = one_number(f)?;
                    if k.duration < 0.0 {
                        return Err(at(f, "duration is seconds, ≥ 0"));
                    }
                }
                "color" => k.color = vec3(f)?,
                "from" => {
                    let w = word(f)?;
                    k.from = SIDES.iter().position(|s| *s == w).ok_or_else(|| {
                        at(f, format!("from '{w}' is not left, right, top or bottom"))
                    })? as u32;
                }
                _ => {
                    return Err(unknown(
                        f,
                        "transition key entry",
                        "kind, mode, duration, color or from",
                    ));
                }
            }
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

fn markers(block: &Block) -> Result<Vec<MarkerKey>, String> {
    let mut ks = Vec::new();
    for e in only_keys(block, "marker")? {
        let mut k = MarkerKey {
            time: key_time(e)?,
            name: String::new(),
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "name" => k.name = string(f)?,
                _ => return Err(unknown(f, "marker key entry", "name")),
            }
        }
        if k.name.is_empty() || k.name.len() > 63 {
            return Err(at(e, "a marker key has a `name \"...\"` of 1 to 63 bytes"));
        }
        ks.push(k);
    }
    Ok(sorted(ks, |k| k.time))
}

/// A key's value: one to four numbers, or one word (true, false, an enum
/// variant).
fn field_value(f: &Entry) -> Result<FieldValue, String> {
    if let [Value::Word(w)] = f.values.as_slice() {
        return Ok(FieldValue::Word(w.clone()));
    }
    if f.values.is_empty() || f.values.len() > 4 {
        return Err(at(
            f,
            format!(
                "`{}` takes one to four numbers (a scalar, a vector, a colour, a quaternion) or one word (true, false, an enum variant)",
                f.key
            ),
        ));
    }
    Ok(FieldValue::Numbers(
        (0..f.values.len())
            .map(|i| number(f, i))
            .collect::<Result<_, _>>()?,
    ))
}

fn fields(block: &Block, kind: FieldsKind, entity: String) -> Result<FieldsTrack, String> {
    let mut t = FieldsTrack {
        kind,
        entity,
        index: 0,
        keys: Vec::new(),
    };
    let property = kind == FieldsKind::Property;
    let mut field: Option<String> = None;
    for e in &block.entries {
        match e.key.as_str() {
            "field" if property => {
                no_block(e)?;
                let f = string(e)?;
                let ok = f
                    .rsplit_once('.')
                    .is_some_and(|(r, n)| !r.is_empty() && !n.is_empty());
                if !ok {
                    return Err(at(
                        e,
                        format!(
                            "field '{f}' is a record and its field: \"og.PointLight.intensity\""
                        ),
                    ));
                }
                field = Some(f);
            }
            "index" if kind == FieldsKind::Material => {
                no_block(e)?;
                let v = one_number(e)?;
                if v < 0.0 || v.fract() != 0.0 {
                    return Err(at(e, "index is a whole number: the model's material"));
                }
                t.index = v as u32;
            }
            "key" => {
                let mut k = FieldKey {
                    time: key_time(e)?,
                    ..FieldKey::default()
                };
                for f in key_block(e)? {
                    no_block(f)?;
                    match f.key.as_str() {
                        "ease" => k.ease = ease(f)?,
                        "value" if property => {
                            let name = field.clone().ok_or_else(|| {
                                at(f, "a property track names its field before its keys: `field \"og.PointLight.intensity\"`")
                            })?;
                            k.values.push((name, field_value(f)?));
                        }
                        _ if property => {
                            return Err(unknown(f, "property key entry", "value or ease"));
                        }
                        name => {
                            let v = field_value(f)?;
                            match kind {
                                FieldsKind::Visibility if name != "visible" => {
                                    return Err(unknown(
                                        f,
                                        "visibility key entry",
                                        "visible or ease",
                                    ));
                                }
                                FieldsKind::Visibility if !matches!(&v, FieldValue::Word(w) if w == "true" || w == "false") =>
                                {
                                    return Err(at(f, "visible takes true or false"));
                                }
                                FieldsKind::Blendshape
                                    if !(name.len() == 2
                                        && name.starts_with('w')
                                        && (b'0'..=b'7').contains(&name.as_bytes()[1])) =>
                                {
                                    return Err(unknown(
                                        f,
                                        "blendshape key entry",
                                        "w0 to w7 or ease",
                                    ));
                                }
                                _ => {}
                            }
                            if k.values.iter().any(|(n, _)| n == name) {
                                return Err(at(f, format!("`{name}` is set twice in one key")));
                            }
                            k.values.push((name.to_string(), v));
                        }
                    }
                }
                if k.values.is_empty() {
                    return Err(at(e, "a key sets at least one field"));
                }
                t.keys.push(k);
            }
            _ => {
                let allowed = match kind {
                    FieldsKind::Property => "field or key",
                    FieldsKind::Material => "index or key",
                    _ => "key",
                };
                return Err(unknown(e, &format!("{} track entry", kind.name()), allowed));
            }
        }
    }
    if property && field.is_none() {
        return Err("a property track names its field: `field \"og.PointLight.intensity\"`".into());
    }
    t.keys = sorted(t.keys, |k| k.time);
    Ok(t)
}

fn nested(block: &Block) -> Result<Vec<NestedKey>, String> {
    let mut out = Vec::new();
    for e in only_keys(block, "timeline")? {
        let mut k = NestedKey {
            time: key_time(e)?,
            play: String::new(),
            speed: 1.0,
            looping: false,
        };
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "play" => k.play = string(f)?,
                "speed" => {
                    let v = one_number(f)?;
                    if v <= 0.0 {
                        return Err(at(f, "speed is more than 0"));
                    }
                    k.speed = v;
                }
                "loop" => k.looping = flag(f)?,
                _ => return Err(unknown(f, "timeline key entry", "play, speed or loop")),
            }
        }
        if k.play.is_empty() {
            return Err(at(
                e,
                "a timeline key names what it plays: `play \"timelines/door.ogt\"`",
            ));
        }
        out.push(k);
    }
    Ok(sorted(out, |k| k.time))
}

fn shots(block: &Block) -> Result<Vec<ShotKey>, String> {
    let mut out = Vec::new();
    for e in only_keys(block, "shot")? {
        let mut k = ShotKey {
            time: key_time(e)?,
            camera: String::new(),
            blend: -1.0,
            ease: Ease::Linear,
        };
        let mut named = false;
        for f in key_block(e)? {
            no_block(f)?;
            match f.key.as_str() {
                "camera" => {
                    k.camera = string(f)?;
                    named = true;
                }
                "none" => named = true,
                "blend" => {
                    let v = one_number(f)?;
                    if v < 0.0 {
                        return Err(at(f, "blend is 0 (a cut) or more seconds"));
                    }
                    k.blend = v;
                }
                "ease" => k.ease = ease(f)?,
                _ => return Err(unknown(f, "shot key entry", "camera, none, blend or ease")),
            }
        }
        if !named {
            return Err(at(
                e,
                "a shot key names its camera, `camera \"closeup\"` (a virtual camera's entity name), or `none` to release it",
            ));
        }
        out.push(k);
    }
    Ok(sorted(out, |k| k.time))
}
