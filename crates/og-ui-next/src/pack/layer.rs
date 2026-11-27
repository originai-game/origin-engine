//! Native style layers, shared by the cook and both runtime hosts.
use super::*;

#[derive(Clone, Debug, Default, serde::Serialize, serde::Deserialize)]
pub struct Layer {
    pub scheme: Option<Scheme>,
    /// Seed overrides, by name, per scheme.
    pub seeds: Vec<(String, [String; 2])>,
    #[serde(with = "super::cooked::themes")]
    pub theme: [Map<String, Value>; 2],
    #[serde(with = "super::cooked::themes")]
    pub contrast: [Map<String, Value>; 2],
    pub variants: BTreeMap<String, String>,
    /// Canonical face IDs, in fallback order. Metadata belongs to each face.
    pub fonts: Vec<String>,
    pub icons: BTreeMap<String, String>,
    pub icon_set: Option<String>,
    pub images: [BTreeMap<String, NineSlice>; 2],
    /// Global events under "", slot-local events under Component.slot[.variant].
    pub sounds: BTreeMap<String, Sounds>,
    pub haptics: BTreeMap<String, Haptics>,
}

impl Layer {
    /// Only engine-generated design data uses JSON; it is not an author input
    /// or a runtime asset fallback. gen-pack and gen-tokens stay drift-checked.
    pub fn default_pack() -> Layer {
        let fields: input::PackInput =
            serde_json::from_str(design::DEFAULT_PACK).expect("generated design fields");
        Self::from_input(fields, Clone::clone).expect("valid generated design")
    }

    pub fn role_families(&self) -> [Vec<String>; 4] {
        let list = |key: &str| -> Vec<String> {
            self.theme[0]
                .get("fontFamily")
                .and_then(|f| f.get(key))
                .and_then(Value::as_str)
                .map(|l| {
                    l.split(',')
                        .map(|n| n.trim().trim_matches(['"', '\'']).to_owned())
                        .filter(|n| !n.is_empty())
                        .collect()
                })
                .unwrap_or_default()
        };
        ["sans", "display", "mono", "rounded"].map(list)
    }

    pub fn font_family(&self, key: &str) -> Option<String> {
        let list = self.theme[0].get("fontFamily")?.get(key)?.as_str()?;
        (!list.trim().is_empty()).then(|| list.trim().to_owned())
    }

    pub fn assets(&self) -> Vec<&str> {
        let mut out: Vec<&str> = self.fonts.iter().map(String::as_str).collect();
        out.extend(self.icons.values().map(String::as_str));
        out.extend(self.icon_set.as_deref());
        out.extend(
            self.images
                .iter()
                .flat_map(|m| m.values().map(|n| n.src.as_str())),
        );
        for sounds in self.sounds.values() {
            out.extend(
                sounds
                    .values()
                    .flat_map(|s| s.clips.iter().map(String::as_str)),
            );
        }
        out.sort_unstable();
        out.dedup();
        out
    }

    /// Bind canonical references to the actual reported producer outputs.
    pub fn bind_assets(&mut self, bind: impl Fn(&str) -> Option<String>) {
        let at = |p: &mut String| {
            if let Some(id) = bind(p) {
                *p = id;
            }
        };
        self.fonts.iter_mut().for_each(at);
        self.icons.values_mut().for_each(at);
        self.icon_set.iter_mut().for_each(at);
        for m in &mut self.images {
            m.values_mut().for_each(|n| at(&mut n.src));
        }
        for sounds in self.sounds.values_mut() {
            sounds
                .values_mut()
                .flat_map(|s| s.clips.iter_mut())
                .for_each(at);
        }
    }
}

pub(super) fn merge(have: &mut Map<String, Value>, over: &Map<String, Value>) {
    for (section, v) in over {
        match (have.get_mut(section), v) {
            (Some(Value::Object(h)), Value::Object(o)) => h.extend(o.clone()),
            (_, v) => {
                have.insert(section.clone(), v.clone());
            }
        }
    }
}

pub(super) fn pair(m: &Map<String, Value>) -> Option<(&Value, &Value)> {
    (m.len() == 2)
        .then(|| Some((m.get("light")?, m.get("dark")?)))
        .flatten()
}

/// Project the adaptive typed values to the scheme-specific NSH table.
pub(super) fn split(v: &Value, scheme: Scheme, at: &str) -> Result<Value, String> {
    let Value::Object(m) = v else {
        return Ok(v.clone());
    };
    if let Some((light, dark)) = pair(m) {
        return split(
            if scheme == Scheme::Dark { dark } else { light },
            scheme,
            at,
        );
    }
    if m.contains_key("light") || m.contains_key("dark") {
        return Err(format!("{at}: a scheme pair has exactly light and dark"));
    }
    m.iter()
        .map(|(k, v)| Ok((k.clone(), split(v, scheme, &format!("{at}.{k}"))?)))
        .collect::<Result<Map<_, _>, _>>()
        .map(Value::Object)
}

pub(super) fn variant_key(k: &str) -> bool {
    let parts: Vec<&str> = k.split('.').collect();
    let name = |p: &str| {
        !p.is_empty()
            && p.starts_with(|c: char| c.is_ascii_lowercase())
            && p.chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    };
    matches!(parts.len(), 2 | 3)
        && parts[0].starts_with(|c: char| c.is_ascii_uppercase())
        && parts[0].chars().all(|c| c.is_ascii_alphanumeric())
        && parts[1..].iter().all(|p| name(p))
}

pub(super) fn kebab(name: &str) -> bool {
    !name.is_empty()
        && name.split('-').all(|w| {
            !w.is_empty()
                && w.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        })
}
