//! A cached class flip that only changes scalar motion targets keeps its
//! control. Real styles, part selectors and positioned descendants still
//! use ordinary resolution and emission.
use super::*;

impl Ui {
    pub(super) fn retarget_document(
        &mut self,
        e: &instances::ElementInstance,
        next: &Resolved,
    ) -> bool {
        let Some(previous) = &e.resolved else {
            return false;
        };
        let old = &previous.values;
        let node = &e.document.nodes[e.index];
        let target = |name: &str| matches!(name, "scale" | "opacity");
        if node.kind != NodeKind::Element(Kind::Button)
            || e.control.is_some()
            || e.volatile
            || !node.groups.is_empty()
            || e.cached.properties != e.resolved_inputs.properties
            || e.cached.expressions.len() != e.resolved_inputs.expressions.len()
            || !e
                .cached
                .expressions
                .iter()
                .zip(&e.resolved_inputs.expressions)
                .all(|((expr, value), (before, was))| {
                    expr == before
                        && (value == was
                            || node.selector_attrs.iter().any(|(name, read)| {
                                (name == "class" || name.starts_with("class:")) && read == expr
                            })
                            || matches!(expr, crate::bind::Expr::HasToken(base, _) if
                                node.selector_attrs.iter().any(|(name, read)| name == "class" && read == base.as_ref())))
                })
            || old.part.is_some()
            || next.part.is_some()
            || old.style.is_some()
            || next.style.is_some()
            || old.smil
            || next.smil
            || !old.animations.is_empty()
            || !next.animations.is_empty()
            || !old.pseudo.is_empty()
            || !next.pseudo.is_empty()
            || !old.instances.is_empty()
            || !next.instances.is_empty()
            || old.generated.is_some()
            || next.generated.is_some()
            || !old.span_styles.is_empty()
            || !next.span_styles.is_empty()
            || old.text != next.text
            || old.transitions != next.transitions
            || old.flow != next.flow
            || old.used_color != next.used_color
            || !old
                .vals
                .iter()
                .filter(|(p, _)| !target(PROPS[*p].name))
                .eq(next.vals.iter().filter(|(p, _)| !target(PROPS[*p].name)))
        {
            return false;
        }
        let (a, b) = (&old.cascade, &next.cascade);
        let resets_target =
            |p: &&usize| matches!(PROPS[**p].name, "scale" | "scaleX" | "scaleY" | "opacity");
        if a.local_names != b.local_names
            || !a
                .unset
                .iter()
                .filter(|p| !resets_target(p))
                .eq(b.unset.iter().filter(|p| !resets_target(p)))
            || a.motion != b.motion
            || a.text != b.text
            || a.svg != b.svg
            || a.container != b.container
            || a.generated != b.generated
            || a.transform != b.transform
            || !a
                .values
                .iter()
                .filter(|(name, ..)| !target(name))
                .eq(b.values.iter().filter(|(name, ..)| !target(name)))
            || !a
                .props
                .iter()
                .filter(|p| !target(p.name()))
                .eq(b.props.iter().filter(|p| !target(p.name())))
            || a.computed
                .changed(&b.computed)
                .iter()
                .any(|name| !target(name))
        {
            return false;
        }
        let (Some(old_host), Some(new_host), Some(slot)) =
            (&old.part_host, &next.part_host, e.slot)
        else {
            return false;
        };
        if old_host.attributes == new_host.attributes
            || !self.tree.holds(slot, previous.key, e.parent_slot)
        {
            return false;
        }
        let Some(fields) = e
            .record
            .render_fields
            .iter()
            .find(|(key, _)| *key == previous.key)
            .map(|(_, fields)| fields)
        else {
            return false;
        };
        let i = fields.index;
        // An explicit scale establishes a containing block even at 1. A
        // class may add/remove it only when nothing positioned can read it.
        let mut pending = self.tree.nodes[i].ordered_children.clone();
        while let Some(child) = pending.pop() {
            if self.tree.nodes[child].props.position != Position::Flow {
                return false;
            }
            pending.extend_from_slice(&self.tree.nodes[child].ordered_children);
        }
        let parts: Vec<_> = e
            .owned
            .iter()
            .filter_map(|&i| {
                let node = &self.tree.nodes[i];
                node.part()
                    .or_else(|| node.part_box.as_ref().map(|p| p.reference))
                    .map(|p| (p, p))
            })
            .collect();
        if !self.part_styles_unchanged_for_class_change(old_host, new_host, &parts) {
            return false;
        }
        let props = self.resolved_props(node, next);
        self.tree.stage(i);
        let p = self.tree.props_mut(i);
        p.transform.scale_x = props.transform.scale_x;
        p.transform.scale_y = props.transform.scale_y;
        p.opacity = props.opacity;
        p.holds_fixed = props.holds_fixed;
        if let Some(host) = self.tree.nodes.iter_mut().rfind(|n| n.key == previous.key) {
            host.part_host = next.part_host.clone();
        }
        // Sample the new targets even when the previous spring had settled;
        // unlike rewrite, this does not count the control as rebuilt.
        self.style.motion_mut().motion_moving.push(i);
        true
    }
}
