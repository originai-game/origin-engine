use super::*;

impl Ui {
    // ── HUD ────────────────────────────────────────────────────────────────

    /// An HTML `<meter>`: a bar of `value` between `min` and `max` whose
    /// fill is dressed by the region [`MeterRegion::of`] puts the value in
    /// (`Meter.fill[region=optimum|suboptimum|poor]`, undressed success,
    /// warn and danger) over `Meter.track`. Role meter with its range.
    pub fn gauge(&mut self, id: impl Into<Id>, desc: &GaugeDesc) {
        let t = self.env.theme;
        let p = t.palette;
        let (min, max) = (desc.min, desc.max.max(desc.min));
        let value = if desc.value.is_finite() {
            desc.value.clamp(min, max)
        } else {
            min
        };
        let region = MeterRegion::of(desc);
        let frac = if max > min {
            (value - min) / (max - min)
        } else {
            0.0
        };
        let i = self.leaf(
            "meter",
            id.into(),
            Element {
                width: desc.width.unwrap_or(Size::Fixed(160.0)),
                height: Size::Fixed(t.space.md),
                ..Element::default()
            },
        );
        let (name, color) = match region {
            MeterRegion::Optimum => ("optimum", p.success),
            MeterRegion::Suboptimum => ("suboptimum", p.warn),
            MeterRegion::Poor => ("poor", p.danger),
        };
        let mut shapes = Vec::new();
        if frac > 0.0 {
            shapes.push(Shape::Part {
                frame: Frame::left(0.0, frac),
                part: crate::dress::PartRef::new("Meter", "fill", crate::tokens::ElementClass::Bar)
                    .attr("region", name),
                fill: Some(color),
            });
        }
        self.bar_track(
            i,
            crate::dress::PartRef::new("Meter", "track", crate::tokens::ElementClass::Bar),
            p.control_strong,
            t.radius.bar,
            shapes,
        );
        let n = &mut self.tree.nodes[i];
        n.a11y.value = Some(format!("{value}"));
        n.a11y.range = Some(crate::a11y::A11yRange {
            min: Some(min),
            max: Some(max),
            now: value,
        });
    }
}
