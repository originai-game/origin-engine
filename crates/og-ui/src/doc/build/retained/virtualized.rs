use super::*;

pub(crate) trait RetainedVirtualizedRetainedHost {
    #[allow(clippy::too_many_arguments)]
    fn emit_virtual(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        build: &mut Build<'_>,
        props: Props,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String>;

    fn dematerialize(&mut self, child: &Child) -> Result<(), String>;
}

impl RetainedVirtualizedRetainedHost for RetainedHost {
    #[allow(clippy::too_many_arguments)]
    fn emit_virtual(
        &mut self,
        ui: &mut Ui,
        element: &ElementInstance,
        build: &mut Build<'_>,
        props: Props,
        env: &dyn Env,
        events: &mut Vec<DocEvent>,
        writes: &mut Vec<(BindingContext, String, Value)>,
    ) -> Result<(), String> {
        let Some(Child::Each(id)) = element.children.first() else {
            return Err("a virtual container requires a keyed each child".into());
        };
        let grid = element.document.nodes[element.index].kind == NodeKind::Element("cells");
        let r = &element.resolved.as_ref().unwrap().values;
        let each = &self.each[id];
        let count = each.order.len() as u32;
        let padding = r.nums("padding").map(insets_of).unwrap_or_default();
        let order = Some(crate::virtualize::Order::Retained {
            keys: &each.order,
            revision: each.revision,
        });
        let range = if grid {
            ui.open_grid_inner(
                element.id.as_str().into(),
                crate::widgets::GridDesc {
                    count,
                    cell_width: r.num("cellWidth").unwrap_or(0.0),
                    cell_height: r.num("cellHeight").unwrap_or(0.0),
                    columns: r.num("cellColumns").unwrap_or(0.0) as u32,
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
                order,
            )
        } else {
            ui.open_list_inner(
                element.id.as_str().into(),
                crate::widgets::ListDesc {
                    count,
                    row_height: r.num("rowHeight").unwrap_or(0.0),
                    gap: r.num("gap").unwrap_or(0.0),
                    width: r.size("width").unwrap_or(Size::grow()),
                    height: r.size("height"),
                    padding,
                },
                order,
            )
        };
        let at = ui.tree.parent();
        if props != Props::default() {
            *ui.tree.props_mut(at) = props;
        }
        let (first, end) = (range.first as usize, range.last.min(count) as usize);
        // What leaves the window and what enters it; the rows staying are
        // read in place, not copied.
        let window: std::collections::HashSet<&String> = each.order[first..end].iter().collect();
        let gone: Vec<String> = each
            .materialized
            .iter()
            .filter(|key| !window.contains(key))
            .cloned()
            .collect();
        let entering: Vec<String> = each.order[first..end]
            .iter()
            .filter(|key| !each.materialized.contains(*key))
            .cloned()
            .collect();
        let leaving: Vec<_> = gone
            .iter()
            .filter_map(|key| each.rows.get(key))
            .flatten()
            .cloned()
            .collect();
        let empty = if count == 0 {
            each.empty.clone()
        } else {
            Vec::new()
        };
        let materialized = &mut self.each.get_mut(id).unwrap().materialized;
        for key in &gone {
            materialized.remove(key);
        }
        materialized.extend(entering);
        for child in leaving {
            self.dematerialize(&child)?;
            self.release(ui, &child);
        }
        self.scrollers.insert(
            element.context.node.clone(),
            ui.interaction
                .state
                .scroll
                .get(&element.context.node.key)
                .copied()
                .unwrap_or(0.0),
        );
        // Each row's materialization: on screen, or in the overscan.
        let shown: Vec<Materialization> = {
            let list = &ui.interaction.state.virtual_lists[&element.context.node.key];
            let offset = ui
                .interaction
                .state
                .scroll
                .get(&element.context.node.key)
                .copied()
                .unwrap_or(0.0);
            (first..end)
                .map(|index| {
                    let top = list.offset(index);
                    let bottom = list.offset((index / list.columns + 1) * list.columns);
                    if top < offset + list.viewport && bottom > offset {
                        Materialization::Visible
                    } else {
                        Materialization::Overscan
                    }
                })
                .collect()
        };
        let result = (|| {
            for child in &empty {
                self.emit(ui, child, env, events, writes)?;
            }
            let list_slot = ui.tree.parent();
            let list_key = ui.tree.nodes[list_slot].key;
            for (index, materialization) in (first..end).zip(shown) {
                let index = index as u32;
                // A row whose own node would be written as it stands (no
                // input on it now or last frame, the same frame-wide
                // inputs) is put back with its subtree: a scroll writes
                // only the rows it brings in. What changed inside it is
                // written in place (`emit_in_place`).
                let each = &self.each[id];
                let key = &each.order[index as usize];
                let row_key =
                    crate::id::child_key(list_key, &crate::virtualize::item_id(key, grid));
                let still = !self.restyled
                    && !self.input.contains(&row_key)
                    && !self.last_input.contains(&row_key)
                    && each.rows[key].iter().all(|child| match child {
                        Child::Element(id) => self
                            .topology
                            .node(id)
                            .is_some_and(|n| n.materialization == materialization),
                        _ => false,
                    });
                if still
                    && let Some(slot) = ui.tree.render_slot(row_key, list_slot)
                    && ui.tree.holds(slot, row_key, list_slot)
                {
                    ui.tree.keep_owned(slot);
                    continue;
                }
                let key = key.clone();
                let children = each.rows[&key].clone();
                let response = if grid {
                    ui.grid_item(index, false)
                } else {
                    ui.list_item(index, false)
                };
                if response.clicked {
                    let path = format!(
                        "{}/{}",
                        element.path,
                        crate::virtualize::item_id(&key, grid)
                    );
                    build.event(&path, "select", &element.id, json!(key));
                }
                let outer = self.materializing.replace(materialization);
                let result = children
                    .iter()
                    .try_for_each(|child| self.emit(ui, child, env, events, writes));
                self.materializing = outer;
                ui.close();
                result?;
            }
            Ok(())
        })();
        ui.close_list();
        result
    }

    fn dematerialize(&mut self, child: &Child) -> Result<(), String> {
        let children = match child {
            Child::Element(id) => {
                if self.topology.node(id).unwrap().materialization == Materialization::Deferred {
                    return Ok(());
                }
                self.topology
                    .set_materialization(id, Materialization::Deferred)?;
                self.elements[id].children.clone()
            }
            Child::Branch(id) => self.branches[id].children.clone(),
            Child::Each(id) => {
                let each = &self.each[id];
                let parent = &self.elements[&each.parent];
                let keys: Box<dyn Iterator<Item = &String>> = if matches!(
                    parent.document.nodes[parent.index].kind,
                    NodeKind::Element("list" | "cells")
                ) {
                    Box::new(each.materialized.iter())
                } else {
                    Box::new(each.order.iter())
                };
                keys.filter_map(|key| each.rows.get(key))
                    .flatten()
                    .chain(&each.empty)
                    .cloned()
                    .collect()
            }
        };
        for child in children {
            self.dematerialize(&child)?;
        }
        Ok(())
    }
}
