//! An operation's fields: read by name and type, unknown fields refused, errors located at the field.

use super::*;

/// Every field of `o` must be in `accepted`.
pub(crate) fn only(
    o: &Map<String, Value>,
    within: &str,
    accepted: &[&str],
) -> Result<(), ModelError> {
    match o.keys().find(|k| !accepted.contains(&k.as_str())) {
        None => Ok(()),
        Some(k) => Err(ModelError::UnknownField {
            within: within.into(),
            field: k.clone(),
            accepted: accepted.iter().map(|s| String::from(*s)).collect(),
        }),
    }
}

/// An operation's own fields (besides `op`) must be the ones it takes.
pub(crate) fn check_fields(o: &Map<String, Value>, op: &str) -> Result<(), ModelError> {
    let Some((_, fields)) = OPS.iter().find(|(name, _)| *name == op) else {
        // `dispatch` names the unknown operation.
        return Ok(());
    };
    match o
        .keys()
        .find(|k| *k != "op" && !fields.contains(&k.as_str()))
    {
        None => Ok(()),
        Some(k) => Err(ModelError::UnknownField {
            within: op.into(),
            field: k.clone(),
            accepted: fields.iter().map(|s| String::from(*s)).collect(),
        }),
    }
}

/// `"#rrggbb"` as sRGB components in 0..=1.
pub(crate) fn hex(s: &str) -> Option<[f64; 3]> {
    let h = s.strip_prefix('#')?;
    if h.len() != 6 || !h.is_ascii() {
        return None;
    }
    let c = |i: usize| {
        u8::from_str_radix(&h[i..i + 2], 16)
            .ok()
            .map(|b| f64::from(b) / 255.0)
    };
    Some([c(0)?, c(2)?, c(4)?])
}

/// A 2D point `[x, y]`.
pub(crate) fn xy(v: &Value) -> Option<DVec2> {
    match v.as_array().map(|l| &l[..]) {
        Some([x, y]) => Some(DVec2::new(x.as_f64()?, y.as_f64()?)),
        _ => None,
    }
}

pub(crate) fn vec3_of(v: &Value) -> Option<DVec3> {
    let l = v.as_array()?;
    match l[..] {
        [ref x, ref y, ref z] => Some(DVec3::new(x.as_f64()?, y.as_f64()?, z.as_f64()?)),
        _ => None,
    }
}

pub(crate) fn frame_of(v: &Value) -> Result<Frame, String> {
    let o = v.as_object().ok_or("a frame is {origin, x, y, z}")?;
    if let Some(k) = o
        .keys()
        .find(|k| !["origin", "x", "y", "z"].contains(&k.as_str()))
    {
        return Err(format!(
            "a frame has no field {k:?}; it takes: origin, x, y, z"
        ));
    }
    let g = |k: &str| {
        v.get(k)
            .and_then(vec3_of)
            .ok_or_else(|| format!("a frame has {k}: [x, y, z]"))
    };
    Ok(Frame {
        origin: g("origin")?,
        x: g("x")?,
        y: g("y")?,
        z: g("z")?,
    })
}

/// One operation's fields.
pub(crate) struct Args<'a> {
    pub(crate) o: &'a Map<String, Value>,
    pub(crate) op: &'a str,
    pub(crate) fonts: &'a Fonts,
    pub(crate) textures: &'a Fonts,
}

impl Args<'_> {
    /// A profile field.
    pub(crate) fn profile(&self, k: &str) -> Result<Profile, ModelError> {
        profile(self.get(k)?, self.fonts).map_err(|e| nest(k, e))
    }

    /// A texture field: the name of an image the caller supplied
    /// ([`Session::texture`]), PNG or JPEG.
    pub(crate) fn texture(&self, k: &str) -> Result<crate::tools::Texture, ModelError> {
        let name = self.str(k)?;
        let bytes = self.textures.get(name).ok_or_else(|| {
            self.bad(
                k,
                &format!(
                    "the name of an image the caller supplies; {name:?} was not supplied (the \
                     CLI and MCP read it as a path relative to the program)"
                ),
            )
        })?;
        let mime = if bytes.starts_with(b"\x89PNG\r\n\x1a\n") {
            "image/png"
        } else if bytes.starts_with(&[0xFF, 0xD8, 0xFF]) {
            "image/jpeg"
        } else {
            return Err(self.bad(k, &format!("a PNG or JPEG image; {name:?} is neither")));
        };
        Ok(crate::tools::Texture {
            name: name.to_string(),
            mime: mime.into(),
            bytes: bytes.clone(),
        })
    }

    /// A path field: points and pieces, or a helix ([`path3`]).
    pub(crate) fn route(&self, k: &str) -> Result<Flat, ModelError> {
        path3(self.get(k)?, k)
    }

    pub(crate) fn bad(&self, field: &str, want: &str) -> ModelError {
        ModelError::BadField(field.into(), want.into())
    }

    /// An error from inside the field `k` (a profile): named under it.
    pub(crate) fn inside(&self, k: &str, e: ModelError) -> ModelError {
        nest(k, e)
    }

    /// A turned solid's axis: `"+Y"` (the default) or `"+Z"`.
    pub(crate) fn axis(&self) -> Result<Axis, ModelError> {
        match self.opt("axis").map(Value::as_str) {
            None | Some(Some("+Y")) => Ok(Axis::Y),
            Some(Some("+Z")) => Ok(Axis::Z),
            Some(_) => Err(self.bad("axis", "\"+Y\" or \"+Z\"")),
        }
    }

    /// A [`View`] of a body: a number, a list of `[y, value]` points
    /// (straight between them), or `{points, smooth}`.
    pub(crate) fn view(&self, k: &str) -> Result<View, ModelError> {
        let want = "a number, [[y, value], …] or {points, smooth}";
        let pairs = |l: &Vec<Value>| -> Result<View, ModelError> {
            l.iter()
                .map(|p| match p.as_array().map(|xy| &xy[..]) {
                    Some([y, v]) => Some(DVec2::new(y.as_f64()?, v.as_f64()?)),
                    _ => None,
                })
                .collect::<Option<Vec<_>>>()
                .map(|points| View {
                    points,
                    ..Default::default()
                })
                .ok_or_else(|| self.bad(k, want))
        };
        match self.get(k)? {
            Value::Number(n) => Ok(View::constant(n.as_f64().unwrap_or(f64::NAN))),
            Value::Array(l) => pairs(l),
            Value::Object(o) => {
                only(o, k, &["points", "smooth"])?;
                let mut v = pairs(
                    o.get("points")
                        .and_then(Value::as_array)
                        .ok_or_else(|| self.bad(&format!("{k}.points"), "[[y, value], …]"))?,
                )?;
                v.smooth = match o.get("smooth") {
                    None | Some(Value::Null) => false,
                    Some(b) => b
                        .as_bool()
                        .ok_or_else(|| self.bad(&format!("{k}.smooth"), "true or false"))?,
                };
                Ok(v)
            }
            _ => Err(self.bad(k, want)),
        }
    }

    pub(crate) fn opt(&self, k: &str) -> Option<&Value> {
        self.o.get(k).filter(|v| !v.is_null())
    }

    pub(crate) fn get(&self, k: &str) -> Result<&Value, ModelError> {
        self.opt(k).ok_or_else(|| self.bad(k, "missing"))
    }

    pub(crate) fn num(&self, k: &str) -> Result<f64, ModelError> {
        self.get(k)?.as_f64().ok_or_else(|| self.bad(k, "a number"))
    }

    pub(crate) fn num_or(&self, k: &str, d: f64) -> Result<f64, ModelError> {
        if self.opt(k).is_some() {
            self.num(k)
        } else {
            Ok(d)
        }
    }

    pub(crate) fn u32_or(&self, k: &str, d: u32) -> Result<u32, ModelError> {
        match self.opt(k) {
            None => Ok(d),
            Some(v) => v
                .as_u64()
                .and_then(|n| u32::try_from(n).ok())
                .ok_or_else(|| self.bad(k, "a whole number")),
        }
    }

    pub(crate) fn bool_or(&self, k: &str, d: bool) -> Result<bool, ModelError> {
        match self.opt(k) {
            None => Ok(d),
            Some(v) => v.as_bool().ok_or_else(|| self.bad(k, "true or false")),
        }
    }

    pub(crate) fn str(&self, k: &str) -> Result<&str, ModelError> {
        self.get(k)?.as_str().ok_or_else(|| self.bad(k, "a string"))
    }

    /// A list of names, or one string of names separated by spaces.
    pub(crate) fn names_or(&self, k: &str) -> Result<Vec<String>, ModelError> {
        match self.opt(k) {
            None => Ok(Vec::new()),
            Some(Value::String(s)) => Ok(s.split_whitespace().map(String::from).collect()),
            Some(Value::Array(l)) => l
                .iter()
                .map(|v| v.as_str().map(String::from))
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| self.bad(k, "a list of names")),
            Some(_) => Err(self.bad(k, "names")),
        }
    }

    pub(crate) fn id(&self, k: &str) -> Result<Id, ModelError> {
        id_of(self.get(k)?).ok_or_else(|| self.bad(k, "a part's id"))
    }

    pub(crate) fn ids(&self, k: &str) -> Result<Vec<Id>, ModelError> {
        match self.get(k)? {
            Value::Array(l) => l
                .iter()
                .map(id_of)
                .collect::<Option<Vec<_>>>()
                .ok_or_else(|| self.bad(k, "a list of ids")),
            v => id_of(v).map(|i| vec![i]).ok_or_else(|| self.bad(k, "ids")),
        }
    }

    pub(crate) fn nums(&self, k: &str) -> Result<Vec<f64>, ModelError> {
        self.get(k)?
            .as_array()
            .and_then(|l| l.iter().map(Value::as_f64).collect::<Option<Vec<_>>>())
            .ok_or_else(|| self.bad(k, "a list of numbers"))
    }

    pub(crate) fn vec3(&self, k: &str) -> Result<DVec3, ModelError> {
        vec3_of(self.get(k)?).ok_or_else(|| self.bad(k, "[x, y, z]"))
    }

    pub(crate) fn points(&self, k: &str) -> Result<Vec<DVec3>, ModelError> {
        self.get(k)?
            .as_array()
            .and_then(|l| l.iter().map(vec3_of).collect::<Option<Vec<_>>>())
            .ok_or_else(|| self.bad(k, "a list of [x, y, z]"))
    }

    pub(crate) fn frame(&self, k: &str) -> Result<Frame, ModelError> {
        frame_of(self.get(k)?).map_err(|w| self.bad(k, &w))
    }

    pub(crate) fn frame_or(&self, k: &str) -> Result<Frame, ModelError> {
        if self.opt(k).is_some() {
            self.frame(k)
        } else {
            Ok(Frame::IDENTITY)
        }
    }

    pub(crate) fn frames(&self, k: &str) -> Result<Vec<Frame>, ModelError> {
        match self.get(k)? {
            Value::Array(l) => l
                .iter()
                .map(frame_of)
                .collect::<Result<Vec<_>, _>>()
                .map_err(|w| self.bad(k, &w)),
            v => frame_of(v).map(|f| vec![f]).map_err(|w| self.bad(k, &w)),
        }
    }

    pub(crate) fn path(&self, k: &str) -> Result<Path, ModelError> {
        let v = self.get(k)?;
        if let Some(o) = v.as_object() {
            only(o, k, &["points", "closed"])?;
        }
        let points = v
            .get("points")
            .and_then(Value::as_array)
            .and_then(|l| l.iter().map(vec3_of).collect::<Option<Vec<_>>>())
            .ok_or_else(|| self.bad(k, "{points, closed}"))?;
        Ok(Path {
            points,
            closed: v.get("closed").and_then(Value::as_bool).unwrap_or(false),
        })
    }
}

pub(crate) fn id_of(v: &Value) -> Option<Id> {
    match v {
        Value::Number(n) => n.as_u64().and_then(|n| u32::try_from(n).ok()).map(Id),
        Value::Object(o) => o.get("id").and_then(id_of),
        _ => None,
    }
}
