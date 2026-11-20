//! The update queue: the game's stores as NSH signals (third_party/
//! nsh-reactive, Solid 1.9's scheduling), each binding a render effect that
//! reads the signals its expression names and pushes a typed [`Op`] when its
//! value changed. A tree drains its ops once per frame and patches only the
//! arena nodes they name; a frame with no writes runs nothing and drains an
//! empty queue, so it allocates nothing.

use nsh_reactive::{Equals, Graph, Host, Kind};
use og_ui::bind::{Env, Expr, Scope};
use serde_json::Value;
use std::cell::UnsafeCell;
use std::rc::Rc;

/// What a binding changed, for the tree that owns it.
#[derive(Clone, Debug, PartialEq)]
pub enum Op {
    /// A text leaf's text.
    Text {
        tree: u32,
        node: u32,
        text: Box<str>,
    },
    /// A compiled style condition of an element ([`crate::doc::DocStyle::conditions`]).
    Holds {
        tree: u32,
        node: u32,
        cond: u32,
        on: bool,
    },
    /// An `{#if}` block's branch.
    Shown { tree: u32, node: u32, on: bool },
    /// A `{#each}` block's keys changed: its rows are made again.
    Rebuild { tree: u32, node: u32 },
}

impl Op {
    fn tree(&self) -> u32 {
        match self {
            Op::Text { tree, .. }
            | Op::Holds { tree, .. }
            | Op::Shown { tree, .. }
            | Op::Rebuild { tree, .. } => *tree,
        }
    }
}

/// How a binding's value becomes an op.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    Text {
        node: u32,
    },
    Holds {
        node: u32,
        cond: u32,
    },
    Shown {
        node: u32,
    },
    /// Its value is a list's keys; a change after the first run rebuilds.
    Rebuild {
        node: u32,
    },
}

type Func = Rc<dyn Fn(Value) -> Option<Value>>;

#[derive(Default)]
pub struct Data {
    ops: Vec<Op>,
    runaway: bool,
}

#[derive(Clone, Copy)]
pub struct Ui;

thread_local! {
    static GRAPH: UnsafeCell<Graph<Ui>> = UnsafeCell::new(Graph::new(Data::default()));
}

// SAFETY: one graph per thread, reached only through these functions, each
// borrow ending before the next call into the host.
unsafe impl Host for Ui {
    type Value = Value;
    type Func = Func;
    type Equals = fn(&Value, &Value) -> bool;
    type Cleanup = Box<dyn FnOnce()>;
    type Context = ();
    type Data = Data;

    unsafe fn graph<'a>(self) -> &'a mut Graph<Self> {
        // SAFETY: the thread's graph lives as long as the thread.
        GRAPH.with(|g| unsafe { &mut *g.get() })
    }
    fn empty() -> Value {
        Value::Null
    }
    fn dup(self, v: &Value) -> Value {
        v.clone()
    }
    fn free(self, _: Value) {}
    fn dup_func(self, f: &Func) -> Func {
        f.clone()
    }
    fn free_func(self, _: Func) {}
    fn dup_equals(self, e: &Self::Equals) -> Self::Equals {
        *e
    }
    fn free_equals(self, _: Self::Equals) {}
    fn call(self, f: &Func, prev: Value) -> Option<Value> {
        f(prev)
    }
    fn same(self, a: &Value, b: &Value) -> bool {
        a == b
    }
    fn truthy(self, v: &Value) -> bool {
        og_ui::bind::truthy(v)
    }
    fn custom_changed(self, e: &Self::Equals, prev: Value, next: &Value) -> bool {
        !e(&prev, next)
    }
    fn run_cleanup(self, f: Box<dyn FnOnce()>) {
        f();
    }
    fn free_cleanup(self, _: Box<dyn FnOnce()>) {}
    fn runaway(self) {
        data().runaway = true;
    }
}

fn data<'a>() -> &'a mut Data {
    // SAFETY: a short borrow, released before any call into the graph.
    unsafe { &mut Ui.graph().data }
}

/// A store: a signal holding its value.
pub fn signal(value: Value) -> u32 {
    // SAFETY: a short borrow.
    unsafe { Ui.graph() }.signal(value, Equals::Default)
}

/// Writes a store; an equal value notifies nothing. The bindings that read
/// it run before this returns and queue their ops.
pub fn set(signal: u32, value: Value) {
    nsh_reactive::run_updates(Ui, false, || nsh_reactive::write(Ui, signal, value));
}

/// A store's value, untracked.
pub fn get(signal: u32) -> Value {
    nsh_reactive::untracked(Ui, || nsh_reactive::read(Ui, signal))
}

/// What a binding reads: `name` in its expression is `signal`'s value.
pub type Reads = Vec<(Box<str>, u32)>;

/// A binding of `tree`: `expr` over the signals it reads, run now and again
/// whenever one of them changes; it queues `target`'s op when its value
/// changed (and on its first run). Answers its effect's id, to dispose.
pub fn bind(tree: u32, target: Target, expr: Expr, reads: Reads, env: Rc<dyn Env>) -> u32 {
    // Its last value: an effect is not handed back what it returned.
    let last: std::cell::RefCell<Option<Value>> = Default::default();
    let func: Func = Rc::new(move |_| {
        let mut vm = serde_json::Map::new();
        for (name, s) in &reads {
            vm.insert(name.to_string(), nsh_reactive::read(Ui, *s));
        }
        let vm = Value::Object(vm);
        let scope = Scope {
            vm: &vm,
            locals: Vec::new(),
            env: &*env,
        };
        let next = expr.eval(&scope);
        let changed = match (&*last.borrow(), target) {
            (None, _) => true,
            (Some(prev), Target::Holds { .. } | Target::Shown { .. }) => {
                og_ui::bind::truthy(&next) != og_ui::bind::truthy(prev)
            }
            (Some(prev), Target::Text { .. } | Target::Rebuild { .. }) => next != *prev,
        };
        let first = last.borrow().is_none();
        if changed && !(first && matches!(target, Target::Rebuild { .. })) {
            let on = og_ui::bind::truthy(&next);
            data().ops.push(match target {
                Target::Text { node } => Op::Text {
                    tree,
                    node,
                    text: text_of(&next).into(),
                },
                Target::Holds { node, cond } => Op::Holds {
                    tree,
                    node,
                    cond,
                    on,
                },
                Target::Shown { node } => Op::Shown { tree, node, on },
                Target::Rebuild { node } => Op::Rebuild { tree, node },
            });
        }
        if changed {
            *last.borrow_mut() = Some(next);
        }
        Some(Value::Null)
    });
    nsh_reactive::run_updates(Ui, false, || {
        nsh_reactive::effect(Ui, Kind::Effect, func, Value::Null)
    })
}

/// A value computed by `f` from the signals it reads (a component's prop,
/// a list's row), re-run when one of them changes; an equal result
/// notifies nothing. Read it as a signal.
pub fn derive(f: impl Fn() -> Value + 'static) -> u32 {
    let func: Func = Rc::new(move |_| Some(f()));
    nsh_reactive::run_updates(Ui, false, || {
        nsh_reactive::memo(Ui, func, Value::Null, Equals::Default, false)
    })
}

/// A signal's value, tracked by the computation running it.
pub fn read(signal: u32) -> Value {
    nsh_reactive::read(Ui, signal)
}

/// Runs `f` under a new owner and answers it: what `f` makes is disposed
/// with [`dispose`] of it (a list's row).
pub fn owned(f: impl FnOnce()) -> u32 {
    // SAFETY: short borrows.
    let root = unsafe { Ui.graph() }.root();
    let saved = unsafe { Ui.graph() }.enter_owner(root);
    f();
    unsafe { Ui.graph() }.leave_owner(saved);
    root
}

/// Disposes a node and what it owns.
pub fn dispose(id: u32) {
    nsh_reactive::dispose(Ui, id);
}

/// Stops a binding (its tree's node is gone).
pub fn unbind(effect: u32) {
    nsh_reactive::dispose(Ui, effect);
}

/// A value as text shows it.
pub fn text_of(v: &Value) -> String {
    match v {
        Value::Null => String::new(),
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        v => v.to_string(),
    }
}

/// Moves `tree`'s queued ops into `out`, in the order they were queued.
/// With none queued this touches no memory beyond the queue's length.
pub fn drain(tree: u32, out: &mut Vec<Op>) {
    let ops = &mut data().ops;
    if ops.is_empty() {
        return;
    }
    let mut k = 0;
    while k < ops.len() {
        if ops[k].tree() == tree {
            out.push(ops.remove(k));
        } else {
            k += 1;
        }
    }
}

/// A flush ran away since the last call (a binding kept changing what it
/// reads).
pub fn ran_away() -> bool {
    std::mem::take(&mut data().runaway)
}

#[cfg(test)]
mod tests {
    use super::*;
    use og_ui::bind::NoEnv;

    fn path(p: &str) -> Expr {
        Expr::Path(p.into())
    }

    #[test]
    fn a_write_queues_only_the_bindings_that_read_it_and_an_equal_write_none() {
        let env: Rc<dyn Env> = Rc::new(NoEnv);
        let clock = signal("06:54".into());
        let sky = signal("Clear".into());
        bind(
            7,
            Target::Text { node: 3 },
            path("$clock"),
            vec![("$clock".into(), clock)],
            env.clone(),
        );
        bind(
            7,
            Target::Text { node: 5 },
            path("$sky"),
            vec![("$sky".into(), sky)],
            env.clone(),
        );
        bind(
            8,
            Target::Shown { node: 1 },
            path("$sky"),
            vec![("$sky".into(), sky)],
            env,
        );
        let mut ops = Vec::new();
        drain(7, &mut ops);
        assert_eq!(ops.len(), 2, "each binding's first value");
        ops.clear();
        drain(8, &mut ops);
        assert_eq!(
            ops,
            [Op::Shown {
                tree: 8,
                node: 1,
                on: true
            }],
            "another tree's ops stay its own"
        );
        ops.clear();
        set(clock, "06:54".into());
        drain(7, &mut ops);
        assert!(ops.is_empty(), "the same value");
        set(clock, "06:55".into());
        set(sky, "Rain".into());
        drain(7, &mut ops);
        assert_eq!(
            ops,
            [
                Op::Text {
                    tree: 7,
                    node: 3,
                    text: "06:55".into()
                },
                Op::Text {
                    tree: 7,
                    node: 5,
                    text: "Rain".into()
                },
            ]
        );
        ops.clear();
        drain(8, &mut ops);
        assert!(ops.is_empty(), "a condition whose truth did not change");
        assert_eq!(get(sky), Value::from("Rain"));
    }
}
