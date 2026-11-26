//! Content references are schema positions, not arbitrary `$ogRef` JSON keys.
use og_core::reflect::source::{AuthorSchema, Diagnostic, InputSchema};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};

/// One evaluated content edge. Paths are JSON Pointers relative to the input.
/// Every applicable `x-og-ref-kinds` constrains the target; `None` allows any
/// kind, while an empty set means the schema's constraints conflict.
/// Factory constraints select primary source declarations, never subresources.
#[derive(Debug, PartialEq, Eq)]
pub struct Reference {
    pub path: String,
    pub id: String,
    pub kinds: Option<BTreeSet<String>>,
    pub factories: Option<BTreeSet<String>>,
}

/// Compile once per bound input schema and reuse across content entries.
/// Reachability is computed on the schema graph before visiting any values:
/// a heightfield's millions of numbers are validated, never reference-walked.
pub struct ReferenceSchema<'a> {
    schema: &'a dyn AuthorSchema,
    nodes: Vec<Node>,
    pointers: BTreeMap<String, usize>,
}

#[derive(Default)]
struct Node {
    pointer: String,
    reference: bool,
    reachable: bool,
    kinds: Option<BTreeSet<String>>,
    factories: Option<BTreeSet<String>>,
    same: Vec<usize>,
    alternatives: Vec<usize>,
    properties: BTreeMap<String, usize>,
    patterns: Vec<(InputSchema<Value>, usize)>,
    additional: Option<usize>,
    prefix: Vec<usize>,
    items: Option<usize>,
    contains: Option<usize>,
}

impl Node {
    fn children(&self) -> impl Iterator<Item = usize> + '_ {
        self.same
            .iter()
            .chain(&self.alternatives)
            .chain(self.properties.values())
            .chain(self.patterns.iter().map(|(_, i)| i))
            .chain(self.additional.iter())
            .chain(&self.prefix)
            .chain(self.items.iter())
            .chain(self.contains.iter())
            .copied()
    }
}

impl<'a> ReferenceSchema<'a> {
    pub fn new(schema: &'a dyn AuthorSchema) -> Result<Self, String> {
        let mut plan = Self {
            schema,
            nodes: Vec::new(),
            pointers: BTreeMap::new(),
        };
        plan.node("")?;
        // Walk backwards from tokens and kind constraints through recursive
        // definitions. Each schema node and edge is visited at most once.
        let mut parents = vec![Vec::new(); plan.nodes.len()];
        let mut pending = Vec::new();
        for (i, node) in plan.nodes.iter().enumerate() {
            if node.reachable {
                pending.push(i);
            }
            for child in node.children() {
                parents[child].push(i);
            }
        }
        while let Some(i) = pending.pop() {
            for &parent in &parents[i] {
                if !plan.nodes[parent].reachable {
                    plan.nodes[parent].reachable = true;
                    pending.push(parent);
                }
            }
        }
        Ok(plan)
    }

    /// Validate through the original schema and Rust decoder before extracting
    /// edges. This does not resolve identities, grant module access or cook.
    pub fn collect(&self, value: &Value) -> Result<Vec<Reference>, Diagnostic> {
        self.schema.check(value)?;
        let mut found = BTreeMap::new();
        let mut constraints = BTreeMap::new();
        self.walk(0, value, "", &mut Vec::new(), &mut found, &mut constraints)
            .map_err(|message| Diagnostic {
                path: String::new(),
                message,
            })?;
        // Constraints apply to references inside that evaluated value, including
        // constraints in another applicable allOf arm at the same position.
        for reference in found.values_mut() {
            for (key, target) in [
                ("x-og-ref-kinds", &mut reference.kinds),
                ("x-og-ref-factories", &mut reference.factories),
            ] {
                let mut scope = reference.path.as_str();
                loop {
                    if let Some(values) = constraints.get(&(scope.to_string(), key)) {
                        match &mut *target {
                            Some(current) => current.retain(|k| values.contains(k)),
                            current @ None => *current = Some(values.clone()),
                        }
                    }
                    match scope.rsplit_once('/') {
                        Some((parent, _)) => scope = parent,
                        None => break,
                    }
                }
            }
        }
        Ok(found.into_values().collect())
    }

    fn node(&mut self, pointer: &str) -> Result<usize, String> {
        if let Some(i) = self.pointers.get(pointer) {
            return Ok(*i);
        }
        let i = self.nodes.len();
        self.pointers.insert(pointer.into(), i);
        self.nodes.push(Node::default());
        let schema = self
            .schema
            .document()
            .pointer(pointer)
            .ok_or_else(|| format!("missing reference subschema {pointer}"))?;
        if !pointer.is_empty() && schema.get("$id").is_some() {
            return Err(format!(
                "{pointer}: author subschemas cannot change reference scope"
            ));
        }
        // Keep the supported author schema applicators explicit. Unknown
        // validation keywords need no traversal, but an unknown applicator
        // must never silently lose references.
        for key in [
            "not",
            "if",
            "then",
            "else",
            "dependentSchemas",
            "$dynamicRef",
            "unevaluatedProperties",
            "unevaluatedItems",
        ] {
            if schema.get(key).is_some() {
                return Err(format!("{pointer}: unsupported reference applicator {key}"));
            }
        }
        let reference = schema.get("x-og-content-ref") == Some(&Value::Bool(true));
        let kinds = constraint(schema, pointer, "x-og-ref-kinds")?;
        let factories = constraint(schema, pointer, "x-og-ref-factories")?;
        let mut node = Node {
            pointer: pointer.into(),
            reference,
            reachable: reference || kinds.is_some() || factories.is_some(),
            kinds,
            factories,
            ..Node::default()
        };
        if let Some(target) = schema.get("$ref").and_then(Value::as_str) {
            let target = target
                .strip_prefix('#')
                .ok_or_else(|| format!("{pointer}: reference schema must be local: {target}"))?;
            node.same.push(self.node(target)?);
        }
        for key in ["allOf", "anyOf", "oneOf", "prefixItems"] {
            if let Some(children) = schema.get(key).and_then(Value::as_array) {
                for j in 0..children.len() {
                    let child = self.node(&format!("{pointer}/{key}/{j}"))?;
                    match key {
                        "allOf" => node.same.push(child),
                        "prefixItems" => node.prefix.push(child),
                        _ => node.alternatives.push(child),
                    }
                }
            }
        }
        if let Some(properties) = schema.get("properties").and_then(Value::as_object) {
            for name in properties.keys() {
                let child = self.node(&format!("{pointer}/properties/{}", escape(name)))?;
                node.properties.insert(name.clone(), child);
            }
        }
        if let Some(patterns) = schema.get("patternProperties").and_then(Value::as_object) {
            for pattern in patterns.keys() {
                let child =
                    self.node(&format!("{pointer}/patternProperties/{}", escape(pattern)))?;
                let matcher = InputSchema::from_document(json!({"pattern": pattern}))?;
                node.patterns.push((matcher, child));
            }
        }
        for key in ["additionalProperties", "items", "contains"] {
            if schema.get(key).is_some() {
                let child = self.node(&format!("{pointer}/{key}"))?;
                match key {
                    "additionalProperties" => node.additional = Some(child),
                    "items" => node.items = Some(child),
                    _ => node.contains = Some(child),
                }
            }
        }
        self.nodes[i] = node;
        Ok(i)
    }

    fn walk(
        &self,
        i: usize,
        value: &Value,
        path: &str,
        active: &mut Vec<(usize, *const Value)>,
        found: &mut BTreeMap<String, Reference>,
        constraints: &mut BTreeMap<(String, &'static str), BTreeSet<String>>,
    ) -> Result<(), String> {
        let node = &self.nodes[i];
        let visit = (i, std::ptr::from_ref(value));
        if !node.reachable || active.contains(&visit) {
            return Ok(());
        }
        active.push(visit);
        for (key, values) in [
            ("x-og-ref-kinds", &node.kinds),
            ("x-og-ref-factories", &node.factories),
        ] {
            if let Some(values) = values {
                constraints
                    .entry((path.into(), key))
                    .and_modify(|current| current.retain(|k| values.contains(k)))
                    .or_insert_with(|| values.clone());
            }
        }
        if node.reference {
            let id = value["$ogRef"]
                .as_str()
                .ok_or_else(|| format!("{path}: content reference is not a token"))?;
            found.entry(path.into()).or_insert_with(|| Reference {
                path: path.into(),
                id: id.into(),
                kinds: None,
                factories: None,
            });
        }
        for &child in &node.same {
            self.walk(child, value, path, active, found, constraints)?;
        }
        for &child in &node.alternatives {
            if self.nodes[child].reachable
                && self.schema.matches(&self.nodes[child].pointer, value)?
            {
                self.walk(child, value, path, active, found, constraints)?;
            }
        }
        if let Some(object) = value.as_object() {
            for (name, &child) in &node.properties {
                if let Some(value) = object.get(name) {
                    self.walk(
                        child,
                        value,
                        &format!("{path}/{}", escape(name)),
                        active,
                        found,
                        constraints,
                    )?;
                }
            }
            if !node.patterns.is_empty() || node.additional.is_some_and(|i| self.nodes[i].reachable)
            {
                for (name, value) in object {
                    let mut matched = node.properties.contains_key(name);
                    let key = Value::String(name.clone());
                    let path = format!("{path}/{}", escape(name));
                    for (pattern, child) in &node.patterns {
                        if pattern.validate(&key).is_ok() {
                            matched = true;
                            self.walk(*child, value, &path, active, found, constraints)?;
                        }
                    }
                    if !matched && let Some(child) = node.additional {
                        self.walk(child, value, &path, active, found, constraints)?;
                    }
                }
            }
        }
        if let Some(array) = value.as_array() {
            for (j, value) in array.iter().take(node.prefix.len()).enumerate() {
                self.walk(
                    node.prefix[j],
                    value,
                    &format!("{path}/{j}"),
                    active,
                    found,
                    constraints,
                )?;
            }
            if let Some(child) = node.items.filter(|i| self.nodes[*i].reachable) {
                for (j, value) in array.iter().enumerate().skip(node.prefix.len()) {
                    self.walk(
                        child,
                        value,
                        &format!("{path}/{j}"),
                        active,
                        found,
                        constraints,
                    )?;
                }
            }
            if let Some(child) = node.contains.filter(|i| self.nodes[*i].reachable) {
                for (j, value) in array.iter().enumerate() {
                    if self.schema.matches(&self.nodes[child].pointer, value)? {
                        self.walk(
                            child,
                            value,
                            &format!("{path}/{j}"),
                            active,
                            found,
                            constraints,
                        )?;
                    }
                }
            }
        }
        active.pop();
        Ok(())
    }
}

fn escape(key: &str) -> String {
    key.replace('~', "~0").replace('/', "~1")
}

fn constraint(
    schema: &Value,
    pointer: &str,
    key: &str,
) -> Result<Option<BTreeSet<String>>, String> {
    schema
        .get(key)
        .map(|v| {
            v.as_array()
                .ok_or_else(|| format!("{pointer}: {key} must be an array"))?
                .iter()
                .map(|v| {
                    v.as_str()
                        .map(str::to_owned)
                        .ok_or_else(|| format!("{pointer}: {key} values must be strings"))
                })
                .collect()
        })
        .transpose()
}
