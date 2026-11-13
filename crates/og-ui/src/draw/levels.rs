use super::*;
use og_draw::Overlay;

fn plain(cmds: Vec<DrawCmd>) -> Vec<(DrawCmd, Affine)> {
    cmds.into_iter().map(|c| (c, Affine::IDENTITY)).collect()
}

/// The levels as every command tested against every earlier one (what
/// the grid must answer).
fn quadratic(cmds: &[DrawCmd]) -> Vec<i32> {
    let mut placed: Vec<(Rect, Batch, i32)> = Vec::new();
    let mut out = Vec::new();
    for cmd in cmds {
        let bounds = cmd.bounds();
        if bounds.is_empty() {
            continue;
        }
        let batch = cmd.batch();
        let mut level = 0;
        for (r, b, l) in &placed {
            if r.overlaps(&bounds) {
                level = level.max(if *b == batch { *l } else { *l + 1 });
            }
        }
        placed.push((bounds, batch, level));
        out.push(level);
    }
    out
}

const ALL: Rect = Rect {
    x: -1e9,
    y: -1e9,
    width: 2e9,
    height: 2e9,
};

fn quad(rect: Rect) -> DrawCmd {
    DrawCmd::Quad {
        rect,
        radius: 0.0,
        color: [1.0; 4],
        clip: ALL,
    }
}

fn image(rect: Rect, slot: u32) -> DrawCmd {
    DrawCmd::Image {
        rect,
        texture: og_draw::TextureHandle::from_slot(slot),
        uv: [0.0, 0.0, 1.0, 1.0],
        color: [1.0; 4],
        clip: ALL,
    }
}

#[test]
fn the_grid_assigns_the_levels_of_every_pair_tested() {
    let mut seed = 0x2545_f491_4f6c_dd1d_u64;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f32 / (1u64 << 53) as f32
    };
    let mut cases: Vec<Vec<DrawCmd>> = Vec::new();
    // Random rectangles of two batches, some touching edge to edge, some
    // far off the frame, some empty or not finite.
    for n in [1, 7, 60, 400] {
        let cmds = (0..n)
            .map(|i| {
                let (x, y) = (rand() * 1400.0 - 60.0, rand() * 800.0 - 40.0);
                let (w, h) = (rand() * 300.0, rand() * 200.0);
                let r = match i % 23 {
                    0 => Rect::new(x, y, 0.0, h),
                    1 => Rect::new(f32::NAN, y, w, h),
                    2 => Rect::new(x, y, f32::INFINITY, h),
                    3 => Rect::new(-1e7, y, 2e7, h),
                    4 => Rect::new((x / 24.0).floor() * 24.0, 0.0, 24.0, 24.0),
                    _ => Rect::new(x, y, w, h),
                };
                if i % 3 == 0 { image(r, i % 2) } else { quad(r) }
            })
            .collect();
        cases.push(cmds);
    }
    // A screen under a dissolve's grid of cells, as a transition draws it.
    let mut screen = vec![quad(Rect::new(0.0, 0.0, 1280.0, 720.0))];
    screen.extend((0..20).map(|i| image(Rect::new(40.0 * i as f32, 30.0, 120.0, 40.0), 0)));
    for y in 0..30 {
        for x in 0..54 {
            screen.push(quad(Rect::new(
                x as f32 * 24.0,
                y as f32 * 24.0,
                24.0,
                24.0,
            )));
        }
    }
    cases.push(screen);
    for cmds in cases {
        let want = quadratic(&cmds);
        let got: Vec<i32> = assign_levels(plain(cmds)).iter().map(|i| i.level).collect();
        assert_eq!(got, want);
    }
}

fn begin() -> DrawCmd {
    DrawCmd::BeginLayer {
        material: UiMaterial::Plain,
        params: [[0.0; 4]; og_draw::UI_MATERIAL_PARAMS],
        mask: None,
        expand: 4.0,
        rect: Rect::default(),
        filter: None,
        role: Default::default(),
    }
}

fn end() -> DrawCmd {
    DrawCmd::EndLayer {
        geometry: None,
        rect: Rect::default(),
        opacity: 1.0,
        blend: UiBlend::Normal,
        clip: ALL,
        ground: None,
    }
}

/// Replacing a few commands and levelling only what they touch gives
/// the items levelling the whole list afresh gives: a command moved
/// away lowers what was stacked over it, transitively, and one moved
/// over others raises what follows; a layer and a clip keep their
/// places. A change inside the layer is refused.
#[test]
fn levelling_again_what_changed_matches_levelling_afresh() {
    let mut seed = 0x9e37_79b9_7f4a_7c15_u64;
    let mut rand = move || {
        seed ^= seed << 13;
        seed ^= seed >> 7;
        seed ^= seed << 17;
        (seed >> 11) as f32 / (1u64 << 53) as f32
    };
    let mut cmd = |i: usize| {
        let r = Rect::new(
            rand() * 900.0,
            rand() * 500.0,
            20.0 + rand() * 260.0,
            20.0 + rand() * 160.0,
        );
        if rand() < 0.4 {
            image(r, i as u32 % 2)
        } else {
            quad(r)
        }
    };
    let mut cmds: Vec<DrawCmd> = (0..120).map(&mut cmd).collect();
    cmds.splice(
        40..40,
        [
            begin(),
            quad(Rect::new(0.0, 0.0, 600.0, 400.0)),
            quad(Rect::new(10.0, 10.0, 50.0, 50.0)),
            end(),
        ],
    );
    cmds.insert(
        70,
        DrawCmd::PushClip {
            geometry: UiGeometry::Rect {
                radii: [0.0; 4],
                corner: og_draw::UiCorner::Round,
            },
            center: (300.0, 200.0),
            half: (200.0, 100.0),
        },
    );
    cmds.insert(90, DrawCmd::PopClip);
    let (mut items, mut levels) = assign_levels_kept(plain(cmds.clone()));
    let movable: Vec<usize> = (0..cmds.len())
        .filter(|&p| !(40..44).contains(&p) && p != 70 && p != 90)
        .collect();
    let mut pick = 0usize;
    for round in 0..40 {
        let mut changed = Vec::new();
        for _ in 0..1 + round % 4 {
            pick = (pick * 31 + 17 + round) % movable.len();
            let p = movable[pick];
            cmds[p] = cmd(p + round);
            changed.push((p, cmds[p].clone(), Affine::IDENTITY));
        }
        assert!(levels.update(&mut items, changed, 1.0));
        assert_eq!(items, assign_levels(plain(cmds.clone())), "round {round}");
    }
    let inside = vec![(42, quad(Rect::new(5.0, 5.0, 5.0, 5.0)), Affine::IDENTITY)];
    let before = items.clone();
    assert!(!levels.update(&mut items, inside, 1.0));
    assert_eq!(items, before);
}

/// A text that measured no width (its face lacks every glyph) goes on
/// as an item, so the renderer counts its missing glyphs; one clipped
/// out of view measured its width and goes like any command that draws
/// nothing, so a long list sends the renderer only what is in view.
#[test]
fn only_a_text_that_measured_no_width_goes_on_without_bounds() {
    let text = |bounds: Rect, clip: Rect| DrawCmd::Text {
        x: bounds.x,
        baseline: bounds.y + 12.0,
        size: 16.0,
        text: "\u{2022} item".into(),
        color: [1.0; 4],
        font: FontHandle::from_bits(1),
        bounds,
        clip,
        tracking: 0.0,
        weight: 0.0,
        glow: None,
        placements: Vec::new(),
    };
    let view = Rect::new(0.0, 0.0, 100.0, 100.0);
    let items = assign_levels(plain(vec![
        text(Rect::new(10.0, 10.0, 0.0, 16.0), view),
        text(Rect::new(10.0, 300.0, 60.0, 16.0), view),
        text(Rect::new(10.0, 30.0, 60.0, 16.0), view),
    ]));
    let ys: Vec<f32> = items
        .iter()
        .map(|i| match &i.cmd {
            DrawCmd::Text { bounds, .. } => bounds.y,
            c => panic!("{c:?}"),
        })
        .collect();
    assert_eq!(ys, [10.0, 30.0]);
}

/// A scroll that keeps the rows' levels and levels only what it brings
/// in, what it clips otherwise and what lies across the moved rows'
/// edge gives the items levelling the frame afresh gives: rows that
/// overlap their neighbours, an earlier header and a later scrollbar
/// across the list's edge, a backdrop under all of it and an overlay
/// over it, a header that goes and comes back, at offsets that move
/// exactly and fractions that round otherwise as they add up. A frame
/// that reorders last frame's commands is refused.
#[test]
fn splicing_a_scroll_matches_levelling_afresh() {
    let clip = Rect::new(100.0, 50.0, 400.0, 300.0);
    let row_quad = |rect: Rect| DrawCmd::Quad {
        rect,
        radius: 0.0,
        color: [1.0; 4],
        clip,
    };
    let row_image = |rect: Rect, slot: u32| DrawCmd::Image {
        rect,
        texture: og_draw::TextureHandle::from_slot(slot),
        uv: [0.0, 0.0, 1.0, 1.0],
        color: [1.0; 4],
        clip,
    };
    // Each command with what it is: 0 backdrop, 1 header, 2 + 2k + part
    // row k's part, then the scrollbar and the overlay.
    let frame = |off: f32| -> Vec<(u32, DrawCmd)> {
        let mut out = vec![(0, quad(Rect::new(90.0, 40.0, 420.0, 320.0)))];
        // The header goes once the list is scrolled far.
        if off < 45.0 {
            out.push((1, image(Rect::new(80.0, 30.0, 200.0, 45.0), 3)));
        }
        for k in 0..40u32 {
            let y = 50.0 + k as f32 * 30.0 - off;
            if y + 34.0 <= clip.y || y >= clip.bottom() {
                continue;
            }
            // A row's image ends where the next row begins.
            out.push((2 + 2 * k, row_quad(Rect::new(100.0, y, 400.0, 30.0))));
            let x = 110.0 + (k % 3) as f32 * 120.0;
            out.push((
                3 + 2 * k,
                row_image(Rect::new(x, y + 10.0, 150.0, 20.0), k % 2),
            ));
        }
        let thumb = 60.0 + off * 0.25;
        out.push((1000, image(Rect::new(490.0, thumb, 8.0, 60.0), 5)));
        out.push((1001, quad(Rect::new(300.0, 200.0, 40.0, 40.0))));
        out
    };
    let mut was = frame(0.0);
    let (mut items, mut levels) =
        assign_levels_kept(plain(was.iter().map(|(_, c)| c.clone()).collect()));
    let mut off = 0.0f32;
    let steps = [
        7.5, 30.0, 12.25, -9.75, 60.0, 0.0, -45.0, 0.5, 200.0, -100.0, -60.0, -60.0, 3.3, -9.7,
    ];
    // Fractions of a pixel that round as they add up.
    let fractions = (0..60).map(|i| [0.1, 0.7, -0.3][i % 3] * (1.0 + i as f32 / 7.0));
    for step in steps.into_iter().chain(fractions) {
        let now = frame(off + step);
        let next = now
            .iter()
            .map(|(id, c)| {
                let from = was.iter().position(|(w, _)| w == id);
                let row = (2..1000).contains(id);
                match from {
                    Some(o) if !row && was[o].1 == *c => Spliced {
                        from: Some(o),
                        delta: (0.0, 0.0),
                        cmd: None,
                    },
                    _ => Spliced {
                        from,
                        delta: if row { (0.0, -step) } else { (0.0, 0.0) },
                        cmd: Some((c.clone(), Affine::IDENTITY)),
                    },
                }
            })
            .collect();
        items = levels
            .splice(items, next, 1.0)
            .unwrap_or_else(|| panic!("step {step} refused"));
        let fresh = assign_levels(plain(now.iter().map(|(_, c)| c.clone()).collect()));
        assert_eq!(items, fresh, "step {step}");
        off += step;
        was = now;
    }
    // The backdrop moved after the header: not last frame's order.
    let mut next: Vec<Spliced> = (0..was.len())
        .map(|o| Spliced {
            from: Some(o),
            delta: (0.0, 0.0),
            cmd: None,
        })
        .collect();
    next.swap(0, 1);
    assert!(levels.splice(items, next, 1.0).is_none());
}

/// A layer levels its own commands from 0 and is one command of its
/// parent, over what it overlaps there; its rectangle is what it draws.
#[test]
fn a_layer_is_one_command_of_its_parent_with_levels_of_its_own() {
    let cmds = vec![
        quad(Rect::new(0.0, 0.0, 100.0, 100.0)),
        image(Rect::new(10.0, 10.0, 20.0, 20.0), 1),
        begin(),
        quad(Rect::new(20.0, 20.0, 40.0, 40.0)),
        image(Rect::new(25.0, 25.0, 10.0, 10.0), 1),
        quad(Rect::new(30.0, 30.0, 10.0, 10.0)),
        end(),
        quad(Rect::new(50.0, 50.0, 10.0, 10.0)),
    ];
    let items = assign_levels(plain(cmds));
    let levels: Vec<i32> = items.iter().map(|i| i.level).collect();
    // Outer: quad 0, image 1 (a texture is its own batch), layer 2 (over
    // the image), quad 3 (over the layer); inside: quad 0, image 1, quad 2.
    assert_eq!(levels, [0, 1, 2, 0, 1, 2, 2, 3]);
    let DrawCmd::BeginLayer { rect, .. } = &items[2].cmd else {
        panic!("{:?}", items[2]);
    };
    assert_eq!(*rect, Rect::new(16.0, 16.0, 48.0, 48.0));
}

/// The emitted layer captures its own range of overlay layers, and its
/// composite draws at its level in the parent; a cut adds a clip shape
/// for what is inside it, nested in the cut around it.
#[test]
fn layers_and_cuts_emit_their_ranges_and_shapes() {
    let rounded = |r: f32| UiGeometry::Rect {
        radii: [r; 4],
        corner: og_draw::UiCorner::Round,
    };
    let cmds = vec![
        quad(Rect::new(0.0, 0.0, 100.0, 100.0)),
        DrawCmd::PushClip {
            geometry: rounded(10.0),
            center: (50.0, 50.0),
            half: (50.0, 50.0),
        },
        DrawCmd::PushClip {
            geometry: rounded(4.0),
            center: (50.0, 50.0),
            half: (20.0, 20.0),
        },
        begin(),
        quad(Rect::new(40.0, 40.0, 20.0, 20.0)),
        end(),
        DrawCmd::PopClip,
        DrawCmd::PopClip,
        quad(Rect::new(0.0, 0.0, 10.0, 10.0)),
    ];
    let items = assign_levels(plain(cmds));
    let mut overlay = Overlay::new(200.0, 200.0, 1.0);
    emit(
        &items,
        &EmitDesc {
            layer: 100,
            time: 2.5,
        },
        &mut overlay,
        &|_| None,
        &[],
    );
    let ui = &overlay.ui;
    assert_eq!(ui.time, 2.5);
    assert_eq!(ui.layers.len(), 1);
    let base = 100 + LAYER_BASE;
    assert_eq!(ui.layers[0].capture, [base, base + LAYER_SPAN - 1]);
    // Background, the layer's quad, its composite, the corner quad.
    assert_eq!(ui.shapes.len(), 4);
    assert_eq!(ui.shapes[1].layer, base);
    assert_eq!(ui.shapes[2].paint, UiPaint::Layer(0));
    assert_eq!(ui.shapes[2].layer, 101);
    // Two cuts' own records, and the shape of each rectangle cut: the
    // layer's quad and its composite inside both cuts.
    let inner = ui.shapes[1].clip.expect("clipped");
    let shape = ui.clips.iter().find(|c| c.rect == inner).expect("its cut");
    assert_eq!(shape.geometry, rounded(4.0));
    let outer = shape.parent.expect("nested") as usize;
    assert_eq!(ui.clips[outer].geometry, rounded(10.0));
    // The corner quad, after both cuts end, is cut by neither.
    let after = ui.shapes[3].clip.expect("a rectangle");
    assert!(ui.clips.iter().all(|c| c.rect != after));
}

/// A path's bounds are its curves' under its transform, not its paint
/// box's: a heart whose point reaches 51 px below a 20 px box, turned
/// 25° and squashed to 60 % high, keeps its point inside what the
/// layer and the levels see.
#[test]
fn a_turned_squashed_path_keeps_its_point_in_its_bounds() {
    let mut path = og_draw::UiPath::new();
    path.move_to(Vec2::new(-60.0, -21.0))
        .cubic_to(
            Vec2::new(-60.0, -60.0),
            Vec2::new(60.0, -60.0),
            Vec2::new(60.0, -21.0),
        )
        .line_to(Vec2::new(0.0, 51.0))
        .close();
    let (s, c) = 25f32.to_radians().sin_cos();
    let mut shape = UiShape::rect([90.0, 90.0, 20.0, 20.0], 0.0, [1.0; 4]);
    shape.geometry = UiGeometry::Path(std::sync::Arc::new(path));
    shape.transform = [c, s, -s * 0.6, c * 0.6];
    let b = shape_bounds(&shape);
    // The point at local (0, 51) on the screen.
    let tip = Vec2::new(100.0 - s * 0.6 * 51.0, 100.0 + c * 0.6 * 51.0);
    assert!(
        b.x <= tip.x && tip.x <= b.right() && b.y <= tip.y && tip.y <= b.bottom(),
        "{b:?} misses the point at {tip}"
    );
    // And the left lobe at (-60, -21).
    let lobe = Vec2::new(
        100.0 - c * 60.0 + s * 0.6 * 21.0,
        100.0 - s * 60.0 - c * 0.6 * 21.0,
    );
    assert!(
        b.x <= lobe.x && lobe.y >= b.y,
        "{b:?} misses the lobe at {lobe}"
    );
}
