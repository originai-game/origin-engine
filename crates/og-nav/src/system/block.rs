use super::*;

/// What a snapshot holds: the paths, and the registry's edits.
#[derive(Debug)]
struct Saved {
    paths: IdMap<u64, PathState>,
    grids: Vec<(u64, Vec<u8>)>,
    obstacles: Vec<(u64, Vec<(u32, Shape)>)>,
    flows: Vec<(u64, Vec<(u32, u32)>)>,
    /// The trailer's (G2-4): per navmesh, its run-time areas, and its
    /// links where they moved from the built ones.
    areas: BTreeMap<u64, Vec<(u32, Shape, u8)>>,
    links: BTreeMap<u64, Vec<(u32, Link)>>,
    /// Per agent, the links its path crosses and the one it is crossing.
    crossing: IdMap<u64, (Vec<PathLink>, Option<Traverse>)>,
    /// Per grid or navmesh whose edit count its paths do not give.
    versions: BTreeMap<u64, u64>,
    /// [`State::origin`] (the trailer's, when not zero).
    origin: Vec3,
}

/// The edit count each grid or navmesh had when its newest path was
/// planned: what a restore sets it back to, and what the trailer need not
/// hold. A nav no path was planned on is left out (nothing saved reads its
/// count).
fn path_versions<'a>(paths: impl Iterator<Item = &'a PathState>) -> BTreeMap<u64, u64> {
    let mut out = BTreeMap::new();
    for p in paths {
        let v = out.entry(p.nav).or_insert(p.version);
        *v = (*v).max(p.version);
    }
    out
}

/// The `og.nav` trailer's tags (`og_ecs::trailer`): what a world using no
/// G2-4 feature never writes, so its block is byte for byte as before.
mod tag {
    /// Per navmesh whose links moved from the built ones: its links.
    pub const LINKS: u16 = 1;
    /// Per navmesh: its obstacles that are not boxes, and its run-time areas.
    pub const SHAPES: u16 = 2;
    /// Per agent whose path crosses a link: those links, and its crossing.
    pub const CROSSING: u16 = 3;
    /// Per grid or navmesh edited after its newest path was planned: its
    /// edit count, which a restore cannot otherwise rebuild.
    pub const VERSIONS: u16 = 4;
    /// The world's nav origin (R12), when og.WorldOrigin has moved it.
    pub const ORIGIN: u16 = 5;
}

fn put_traverse(o: &mut Vec<u8>, t: &Traverse) {
    let mut w = Writer(o);
    w.u32(t.link);
    w.0.push(t.kind as u8);
    w.v3(t.from);
    w.v3(t.to);
    w.f32(t.elapsed);
    w.f32(t.duration);
    w.0.push(u8::from(t.game) | u8::from(t.finished) << 1);
    w.u32(t.corner);
    w.u32(t.land);
}

fn get_traverse(r: &mut Reader<'_>) -> Result<Traverse> {
    let link = r.u32()?;
    let kind = LinkKind::from_index(r.take(1)?[0] as u32);
    let (from, to) = (r.v3()?, r.v3()?);
    let (elapsed, duration) = (r.f32()?, r.f32()?);
    let flags = r.take(1)?[0];
    Ok(Traverse {
        link,
        kind,
        from,
        to,
        elapsed,
        duration,
        game: flags & 1 != 0,
        finished: flags & 2 != 0,
        corner: r.u32()?,
        land: r.u32()?,
    })
}

fn put_shape(o: &mut Vec<u8>, s: &Shape) {
    s.write(o);
}

fn get_shape(r: &mut Reader<'_>) -> Result<Shape> {
    let mut b = &r.b[r.at..];
    let before = b.len();
    let s = Shape::read(&mut b).ok_or_else(|| {
        Error::new(
            ErrorKind::Invalid,
            "the og.nav state block holds a shape it cannot read",
        )
    })?;
    r.at += before - b.len();
    Ok(s)
}

struct Writer<'a>(&'a mut Vec<u8>);
impl Writer<'_> {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn v3(&mut self, v: Vec3) {
        self.f32(v.x);
        self.f32(v.y);
        self.f32(v.z);
    }
}

struct Reader<'a> {
    b: &'a [u8],
    at: usize,
}
impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8]> {
        if self.at + n > self.b.len() {
            return Err(Error::new(
                ErrorKind::Invalid,
                "the og.nav state block is truncated",
            ));
        }
        let s = &self.b[self.at..self.at + n];
        self.at += n;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn v3(&mut self) -> Result<Vec3> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
}

impl StateBlock for NavBlock {
    fn save(&self, out: &mut Vec<u8>) {
        let st = lock(&self.0.state);
        let navs = lock(&self.0.navs);
        let mut w = Writer(out);
        w.u32(st.paths.len() as u32);
        for (k, p) in &st.paths {
            w.u64(*k);
            w.u32(p.corners.len() as u32);
            for c in &p.corners {
                w.v3(*c);
            }
            w.u32(p.next);
            w.v3(p.target);
            w.u64(p.nav);
            w.u64(p.version);
            w.u32(p.age);
            w.u32(p.poly);
            w.u32(p.status);
        }
        let grids: Vec<(&u64, &Grid)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Grid(g) = n {
                    Some((h, g))
                } else {
                    None
                }
            })
            .collect();
        w.u32(grids.len() as u32);
        for (h, g) in grids {
            w.u64(*h);
            w.u32(g.costs().len() as u32);
            w.0.extend_from_slice(g.costs());
        }
        let meshes: Vec<(&u64, &NavMesh)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Mesh(m) = n {
                    Some((h, m.as_ref()))
                } else {
                    None
                }
            })
            .collect();
        w.u32(meshes.len() as u32);
        // Boxes in the block's first form; other shapes, areas and moved
        // links in the trailer.
        let mut shapes = Vec::new();
        let mut shaped = 0u32;
        let mut links = Vec::new();
        let mut linked = 0u32;
        for (h, m) in &meshes {
            if m.links_moved() {
                linked += 1;
                let mut x = Writer(&mut links);
                x.u64(**h);
                let list: Vec<(u32, &Link)> = m.links().collect();
                x.u32(list.len() as u32);
                for (id, l) in list {
                    x.u32(id);
                    l.write(x.0);
                }
            }
            w.u64(**h);
            let boxes: Vec<(u32, Vec3, Vec3)> = m
                .obstacles()
                .filter_map(|(id, o)| match o {
                    Shape::Box { min, max } => Some((id, *min, *max)),
                    _ => None,
                })
                .collect();
            w.u32(boxes.len() as u32);
            for (id, min, max) in boxes {
                w.u32(id);
                w.v3(min);
                w.v3(max);
            }
            let others: Vec<(u32, &Shape)> = m
                .obstacles()
                .filter(|(_, o)| !matches!(o, Shape::Box { .. }))
                .collect();
            let areas: Vec<(u32, &(Shape, u8))> = m.areas().collect();
            if others.is_empty() && areas.is_empty() {
                continue;
            }
            shaped += 1;
            let mut x = Writer(&mut shapes);
            x.u64(**h);
            x.u32(others.len() as u32);
            for (id, o) in others {
                x.u32(id);
                put_shape(x.0, o);
            }
            x.u32(areas.len() as u32);
            for (id, (shape, area)) in areas {
                x.u32(id);
                x.0.push(*area);
                put_shape(x.0, shape);
            }
        }
        let flows: Vec<(&u64, &Vec<(u32, u32)>)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                if let Nav::Flow { targets, .. } = n {
                    Some((h, targets))
                } else {
                    None
                }
            })
            .collect();
        w.u32(flows.len() as u32);
        for (h, t) in flows {
            w.u64(*h);
            w.u32(t.len() as u32);
            for &(x, y) in t {
                w.u32(x);
                w.u32(y);
            }
        }
        let mut sections = Vec::new();
        if linked > 0 {
            let mut b = linked.to_le_bytes().to_vec();
            b.extend_from_slice(&links);
            sections.push((tag::LINKS, b));
        }
        if shaped > 0 {
            let mut b = shaped.to_le_bytes().to_vec();
            b.extend_from_slice(&shapes);
            sections.push((tag::SHAPES, b));
        }
        let crossing: Vec<(&u64, &PathState)> = st
            .paths
            .iter()
            .filter(|(_, p)| !p.links.is_empty() || p.traverse.is_some())
            .collect();
        if !crossing.is_empty() {
            let mut b = Vec::new();
            let mut x = Writer(&mut b);
            x.u32(crossing.len() as u32);
            for (k, p) in crossing {
                x.u64(*k);
                x.u32(p.links.len() as u32);
                for l in &p.links {
                    x.u32(l.corner);
                    x.u32(l.link);
                    x.0.push(u8::from(l.reverse));
                }
                match &p.traverse {
                    Some(t) => {
                        x.0.push(1);
                        put_traverse(x.0, t);
                    }
                    None => x.0.push(0),
                }
            }
            sections.push((tag::CROSSING, b));
        }
        let planned = path_versions(st.paths.values());
        let stale: Vec<(u64, u64)> = navs
            .entries
            .iter()
            .filter_map(|(h, n)| {
                let v = match n {
                    Nav::Grid(g) => g.version(),
                    Nav::Mesh(m) => m.version(),
                    Nav::Flow { .. } => return None,
                };
                planned.get(h).is_some_and(|&p| p != v).then_some((*h, v))
            })
            .collect();
        if !stale.is_empty() {
            let mut b = Vec::new();
            let mut x = Writer(&mut b);
            x.u32(stale.len() as u32);
            for (h, v) in stale {
                x.u64(h);
                x.u64(v);
            }
            sections.push((tag::VERSIONS, b));
        }
        if st.origin != Vec3::ZERO {
            let mut b = Vec::new();
            Writer(&mut b).v3(st.origin);
            sections.push((tag::ORIGIN, b));
        }
        og_ecs::trailer::write(out, &sections);
    }

    fn decode(&self, bytes: &[u8]) -> Result<Box<dyn Any + Send>> {
        let mut r = Reader { b: bytes, at: 0 };
        let mut paths = IdMap::new();
        for _ in 0..r.u32()? {
            let k = r.u64()?;
            let n = r.u32()? as usize;
            let mut corners = Vec::with_capacity(n.min(4096));
            for _ in 0..n {
                corners.push(r.v3()?);
            }
            let p = PathState {
                corners,
                next: r.u32()?,
                target: r.v3()?,
                nav: r.u64()?,
                version: r.u64()?,
                age: r.u32()?,
                poly: r.u32()?,
                status: r.u32()?,
                links: Vec::new(),
                traverse: None,
            };
            paths.insert(k, p);
        }
        let mut grids = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let n = r.u32()? as usize;
            grids.push((h, r.take(n)?.to_vec()));
        }
        let mut obstacles = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let mut list = Vec::new();
            for _ in 0..r.u32()? {
                list.push((
                    r.u32()?,
                    Shape::Box {
                        min: r.v3()?,
                        max: r.v3()?,
                    },
                ));
            }
            obstacles.push((h, list));
        }
        let mut flows = Vec::new();
        for _ in 0..r.u32()? {
            let h = r.u64()?;
            let mut t = Vec::new();
            for _ in 0..r.u32()? {
                t.push((r.u32()?, r.u32()?));
            }
            flows.push((h, t));
        }
        let mut areas = BTreeMap::new();
        let mut links = BTreeMap::new();
        let mut crossing = IdMap::new();
        let mut versions = BTreeMap::new();
        let mut origin = Vec3::ZERO;
        for (t, body) in og_ecs::trailer::read(SYSTEM_NAME, &bytes[r.at..])? {
            let mut r = Reader { b: body, at: 0 };
            if t == tag::ORIGIN {
                origin = r.v3()?;
            }
            if t == tag::LINKS {
                for _ in 0..r.u32()? {
                    let h = r.u64()?;
                    let mut list = Vec::new();
                    for _ in 0..r.u32()? {
                        let id = r.u32()?;
                        let mut b = &r.b[r.at..];
                        let before = b.len();
                        let l = Link::read(&mut b).ok_or_else(|| {
                            Error::new(
                                ErrorKind::Invalid,
                                "the og.nav state block's links are cut short",
                            )
                        })?;
                        r.at += before - b.len();
                        list.push((id, l));
                    }
                    links.insert(h, list);
                }
            }
            if t == tag::CROSSING {
                for _ in 0..r.u32()? {
                    let k = r.u64()?;
                    let mut list = Vec::new();
                    for _ in 0..r.u32()? {
                        list.push(PathLink {
                            corner: r.u32()?,
                            link: r.u32()?,
                            reverse: r.take(1)?[0] != 0,
                        });
                    }
                    let tr = match r.take(1)?[0] {
                        0 => None,
                        _ => Some(get_traverse(&mut r)?),
                    };
                    crossing.insert(k, (list, tr));
                }
            }
            if t == tag::VERSIONS {
                for _ in 0..r.u32()? {
                    versions.insert(r.u64()?, r.u64()?);
                }
            }
            if t == tag::SHAPES {
                for _ in 0..r.u32()? {
                    let h = r.u64()?;
                    let mut shapes = Vec::new();
                    for _ in 0..r.u32()? {
                        shapes.push((r.u32()?, get_shape(&mut r)?));
                    }
                    if let Some((_, list)) = obstacles.iter_mut().find(|(m, _)| *m == h) {
                        list.extend(shapes);
                        og_core::sort::sort_by_key(list, |(id, _)| *id);
                    }
                    let mut list = Vec::new();
                    for _ in 0..r.u32()? {
                        let id = r.u32()?;
                        let area = r.take(1)?[0];
                        list.push((id, get_shape(&mut r)?, area));
                    }
                    areas.insert(h, list);
                }
            }
        }
        Ok(Box::new(Saved {
            paths,
            grids,
            obstacles,
            flows,
            areas,
            links,
            crossing,
            versions,
            origin,
        }))
    }

    fn apply(&mut self, decoded: Option<Box<dyn Any + Send>>) {
        let mut st = lock(&self.0.state);
        let Some(saved) = decoded.and_then(|d| d.downcast::<Saved>().ok()) else {
            st.paths.clear();
            st.origin = Vec3::ZERO;
            return;
        };
        st.origin = saved.origin;
        let mut versions = path_versions(saved.paths.values());
        versions.extend(saved.versions);
        st.paths = saved.paths;
        for (k, (links, traverse)) in saved.crossing {
            if let Some(p) = st.paths.get_mut(&k) {
                p.links = links;
                p.traverse = traverse;
            }
        }
        let mut navs = lock(&self.0.navs);
        let mut saved_links = saved.links;
        for (h, costs) in saved.grids {
            if let Some(Nav::Grid(g)) = navs.get_mut(h)
                && g.costs() != costs.as_slice()
                && costs.len() == g.costs().len()
            {
                for (i, &c) in costs.iter().enumerate() {
                    let (x, y) = (i as u32 % g.width, i as u32 / g.width);
                    g.set(x, y, c);
                }
            }
        }
        for (h, list) in saved.obstacles {
            if let Some(Nav::Mesh(m)) = navs.get_mut(h) {
                let have: Vec<(u32, Shape)> = m.obstacles().map(|(k, o)| (k, o.clone())).collect();
                if have != list {
                    for (k, _) in have {
                        m.carve(k, None);
                    }
                    for (k, o) in list {
                        m.carve(k, Some(o));
                    }
                }
                m.restore_links(saved_links.remove(&h));
                let want = saved.areas.get(&h).cloned().unwrap_or_default();
                let have: Vec<(u32, Shape, u8)> =
                    m.areas().map(|(k, (s, a))| (k, s.clone(), *a)).collect();
                if have != want {
                    for (k, ..) in have {
                        m.set_area(k, None);
                    }
                    for (k, s, a) in want {
                        m.set_area(k, Some((s, a)));
                    }
                }
            }
        }
        // Rebuilding the grids and meshes above counted edits of its own:
        // the saved counts keep every restored path current (or stale) as it was.
        for (h, v) in versions {
            match navs.get_mut(h) {
                Some(Nav::Grid(g)) => g.restore_version(v),
                Some(Nav::Mesh(m)) => m.restore_version(v),
                _ => {}
            }
        }
        for (h, targets) in saved.flows {
            let grid = match navs.get(h) {
                Some(Nav::Flow { grid, .. }) => *grid,
                _ => continue,
            };
            let Some(Nav::Grid(g)) = navs.get(grid).cloned() else {
                continue;
            };
            if let Some(Nav::Flow {
                targets: t, field, ..
            }) = navs.get_mut(h)
                && *t != targets
            {
                *field = FlowField::build(&g, &targets);
                *t = targets;
            }
        }
        navs.refresh_flows();
    }

    /// Every path's corners, target and crossing (agents are roots: their
    /// positions are in the world) and the nav origin the data is read at:
    /// the grids, navmeshes and flow fields stay as they are, in their
    /// frames ([`Navs::offset`]).
    fn shift(&mut self, s: &og_ecs::Shift) {
        let mut st = lock(&self.0.state);
        for p in st.paths.values_mut() {
            for c in &mut p.corners {
                *c = s.moved(*c);
            }
            p.target = s.moved(p.target);
            if let Some(t) = &mut p.traverse {
                t.from = s.moved(t.from);
                t.to = s.moved(t.to);
            }
        }
        st.origin = s.moved(st.origin);
    }
}
