//! The inspector's frame: its panel built, the gizmo placed and dragged, watches read, and what it draws.

use super::*;

/// Builds the panel for this frame and does what it asked: between ticks,
/// before the frame is drawn.
pub(crate) fn frame(eng: &Engine) {
    // Closed (as in every run that never opens it), it costs nothing.
    let open = eng
        .render
        .try_borrow()
        .is_ok_and(|rs| rs.inspector.as_ref().is_some_and(|i| i.open));
    if !open {
        return;
    }
    let viewport = crate::bind::modules::viewport_of(eng.id);
    let Ok(world) = eng.world() else {
        return;
    };
    let tick = world.time().tick;
    let paused = eng.probe.borrow().paused();
    let layouts = eng.state.borrow().layouts.clone();
    let (sounds, assets) = handle_names(eng);
    let camera = crate::host::api::resource_value(eng, "og.Camera").ok();
    let requests = {
        let Ok(mut rs) = eng.render.try_borrow_mut() else {
            return;
        };
        let fonts = rs.ui.as_ref().map(|u| {
            let u = u.lock();
            (u.fonts().clone(), u.icons().clone())
        });
        let game = rs.ui.clone();
        let Some(ins) = rs.inspector.as_mut().filter(|i| i.open) else {
            return;
        };
        if let Some((fonts, icons)) = fonts
            && let Some(ui) = ins.ui.as_mut()
            && *ui.fonts() != fonts
        {
            ui.set_fonts(fonts);
            ui.set_icons(icons);
        }
        ins.frames += 1;
        // The UI view reads the game's UI only while it is the tab shown.
        ins.ui_view = match game.filter(|_| ins.tab == ui_debug::TAB) {
            Some(game) => ui_debug::read(ins, &game.lock()),
            None => ui_debug::View::default(),
        };
        if ins.selected.is_some_and(|e| !world.is_alive(e)) {
            ins.selected = None;
        }
        let probe = eng.probe.borrow();
        if let Some((x, y)) = ins.pick_at.take() {
            drop(probe);
            let picked = eng
                .probe
                .borrow_mut()
                .pick(world, x, y)
                .and_then(|v| v["entity"].as_str().map(str::to_string));
            let probe = eng.probe.borrow();
            match picked.and_then(|id| probe.names().resolve(world, &id)) {
                Some(e) => {
                    ins.selected = Some(e);
                    ins.status = format!("Picked {}", probe.names().id_of(world, e));
                }
                None => ins.status = "Nothing under the pointer".into(),
            }
        }
        let probe = eng.probe.borrow();
        watch(ins, world, &layouts, tick);
        if ins.prefab.as_ref().map(|(e, _)| *e) != ins.selected {
            ins.prefab = ins.selected.map(|e| (e, probe.entity_prefab(world, e)));
        }
        let prefab = ins.prefab.take().filter(|(e, _)| Some(*e) == ins.selected);
        panel::build(
            ins,
            &panel::Ctx {
                world,
                names: probe.names(),
                layouts: &layouts,
                paused,
                tick,
                viewport,
                sounds: &sounds,
                assets: &assets,
                prefab: prefab.as_ref().and_then(|(_, v)| v.as_ref()),
            },
        );
        if ins.prefab.is_none() {
            ins.prefab = prefab;
        }
        if let Some(id) = ins.pending_select.take() {
            ins.selected = probe.names().resolve(world, &id).or(ins.selected);
            ins.tab = 0;
        }
        ins.tool.view = camera.map(|c| gizmo::view_of(&c, viewport.width, viewport.height));
        place(ins, world);
        drag(ins, world, probe.names());
        std::mem::take(&mut ins.requests)
    };
    for r in requests {
        apply(eng, r);
    }
}

/// og.Transform's pose of an entity, if it has one.
pub(super) fn local_pose(world: &World, e: Entity) -> Option<Pose> {
    let id = world.id_by_name("og.Transform").ok()?;
    let schema = world.schema(id).ok()?;
    let bytes = world.get_bytes(e, id).ok()?;
    let layout = og_core::reflect::Layout::of(schema);
    let lanes = |name: &str, n: usize| -> Option<Vec<f32>> {
        let b = bytes.get(layout.range(schema.field_index(name)?))?;
        (0..n)
            .map(|k| {
                Some(f32::from_le_bytes(
                    b.get(k * 4..k * 4 + 4)?.try_into().ok()?,
                ))
            })
            .collect()
    };
    let p = lanes("position", 3)?;
    let q = lanes("quaternion", 4)?;
    let s = lanes("scale", 3)?;
    Some((
        Vec3::new(p[0], p[1], p[2]),
        Quat::from_xyzw(q[0], q[1], q[2], q[3]).normalize(),
        Vec3::new(s[0], s[1], s[2]),
    ))
}

/// The world pose of an entity's parent, as simulation composes it
/// (og_core::math::world_pose); none for a root.
pub(super) fn parent_pose(world: &World, e: Entity) -> Option<Pose> {
    let p = model::parent_of(world, e)?;
    let local = local_pose(world, p)?;
    Some(og_core::math::world_pose(
        p,
        local,
        |k| model::parent_of(world, k),
        |k| local_pose(world, k),
    ))
}

/// The selected entity's gizmo this frame (none without og.Transform or a
/// camera).
pub(super) fn place(ins: &mut Inspector, world: &World) {
    let tool = &mut ins.tool;
    tool.on = None;
    let (Some(e), Some(view)) = (ins.selected, tool.view) else {
        tool.hover = None;
        return;
    };
    let Some(local) = local_pose(world, e) else {
        tool.hover = None;
        return;
    };
    let parent = parent_pose(world, e);
    let world_pose = match parent {
        Some((pp, pq, ps)) => (
            pp + pq * (ps * local.0),
            (pq * local.1).normalize(),
            ps * local.2,
        ),
        None => local,
    };
    let g = gizmo::Gizmo::new(&view, tool.mode, tool.space, world_pose.0, world_pose.1);
    // What the pointer rests on, as this frame's gizmo has it (the mode
    // may have changed under a still pointer).
    let at = ins.input.pointer;
    let over = ins
        .panel
        .is_some_and(|[px, py, w, h]| at.x >= px && at.y >= py && at.x < px + w && at.y < py + h);
    tool.hover = match &tool.drag {
        Some(d) => Some(d.drag.handle),
        None if !over => g.hit(&view, Vec2::new(at.x, at.y)),
        None => None,
    };
    tool.on = Some(On {
        entity: e,
        gizmo: g,
        world: world_pose,
        local,
        parent,
    });
}

/// A drag's `set` for this frame, when the pointer moved since the last:
/// the one og.Transform field its mode writes, folded into the drag's
/// first step.
pub(super) fn drag(ins: &mut Inspector, world: &World, names: &og_probe::Names) {
    let snap = ins.tool.ctrl;
    let Some(d) = ins.tool.drag.as_mut() else {
        return;
    };
    let released = d.released;
    if world.is_alive(d.on.entity)
        && (d.written != Some(d.at))
        && (d.steps > 0 || d.at != d.drag.grab)
    {
        let w = d.drag.pose(&d.view, d.at, snap);
        let local = gizmo::to_local(d.on.parent, w);
        let mode = d.drag.gizmo.mode;
        let text = |pose: Pose| match mode {
            gizmo::Mode::Translate => lanes(&pose.0.to_array()),
            gizmo::Mode::Rotate => lanes(&pose.1.to_array()),
            gizmo::Mode::Scale => lanes(&pose.2.to_array()),
        };
        let id = names.id_of(world, d.on.entity);
        let edit = Edit {
            target: format!("{id}.og.Transform.{}", mode.field()),
            before: text(d.on.local),
            after: text(local),
        };
        let merge = d.steps > 0;
        d.steps += 1;
        d.written = Some(d.at);
        ins.requests.push(Request::Tune { edit, merge });
    }
    if released {
        ins.tool.drag = None;
    }
}

/// Numbers as `set` takes a vector's text.
pub(super) fn lanes(v: &[f32]) -> String {
    v.iter()
        .map(|x| format!("{x}"))
        .collect::<Vec<_>>()
        .join(",")
}

/// The ids of the engine's loaded sounds and assets by handle, for the
/// handle fields the panel shows.
pub(super) fn handle_names(eng: &Engine) -> (BTreeMap<u64, String>, BTreeMap<u64, String>) {
    let Ok(a) = eng.assets.try_borrow() else {
        return Default::default();
    };
    let sounds = a
        .sound_handles
        .iter()
        .zip(&a.sounds)
        .map(|(h, s)| (*h, s.id.clone()))
        .collect();
    let assets = a
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (crate::world::assets::handle(i), e.path.clone()))
        .collect();
    (sounds, assets)
}

/// Reads again the fields edited since a tick ran: one the game wrote
/// back flashes.
pub(super) fn watch(
    ins: &mut Inspector,
    world: &World,
    layouts: &BTreeMap<String, crate::host::codec::Layout>,
    tick: u64,
) {
    let frames = ins.frames;
    ins.flash.retain(|_, until| *until > frames);
    let mut keep = Vec::new();
    for w in std::mem::take(&mut ins.watch) {
        if tick <= w.tick {
            keep.push(w);
            continue;
        }
        let now = panel::read_parts(world, layouts, w.entity, &w.component, &w.field);
        if now.as_ref().is_some_and(|n| *n != w.shown) {
            ins.flash.insert(w.target.clone(), frames + 120);
        } else if tick < w.tick + 30 {
            // The game may write it on a later tick (a system that runs
            // every few): keep looking for half a second.
            keep.push(w);
        }
    }
    ins.watch = keep;
}

/// Draws the gizmo and the selected meshes' box over the frame.
pub(super) fn emit_gizmo(rs: &mut RenderState) {
    let Some(ins) = rs.inspector.as_ref().filter(|i| i.open) else {
        return;
    };
    let tool = &ins.tool;
    let (Some(view), Some(on)) = (tool.view, tool.on) else {
        return;
    };
    let Some(p) = ins.ui.as_ref().map(|u| u.theme().palette) else {
        return;
    };
    let colors = [p.danger, p.success, p.info, p.ink].map(|c| c.linear());
    let lit = tool.drag.as_ref().map(|d| d.drag.handle).or(tool.hover);
    let strokes = on.gizmo.strokes(&view, colors, lit, p.highlight.linear());
    let bits = on.entity.to_bits();
    let boxed = p.focus.linear();
    let instances: Vec<(og_render::MeshHandle, glam::Mat4)> = rs
        .ids
        .instances
        .iter()
        .zip(&rs.scene.objects.instances)
        .filter(|(e, _)| **e == bits)
        .map(|(_, i)| (i.mesh, i.transform))
        .collect();
    let mut bounds = Vec::new();
    for (mesh, m) in instances {
        if let Some((min, max)) = crate::present::geometry::mesh_bounds(rs, mesh) {
            bounds.extend(gizmo::box_lines(m, min, max));
        }
    }
    if !bounds.is_empty() {
        rs.scene
            .effects
            .lines
            .push(og_render::debug::segments(bounds, boxed, 1.0));
    }
    for (points, color, width) in strokes {
        rs.scene
            .effects
            .lines
            .push(og_render::debug::segments(points, color, width));
    }
}

/// Draws the panel over the frame (the last UI frame it built).
pub(crate) fn emit(rs: &mut RenderState) {
    emit_gizmo(rs);
    let Some(ui) = rs
        .inspector
        .as_ref()
        .filter(|i| i.open)
        .and_then(|i| i.ui.as_ref())
    else {
        return;
    };
    let frame = ui.frame();
    if frame.items.is_empty() {
        return;
    }
    let (w, h) = (
        frame.viewport.width * frame.unit,
        frame.viewport.height * frame.unit,
    );
    let time = ui.seconds() as f32;
    let overlay = rs
        .scene
        .ui
        .overlay
        .get_or_insert_with(|| og_render::Overlay::new(w.max(1.0), h.max(1.0), 0.0));
    let first = overlay.texts.len();
    frame.emit(&og_ui::EmitDesc { layer: LAYER, time }, overlay);
    crate::present::extract::draw_other_pages(overlay, first, &rs.ui_pages);
}
