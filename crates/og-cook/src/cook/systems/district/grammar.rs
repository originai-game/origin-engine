//! Checked author IR becomes native shape rules, never generated source text.
use crate::cook::source_schemas::world::grammar as input;
use og_shape::generators::grammar as native;

pub(crate) fn declaration(
    input: input::GrammarInput,
    line: impl Fn(&str) -> usize,
) -> Result<native::Grammar, String> {
    let grammar = native::Grammar {
        params: input.params,
        start: input.start,
        rules: input
            .rules
            .into_iter()
            .map(|(name, ops)| {
                let rule = native::Rule {
                    line: line(&name),
                    name: name.clone(),
                    ops: ops.into_iter().map(operation).collect(),
                };
                (name, rule)
            })
            .collect(),
    };
    native::validate(&grammar)?;
    Ok(grammar)
}

fn number(n: input::Number) -> native::Num {
    match n {
        input::Number::Literal(n) => native::Num::Lit(n),
        input::Number::Parameter(p) => native::Num::Param(p),
    }
}

fn operation(op: input::Operation) -> native::Op {
    use input::{Condition, Operation as I, Roof};
    use native::{Cond, Op};
    match op {
        I::Recess { depth, jamb } => Op::Recess {
            depth: number(depth),
            jamb,
        },
        I::Detail { level } => Op::Detail(level),
        I::Label { label } => Op::Label(label),
        I::Insert { model } => Op::Insert(model.id),
        I::Drop => Op::Drop,
        I::Extrude { depth, faces } => Op::Extrude {
            depth: number(depth),
            faces: native::Faces {
                top: faces.top,
                bottom: faces.bottom,
                front: faces.front,
                back: faces.back,
                side: faces.side,
                sides: faces.sides,
            },
        },
        I::Split { axis, parts } => Op::Split {
            axis: match axis {
                input::Axis::X => native::Axis::X,
                input::Axis::Y => native::Axis::Y,
            },
            parts: parts
                .into_iter()
                .map(|p| native::SplitPart {
                    size: match p.size {
                        input::Size::Abs => native::Size::Abs,
                        input::Size::Fill => native::Size::Fill,
                        input::Size::Repeat => native::Size::Repeat,
                    },
                    length: number(p.length),
                    rule: p.rule,
                })
                .collect(),
        },
        I::Inset {
            width,
            border,
            inner,
        } => Op::Inset {
            width: number(width),
            border,
            inner,
        },
        I::Roof {
            roof,
            slope,
            gable,
            eave,
            overhang,
        } => {
            let (kind, angle) = match roof {
                Roof::Flat => (native::RoofKind::Flat, native::Num::Lit(0.0)),
                Roof::Gable { angle } => (native::RoofKind::Gable, number(angle)),
                Roof::Hip { angle } => (native::RoofKind::Hip, number(angle)),
            };
            Op::Roof {
                kind,
                angle,
                slope,
                gable,
                eave,
                overhang: overhang.map_or(native::Num::Lit(0.0), number),
            }
        }
        I::Pick { options } => Op::Pick(options.into_iter().map(|o| (o.weight, o.rule)).collect()),
        I::Case { arms, otherwise } => Op::Case {
            arms: arms
                .into_iter()
                .map(|a| {
                    let condition = match a.condition {
                        Condition::Wider { value } => Cond::Wider(number(value)),
                        Condition::Taller { value } => Cond::Taller(number(value)),
                        Condition::Param { name, value } => Cond::Param(name, number(value)),
                    };
                    (condition, a.rule)
                })
                .collect(),
            otherwise,
        },
    }
}
