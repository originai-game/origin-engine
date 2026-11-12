//! Every loop ends in the modules the renderer makes without wgpu's forced
//! loop bounding (`og_render::UNBOUNDED_MODULES`): that is sound only for
//! loops that end, and an endless loop in MSL is undefined behaviour the
//! Metal compiler may use to drop the bounds checks around it.
//!
//! A loop passes when naga's IR shows it is counted: its head leaves on a
//! comparison of a local with a bound the loop cannot change, and the
//! loop's only write to that local is one constant step toward the bound in
//! its `continuing` block. A bound at the end of the counter's range (`i <=
//! n`, a step above one, a float counter) must be capped by a constant in
//! the shader. Any other loop is named in [`ENDS`] with why it ends; a new
//! one fails here until its author adds that line.

use naga::{BinaryOperator as Op, Expression as Ex, Handle, Statement as St};
use std::collections::{BTreeMap, HashMap, HashSet};

/// Functions with a loop that is not counted, and why each one ends.
const ENDS: &[(&str, &str)] = &[
    ("bayer", "`size` doubles from 1 to `n`, which is 2, 4 or 8"),
    ("conformed", "r, g and b count to `hi`, at most 255 >> 4"),
    (
        "resident_from",
        "`lvl` starts at or below `top` and steps by one to it; a layout has 1 to 16 levels",
    ),
];

type Local = Handle<naga::LocalVariable>;

/// Calls `f` with every statement of a block, nested ones too.
fn each<'a>(block: &'a naga::Block, f: &mut dyn FnMut(&'a St)) {
    for s in block.iter() {
        f(s);
        match s {
            St::Block(b) => each(b, f),
            St::If { accept, reject, .. } => {
                each(accept, f);
                each(reject, f);
            }
            St::Switch { cases, .. } => cases.iter().for_each(|c| each(&c.body, f)),
            St::Loop {
                body, continuing, ..
            } => {
                each(body, f);
                each(continuing, f);
            }
            _ => {}
        }
    }
}

/// The local a pointer expression points into.
fn pointee(func: &naga::Function, mut pointer: Handle<Ex>) -> Option<Local> {
    loop {
        match func.expressions[pointer] {
            Ex::Access { base, .. } | Ex::AccessIndex { base, .. } => pointer = base,
            Ex::LocalVariable(v) => return Some(v),
            _ => return None,
        }
    }
}

/// The local an expression loads.
fn loaded(func: &naga::Function, e: Handle<Ex>) -> Option<Local> {
    match func.expressions[e] {
        Ex::Load { pointer } => match func.expressions[pointer] {
            Ex::LocalVariable(v) => Some(v),
            _ => None,
        },
        _ => None,
    }
}

/// A literal's or a constant's number.
fn number(module: &naga::Module, func: &naga::Function, e: Handle<Ex>) -> Option<f64> {
    let literal = |l: naga::Literal| match l {
        naga::Literal::I32(n) => Some(f64::from(n)),
        naga::Literal::U32(n) => Some(f64::from(n)),
        naga::Literal::F32(n) => Some(f64::from(n)),
        _ => None,
    };
    match func.expressions[e] {
        Ex::Literal(l) => literal(l),
        Ex::Constant(c) => match module.global_expressions[module.constants[c].init] {
            Ex::Literal(l) => literal(l),
            _ => None,
        },
        _ => None,
    }
}

/// A constant the expression cannot pass, above (`upper`) or below: its own
/// value, or the one a `min`, `max` or `clamp` holds it to.
fn cap(module: &naga::Module, func: &naga::Function, e: Handle<Ex>, upper: bool) -> Option<f64> {
    use naga::MathFunction as M;
    if let Some(n) = number(module, func, e) {
        return Some(n);
    }
    let cap = |e| cap(module, func, e, upper);
    match func.expressions[e] {
        Ex::Math {
            fun: M::Min,
            arg,
            arg1: Some(other),
            ..
        } if upper => match (cap(arg), cap(other)) {
            (Some(a), Some(b)) => Some(a.min(b)),
            (a, b) => a.or(b),
        },
        Ex::Math {
            fun: M::Max,
            arg,
            arg1: Some(other),
            ..
        } if !upper => match (cap(arg), cap(other)) {
            (Some(a), Some(b)) => Some(a.max(b)),
            (a, b) => a.or(b),
        },
        Ex::Math {
            fun: M::Clamp,
            arg1: Some(low),
            arg2: Some(high),
            ..
        } => cap(if upper { high } else { low }),
        Ex::As {
            expr,
            convert: Some(_),
            ..
        } => cap(expr),
        _ => None,
    }
}

/// One loop of a function.
struct Loop<'a> {
    module: &'a naga::Module,
    func: &'a naga::Function,
    body: &'a naga::Block,
    continuing: &'a naga::Block,
    /// The locals the loop stores to or passes to a call by pointer, once a
    /// statement.
    written: Vec<Local>,
    /// The expressions evaluated inside the loop.
    inside: HashSet<Handle<Ex>>,
}

impl<'a> Loop<'a> {
    fn new(
        module: &'a naga::Module,
        func: &'a naga::Function,
        body: &'a naga::Block,
        continuing: &'a naga::Block,
    ) -> Loop<'a> {
        let mut written = Vec::new();
        let mut inside = HashSet::new();
        for block in [body, continuing] {
            each(block, &mut |s| match s {
                St::Emit(range) => inside.extend(range.clone()),
                St::Store { pointer, .. } => written.extend(pointee(func, *pointer)),
                St::Call {
                    arguments, result, ..
                } => {
                    written.extend(arguments.iter().filter_map(|a| pointee(func, *a)));
                    inside.extend(*result);
                }
                St::Atomic { result, .. } => inside.extend(*result),
                St::WorkGroupUniformLoad { result, .. } => {
                    inside.insert(*result);
                }
                _ => {}
            });
        }
        Loop {
            module,
            func,
            body,
            continuing,
            written,
            inside,
        }
    }

    /// Why the loop is not counted.
    fn uncounted(&self) -> Option<String> {
        let func = self.func;
        // The head a `for` or a `while` lowers to: its condition, then
        // `if (condition) {} else { break }`. `a && b` is naga's short
        // circuit before it, `if (a) { t = b } else { t = false }`.
        let mut and = HashMap::new();
        let mut condition = None;
        for s in self.body.iter() {
            match s {
                St::Emit(_) => {}
                St::If {
                    condition: c,
                    accept,
                    reject,
                } if accept.is_empty() && matches!(reject[..], [St::Break]) => {
                    condition = Some(*c);
                    break;
                }
                St::If {
                    condition: a,
                    accept,
                    reject,
                } => {
                    let (
                        Some(St::Store { pointer, value: b }),
                        [
                            St::Store {
                                pointer: same,
                                value: no,
                            },
                        ],
                    ) = (accept.last(), &reject[..])
                    else {
                        break;
                    };
                    let (Ex::LocalVariable(t), Ex::Literal(naga::Literal::Bool(false))) =
                        (&func.expressions[*pointer], &func.expressions[*no])
                    else {
                        break;
                    };
                    if pointer != same {
                        break;
                    }
                    and.insert(*t, [*a, *b]);
                }
                _ => break,
            }
        }
        let Some(condition) = condition else {
            return Some("no `if (..) break` at its head".into());
        };
        let mut why = Vec::new();
        let mut terms = vec![condition];
        while let Some(e) = terms.pop() {
            match loaded(func, e).and_then(|t| and.get(&t)) {
                Some(both) => terms.extend(both),
                None => match self.counts(e) {
                    Ok(()) => return None,
                    Err(e) => why.push(e),
                },
            }
        }
        Some(why.join("; "))
    }

    /// Whether one term of the head's condition counts the loop out.
    fn counts(&self, term: Handle<Ex>) -> Result<(), String> {
        let (module, func) = (self.module, self.func);
        let Ex::Binary { op, left, right } = func.expressions[term] else {
            return Err("a condition that is not a comparison".into());
        };
        // The counter on the left: `n > i` is `i < n`.
        let (op, v, bound) = match (loaded(func, left), loaded(func, right)) {
            (Some(v), _) => (op, v, right),
            (None, Some(v)) => {
                let op = match op {
                    Op::Less => Op::Greater,
                    Op::LessEqual => Op::GreaterEqual,
                    Op::Greater => Op::Less,
                    Op::GreaterEqual => Op::LessEqual,
                    other => other,
                };
                (op, v, left)
            }
            _ => return Err("a comparison with no local on either side".into()),
        };
        let (up, strict) = match op {
            Op::Less => (true, true),
            Op::LessEqual => (true, false),
            Op::Greater => (false, true),
            Op::GreaterEqual => (false, false),
            other => return Err(format!("a condition with {other:?}")),
        };
        let name = func.local_variables[v].name.clone().unwrap_or_default();
        let kind = match module.types[func.local_variables[v].ty].inner {
            naga::TypeInner::Scalar(s) if s.width == 4 => s.kind,
            _ => return Err(format!("`{name}` is not a scalar")),
        };
        let steps: Vec<_> = self
            .continuing
            .iter()
            .filter_map(|s| match s {
                St::Store { pointer, value }
                    if matches!(func.expressions[*pointer], Ex::LocalVariable(w) if w == v) =>
                {
                    Some(*value)
                }
                _ => None,
            })
            .collect();
        let stores = self.written.iter().filter(|w| **w == v).count();
        let (&[step], 1) = (&steps[..], stores) else {
            return Err(format!(
                "`{name}` is written {stores} times in the loop, not once in `continuing`"
            ));
        };
        let Ex::Binary {
            op: by,
            left,
            right,
        } = func.expressions[step]
        else {
            return Err(format!("`{name}` does not step by a constant"));
        };
        let size = number(module, func, right).filter(|n| *n > 0.0);
        let (Some(size), true) = (size, loaded(func, left) == Some(v)) else {
            return Err(format!("`{name}` does not step by a constant"));
        };
        if let Some(reason) = self.varies(bound) {
            return Err(format!("the bound of `{name}` {reason}"));
        }
        use naga::ScalarKind as K;
        // A halving unsigned counter reaches zero, below any bound.
        if (by, up, strict, kind) == (Op::ShiftRight, false, true, K::Uint) {
            return Ok(());
        }
        if by != if up { Op::Add } else { Op::Subtract } {
            return Err(format!("`{name}` steps by {by:?}, away from its bound"));
        }
        // An integer counting by one to a bound it must pass cannot wrap first.
        if strict && size == 1.0 && kind != K::Float {
            return Ok(());
        }
        // Otherwise the bound must leave the counter room to pass it: below
        // the type's limit by a step, and for a float where a step still
        // changes the counter.
        let (low, high) = match kind {
            K::Sint => (f64::from(i32::MIN), f64::from(i32::MAX)),
            K::Uint => (0.0, f64::from(u32::MAX)),
            K::Float => {
                let exact = 16_777_216.0 * size.min(1.0);
                (-exact, exact)
            }
            _ => return Err(format!("`{name}` is not a number")),
        };
        let fits = match cap(module, func, bound, up) {
            Some(c) if up => c + size <= high,
            Some(c) => c - size >= low,
            None => false,
        };
        if fits {
            Ok(())
        } else {
            Err(format!(
                "`{name}` ({kind:?}, step {size}, {op:?}) needs a bound a constant caps"
            ))
        }
    }

    /// Why an expression may change between iterations, or `None` when the
    /// loop cannot change it: evaluated before the loop, or reading only
    /// constants, arguments, read-only bindings and locals the loop leaves
    /// alone.
    fn varies(&self, e: Handle<Ex>) -> Option<String> {
        let (module, func) = (self.module, self.func);
        let all = |es: &[Handle<Ex>]| es.iter().find_map(|e| self.varies(*e));
        let expression = &func.expressions[e];
        if expression.needs_pre_emit() {
            return match expression {
                Ex::LocalVariable(v) => self
                    .written
                    .contains(v)
                    .then(|| "reads a local the loop writes".to_string()),
                Ex::GlobalVariable(g) => match module.global_variables[*g].space {
                    naga::AddressSpace::Uniform | naga::AddressSpace::Handle => None,
                    naga::AddressSpace::Storage { access }
                        if !access.contains(naga::StorageAccess::STORE) =>
                    {
                        None
                    }
                    space => Some(format!("reads a writable global ({space:?})")),
                },
                _ => None,
            };
        }
        if !self.inside.contains(&e) {
            return None;
        }
        match expression {
            Ex::Load { pointer } => match func.expressions[*pointer] {
                Ex::FunctionArgument(_) => Some("reads through a pointer argument".to_string()),
                _ => all(&[*pointer]),
            },
            Ex::Access { base, index } => all(&[*base, *index]),
            Ex::AccessIndex { base, .. } => all(&[*base]),
            Ex::Binary { left, right, .. } => all(&[*left, *right]),
            Ex::Unary { expr, .. } | Ex::As { expr, .. } => all(&[*expr]),
            Ex::Splat { value, .. } => all(&[*value]),
            Ex::Swizzle { vector, .. } => all(&[*vector]),
            Ex::Compose { components, .. } => all(components),
            Ex::Select {
                condition,
                accept,
                reject,
            } => all(&[*condition, *accept, *reject]),
            Ex::Math {
                arg,
                arg1,
                arg2,
                arg3,
                ..
            } => all(&[Some(*arg), *arg1, *arg2, *arg3]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>()),
            Ex::ArrayLength(_) => None,
            Ex::CallResult(_) => Some("is a call's result inside the loop".to_string()),
            other => Some(format!("is a {other:?}")),
        }
    }
}

#[test]
fn every_loop_ends() {
    // Function, then why its loop is not counted and one module that has it.
    let mut uncounted = BTreeMap::<String, (String, String)>::new();
    let mut loops = 0;
    let mut found = HashSet::new();
    for s in og_render::shader_sources() {
        // A variant's label adds its name in brackets.
        let base = s.label.split(" (").next().unwrap_or_default();
        let Some(listed) = og_render::UNBOUNDED_MODULES.iter().find(|m| **m == base) else {
            continue;
        };
        found.insert(*listed);
        let (module, _) = super::shaders::checked(
            &s.label,
            &s.wgsl,
            &s.constants,
            super::shaders::full_capabilities(),
        )
        .unwrap_or_else(|e| panic!("{e}"));
        let functions = module
            .functions
            .iter()
            .map(|(_, f)| f)
            .chain(module.entry_points.iter().map(|e| &e.function));
        for func in functions {
            each(&func.body, &mut |statement| {
                let St::Loop {
                    body, continuing, ..
                } = statement
                else {
                    return;
                };
                loops += 1;
                if let Some(why) = Loop::new(&module, func, body, continuing).uncounted() {
                    uncounted
                        .entry(func.name.clone().unwrap_or_default())
                        .or_insert((why, s.label.clone()));
                }
            });
        }
    }
    assert_eq!(
        found.len(),
        og_render::UNBOUNDED_MODULES.len(),
        "an unbounded module is not in shader_sources: {found:?}"
    );
    assert!(loops > 100, "{loops} loops");
    let unknown: Vec<String> = uncounted
        .iter()
        .filter(|(name, _)| !ENDS.iter().any(|(f, _)| f == name))
        .map(|(name, (why, label))| format!("{name} ({label}): {why}"))
        .collect();
    assert!(
        unknown.is_empty(),
        "{} of {loops} loops are neither counted nor named in ENDS:\n{}",
        unknown.len(),
        unknown.join("\n")
    );
    let stale: Vec<&str> = ENDS
        .iter()
        .map(|(f, _)| *f)
        .filter(|f| !uncounted.contains_key(*f))
        .collect();
    assert!(stale.is_empty(), "ENDS names counted loops: {stale:?}");
}
