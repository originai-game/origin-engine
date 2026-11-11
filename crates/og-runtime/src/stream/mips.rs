//! Texture mip streaming (G3-3 batch 3, `docs/design.md` §2.5).
//!
//! A texture cooked with `texture { stream true }` (or a model's with
//! `model { stream_textures true }`) is two outputs: `<id>`, its mip tail
//! (the whole chain's levels from the first at or under
//! [`og_bundle::runtime::MIP_TAIL`] down, byte for byte), which every load
//! draws, and `<id>#mips`, the whole chain. Before each frame, after
//! extraction, each such texture's footprint is the most pixels any
//! instance drawing it covers across (its mesh's bounding sphere seen from
//! the camera, as `og_render::lod` measures). One whose footprint passes
//! its tail's longest side is wanted whole, and kept wanted for
//! [`KEEP_FRAMES`] after; the wanted ones are admitted by footprint over
//! tail (then id) within the mip budget ([`DEFAULT_MIP_BUDGET`], separate
//! from the groups'), read through the streaming fetcher after the sound
//! chunks and the packs, transcoded by the host and swapped in under the
//! same handle; one no longer admitted goes back to its tail. The
//! renderer reports the whole chain's size for the texture throughout
//! (`RenderScene::set_texture_size`), so nothing sized from it moves.
//!
//! Presentation only: nothing here is world state, nothing a tick reads
//! depends on it, and a frame shows the same world streamed or not. The
//! probe's `mip_stream:resident` reads and shows every streamed texture
//! whole before each frame, as a build without streaming would (natively
//! a read completes at once, so a capture is the whole texture's);
//! `mip_stream:stream[:<MiB>]` streams again, within that budget.

use super::*;
use og_bundle::runtime::{CookedTexture, MipChain, mips_id};
use og_render::lod;
use std::collections::{HashMap, HashSet};

/// The GPU memory whole mip chains may take until `mip_stream` says
/// otherwise: 128 MiB.
pub const DEFAULT_MIP_BUDGET: u64 = 128 << 20;

/// Frames a texture stays wanted whole after the last frame it was: a
/// camera turning away and back does not read it again.
const KEEP_FRAMES: u64 = 120;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum State {
    /// Drawing its tail.
    Tail,
    /// Admitted; its whole chain waits for a read.
    Wanted,
    /// Its read is in the fetcher.
    Reading,
    /// Read; the host is transcoding it.
    Transcoding,
    /// Drawing its whole chain.
    Whole,
    /// Its whole chain could not be read or decoded: drawn as its tail.
    Failed,
}

impl State {
    fn name(self) -> &'static str {
        match self {
            State::Tail => "tail",
            State::Wanted => "wanted",
            State::Reading => "reading",
            State::Transcoding => "transcoding",
            State::Whole => "whole",
            State::Failed => "failed",
        }
    }
}

struct Entry {
    id: String,
    handle: TextureHandle,
    chain: MipChain,
    /// The tail's level 0.
    tail: (u32, u32),
    /// What the whole chain takes on the GPU in the encoding it reaches
    /// this device in.
    bytes: u64,
    state: State,
    /// The last frame's footprint, in pixels.
    footprint: f32,
    /// The last frame it was wanted whole.
    wanted_at: Option<u64>,
    /// Its footprint over its tail's longest side that frame: its place in
    /// admission while it stays wanted, so a texture that has scrolled off
    /// keeps the rank it was admitted at instead of falling to id order.
    score: f32,
    /// Read, waiting for the host's transcode.
    pending: Option<CookedTexture>,
    reason: String,
}

impl Entry {
    fn tail_side(&self) -> f32 {
        self.tail.0.max(self.tail.1) as f32
    }

    /// Whether it takes (or is about to take) its whole chain's memory.
    fn admitted(&self) -> bool {
        matches!(
            self.state,
            State::Wanted | State::Reading | State::Transcoding | State::Whole
        )
    }
}

/// An engine's streamed textures (in its streams).
#[derive(Default)]
pub(crate) struct Mips {
    /// By id, then handle.
    entries: Vec<Entry>,
    /// The cooked textures already looked at.
    seen: HashSet<TextureHandle>,
    budget: Option<u64>,
    resident: bool,
    frame: u64,
    /// Each mesh's bounding sphere in its own space.
    spheres: HashMap<og_render::MeshHandle, (glam::Vec3, f32)>,
    swaps: u64,
    evictions: u64,
    peak: u64,
}

impl Mips {
    fn budget(&self) -> u64 {
        self.budget.unwrap_or(DEFAULT_MIP_BUDGET)
    }

    fn used(&self) -> u64 {
        self.entries
            .iter()
            .filter(|e| e.admitted())
            .map(|e| e.bytes)
            .sum()
    }

    fn find(&self, id: &str, handle: TextureHandle) -> Option<usize> {
        self.entries
            .iter()
            .position(|e| e.id == id && e.handle == handle)
    }
}

/// The encodings the attached renderer takes (`None`: RGBA8).
fn formats(rs: &extract::RenderState) -> Option<og_render::TextureFormats> {
    rs.backend.as_ref().and_then(|b| b.texture_formats())
}

/// What `tex`'s whole chain takes on the GPU in the encoding it reaches
/// a device taking `formats` in.
fn chain_bytes(
    tex: &CookedTexture,
    chain: MipChain,
    formats: Option<&og_render::TextureFormats>,
) -> u64 {
    let f = tex.target_format(formats);
    (0..chain.levels)
        .map(|l| f.level_bytes(chain.width, chain.height, l) as u64)
        .sum()
}

/// Decodes `tex` through the host's texture hook.
fn transcode(
    tex: &CookedTexture,
    formats: Option<&og_render::TextureFormats>,
) -> super::assets::Fetch<TextureSource> {
    match hooks().map(|h| h.texture(tex, formats)) {
        Some(f) => f,
        None => super::assets::Fetch::Failed("no host is installed to decode textures".into()),
    }
}

/// The streamed textures among the cooked ones the renderer holds that
/// were not looked at yet; drops the ones it no longer holds.
fn register(a: &super::assets::Assets, m: &mut Mips, rs: &mut extract::RenderState) {
    let fresh: Vec<(TextureHandle, String)> = rs
        .decoded
        .ids
        .iter()
        .filter(|(h, _)| !m.seen.contains(*h))
        .map(|(h, id)| (*h, id.clone()))
        .collect();
    let formats = formats(rs);
    let mut added = false;
    for (h, id) in fresh {
        m.seen.insert(h);
        if a.bundle_for(&mips_id(&id)).is_err() {
            continue;
        }
        let Ok(tex) = a
            .blob(&id)
            .and_then(|(b, bundle)| bundle.texture(&id, b.bytes()).map_err(|e| e.to_string()))
        else {
            continue;
        };
        let Some(chain) = tex.mips else {
            continue;
        };
        rs.scene
            .set_texture_size(h, Some((chain.width, chain.height)));
        m.entries.push(Entry {
            id,
            handle: h,
            chain,
            tail: (tex.meta.width, tex.meta.height),
            bytes: chain_bytes(&tex, chain, formats.as_ref()),
            state: State::Tail,
            footprint: 0.0,
            wanted_at: None,
            score: 0.0,
            pending: None,
            reason: String::new(),
        });
        added = true;
    }
    let scene = &rs.scene;
    m.entries.retain(|e| scene.contains_texture(e.handle));
    m.seen.retain(|h| scene.contains_texture(*h));
    if added {
        og_core::sort::sort_by(&mut m.entries, |x, y| {
            x.id.cmp(&y.id).then(x.handle.slot().cmp(&y.handle.slot()))
        });
    }
}

/// Each streamed texture's footprint this frame: the most pixels across
/// any instance drawing it covers, in a view `view_height` pixels tall.
fn measure(m: &mut Mips, scene: &og_render::RenderScene, view_height: f32) {
    for e in &mut m.entries {
        e.footprint = 0.0;
    }
    let index: HashMap<TextureHandle, usize> = m
        .entries
        .iter()
        .enumerate()
        .map(|(i, e)| (e.handle, i))
        .collect();
    let mut by_material: HashMap<og_render::MaterialHandle, Vec<usize>> = HashMap::new();
    for inst in &scene.objects.instances {
        let streamed = by_material.entry(inst.material).or_insert_with(|| {
            scene
                .material_textures(inst.material)
                .into_iter()
                .filter_map(|t| index.get(&t).copied())
                .collect()
        });
        if streamed.is_empty() {
            continue;
        }
        let sphere = match m.spheres.get(&inst.mesh) {
            Some(s) => Some(*s),
            None => scene
                .mesh(inst.mesh)
                .and_then(|d| d.bounds())
                .map(|(lo, hi)| {
                    let s = ((lo + hi) * 0.5, (hi - lo).length() * 0.5);
                    m.spheres.insert(inst.mesh, s);
                    s
                }),
        };
        let Some((center, radius)) = sphere else {
            continue;
        };
        let scale = inst
            .transform
            .to_scale_rotation_translation()
            .0
            .abs()
            .max_element();
        let r = radius * scale;
        let per_unit = lod::pixels_per_unit(
            &scene.frame.camera,
            inst.transform.transform_point3(center),
            r,
            view_height,
        );
        let covered = (2.0 * r * per_unit).min(1.0e9);
        for &i in streamed.iter() {
            let e = &mut m.entries[i];
            e.footprint = e.footprint.max(covered);
        }
    }
}

/// Swaps `e`'s whole chain in.
fn show_whole(m: &mut Mips, i: usize, rs: &mut extract::RenderState, source: TextureSource) {
    let e = &mut m.entries[i];
    super::replace_texture(&mut rs.scene, e.handle, source);
    e.state = State::Whole;
    e.pending = None;
    m.swaps += 1;
}

/// A read of `e`'s whole chain landed: transcoded and shown, or left to
/// the host's transcode.
fn land(
    a: &super::assets::Assets,
    m: &mut Mips,
    i: usize,
    rs: &mut extract::RenderState,
    bytes: Result<Vec<u8>, String>,
) {
    let id = mips_id(&m.entries[i].id);
    let tex = bytes.and_then(|b| {
        let (bundle, _) = a.bundle_for(&id)?;
        bundle
            .texture(&id, super::assets::aligned(&b).bytes())
            .map_err(|e| e.to_string())
    });
    let tex = match tex {
        Ok(t) => t,
        Err(e) => {
            let e2 = &mut m.entries[i];
            e2.state = State::Failed;
            e2.reason = e;
            return;
        }
    };
    let formats = formats(rs);
    match transcode(&tex, formats.as_ref()) {
        super::assets::Fetch::Ready(s) => show_whole(m, i, rs, s),
        super::assets::Fetch::Pending => {
            let e = &mut m.entries[i];
            e.state = State::Transcoding;
            e.pending = Some(tex);
        }
        super::assets::Fetch::Failed(err) => {
            let e = &mut m.entries[i];
            e.state = State::Failed;
            e.reason = err;
        }
    }
}

/// Puts `e`'s tail back. False while the host is still transcoding it.
fn show_tail(
    a: &super::assets::Assets,
    m: &mut Mips,
    i: usize,
    rs: &mut extract::RenderState,
) -> bool {
    let formats = formats(rs);
    let e = &m.entries[i];
    let tex = a
        .blob(&e.id)
        .and_then(|(b, bundle)| bundle.texture(&e.id, b.bytes()).map_err(|x| x.to_string()));
    let source = match tex.map(|t| transcode(&t, formats.as_ref())) {
        Ok(super::assets::Fetch::Ready(s)) => s,
        Ok(super::assets::Fetch::Pending) => return false,
        Ok(super::assets::Fetch::Failed(_)) | Err(_) => placeholder_source(),
    };
    super::replace_texture(&mut rs.scene, e.handle, source);
    let e = &mut m.entries[i];
    e.state = State::Tail;
    e.pending = None;
    m.evictions += 1;
    true
}

fn placeholder_source() -> TextureSource {
    TextureSource::Rgba(super::placeholder())
}

/// The path under the bundle of `id`'s whole chain.
fn path_of(a: &super::assets::Assets, id: &str) -> Option<String> {
    let mips = mips_id(id);
    let (bundle, dir) = a.bundle_for(&mips).ok()?;
    let o = bundle.output(&mips).ok()?;
    Some(format!("{dir}/{}", o.file))
}

/// Before each frame, after extraction: registers the streamed textures,
/// measures their footprints, admits the wanted ones within the budget,
/// puts the unadmitted ones back to their tails and swaps in what the host
/// finished transcoding. In resident mode every streamed texture is read
/// and shown whole now.
pub(crate) fn frame(eng: &Engine, rs: &mut extract::RenderState) {
    let Ok(mut a) = eng.assets.try_borrow_mut() else {
        return;
    };
    let mut m = std::mem::take(&mut a.stream.mips);
    drop(a);
    let a = eng.assets.borrow();
    m.frame += 1;
    register(&a, &mut m, rs);
    if !m.entries.is_empty() {
        decide(&a, &mut m, rs);
    }
    drop(a);
    eng.assets.borrow_mut().stream.mips = m;
}

fn decide(a: &super::assets::Assets, m: &mut Mips, rs: &mut extract::RenderState) {
    // Transcodes the host finished.
    let formats = formats(rs);
    for i in 0..m.entries.len() {
        if m.entries[i].state != State::Transcoding {
            continue;
        }
        let Some(tex) = m.entries[i].pending.take() else {
            m.entries[i].state = State::Tail;
            continue;
        };
        match transcode(&tex, formats.as_ref()) {
            super::assets::Fetch::Ready(s) => show_whole(m, i, rs, s),
            super::assets::Fetch::Pending => m.entries[i].pending = Some(tex),
            super::assets::Fetch::Failed(err) => {
                m.entries[i].state = State::Failed;
                m.entries[i].reason = err;
            }
        }
    }
    let view_height = match rs.backend.as_ref().map(|b| b.size()) {
        Some((_, h)) if h > 0 => h as f32,
        _ => super::fx::viewport(rs).1,
    };
    measure(m, &rs.scene, view_height);
    let admitted = admit(m);
    for (i, &admit) in admitted.iter().enumerate() {
        match (admit, m.entries[i].state) {
            (true, State::Tail) => m.entries[i].state = State::Wanted,
            (false, State::Wanted | State::Reading | State::Transcoding) => {
                m.entries[i].state = State::Tail;
                m.entries[i].pending = None;
            }
            (false, State::Whole) => {
                show_tail(a, m, i, rs);
            }
            _ => {}
        }
    }
    if m.resident {
        // Read now: natively a read completes at once, so this frame draws
        // every streamed texture whole; a host that answers `Pending`
        // leaves the read to the fetcher.
        for i in 0..m.entries.len() {
            if !matches!(m.entries[i].state, State::Wanted | State::Reading) {
                continue;
            }
            let Some(path) = path_of(a, &m.entries[i].id) else {
                continue;
            };
            match hooks().map(|h| h.read_asset(&path)) {
                Some(super::assets::Fetch::Ready(b)) => land(a, m, i, rs, Ok(b)),
                Some(super::assets::Fetch::Failed(e)) => land(a, m, i, rs, Err(e)),
                Some(super::assets::Fetch::Pending) => m.entries[i].state = State::Reading,
                None => {}
            }
        }
    }
    m.peak = m.peak.max(m.used());
}

/// Which streamed textures hold their whole chain this frame: the ones
/// whose footprint passed their tail's longest side within the last
/// [`KEEP_FRAMES`] (every one in resident mode), by footprint over tail
/// (then id), as many as the budget holds (all in resident mode). A
/// texture whose whole chain failed stays on its tail.
fn admit(m: &mut Mips) -> Vec<bool> {
    let frame = m.frame;
    let resident = m.resident;
    for e in &mut m.entries {
        if resident || e.footprint > e.tail_side() {
            e.wanted_at = Some(frame);
            e.score = e.footprint / e.tail_side();
        }
    }
    let mut order: Vec<usize> = (0..m.entries.len())
        .filter(|&i| {
            let e = &m.entries[i];
            e.state != State::Failed
                && e.wanted_at
                    .is_some_and(|w| resident || frame - w < KEEP_FRAMES)
        })
        .collect();
    og_core::sort::sort_by(&mut order, |&x, &y| {
        let (ex, ey) = (&m.entries[x], &m.entries[y]);
        ey.score.total_cmp(&ex.score).then(ex.id.cmp(&ey.id))
    });
    let budget = m.budget();
    let mut used = 0u64;
    let mut admitted = vec![false; m.entries.len()];
    for i in order {
        let bytes = m.entries[i].bytes;
        if resident || used + bytes <= budget {
            used += bytes;
            admitted[i] = true;
        }
    }
    admitted
}

/// The reads the fetcher should make for whole chains, most wanted first.
pub(super) fn wanted(a: &super::assets::Assets) -> Vec<Wanted> {
    let m = &a.stream.mips;
    let mut out: Vec<(f32, Wanted)> = m
        .entries
        .iter()
        .filter(|e| matches!(e.state, State::Wanted | State::Reading))
        .filter_map(|e| {
            Some((
                e.footprint / e.tail_side(),
                Wanted {
                    path: path_of(a, &e.id)?,
                    need: Need::Mips {
                        id: e.id.clone(),
                        handle: e.handle.slot(),
                    },
                },
            ))
        })
        .collect();
    og_core::sort::sort_by(&mut out, |x, y| y.0.total_cmp(&x.0));
    out.into_iter().map(|(_, w)| w).collect()
}

/// The fetcher's read of `id`'s whole chain is in flight.
pub(super) fn reading(eng: &Engine, id: &str, slot: u32) {
    let mut a = eng.assets.borrow_mut();
    let m = &mut a.stream.mips;
    if let Some(e) = m
        .entries
        .iter_mut()
        .find(|e| e.id == id && e.handle.slot() == slot && e.state == State::Wanted)
    {
        e.state = State::Reading;
    }
}

/// The fetcher's read of `id`'s whole chain landed.
pub(super) fn deliver(eng: &Engine, id: &str, slot: u32, bytes: Result<Vec<u8>, String>) {
    let Ok(mut rs) = eng.render.try_borrow_mut() else {
        return;
    };
    let mut m = std::mem::take(&mut eng.assets.borrow_mut().stream.mips);
    {
        let a = eng.assets.borrow();
        let handle = TextureHandle::from_slot(slot);
        let i = handle.and_then(|h| m.find(id, h));
        // Let go of while it was read: dropped.
        if let Some(i) = i.filter(|&i| matches!(m.entries[i].state, State::Wanted | State::Reading))
        {
            land(&a, &mut m, i, &mut rs, bytes);
        }
    }
    eng.assets.borrow_mut().stream.mips = m;
}

/// A texture the renderer replaced with its tail (a lost device decoded it
/// again): it draws its tail now.
pub(crate) fn reset(eng: &Engine, handle: TextureHandle) {
    let mut a = eng.assets.borrow_mut();
    for e in &mut a.stream.mips.entries {
        if e.handle == handle && e.state != State::Failed {
            e.state = State::Tail;
            e.pending = None;
        }
    }
}

/// The probe's `mip_stream:<stream|resident>[:<MiB>]`.
pub(crate) fn set_mode(engine_id: u32, args: &[String]) -> Result<(), String> {
    const USAGE: &str = "write mip_stream:<stream|resident>[:<MiB>], such as mip_stream:resident or mip_stream:stream:64";
    let resident = match args.first().map(String::as_str) {
        Some("resident") => true,
        Some("stream") => false,
        _ => return Err(USAGE.into()),
    };
    let budget = match args.get(1).filter(|a| !a.is_empty()) {
        None => None,
        Some(x) => match x.parse::<f64>() {
            Ok(mib) if mib.is_finite() && mib >= 0.0 => Some((mib * 1048576.0) as u64),
            _ => return Err(format!("mip_stream: '{x}' is not a number of MiB; {USAGE}")),
        },
    };
    let eng = engine(engine_id)?;
    let mut a = eng
        .assets
        .try_borrow_mut()
        .map_err(|_| "mip_stream: the assets are busy".to_string())?;
    let m = &mut a.stream.mips;
    m.resident = resident;
    if budget.is_some() {
        m.budget = budget;
    }
    Ok(())
}

/// `dump_stream`'s `mips`: the mode, the budget, what is used and its
/// peak, and every streamed texture.
pub(super) fn dump(m: &Mips) -> Value {
    json!({
        "mode": if m.resident { "resident" } else { "stream" },
        "budget": m.budget() as f64,
        "used": m.used() as f64,
        "peak": m.peak as f64,
        "swaps": m.swaps as f64,
        "evictions": m.evictions as f64,
        "textures": m.entries.iter().map(|e| json!({
            "id": e.id,
            "state": e.state.name(),
            "footprint": e.footprint,
            "tail": [e.tail.0, e.tail.1],
            "whole": [e.chain.width, e.chain.height],
            "levels": e.chain.levels,
            "bytes": e.bytes as f64,
            "reason": e.reason,
        })).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use glam::{Mat4, Vec3};
    use og_render::{Material, RenderScene, TextureData};

    fn entry(id: &str, handle: TextureHandle, tail: u32, footprint: f32, bytes: u64) -> Entry {
        Entry {
            id: id.into(),
            handle,
            chain: MipChain {
                width: tail * 4,
                height: tail * 4,
                levels: 12,
            },
            tail: (tail, tail),
            bytes,
            state: State::Tail,
            footprint,
            wanted_at: None,
            score: 0.0,
            pending: None,
            reason: String::new(),
        }
    }

    fn texture(scene: &mut RenderScene) -> TextureHandle {
        scene.add_texture(TextureData {
            width: 1,
            height: 1,
            rgba: vec![255; 4],
            ..Default::default()
        })
    }

    /// A 2 m plane (bounding sphere radius √2) seen by the default 50°
    /// camera at (0, 0, 10): the footprint is the larger of its two
    /// instances', worked out here from the camera's own numbers, and a
    /// texture no instance draws is not seen.
    #[test]
    fn a_footprint_is_the_most_any_instance_covers() {
        let mut scene = RenderScene::new();
        scene.frame.camera.position = Vec3::new(0.0, 0.0, 10.0);
        scene.frame.camera.target = Vec3::ZERO;
        let mesh = scene.add_mesh(og_render::shapes::plane(2.0, 1.0));
        let (streamed, other, unseen) = (
            texture(&mut scene),
            texture(&mut scene),
            texture(&mut scene),
        );
        let material = scene.add_material(Material {
            normal_texture: Some(streamed),
            base_color_texture: Some(other),
            ..Default::default()
        });
        // Near and small, and 100 m out and four times the size.
        scene.add_instance(mesh, material, Mat4::IDENTITY);
        scene.add_instance(
            mesh,
            material,
            Mat4::from_scale_rotation_translation(
                Vec3::splat(4.0),
                glam::Quat::IDENTITY,
                Vec3::new(0.0, 0.0, -90.0),
            ),
        );
        let mut m = Mips {
            entries: vec![
                entry("a", streamed, 256, 0.0, 1),
                entry("b", unseen, 256, 7.0, 1),
            ],
            ..Default::default()
        };
        measure(&mut m, &scene, 720.0);
        let (sin, cos) = og_core::math::sin_cos(50.0f32.to_radians() * 0.5);
        let tan = sin / cos;
        let covered = |r: f32, d: f32| 2.0 * r * 720.0 / (2.0 * (d - r) * tan);
        let r = 2.0f32.sqrt();
        let want = covered(r, 10.0).max(covered(4.0 * r, 100.0));
        assert!(
            (m.entries[0].footprint - want).abs() < want * 1e-4,
            "{} vs {want}",
            m.entries[0].footprint
        );
        assert!(covered(r, 10.0) > covered(4.0 * r, 100.0));
        assert_eq!(m.entries[1].footprint, 0.0, "drawn by no instance");
    }

    /// Admission goes by footprint over tail, not by footprint: `b`'s
    /// smaller tail is the more magnified, so it takes the one place the
    /// budget has. A texture smaller on screen than its tail is not
    /// wanted; one that was stays whole for KEEP_FRAMES after; resident
    /// mode takes every one, past the budget; a failed one never.
    #[test]
    fn admission_takes_the_most_magnified_within_the_budget() {
        let h = |i: u32| TextureHandle::from_slot(i + 1).unwrap();
        let mut m = Mips {
            entries: vec![
                entry("a", h(0), 256, 1000.0, 40),
                entry("b", h(1), 128, 600.0, 40),
                entry("c", h(2), 256, 200.0, 40),
                entry("d", h(3), 256, 5000.0, 400),
            ],
            budget: Some(50),
            frame: 1,
            ..Default::default()
        };
        assert_eq!(admit(&mut m), [false, true, false, false]);
        // Nothing seen for a while: still whole, at the rank it was
        // admitted at, until KEEP_FRAMES pass.
        for e in &mut m.entries {
            e.footprint = 0.0;
        }
        m.frame += KEEP_FRAMES - 1;
        assert_eq!(admit(&mut m), [false, true, false, false]);
        m.frame += 1;
        assert_eq!(admit(&mut m), [false, false, false, false]);
        m.resident = true;
        m.entries[3].state = State::Failed;
        assert_eq!(admit(&mut m), [true, true, true, false]);
    }
}
