//! Yarn v3.2.2's built-in policies, plus the engine's weighted policy.
//! Selection semantics ported from Yarn/Saliency (MIT; third_party/yarn-spinner).
use crate::expr::{Expr, Op};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Saliency {
    First,
    Best,
    BestLeastRecentlyViewed,
    RandomBestLeastRecentlyViewed,
    #[default]
    ConditionalFirstWeighted,
}

pub(crate) fn complexity(expr: &Expr) -> u32 {
    fn operators(expr: &Expr) -> u32 {
        match expr {
            Expr::Bin(op, a, b) => {
                u32::from(matches!(op, Op::And | Op::Or | Op::Xor)) + operators(a) + operators(b)
            }
            Expr::Yarn(e) | Expr::Not(e) | Expr::Neg(e) => operators(e),
            Expr::Invoke(_, args) => args.iter().map(operators).sum(),
            _ => 0,
        }
    }
    1 + operators(expr)
}
