//! Container queries on mounted elements: settlement restyles the elements
//! whose `@container` conditions read a changed box, from their cached inputs.
use super::*;

impl RetainedHost {
    /// The layout keys of mounted query containers.
    pub(crate) fn containers(&self) -> impl Iterator<Item = u64> + '_ {
        self.elements
            .values()
            .filter_map(|e| e.resolved.as_ref()?.values.container)
    }

    /// Resolves the materialized elements that read `changed` containers,
    /// and those that inherit from them, then replaces their render fields
    /// from the unstyled baseline. The resolved values stay on the element,
    /// so its next emission keeps them. Answers the layout nodes restyled.
    pub(crate) fn requery(
        &mut self,
        ui: &mut Ui,
        changed: &BTreeSet<u64>,
    ) -> Result<Vec<usize>, String> {
        if changed.is_empty() {
            return Ok(Vec::new());
        }
        let readers: Vec<MountedNode> = self
            .elements
            .iter()
            .filter(|(_, e)| {
                e.resolved
                    .as_ref()
                    .is_some_and(|r| !r.values.cascade.container_reads.is_disjoint(changed))
            })
            .map(|(identity, _)| identity.clone())
            .collect();
        self.dirty.extend(readers);
        let mut restyled = Vec::new();
        while let Some(node) = self.dirty.pop_first() {
            self.elements.get_mut(&node).unwrap().style_dirty = true;
            if self.topology.node(&node).unwrap().materialization == Materialization::Deferred {
                continue;
            }
            self.resolve(ui, &node, &crate::bind::NoEnv)?;
            let e = &self.elements[&node];
            let resolved = e.resolved.as_ref().unwrap();
            let fields = e
                .record
                .render_fields
                .iter()
                .find(|(k, _)| *k == resolved.key)
                .map(|(_, fields)| fields.clone());
            if let Some(index) = ui.replace_fields(
                fields,
                &resolved.path,
                &e.document.nodes[e.index],
                &resolved.values,
            ) {
                restyled.push(index);
            }
        }
        Ok(restyled)
    }
}

#[cfg(test)]
mod tests {
    use crate::doc::UiDocument;
    use crate::reactive::SourceId;
    use crate::{InputFrame, Ui, UiDesc, Viewport};
    use std::collections::BTreeMap;

    /// A mounted element restyles from its cached inputs when its container
    /// is first measured and on resize, without touching its siblings; the
    /// queried values stay on the element through later frames.
    #[test]
    fn mounted_container_queries_restyle_only_their_readers() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "shelf.svelte",
                r#"<meta name="component" content="shelf">
<column id="Shelf">
  <panel id="A"/><panel id="B"/><panel id="C"/><panel id="Badge"/>
</column>
<style>
#Shelf { width: 60%; padding: 0px; container-type: inline-size; }
panel { width: 10px; height: 10px; }
@container (width > 250px) { #Badge { width: 44px; } }
</style>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "shelf"),
            "shelf",
            BTreeMap::new(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui, width: f32| {
            ui.begin(InputFrame::default(), Viewport::new(width, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let width = |id: &str| {
                ui.probe_elements()
                    .iter()
                    .find(|e| e.path == format!("test:shelf/{id}"))
                    .unwrap()
                    .width
            };
            (width("Badge"), width("A"), ui.docs.retained.resolved)
        };
        // 500: Shelf 300. The first build cannot know; settlement resolves
        // the one reader before paint.
        let (badge, a, _) = frame(&mut ui, 500.0);
        assert_eq!((badge, a), (44.0, 10.0));
        // Idle: nothing resolves, the queried width stays.
        assert_eq!(frame(&mut ui, 500.0), (44.0, 10.0, 0));
        // 400: Shelf 240. Only Badge resolves again.
        assert_eq!(frame(&mut ui, 400.0), (10.0, 10.0, 1));
        assert_eq!(frame(&mut ui, 400.0), (10.0, 10.0, 0));
        assert_eq!(frame(&mut ui, 700.0), (44.0, 10.0, 1));
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }
}
