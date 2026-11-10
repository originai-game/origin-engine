use super::*;

impl RetainedHost {
    /// The document cascade's winning declarations of the element the
    /// render key `key` belongs to (`file:line` each), and its bindings, as
    /// the UI debugger shows them (`Ui::debug_element`): read from what its
    /// last resolve kept, for that one element.
    pub(crate) fn debug_styles(
        &self,
        key: u64,
    ) -> Option<(Vec<crate::UiDebugStyle>, crate::ui::DebugBindings)> {
        let e = self.elements.get(self.owner_of.get(&key)?)?;
        let resolved = e.resolved.as_ref()?;
        let source = &e.document.source;
        let styles = resolved
            .values
            .cascade
            .values
            .iter()
            .map(|(name, value, line)| crate::UiDebugStyle {
                name: name.clone(),
                value: value.trim().to_owned(),
                source: format!("{source}:{line}"),
            })
            .collect();
        let bindings = e
            .record
            .built
            .iter()
            .filter(|b| b.path == e.path)
            .flat_map(|b| &b.bindings)
            .map(|(p, x, v)| {
                let v = v.as_str().map_or_else(|| v.to_string(), str::to_owned);
                (p.clone(), x.clone(), v)
            })
            .collect();
        Some((styles, bindings))
    }

    /// The retained elements' records, mounted ones in tree order.
    pub(crate) fn records(&self) -> impl Iterator<Item = &super::mount::DocRecord> {
        let mut order = Vec::new();
        for root in self.roots.values() {
            self.collect_rendered(root, &mut order);
        }
        order
            .into_iter()
            .map(|identity| &self.elements[identity].record)
    }

    fn collect_rendered<'a>(&'a self, child: &'a Child, out: &mut Vec<&'a MountedNode>) {
        match child {
            Child::Element(identity) => {
                let element = &self.elements[identity];
                if element.slot.is_none() {
                    return;
                }
                out.push(identity);
                for child in &element.children {
                    self.collect_rendered(child, out);
                }
            }
            Child::Branch(identity) => {
                for child in &self.branches[identity].children {
                    self.collect_rendered(child, out);
                }
            }
            Child::Each(identity) => {
                let each = &self.each[identity];
                for key in &each.order {
                    for child in &each.rows[key] {
                        self.collect_rendered(child, out);
                    }
                }
                for child in &each.empty {
                    self.collect_rendered(child, out);
                }
            }
        }
    }

    #[cfg(test)]
    pub(crate) fn record(&self, key: u64) -> Option<&super::mount::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&self.elements.get(identity)?.record)
    }

    pub(crate) fn record_mut(&mut self, key: u64) -> Option<&mut super::mount::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&mut self.elements.get_mut(identity)?.record)
    }

    /// A widget memory a mounted element keeps while it is not built.
    pub(crate) fn keeps_memory(&self, key: u64) -> bool {
        self.memories.contains_key(&key)
    }
}
