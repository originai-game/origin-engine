//! The render graph: passes that declare which named targets they read and
//! write, ordered by that data flow, with the transient targets' lifetimes
//! and their physical textures (aliased when lifetimes do not overlap).
//!
//! A [`RenderGraph`] is plain data: declare targets ([`RenderGraph::transient`]
//! for textures the frame allocates, [`RenderGraph::import`] for ones that
//! outlive it: the output, shadow maps, TAA history), then passes
//! ([`RenderGraph::pass`]), then [`RenderGraph::compile`]. The
//! [`Renderer`](crate::Renderer) builds one per frame shape from the passes
//! its features turn on, and records its passes in the compiled order.
//!
//! **What a read binds to.** A pass reading a target depends on the last pass
//! declared before it that writes the target; when none was declared before
//! it, on the first later one (a graph may be declared out of order). A
//! write depends on the target's previous writer (writes keep declaration
//! order) and on every pass that read the previous contents, so nothing
//! overwrites a target before its readers ran. A pass that writes a target
//! it does not read (a clear) still waits for the previous writer: the
//! renderer's passes load what earlier passes drew (sprites over the scene,
//! bloom levels added back).
//!
//! **Validation** refuses, with the names involved: a target declared twice,
//! a pass declared twice, a pass naming an undeclared target, a transient
//! target read but never written, and a cycle (`a -> b -> a`).

use std::collections::BinaryHeap;
use std::fmt;

/// A texture the graph allocates for one frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TargetDesc {
    pub width: u32,
    pub height: u32,
    pub format: wgpu::TextureFormat,
    /// 1, or the MSAA sample count.
    pub samples: u32,
    pub usage: wgpu::TextureUsages,
}

/// Why a graph does not compile. Every variant names what is wrong.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum GraphError {
    DuplicateTarget(String),
    DuplicatePass(String),
    /// A pass reads or writes a target that was never declared.
    MissingTarget {
        pass: String,
        target: String,
    },
    /// A transient target is read but no pass writes it.
    NeverWritten {
        target: String,
        pass: String,
    },
    /// Passes that depend on each other, in dependency order, the first
    /// repeated at the end (`["a", "b", "a"]`).
    Cycle(Vec<String>),
}

impl fmt::Display for GraphError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            GraphError::DuplicateTarget(t) => {
                write!(f, "render graph: target '{t}' is declared twice")
            }
            GraphError::DuplicatePass(p) => write!(f, "render graph: pass '{p}' is declared twice"),
            GraphError::MissingTarget { pass, target } => write!(
                f,
                "render graph: pass '{pass}' uses target '{target}', which is not declared"
            ),
            GraphError::NeverWritten { target, pass } => write!(
                f,
                "render graph: pass '{pass}' reads transient target '{target}', which no pass writes"
            ),
            GraphError::Cycle(passes) => {
                write!(f, "render graph: cycle {}", passes.join(" -> "))
            }
        }
    }
}

impl std::error::Error for GraphError {}

#[derive(Clone, Debug)]
struct TargetDecl {
    name: String,
    /// `None` for an imported target.
    desc: Option<TargetDesc>,
}

#[derive(Clone, Debug)]
struct PassDecl {
    name: String,
    reads: Vec<String>,
    writes: Vec<String>,
}

/// A frame's passes and targets, before ordering. See the module docs.
#[derive(Clone, Debug, Default)]
pub struct RenderGraph {
    targets: Vec<TargetDecl>,
    passes: Vec<PassDecl>,
}

impl RenderGraph {
    pub fn new() -> RenderGraph {
        RenderGraph::default()
    }

    /// Declares a texture the frame allocates (and may alias with another
    /// transient target of the same description whose lifetime has ended).
    pub fn transient(&mut self, name: impl Into<String>, desc: TargetDesc) -> &mut Self {
        self.targets.push(TargetDecl {
            name: name.into(),
            desc: Some(desc),
        });
        self
    }

    /// Declares a target owned outside the frame (it keeps its contents
    /// between frames; a read with no writer reads those).
    pub fn import(&mut self, name: impl Into<String>) -> &mut Self {
        self.targets.push(TargetDecl {
            name: name.into(),
            desc: None,
        });
        self
    }

    /// Declares a pass with the targets it samples or loads (`reads`) and the
    /// ones it renders into (`writes`).
    pub fn pass(&mut self, name: impl Into<String>, reads: &[&str], writes: &[&str]) -> &mut Self {
        self.passes.push(PassDecl {
            name: name.into(),
            reads: reads.iter().map(|s| s.to_string()).collect(),
            writes: writes.iter().map(|s| s.to_string()).collect(),
        });
        self
    }

    pub fn pass_count(&self) -> usize {
        self.passes.len()
    }

    /// Orders the passes, checks the graph and assigns transient targets to
    /// physical textures.
    pub fn compile(&self) -> Result<GraphPlan, GraphError> {
        let n = self.passes.len();
        for (i, t) in self.targets.iter().enumerate() {
            if self.targets[..i].iter().any(|u| u.name == t.name) {
                return Err(GraphError::DuplicateTarget(t.name.clone()));
            }
        }
        for (i, p) in self.passes.iter().enumerate() {
            if self.passes[..i].iter().any(|q| q.name == p.name) {
                return Err(GraphError::DuplicatePass(p.name.clone()));
            }
        }
        let target_index = |pass: &PassDecl, name: &str| {
            self.targets
                .iter()
                .position(|t| t.name == name)
                .ok_or_else(|| GraphError::MissingTarget {
                    pass: pass.name.clone(),
                    target: name.to_string(),
                })
        };
        // Per pass, the target indices it reads and writes.
        let mut reads = Vec::with_capacity(n);
        let mut writes = Vec::with_capacity(n);
        for p in &self.passes {
            reads.push(
                p.reads
                    .iter()
                    .map(|t| target_index(p, t))
                    .collect::<Result<Vec<_>, _>>()?,
            );
            writes.push(
                p.writes
                    .iter()
                    .map(|t| target_index(p, t))
                    .collect::<Result<Vec<_>, _>>()?,
            );
        }
        let writers: Vec<Vec<usize>> = (0..self.targets.len())
            .map(|t| (0..n).filter(|&p| writes[p].contains(&t)).collect())
            .collect();
        // The writer whose contents pass `p` reads from target `t`.
        let bound_writer = |p: usize, t: usize| -> Result<Option<usize>, GraphError> {
            let w = &writers[t];
            if let Some(&before) = w.iter().rev().find(|&&q| q < p) {
                return Ok(Some(before));
            }
            if let Some(&after) = w.iter().find(|&&q| q > p) {
                return Ok(Some(after));
            }
            if self.targets[t].desc.is_some() {
                return Err(GraphError::NeverWritten {
                    target: self.targets[t].name.clone(),
                    pass: self.passes[p].name.clone(),
                });
            }
            Ok(None)
        };
        let mut edges: Vec<Vec<usize>> = vec![Vec::new(); n];
        let add = |from: usize, to: usize, edges: &mut Vec<Vec<usize>>| {
            if from != to && !edges[from].contains(&to) {
                edges[from].push(to);
            }
        };
        for p in 0..n {
            for &t in &reads[p] {
                if let Some(w) = bound_writer(p, t)? {
                    add(w, p, &mut edges);
                }
            }
            for &t in &writes[p] {
                let previous = writers[t].iter().rev().find(|&&q| q < p).copied();
                if let Some(w) = previous {
                    add(w, p, &mut edges);
                }
                // Readers of the contents this write replaces run first.
                for (r, read) in reads.iter().enumerate().take(p) {
                    if read.contains(&t) && bound_writer(r, t)? == previous {
                        add(r, p, &mut edges);
                    }
                }
            }
        }

        // Kahn's algorithm; among ready passes the earliest declared goes
        // first, so a graph declared in a valid order keeps that order.
        let mut indegree = vec![0usize; n];
        for e in &edges {
            for &to in e {
                indegree[to] += 1;
            }
        }
        let mut ready: BinaryHeap<std::cmp::Reverse<usize>> = (0..n)
            .filter(|&p| indegree[p] == 0)
            .map(std::cmp::Reverse)
            .collect();
        let mut order = Vec::with_capacity(n);
        while let Some(std::cmp::Reverse(p)) = ready.pop() {
            order.push(p);
            for &to in &edges[p] {
                indegree[to] -= 1;
                if indegree[to] == 0 {
                    ready.push(std::cmp::Reverse(to));
                }
            }
        }
        if order.len() < n {
            return Err(GraphError::Cycle(self.find_cycle(&edges, &indegree)));
        }

        // Lifetimes (positions in `order`) and physical slots.
        let mut position = vec![0usize; n];
        for (i, &p) in order.iter().enumerate() {
            position[p] = i;
        }
        let mut lifetimes: Vec<Option<(usize, usize)>> = vec![None; self.targets.len()];
        for p in 0..n {
            for &t in reads[p].iter().chain(&writes[p]) {
                if self.targets[t].desc.is_none() {
                    continue;
                }
                let at = position[p];
                lifetimes[t] = Some(match lifetimes[t] {
                    Some((a, b)) => (a.min(at), b.max(at)),
                    None => (at, at),
                });
            }
        }
        let mut by_first: Vec<usize> = (0..self.targets.len())
            .filter(|&t| lifetimes[t].is_some())
            .collect();
        og_core::sort::sort_by_key(&mut by_first, |&t| (lifetimes[t].expect("filtered").0, t));
        let mut physical: Vec<(TargetDesc, usize)> = Vec::new(); // (desc, last use)
        let mut slots = vec![None; self.targets.len()];
        for t in by_first {
            let desc = self.targets[t].desc.expect("transient");
            let (first, last) = lifetimes[t].expect("used");
            let slot = match physical
                .iter()
                .position(|&(d, end)| d == desc && end < first)
            {
                Some(s) => {
                    physical[s].1 = last;
                    s
                }
                None => {
                    physical.push((desc, last));
                    physical.len() - 1
                }
            };
            slots[t] = Some(slot);
        }
        Ok(GraphPlan {
            pass_names: self.passes.iter().map(|p| p.name.clone()).collect(),
            target_names: self.targets.iter().map(|t| t.name.clone()).collect(),
            order,
            lifetimes,
            slots,
            physical: physical.into_iter().map(|(d, _)| d).collect(),
        })
    }

    /// A cycle among the passes Kahn's algorithm could not schedule.
    fn find_cycle(&self, edges: &[Vec<usize>], indegree: &[usize]) -> Vec<String> {
        let stuck: Vec<bool> = indegree.iter().map(|&d| d > 0).collect();
        // Walk backwards along stuck predecessors until a pass repeats.
        let preds = |p: usize| (0..edges.len()).find(|&q| stuck[q] && edges[q].contains(&p));
        let start = (0..edges.len()).find(|&p| stuck[p]).expect("a stuck pass");
        let mut path = vec![start];
        let mut at = start;
        loop {
            let q = preds(at).expect("a stuck pass has a stuck predecessor");
            if let Some(i) = path.iter().position(|&p| p == q) {
                let mut cycle: Vec<usize> = path[i..].to_vec();
                cycle.reverse();
                let first = cycle[0];
                cycle.push(first);
                return cycle.iter().map(|&p| self.passes[p].name.clone()).collect();
            }
            path.push(q);
            at = q;
        }
    }
}

/// A compiled [`RenderGraph`]: the execution order and the transient
/// targets' physical textures.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GraphPlan {
    pass_names: Vec<String>,
    target_names: Vec<String>,
    order: Vec<usize>,
    lifetimes: Vec<Option<(usize, usize)>>,
    slots: Vec<Option<usize>>,
    physical: Vec<TargetDesc>,
}

impl GraphPlan {
    /// Pass indices (in declaration order) in execution order.
    pub fn order(&self) -> &[usize] {
        &self.order
    }

    /// The pass names in execution order.
    pub fn ordered_names(&self) -> Vec<&str> {
        self.order
            .iter()
            .map(|&p| self.pass_names[p].as_str())
            .collect()
    }

    /// The physical texture a transient target lives in (`None` for an
    /// imported or unused target, or an unknown name).
    pub fn slot(&self, target: &str) -> Option<usize> {
        let t = self.target_names.iter().position(|n| n == target)?;
        self.slots[t]
    }

    /// The first and last position in [`GraphPlan::order`] that uses a
    /// transient target.
    pub fn lifetime(&self, target: &str) -> Option<(usize, usize)> {
        let t = self.target_names.iter().position(|n| n == target)?;
        self.lifetimes[t]
    }

    /// The physical textures the frame allocates, by slot.
    pub fn physical(&self) -> &[TargetDesc] {
        &self.physical
    }
}
