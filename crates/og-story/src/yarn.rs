//! Yarn 3 authoring compiled to engine-owned, stable-ID continuation blocks.
//! This is an original Rust implementation; see third_party/yarn-spinner/.

use crate::asset::*;
use crate::expr::{Expr, Val};
use std::collections::{BTreeMap, BTreeSet};

pub const PROFILE: &str = "yarn-3.0-og-1";

#[derive(Clone)]
struct Source {
    text: String,
    indent: usize,
    line: usize,
}

enum Statement {
    Line(Line, Option<Expr>),
    Action(String, Step, usize),
    Detour(String, String, usize),
    Branch(String, Vec<(Expr, Vec<Statement>)>, Vec<Statement>, usize),
    Options(Vec<(Line, Option<Expr>, f64, Vec<Statement>)>, bool, usize),
}

struct Compiler {
    dialogue: Dialogue,
    ids: BTreeSet<String>,
    blocks: BTreeSet<String>,
    tags: BTreeMap<String, BTreeMap<String, String>>,
    title: String,
    locals: BTreeSet<String>,
}

fn error(line: usize, message: impl std::fmt::Display) -> String {
    format!("line {line}: {message}")
}

fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-'))
}

/// Translate only expression tokens, never dollars or operators inside strings.
fn expression(s: &str, line: usize) -> Result<Expr, String> {
    let mut text = String::new();
    let mut quote = None;
    let mut escaped = false;
    for c in s.chars() {
        if escaped {
            text.push(c);
            escaped = false;
        } else if quote.is_some() && c == '\\' {
            text.push(c);
            escaped = true;
        } else if quote == Some(c) {
            text.push(c);
            quote = None;
        } else if quote.is_none() && matches!(c, '\'' | '"') {
            quote = Some(c);
            text.push(c);
        } else if quote.is_some() || c != '$' {
            text.push(c);
        }
    }
    Expr::parse(&text)
        .map_err(|e| error(line, format!("unsupported or invalid Yarn expression: {e}")))
}

fn command(s: &str) -> Option<&str> {
    s.strip_prefix("<<")?.strip_suffix(">>").map(str::trim)
}

fn split_tags(s: &str, line: usize) -> Result<(&str, BTreeMap<&str, &str>), String> {
    let at = s.find(" #").unwrap_or(s.len());
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
        for tag in tags.keys() {
            if !matches!(
                *tag,
                "line"
                    | "weight"
                    | "voice"
                    | "expression"
                    | "shot"
                    | "check"
                    | "fail"
                    | "hide"
                    | "kind"
                    | "timer"
                    | "reason"
                    | "reason-arg"
            ) {
                return Err(error(
                    at,
                    format!("unsupported Yarn tag #{tag} in {PROFILE}"),
                ));
            }
        }
        self.tags.insert(
            key.clone(),
            tags.iter()
                .map(|(k, v)| ((*k).into(), (*v).into()))
                .collect(),
        );
        let cond = if let Some((prefix, suffix)) = text.rsplit_once("<<if ") {
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
        if text.contains("<<") || text.contains("[[") || text.contains('[') {
            return Err(error(
                at,
                "unsupported Yarn line condition, legacy option or markup",
            ));
        }
        let (speaker, text) = match text.split_once(": ") {
            Some((s, t)) if identifier(s) => (s, t),
            _ => (NARRATOR, text),
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
        let mut rest = text;
        while let Some((before, after)) = rest.split_once('{') {
            base.push_str(before);
            let (expr, tail) = after
                .split_once('}')
                .ok_or_else(|| error(at, "close the inline expression with }"))?;
            let simple = expr.trim().strip_prefix('$').unwrap_or(expr.trim());
            let name = if identifier(simple) {
                simple.rsplit('.').next().unwrap().to_string()
            } else {
                format!("arg{}", args.len())
            };
            let parsed = expression(expr, at)?;
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
            rest = tail;
        }
        if rest.contains('}') {
            return Err(error(at, "unmatched } in Yarn line"));
        }
        base.push_str(rest);
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
            if matches!(command(text), Some("else" | "endif")) || text.starts_with("<<elseif ") {
                break;
            }
            if text.starts_with("-> ") || text.starts_with("=> ") {
                let random = text.starts_with("=>");
                let prefix = if random { "=> " } else { "-> " };
                let group_indent = src.indent;
                let group_line = src.line;
                let mut options = Vec::new();
                while let Some(src) = lines
                    .get(*at)
                    .filter(|s| s.indent == group_indent && s.text.starts_with(prefix))
                {
                    let (line, cond, weight) = self.line(&src.text[3..], src.line)?;
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
                }
                out.push(Statement::Options(options, random, group_line));
                continue;
            }
            *at += 1;
            if let Some(cmd) = command(text) {
                if let Some(cond) = cmd.strip_prefix("if ") {
                    let id = format!("{owner}/if:{cond}");
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
                } else if let Some(decl) = cmd.strip_prefix("declare ") {
                    let (name, value) = assignment(decl, src.line)?;
                    let value = match expression(value, src.line)? {
                        Expr::Lit(value) => value,
                        Expr::Neg(x) => match *x {
                            Expr::Lit(Val::Num(n)) => Val::Num(-n),
                            _ => {
                                return Err(error(
                                    src.line,
                                    "unsupported smart variable declaration; use a literal default",
                                ));
                            }
                        },
                        _ => {
                            return Err(error(
                                src.line,
                                "unsupported smart variable declaration; use a literal default",
                            ));
                        }
                    };
                    if self.dialogue.vars.iter().any(|v| v.name == name) {
                        return Err(error(src.line, format!("duplicate declaration ${name}")));
                    }
                    let global = !self.locals.contains(&name);
                    self.dialogue.vars.push(VarDecl {
                        name,
                        value,
                        global,
                    });
                } else {
                    // The command text is a semantic identity, not its bytecode/source position.
                    // Repeated identical commands belong in distinct authored nodes.
                    let id = format!("{owner}/command:{text}");
                    if let Some(target) = cmd.strip_prefix("detour ") {
                        if !identifier(target) {
                            return Err(error(src.line, "unsupported dynamic detour"));
                        }
                        out.push(Statement::Detour(id, target.into(), src.line));
                    } else {
                        let step = if let Some(value) = cmd.strip_prefix("set ") {
                            let (name, expr) = assignment(value, src.line)?;
                            Step::Set(name, expression(expr, src.line)?)
                        } else if let Some(target) = cmd.strip_prefix("jump ") {
                            if !identifier(target) {
                                return Err(error(src.line, "unsupported dynamic jump"));
                            }
                            Step::Goto(target.into())
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
                            let (name, arg) = cmd.split_once(' ').unwrap_or((cmd, ""));
                            if !identifier(name) {
                                return Err(error(src.line, "invalid command name"));
                            }
                            let arg = if arg.is_empty() {
                                None
                            } else if let Some(expr) =
                                arg.strip_prefix('{').and_then(|s| s.strip_suffix('}'))
                            {
                                Some(expression(expr, src.line)?)
                            } else if arg.contains(['{', '}']) {
                                return Err(error(
                                    src.line,
                                    "unsupported mixed command interpolation; pass one {expression}",
                                ));
                            } else {
                                Some(Expr::Lit(Val::Str(arg.into())))
                            };
                            Step::Emit(name.into(), arg)
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
            return Err(error(
                at,
                format!(
                    "duplicate instruction identity '{id}'; put repeated commands in distinct nodes"
                ),
            ));
        }
        self.dialogue.nodes.push(Node {
            id: id.clone(),
            title: self.title.clone(),
            lines: vec![at; steps.len()],
            steps,
            choices,
            timer: None,
            choice_line: None,
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
                    let line_step = steps.len();
                    steps.push(Step::Say(line));
                    let mut pending = Vec::new();
                    let mut cursor = next.as_str();
                    let mut visited = BTreeSet::new();
                    let mut choices = Vec::new();
                    let mut timer = None;
                    while visited.insert(cursor.to_string()) {
                        let Some(node) = self.dialogue.node(cursor) else {
                            break;
                        };
                        if !node.choices.is_empty() {
                            choices = node.choices.clone();
                            timer = node.timer.clone();
                            break;
                        }
                        match node.steps.as_slice() {
                            [
                                action @ (Step::Set(..) | Step::Emit(..) | Step::Quest(..)),
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
                    self.block(id, vec![Step::Detour(target, resume)], Vec::new(), at)?
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
                    for (line, cond, weight, body) in options {
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
                    let steps = if random {
                        vec![Step::Select(picks), Step::Goto(next)]
                    } else {
                        Vec::new()
                    };
                    let id = self.block(id, steps, choices, at)?;
                    self.dialogue.nodes.last_mut().unwrap().timer = timer;
                    id
                }
            };
        }
        Ok(next)
    }
}

fn assignment(s: &str, at: usize) -> Result<(String, &str), String> {
    let (name, value) = s
        .split_once(" = ")
        .or_else(|| s.split_once(" to "))
        .ok_or_else(|| error(at, "unsupported assignment; write $variable = expression"))?;
    let name = name
        .strip_prefix('$')
        .filter(|n| {
            n.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_')
                && n.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
        })
        .ok_or_else(|| {
            error(
                at,
                "a Yarn variable starts with $ and uses letters, digits and underscores",
            )
        })?;
    Ok((name.into(), value))
}

pub fn compile(text: &str) -> Result<Dialogue, String> {
    if text.trim_start().starts_with("type dialogue") {
        return Err("type dialogue .ogt was removed; convert to .yarn with #og-profile:yarn-3.0-og-1, title:/---/=== nodes and an explicit #line: id on every line and option".into());
    }
    let mut c = Compiler {
        dialogue: Dialogue {
            name: String::new(),
            start: String::new(),
            speakers: Vec::new(),
            vars: Vec::new(),
            nodes: Vec::new(),
            barks: Vec::new(),
            strings: BTreeMap::new(),
            locale: "en".into(),
        },
        ids: BTreeSet::new(),
        blocks: BTreeSet::new(),
        tags: BTreeMap::new(),
        title: String::new(),
        locals: BTreeSet::new(),
    };
    let mut profile = None;
    let lines: Vec<_> = text
        .lines()
        .enumerate()
        .filter_map(|(i, s)| {
            let trimmed = s.trim();
            if trimmed.is_empty() || trimmed.starts_with("//") {
                return None;
            }
            Some(Source {
                text: trimmed.into(),
                indent: s.len() - s.trim_start().len(),
                line: i + 1,
            })
        })
        .collect();
    let mut at = 0;
    while let Some(src) = lines.get(at).filter(|s| s.text.starts_with('#')) {
        let (key, value) = src
            .text
            .split_once(':')
            .ok_or_else(|| error(src.line, "file tag needs a value"))?;
        match key {
            "#og-profile" => profile = Some(value),
            "#og-name" => c.dialogue.name = value.into(),
            "#og-locale" => c.dialogue.locale = value.into(),
            "#og-local" => {
                c.locals.insert(value.into());
            }
            _ => return Err(error(src.line, format!("unsupported Yarn file tag {key}"))),
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
    while at < lines.len() {
        let first = lines[at].line;
        let mut headers = BTreeMap::new();
        while let Some(src) = lines.get(at).filter(|s| s.text != "---") {
            let (key, value) = src
                .text
                .split_once(':')
                .ok_or_else(|| error(src.line, "expected title: NodeName or ---"))?;
            if !matches!(
                key,
                "title"
                    | "tags"
                    | "og-speaker"
                    | "og-topic"
                    | "og-cooldown"
                    | "og-shared"
                    | "og-priority"
                    | "og-duration"
                    | "og-weight"
                    | "og-once"
                    | "og-order"
                    | "og-when"
            ) {
                return Err(error(
                    src.line,
                    format!("unsupported Yarn header/feature '{key}'"),
                ));
            }
            if headers.insert(key, value.trim()).is_some() {
                return Err(error(src.line, format!("duplicate header {key}")));
            }
            at += 1;
        }
        let title = *headers
            .get("title")
            .ok_or_else(|| error(first, "node needs title: NodeName"))?;
        c.title = title.into();
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
        if !identifier(title) || !titles.insert(title.to_string()) {
            return Err(error(
                first,
                "invalid or duplicate node title (node groups are unsupported)",
            ));
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
        let mut cursor = 0;
        let body = c.body(&lines[start..at], &mut cursor, 0, title)?;
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
                    .map(|s| expression(s, first))
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
                        step @ (Step::Set(..) | Step::Emit(..) | Step::Quest(..)),
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
                format!("{title}/return"),
                vec![Step::Return],
                Vec::new(),
                first,
            )?;
            let entry = c.lower(body, end)?;
            c.block(title.into(), vec![Step::Goto(entry)], Vec::new(), first)?;
        }
    }
    if c.dialogue.nodes.is_empty() && c.dialogue.barks.is_empty() {
        return Err("Yarn needs at least one node".into());
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
        let dialogue: Self =
            serde_json::from_str(ir).map_err(|e| format!("invalid story IR: {e}"))?;
        dialogue.validate()?;
        Ok(dialogue)
    }
}
