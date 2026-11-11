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

    /// The containers whose scroll state mounted elements read.
    pub(crate) fn scroll_reads(&self) -> impl Iterator<Item = u64> + '_ {
        self.elements
            .values()
            .filter_map(|e| e.resolved.as_ref())
            .flat_map(|r| r.values.cascade.scroll_reads.iter().copied())
    }

    /// Marks the mounted elements that read a `changed` scroll state dirty.
    pub(crate) fn rescroll(&mut self, changed: &BTreeSet<u64>) {
        if changed.is_empty() {
            return;
        }
        let readers = self.elements.iter().filter(|(_, e)| {
            e.resolved
                .as_ref()
                .is_some_and(|r| !r.values.cascade.scroll_reads.is_disjoint(changed))
        });
        let readers: Vec<MountedNode> = readers.map(|(identity, _)| identity.clone()).collect();
        self.dirty.extend(readers);
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
                &e.document,
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
            Default::default(),
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
            (width("Badge"), width("A"), ui.docs.host.retained.resolved)
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

    /// `style()` reads the container's custom properties as inherited
    /// values and container units its settled box: a change restyles only
    /// those readers.
    #[test]
    fn style_queries_and_container_units_restyle_only_their_readers() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "shelf.svelte",
                r#"<meta name="component" content="shelf">
<column id="Shelf">
  <panel id="A"/><panel id="Badge"/><panel id="Dot"/><panel id="Unit"/>
</column>
<style>
#Shelf { width: 60%; padding: 0px; container: shelf / inline-size; --tone: cool; }
@media (width > 450px) { #Shelf { --tone: warm; } }
panel { width: 10px; height: 10px; }
#Unit { width: 10cqi; }
@container shelf style(--tone: warm) { #Badge { width: 44px; } }
@container style(--tone: cool) { #Dot { width: 33px; } }
</style>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "shelf"),
            "shelf",
            BTreeMap::new(),
            Default::default(),
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
            (
                [width("A"), width("Badge"), width("Dot"), width("Unit")],
                ui.docs.host.retained.resolved,
            )
        };
        // 500: Shelf 300, warm.
        assert_eq!(frame(&mut ui, 500.0).0, [10.0, 44.0, 10.0, 30.0]);
        assert_eq!(frame(&mut ui, 500.0), ([10.0, 44.0, 10.0, 30.0], 0));
        // 400: Shelf 240, cool: Shelf, Badge, Dot and Unit resolve; A not.
        assert_eq!(frame(&mut ui, 400.0), ([10.0, 10.0, 33.0, 24.0], 4));
        assert_eq!(frame(&mut ui, 400.0).1, 0);
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }

    /// `scroll-state()` reads the container's last laid-out scroll state:
    /// scrolling restyles the readers on the next frame, idle frames none.
    #[test]
    fn scroll_state_queries_follow_the_scroll_container() {
        let mut ui = Ui::new(UiDesc::default());
        ui.define_document(
            UiDocument::parse(
                "list.svelte",
                r#"<meta name="component" content="list">
<column id="List">
  <column id="Head"><panel id="Mark"/></column>
  <panel id="Gap"/><panel id="Up"/><panel id="Down"/><panel id="Tall"/>
</column>
<style>
#List { height: 100px; padding: 0px; overflow-y: auto; container-type: scroll-state; }
#Gap { height: 40px; }
panel { width: 10px; height: 10px; }
#Tall { height: 300px; }
@container scroll-state(scrollable: top) { #Up { width: 44px; } }
@container scroll-state(scrolled: bottom) { #Down { width: 33px; } }
#Head { position: sticky; top: 0px; container-type: scroll-state; }
@container scroll-state(stuck: top) { #Mark { width: 22px; } }
</style>"#,
            )
            .unwrap(),
        );
        ui.mount_document(
            SourceId::new("test", "list"),
            "list",
            BTreeMap::new(),
            Default::default(),
            &crate::bind::NoEnv,
        )
        .unwrap();
        let frame = |ui: &mut Ui| {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.build_retained_documents(&crate::bind::NoEnv).unwrap();
            ui.end();
            let width = |id: &str| {
                ui.probe_elements()
                    .iter()
                    .find(|e| e.path == format!("test:list/{id}"))
                    .unwrap()
                    .width
            };
            (
                [width("Up"), width("Down"), width("Head/Mark")],
                ui.docs.host.retained.resolved,
            )
        };
        frame(&mut ui);
        assert_eq!(frame(&mut ui).0, [10.0, 10.0, 10.0]);
        assert_eq!(frame(&mut ui), ([10.0, 10.0, 10.0], 0));
        // As a wheel scroll leaves the list (the test does not paint, so
        // the range `scroll_by` clamps to is never recorded); the sticky
        // header, a container that does not scroll, ignores its offset.
        let keys: Vec<u64> = ui.docs.host.scroll_states.keys().copied().collect();
        for k in keys {
            ui.interaction.state.scroll.insert(k, 40.0);
        }
        let (now, _) = frame(&mut ui);
        assert_eq!(now[..2], [44.0, 33.0]);
        // The header is held at the top from the frame that laid it out so.
        assert_eq!(frame(&mut ui).0, [44.0, 33.0, 22.0], "stuck: top");
        assert_eq!(frame(&mut ui).1, 0);
        assert!(ui.frame().warnings.is_empty(), "{:?}", ui.frame().warnings);
    }
}
