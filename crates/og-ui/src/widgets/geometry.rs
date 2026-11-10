//! Derived widget geometry settles after CSS and layout, without replaying input.
use crate::layout::{PartGeometry, Size};
use crate::ui::settlement::{ContainerMeasurement, ContentBox};
use crate::{Prop, Ui};

impl Ui {
    pub(crate) fn settle_part_geometry(&mut self) -> (Vec<ContainerMeasurement>, Vec<usize>) {
        let mut measurements = Vec::new();
        let mut dirty = Vec::new();
        for host in self.kept.laid.clone() {
            let Some(geometry) = self.nodes[host].part_geometry.clone() else {
                continue;
            };
            match geometry {
                PartGeometry::Switch {
                    track,
                    knob,
                    mark,
                    tick,
                    measured,
                } => {
                    let [track, knob, mark, tick] = [track, knob, mark, tick]
                        .map(|key| self.tree.find(key).expect("owned switch part"));
                    let t = self.nodes[track].laid_rect;
                    let k = self.nodes[knob].laid_rect;
                    let current = [t.width, t.height, k.width, k.height];
                    if measured == Some(current) {
                        continue;
                    }
                    for (index, at) in [(track, 0), (knob, 2)] {
                        measurements.push(ContainerMeasurement {
                            key: self.nodes[index].key,
                            previous: measured.map(|v| ContentBox {
                                width: v[at],
                                height: v[at + 1],
                            }),
                            current: ContentBox {
                                width: current[at],
                                height: current[at + 1],
                            },
                        });
                    }
                    let Some(PartGeometry::Switch { measured, .. }) =
                        &mut self.nodes[host].part_geometry
                    else {
                        unreachable!()
                    };
                    *measured = Some(current);

                    let inset = (t.height * 0.11).round().max(2.0);
                    let diameter = (t.height - 2.0 * inset).max(0.0);
                    let travel = (t.width - k.width - 2.0 * inset)
                        * if self.direction() == og_i18n::Direction::Rtl {
                            -1.0
                        } else {
                            1.0
                        };
                    let part = self.nodes[knob].part_box.as_mut().expect("knob part");
                    part.element.width = Size::px(diameter);
                    part.element.height = Size::px(diameter);
                    part.element.radius = diameter / 2.0;
                    part.intrinsic_props.insets.start = Some(inset);
                    part.intrinsic_props.insets.top = Some(inset);
                    for group in &mut part.intrinsic_props.states {
                        if group.name == "selection" {
                            for state in &mut group.states {
                                if state.name == "on" {
                                    for (prop, value) in &mut state.values {
                                        if *prop == Prop::X {
                                            *value = travel;
                                        }
                                    }
                                }
                            }
                        }
                    }
                    self.apply_part_appearance(knob);
                    let tick_size = (k.width.min(k.height) * 0.62).round();
                    for (index, width, height) in
                        [(mark, k.width, k.height), (tick, tick_size, tick_size)]
                    {
                        if !self.emitted.contains(&index) {
                            self.kept
                                .prev
                                .entry(index)
                                .or_insert_with(|| self.nodes[index].clone());
                            self.emitted.push(index);
                        }
                        self.nodes[index].el.width = Size::px(width);
                        self.nodes[index].el.height = Size::px(height);
                    }
                    let draw = self.draws.get_mut(&tick).expect("switch tick draw");
                    if let crate::paint::ShapeKind::Path { points, width, .. } = &mut draw.shape {
                        *points = [[0.2, 0.53], [0.42, 0.74], [0.8, 0.28]]
                            .map(|p| [p[0] * tick_size, p[1] * tick_size])
                            .to_vec();
                        *width = (k.width.min(k.height) / 9.0).max(1.5);
                    }
                    dirty.extend([knob, mark, tick]);
                }
            }
        }
        (measurements, dirty)
    }
}
