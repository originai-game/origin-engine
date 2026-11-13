use super::*;

impl RetainedHost {
    /// The retained elements' records, mounted ones in tree order.
    pub(crate) fn records(&self) -> impl Iterator<Item = &super::host::DocRecord> {
        let mut order = Vec::new();
        for root in self.roots.values() {
            self.collect_rendered(root, &mut order);
        }
        order
            .into_iter()
            .map(|identity| &self.elements[identity].record)
    }

    pub(crate) fn collect_rendered<'a>(&'a self, child: &'a Child, out: &mut Vec<&'a MountedNode>) {
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
    pub(crate) fn record(&self, key: u64) -> Option<&super::host::DocRecord> {
        let identity = self.by_key.get(&key)?;
        Some(&self.elements.get(identity)?.record)
    }
}
