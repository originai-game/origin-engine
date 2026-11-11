use super::*;

impl Ui {
    /// The average CSS fill beneath a part's label.
    pub(crate) fn part_ground(&self, part: &crate::dress::PartRef, fill: Srgba) -> Srgba {
        let values = self.part_values(part, None);
        self.part_draw_values(&values, [1.0, 1.0], Some(fill), None)
            .map(|draw| crate::dress::ground_of(&draw, &self.env.theme, Srgba::TRANSPARENT))
            .unwrap_or(Srgba::TRANSPARENT)
    }

    #[allow(clippy::too_many_arguments)]
    pub(crate) fn part_commands(
        &self,
        key: Option<u64>,
        part: &crate::dress::PartRef,
        rect: Rect,
        fill: Option<Srgba>,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        self.part_commands_at(key, part, rect, fill, 0.0, alpha, clip, cmds);
    }

    /// A part without a CSS radius retains its element's radius.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn part_commands_at(
        &self,
        key: Option<u64>,
        part: &crate::dress::PartRef,
        rect: Rect,
        fill: Option<Srgba>,
        radius: f32,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let values = self.part_values(part, key);
        let radius = values.num("radius").unwrap_or(radius);
        let Ok(draw) = self.part_draw_values(&values, [rect.width, rect.height], fill, None) else {
            return;
        };
        let at = crate::paint::Placed {
            rect,
            radius,
            clip: if self.paint.drawing.is_identity() {
                clip
            } else {
                self.paint.drawing.map_rect(&clip)
            },
            alpha,
        };
        cmds.extend(draw.enter(&self.env.theme, &at));
        cmds.extend(draw.children(&at));
        cmds.extend(draw.leave(&self.env.theme, &at));
    }

    pub(super) fn build_part_draws(&mut self, order: &[usize]) {
        let previous = std::mem::take(&mut self.paint.part_draws);
        self.paint.part_changed.clear();
        for &i in order {
            if self.tree.draws.contains_key(&i) && !self.tree.dressable.contains(&i) {
                continue;
            }
            let n = &self.tree.nodes[i];
            if n.part_box.is_some() || n.content_slot != NONE {
                continue;
            }
            let part = match (n.el.part, n.el.frame) {
                (Some(p), _) => p,
                (None, Some(f)) if n.sampled.background.is_some() => f.part_ref(),
                _ => continue,
            };
            let values = self.part_values(&part, Some(n.key));
            match self.part_draw_values(
                &values,
                [n.rect.width, n.rect.height],
                n.sampled.background,
                self.tree.draws.get(&i),
            ) {
                Ok(draw) => {
                    self.paint.part_draws.insert(i, draw);
                }
                Err(errors) => self.out.warnings.extend(errors),
            }
        }
        self.paint.part_changed.extend(
            previous
                .keys()
                .chain(self.paint.part_draws.keys())
                .copied()
                .filter(|i| previous.get(i) != self.paint.part_draws.get(i)),
        );
    }

    pub(crate) fn behind(&self, i: usize) -> Srgba {
        self.background_behind(i, false)
    }

    pub(crate) fn sampled_behind(&self, i: usize) -> Srgba {
        self.background_behind(i, true)
    }

    fn background_behind(&self, i: usize, sampled: bool) -> Srgba {
        let mut fills = Vec::new();
        let mut j = self.tree.nodes[i].parent;
        while j != NONE {
            let node = &self.tree.nodes[j];
            let fill = if sampled {
                node.sampled.background
            } else {
                node.el.background
            };
            let draw = if let Some(part) = node
                .el
                .part
                .or_else(|| node.el.frame.map(StylePart::part_ref))
            {
                let values = self.part_values(&part, Some(node.key));
                self.part_draw_values(&values, [node.rect.width, node.rect.height], fill, None)
                    .unwrap_or_default()
            } else {
                crate::paint::ElementDraw {
                    fills: fill
                        .into_iter()
                        .map(|c| crate::paint::Paint::Solid(crate::paint::ColorRef::Literal(c)))
                        .collect(),
                    ..Default::default()
                }
            };
            let draw = node.decor.as_ref().map_or_else(
                || draw.clone(),
                |decor| {
                    decor.over(
                        Some(&draw),
                        crate::doc::paint_css::LayerFrame {
                            size: [node.rect.width, node.rect.height],
                            viewport: [self.env.viewport.width, self.env.viewport.height],
                            ..Default::default()
                        },
                    )
                },
            );
            let ground = crate::dress::ground_of(&draw, &self.env.theme, Srgba::TRANSPARENT);
            fills.push(ground);
            if ground.alpha >= 1.0 {
                break;
            }
            j = node.parent;
        }
        let mut color = self.env.theme.palette.backdrop;
        for fill in fills.iter().rev() {
            color = fill.over(color);
        }
        color
    }
}

impl Ui {
    /// An ornament in `rect`, mirrored across x and or y.
    #[allow(clippy::too_many_arguments)]
    pub(crate) fn ornament(
        &self,
        o: Ornament,
        rect: Rect,
        flip_x: bool,
        flip_y: bool,
        tint: Option<[f32; 4]>,
        alpha: f32,
        clip: Rect,
        cmds: &mut Vec<DrawCmd>,
    ) {
        let Some(atlas) = self.env.theme.art.atlas else {
            return;
        };
        let [u0, v0, u1, v1] = o.uv;
        let (u0, u1) = if flip_x { (u1, u0) } else { (u0, u1) };
        let (v0, v1) = if flip_y { (v1, v0) } else { (v0, v1) };
        let mut color = tint.unwrap_or_else(|| match o.ink {
            Ink::Authored => Srgba::WHITE.linear(),
            ink => ink.color(&self.env.theme.palette).linear(),
        });
        color[3] *= alpha;
        cmds.push(DrawCmd::Image {
            rect,
            texture: Some(atlas),
            uv: [u0, v0, u1, v1],
            color,
            clip,
        });
    }

    pub(crate) fn draw_of(&self, i: usize) -> Option<&crate::paint::ElementDraw> {
        self.paint
            .decor_draws
            .get(&i)
            .or_else(|| self.paint.part_draws.get(&i))
            .or_else(|| self.tree.draws.get(&i))
    }
}
