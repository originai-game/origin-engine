use super::*;

impl RetainedHost {
    #[allow(clippy::too_many_arguments)]
    pub(super) fn emit_virtual(
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
        let at = ui.parent();
        if props != Props::default() {
            ui.nodes[at].props = props;
        }
        let visible: BTreeSet<_> = each.order[range.first as usize..range.last.min(count) as usize]
            .iter()
            .cloned()
            .collect();
        let leaving: Vec<_> = each
            .materialized
            .difference(&visible)
            .filter_map(|key| each.rows.get(key))
            .flatten()
            .cloned()
            .collect();
        let rows: Vec<_> = (range.first..range.last.min(count))
            .map(|index| {
                let key = &each.order[index as usize];
                (index, key.clone(), each.rows[key].clone())
            })
            .collect();
        let empty = each.empty.clone();
        self.each.get_mut(id).unwrap().materialized = visible;
        for child in leaving {
            self.dematerialize(&child)?;
        }
        let result = (|| {
            if count == 0 {
                for child in &empty {
                    self.emit(ui, child, env, events, writes)?;
                }
            }
            for (index, key, children) in rows {
                let list = &ui.state.virtual_lists[&element.context.node.key];
                let offset = ui
                    .state
                    .scroll
                    .get(&element.context.node.key)
                    .copied()
                    .unwrap_or(0.0);
                let top = list.offset(index as usize);
                let bottom = list.offset((index as usize / list.columns + 1) * list.columns);
                let materialization = if top < offset + list.viewport && bottom > offset {
                    Materialization::Visible
                } else {
                    Materialization::Overscan
                };
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
