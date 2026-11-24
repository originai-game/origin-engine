//! Scene write-back (`save_scene`, `docs/design.md` §3.4): a scene's `.ogt`
//! text rewritten line by line from the values a world holds now.
//!
//! The text is the source of truth for everything but values: every line
//! keeps its place, its indentation and its trailing `# comment`; a field
//! line is rewritten only when the value it reads as (after the f32
//! rounding the cook applies) differs from the current one. A tuned field
//! the text does not list gets a line before its component's closing `}`
//! (or a new component block before its entity's). The result therefore
//! cooks to the world it was written from, and writing it again changes
//! nothing.
//!
//! Resources are `resource <Name> { ... }` blocks at the top level (the
//! cook's `resource` entry): their field lines are rewritten as a
//! component's are, under the owner [`RESOURCE`]; a tuned resource no
//! block sets gets one at the end of the authored text where
//! [`Plan::add_resources`] says so (the boot scene's).
//!
//! Entities the game spawned at run time go under a marked section at the
//! end ([`SPAWNED_MARK`], [`Plan::spawned`]): regenerated whole from the
//! world on every save, so a reader tells them from the authored entities
//! above it, and cooked like any other entity.
//!
//! Prefab instances (`docs/design.md` §2.5) stay instances
//! ([`rewrite_instances`]): an instance's block holds only what differs
//! from its prefab, so a field of a placed entity is written as an
//! override (in the instance's own block for its root, in an `override
//! "<node>"` block for a descendant, which is named `<root>/<node>` in the
//! world) and only when it differs from the prefab's value; one back at
//! the prefab's value has its line removed, and a block left empty goes
//! with it. A prefab's own text is rewritten by node path
//! ([`Instances::paths`]): its root is `""`, a child `lid`, a grandchild
//! `lid/hinge`.

/// A field's current value, as the scene text would give it.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Numbers, one per lane (a vector's `x y z`, a scalar's one, a list's
    /// elements: none for an empty list).
    Lanes(Vec<f64>),
    /// A flag: `true`/`false`.
    Bool(bool),
    /// A string field: one quoted string.
    Text(String),
}

/// One tuned field: `(entity, component, field)` as the scene names them;
/// a resource's is `(RESOURCE, resource, field)`.
pub type Tuned = (String, String, String);

/// The owner of a resource's fields in [`Tuned`] and in `current`'s first
/// argument: no entity.
pub const RESOURCE: &str = "";

/// The line that opens the section of entities spawned at run time; what
/// follows it to the end of the text is that section.
pub const SPAWNED_MARK: &str = "# ── spawned at run time";

/// The whole line [`SPAWNED_MARK`] opens.
const SPAWNED_LINE: &str = "# ── spawned at run time ── entities the game made while it ran, written by save_scene from the world; not authored: edit above this line";

/// An entity the game spawned at run time, as the section writes it: its
/// components with every field the text can give (handles are the
/// engine's: a component holding one is left out with a note), and its
/// children spawned at run time.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SpawnedEntity {
    pub name: String,
    /// A model it instantiates (`model "<asset id>"`).
    pub model: Option<String>,
    pub components: Vec<(String, Vec<(String, FieldValue)>)>,
    /// Comment lines inside its block (what was not written, and why).
    pub notes: Vec<String>,
    pub children: Vec<SpawnedEntity>,
}

/// New component blocks: per component, its tuned fields (index, name).
type Blocks = Vec<(String, Vec<(usize, String)>)>;

/// A tuned field placed in a new block: its index in the tuned list, its
/// name and value.
type Placed = (usize, String, FieldValue);

/// What [`rewrite_with`] writes beside the authored lines.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Plan {
    pub tuned: Vec<Tuned>,
    /// Add a `resource` block for each tuned resource the text does not
    /// set (one scene gets them: the boot scene's).
    pub add_resources: bool,
    /// Regenerate the spawned section from these (`None` keeps the text's
    /// section, if any, as authored text).
    pub spawned: Option<Vec<SpawnedEntity>>,
}

/// What [`rewrite`] made.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Rewrite {
    pub text: String,
    /// The 1-based lines of the new text that changed or were added.
    pub lines: Vec<usize>,
    /// Tuned fields that were not written, each with the reason.
    pub skipped: Vec<String>,
    /// Lines removed: overrides back at their prefab's value, and the
    /// blocks they left empty.
    pub removed: usize,
}

impl Rewrite {
    /// Whether the text changed at all.
    pub fn changed(&self) -> bool {
        !self.lines.is_empty() || self.removed > 0
    }
}

/// The prefab's value of a placed entity's field, for [`rewrite_instances`]:
/// `None` for an entity not placed from a prefab (its lines are written
/// as any entity's), `Some(None)` for a placed one whose prefab node lacks
/// the field (an override is always written), `Some(Some(v))` the
/// prefab's value (an override equal to it is removed).
pub type BaseFn<'a> = &'a dyn Fn(&str, &str, &str) -> Option<Option<FieldValue>>;

/// How [`rewrite_instances`] reads a text's instances.
pub struct Instances<'a> {
    pub base: BaseFn<'a>,
    /// Entities are named by node path, as in a prefab's own text (the
    /// top-level entity `""`, a child `lid`), instead of by their names.
    pub paths: bool,
}

/// No prefab anywhere: every entity written as it is.
fn no_base(_: &str, _: &str, _: &str) -> Option<Option<FieldValue>> {
    None
}

enum Frame {
    Entity {
        name: String,
        indent: String,
        /// An instance's root (a `prefab` line in it, or a variant's
        /// root): descendants' overrides it lacks go in its block.
        instance: bool,
        /// An `override "<node>"` block: where in the text it began.
        start: Option<usize>,
        /// Lines removed inside it.
        dropped: bool,
    },
    Component {
        entity: String,
        name: String,
        indent: String,
        start: usize,
        kept: usize,
        dropped: bool,
    },
    Other,
}

/// `parent/child`, or `child` under the root (`""`).
fn join(parent: &str, child: &str) -> String {
    if parent.is_empty() {
        child.to_string()
    } else if child.is_empty() {
        parent.to_string()
    } else {
        format!("{parent}/{child}")
    }
}

/// Whether two values read the same once cooked (f32 lanes).
fn equal(a: &FieldValue, b: &FieldValue) -> bool {
    match (a, b) {
        (FieldValue::Lanes(x), FieldValue::Lanes(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| *p as f32 == *q as f32)
        }
        (FieldValue::Bool(x), FieldValue::Bool(y)) => x == y,
        (FieldValue::Bool(x), FieldValue::Lanes(y))
        | (FieldValue::Lanes(y), FieldValue::Bool(x)) => {
            y.len() == 1 && y[0] as f32 == if *x { 1.0 } else { 0.0 }
        }
        (FieldValue::Text(x), FieldValue::Text(y)) => x == y,
        _ => false,
    }
}

/// Splits a line into its code and its comment (from the first `#` outside
/// a quoted string, with the whitespace before it).
fn split_comment(line: &str) -> (&str, &str) {
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in line.char_indices() {
        match c {
            _ if escaped => escaped = false,
            '\\' if quoted => escaped = true,
            '"' => quoted = !quoted,
            '#' if !quoted => {
                let code = line[..i].trim_end();
                return (code, &line[code.len()..]);
            }
            _ => {}
        }
    }
    (line, "")
}

/// The tokens of a line's code: words, numbers and quoted strings (with
/// their quotes).
fn tokens(code: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let bytes = code.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i].is_ascii_whitespace() {
            i += 1;
            continue;
        }
        let start = i;
        if bytes[i] == b'"' {
            i += 1;
            while i < bytes.len() && bytes[i] != b'"' {
                i += if bytes[i] == b'\\' { 2 } else { 1 };
            }
            i = (i + 1).min(bytes.len());
        } else {
            while i < bytes.len() && !bytes[i].is_ascii_whitespace() {
                i += 1;
            }
        }
        out.push(&code[start..i]);
    }
    out
}

fn unquote(token: &str) -> Option<String> {
    let inner = token.strip_prefix('"')?.strip_suffix('"')?;
    let mut s = String::new();
    let mut chars = inner.chars();
    while let Some(c) = chars.next() {
        if c == '\\' {
            match chars.next()? {
                'n' => s.push('\n'),
                't' => s.push('\t'),
                c => s.push(c),
            }
        } else {
            s.push(c);
        }
    }
    Some(s)
}

fn quote(s: &str) -> String {
    let mut out = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

/// A number as the shortest text that cooks back to the same f32 (`0` for
/// either zero).
pub fn number(x: f64) -> String {
    let f = x as f32;
    if f == 0.0 { "0".into() } else { format!("{f}") }
}

/// A token's value as one lane: a number, or `true`/`false` as 1/0.
fn lane(token: &str) -> Option<f32> {
    match token {
        "true" => Some(1.0),
        "false" => Some(0.0),
        t => t.parse::<f64>().ok().map(|x| x as f32),
    }
}

/// The value tokens of `value`, each after a space, with the old tokens'
/// spelling kept where they read the same (a `true` stays a word); nothing
/// for an empty list, whose line is its key alone.
fn spell(value: &FieldValue, old: &[&str]) -> String {
    let tokens = spell_tokens(value, old);
    if tokens.is_empty() {
        tokens
    } else {
        format!(" {tokens}")
    }
}

fn spell_tokens(value: &FieldValue, old: &[&str]) -> String {
    match value {
        FieldValue::Text(s) => quote(s),
        FieldValue::Bool(b) => b.to_string(),
        FieldValue::Lanes(lanes) => lanes
            .iter()
            .enumerate()
            .map(|(i, x)| match old.get(i) {
                Some(&t) if lane(t) == Some(*x as f32) => t.to_string(),
                Some(&"true" | &"false") if *x == 0.0 || *x == 1.0 => (*x == 1.0).to_string(),
                _ => number(*x),
            })
            .collect::<Vec<_>>()
            .join(" "),
    }
}

/// Whether the old tokens already read as `value`.
fn same(value: &FieldValue, old: &[&str]) -> bool {
    match value {
        FieldValue::Text(s) => old.len() == 1 && unquote(old[0]).as_deref() == Some(s),
        FieldValue::Bool(b) => old.len() == 1 && lane(old[0]) == Some(if *b { 1.0 } else { 0.0 }),
        FieldValue::Lanes(lanes) => {
            old.len() == lanes.len()
                && old
                    .iter()
                    .zip(lanes)
                    .all(|(t, x)| lane(t) == Some(*x as f32))
        }
    }
}

/// Rewrites a scene's text from `current(entity, component, field)` (the
/// value now, `None` for one the world does not have: the line is kept),
/// adding a line for each of `tuned` the text does not list. Only the body
/// (after `---`) is read; lines it cannot read as fields are kept as they
/// are.
pub fn rewrite(
    text: &str,
    current: &dyn Fn(&str, &str, &str) -> Option<FieldValue>,
    tuned: &[Tuned],
) -> Rewrite {
    rewrite_with(
        text,
        current,
        &Plan {
            tuned: tuned.to_vec(),
            ..Plan::default()
        },
    )
}

/// The names of the entities in a text's spawned section ([`SPAWNED_MARK`]),
/// at every depth, in order.
pub fn spawned_names(text: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut inside = false;
    for line in text.split('\n') {
        if line.trim_start().starts_with(SPAWNED_MARK) {
            inside = true;
            continue;
        }
        if !inside {
            continue;
        }
        let (code, _) = split_comment(line);
        if let Some(head) = code.trim().strip_suffix('{')
            && let ["entity", name] = tokens(head).as_slice()
            && let Some(name) = unquote(name)
        {
            out.push(name);
        }
    }
    out
}

/// [`rewrite`] with resources added and the spawned section regenerated as
/// `plan` says.
pub fn rewrite_with(
    text: &str,
    current: &dyn Fn(&str, &str, &str) -> Option<FieldValue>,
    plan: &Plan,
) -> Rewrite {
    rewrite_instances(
        text,
        current,
        plan,
        &Instances {
            base: &no_base,
            paths: false,
        },
    )
}

/// [`rewrite_with`] keeping prefab instances as instances: a placed
/// entity's field is an override, written only when it differs from the
/// prefab's value (`instances.base`), and removed when it is back at it.
pub fn rewrite_instances(
    text: &str,
    current: &dyn Fn(&str, &str, &str) -> Option<FieldValue>,
    plan: &Plan,
    instances: &Instances,
) -> Rewrite {
    let all: Vec<&str> = text.split('\n').collect();
    let mark = all
        .iter()
        .position(|l| l.trim_start().starts_with(SPAWNED_MARK));
    let (head, old_section): (&[&str], &[&str]) = match (&plan.spawned, mark) {
        (Some(_), Some(m)) => (&all[..m], &all[m..]),
        _ => (&all[..], &[]),
    };
    let mut r = rewrite_lines(head, current, plan, instances);
    let Some(spawned) = &plan.spawned else {
        return r;
    };
    // The section, regenerated: its old lines dropped, the new ones after
    // the authored text, each line that differs from the old one's counted.
    let mut section: Vec<String> = Vec::new();
    if !spawned.is_empty() {
        section.push(SPAWNED_LINE.to_string());
        for e in spawned {
            spawned_block(e, "", &mut section);
        }
    }
    // The authored text without its trailing empty line, then the section
    // and a final newline.
    let mut lines: Vec<String> = r.text.split('\n').map(str::to_string).collect();
    let ended = lines.last().is_some_and(String::is_empty);
    if !section.is_empty() || mark.is_some() {
        if ended {
            lines.pop();
        }
        let at = lines.len();
        let old = match old_section.last() {
            Some(&"") => &old_section[..old_section.len() - 1],
            _ => old_section,
        };
        for (k, l) in section.iter().enumerate() {
            if old.get(k) != Some(&l.as_str()) {
                r.lines.push(at + k + 1);
            }
        }
        lines.extend(section);
        lines.push(String::new());
    }
    r.text = lines.join("\n");
    r
}

/// One spawned entity's block at `indent`.
fn spawned_block(e: &SpawnedEntity, indent: &str, out: &mut Vec<String>) {
    out.push(format!("{indent}entity {} {{", quote(&e.name)));
    if let Some(m) = &e.model {
        out.push(format!("{indent}  model {}", quote(m)));
    }
    for (c, fields) in &e.components {
        out.push(format!("{indent}  {c} {{"));
        for (f, v) in fields {
            out.push(format!("{indent}    {f}{}", spell(v, &[])));
        }
        out.push(format!("{indent}  }}"));
    }
    for n in &e.notes {
        out.push(format!("{indent}  # {n}"));
    }
    let deeper = format!("{indent}  ");
    for c in &e.children {
        spawned_block(c, &deeper, out);
    }
    out.push(format!("{indent}}}"));
}

fn rewrite_lines(
    lines: &[&str],
    current: &dyn Fn(&str, &str, &str) -> Option<FieldValue>,
    plan: &Plan,
    instances: &Instances,
) -> Rewrite {
    let tuned = &plan.tuned;
    let base = instances.base;
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + tuned.len());
    let mut changed: Vec<usize> = Vec::new();
    let mut removed = 0usize;
    let mut stack: Vec<Frame> = Vec::new();
    // Per (entity, component): the fields the text lists.
    let mut listed: std::collections::BTreeSet<(String, String, String)> = Default::default();
    let mut components: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut entities: std::collections::BTreeSet<String> = Default::default();
    let mut placed: std::collections::BTreeSet<usize> = Default::default();
    let mut body = !lines.iter().any(|l| l.trim() == "---");
    // A prefab's text with a `base` line: its entity is an instance.
    let mut variant = false;
    let indent_of = |l: &str| l[..l.len() - l.trim_start().len()].to_string();
    let push = |out: &mut Vec<String>, changed: &mut Vec<usize>, line: String, new: bool| {
        out.push(line);
        if new {
            changed.push(out.len());
        }
    };
    // Whether a value is an override the instance no longer needs.
    let at_base = |e: &str, c: &str, f: &str, v: &FieldValue| -> bool {
        matches!(base(e, c, f), Some(Some(b)) if equal(&b, v))
    };
    for raw in lines {
        let line = *raw;
        if !body {
            body = line.trim() == "---";
            out.push(line.to_string());
            continue;
        }
        let (code, comment) = split_comment(line);
        let trimmed = code.trim();
        let indent = indent_of(line);
        if stack.is_empty() && tokens(trimmed).first() == Some(&"base") {
            variant = true;
        }
        if let Some(Frame::Entity { instance, .. }) = stack.last_mut()
            && tokens(trimmed).first() == Some(&"prefab")
        {
            *instance = true;
        }
        if let Some(head) = trimmed.strip_suffix('{') {
            let t = tokens(head);
            let frame = match (stack.last(), t.as_slice()) {
                (None | Some(Frame::Entity { .. }), ["entity", name]) => match unquote(name) {
                    Some(name) => {
                        let name = match (instances.paths, stack.last()) {
                            (true, Some(Frame::Entity { name: parent, .. })) => join(parent, &name),
                            (true, _) => String::new(),
                            (false, _) => name,
                        };
                        entities.insert(name.clone());
                        Frame::Entity {
                            instance: instances.paths && variant && stack.is_empty(),
                            name,
                            indent: indent.clone(),
                            start: None,
                            dropped: false,
                        }
                    }
                    None => Frame::Other,
                },
                (Some(Frame::Entity { name: parent, .. }), ["override", node]) => {
                    match unquote(node) {
                        Some(node) => {
                            let name = join(parent, &node);
                            entities.insert(name.clone());
                            Frame::Entity {
                                name,
                                indent: indent.clone(),
                                instance: false,
                                start: Some(out.len()),
                                dropped: false,
                            }
                        }
                        None => Frame::Other,
                    }
                }
                (None, ["resource", r]) => {
                    let r = unquote(r).unwrap_or_else(|| r.to_string());
                    components.insert((RESOURCE.to_string(), r.clone()));
                    Frame::Component {
                        entity: RESOURCE.to_string(),
                        name: r,
                        indent: indent.clone(),
                        start: out.len(),
                        kept: 0,
                        dropped: false,
                    }
                }
                (Some(Frame::Entity { name, .. }), [comp]) => {
                    components.insert((name.clone(), comp.to_string()));
                    Frame::Component {
                        entity: name.clone(),
                        name: comp.to_string(),
                        indent: indent.clone(),
                        start: out.len(),
                        kept: 0,
                        dropped: false,
                    }
                }
                _ => Frame::Other,
            };
            stack.push(frame);
            out.push(line.to_string());
            continue;
        }
        if trimmed == "}" {
            match stack.pop() {
                Some(Frame::Component {
                    entity,
                    name,
                    indent: ci,
                    start,
                    mut kept,
                    dropped,
                }) => {
                    // Tuned fields of this component the text lacks.
                    for (i, (e, c, f)) in tuned.iter().enumerate() {
                        if *e == entity
                            && *c == name
                            && !listed.contains(&(e.clone(), c.clone(), f.clone()))
                            && let Some(v) = current(e, c, f)
                        {
                            listed.insert((e.clone(), c.clone(), f.clone()));
                            placed.insert(i);
                            if at_base(e, c, f, &v) {
                                continue;
                            }
                            let l = format!("{ci}  {f}{}", spell(&v, &[]));
                            push(&mut out, &mut changed, l, true);
                            kept += 1;
                        }
                    }
                    // An override block its removals emptied goes too.
                    if dropped && kept == 0 {
                        removed += out.len() - start + 1;
                        out.truncate(start);
                        changed.retain(|&l| l <= start);
                        if let Some(Frame::Entity { dropped, .. }) = stack.last_mut() {
                            *dropped = true;
                        }
                        continue;
                    }
                }
                Some(Frame::Entity {
                    name,
                    indent: ei,
                    instance,
                    start,
                    dropped,
                }) => {
                    // Tuned fields of components the entity lacks: one
                    // block per component, in the order first tuned.
                    let mut blocks: Vec<(String, Vec<(usize, String)>)> = Vec::new();
                    for (i, (e, c, f)) in tuned.iter().enumerate() {
                        if *e != name
                            || components.contains(&(e.clone(), c.clone()))
                            || placed.contains(&i)
                        {
                            continue;
                        }
                        match blocks.iter_mut().find(|b| b.0 == *c) {
                            Some(b) => b.1.push((i, f.clone())),
                            None => blocks.push((c.clone(), vec![(i, f.clone())])),
                        }
                    }
                    for (c, fields) in blocks {
                        let values: Vec<(usize, String, FieldValue)> = fields
                            .into_iter()
                            .filter_map(|(i, f)| current(&name, &c, &f).map(|v| (i, f, v)))
                            .filter(|(i, f, v)| {
                                if at_base(&name, &c, f, v) {
                                    placed.insert(*i);
                                    false
                                } else {
                                    true
                                }
                            })
                            .collect();
                        if values.is_empty() {
                            continue;
                        }
                        push(&mut out, &mut changed, format!("{ei}  {c} {{"), true);
                        for (i, f, v) in values {
                            push(
                                &mut out,
                                &mut changed,
                                format!("{ei}    {f}{}", spell(&v, &[])),
                                true,
                            );
                            placed.insert(i);
                        }
                        push(&mut out, &mut changed, format!("{ei}  }}"), true);
                        components.insert((name.clone(), c));
                    }
                    // An instance's descendants without a block of their
                    // own: an override block each, for what differs.
                    if instance {
                        let mut nodes: Vec<(String, Blocks)> = Vec::new();
                        for (i, (e, c, f)) in tuned.iter().enumerate() {
                            let under = if name.is_empty() {
                                !e.is_empty()
                            } else {
                                e.starts_with(&format!("{name}/"))
                            };
                            if !under
                                || entities.contains(e)
                                || placed.contains(&i)
                                || base(e, c, f).is_none()
                            {
                                continue;
                            }
                            let node = if name.is_empty() {
                                e.clone()
                            } else {
                                e[name.len() + 1..].to_string()
                            };
                            let at = match nodes.iter().position(|n| n.0 == node) {
                                Some(k) => k,
                                None => {
                                    nodes.push((node, Vec::new()));
                                    nodes.len() - 1
                                }
                            };
                            let comps = &mut nodes[at].1;
                            match comps.iter_mut().find(|b| b.0 == *c) {
                                Some(b) => b.1.push((i, f.clone())),
                                None => comps.push((c.clone(), vec![(i, f.clone())])),
                            }
                        }
                        for (node, comps) in nodes {
                            let e = join(&name, &node);
                            let mut lines = Vec::new();
                            for (c, fields) in comps {
                                let mut block = Vec::new();
                                for (i, f) in fields {
                                    placed.insert(i);
                                    let Some(v) = current(&e, &c, &f) else {
                                        continue;
                                    };
                                    if !at_base(&e, &c, &f, &v) {
                                        block.push(format!("{ei}      {f}{}", spell(&v, &[])));
                                    }
                                }
                                if !block.is_empty() {
                                    lines.push(format!("{ei}    {c} {{"));
                                    lines.extend(block);
                                    lines.push(format!("{ei}    }}"));
                                }
                            }
                            if lines.is_empty() {
                                continue;
                            }
                            push(
                                &mut out,
                                &mut changed,
                                format!("{ei}  override {} {{", quote(&node)),
                                true,
                            );
                            for l in lines {
                                push(&mut out, &mut changed, l, true);
                            }
                            push(&mut out, &mut changed, format!("{ei}  }}"), true);
                            entities.insert(e);
                        }
                    }
                    // An override block its removals emptied goes too.
                    if let Some(start) = start
                        && dropped
                        && out.len() == start + 1
                    {
                        removed += 2;
                        out.truncate(start);
                        changed.retain(|&l| l <= start);
                        continue;
                    }
                }
                _ => {}
            }
            out.push(line.to_string());
            continue;
        }
        if let Some(Frame::Component {
            entity,
            name,
            kept,
            dropped,
            ..
        }) = stack.last_mut()
        {
            let t = tokens(trimmed);
            if let Some((key, old)) = t.split_first() {
                listed.insert((entity.clone(), name.clone(), key.to_string()));
                if let Some(v) = current(entity, name, key) {
                    if at_base(entity, name, key, &v) {
                        // Back at the prefab's value: the override goes.
                        removed += 1;
                        *dropped = true;
                        continue;
                    }
                    *kept += 1;
                    if !same(&v, old) {
                        let l = format!("{indent}{key}{}{comment}", spell(&v, old));
                        push(&mut out, &mut changed, l, true);
                        continue;
                    }
                } else {
                    *kept += 1;
                }
            }
        }
        out.push(line.to_string());
    }
    // Tuned resources no block sets: one block each at the end of the
    // authored text, where the plan adds them.
    if plan.add_resources {
        let mut blocks: Vec<(String, Vec<Placed>)> = Vec::new();
        for (i, (e, c, f)) in tuned.iter().enumerate() {
            if e != RESOURCE
                || placed.contains(&i)
                || listed.contains(&(e.clone(), c.clone(), f.clone()))
                || components.contains(&(e.clone(), c.clone()))
            {
                continue;
            }
            let Some(v) = current(e, c, f) else { continue };
            match blocks.iter_mut().find(|b| b.0 == *c) {
                Some(b) => b.1.push((i, f.clone(), v)),
                None => blocks.push((c.clone(), vec![(i, f.clone(), v)])),
            }
        }
        if !blocks.is_empty() {
            let ended = out.last().is_some_and(String::is_empty);
            if ended {
                out.pop();
            }
            for (c, fields) in blocks {
                push(&mut out, &mut changed, format!("resource {c} {{"), true);
                for (i, f, v) in fields {
                    push(
                        &mut out,
                        &mut changed,
                        format!("  {f}{}", spell(&v, &[])),
                        true,
                    );
                    placed.insert(i);
                }
                push(&mut out, &mut changed, "}".to_string(), true);
            }
            if ended {
                out.push(String::new());
            }
        }
    }
    let mut skipped = Vec::new();
    for (i, (e, c, f)) in tuned.iter().enumerate() {
        if placed.contains(&i) || listed.contains(&(e.clone(), c.clone(), f.clone())) {
            continue;
        }
        skipped.push(if e == RESOURCE {
            format!("{c}.{f}: a resource this scene does not set")
        } else if entities.contains(e) {
            format!("{e}.{c}.{f}: the world has no such field")
        } else {
            format!("{e}.{c}.{f}: '{e}' is not an entity of the scene")
        });
    }
    Rewrite {
        text: out.join("\n"),
        lines: changed,
        skipped,
        removed,
    }
}

/// The fields a text lists for entity `entity` (named as
/// [`rewrite_instances`] names it), as `(component, field)` in text order:
/// an instance's overrides of it.
pub fn listed_fields(text: &str, entity: &str, paths: bool) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let record = std::cell::RefCell::new(Vec::new());
    let current = |e: &str, c: &str, f: &str| -> Option<FieldValue> {
        if e == entity {
            record.borrow_mut().push((c.to_string(), f.to_string()));
        }
        None
    };
    rewrite_instances(
        text,
        &current,
        &Plan::default(),
        &Instances {
            base: &no_base,
            paths,
        },
    );
    for x in record.into_inner() {
        if !out.contains(&x) {
            out.push(x);
        }
    }
    out
}
