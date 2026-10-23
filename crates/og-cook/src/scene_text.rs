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

/// A field's current value, as the scene text would give it.
#[derive(Clone, Debug, PartialEq)]
pub enum FieldValue {
    /// Numbers, one per lane (a vector's `x y z`, a scalar's one).
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
}

enum Frame {
    Entity {
        name: String,
        indent: String,
    },
    Component {
        entity: String,
        name: String,
        indent: String,
    },
    Other,
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

/// The value tokens of `value`, with the old tokens' spelling kept where
/// they read the same (a `true` stays a word).
fn spell(value: &FieldValue, old: &[&str]) -> String {
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
    let all: Vec<&str> = text.split('\n').collect();
    let mark = all
        .iter()
        .position(|l| l.trim_start().starts_with(SPAWNED_MARK));
    let (head, old_section): (&[&str], &[&str]) = match (&plan.spawned, mark) {
        (Some(_), Some(m)) => (&all[..m], &all[m..]),
        _ => (&all[..], &[]),
    };
    let mut r = rewrite_lines(head, current, plan);
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
            out.push(format!("{indent}    {f} {}", spell(v, &[])));
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
) -> Rewrite {
    let tuned = &plan.tuned;
    let mut out: Vec<String> = Vec::with_capacity(lines.len() + tuned.len());
    let mut changed: Vec<usize> = Vec::new();
    let mut stack: Vec<Frame> = Vec::new();
    // Per (entity, component): the fields the text lists.
    let mut listed: std::collections::BTreeSet<(String, String, String)> = Default::default();
    let mut components: std::collections::BTreeSet<(String, String)> = Default::default();
    let mut entities: std::collections::BTreeSet<String> = Default::default();
    let mut placed: std::collections::BTreeSet<usize> = Default::default();
    let mut body = !lines.iter().any(|l| l.trim() == "---");
    let indent_of = |l: &str| l[..l.len() - l.trim_start().len()].to_string();
    let push = |out: &mut Vec<String>, changed: &mut Vec<usize>, line: String, new: bool| {
        out.push(line);
        if new {
            changed.push(out.len());
        }
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
        if let Some(head) = trimmed.strip_suffix('{') {
            let t = tokens(head);
            let frame = match (stack.last(), t.as_slice()) {
                (None | Some(Frame::Entity { .. }), ["entity", name]) => match unquote(name) {
                    Some(name) => {
                        entities.insert(name.clone());
                        Frame::Entity {
                            name,
                            indent: indent.clone(),
                        }
                    }
                    None => Frame::Other,
                },
                (None, ["resource", r]) => {
                    let r = unquote(r).unwrap_or_else(|| r.to_string());
                    components.insert((RESOURCE.to_string(), r.clone()));
                    Frame::Component {
                        entity: RESOURCE.to_string(),
                        name: r,
                        indent: indent.clone(),
                    }
                }
                (Some(Frame::Entity { name, .. }), [comp]) => {
                    components.insert((name.clone(), comp.to_string()));
                    Frame::Component {
                        entity: name.clone(),
                        name: comp.to_string(),
                        indent: indent.clone(),
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
                }) => {
                    // Tuned fields of this component the text lacks.
                    for (i, (e, c, f)) in tuned.iter().enumerate() {
                        if *e == entity
                            && *c == name
                            && !listed.contains(&(e.clone(), c.clone(), f.clone()))
                            && let Some(v) = current(e, c, f)
                        {
                            let l = format!("{ci}  {f} {}", spell(&v, &[]));
                            push(&mut out, &mut changed, l, true);
                            listed.insert((e.clone(), c.clone(), f.clone()));
                            placed.insert(i);
                        }
                    }
                }
                Some(Frame::Entity { name, indent: ei }) => {
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
                            .collect();
                        if values.is_empty() {
                            continue;
                        }
                        push(&mut out, &mut changed, format!("{ei}  {c} {{"), true);
                        for (i, f, v) in values {
                            push(
                                &mut out,
                                &mut changed,
                                format!("{ei}    {f} {}", spell(&v, &[])),
                                true,
                            );
                            placed.insert(i);
                        }
                        push(&mut out, &mut changed, format!("{ei}  }}"), true);
                        components.insert((name.clone(), c));
                    }
                }
                _ => {}
            }
            out.push(line.to_string());
            continue;
        }
        if let Some(Frame::Component { entity, name, .. }) = stack.last() {
            let t = tokens(trimmed);
            if let Some((key, old)) = t.split_first() {
                listed.insert((entity.clone(), name.clone(), key.to_string()));
                if let Some(v) = current(entity, name, key)
                    && !same(&v, old)
                {
                    let l = format!("{indent}{key} {}{comment}", spell(&v, old));
                    push(&mut out, &mut changed, l, true);
                    continue;
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
                        format!("  {f} {}", spell(&v, &[])),
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const SCENE: &str = "type scene
version 1
---
# the level
name \"Main\"
entity \"ship\" {
  model \"models/ship.glb\"   # the hull
  Transform {
    position 0 1.5 0 # spawn
    quaternion 0 0 0 1
    scale 1 1 1
  }
  entity \"light\" {
    Transform {
      position 0 3 0
    }
    Light {
      on true
      label \"key # light\"
    }
  }
}
";

    fn world(
        changes: &[(&str, &str, &str, FieldValue)],
    ) -> impl Fn(&str, &str, &str) -> Option<FieldValue> {
        let base: Vec<(String, String, String, FieldValue)> = vec![
            (
                "ship",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 1.5, 0.0]),
            ),
            (
                "ship",
                "Transform",
                "quaternion",
                FieldValue::Lanes(vec![0.0, 0.0, 0.0, 1.0]),
            ),
            (
                "ship",
                "Transform",
                "scale",
                FieldValue::Lanes(vec![1.0, 1.0, 1.0]),
            ),
            (
                "light",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 3.0, 0.0]),
            ),
            (
                "light",
                "Transform",
                "scale",
                FieldValue::Lanes(vec![1.0, 1.0, 1.0]),
            ),
            ("light", "Light", "on", FieldValue::Bool(true)),
            (
                "light",
                "Light",
                "label",
                FieldValue::Text("key # light".into()),
            ),
            ("light", "Light", "range", FieldValue::Lanes(vec![10.0])),
        ]
        .into_iter()
        .map(|(e, c, f, v)| (e.into(), c.into(), f.into(), v))
        .collect();
        let changes: Vec<(String, String, String, FieldValue)> = changes
            .iter()
            .map(|(e, c, f, v)| (e.to_string(), c.to_string(), f.to_string(), v.clone()))
            .collect();
        move |e, c, f| {
            changes
                .iter()
                .chain(base.iter())
                .find(|(a, b, d, _)| a == e && b == c && d == f)
                .map(|x| x.3.clone())
        }
    }

    #[test]
    fn unchanged_world_rewrites_nothing() {
        let r = rewrite(SCENE, &world(&[]), &[]);
        assert_eq!(r.text, SCENE);
        assert!(r.lines.is_empty());
    }

    #[test]
    fn values_rounding_to_the_same_f32_keep_their_line() {
        // 1.5000000001 is 1.5 as an f32: the line is not rewritten.
        let r = rewrite(
            SCENE,
            &world(&[(
                "ship",
                "Transform",
                "position",
                FieldValue::Lanes(vec![0.0, 1.500_000_000_1, -0.0]),
            )]),
            &[],
        );
        assert_eq!(r.text, SCENE);
    }

    #[test]
    fn changed_fields_rewrite_their_line_only_with_comment_and_spelling() {
        let r = rewrite(
            SCENE,
            &world(&[
                (
                    "ship",
                    "Transform",
                    "position",
                    FieldValue::Lanes(vec![-2.25, 1.5, 0.1]),
                ),
                ("light", "Light", "on", FieldValue::Bool(false)),
                (
                    "light",
                    "Light",
                    "label",
                    FieldValue::Text("fill \"b\"".into()),
                ),
            ]),
            &[],
        );
        let want = SCENE
            .replace("position 0 1.5 0 # spawn", "position -2.25 1.5 0.1 # spawn")
            .replace("on true", "on false")
            .replace("label \"key # light\"", "label \"fill \\\"b\\\"\"");
        assert_eq!(r.text, want);
        assert_eq!(r.lines, vec![9, 18, 19]);
    }

    #[test]
    fn tuned_fields_the_text_lacks_are_added_in_place() {
        let tuned: Vec<Tuned> = [
            ("light", "Transform", "scale"),
            ("light", "Light", "range"),
            ("ship", "Light", "range"),
            ("ghost", "Transform", "scale"),
        ]
        .iter()
        .map(|(e, c, f)| (e.to_string(), c.to_string(), f.to_string()))
        .collect();
        let r = rewrite(
            SCENE,
            &world(&[("ship", "Light", "range", FieldValue::Lanes(vec![2.5]))]),
            &tuned,
        );
        let want = SCENE
            .replace(
                "      position 0 3 0\n    }",
                "      position 0 3 0\n      scale 1 1 1\n    }",
            )
            .replace(
                "      label \"key # light\"\n    }",
                "      label \"key # light\"\n      range 10\n    }",
            )
            .replace(
                "    }\n  }\n}\n",
                "    }\n  }\n  Light {\n    range 2.5\n  }\n}\n",
            );
        assert_eq!(r.text, want);
        assert_eq!(r.lines, vec![16, 21, 24, 25, 26]);
        assert_eq!(
            r.skipped,
            vec!["ghost.Transform.scale: 'ghost' is not an entity of the scene"]
        );
        // Written again from the same world: nothing moves.
        let again = rewrite(
            &r.text,
            &world(&[("ship", "Light", "range", FieldValue::Lanes(vec![2.5]))]),
            &tuned,
        );
        assert_eq!(again.text, r.text);
        assert!(again.lines.is_empty());
    }

    #[test]
    fn numbers_are_the_shortest_f32_text() {
        assert_eq!(number(0.823_456_7), "0.8234567");
        assert_eq!(number(-0.0), "0");
        assert_eq!(number(2.0), "2");
        assert_eq!(number(0.1_f32 as f64), "0.1");
    }
}
