use super::*;

impl Ui {
    // ── inputs from the host ─────────────────────────────────────────────

    /// The camera this tick's anchored elements project with and panel
    /// rays start from (the host sets it before [`Ui::end`]; `None`: none,
    /// so anchors are hidden).
    pub fn set_world_view(&mut self, view: Option<WorldView>) {
        self.world.view = view;
    }

    pub fn world_view(&self) -> Option<WorldView> {
        self.world.view
    }

    /// The entities' world poses (position, rotation, scale) for anchors and
    /// panels on entities: replaces the last ones.
    pub fn set_world_poses(&mut self, poses: &[(u64, Pose)]) {
        self.world.poses = poses.iter().copied().collect();
    }

    /// The anchored elements the last tick placed, with what they are on.
    pub fn world_anchor_points(&self) -> &[AnchorPoint] {
        &self.world.placed
    }

    /// The entities this frame's anchors and panels are on, for the host to
    /// hand their poses in before [`Ui::end`].
    pub fn world_entities(&self) -> Vec<u64> {
        let mut v = self.world.wanted.clone();
        v.sort_unstable();
        v
    }

    // ── anchored elements ────────────────────────────────────────────────

    /// Where an anchor stands as of now: its distance and level from the
    /// current view, the rest from last tick.
    fn anchor_info(&self, key: u64, desc: &WorldAnchorDesc) -> AnchorInfo {
        let mut info = self.world.last.get(&key).copied().unwrap_or_default();
        if let (Some(view), Some((p, _))) = (self.world.view, self.world.pose_of(desc.target)) {
            let p = p + desc.offset;
            info.distance = (p - view.eye).length();
            info.lod = lod_of(&desc.lod, info.distance);
        }
        info
    }

    /// Opens an anchored element; its children follow until [`Ui::close`].
    pub fn open_world_anchor(&mut self, id: impl Into<Id>, desc: WorldAnchorDesc) -> AnchorInfo {
        self.open_world_anchor_keyed(id.into(), None, desc)
    }

    /// The world layer of an anchor or panel: keyed from its id, or by
    /// `key` itself, `id` then being the full path `key` is the key of (a
    /// document's element: its subtree is keyed and named as if in place).
    fn open_world_layer(
        &mut self,
        kind: &'static str,
        id: Id,
        key: Option<u64>,
        el: Element,
    ) -> usize {
        match key {
            Some(key) => {
                self.open_layer_keyed(kind, key, id.as_str().to_owned(), el, Float::World, NONE)
            }
            None => self.open_layer(kind, id, el, Float::World, NONE),
        }
    }

    /// [`Ui::open_world_anchor`], its layer keyed by `key` when given.
    pub(crate) fn open_world_anchor_keyed(
        &mut self,
        id: Id,
        key: Option<u64>,
        desc: WorldAnchorDesc,
    ) -> AnchorInfo {
        let mut el = desc.element;
        // Laid out at its own size: never stretched to the screen.
        if matches!(el.width, Size::Grow { .. } | Size::Percent(_)) {
            el.width = Size::fit();
        }
        if matches!(el.height, Size::Grow { .. } | Size::Percent(_)) {
            el.height = Size::fit();
        }
        let i = self.open_world_layer("world", id, key, el);
        self.world.want(desc.target);
        let info = self.anchor_info(self.nodes[i].key, &desc);
        self.world.anchors.push(AnchorEntry { node: i, desc });
        info
    }

    /// Places again the anchors of mounted documents that were not built
    /// this tick (a retained element keeps its node, not its registration),
    /// and forgets those no longer mounted. Work: one lookup per anchor,
    /// which follows the camera every tick anyway.
    pub(crate) fn keep_world_anchors(&mut self) {
        let built: std::collections::BTreeSet<u64> = self
            .world
            .anchors
            .iter()
            .map(|a| self.nodes[a.node].key)
            .collect();
        let mut kept = std::mem::take(&mut self.world.kept);
        kept.retain(|key, (desc, owner)| {
            if built.contains(key) {
                return true;
            }
            match self.tree.by_key.get(key) {
                Some(&node) if self.docs.retained.owns(*owner) => {
                    self.world.want(desc.target);
                    self.world.anchors.push(AnchorEntry {
                        node,
                        desc: desc.clone(),
                    });
                    true
                }
                _ => false,
            }
        });
        self.world.kept = kept;
        let built: std::collections::BTreeSet<u64> = self
            .world
            .panels
            .iter()
            .map(|p| self.nodes[p.node].key)
            .collect();
        let mut kept = std::mem::take(&mut self.world.kept_panels);
        kept.retain(|key, (desc, rect, owner)| {
            if built.contains(key) {
                return true;
            }
            match self.tree.by_key.get(key) {
                Some(&node) if self.docs.retained.owns(*owner) => {
                    if let Some((t, ..)) = desc.quad {
                        self.world.want(t);
                    }
                    self.world.panels.push(PanelEntry {
                        node,
                        desc: desc.clone(),
                        rect: *rect,
                    });
                    true
                }
                _ => false,
            }
        });
        self.world.kept_panels = kept;
    }

    /// Keeps a document's world panel `node` for placement and rays on the
    /// ticks it is not built again, while the retained element `owner`
    /// stands (see [`Ui::keep_world_anchors`]).
    pub(crate) fn keep_world_panel(&mut self, node: usize, owner: u64) {
        let key = self.nodes[node].key;
        if let Some(p) = self.world.panels.iter().rev().find(|p| p.node == node) {
            let kept = (p.desc.clone(), p.rect, owner);
            self.world.kept_panels.insert(key, kept);
        }
    }

    /// An anchored element built by `f`, which reads where it stands (its
    /// level of detail, its distance, whether it is clamped).
    pub fn world_anchor(
        &mut self,
        id: impl Into<Id>,
        desc: WorldAnchorDesc,
        f: impl FnOnce(&mut Ui, &AnchorInfo),
    ) -> AnchorInfo {
        let info = self.open_world_anchor(id, desc);
        f(self, &info);
        self.close();
        info
    }

    // ── world panels ─────────────────────────────────────────────────────

    /// Opens a world panel; its children follow until [`Ui::close`].
    pub fn open_world_panel(&mut self, id: impl Into<Id>, desc: WorldPanelDesc) -> PanelInfo {
        self.open_world_panel_keyed(id.into(), None, desc)
    }

    /// [`Ui::open_world_panel`], its layer keyed by `key` when given.
    pub(crate) fn open_world_panel_keyed(
        &mut self,
        id: Id,
        key: Option<u64>,
        desc: WorldPanelDesc,
    ) -> PanelInfo {
        let mut el = desc.element;
        el.width = Size::Fixed(desc.width.max(1.0));
        el.height = Size::Fixed(desc.height.max(1.0));
        let k = self.world.panels.len() as f32;
        let rect = Rect::new(
            PANEL_X + k * PANEL_SLOT,
            0.0,
            desc.width.max(1.0),
            desc.height.max(1.0),
        );
        let i = self.open_world_layer("panel", id, key, el);
        if let Some((t, ..)) = desc.quad {
            self.world.want(t);
        }
        let key = self.nodes[i].key;
        self.world.panels.push(PanelEntry {
            node: i,
            desc,
            rect,
        });
        match self.world.pointed {
            Some((k, info)) if k == key => info,
            _ => PanelInfo::default(),
        }
    }

    /// A world panel built by `f`.
    pub fn world_panel(
        &mut self,
        id: impl Into<Id>,
        desc: WorldPanelDesc,
        f: impl FnOnce(&mut Ui, &PanelInfo),
    ) -> PanelInfo {
        let info = self.open_world_panel(id, desc);
        f(self, &info);
        self.close();
        info
    }

    // ── mass elements ────────────────────────────────────────────────────

    /// Declares a mass element template (configuration, like a timeline): a
    /// second declaration of a name replaces it and keeps its handle.
    /// Text and numbers without a font take the UI's body font.
    pub fn define_mass(
        &mut self,
        name: &str,
        template: UiMassTemplate,
        space: UiMassSpace,
    ) -> MassHandle {
        let mut t = template;
        if t.font.is_none() {
            t.font = self.fonts.body.handle;
        }
        let t = Arc::new(t);
        if let Some(k) = self.world.templates.iter().position(|(n, ..)| n == name) {
            self.world.templates[k] = (name.to_owned(), t, space);
            return MassHandle(k as u32);
        }
        self.world.templates.push((name.to_owned(), t, space));
        MassHandle(self.world.templates.len() as u32 - 1)
    }

    /// A template by name.
    pub fn mass_handle(&self, name: &str) -> Option<MassHandle> {
        self.world
            .templates
            .iter()
            .position(|(n, ..)| n == name)
            .map(|k| MassHandle(k as u32))
    }

    /// Starts an instance of a template on this tick: it plays its
    /// template's timeline over its lifetime and leaves the pool after it.
    pub fn spawn_mass(&mut self, h: MassHandle, instance: UiMassInstance) {
        let Some((_, t, _)) = self.world.templates.get(h.0 as usize) else {
            self.warnings
                .push(format!("spawn_mass: no template {}; define it first", h.0));
            return;
        };
        if t.lifetime == 0 {
            // No life to play: shown this tick only.
            self.push_mass(h, instance);
            return;
        }
        let pool = self.world.state.pools.entry(h.0).or_default();
        if pool.len() >= MASS_POOL_LIMIT {
            pool.remove(0);
        }
        pool.push(UiMassInstance {
            start: self.state.tick as u32,
            ..instance
        });
    }

    /// Shows an instance of a template this tick only (an overhead bar, a
    /// nameplate, a status icon: rebuilt every tick with its values).
    pub fn push_mass(&mut self, h: MassHandle, instance: UiMassInstance) {
        if self.world.templates.get(h.0 as usize).is_none() {
            self.warnings
                .push(format!("push_mass: no template {}; define it first", h.0));
            return;
        }
        self.world
            .immediate
            .entry(h.0)
            .or_default()
            .push(UiMassInstance {
                start: self.state.tick as u32,
                ..instance
            });
    }

    /// The frame's own mass elements (the templates' pools and this tick's
    /// instances), to emit with [`WorldFrame::emit`] beside the UI frame.
    pub fn world_frame(&self) -> &WorldFrame {
        &self.world.frame
    }

    /// The live instances a template's pool holds.
    pub fn mass_count(&self, h: MassHandle) -> usize {
        self.world.state.pools.get(&h.0).map_or(0, Vec::len)
    }

    // ── meta UI ──────────────────────────────────────────────────────────

    /// A meta layer: full screen, over the scene and under the screen's
    /// elements, never taking input.
    pub fn meta(&mut self, id: impl Into<Id>, desc: MetaDesc) {
        let vp = self.viewport;
        let el = Element {
            width: Size::Fixed(vp.width),
            height: Size::Fixed(vp.height),
            ..Element::default()
        };
        let i = self.open_layer("meta", id.into(), el, Float::World, NONE);
        self.nodes[i].props.inert = true;
        self.close();
        self.world.metas.push((i, desc));
    }

    // ── the frame ────────────────────────────────────────────────────────

    /// Starts the world part of a frame: the pointer retargeted onto a
    /// world panel its ray meets, before anything is hit tested.
    pub(crate) fn world_begin(&mut self, input: &mut crate::input::InputFrame) {
        let w = &mut self.world;
        w.last_pointer = self.state.pointer;
        w.wanted.clear();
        w.anchors.clear();
        w.panels.clear();
        w.emitters.clear();
        w.metas.clear();
        w.immediate.clear();
        w.marks.clear();
        w.pointed = None;
        let Some(view) = w.view else {
            return;
        };
        if !input.pointer_inside || w.last_panels.is_empty() {
            return;
        }
        // The screen's own elements are over the world: a pointer on one
        // stays there.
        let p = input.pointer;
        let on_screen = self.state.hits.iter().any(|h| {
            !off_screen(&h.rect) && h.contains(p.x, p.y) && h.flags & crate::ui::HIT_POINTER != 0
        });
        if on_screen {
            return;
        }
        let vp = self.viewport;
        let (o, d) = view.ray(p.x, p.y, vp.width, vp.height);
        let mut best: Option<(f32, PanelHit, Vec2)> = None;
        for ph in &w.last_panels {
            let n = ph.rotation * Vec3::Z;
            let den = n.dot(d);
            if den.abs() < 1e-6 {
                continue;
            }
            let t = n.dot(ph.center - o) / den;
            if t <= 0.0 || best.is_some_and(|(bt, ..)| bt <= t) {
                continue;
            }
            let local = ph.rotation.inverse() * (o + d * t - ph.center);
            let u = local.x / ph.size.x + 0.5;
            let v = 0.5 - local.y / ph.size.y;
            if (0.0..=1.0).contains(&u) && (0.0..=1.0).contains(&v) {
                best = Some((t, *ph, Vec2::new(u, v)));
            }
        }
        if let Some((t, ph, uv)) = best {
            let at = Vec2::new(
                ph.rect.x + uv.x * ph.rect.width,
                ph.rect.y + uv.y * ph.rect.height,
            );
            input.pointer = at;
            w.pointed = Some((
                ph.key,
                PanelInfo {
                    pointed: true,
                    x: uv.x * ph.rect.width,
                    y: uv.y * ph.rect.height,
                    distance: t,
                },
            ));
        }
    }

    /// Places this frame's world layers once laid out: anchors on their
    /// projected points (scaled, faded, decluttered, clamped), panels in
    /// their targets, meta layers over the screen. Called by `end` before
    /// transforms are applied.
    pub(crate) fn world_place(&mut self) {
        self.world_anchors();
        self.world_panels();
        self.world_metas();
    }

    /// Spawns particles where their elements are drawn, ages the pools and
    /// makes the frame's own batches. Called by `end` once transforms are
    /// applied.
    pub(crate) fn world_finish(&mut self) {
        let tick = self.state.tick as u32;
        self.world_particles(tick);
        // Pools: instances past their life leave.
        let templates = &self.world.templates;
        self.world.state.pools.retain(|&k, pool| {
            let Some((_, t, _)) = templates.get(k as usize) else {
                return false;
            };
            pool.retain(|i| tick.wrapping_sub(i.start) < t.lifetime);
            !pool.is_empty()
        });
        // The frame's own batches: pools, then this tick's instances.
        let mut batches = Vec::new();
        for (k, (_, t, space)) in self.world.templates.iter().enumerate() {
            let k = k as u32;
            let mut instances = self.world.state.pools.get(&k).cloned().unwrap_or_default();
            if let Some(now) = self.world.immediate.get(&k) {
                instances.extend_from_slice(now);
            }
            if instances.is_empty() {
                continue;
            }
            let mut b = UiMassBatch {
                template: t.clone(),
                space: *space,
                instances,
                layer: 0,
                clip: None,
            };
            // Drawn in CSS px like the frame's commands (the og-ui unit).
            crate::draw::zoom_mass(&mut b, self.unit);
            batches.push(b);
        }
        self.world.frame = WorldFrame { tick, batches };
    }
}

impl Ui {
    // ── state ────────────────────────────────────────────────────────────

    /// The world state's bytes: spawned mass elements and particles, each
    /// anchor's eased values. The host keeps them beside the UI state.
    pub fn save_world_state(&self) -> Vec<u8> {
        let s = &self.world.state;
        let mut out = Vec::new();
        out.extend_from_slice(b"OGUW");
        out.extend_from_slice(&1u32.to_le_bytes());
        let put = |out: &mut Vec<u8>, v: u64| out.extend_from_slice(&v.to_le_bytes());
        put(&mut out, s.pools.len() as u64);
        for (k, pool) in &s.pools {
            put(&mut out, u64::from(*k));
            put(&mut out, pool.len() as u64);
            out.extend_from_slice(bytemuck::cast_slice(pool));
        }
        put(&mut out, s.particles.len() as u64);
        for (k, e) in &s.particles {
            put(&mut out, *k);
            e.save(&mut out);
        }
        put(&mut out, s.smooth.len() as u64);
        for (k, v) in &s.smooth {
            put(&mut out, *k);
            for f in [v.alpha, v.dx, v.dy] {
                out.extend_from_slice(&f.to_le_bytes());
            }
        }
        out
    }

    /// Moves the world-space pools' instances by `d` (og.WorldOrigin moved
    /// the world's zero): every pool whose template is not in the screen's
    /// space. Emitters' particles and anchors' eased offsets are CSS px.
    pub fn shift_world(&mut self, d: og_core::glam::Vec3) {
        let d = d.as_dvec3();
        for (k, pool) in self.world.state.pools.iter_mut() {
            let world = self
                .world
                .templates
                .get(*k as usize)
                .is_some_and(|(_, _, space)| *space != UiMassSpace::Screen);
            if !world {
                continue;
            }
            for i in pool {
                let p = og_core::glam::Vec3::from_array(i.position).as_dvec3() + d;
                i.position = p.as_vec3().to_array();
            }
        }
    }

    /// Restores [`Ui::save_world_state`]'s bytes (empty: the empty state).
    pub fn restore_world_state(&mut self, bytes: &[u8]) -> og_core::Result<()> {
        self.world.state = decode_world(bytes).ok_or_else(|| {
            og_core::Error::new(
                og_core::ErrorKind::Parse,
                "og.ui world state: unreadable bytes",
            )
        })?;
        Ok(())
    }
}

/// Reads a world state's bytes.
fn decode_world(bytes: &[u8]) -> Option<WorldState> {
    let mut s = WorldState::default();
    if bytes.is_empty() {
        return Some(s);
    }
    let mut r = Reader { b: bytes, at: 0 };
    if r.take(4)? != b"OGUW" || r.u32()? != 1 {
        return None;
    }
    for _ in 0..r.u64()? {
        let k = r.u64()? as u32;
        let n = r.u64()? as usize;
        let size = std::mem::size_of::<UiMassInstance>();
        let raw = r.take(n.checked_mul(size)?)?;
        let pool: Vec<UiMassInstance> = raw
            .chunks_exact(size)
            .map(bytemuck::pod_read_unaligned)
            .collect();
        s.pools.insert(k, pool);
    }
    for _ in 0..r.u64()? {
        let k = r.u64()?;
        s.particles
            .insert(k, crate::particles::EmitterState::load(&mut r)?);
    }
    for _ in 0..r.u64()? {
        let k = r.u64()?;
        s.smooth.insert(
            k,
            Smooth {
                alpha: r.f32()?,
                dx: r.f32()?,
                dy: r.f32()?,
            },
        );
    }
    (r.at == bytes.len()).then_some(s)
}

/// Little-endian reads over a byte slice.
pub(crate) struct Reader<'a> {
    pub b: &'a [u8],
    pub at: usize,
}

impl Reader<'_> {
    pub fn take(&mut self, n: usize) -> Option<&[u8]> {
        let s = self.b.get(self.at..self.at.checked_add(n)?)?;
        self.at += n;
        Some(s)
    }
    pub fn u32(&mut self) -> Option<u32> {
        Some(u32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
    pub fn u64(&mut self) -> Option<u64> {
        Some(u64::from_le_bytes(self.take(8)?.try_into().ok()?))
    }
    pub fn f32(&mut self) -> Option<f32> {
        Some(f32::from_le_bytes(self.take(4)?.try_into().ok()?))
    }
}
