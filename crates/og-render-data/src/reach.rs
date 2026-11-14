//! What a surface takes of the shared shading ([`Capability`]): the cell
//! of og-render's capability table (`tests/shader_capabilities.rs`), read
//! from a module's text and the code its entry points reach (naga's call
//! graph). The table and `og check`'s findings on a game's material
//! shaders both read it here; the `reach` feature builds it, never the
//! runtime.

use crate::{Capability, Holds, MaterialShader, ShaderMode};
use naga::Statement as St;
use std::collections::BTreeSet;

/// How a module's surface carries a capability: a cell of the table.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Cell {
    /// The surface takes it.
    Full,
    /// Its code is in the module and no entry point reaches it.
    Idle,
    /// Its stub: a call compiles and does nothing.
    Stub,
    /// Neither.
    None,
}

impl Cell {
    pub fn name(self) -> &'static str {
        match self {
            Cell::Full => "full",
            Cell::Idle => "idle",
            Cell::Stub => "stub",
            Cell::None => "-",
        }
    }
}

/// A module's text and the functions its entry points reach.
pub struct Reach<'a> {
    wgsl: &'a str,
    reached: BTreeSet<String>,
}

impl<'a> Reach<'a> {
    /// Parses `wgsl` and walks its call graph from the entry points. A
    /// branch a constant shuts (`if (!CUSTOM_LIT) { return ...; }`) is not
    /// followed, nor what follows a return every path takes.
    pub fn new(wgsl: &'a str) -> Result<Self, String> {
        let module = naga::front::wgsl::parse_str(wgsl).map_err(|e| e.emit_to_string(wgsl))?;
        let mut seen = BTreeSet::new();
        let mut open = Vec::new();
        for ep in &module.entry_points {
            walk(&module, &ep.function, &ep.function.body, &mut open);
        }
        while let Some(f) = open.pop() {
            if seen.insert(f) {
                let function = &module.functions[f];
                walk(&module, function, &function.body, &mut open);
            }
        }
        let reached = seen
            .into_iter()
            .filter_map(|f| module.functions[f].name.clone())
            .collect();
        Ok(Reach { wgsl, reached })
    }

    /// The cell of `c`. The two a material shader's hooks may call
    /// ([`Capability::HOOKS_CALL`]) are full where the module holds them,
    /// reached or not: they are there for the hooks.
    pub fn cell(&self, c: Capability) -> Cell {
        match c.held_by(self.wgsl) {
            Holds::Full
                if !Capability::HOOKS_CALL.contains(&c) && !self.reached.contains(c.function()) =>
            {
                Cell::Idle
            }
            Holds::Full => Cell::Full,
            Holds::Stub => Cell::Stub,
            Holds::None => Cell::None,
        }
    }
}

/// The value of `e` when it is a constant boolean (a `const`, a literal,
/// or one negated).
fn constant(
    module: &naga::Module,
    exprs: &naga::Arena<naga::Expression>,
    e: naga::Handle<naga::Expression>,
) -> Option<bool> {
    use naga::{Expression as E, Literal, UnaryOperator};
    let global = |h: naga::Handle<naga::Expression>| match module.global_expressions[h] {
        E::Literal(Literal::Bool(b)) => Some(b),
        _ => None,
    };
    match exprs[e] {
        E::Literal(Literal::Bool(b)) => Some(b),
        E::Constant(c) => global(module.constants[c].init),
        E::Unary {
            op: UnaryOperator::LogicalNot,
            expr,
        } => constant(module, exprs, expr).map(|b| !b),
        _ => None,
    }
}

/// Queues every function `block` calls on the paths it can take; true
/// when every path through it returns.
fn walk(
    module: &naga::Module,
    function: &naga::Function,
    block: &naga::Block,
    open: &mut Vec<naga::Handle<naga::Function>>,
) -> bool {
    for s in block.iter() {
        let returns = match s {
            St::Call { function, .. } => {
                open.push(*function);
                false
            }
            St::Return { .. } | St::Kill => true,
            St::Block(b) => walk(module, function, b, open),
            St::If {
                condition,
                accept,
                reject,
            } => match constant(module, &function.expressions, *condition) {
                Some(true) => walk(module, function, accept, open),
                Some(false) => walk(module, function, reject, open),
                None => {
                    let a = walk(module, function, accept, open);
                    let r = walk(module, function, reject, open);
                    a && r
                }
            },
            St::Switch { cases, .. } => {
                for c in cases {
                    walk(module, function, &c.body, open);
                }
                false
            }
            St::Loop {
                body, continuing, ..
            } => {
                walk(module, function, body, open);
                walk(module, function, continuing, open);
                false
            }
            _ => false,
        };
        if returns {
            return true;
        }
    }
    false
}

/// The capabilities a game sees ([`Capability::seen_by_a_game`]) that the
/// surface of a mesh drawn with `shader` does not take, in
/// [`Capability::ALL`] order: its lit module as the renderer builds it with
/// the planar reflections' effect group (the most a lit module carries),
/// read cell by cell as the table reads the built-in ones. Empty for a
/// sprite shader, which no mesh runs.
pub fn material_shader_lacks(shader: &MaterialShader) -> Result<Vec<Capability>, String> {
    if shader.mode == ShaderMode::Sprite {
        return Ok(Vec::new());
    }
    let (lit, _) = crate::custom::module_sources(shader, true);
    let reach = Reach::new(&lit)?;
    Ok(Capability::ALL
        .into_iter()
        .filter(|&c| c.seen_by_a_game() && reach.cell(c) != Cell::Full)
        .collect())
}
