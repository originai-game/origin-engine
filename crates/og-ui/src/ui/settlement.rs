use super::*;
use std::collections::BTreeMap;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ContentBox {
    pub width: f32,
    pub height: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct ContainerMeasurement {
    pub key: u64,
    pub previous: Option<ContentBox>,
    pub current: ContentBox,
}

#[derive(Clone, Debug, PartialEq)]
pub(crate) enum SettlementError {
    Resolver(String),
    Cycle(Vec<u64>),
    InvalidNode(usize),
}

impl From<String> for SettlementError {
    fn from(error: String) -> Self {
        Self::Resolver(error)
    }
}

impl std::fmt::Display for SettlementError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Resolver(error) => write!(f, "container query: {error}"),
            Self::Cycle(keys) => write!(f, "container query geometry did not converge: {keys:?}"),
            Self::InvalidNode(i) => write!(f, "container query returned missing layout node {i}"),
        }
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct Measurements {
    watched: BTreeMap<u64, Option<ContentBox>>,
    pending: BTreeMap<u64, ContainerMeasurement>,
}

#[derive(Default)]
pub(crate) struct Settlement {
    changed: BTreeMap<u64, ContentBox>,
    seen: Vec<BTreeMap<u64, ContentBox>>,
}

pub(crate) struct LayoutInputs {
    enabled: bool,
    sizing: BTreeMap<usize, (Size, Size, Insets)>,
}

impl LayoutInputs {
    pub(crate) fn new(measurements: &Measurements, part_geometry: bool) -> Self {
        Self {
            enabled: part_geometry || !measurements.watched.is_empty(),
            sizing: BTreeMap::new(),
        }
    }

    pub(crate) fn remember(&mut self, i: usize, node: &Node) {
        if !self.enabled {
            return;
        }
        self.sizing
            .entry(i)
            .or_insert((node.el.width, node.el.height, node.el.padding));
    }

    fn swap(&mut self, nodes: &mut [Node]) {
        for (&i, (width, height, padding)) in &mut self.sizing {
            std::mem::swap(width, &mut nodes[i].el.width);
            std::mem::swap(height, &mut nodes[i].el.height);
            std::mem::swap(padding, &mut nodes[i].el.padding);
        }
    }
}

impl Measurements {
    pub(crate) fn watch(&mut self, key: u64) {
        self.watched.entry(key).or_insert(None);
    }

    pub(crate) fn unwatch(&mut self, key: u64) {
        self.watched.remove(&key);
        self.pending.remove(&key);
    }

    pub(crate) fn observe(&mut self, key: u64, current: ContentBox) {
        let Some(previous) = self.watched.get_mut(&key) else {
            return;
        };
        if *previous == Some(current) {
            return;
        }
        let change = self.pending.entry(key).or_insert(ContainerMeasurement {
            key,
            previous: *previous,
            current,
        });
        change.current = current;
        *previous = Some(current);
        if change.previous == Some(current) {
            self.pending.remove(&key);
        }
    }

    pub(crate) fn take(&mut self) -> Vec<ContainerMeasurement> {
        std::mem::take(&mut self.pending).into_values().collect()
    }
}

impl Ui {
    /// Registers an actual layout node, not a document selector index. The
    /// subscription's first layout publishes even a zero-sized content box.
    pub fn watch_layout_container(&mut self, key: u64) {
        self.layout.cache.measurements.watch(key);
    }

    /// Removes a native layout subscription and any unpublished measurement.
    pub fn unwatch_layout_container(&mut self, key: u64) {
        self.layout.cache.measurements.unwatch(key);
    }

    pub(crate) fn settle_layout_queries(
        &mut self,
        settlement: &mut Settlement,
        mut inputs: LayoutInputs,
        resolve: &mut impl FnMut(
            &mut Ui,
            &[ContainerMeasurement],
        ) -> Result<Vec<usize>, SettlementError>,
    ) -> Result<bool, SettlementError> {
        let changes = self.layout.cache.measurements.take();
        // Placement temporarily fixes floating dimensions and adds safe-area
        // padding. Resolvers patch authored inputs, not those derived values.
        inputs.swap(&mut self.nodes);
        let (geometry, mut dirty) = self.settle_part_geometry();
        if changes.is_empty() && geometry.is_empty() {
            inputs.swap(&mut self.nodes);
            return Ok(false);
        }
        for change in changes.iter().chain(&geometry) {
            settlement.changed.insert(change.key, change.current);
        }
        if settlement.seen.contains(&settlement.changed) {
            return Err(SettlementError::Cycle(
                changes.iter().chain(&geometry).map(|c| c.key).collect(),
            ));
        }
        settlement.seen.push(settlement.changed.clone());
        dirty.extend(resolve(self, &changes)?);
        if dirty.is_empty() {
            inputs.swap(&mut self.nodes);
            return Ok(false);
        }
        let mut roots = std::collections::BTreeSet::new();
        for i in dirty {
            if i >= self.nodes.len() {
                return Err(SettlementError::InvalidNode(i));
            }
            let parent = self.nodes[i].parent;
            if parent != NONE
                && self.nodes[i].props.out_of_flow() != self.nodes[parent].out.contains(&i)
            {
                let (out, flow) = self.nodes[parent]
                    .ordered_children
                    .iter()
                    .copied()
                    .partition(|&child| self.nodes[child].props.out_of_flow());
                self.nodes[parent].out = out;
                self.nodes[parent].children = flow;
            }
            // Its subtree and the path to it are laid out again.
            self.layout.cache.reach(&self.nodes, i, true);
            let mut at = i;
            loop {
                self.layout.cache.clean[at] = false;
                self.layout.cache.unclean.push(at);
                let parent = self.nodes[at].parent;
                if parent == NONE {
                    roots.insert(at);
                    break;
                }
                at = parent;
            }
        }
        // Popovers depend on anchors and HUD groups avoid each other. Their
        // placement must settle too, even if their own styles did not change.
        roots.extend(self.floating.iter().copied());
        self.layout.cache.query_roots = Some(roots);
        Ok(true)
    }

    pub(crate) fn abort_layout_queries(&mut self) {
        self.layout.cache.query_roots = None;
        self.invalidation.epoch += 1;
        self.layout.cache.measurements.pending.clear();
        self.layout
            .cache
            .measurements
            .watched
            .values_mut()
            .for_each(|v| *v = None);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(ui: &mut Ui, width: f32) -> (usize, usize) {
        ui.begin(InputFrame::default(), Viewport::new(width, 300.0));
        ui.open(
            "container",
            Element {
                width: Size::grow(),
                height: Size::Fixed(120.0),
                padding: Insets {
                    left: 11.0,
                    right: 17.0,
                    top: 7.0,
                    bottom: 13.0,
                },
                ..Element::column()
            },
        );
        let container = ui.parent();
        ui.watch_layout_container(ui.nodes[container].key);
        ui.open(
            "child",
            Element {
                width: Size::Fixed(3.0),
                height: Size::Fixed(10.0),
                ..Element::default()
            },
        );
        let child = ui.parent();
        ui.close();
        ui.close();
        (container, child)
    }

    #[test]
    fn first_frame_and_resize_settle_before_frame_publication() {
        let mut ui = Ui::default();
        for width in [400.0, 220.0, 400.0] {
            let (container, child) = build(&mut ui, width);
            // Once per finish: the bounded frame's check finishes a whole
            // twin too (`end_with_layout_queries`), resolving it as well.
            #[cfg(debug_assertions)]
            let finishes = if crate::ui::bounded::checking() && !ui.invalidation.whole {
                2
            } else {
                1
            };
            #[cfg(not(debug_assertions))]
            let finishes = 1;
            let mut calls = 0;
            let frame = ui
                .end_with_layout_queries(|ui, sizes| {
                    calls += 1;
                    assert_eq!(sizes.len(), 1);
                    assert_eq!(sizes[0].key, ui.nodes[container].key);
                    assert_eq!(
                        sizes[0].current,
                        ContentBox {
                            width: width - 28.0,
                            height: 100.0
                        }
                    );
                    if width == 400.0 && sizes[0].previous.is_none() {
                        assert_eq!(ui.out.frame.items.len(), 0);
                    }
                    ui.nodes[child].el.width = Size::Fixed(if sizes[0].current.width > 300.0 {
                        91.0
                    } else {
                        37.0
                    });
                    Ok(vec![child])
                })
                .unwrap();
            assert_eq!(calls, finishes);
            let _ = frame;
            let child = ui.elements().iter().find(|e| e.id == "child").unwrap();
            assert_eq!(child.width, if width == 400.0 { 91.0 } else { 37.0 });
        }
    }

    #[test]
    fn subscriptions_publish_zero_once_and_coalesce_intermediate_layouts() {
        let mut measurements = Measurements::default();
        let zero = ContentBox {
            width: 0.0,
            height: 0.0,
        };
        measurements.watch(7);
        measurements.observe(7, zero);
        assert_eq!(
            measurements.take(),
            vec![ContainerMeasurement {
                key: 7,
                previous: None,
                current: zero
            }]
        );
        measurements.observe(
            7,
            ContentBox {
                width: 40.0,
                height: 10.0,
            },
        );
        measurements.observe(7, zero);
        assert!(measurements.take().is_empty());
        measurements.unwatch(7);
        measurements.observe(7, zero);
        assert!(measurements.take().is_empty());
        measurements.watch(7);
        measurements.observe(7, zero);
        assert_eq!(measurements.take()[0].previous, None);
    }

    #[test]
    fn restyling_labels_and_buttons_does_not_replay_input_or_motion() {
        let mut ui = Ui::default();
        for width in [400.0, 220.0, 400.0] {
            ui.begin(
                InputFrame {
                    keys: vec![crate::UiKey::Enter],
                    ..Default::default()
                },
                Viewport::new(width, 300.0),
            );
            ui.watch_layout_container(ui.nodes[0].key);
            ui.label(
                "label",
                "before",
                LabelDesc {
                    on_scene: true,
                    ..Default::default()
                },
            );
            let label = ui.last_closed;
            let desc = ButtonDesc {
                variant: ButtonVariant::Primary,
                ..Default::default()
            };
            let response = ui.button("button", "Confirm", desc);
            let button = ui.last_closed;
            ui.set_focus("button");
            ui.end_with_layout_queries(|ui, sizes| {
                let before = ui.save_state();
                let feedback = ui.clone().take_feedback();
                let width = if sizes[0].current.width > 300.0 {
                    91.0
                } else {
                    37.0
                };
                ui.apply_label(
                    label,
                    "after",
                    LabelDesc {
                        width: Size::Fixed(width),
                        ..Default::default()
                    },
                );
                ui.apply_button(
                    button,
                    "Confirm",
                    ButtonDesc {
                        width: Some(Size::Fixed(width)),
                        ..desc
                    },
                    response,
                    (0.0, 0.0),
                    0.0,
                );
                assert_eq!(ui.save_state(), before);
                assert_eq!(ui.clone().take_feedback(), feedback);
                Ok(vec![label, button])
            })
            .unwrap();
            assert_eq!(ui.element("label").unwrap().text, "after");
            assert_eq!(
                ui.element("button").unwrap().width,
                if width > 300.0 { 91.0 } else { 37.0 }
            );
            assert_eq!(ui.frame().stats.elements, 4);
        }
    }

    #[test]
    fn replacing_span_and_image_styles_matches_a_fresh_frame() {
        let attributes = [crate::TextSpan {
            name: "accent".into(),
            position: 1,
            length: 1,
            properties: Default::default(),
        }];
        let rich = crate::RichTextDesc {
            width: Size::Fixed(120.0),
            ..Default::default()
        };
        let image = ImageDesc {
            width: Size::Fixed(31.0),
            height: Size::Fixed(17.0),
            label: "Preview".into(),
            ..Default::default()
        };
        let mut patched = Ui::default();
        let mut direct = Ui::default();
        for (ui, styled) in [(&mut patched, true), (&mut direct, false)] {
            ui.begin(InputFrame::default(), Viewport::new(400.0, 300.0));
            ui.rich_text_spans("words", "A🙂B", &attributes, rich, |_| {
                crate::TextSpanStyle {
                    bold: styled.then_some(true),
                    color: styled.then_some(crate::paint::ColorRef::Literal(Srgba::BLACK)),
                    ..Default::default()
                }
            });
            ui.image(
                "preview",
                ImageDesc {
                    tint: styled.then_some(Srgba::BLACK),
                    ..image.clone()
                },
            );
        }
        patched.watch_layout_container(patched.nodes[0].key);
        patched
            .end_with_layout_queries(|ui, _| {
                let before = ui.save_state();
                // A query becomes true, then false again during settlement.
                // Replacement must remove overrides instead of accumulating them.
                for active in [true, false] {
                    ui.apply_text_spans(1, "A🙂B", &attributes, rich, |_| crate::TextSpanStyle {
                        bold: active.then_some(true),
                        color: active.then_some(crate::paint::ColorRef::Literal(Srgba::BLACK)),
                        ..Default::default()
                    });
                    ui.apply_image(
                        2,
                        ImageDesc {
                            tint: active.then_some(Srgba::BLACK),
                            ..image.clone()
                        },
                    );
                }
                assert_eq!(ui.save_state(), before);
                Ok(vec![1, 2])
            })
            .unwrap();
        direct.end();
        assert_eq!(patched.elements(), direct.elements());
        assert_eq!(patched.out.frame.items, direct.out.frame.items);
        assert_eq!(patched.a11y(), direct.a11y());
    }

    #[test]
    fn unchanged_geometry_never_invokes_the_resolver() {
        let mut ui = Ui::default();
        build(&mut ui, 400.0);
        ui.end_with_layout_queries(|_, _| Ok(Vec::new())).unwrap();
        build(&mut ui, 400.0);
        ui.end_with_layout_queries(|_, _| panic!("idle container subscription ran"))
            .unwrap();
    }

    fn floating(ui: &mut Ui, width: f32, child_width: f32) -> usize {
        let mut viewport = Viewport::new(width, 300.0);
        viewport.safe_area = Insets {
            left: 7.0,
            right: 19.0,
            top: 11.0,
            bottom: 23.0,
        };
        ui.begin(InputFrame::default(), viewport);
        ui.open_anchor(
            "hud",
            crate::AnchorDesc {
                anchor: Anchor::BottomRight,
                margin: Some(13.0),
                width: Size::grow(),
                ..Default::default()
            },
        );
        let container = ui.parent();
        ui.watch_layout_container(ui.nodes[container].key);
        ui.open(
            "patch",
            Element {
                width: Size::Fixed(child_width),
                height: Size::Fixed(child_width / 2.0),
                background: Some(Srgba::rgb(36, 174, 133)),
                ..Element::default()
            },
        );
        let child = ui.parent();
        ui.close();
        ui.close();
        child
    }

    fn settled_floating(ui: &mut Ui, width: f32) {
        let child = floating(ui, width, 3.0);
        ui.end_with_layout_queries(|ui, sizes| {
            let side = if sizes[0].current.width > 300.0 {
                91.0
            } else {
                37.0
            };
            if ui.nodes[child].el.width == Size::Fixed(side) {
                return Ok(Vec::new());
            }
            ui.nodes[child].el.width = Size::Fixed(side);
            ui.nodes[child].el.height = Size::Fixed(side / 2.0);
            Ok(vec![child])
        })
        .unwrap();
    }

    #[test]
    fn floating_queries_recompute_intrinsic_size_and_anchor_placement() {
        let mut queried = Ui::default();
        let mut direct = Ui::default();
        for width in [400.0, 220.0, 400.0] {
            settled_floating(&mut queried, width);
            floating(&mut direct, width, if width == 400.0 { 91.0 } else { 37.0 });
            direct.end();
            assert_eq!(queried.out.frame.items, direct.out.frame.items);
            assert_eq!(queried.elements(), direct.elements());
            let checkpoint = queried.save_state();
            queried.restore_state(&checkpoint).unwrap();
        }
    }

    #[test]
    fn cycles_and_resolver_errors_do_not_publish_an_intermediate_frame() {
        let mut ui = Ui::default();
        let (container, _) = build(&mut ui, 400.0);
        let result = ui.end_with_layout_queries(|ui, sizes| {
            ui.nodes[container].el.width = Size::Fixed(if sizes[0].current.width > 200.0 {
                100.0
            } else {
                400.0
            });
            Ok(vec![container])
        });
        assert!(matches!(result, Err(SettlementError::Cycle(_))));
        assert!(ui.elements().is_empty());
        let mut ui = Ui::default();
        build(&mut ui, 400.0);
        assert!(matches!(
            ui.end_with_layout_queries(|_, _| Err("invalid container dependency"
                .to_owned()
                .into())),
            Err(SettlementError::Resolver(_))
        ));
        assert!(ui.elements().is_empty());
        build(&mut ui, 400.0);
        let mut recovered = false;
        ui.end_with_layout_queries(|_, sizes| {
            recovered = true;
            assert!(sizes[0].previous.is_none());
            Ok(Vec::new())
        })
        .unwrap();
        assert!(recovered);
        assert!(!ui.elements().is_empty());
    }
}
