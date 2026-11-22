//! CSS `calc()`, `min()`, `max()` and `clamp()` for Taffy's calc lengths
//! (CSS Values 4 §10): an expression of lengths and percentages, resolved
//! when layout knows the basis a percentage is of (the containing block's
//! width or height), as the browser resolves it.
//!
//! Taffy keeps a calc length as an opaque handle ([`handle`]); the layout
//! store resolves it through [`resolve`]. Equal expressions share one
//! handle, so equal styles stay equal. The table only grows: a handle is
//! valid for the life of the process, and its expressions are the distinct
//! ones a program's styles write, which the style caches keep too.

use std::collections::HashMap;
use std::sync::{Mutex, OnceLock};

/// A length expression.
#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    /// `px + pct × basis` (`calc(50% - 2rem)`: px −32, pct 0.5).
    Sum { px: f32, pct: f32 },
    /// The sum of two expressions.
    Add(Box<[Expr; 2]>),
    /// An expression times a number.
    Scale(Box<Expr>, f32),
    /// `min(a, b, …)`.
    Min(Vec<Expr>),
    /// `max(a, b, …)`; `clamp(lo, v, hi)` is `max(lo, min(v, hi))`.
    Max(Vec<Expr>),
}

impl Expr {
    /// The expression's length for a percentage basis of `basis`.
    pub fn resolve(&self, basis: f32) -> f32 {
        match self {
            Expr::Sum { px, pct } => px + pct * basis,
            Expr::Add(b) => b[0].resolve(basis) + b[1].resolve(basis),
            Expr::Scale(e, k) => e.resolve(basis) * k,
            Expr::Min(v) => v
                .iter()
                .map(|e| e.resolve(basis))
                .fold(f32::INFINITY, f32::min),
            Expr::Max(v) => v
                .iter()
                .map(|e| e.resolve(basis))
                .fold(f32::NEG_INFINITY, f32::max),
        }
    }

    fn key(&self, out: &mut Vec<u32>) {
        match self {
            Expr::Sum { px, pct } => out.extend([0, px.to_bits(), pct.to_bits()]),
            Expr::Add(b) => {
                out.push(1);
                b.iter().for_each(|e| e.key(out));
            }
            Expr::Scale(e, k) => {
                out.extend([2, k.to_bits()]);
                e.key(out);
            }
            Expr::Min(v) | Expr::Max(v) => {
                out.extend([
                    if matches!(self, Expr::Min(_)) { 3 } else { 4 },
                    v.len() as u32,
                ]);
                v.iter().for_each(|e| e.key(out));
            }
        }
    }
}

#[derive(Default)]
struct Table {
    exprs: Vec<Expr>,
    by_key: HashMap<Vec<u32>, u32>,
}

fn table() -> &'static Mutex<Table> {
    static TABLE: OnceLock<Mutex<Table>> = OnceLock::new();
    TABLE.get_or_init(Mutex::default)
}

/// Taffy's handle for `expr` (`CompactLength::calc`): its index in the
/// table, past the three tag bits and never 0. Equal expressions get the
/// same handle.
pub fn handle(expr: Expr) -> *const () {
    let mut key = Vec::new();
    expr.key(&mut key);
    let mut t = table().lock().unwrap_or_else(|e| e.into_inner());
    let n = t.exprs.len() as u32;
    let index = *t.by_key.entry(key).or_insert(n);
    if index == n {
        t.exprs.push(expr);
    }
    ((index as usize + 1) << 3) as *const ()
}

/// The handle of `handle`'s expression plus `px` (a safe-area inset added
/// to a side written with `calc()`).
pub fn plus(handle: *const (), px: f32) -> *const () {
    let index = ((handle as usize) >> 3).wrapping_sub(1);
    let e = table()
        .lock()
        .unwrap_or_else(|e| e.into_inner())
        .exprs
        .get(index)
        .cloned();
    let e = e.unwrap_or(Expr::Sum { px: 0.0, pct: 0.0 });
    self::handle(Expr::Add(Box::new([e, Expr::Sum { px, pct: 0.0 }])))
}

/// The length a handle stands for at `basis`.
pub fn resolve(handle: *const (), basis: f32) -> f32 {
    let index = ((handle as usize) >> 3).wrapping_sub(1);
    let t = table().lock().unwrap_or_else(|e| e.into_inner());
    t.exprs.get(index).map_or(0.0, |e| e.resolve(basis))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_expressions_share_a_handle_and_resolve_against_the_basis() {
        let half = || Expr::Sum {
            px: -32.0,
            pct: 0.5,
        };
        assert_eq!(handle(half()), handle(half()));
        assert_eq!(resolve(handle(half()), 200.0), 68.0);
        let px = |px| Expr::Sum { px, pct: 0.0 };
        let clamp = handle(Expr::Max(vec![
            px(10.0),
            Expr::Min(vec![Expr::Sum { px: 0.0, pct: 0.5 }, px(80.0)]),
        ]));
        assert_eq!(
            [0.0, 100.0, 400.0].map(|w| resolve(clamp, w)),
            [10.0, 50.0, 80.0]
        );
        assert_eq!(resolve(plus(clamp, 4.0), 100.0), 54.0);
    }
}
