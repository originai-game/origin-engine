use super::types::{Type, Types};
use crate::asset::{Dialogue, QuestOp, Step, Text, VarDecl};
use crate::expr::{Expr, Val};
use std::collections::{BTreeMap, BTreeSet};

pub(super) type Declaration = (String, Expr, Option<Type>, usize);

pub(super) fn literal(expr: &Expr) -> Option<Val> {
    match expr {
        Expr::Yarn(e) => literal(e),
        Expr::Lit(v) | Expr::EnumValue(_, v) => Some(v.clone()),
        Expr::Neg(e) => match literal(e)? {
            Val::Num(n) => Some(Val::Num(-n)),
            _ => None,
        },
        _ => None,
    }
}

fn expression(
    types: &mut Types,
    e: &mut Expr,
    expected: Option<&Type>,
    final_pass: bool,
) -> Result<(), String> {
    types.infer(e, expected)?;
    if final_pass {
        types.require_resolved(e)?;
    }
    Ok(())
}

fn text(types: &mut Types, text: &mut Text, final_pass: bool) -> Result<(), String> {
    for (_, e) in &mut text.args {
        expression(types, e, None, final_pass)?;
    }
    Ok(())
}

fn steps(types: &mut Types, steps: &mut [Step], final_pass: bool) -> Result<(), String> {
    for step in steps {
        match step {
            Step::Say(line) => text(types, &mut line.text, final_pass)?,
            Step::Set(name, e) => {
                let expected = types.variables.get(name).cloned();
                if let Some(found) = types.infer(e, expected.as_ref())? {
                    types.bind(name, &found)?;
                }
                if final_pass {
                    types.require_resolved(e)?;
                }
            }
            Step::If(e, _) => expression(types, e, Some(&Type::Bool), final_pass)?,
            Step::Jump(e) | Step::DynamicDetour(e, _) => {
                expression(types, e, Some(&Type::String), final_pass)?;
            }
            Step::Select(picks) => {
                for (_, e, _) in picks {
                    if let Some(e) = e {
                        expression(types, e, Some(&Type::Bool), final_pass)?;
                    }
                }
            }
            Step::Call(e) | Step::Emit(_, Some(e)) => expression(types, e, None, final_pass)?,
            Step::Command(e) => expression(types, e, Some(&Type::String), final_pass)?,
            Step::Quest(QuestOp::Progress(_, _, e)) => {
                expression(types, e, Some(&Type::Number), final_pass)?
            }
            _ => {}
        }
    }
    Ok(())
}

fn program(types: &mut Types, dialogue: &mut Dialogue, final_pass: bool) -> Result<(), String> {
    for node in &mut dialogue.nodes {
        steps(types, &mut node.steps, final_pass)?;
        for choice in &mut node.choices {
            text(types, &mut choice.text, final_pass)?;
            if let Some(e) = &mut choice.cond {
                expression(types, e, Some(&Type::Bool), final_pass)?;
            }
            if let Some(reason) = &mut choice.reason {
                text(types, reason, final_pass)?;
            }
            if let Some(check) = &mut choice.check {
                types.bind(&check.stat, &Type::Number)?;
                expression(types, &mut check.target, Some(&Type::Number), final_pass)?;
            }
            steps(types, &mut choice.steps, final_pass)?;
        }
    }
    for bark in &mut dialogue.barks {
        for line in &mut bark.lines {
            text(types, &mut line.text, final_pass)?;
        }
        steps(types, &mut bark.steps, final_pass)?;
    }
    Ok(())
}

fn acyclic(
    name: &str,
    smart: &BTreeMap<String, Expr>,
    active: &mut BTreeSet<String>,
    done: &mut BTreeSet<String>,
) -> Result<(), String> {
    if done.contains(name) {
        return Ok(());
    }
    if !active.insert(name.into()) {
        return Err(format!("smart variable cycle through ${name}"));
    }
    let mut reads = Vec::new();
    smart[name].vars(&mut reads);
    for name in reads {
        if smart.contains_key(&name) {
            acyclic(&name, smart, active, done)?;
        }
    }
    active.remove(name);
    done.insert(name.into());
    Ok(())
}

pub(super) fn check(
    dialogue: &mut Dialogue,
    mut declarations: Vec<Declaration>,
    types: &mut Types,
    locals: &BTreeSet<String>,
) -> Result<(), String> {
    for (name, _, declared, _) in &declarations {
        if let Some(ty) = declared {
            types.bind(name, ty)?;
        }
    }
    loop {
        let before = types.variables.len();
        for (name, expr, declared, line) in &mut declarations {
            let expected = declared
                .as_ref()
                .or_else(|| types.variables.get(name))
                .cloned();
            let ty = types
                .infer(expr, expected.as_ref())
                .map_err(|e| super::error(*line, e))?;
            if let Some(ty) = ty {
                types.bind(name, &ty)?;
            }
        }
        program(types, dialogue, false)?;
        if before == types.variables.len() {
            break;
        }
    }
    program(types, dialogue, true)?;
    for (name, expr, _, line) in declarations {
        types
            .require_resolved(&expr)
            .map_err(|e| super::error(line, e))?;
        if let Some(value) = literal(&expr) {
            dialogue.vars.push(VarDecl {
                global: !locals.contains(&name),
                name,
                value,
            });
        } else {
            dialogue.smart.insert(name, expr);
        }
    }
    for (name, ty) in &types.variables {
        if name.contains('.')
            || dialogue.vars.iter().any(|v| &v.name == name)
            || dialogue.smart.contains_key(name)
        {
            continue;
        }
        let value = match ty {
            Type::Number => Val::Num(0.0),
            Type::String => Val::Str(String::new()),
            Type::Bool => Val::Bool(false),
            Type::Enum(name) => types
                .enums
                .get(name)
                .and_then(|e| e.members.values().next())
                .cloned()
                .ok_or_else(|| format!("unknown or empty enum {name}"))?,
            Type::Any => return Err(format!("cannot infer type of ${name}")),
        };
        dialogue.vars.push(VarDecl {
            global: !locals.contains(name),
            name: name.clone(),
            value,
        });
    }
    let mut done = BTreeSet::new();
    for name in dialogue.smart.keys() {
        acyclic(name, &dialogue.smart, &mut BTreeSet::new(), &mut done)?;
    }
    for node in &dialogue.nodes {
        for step in node
            .steps
            .iter()
            .chain(node.choices.iter().flat_map(|c| &c.steps))
        {
            if let Step::Set(name, _) = step
                && dialogue.smart.contains_key(name)
            {
                return Err(format!("cannot assign to smart variable ${name}"));
            }
        }
    }
    Ok(())
}
