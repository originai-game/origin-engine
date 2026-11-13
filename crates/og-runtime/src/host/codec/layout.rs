//! A record's layout: built from its schema, and its rows encoded and decoded.

use super::*;

impl Layout {
    /// A layout from the wire form of `Fields`: a JSON list of
    /// `{ name, kind, default, options }`, in declaration order.
    pub fn from_fields(schema: &str, kind: RecordKind, fields: &Value) -> Result<Layout, String> {
        let list = fields.as_array().ok_or_else(|| {
            format!("'{schema}': fields cross as a list of {{ name, kind, default, options }}")
        })?;
        let mut out = Vec::with_capacity(list.len());
        for f in list {
            let name = f["name"]
                .as_str()
                .ok_or_else(|| format!("'{schema}': a field without a name"))?
                .to_string();
            let kind_name = f["kind"].as_str().unwrap_or_default();
            let variants = enum_variants(kind_name);
            let k = match variants {
                _ if kind_name == "Ease" => Kind::Ease,
                Some(_) => Kind::Enum,
                None => Kind::parse(kind_name).ok_or_else(|| {
                    format!("'{schema}.{name}': `{kind_name}` is not a column kind; use a column factory such as f32() or Vector3()")
                })?,
            };
            let capacity = match f.get("capacity") {
                None | Some(Value::Null) => None,
                Some(c) => Some(
                    c.as_u64()
                        .and_then(|c| usize::try_from(c).ok())
                        .ok_or_else(|| {
                            format!(
                                "'{schema}.{name}': a capacity is a whole number of bytes, got {c}"
                            )
                        })?,
                ),
            };
            let list = match f.get("list") {
                None | Some(Value::Null) => None,
                Some(c) => Some(
                    c.as_u64()
                        .and_then(|c| usize::try_from(c).ok())
                        .ok_or_else(|| {
                            format!(
                                "'{schema}.{name}': a list's capacity is a whole number of elements, got {c}"
                            )
                        })?,
                ),
            };
            let ty = k
                .field_type(kind == RecordKind::Component, capacity, list)
                .map_err(|e| format!("'{schema}.{name}': {e}"))?;
            let o = &f["options"];
            if list.is_some() && o.get("tunable").and_then(Value::as_bool) == Some(true) {
                return Err(format!(
                    "'{schema}.{name}': a list is never tunable (Tune and the probe's bounds move one number)"
                ));
            }
            let opt_num = |key: &str| o.get(key).and_then(Value::as_f64);
            let opt_str = |key: &str| {
                o.get(key)
                    .and_then(Value::as_str)
                    .filter(|s| !s.is_empty())
                    .map(str::to_string)
            };
            let worldspace = opt_str("worldspace");
            if let Some(w) = &worldspace {
                og_core::Worldspace::of(w, ty).map_err(|e| format!("'{schema}.{name}': {e}"))?;
            }
            out.push(FieldLayout {
                name,
                kind: k,
                ty,
                default: f.get("default").cloned().unwrap_or(Value::Null),
                readonly: false,
                tunable: o.get("tunable").and_then(Value::as_bool).unwrap_or(false),
                min: opt_num("min"),
                max: opt_num("max"),
                step: opt_num("step"),
                unit: opt_str("unit"),
                label: opt_str("label"),
                doc: opt_str("doc").unwrap_or_default(),
                variants: variants.unwrap_or_default(),
                worldspace,
            });
        }
        let layout = Layout {
            schema: schema.to_string(),
            kind,
            fields: out.into(),
            by_name: Default::default(),
            kept: Default::default(),
            kept_rows: Default::default(),
        };
        // Defaults must encode: refuse a bad one at declaration.
        layout.encode(None, None)?;
        Ok(layout)
    }

    /// A layout of an engine-owned record from the generated table.
    pub fn from_decl(decl: &og_api::host::RecordDecl) -> Result<Layout, String> {
        let kind = match decl.kind {
            "component" => RecordKind::Component,
            "resource" => RecordKind::Resource,
            _ => RecordKind::Event,
        };
        let mut fields = Vec::new();
        for f in decl.fields {
            let k = if f.column == "Ease" {
                Kind::Ease
            } else if f.variants.is_empty() {
                Kind::parse(f.column).ok_or_else(|| {
                    format!(
                        "'{}.{}' has type {}, which og-core cannot store yet",
                        decl.schema, f.name, f.column
                    )
                })?
            } else {
                Kind::Enum
            };
            // A text field's capacity is the IDL's (`str.48`), a list's too
            // (`List.Post.8`).
            let capacity = (f.capacity > 0).then_some(f.capacity as usize);
            let list = (f.list > 0).then_some(f.list as usize);
            let ty = k
                .field_type(kind == RecordKind::Component, capacity, list)
                .map_err(|e| format!("'{}.{}': {e}", decl.schema, f.name))?;
            fields.push(FieldLayout {
                name: f.name.to_string(),
                kind: k,
                ty,
                default: serde_json::from_str(f.default).unwrap_or(Value::Null),
                readonly: f.readonly,
                tunable: f.tunable,
                min: f.min,
                max: f.max,
                step: f.step,
                unit: (!f.unit.is_empty()).then(|| f.unit.to_string()),
                label: (!f.label.is_empty()).then(|| f.label.to_string()),
                doc: f.doc.to_string(),
                variants: f.variants.iter().map(|v| v.to_string()).collect(),
                worldspace: (!f.worldspace.is_empty()).then(|| f.worldspace.to_string()),
            });
        }
        Ok(Layout {
            schema: decl.schema.to_string(),
            kind,
            fields: fields.into(),
            by_name: Default::default(),
            kept: Default::default(),
            kept_rows: Default::default(),
        })
    }

    /// The layout of a type another module registered (og-input's
    /// `og.Input` and input events), read from the world's schema.
    pub fn from_schema(schema: &Schema, kind: RecordKind) -> Layout {
        Layout {
            schema: schema.name.clone(),
            kind,
            fields: schema
                .fields
                .iter()
                .map(|f| FieldLayout {
                    name: f.name.clone(),
                    kind: Kind::of(f.ty),
                    ty: f.ty,
                    default: Value::Null,
                    readonly: true,
                    tunable: f.tunable,
                    min: f.min,
                    max: f.max,
                    step: None,
                    unit: f.unit.clone(),
                    label: f.label.clone(),
                    doc: f.doc.clone(),
                    variants: f.variants.clone(),
                    // A module's derive carries no attribute: the IDL's
                    // declaration of the same record is the one place.
                    worldspace: f
                        .worldspace
                        .clone()
                        .or_else(|| idl_worldspace(&schema.name, &f.name)),
                })
                .collect::<Vec<_>>()
                .into(),
            by_name: Default::default(),
            kept: Default::default(),
            kept_rows: Default::default(),
        }
    }

    /// The og-core schema this layout registers as.
    pub fn schema(&self) -> Schema {
        Schema {
            name: self.schema.clone(),
            version: 1,
            doc: String::new(),
            fields: self
                .fields
                .iter()
                .map(|f| {
                    let mut s = FieldSchema::new(f.name.clone(), f.ty);
                    s.doc = f.doc.clone();
                    s.label = f.label.clone();
                    s.unit = f.unit.clone();
                    s.min = f.min;
                    s.max = f.max;
                    s.tunable = f.tunable;
                    if f.kind != Kind::Ease {
                        s.variants = f.variants.clone();
                    }
                    s.worldspace = f.worldspace.clone();
                    s.default = default_lanes(f);
                    s
                })
                .collect(),
        }
    }

    pub fn size(&self) -> usize {
        self.fields.iter().map(|f| f.ty.size()).sum()
    }

    pub fn field(&self, name: &str) -> Option<(usize, &FieldLayout)> {
        self.fields.iter().enumerate().find(|(_, f)| f.name == name)
    }

    /// The field named `key` and its byte offset in packed form, by binary
    /// search over the names (a per-frame reader resolves its fields once
    /// with [`Slot::of`] instead).
    pub fn slot(&self, key: &str) -> Option<(&FieldLayout, usize)> {
        let (i, at) = self.index_of(key)?;
        Some((&self.fields[i], at))
    }

    /// The index of the field named `key` and its byte offset in packed form.
    pub(super) fn index_of(&self, key: &str) -> Option<(usize, usize)> {
        let index = self.by_name.get_or_init(|| {
            let mut at = 0;
            let mut index: Vec<(usize, usize)> = self
                .fields
                .iter()
                .enumerate()
                .map(|(i, f)| {
                    let slot = (i, at);
                    at += f.ty.size();
                    slot
                })
                .collect();
            og_core::sort::sort_by(&mut index, |a, b| {
                self.fields[a.0].name.cmp(&self.fields[b.0].name)
            });
            index
        });
        let k = index
            .binary_search_by(|(i, _)| self.fields[*i].name.as_str().cmp(key))
            .ok()?;
        Some(index[k])
    }

    /// Byte offset of field `i` in packed form.
    pub fn offset(&self, i: usize) -> usize {
        self.fields[..i].iter().map(|f| f.ty.size()).sum()
    }

    /// Packed bytes: each field from `value` when it names it, else from
    /// `base` (the current value) or the field's default. Unknown keys are
    /// refused, so a typo is not silently dropped.
    pub fn encode(&self, base: Option<&[u8]>, value: Option<&Value>) -> Result<Vec<u8>, String> {
        let obj = match value {
            None | Some(Value::Null) => None,
            Some(Value::Object(o)) => Some(o),
            Some(other) => {
                return Err(format!(
                    "a '{}' value is an object, got {other}",
                    self.schema
                ));
            }
        };
        if let Some(o) = obj
            && let Some(k) = o.keys().find(|k| self.field(k).is_none())
        {
            return Err(format!("'{}' has no field '{k}'", self.schema));
        }
        let mut out = Vec::with_capacity(self.size());
        for (i, f) in self.fields.iter().enumerate() {
            match obj.and_then(|o| o.get(&f.name)) {
                Some(v) => encode_field(f, v, &mut out)
                    .map_err(|e| format!("'{}.{}': {e}", self.schema, f.name))?,
                None => match base {
                    Some(b) => {
                        let at = self.offset(i);
                        out.extend_from_slice(&b[at..at + f.ty.size()]);
                    }
                    None => encode_field(f, &f.default, &mut out)
                        .map_err(|e| format!("'{}.{}' default: {e}", self.schema, f.name))?,
                },
            }
        }
        Ok(out)
    }

    /// Check insert values against reflection, without clamping any lane.
    pub fn check_ranges(&self, value: &Value) -> Result<(), String> {
        fn check(f: &FieldLayout, v: &Value) -> Result<(), String> {
            match v {
                Value::Number(n) => {
                    let n = n.as_f64().expect("JSON number");
                    if let Some(min) = f.min.filter(|min| n < *min) {
                        return Err(format!("{n} is below the minimum {min}"));
                    }
                    if let Some(max) = f.max.filter(|max| n > *max) {
                        return Err(format!("{n} is above the maximum {max}"));
                    }
                }
                Value::Array(values) => {
                    for v in values {
                        check(f, v)?;
                    }
                }
                Value::Object(values) => {
                    for v in values.values() {
                        check(f, v)?;
                    }
                }
                _ => {}
            }
            Ok(())
        }
        for f in self
            .fields
            .iter()
            .filter(|f| f.min.is_some() || f.max.is_some())
        {
            check(f, value.get(&f.name).unwrap_or(&f.default))
                .map_err(|e| format!("'{}.{}': {e}", self.schema, f.name))?;
        }
        Ok(())
    }

    /// Packed bytes as the row type the IDL generates for the record
    /// (`wire::record`): [`Layout::decode`] read by its fields' rules, so a
    /// field the runtime reads is the IDL's by name, type and default.
    pub fn read<T: crate::host::wire::FromJson>(&self, bytes: &[u8]) -> Result<T, String> {
        T::from_json(&self.decode(bytes)).map_err(|e| format!("'{}': {e}", self.schema))
    }

    /// The value of packed bytes as JSON.
    pub fn decode(&self, bytes: &[u8]) -> Value {
        let mut map = Map::new();
        let mut at = 0;
        for f in self.fields.iter() {
            let size = f.ty.size();
            map.insert(f.name.clone(), decode_field(f, &bytes[at..at + size]));
            at += size;
        }
        Value::Object(map)
    }

    /// [`Layout::decode`] of a record read every frame, a resource: the
    /// last bytes decoded and their JSON are kept, so a record whose bytes
    /// did not change is not decoded again (the camera's, the render
    /// settings' and the post effects' decodes, their maps built and
    /// dropped, were about a tenth of a frame's wasm on the retired WebGL2
    /// tier: the former test sample
    pub fn decode_kept(&self, bytes: Vec<u8>) -> std::sync::Arc<Value> {
        let mut kept = self
            .kept
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        if let Some((b, v)) = kept.as_ref()
            && *b == bytes
        {
            return v.clone();
        }
        let v = std::sync::Arc::new(self.decode(&bytes));
        *kept = Some((bytes, v.clone()));
        v
    }

    /// One field of packed bytes as JSON.
    pub fn decode_one(&self, bytes: &[u8], i: usize) -> Value {
        let at = self.offset(i);
        let f = &self.fields[i];
        decode_field(f, &bytes[at..at + f.ty.size()])
    }
}
