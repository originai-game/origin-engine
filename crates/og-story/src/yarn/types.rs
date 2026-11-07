//! Cook-time types. The same signatures accompany registered function bodies.
use crate::expr::{Expr, Func, Op, Val};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Type {
    Number,
    String,
    Bool,
    Enum(String),
    /// Only a function parameter may accept any Yarn value.
    Any,
}

impl Type {
    pub fn of(value: &Val) -> Self {
        match value {
            Val::Num(_) => Self::Number,
            Val::Bool(_) => Self::Bool,
            Val::Str(_) => Self::String,
        }
    }

    pub fn named(name: &str) -> Self {
        match name {
            "number" => Self::Number,
            "string" => Self::String,
            "bool" => Self::Bool,
            name => Self::Enum(name.into()),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Signature {
    pub parameters: Vec<Type>,
    pub variadic: Option<Type>,
    pub returns: Type,
}

/// Signatures from Yarn v3.2.2 Dialogue.StandardLibrary (MIT; third_party/yarn-spinner).
pub fn builtin(name: &str) -> Option<Signature> {
    use Type::*;
    let (parameters, returns) = match name {
        "random" => (vec![], Number),
        "random_range" | "random_range_float" | "min" | "max" | "round_places" => {
            (vec![Number, Number], Number)
        }
        "dice" | "round" | "floor" | "ceil" | "inc" | "dec" | "decimal" | "int" => {
            (vec![Number], Number)
        }
        "string" => (vec![Any], String),
        "format_invariant" => (vec![Number], String),
        "number" => (vec![Any], Number),
        "bool" => (vec![Any], Bool),
        "format" => (vec![String, Any], String),
        "visited" | "has_any_content" => (vec![String], Bool),
        "visited_count" => (vec![String], Number),
        _ => return None,
    };
    Some(Signature {
        parameters,
        variadic: None,
        returns,
    })
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Enum {
    pub raw: Type,
    pub members: BTreeMap<String, Val>,
}

/// Registered functions have no host callbacks. Their typed Yarn expressions
/// can read only parameters and call other pure registered functions.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub signature: Signature,
    pub parameters: Vec<String>,
    pub body: Expr,
}

#[derive(Default)]
pub struct Types {
    pub variables: BTreeMap<String, Type>,
    pub enums: BTreeMap<String, Enum>,
    pub functions: BTreeMap<String, Signature>,
    pub tracked_nodes: BTreeSet<String>,
}

impl Types {
    fn compatible(&self, found: &Type, expected: &Type) -> bool {
        found == expected
            || *expected == Type::Any
            || match (found, expected) {
                (Type::Enum(name), Type::String) => {
                    self.enums.get(name).is_some_and(|e| e.raw == Type::String)
                }
                (Type::String, Type::Enum(name)) => {
                    self.enums.get(name).is_some_and(|e| e.raw == Type::String)
                }
                _ => false,
            }
    }

    pub fn bind(&mut self, name: &str, ty: &Type) -> Result<(), String> {
        if *ty == Type::Any {
            return Ok(());
        }
        match self.variables.get(name) {
            Some(prior) if prior != ty => Err(format!(
                "${name} has conflicting types {prior:?} and {ty:?}"
            )),
            Some(_) => Ok(()),
            None => {
                self.variables.insert(name.into(), ty.clone());
                Ok(())
            }
        }
    }

    /// Propagates constraints; None means another expression must supply a type.
    pub fn infer(
        &mut self,
        expr: &mut Expr,
        expected: Option<&Type>,
    ) -> Result<Option<Type>, String> {
        let expected = expected.filter(|t| **t != Type::Any);
        let ty = match expr {
            Expr::Once(_) => Some(Type::Bool),
            Expr::Yarn(inner) => self.infer(inner, expected)?,
            Expr::Lit(value) => Some(Type::of(value)),
            Expr::EnumValue(name, _) => Some(Type::Enum(name.clone())),
            Expr::Member(name, member) => {
                let candidates: Vec<_> = self
                    .enums
                    .iter()
                    .filter(|(n, e)| {
                        (name.is_empty() || *n == name)
                            && (!name.is_empty()
                                || expected.is_none_or(
                                    |t| !matches!(t, Type::Enum(wanted) if wanted != *n),
                                ))
                            && e.members.contains_key(member)
                    })
                    .collect();
                match candidates.as_slice() {
                    [(name, enumeration)] => {
                        let ty = Type::Enum((*name).clone());
                        *expr =
                            Expr::EnumValue((*name).clone(), enumeration.members[member].clone());
                        Some(ty)
                    }
                    [] => return Err(format!("unknown enum member {name}.{member}")),
                    _ => None,
                }
            }
            Expr::Var(name) => {
                if !self.variables.contains_key(name)
                    && let Some(ty) = expected
                {
                    self.bind(name, ty)?;
                }
                self.variables.get(name).cloned()
            }
            Expr::Not(inner) => {
                self.infer(inner, Some(&Type::Bool))?;
                Some(Type::Bool)
            }
            Expr::Neg(inner) => {
                self.infer(inner, Some(&Type::Number))?;
                Some(Type::Number)
            }
            Expr::Bin(op, a, b) => {
                let operand = match op {
                    Op::And | Op::Or | Op::Xor => Some(Type::Bool),
                    Op::Sub | Op::Mul | Op::Div | Op::Rem | Op::Lt | Op::Le | Op::Gt | Op::Ge => {
                        Some(Type::Number)
                    }
                    Op::Add => expected.cloned(),
                    Op::Eq | Op::Ne => None,
                };
                let left = self.infer(a, operand.as_ref())?;
                let right = self.infer(b, operand.as_ref().or(left.as_ref()))?;
                let left = self.infer(a, operand.as_ref().or(right.as_ref()))?;
                if let (Some(left), Some(right)) = (&left, &right)
                    && !self.compatible(left, right)
                {
                    return Err(format!(
                        "operator {op:?} cannot combine {left:?} and {right:?}"
                    ));
                }
                match op {
                    Op::Add => {
                        if left
                            .as_ref()
                            .is_some_and(|t| !matches!(t, Type::Number | Type::String))
                        {
                            return Err("addition needs two numbers or two strings".into());
                        }
                        left
                    }
                    Op::Sub | Op::Mul | Op::Div | Op::Rem => Some(Type::Number),
                    _ => Some(Type::Bool),
                }
            }
            Expr::Invoke(name, args) => {
                if matches!(name.as_str(), "visited" | "visited_count")
                    && let Some(Expr::Lit(Val::Str(node))) = args.first()
                {
                    self.tracked_nodes.insert(node.clone());
                }
                let signature = builtin(name).or_else(|| self.functions.get(name).cloned())
                    .ok_or_else(|| format!("function '{name}' must be registered with a typed signature before cook"))?;
                if args.len() < signature.parameters.len()
                    || (signature.variadic.is_none() && args.len() != signature.parameters.len())
                {
                    return Err(format!(
                        "function '{name}' expects {}{} arguments, got {}",
                        signature.parameters.len(),
                        if signature.variadic.is_some() {
                            " or more"
                        } else {
                            ""
                        },
                        args.len()
                    ));
                }
                for (i, arg) in args.iter_mut().enumerate() {
                    self.infer(
                        arg,
                        signature.parameters.get(i).or(signature.variadic.as_ref()),
                    )?;
                }
                Some(signature.returns)
            }
            Expr::Call(function) => Some(match function {
                Func::Seen(_) => Type::Bool,
                Func::Status(_) | Func::Stage(_) => Type::String,
                Func::Count(..) => Type::Number,
            }),
        };
        if let (Some(found), Some(expected)) = (&ty, expected)
            && !self.compatible(found, expected)
        {
            return Err(format!("expected {expected:?}, found {found:?}"));
        }
        Ok(ty)
    }

    pub fn require_resolved(&self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Var(v) if !v.contains('.') && !self.variables.contains_key(v) => Err(format!(
                "cannot infer type of ${v}; declare it or supply a typed use"
            )),
            Expr::Member(ty, member) => Err(format!(
                "cannot resolve enum member {ty}.{member}; specify its type"
            )),
            Expr::Yarn(e) | Expr::Not(e) | Expr::Neg(e) => self.require_resolved(e),
            Expr::Bin(_, a, b) => {
                self.require_resolved(a)?;
                self.require_resolved(b)
            }
            Expr::Invoke(_, args) => args.iter().try_for_each(|a| self.require_resolved(a)),
            _ => Ok(()),
        }
    }
}
