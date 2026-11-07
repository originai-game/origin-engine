//! Yarn 3 authoring compiled to engine-owned, stable-ID continuation blocks.
//! This is an original Rust implementation; see third_party/yarn-spinner/.

use crate::asset::*;
use crate::expr::{Expr, Val};
use std::collections::{BTreeMap, BTreeSet};

pub mod functions;
pub mod markup;
pub mod saliency;
mod syntax;
mod text;
pub mod types;
mod typing;

/// Parses the Yarn expression grammar without changing quest expressions.
pub fn parse_expression(text: &str) -> Result<Expr, String> {
    syntax::parse(text)
}

pub const PROFILE: &str = "yarn-spinner-3.2.2";

#[derive(Clone)]
struct Source {
    text: String,
    indent: usize,
    line: usize,
    blank_before: bool,
}

enum Statement {
    Line(Line, Option<Expr>),
    Action(String, Step, usize),
    Detour(String, Expr, usize),
    Branch(String, Vec<(Expr, Vec<Statement>)>, Vec<Statement>, usize),
    Options(Vec<(Line, Option<Expr>, f64, Vec<Statement>)>, bool, usize),
}

struct Compiler {
    dialogue: Dialogue,
    ids: BTreeSet<String>,
    blocks: BTreeSet<String>,
    instructions: BTreeMap<String, u32>,
    tags: BTreeMap<String, BTreeMap<String, String>>,
    shadows: BTreeMap<String, (String, usize)>,
    title: String,
    locals: BTreeSet<String>,
    once_lines: BTreeSet<String>,
    declarations: Vec<typing::Declaration>,
    types: types::Types,
    bark: bool,
}

fn error(line: usize, message: impl std::fmt::Display) -> String {
    format!("line {line}: {message}")
}

fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| syntax::identifier_continue(c) || matches!(c, '.' | '-'))
}

fn expression(s: &str, line: usize) -> Result<Expr, String> {
    parse_expression(s).map_err(|e| error(line, e))
}

fn once_condition(id: &str, suffix: &str, line: usize) -> Result<Expr, String> {
    let once = Expr::Not(Box::new(Expr::Once(id.into())));
    if suffix.is_empty() {
        return Ok(once);
    }
    let condition = suffix
        .strip_prefix("if ")
        .ok_or_else(|| error(line, "once accepts only an optional if condition"))?;
    Ok(Expr::Bin(
        crate::expr::Op::And,
        Box::new(once),
        Box::new(expression(condition, line)?),
    ))
}

fn destination(s: &str, line: usize) -> Result<Expr, String> {
    if identifier(s) {
        Ok(Expr::Lit(Val::Str(s.into())))
    } else if let Some(s) = s.strip_prefix('{').and_then(|s| s.strip_suffix('}')) {
        expression(s, line)
    } else {
        Err(error(
            line,
            "jump or detour needs a node name or {string expression}",
        ))
    }
}

fn command(s: &str) -> Option<&str> {
    s.strip_prefix("<<")?.strip_suffix(">>").map(str::trim)
}

/// Comments do not start inside quoted strings or after an escape.
pub fn without_comment(s: &str) -> &str {
    let mut quoted = false;
    let mut escaped = false;
    for (i, c) in s.char_indices() {
        if escaped {
            escaped = false;
        } else if c == '\\' {
            escaped = true;
        } else if c == '"' {
            quoted = !quoted;
        } else if !quoted && s[i..].starts_with("//") {
            return s[..i].trim_end();
        }
    }
    s
}

fn split_tags(s: &str, line: usize) -> Result<(&str, BTreeMap<&str, &str>), String> {
    let mut escaped = false;
    let mut quoted = false;
    let mut expression = false;
    let at = s
        .char_indices()
        .find_map(|(i, c)| {
            if escaped {
                escaped = false;
                return None;
            }
            if c == '\\' {
                escaped = true;
                return None;
            }
            if c == '"' && expression {
                quoted = !quoted;
            }
            if !quoted {
                if c == '{' {
                    expression = true;
                }
                if c == '}' {
                    expression = false;
                }
                if c == '#' && !expression {
                    return Some(i);
                }
            }
            None
        })
        .unwrap_or(s.len());
    let mut tags = BTreeMap::new();
    for t in s[at..].split_whitespace() {
        let t = t.strip_prefix('#').ok_or_else(|| {
            error(
                line,
                "text after a hashtag is unsupported; place text before tags",
            )
        })?;
        let (key, value) = t.split_once(':').unwrap_or((t, ""));
        if tags.insert(key, value).is_some() {
            return Err(error(line, format!("duplicate #{key} tag")));
        }
    }
    Ok((s[..at].trim_end(), tags))
}

impl Compiler {
    fn instruction(&mut self, owner: &str, kind: &str, text: &str) -> String {
        let identity = format!("{owner}/{kind}:{text}");
        let occurrence = self.instructions.entry(identity.clone()).or_default();
        let id = if *occurrence == 0 {
            identity
        } else {
            format!("{identity}/occurrence:{occurrence}")
        };
        *occurrence += 1;
        id
    }

    fn enumeration(
        &mut self,
        name: &str,
        lines: &[Source],
        at: &mut usize,
        line: usize,
    ) -> Result<(), String> {
        use types::{Enum, Type};
        if !syntax::identifier(name) || self.types.enums.contains_key(name) {
            return Err(error(line, "invalid or duplicate enum name"));
        }
        let mut cases = Vec::new();
        loop {
            let source = lines
                .get(*at)
                .ok_or_else(|| error(line, "missing <<endenum>>"))?;
            *at += 1;
            let cmd = command(&source.text)
                .ok_or_else(|| error(source.line, "enum needs <<case Name>>"))?;
            if cmd == "endenum" {
                break;
            }
            let case = cmd
                .strip_prefix("case ")
                .ok_or_else(|| error(source.line, "enum needs <<case Name>>"))?;
            let (member, raw) = case
                .split_once('=')
                .or_else(|| case.split_once(" to "))
                .map_or((case.trim(), None), |(name, value)| {
                    (name.trim(), Some(value.trim()))
                });
            if !syntax::identifier(member) || cases.iter().any(|(name, _)| name == member) {
                return Err(error(source.line, "enum cases need unique names"));
            }
            let raw = raw
                .map(|raw| {
                    let value =
                        typing::literal(&expression(raw, source.line)?).ok_or_else(|| {
                            error(
                                source.line,
                                "enum raw values must be constant number or string literals",
                            )
                        })?;
                    match &value {
                        Val::Str(_) => Ok(value),
                        Val::Num(n) if n.fract() == 0.0 => Ok(value),
                        _ => Err(error(
                            source.line,
                            "enum raw values must be integer numbers or strings",
                        )),
                    }
                })
                .transpose()?;
            cases.push((member.to_string(), raw));
        }
        if cases.is_empty() {
            return Err(error(line, "an enum must have cases"));
        }
        let specified = cases.iter().filter(|(_, raw)| raw.is_some()).count();
        if specified != 0 && specified != cases.len() {
            return Err(error(
                line,
                "all enum cases must have raw values when any case does",
            ));
        }
        let mut members = BTreeMap::new();
        let mut raw_type = None;
        for (i, (name, raw)) in cases.into_iter().enumerate() {
            let value = raw.unwrap_or(Val::Num(i as f64));
            let ty = Type::of(&value);
            if raw_type.as_ref().is_some_and(|prior| *prior != ty) {
                return Err(error(line, "enum raw values must have the same type"));
            }
            if members.values().any(|prior| prior == &value) {
                return Err(error(line, "enum cases must have unique raw values"));
            }
            raw_type = Some(ty);
            members.insert(name, value);
        }
        self.types.enums.insert(
            name.into(),
            Enum {
                raw: raw_type.unwrap(),
                members,
            },
        );
        Ok(())
    }

    fn line(&mut self, s: &str, at: usize) -> Result<(Line, Option<Expr>, f64), String> {
        let (mut text, tags) = split_tags(s, at)?;
        let key = tags
            .get("line")
            .ok_or_else(|| {
                error(
                    at,
                    "missing explicit #line: id; append #line:unique_id to every line and option",
                )
            })?
            .to_string();
        if !identifier(&key) || !self.ids.insert(key.clone()) {
            return Err(error(
                at,
                format!("invalid or duplicate #line:{key}; use a unique stable id"),
            ));
        }
        if let Some(source) = tags.get("shadow") {
            self.shadows.insert(key.clone(), ((*source).into(), at));
        }
        self.tags.insert(
            key.clone(),
            tags.iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        );
        let cond = if let Some((prefix, suffix)) = text.rsplit_once("<<once") {
            text = prefix.trim_end();
            let suffix = suffix
                .strip_suffix(">>")
                .ok_or_else(|| error(at, "close <<once>>"))?
                .trim();
            self.once_lines.insert(key.clone());
            Some(once_condition(&format!("line:{key}"), suffix, at)?)
        } else if let Some((prefix, suffix)) = text.rsplit_once("<<if ") {
            text = prefix.trim_end();
            Some(expression(
                suffix
                    .strip_suffix(">>")
                    .ok_or_else(|| error(at, "close <<if ...>>"))?,
                at,
            )?)
        } else {
            None
        };
        if text.contains("<<") {
            return Err(error(at, "unsupported Yarn line condition"));
        }
        let speaker = match text.split_once(": ") {
            Some((s, _)) if identifier(s) => s,
            _ => NARRATOR,
        };
        if speaker != NARRATOR && self.dialogue.speaker(speaker).is_none() {
            self.dialogue.speakers.push(Speaker {
                id: speaker.into(),
                side: "left".into(),
                ..Default::default()
            });
        }
        let mut base = String::new();
        let mut args = Vec::new();
        for part in text::parts(text).map_err(|e| error(at, e))? {
            let expr = match part {
                text::Part::Text(text) => {
                    base.push_str(&text::icu_literal(&text));
                    continue;
                }
                text::Part::Expression(expr) => expr,
            };
            let simple = expr.trim().strip_prefix('$').unwrap_or(expr.trim());
            let name = if !simple.is_empty()
                && simple
                    .chars()
                    .all(|c| c.is_alphanumeric() || matches!(c, '_' | '.' | '-'))
            {
                simple.rsplit('.').next().unwrap().to_string()
            } else {
                format!("arg{}", args.len())
            };
            let parsed = if self.bark && !expr.contains('$') && expr.contains('.') {
                Expr::parse(&expr).map_err(|e| error(at, e))?
            } else {
                expression(&expr, at)?
            };
            if let Some((_, prior)) = args.iter().find(|(n, _)| n == &name) {
                if prior != &parsed {
                    return Err(error(
                        at,
                        format!("conflicting inline argument {name}; assign distinct variables"),
                    ));
                }
            } else {
                args.push((name.clone(), parsed));
            }
            base.push_str(&format!("{{{name}}}"));
        }
        if self.shadows.contains_key(&key) && !args.is_empty() {
            return Err(error(at, "shadow lines must not have expressions"));
        }
        self.dialogue.strings.insert(key.clone(), base);
        let weight = tags
            .get("weight")
            .map_or(Ok(1.0), |w| w.parse::<f64>())
            .map_err(|_| error(at, "#weight needs a positive number"))?;
        if !weight.is_finite() || weight <= 0.0 {
            return Err(error(at, "#weight needs a positive finite number"));
        }
        Ok((
            Line {
                speaker: speaker.into(),
                text: Text { key, args },
                voice: tags.get("voice").copied().unwrap_or_default().into(),
                expression: tags.get("expression").copied().unwrap_or_default().into(),
                shot: tags.get("shot").copied().unwrap_or_default().into(),
                line: at,
            },
            cond,
            weight,
        ))
    }

    fn body(
        &mut self,
        lines: &[Source],
        at: &mut usize,
        indent: usize,
        owner: &str,
    ) -> Result<Vec<Statement>, String> {
        let mut out = Vec::new();
        while let Some(src) = lines.get(*at) {
            if src.indent < indent {
                break;
            }
            let text = src.text.as_str();
            if matches!(command(text), Some("else" | "endif" | "endonce"))
                || text.starts_with("<<elseif ")
            {
                break;
            }
            if text.starts_with("->") || text.starts_with("=>") {
                let random = text.starts_with("=>");
                let prefix = if random { "=>" } else { "->" };
                let group_indent = src.indent;
                let group_line = src.line;
                let mut options = Vec::new();
                while let Some(src) = lines
                    .get(*at)
                    .filter(|s| s.indent == group_indent && s.text.starts_with(prefix))
                {
                    let (line, cond, weight) = self.line(src.text[2..].trim_start(), src.line)?;
                    *at += 1;
                    let body =
                        if let Some(next) = lines.get(*at).filter(|s| s.indent > group_indent) {
                            self.body(
                                lines,
                                at,
                                next.indent,
                                &format!("{owner}/option:{}", line.text.key),
                            )?
                        } else {
                            Vec::new()
                        };
                    options.push((line, cond, weight, body));
                    if lines.get(*at).is_some_and(|s| s.blank_before) {
                        break;
                    }
                }
                out.push(Statement::Options(options, random, group_line));
                continue;
            }
            *at += 1;
            if let Some(cmd) = command(text) {
                if let Some(cond) = cmd.strip_prefix("if ") {
                    let id = self.instruction(owner, "if", cond);
                    let mut branches = vec![(
                        expression(cond, src.line)?,
                        self.body(lines, at, indent, owner)?,
                    )];
                    while let Some(next) = lines.get(*at).and_then(|s| {
                        command(&s.text)
                            .and_then(|c| c.strip_prefix("elseif "))
                            .map(|c| (s.line, c))
                    }) {
                        *at += 1;
                        branches.push((
                            expression(next.1, next.0)?,
                            self.body(lines, at, indent, owner)?,
                        ));
                    }
                    let otherwise = if lines.get(*at).is_some_and(|s| s.text == "<<else>>") {
                        *at += 1;
                        self.body(lines, at, indent, owner)?
                    } else {
                        Vec::new()
                    };
                    if !lines.get(*at).is_some_and(|s| s.text == "<<endif>>") {
                        return Err(error(src.line, "missing <<endif>>"));
                    }
                    *at += 1;
                    out.push(Statement::Branch(id, branches, otherwise, src.line));
                } else if cmd == "once" || cmd.starts_with("once if ") {
                    let id = self.instruction(owner, "once", cmd);
                    let cond =
                        once_condition(&id, cmd.strip_prefix("once").unwrap().trim(), src.line)?;
                    let mut body = self.body(lines, at, indent, owner)?;
                    body.insert(
                        0,
                        Statement::Action(
                            format!("{id}/mark"),
                            Step::MarkOnce(id.clone()),
                            src.line,
                        ),
                    );
                    let otherwise = if lines.get(*at).is_some_and(|s| s.text == "<<else>>") {
                        *at += 1;
                        self.body(lines, at, indent, owner)?
                    } else {
                        Vec::new()
                    };
                    if !lines.get(*at).is_some_and(|s| s.text == "<<endonce>>") {
                        return Err(error(src.line, "missing <<endonce>>"));
                    }
                    *at += 1;
                    out.push(Statement::Branch(
                        id,
                        vec![(cond, body)],
                        otherwise,
                        src.line,
                    ));
                } else if let Some(name) = cmd.strip_prefix("enum ") {
                    self.enumeration(name, lines, at, src.line)?;
                } else if let Some(decl) = cmd.strip_prefix("declare ") {
                    let (name, value, op) = assignment(decl, src.line)?;
                    if op.is_some() {
                        return Err(error(
                            src.line,
                            "a declaration uses = or to, not compound assignment",
                        ));
                    }
                    let (value, ty) = value
                        .rsplit_once(" as ")
                        .filter(|(_, ty)| identifier(ty))
                        .map_or((value, None), |(value, ty)| {
                            (value, Some(types::Type::named(ty)))
                        });
                    if self.declarations.iter().any(|v| v.0 == name) {
                        return Err(error(src.line, format!("duplicate declaration ${name}")));
                    }
                    self.declarations
                        .push((name, expression(value, src.line)?, ty, src.line));
                } else {
                    let id = self.instruction(owner, "command", text);
                    if let Some(target) = cmd.strip_prefix("detour ") {
                        out.push(Statement::Detour(
                            id,
                            destination(target, src.line)?,
                            src.line,
                        ));
                    } else {
                        let step = if let Some(value) = cmd.strip_prefix("set ") {
                            let (name, expr, op) = assignment(value, src.line)?;
                            let mut expr = expression(expr, src.line)?;
                            if let Some(op) = op {
                                expr = Expr::Yarn(Box::new(Expr::Bin(
                                    op,
                                    Box::new(Expr::Var(name.clone())),
                                    Box::new(expr),
                                )));
                            }
                            Step::Set(name, expr)
                        } else if let Some(call) = cmd.strip_prefix("call ") {
                            let call = expression(call, src.line)?;
                            if !matches!(&call, Expr::Yarn(e) if matches!(e.as_ref(), Expr::Invoke(..)))
                            {
                                return Err(error(src.line, "<<call>> needs a function call"));
                            }
                            Step::Call(call)
                        } else if let Some(target) = cmd.strip_prefix("jump ") {
                            Step::Jump(destination(target, src.line)?)
                        } else if cmd == "return" {
                            Step::Return
                        } else if cmd == "stop" {
                            Step::End
                        } else if let Some(op) = cmd.strip_prefix("quest ") {
                            let words: Vec<_> = op.split_whitespace().collect();
                            Step::Quest(match words.as_slice() {
                                ["start", q] => QuestOp::Start((*q).into()),
                                ["complete", q] => QuestOp::Complete((*q).into()),
                                ["fail", q] => QuestOp::Fail((*q).into()),
                                ["stage", q, stage] => QuestOp::Stage((*q).into(), (*stage).into()),
                                ["progress", q, objective] => QuestOp::Progress(
                                    (*q).into(),
                                    (*objective).into(),
                                    Expr::Lit(Val::Num(1.0)),
                                ),
                                _ => {
                                    return Err(error(
                                        src.line,
                                        "unsupported quest command; use start, complete, fail, stage or progress",
                                    ));
                                }
                            })
                        } else if matches!(
                            cmd.split_whitespace().next(),
                            Some(
                                "once"
                                    | "endonce"
                                    | "enum"
                                    | "case"
                                    | "endenum"
                                    | "else"
                                    | "endif"
                                    | "elseif"
                                    | "if"
                                    | "declare"
                                    | "set"
                                    | "jump"
                                    | "detour"
                                    | "return"
                                    | "stop"
                                    | "quest"
                            )
                        ) {
                            return Err(error(
                                src.line,
                                format!("unsupported Yarn feature '{cmd}' in {PROFILE}"),
                            ));
                        } else {
                            let mut value = Expr::Lit(Val::Str(String::new()));
                            for part in text::parts(cmd).map_err(|e| error(src.line, e))? {
                                let part = match part {
                                    text::Part::Text(s) => Expr::Lit(Val::Str(s)),
                                    text::Part::Expression(e) => Expr::Invoke(
                                        "string".into(),
                                        vec![expression(&e, src.line)?],
                                    ),
                                };
                                value = Expr::Bin(
                                    crate::expr::Op::Add,
                                    Box::new(value),
                                    Box::new(part),
                                );
                            }
                            Step::Command(Expr::Yarn(Box::new(value)))
                        };
                        out.push(Statement::Action(id, step, src.line));
                    }
                }
            } else if text.starts_with("<<") || text.starts_with("type dialogue") {
                return Err(error(
                    src.line,
                    "unsupported or malformed Yarn statement; use .yarn title:/---/=== nodes",
                ));
            } else {
                let (line, cond, _) = self.line(text, src.line)?;
                out.push(Statement::Line(line, cond));
            }
        }
        Ok(out)
    }

    fn block(
        &mut self,
        id: String,
        steps: Vec<Step>,
        choices: Vec<Choice>,
        at: usize,
    ) -> Result<String, String> {
        if !self.blocks.insert(id.clone()) {
            return Err(error(at, format!("duplicate instruction identity '{id}'")));
        }
        self.dialogue.nodes.push(Node {
            id: id.clone(),
            title: self.title.clone(),
            lines: vec![at; steps.len()],
            steps,
            choices,
            timer: None,
            choice_line: None,
            choice_fallback: None,
            shot: String::new(),
            line: at,
        });
        Ok(id)
    }

    fn lower(&mut self, body: Vec<Statement>, mut next: String) -> Result<String, String> {
        for statement in body.into_iter().rev() {
            next = match statement {
                Statement::Line(line, cond) => {
                    let id = format!("line:{}", line.text.key);
                    let at = line.line;
                    let mut steps = Vec::new();
                    if let Some(cond) = cond {
                        steps.push(Step::If(Expr::Not(Box::new(cond)), next.clone()));
                    }
                    if self.once_lines.contains(&line.text.key) {
                        steps.push(Step::MarkOnce(format!("line:{}", line.text.key)));
                    }
                    let line_step = steps.len();
                    steps.push(Step::Say(line));
                    let mut pending = Vec::new();
                    let mut cursor = next.as_str();
                    let mut visited = BTreeSet::new();
                    let mut choices = Vec::new();
                    let mut timer = None;
                    let mut choice_fallback = None;
                    while visited.insert(cursor.to_string()) {
                        let Some(node) = self.dialogue.node(cursor) else {
                            break;
                        };
                        if !node.choices.is_empty() && node.choice_line.is_none() {
                            choices = node.choices.clone();
                            timer = node.timer.clone();
                            choice_fallback = node.choice_fallback.clone();
                            break;
                        }
                        match node.steps.as_slice() {
                            [
                                action @ (Step::Set(..)
                                | Step::Emit(..)
                                | Step::Command(..)
                                | Step::Call(..)
                                | Step::Quest(..)),
                                Step::Goto(to),
                            ] => {
                                pending.push(action.clone());
                                cursor = to;
                            }
                            _ => break,
                        }
                    }
                    let choice_line = if choices.is_empty() {
                        steps.push(Step::Goto(next));
                        None
                    } else {
                        steps.extend(pending);
                        Some(line_step)
                    };
                    let id = self.block(id, steps, choices, at)?;
                    let node = self.dialogue.nodes.last_mut().unwrap();
                    node.choice_line = choice_line;
                    node.timer = timer;
                    node.choice_fallback = choice_fallback;
                    id
                }
                Statement::Action(id, step, at) => {
                    self.block(id, vec![step, Step::Goto(next)], Vec::new(), at)?
                }
                Statement::Detour(id, target, at) => {
                    let resume = self.block(
                        format!("{id}/return"),
                        vec![Step::Goto(next)],
                        Vec::new(),
                        at,
                    )?;
                    let step = match target {
                        Expr::Lit(Val::Str(target)) => Step::Detour(target, resume),
                        target => Step::DynamicDetour(target, resume),
                    };
                    self.block(id, vec![step], Vec::new(), at)?
                }
                Statement::Branch(id, branches, otherwise, at) => {
                    let fallback = self.lower(otherwise, next.clone())?;
                    let mut steps = Vec::new();
                    for (cond, body) in branches {
                        steps.push(Step::If(cond, self.lower(body, next.clone())?));
                    }
                    steps.push(Step::Goto(fallback));
                    self.block(id, steps, Vec::new(), at)?
                }
                Statement::Options(options, random, at) => {
                    let mut ids: Vec<_> = options.iter().map(|o| o.0.text.key.as_str()).collect();
                    ids.sort();
                    let id = format!("options:{}", ids.join("/"));
                    let mut choices = Vec::new();
                    let mut picks = Vec::new();
                    let mut timer = None;
                    for (line, cond, weight, mut body) in options {
                        if !random && self.once_lines.contains(&line.text.key) {
                            let id = format!("line:{}", line.text.key);
                            body.insert(
                                0,
                                Statement::Action(
                                    format!("{id}/mark"),
                                    Step::MarkOnce(id),
                                    line.line,
                                ),
                            );
                        }
                        let target = self.lower(body, next.clone())?;
                        if random {
                            let target = self.lower(vec![Statement::Line(line, None)], target)?;
                            picks.push((target, cond, weight));
                        } else {
                            let tags = &self.tags[&line.text.key];
                            let check = tags.get("check").map(|s| {
                                let parts: Vec<_> = s.split(':').collect();
                                match parts.as_slice() {
                                    [kind @ ("active" | "passive"), stat, value] => Ok(Check { active: *kind == "active", stat: (*stat).into(), target: expression(value, at)? }),
                                    _ => Err(error(at, "#check:active:stat:target or #check:passive:stat:target")),
                                }
                            }).transpose()?;
                            let fail = tags.get("fail").cloned();
                            if check.as_ref().is_some_and(|k| k.active) && fail.is_none() {
                                return Err(error(at, "active check needs #fail:Node"));
                            }
                            if let Some(ticks) = tags.get("timer") {
                                let ticks: u32 = ticks
                                    .parse()
                                    .map_err(|_| error(at, "#timer needs positive ticks"))?;
                                if ticks == 0 || timer.is_some() {
                                    return Err(error(at, "one positive timer per option group"));
                                }
                                timer = Some((ticks, line.text.key.clone()));
                            }
                            let mut reason = tags.get("reason").map(|key| Text {
                                key: key.clone(),
                                args: Vec::new(),
                            });
                            if let Some(arg) = tags.get("reason-arg") {
                                let (name, expr) = arg
                                    .split_once('=')
                                    .ok_or_else(|| error(at, "#reason-arg:name=expression"))?;
                                reason
                                    .as_mut()
                                    .ok_or_else(|| error(at, "#reason-arg needs #reason:key"))?
                                    .args
                                    .push((name.into(), expression(expr, at)?));
                            }
                            choices.push(Choice {
                                id: line.text.key.clone(),
                                text: line.text,
                                cond,
                                hide: tags.contains_key("hide"),
                                reason,
                                kind: tags.get("kind").cloned().unwrap_or_default(),
                                check,
                                steps: Vec::new(),
                                goto: Some(target),
                                fail,
                                line: line.line,
                            });
                        }
                    }
                    let fallback = (!random).then(|| next.clone());
                    let steps = if random {
                        vec![Step::Select(picks), Step::Goto(next)]
                    } else {
                        Vec::new()
                    };
                    let id = self.block(id, steps, choices, at)?;
                    let node = self.dialogue.nodes.last_mut().unwrap();
                    node.timer = timer;
                    node.choice_fallback = fallback;
                    id
                }
            };
        }
        Ok(next)
    }
}

fn assignment(s: &str, at: usize) -> Result<(String, &str, Option<crate::expr::Op>), String> {
    use crate::expr::Op;
    let s = s
        .trim()
        .strip_prefix('$')
        .ok_or_else(|| error(at, "a Yarn variable starts with $"))?;
    let end = s
        .char_indices()
        .find(|(_, c)| !c.is_alphanumeric() && *c != '_')
        .map_or(s.len(), |(i, _)| i);
    let name = &s[..end];
    if name.is_empty() || name.starts_with(|c: char| c.is_ascii_digit()) {
        return Err(error(at, "invalid variable name"));
    }
    let rest = s[end..].trim_start();
    for (token, op) in [
        ("+=", Some(Op::Add)),
        ("-=", Some(Op::Sub)),
        ("*=", Some(Op::Mul)),
        ("/=", Some(Op::Div)),
        ("%=", Some(Op::Rem)),
        ("=", None),
        ("to ", None),
    ] {
        if let Some(value) = rest.strip_prefix(token) {
            return Ok((name.into(), value.trim(), op));
        }
    }
    Err(error(at, "assignment needs =, to, +=, -=, *=, /= or %="))
}

pub fn compile(text: &str) -> Result<Dialogue, String> {
    compile_with_functions(text, BTreeMap::new())
}

/// Cook with engine-registered, typed pure functions. Their checked bodies are
/// part of the IR, so neither execution host can substitute a different callback.
pub fn compile_with_functions(
    text: &str,
    mut functions: BTreeMap<String, types::Function>,
) -> Result<Dialogue, String> {
    functions::validate(&mut functions)?;
    let mut option_indent = None;
    let mut empty_suite = None;
    for (index, raw) in text.lines().enumerate() {
        let line = without_comment(raw);
        let indent = line.len() - line.trim_start().len();
        if line.trim().is_empty() {
            if let Some(base) = option_indent
                && indent > base
            {
                empty_suite = Some((base, index + 1));
            }
            continue;
        }
        if let Some((base, at)) = empty_suite.take()
            && indent <= base
        {
            return Err(error(
                at,
                "indented lines following options must have content",
            ));
        }
        option_indent = line.trim_start().starts_with("->").then_some(indent);
    }
    let signatures = functions
        .iter()
        .map(|(n, f)| (n.clone(), f.signature.clone()))
        .collect();
    if text.trim_start().starts_with("type dialogue") {
        return Err(format!(
            "type dialogue .ogt was removed; convert to .yarn with #og-profile:{PROFILE}, title:/---/=== nodes and an explicit #line: id on every line and option"
        ));
    }
    let mut c = Compiler {
        dialogue: Dialogue {
            name: String::new(),
            start: String::new(),
            file_tags: Vec::new(),
            headers: BTreeMap::new(),
            speakers: Vec::new(),
            vars: Vec::new(),
            smart: BTreeMap::new(),
            functions,
            group_members: BTreeMap::new(),
            saliency_scores: BTreeMap::new(),
            untracked: BTreeSet::new(),
            nodes: Vec::new(),
            barks: Vec::new(),
            strings: BTreeMap::new(),
            locale: "en".into(),
        },
        ids: BTreeSet::new(),
        blocks: BTreeSet::new(),
        instructions: BTreeMap::new(),
        tags: BTreeMap::new(),
        shadows: BTreeMap::new(),
        title: String::new(),
        locals: BTreeSet::new(),
        once_lines: BTreeSet::new(),
        declarations: Vec::new(),
        types: types::Types {
            functions: signatures,
            ..Default::default()
        },
        bark: false,
    };
    let mut profile = None;
    let mut blank_before = false;
    let lines: Vec<_> = text
        .lines()
        .enumerate()
        .filter_map(|(i, s)| {
            if s.trim().is_empty() {
                blank_before = true;
                return None;
            }
            let s = without_comment(s.trim_start_matches('\u{feff}'));
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                return None;
            }
            Some(Source {
                text: trimmed.into(),
                indent: s.len() - s.trim_start().len(),
                line: i + 1,
                blank_before: std::mem::take(&mut blank_before),
            })
        })
        .collect();
    let mut at = 0;
    while let Some(src) = lines.get(at).filter(|s| s.text.starts_with('#')) {
        let (key, value) = src.text.split_once(':').unwrap_or((&src.text, ""));
        match key {
            "#og-profile" => profile = Some(value),
            "#og-name" => c.dialogue.name = value.into(),
            "#og-locale" => c.dialogue.locale = value.into(),
            "#og-local" => {
                c.locals.insert(value.into());
            }
            _ if key.starts_with("#og-") => {
                return Err(error(
                    src.line,
                    format!("unsupported engine file tag {key}"),
                ));
            }
            _ => c
                .dialogue
                .file_tags
                .push(src.text.trim_start_matches('#').into()),
        }
        at += 1;
    }
    if profile != Some(PROFILE) {
        return Err(format!(
            "declare #og-profile:{PROFILE}; unsupported or missing Yarn version profile"
        ));
    }
    if !identifier(&c.dialogue.name) {
        return Err("declare #og-name:dialogue_name".into());
    }
    if c.dialogue.locale.is_empty() {
        return Err("#og-locale needs a locale, such as en".into());
    }
    let mut titles = BTreeSet::new();
    let mut groups: BTreeMap<String, Vec<(String, Option<Expr>, f64)>> = BTreeMap::new();
    while at < lines.len() {
        let first = lines[at].line;
        let mut headers = BTreeMap::new();
        let mut whens = Vec::new();
        while let Some(src) = lines.get(at).filter(|s| s.text != "---") {
            let (key, value) = src
                .text
                .split_once(':')
                .ok_or_else(|| error(src.line, "expected title: NodeName or ---"))?;
            if key == "when" {
                if value.trim().is_empty() {
                    return Err(error(src.line, "when header needs an expression"));
                }
                whens.push(value.trim());
            } else if headers.insert(key, value.trim()).is_some() {
                return Err(error(src.line, format!("duplicate header {key}")));
            }
            at += 1;
        }
        let title = *headers
            .get("title")
            .ok_or_else(|| error(first, "node needs title: NodeName"))?;
        let grouped = !whens.is_empty();
        if !syntax::identifier(title) {
            return Err(error(first, "invalid or duplicate node title"));
        }
        if (grouped && titles.contains(title)) || (!grouped && groups.contains_key(title)) {
            return Err(error(
                first,
                "all nodes sharing a title must have when headers; node and group names must be unique",
            ));
        }
        let node_name = if grouped {
            let subtitle = if let Some(subtitle) = headers.get("subtitle") {
                if subtitle.is_empty() {
                    return Err(error(first, "invalid subtitle"));
                }
                (*subtitle).to_string()
            } else {
                let key = whens.join("\n");
                let hash = key.bytes().fold(0xcbf29ce484222325_u64, |h, b| {
                    (h ^ u64::from(b)).wrapping_mul(0x100000001b3)
                });
                let base = format!("{title}.when_{hash:016x}");
                let occurrence = c.instructions.entry(base).or_default();
                let subtitle = format!("when_{hash:016x}_{}", *occurrence);
                *occurrence += 1;
                subtitle
            };
            format!("{title}.{subtitle}")
        } else {
            title.to_string()
        };
        if !titles.insert(node_name.clone()) {
            return Err(error(first, "invalid or duplicate node title or subtitle"));
        }
        let mut metadata: BTreeMap<String, String> = headers
            .iter()
            .map(|(key, value)| ((*key).into(), (*value).into()))
            .collect();
        metadata.insert("title".into(), node_name.clone());
        c.dialogue.headers.insert(node_name.clone(), metadata);
        c.title = node_name.clone();
        let mut node_once = false;
        if grouped {
            let mut combined = None;
            let mut score = 0;
            for when in whens {
                let condition = if when == "always" {
                    Expr::Lit(Val::Bool(true))
                } else if when == "once" || when.starts_with("once if ") {
                    node_once = true;
                    let cond = once_condition(
                        &node_name,
                        when.strip_prefix("once").unwrap().trim(),
                        first,
                    )?;
                    score += saliency::complexity(&cond);
                    cond
                } else {
                    let cond = expression(when, first)?;
                    score += saliency::complexity(&cond);
                    cond
                };
                combined = Some(match combined {
                    None => condition,
                    Some(prior) => {
                        Expr::Bin(crate::expr::Op::Or, Box::new(prior), Box::new(condition))
                    }
                });
            }
            c.dialogue
                .group_members
                .insert(node_name.clone(), title.into());
            c.dialogue.saliency_scores.insert(node_name.clone(), score);
            groups
                .entry(title.into())
                .or_default()
                .push((node_name.clone(), combined, 1.0));
        }
        match headers.get("tracking").copied() {
            Some("never") => {
                c.dialogue.untracked.insert(node_name.clone());
            }
            None | Some("always") => {}
            Some(_) => return Err(error(first, "tracking must be always or never")),
        }
        if let Some(speaker) = headers.get("og-speaker") {
            let fields: Vec<_> = speaker.split('|').collect();
            let [id, name, portrait, side] = fields.as_slice() else {
                return Err(error(first, "og-speaker: id|name-key|portrait|side"));
            };
            if !matches!(*side, "left" | "right" | "center") || c.dialogue.speaker(id).is_some() {
                return Err(error(first, "invalid or duplicate og-speaker"));
            }
            c.dialogue.speakers.push(Speaker {
                id: (*id).into(),
                name: (*name).into(),
                portrait: (*portrait).into(),
                side: (*side).into(),
                expression: String::new(),
            });
        }
        if at == lines.len() {
            return Err(error(first, "node needs ---"));
        }
        at += 1;
        let start = at;
        while at < lines.len() && lines[at].text != "===" {
            at += 1;
        }
        if at == lines.len() {
            return Err(error(first, "node needs closing ==="));
        }
        c.bark = headers.contains_key("og-topic");
        let mut cursor = 0;
        let mut body = c.body(&lines[start..at], &mut cursor, 0, &node_name)?;
        if node_once {
            body.insert(
                0,
                Statement::Action(
                    format!("{node_name}/once"),
                    Step::MarkOnce(node_name.clone()),
                    first,
                ),
            );
        }
        if cursor != at - start {
            return Err(error(
                lines[start + cursor].line,
                "unexpected Yarn block terminator",
            ));
        }
        at += 1;
        if let Some(topic) = headers.get("og-topic") {
            let mut bark = Bark {
                id: title.into(),
                topic: (*topic).into(),
                speaker: String::new(),
                lines: Vec::new(),
                when: headers
                    .get("og-when")
                    .map(|s| Expr::parse(s).map_err(|e| error(first, e)))
                    .transpose()?,
                priority: 0,
                weight: 1.0,
                cooldown: 0,
                shared: 0,
                duration: 180,
                random: false,
                once: false,
                steps: Vec::new(),
                line: first,
            };
            for statement in body {
                match statement {
                    Statement::Line(line, None) => {
                        bark.lines.push(line);
                    }
                    Statement::Action(
                        _,
                        step @ (Step::Set(..)
                        | Step::Emit(..)
                        | Step::Command(..)
                        | Step::Call(..)
                        | Step::Quest(..)),
                        _,
                    ) => bark.steps.push(step),
                    _ => {
                        return Err(error(
                            first,
                            "unsupported flow control in bark; use og-when and scheduler headers",
                        ));
                    }
                }
            }
            for (key, dest) in [
                ("og-cooldown", &mut bark.cooldown),
                ("og-shared", &mut bark.shared),
                ("og-duration", &mut bark.duration),
            ] {
                if let Some(s) = headers.get(key) {
                    *dest = s
                        .parse()
                        .map_err(|_| error(first, format!("{key} needs integer ticks")))?;
                }
            }
            if let Some(s) = headers.get("og-priority") {
                bark.priority = s
                    .parse()
                    .map_err(|_| error(first, "og-priority needs an integer"))?;
            }
            if let Some(s) = headers.get("og-weight") {
                bark.weight = s
                    .parse()
                    .map_err(|_| error(first, "og-weight needs a number"))?;
            }
            bark.once = match headers.get("og-once").copied().unwrap_or("false") {
                "true" => true,
                "false" => false,
                _ => return Err(error(first, "og-once needs true or false")),
            };
            bark.random = match headers.get("og-order").copied().unwrap_or("sequence") {
                "random" => true,
                "sequence" => false,
                _ => return Err(error(first, "og-order needs random or sequence")),
            };
            if bark.lines.is_empty()
                || bark.duration == 0
                || !bark.weight.is_finite()
                || bark.weight <= 0.0
            {
                return Err(error(
                    first,
                    "bark needs lines, a positive duration and positive finite weight",
                ));
            }
            c.dialogue.barks.push(bark);
        } else {
            if let Some(key) = headers
                .keys()
                .find(|k| k.starts_with("og-") && **k != "og-speaker")
            {
                return Err(error(
                    first,
                    format!("{key} requires an og-topic bark node"),
                ));
            }
            if c.dialogue.start.is_empty() {
                c.dialogue.start = title.into();
            }
            let end = c.block(
                format!("{node_name}/return"),
                vec![Step::Return],
                Vec::new(),
                first,
            )?;
            let entry = c.lower(body, end)?;
            c.block(node_name, vec![Step::Goto(entry)], Vec::new(), first)?;
        }
    }
    for (group, members) in groups {
        c.title = group.clone();
        c.block(
            group,
            vec![Step::Select(members), Step::Return],
            Vec::new(),
            1,
        )?;
    }
    if c.dialogue.nodes.is_empty() && c.dialogue.barks.is_empty() {
        return Err("Yarn needs at least one node".into());
    }
    for (key, (source, at)) in &c.shadows {
        if c.shadows.contains_key(source) || !c.dialogue.strings.contains_key(source) {
            return Err(error(
                *at,
                format!("shadow line source #line:{source} does not exist"),
            ));
        }
        if c.dialogue.strings.get(key) != c.dialogue.strings.get(source) {
            return Err(error(
                *at,
                "shadow lines must be identical to their source line",
            ));
        }
    }
    // Continuation and option identities already use the explicit line ID.
    // Only the localized content key aliases the source; saves remain distinct.
    let resolve_shadow = |text: &mut Text| {
        if let Some((source, _)) = c.shadows.get(&text.key) {
            text.key = source.clone();
        }
    };
    for node in &mut c.dialogue.nodes {
        for step in &mut node.steps {
            if let Step::Say(line) = step {
                resolve_shadow(&mut line.text);
            }
        }
        for choice in &mut node.choices {
            resolve_shadow(&mut choice.text);
        }
    }
    for bark in &mut c.dialogue.barks {
        for line in &mut bark.lines {
            resolve_shadow(&mut line.text);
        }
    }
    for key in c.shadows.keys() {
        c.dialogue.strings.remove(key);
    }
    typing::check(&mut c.dialogue, c.declarations, &mut c.types, &c.locals)?;
    let mut tracked = c.types.tracked_nodes;
    for (node, headers) in &c.dialogue.headers {
        if headers
            .get("tracking")
            .is_some_and(|value| value == "always")
        {
            tracked.insert(node.clone());
            if let Some(group) = c.dialogue.group_members.get(node) {
                tracked.insert(group.clone());
            }
        }
    }
    for node in c.dialogue.headers.keys() {
        if !tracked.contains(node) {
            c.dialogue.untracked.insert(node.clone());
        }
    }
    for group in c.dialogue.group_members.values() {
        let all_never = c
            .dialogue
            .group_members
            .iter()
            .filter(|(_, g)| *g == group)
            .all(|(node, _)| {
                c.dialogue.headers[node]
                    .get("tracking")
                    .is_some_and(|v| v == "never")
            });
        if !tracked.contains(group) || all_never {
            c.dialogue.untracked.insert(group.clone());
        }
    }
    c.dialogue.validate()?;
    Ok(c.dialogue)
}

impl Dialogue {
    /// Serialized engine IR, not Yarn source. Only the cook invokes the compiler.
    pub fn encode(&self) -> Result<String, String> {
        serde_json::to_string(self).map_err(|e| e.to_string())
    }

    pub fn decode(ir: &str) -> Result<Self, String> {
        let mut dialogue: Self =
            serde_json::from_str(ir).map_err(|e| format!("invalid story IR: {e}"))?;
        functions::validate(&mut dialogue.functions)?;
        dialogue.validate()?;
        Ok(dialogue)
    }
}
