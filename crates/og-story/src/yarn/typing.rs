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
    Ok(())
}

/// Shared native invariant: typed TS and imported Yarn must not recurse while
/// reading smart variables, or assign to their read-only expressions.
pub(crate) fn validate_smart(dialogue: &Dialogue) -> Result<(), String> {
    let mut done = BTreeSet::new();
    for name in dialogue.smart.keys() {
        acyclic(name, &dialogue.smart, &mut BTreeSet::new(), &mut done)?;
    }
    for step in dialogue
        .nodes
        .iter()
        .flat_map(|node| {
            node.steps
                .iter()
                .chain(node.choices.iter().flat_map(|c| &c.steps))
        })
        .chain(dialogue.barks.iter().flat_map(|b| &b.steps))
    {
        if let Step::Set(name, _) = step
            && dialogue.smart.contains_key(name)
        {
            return Err(format!("cannot assign to smart variable ${name}"));
        }
    }
    Ok(())
}

/// Check structured declarations with the same signatures and expression
/// constraints as the importer, without Yarn's source grammar or wrappers.
pub(crate) fn check_native(dialogue: &mut Dialogue) -> Result<(), String> {
    let mut types = Types {
        variables: dialogue
            .vars
            .iter()
            .map(|v| (v.name.clone(), Type::of(&v.value)))
            .collect(),
        functions: dialogue
            .functions
            .iter()
            .map(|(n, f)| (n.clone(), f.signature.clone()))
            .collect(),
        ..Default::default()
    };
    loop {
        let before = types.variables.len();
        for (name, expr) in &mut dialogue.smart {
            if let Some(ty) = types.infer(expr, None)? {
                types.bind(name, &ty)?;
            }
        }
        if before == types.variables.len() {
            break;
        }
    }
    for expr in dialogue.smart.values() {
        types.require_resolved(expr)?;
    }
    program(&mut types, dialogue, true)?;
    for bark in &mut dialogue.barks {
        if let Some(condition) = &mut bark.when {
            expression(&mut types, condition, Some(&Type::Bool), true)?;
        }
    }
    dialogue.validate()
}

/// Quest conditions use the same expression signatures as dialogue. World
/// reads in event filters acquire their type from the surrounding expression.
pub(crate) fn check_quest(quest: &crate::asset::Quest) -> Result<(), String> {
    let types = || Types {
        variables: quest
            .vars
            .iter()
            .map(|v| (v.name.clone(), Type::of(&v.value)))
            .collect(),
        ..Default::default()
    };
    for stage in &quest.stages {
        for condition in stage.fail.iter().chain(
            stage
                .objectives
                .iter()
                .flat_map(|o| o.when.iter().chain(&o.filter)),
        ) {
            expression(
                &mut types(),
                &mut condition.clone(),
                Some(&Type::Bool),
                true,
            )?;
        }
        for objective in &stage.objectives {
            text(&mut types(), &mut objective.text.clone(), true)?;
        }
    }
    Ok(())
}
