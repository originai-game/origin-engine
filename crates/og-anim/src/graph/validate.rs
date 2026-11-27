//! Structural validation shared by typed construction and native decoding.
use super::*;

impl GraphAsset {
    pub fn validate(&self) -> Result<()> {
        let invalid = |s: &str| Error::new(ErrorKind::Parse, format!("animation graph: {s}"));
        fn unique<'a>(names: impl Iterator<Item = &'a str>) -> bool {
            let mut seen = std::collections::BTreeSet::new();
            names.into_iter().all(|n| !n.is_empty() && seen.insert(n))
        }
        if self.params.len() > GRAPH_PARAMS
            || !unique(self.params.iter().map(String::as_str))
            || !unique(self.sync_groups.iter().map(String::as_str))
            || !unique(self.nodes.iter().map(|n| n.name.as_str()))
            || !unique(self.layers.iter().map(|l| l.name.as_str()))
        {
            return Err(invalid(
                "names must be nonempty and unique; at most eight parameters",
            ));
        }
        let param = |p: usize| p < self.params.len();
        let node = |n: usize| n < self.nodes.len();
        for n in &self.nodes {
            let valid = match &n.motion {
                Motion::Clip { clip, speed, .. } => *clip != 0 && speed.is_finite() && *speed > 0.0,
                Motion::Blend1D { param: p, children } => {
                    param(*p)
                        && !children.is_empty()
                        && children.iter().all(|(t, n)| t.is_finite() && node(*n))
                        && children.windows(2).all(|w| w[0].0 <= w[1].0)
                }
                Motion::Blend2D { x, y, children } => {
                    param(*x)
                        && param(*y)
                        && !children.is_empty()
                        && children.iter().all(|(p, n)| p.is_finite() && node(*n))
                }
            };
            if !valid {
                return Err(invalid("invalid motion, parameter, child or playback rate"));
            }
        }
        // Three colours visit a shared subtree once and reject back edges.
        fn visit(g: &GraphAsset, n: usize, marks: &mut [u8]) -> bool {
            if marks[n] != 0 {
                return marks[n] == 2;
            }
            marks[n] = 1;
            let valid = match &g.nodes[n].motion {
                Motion::Clip { .. } => true,
                Motion::Blend1D { children, .. } => children.iter().all(|c| visit(g, c.1, marks)),
                Motion::Blend2D { children, .. } => children.iter().all(|c| visit(g, c.1, marks)),
            };
            marks[n] = 2;
            valid
        }
        let mut marks = vec![0; self.nodes.len()];
        if (0..self.nodes.len()).any(|n| !visit(self, n, &mut marks)) {
            return Err(invalid("motion contains itself"));
        }
        if self.layers.is_empty() {
            return Err(invalid("at least one layer is required"));
        }
        for l in &self.layers {
            if l.states.is_empty()
                || l.entry >= l.states.len()
                || !unique(l.states.iter().map(|s| s.name.as_str()))
                || !l.weight.is_finite()
                || !(0.0..=1.0).contains(&l.weight)
                || l.weight_param.is_some_and(|p| !param(p))
                || !unique(l.mask.iter().map(|m| m.node.as_str()))
                || l.mask
                    .iter()
                    .any(|m| !m.weight.is_finite() || !(0.0..=1.0).contains(&m.weight))
            {
                return Err(invalid("invalid layer, entry, mask or weight"));
            }
            for s in &l.states {
                if !node(s.motion)
                    || !s.speed.is_finite()
                    || s.speed < 0.0
                    || s.sync.is_some_and(|p| p >= self.sync_groups.len())
                {
                    return Err(invalid("invalid state motion, speed or sync group"));
                }
            }
            for t in &l.transitions {
                if t.from.is_some_and(|s| s >= l.states.len())
                    || t.to >= l.states.len()
                    || !t.fade.is_finite()
                    || t.fade < 0.0
                    || t.exit.is_some_and(|e| !e.is_finite() || e < 0.0)
                    || (t.conditions.is_empty() && t.exit.is_none())
                    || t.conditions
                        .iter()
                        .any(|c| !param(c.param) || !c.value.is_finite())
                {
                    return Err(invalid("invalid transition state, condition, exit or fade"));
                }
            }
        }
        Ok(())
    }
}
