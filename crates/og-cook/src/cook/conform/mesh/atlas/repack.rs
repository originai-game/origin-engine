//! Repack authored UV charts once, before the model and animation jobs split.
//! All material maps share the same relocation. Split vertices copy every
//! attribute and morph delta from their source; chart orientation stays fixed.
use super::*;
use crate::cook::conform::mesh::{Conform, texel_span};
use og_render_data::UvTransform;
use serde_json::json;

fn key(v: Vec2) -> [u32; 2] {
    v.to_array().map(|x| (x + 0.0).to_bits())
}

struct Packed {
    source: Atlas,
    atlas: Atlas,
    size: u32,
    padding: Padding,
}

impl Packed {
    fn new(source: Atlas, target: u32, world_areas: &[f32], texels_per_metre: f32) -> Option<Self> {
        let mut lo = vec![Vec2::splat(f32::MAX); source.widths.len()];
        let mut area = vec![0.0; source.widths.len()];
        let mut largest = vec![0.0f32; source.widths.len()];
        let mut world = vec![0.0; source.widths.len()];
        for (i, &(t, c)) in source.triangles.iter().enumerate() {
            for v in t {
                lo[c] = lo[c].min(v);
            }
            let triangle_area = (t[1] - t[0]).perp_dot(t[2] - t[0]).abs() * 0.5;
            area[c] += triangle_area;
            largest[c] = largest[c].max(triangle_area);
            world[c] += world_areas[i];
        }
        // Tiny charts get their own four-texel floor, not a global upscale.
        // Reduce only the larger charts if padding makes the first pack overflow.
        for attempt in 0..12 {
            let density = target as f32 * 0.85f32.powi(attempt);
            let scales: Vec<_> = source
                .widths
                .iter()
                .enumerate()
                .map(|(c, width)| {
                    let desired = if texels_per_metre > 0.0 {
                        texels_per_metre * (world[c] / area[c]).sqrt()
                    } else {
                        density
                    };
                    desired.max(4.0 / width.min_element())
                })
                .collect();
            let retry = texels_per_metre <= 0.0
                && scales
                    .iter()
                    .zip(&source.widths)
                    .any(|(&s, width)| s > 4.0 / width.min_element());
            // One triangle per chart is a lower bound even when faces in a
            // chart overlap. Different charts must not overlap. Reject an
            // impossible page before xatlas builds an unbounded atlas for it;
            // thousands of tiny islands can exceed even the source's size.
            let minimum_area: f64 = largest
                .iter()
                .zip(&scales)
                .map(|(&a, &s)| f64::from(a) * f64::from(s).powi(2))
                .sum();
            // xatlas rounds each span up and aligns it to texel centres
            // (+1); the output below adds two texels on each page edge (+4).
            if minimum_area > f64::from(target).powi(2)
                || source
                    .widths
                    .iter()
                    .zip(&scales)
                    .any(|(width, &s)| (width * s).max_element().ceil() + 5.0 > target as f32)
            {
                if !retry {
                    return None;
                }
                continue;
            }
            let mut vertices = BTreeMap::new();
            let mut input_charts = Vec::new();
            let mut uv = Vec::new();
            let mut indices = Vec::new();
            for &(t, c) in &source.triangles {
                let scale = scales[c];
                for v in t {
                    let next = vertices.len() as u32;
                    let i = *vertices.entry((c, key(v))).or_insert_with(|| {
                        input_charts.push(c);
                        uv.push(((v - lo[c]) * scale).to_array());
                        next
                    });
                    indices.push(i);
                }
            }
            let charts: Vec<_> = source.triangles.iter().map(|(_, c)| *c as u32).collect();
            let packed = og_bake::atlas::repack(&uv, &indices, &charts, 2).ok()?;
            if packed.width + 4 > target || packed.height + 4 > target || packed.pages > 1 {
                if !retry {
                    return None;
                }
                continue;
            }
            let mesh = &packed.meshes[0];
            // xatlas omits zero-area UV faces, though their geometry may be
            // valid. Recover their coordinates from their authored chart's
            // fixed-orientation affine placement, never the unplaced UVs.
            let mut bounds = vec![
                (
                    Vec2::splat(f32::MAX),
                    Vec2::splat(f32::MIN),
                    Vec2::ZERO,
                    Vec2::ZERO
                );
                source.widths.len()
            ];
            for v in mesh.vertices.iter().filter(|v| v.page != u32::MAX) {
                let input = Vec2::from(uv[v.xref as usize]);
                let output = Vec2::from(v.uv);
                let b = &mut bounds[input_charts[v.xref as usize]];
                for axis in 0..2 {
                    if input[axis] < b.0[axis] {
                        b.0[axis] = input[axis];
                        b.2[axis] = output[axis];
                    }
                    if input[axis] > b.1[axis] {
                        b.1[axis] = input[axis];
                        b.3[axis] = output[axis];
                    }
                }
            }
            let placements: Vec<_> = bounds
                .into_iter()
                .map(|(lo, hi, to_lo, to_hi)| {
                    let scale = (to_hi - to_lo) / (hi - lo);
                    (scale, to_lo - lo * scale)
                })
                .collect();
            if placements
                .iter()
                .any(|(s, o)| !s.is_finite() || !o.is_finite())
            {
                return None;
            }
            for v in mesh.vertices.iter().filter(|v| v.page != u32::MAX) {
                let (scale, offset) = placements[input_charts[v.xref as usize]];
                if (Vec2::from(uv[v.xref as usize]) * scale + offset - Vec2::from(v.uv))
                    .abs()
                    .max_element()
                    > 0.001
                {
                    return None;
                }
            }
            let output_uv: Vec<_> = mesh
                .vertices
                .iter()
                .map(|v| {
                    let (scale, offset) = placements[input_charts[v.xref as usize]];
                    let value = if v.page == u32::MAX {
                        Vec2::from(uv[v.xref as usize]) * scale + offset
                    } else {
                        Vec2::from(v.uv)
                    };
                    (value + Vec2::splat(2.0)) / target as f32
                })
                .collect();
            debug_assert_eq!(mesh.indices.len(), indices.len());
            debug_assert!(
                mesh.indices
                    .iter()
                    .zip(&indices)
                    .all(|(&out, &input)| mesh.vertices[out as usize].xref == input)
            );
            let triangles = mesh
                .indices
                .as_chunks::<3>()
                .0
                .iter()
                .zip(&source.triangles)
                .map(|(t, &(_, c))| (t.map(|i| output_uv[i as usize]), c))
                .collect();
            let atlas = Atlas {
                triangles,
                widths: source.widths.clone(),
            };
            let Some(padding) = dilation(&atlas, target) else {
                if !retry {
                    return None;
                }
                continue;
            };
            return Some(Self {
                source,
                atlas,
                size: target,
                padding,
            });
        }
        None
    }

    fn image(&self, image: &Image, kind: u32) -> Option<Image> {
        let (w, h) = (image.width as usize, image.height as usize);
        let mut owner = vec![usize::MAX; w * h];
        let mut chart_pixels = vec![Vec::new(); self.source.widths.len()];
        let mut degenerate = vec![false; self.source.widths.len()];
        for &(t, c) in &self.source.triangles {
            if (t[1] - t[0]).perp_dot(t[2] - t[0]).abs() <= 1e-12 {
                degenerate[c] = true;
                continue;
            }
            raster(t, image.width, image.height, 0.5, |x, y, _| {
                let at = y * w + x;
                owner[at] = c;
                chart_pixels[c].push(at);
            });
        }
        for pixels in &mut chart_pixels {
            pixels.sort_unstable();
            pixels.dedup();
        }
        let sample = |uv: Vec2, chart: usize| -> Option<[f32; 4]> {
            let p = uv * Vec2::new(w as f32, h as f32) - Vec2::splat(0.5);
            let (x, y) = (p.x.round() as i32, p.y.round() as i32);
            if degenerate[chart] {
                let at = y.clamp(0, h as i32 - 1) as usize * w + x.clamp(0, w as i32 - 1) as usize;
                return Some(decode(
                    image.rgba[at * 4..at * 4 + 4].try_into().unwrap(),
                    kind,
                ));
            }
            // Never read an adjacent chart or the unused source background,
            // even along a subpixel edge of the original chart.
            let mut best = None;
            let mut distance = f32::MAX;
            for dy in -2..=2 {
                for dx in -2..=2 {
                    let (sx, sy) = (x + dx, y + dy);
                    if sx < 0 || sy < 0 || sx >= w as i32 || sy >= h as i32 {
                        continue;
                    }
                    let at = sy as usize * w + sx as usize;
                    let d = (Vec2::new(sx as f32, sy as f32) - p).length_squared();
                    if owner[at] == chart && d < distance {
                        distance = d;
                        best = Some(image.rgba[at * 4..at * 4 + 4].try_into().unwrap());
                    }
                }
            }
            // An acute chart tip can be many pixels from its first covered
            // texel. Search that chart, not an arbitrary-radius neighbourhood
            // that would reject it or sample another island's colours.
            if best.is_none() {
                for &at in &chart_pixels[chart] {
                    let q = Vec2::new((at % w) as f32, (at / w) as f32);
                    let d = (q - p).length_squared();
                    if d < distance {
                        distance = d;
                        best = Some(image.rgba[at * 4..at * 4 + 4].try_into().unwrap());
                    }
                }
            }
            best.map(|p| decode(p, kind))
        };
        let mut out = Image {
            width: self.size,
            height: self.size,
            rgba: vec![0; (self.size * self.size * 4) as usize],
        };
        let mut valid = true;
        for (&(to, chart), &(from, _)) in self.atlas.triangles.iter().zip(&self.source.triangles) {
            let footprint = glam::Mat2::from_cols(from[1] - from[0], from[2] - from[0])
                * glam::Mat2::from_cols(to[1] - to[0], to[2] - to[0]).inverse()
                / self.size as f32;
            raster(to, self.size, self.size, 0.0, |x, y, bary| {
                let src = from[0] * bary[0] + from[1] * bary[1] + from[2] * bary[2];
                let mut sum = [0.0; 4];
                let mut samples = 0;
                // Stratified area filtering, in premultiplied linear light;
                // normal vectors are averaged then normalized by encode.
                for sy in 0..4 {
                    for sx in 0..4 {
                        let offset =
                            Vec2::new((sx as f32 + 0.5) / 4.0 - 0.5, (sy as f32 + 0.5) / 4.0 - 0.5);
                        if let Some(p) = sample(src + footprint * offset, chart) {
                            for i in 0..4 {
                                sum[i] += p[i];
                            }
                            samples += 1;
                        }
                    }
                }
                if samples == 0 {
                    valid = false;
                } else {
                    sum = sum.map(|v| v / samples as f32);
                }
                let at = (y * self.size as usize + x) * 4;
                out.rgba[at..at + 4].copy_from_slice(&encode(sum, kind));
            });
        }
        if !valid {
            return None;
        }
        for &(to, from) in &self.padding.copies {
            let pixel: [u8; 4] = out.rgba[from * 4..from * 4 + 4].try_into().unwrap();
            out.rgba[to * 4..to * 4 + 4].copy_from_slice(&pixel);
        }
        Some(out)
    }
}

fn dilation(atlas: &Atlas, size: u32) -> Option<Padding> {
    let width = size as usize;
    let mut owner = vec![usize::MAX; width * width];
    let mut valid = true;
    for &(t, chart) in &atlas.triangles {
        raster(t, size, size, 0.0, |x, y, _| {
            let old = &mut owner[y * width + x];
            valid &= *old == usize::MAX || *old == chart;
            *old = chart;
        });
    }
    let mut seen = vec![false; atlas.widths.len()];
    let mut source = vec![usize::MAX; owner.len()];
    let mut queue = VecDeque::new();
    for (at, &chart) in owner.iter().enumerate() {
        if chart != usize::MAX {
            seen[chart] = true;
            source[at] = at;
            queue.push_back(at);
        }
    }
    if !valid || seen.contains(&false) {
        return None;
    }
    // Fill only unused texels. Adjacent charts retain their own interiors;
    // their nearest-border colours meet in the free space without changing UVs.
    while let Some(at) = queue.pop_front() {
        let (x, y) = (at % width, at / width);
        for dy in -1..=1 {
            for dx in -1..=1 {
                let (nx, ny) = (x as isize + dx, y as isize + dy);
                if nx < 0 || ny < 0 || nx >= width as isize || ny >= width as isize {
                    continue;
                }
                let next = ny as usize * width + nx as usize;
                if source[next] == usize::MAX {
                    source[next] = source[at];
                    queue.push_back(next);
                }
            }
        }
    }
    Some(Padding {
        copies: source
            .into_iter()
            .enumerate()
            .filter(|(to, from)| to != from)
            .collect(),
    })
}

fn raster(
    t: [Vec2; 3],
    w: u32,
    h: u32,
    half_width: f32,
    mut pixel: impl FnMut(usize, usize, [f32; 3]),
) {
    let t = t.map(|v| v * Vec2::new(w as f32, h as f32));
    let lo = t[0].min(t[1]).min(t[2]).floor().max(Vec2::ZERO);
    let hi = t[0]
        .max(t[1])
        .max(t[2])
        .ceil()
        .min(Vec2::new(w as f32, h as f32));
    let det = (t[1] - t[0]).perp_dot(t[2] - t[0]);
    // Triangle/texel-box separating axes. Source pixels intersecting a
    // subpixel chart remain samples even when their centres lie outside it.
    let margin = [t[2] - t[1], t[2] - t[0], t[1] - t[0]]
        .map(|edge| half_width * edge.abs().element_sum() / det.abs());
    for y in lo.y as usize..hi.y as usize {
        for x in lo.x as usize..hi.x as usize {
            let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
            let b = (p - t[0]).perp_dot(t[2] - t[0]) / det;
            let c = (t[1] - t[0]).perp_dot(p - t[0]) / det;
            let a = 1.0 - b - c;
            if a >= -margin[0] - 1e-5 && b >= -margin[1] - 1e-5 && c >= -margin[2] - 1e-5 {
                pixel(x, y, [a, b, c]);
            }
        }
    }
}

fn decode(p: [u8; 4], kind: u32) -> [f32; 4] {
    let mut v = p.map(|x| x as f32 / 255.0);
    if matches!(
        kind,
        crate::format::kind::COLOR | crate::format::kind::PIXEL
    ) {
        let a = v[3];
        for x in &mut v[..3] {
            *x = if *x <= 0.04045 {
                *x / 12.92
            } else {
                ((*x + 0.055) / 1.055).powf(2.4)
            };
            *x *= a;
        }
    } else if kind == crate::format::kind::NORMAL {
        for x in &mut v[..3] {
            *x = *x * 2.0 - 1.0;
        }
    }
    v
}

fn encode(mut v: [f32; 4], kind: u32) -> [u8; 4] {
    if matches!(
        kind,
        crate::format::kind::COLOR | crate::format::kind::PIXEL
    ) {
        let a = v[3];
        for x in &mut v[..3] {
            *x = if a > 0.0 { *x / a } else { 0.0 };
            *x = if *x <= 0.0031308 {
                *x * 12.92
            } else {
                1.055 * x.powf(1.0 / 2.4) - 0.055
            };
        }
    } else if kind == crate::format::kind::NORMAL {
        let n = glam::Vec3::new(v[0], v[1], v[2]).normalize_or(glam::Vec3::Z);
        v[..3].copy_from_slice(&n.to_array().map(|x| x * 0.5 + 0.5));
    }
    v.map(|x| (x * 255.0).round().clamp(0.0, 255.0) as u8)
}

/// Relocate one shared authored UV domain and all its maps together. Mixed
/// UV domains and repeating UVs keep the safe native atlas.
pub(in crate::cook) fn repack(c: &Conform, src: &mut GltfSource) -> Result<(), String> {
    if (c.density <= 0.0 && c.max_size == 0) || src.textures.is_empty() {
        return Ok(());
    }
    let uv_plan = crate::cook::model::ext::uv_plan(src);
    let mut placed_areas = BTreeMap::new();
    for (prim, world) in scene_primitives(src) {
        let reader = prim.reader(|b| Some(&src.buffers[b.index()].0));
        let Some(positions) = reader.read_positions() else {
            continue;
        };
        let positions: Vec<_> = positions
            .map(|v| world.transform_point3(glam::Vec3::from(v)))
            .collect();
        let indices: Vec<_> = reader.read_indices().map_or_else(
            || (0..positions.len() as u32).collect(),
            |i| i.into_u32().collect(),
        );
        let position = prim.get(&gltf::Semantic::Positions).unwrap().index();
        for (i, tri) in indices.as_chunks::<3>().0.iter().enumerate() {
            let [a, b, c] = tri.map(|v| positions[v as usize]);
            let area = (b - a).cross(c - a).length() * 0.5;
            let key = (position, prim.indices().map(|v| v.index()), i);
            let old = placed_areas.entry(key).or_insert(0.0f32);
            *old = old.max(area);
        }
    }
    let mut transform: Option<UvTransform> = None;
    let mut originals = Vec::new();
    let mut triangles = Vec::new();
    let mut areas = Vec::new();
    for mesh in src.doc.meshes() {
        for prim in mesh.primitives() {
            if prim
                .material()
                .index()
                .is_some_and(|m| uv_plan.materials[m].iter().any(|t| !t.is_identity()))
            {
                return Ok(());
            }
            let t = uv_plan
                .bake
                .get(&(mesh.index(), prim.index()))
                .copied()
                .unwrap_or(UvTransform::IDENTITY);
            for use_ in &src.textures {
                if uv_set(src, &prim.material(), use_.image)
                    .is_some_and(|(set, transform)| set != 0 || transform != t)
                {
                    return Ok(());
                }
            }
            if transform.is_some_and(|old| old != t) {
                return Ok(());
            }
            transform = Some(t);
            let reader = prim.reader(|b| Some(&src.buffers[b.index()].0));
            let Some(uv) = reader.read_tex_coords(0) else {
                return Ok(());
            };
            let uv: Vec<_> = uv.into_f32().map(|v| Vec2::from(t.apply(v))).collect();
            if uv
                .iter()
                .any(|v| !v.is_finite() || v.min_element() < 0.0 || v.max_element() > 1.0)
            {
                return Ok(());
            }
            let idx: Vec<_> = reader.read_indices().map_or_else(
                || (0..uv.len() as u32).collect(),
                |i| i.into_u32().collect(),
            );
            let position = prim.get(&gltf::Semantic::Positions).unwrap().index();
            for (i, tri) in idx.as_chunks::<3>().0.iter().enumerate() {
                let tri = tri.map(|i| uv[i as usize]);
                triangles.push(tri);
                let Some(&area) =
                    placed_areas.get(&(position, prim.indices().map(|v| v.index()), i))
                else {
                    return Ok(());
                };
                areas.push(area);
            }
            originals.push((mesh.index(), prim.index(), uv, idx));
        }
    }
    // A valid geometric face can have point or line UVs. Give it its own
    // nondegenerate destination chart, while retaining its original UVs
    // for sampling. It must not keep old coordinates in the new atlas.
    let layout = triangles
        .iter()
        .enumerate()
        .map(|(i, &t)| {
            if (t[1] - t[0]).perp_dot(t[2] - t[0]).abs() > 1e-12 {
                t
            } else {
                let origin = Vec2::splat(2.0 + i as f32 * 2.0);
                [origin, origin + Vec2::X, origin + Vec2::Y]
            }
        })
        .collect();
    let Some(atlas) = Atlas::charts(layout) else {
        return Ok(());
    };
    let mut images = Vec::new();
    let mut target = 0;
    let mut longest = 0;
    let mut native_floor = u32::MAX;
    let mut needs = false;
    let mut kinds = BTreeMap::new();
    for use_ in &src.textures {
        if let Some(previous) = kinds.insert(use_.image, use_.kind)
            && previous != use_.kind
        {
            // One image sampled both as normals and colour needs distinct
            // filtered images, not one encoded replacement for both slots.
            return Ok(());
        }
        let image = crate::cook::texture::decode(&src.images[use_.image])?;
        let side = image.width.max(image.height);
        let mut wanted = side;
        if c.density > 0.0
            && let Some(span) = texel_span(src, use_.image)
        {
            let span = span * side as f32 / ((image.width as f32) * (image.height as f32)).sqrt();
            wanted = wanted.min(2f32.powf((c.density * span).max(4.0).log2().round()) as u32);
        }
        if c.max_size > 0 {
            wanted = wanted.min(c.max_size);
        }
        let safe = atlas.fit(wanted, image.width, image.height).0;
        native_floor = native_floor.min(safe);
        needs |= safe > wanted;
        target = target.max(wanted);
        longest = longest.max(side);
        images.push((use_.image, use_.kind, image));
    }
    if !needs || target >= longest {
        return Ok(());
    }
    let mut packed = None;
    while target < longest.min(native_floor) {
        packed = Packed::new(atlas.clone(), target, &areas, c.density);
        if packed.is_some() {
            break;
        }
        target = target.saturating_mul(2).min(longest);
    }
    let Some(mut packed) = packed else {
        return Ok(());
    };
    for ((uv, _), original) in packed.source.triangles.iter_mut().zip(triangles) {
        *uv = original;
    }
    let mut encoded = BTreeMap::new();
    for (i, kind, image) in images {
        let Some(image) = packed.image(&image, kind) else {
            return Ok(());
        };
        let mut png = std::io::Cursor::new(Vec::new());
        image::RgbaImage::from_raw(image.width, image.height, image.rgba)
            .expect("sized RGBA")
            .write_to(&mut png, image::ImageFormat::Png)
            .map_err(|e| e.to_string())?;
        encoded.insert(i, png.into_inner());
    }
    let mut json = src.json.clone();
    let buffer = src.buffers.len();
    let mut bytes = Vec::new();
    let mut triangle = 0;
    for (m, p, uv, indices) in originals {
        let mut xref: Vec<u32> = (0..uv.len() as u32).collect();
        let mut output_uv = uv.clone();
        let mut assigned = vec![false; uv.len()];
        let mut split = BTreeMap::new();
        let mut output_indices = Vec::new();
        for face in indices.as_chunks::<3>().0 {
            for (corner, &old) in face.iter().enumerate() {
                let new_uv = packed.atlas.triangles[triangle].0[corner];
                let i = *split.entry((old, key(new_uv))).or_insert_with(|| {
                    if !assigned[old as usize] {
                        assigned[old as usize] = true;
                        output_uv[old as usize] = new_uv;
                        old
                    } else {
                        let i = xref.len() as u32;
                        xref.push(old);
                        output_uv.push(new_uv);
                        i
                    }
                });
                output_indices.push(i);
            }
            triangle += 1;
        }
        let original = &src.json["meshes"][m]["primitives"][p];
        for (name, id) in original["attributes"].as_object().unwrap() {
            if name == "TEXCOORD_0" {
                continue;
            }
            let accessor = copy_accessor(
                src,
                id.as_u64().unwrap() as usize,
                &xref,
                &mut json,
                &mut bytes,
            );
            json["meshes"][m]["primitives"][p]["attributes"][name] = json!(accessor);
        }
        for (target, values) in original["targets"]
            .as_array()
            .into_iter()
            .flatten()
            .enumerate()
        {
            for (name, id) in values.as_object().unwrap() {
                let accessor = copy_accessor(
                    src,
                    id.as_u64().unwrap() as usize,
                    &xref,
                    &mut json,
                    &mut bytes,
                );
                json["meshes"][m]["primitives"][p]["targets"][target][name] = json!(accessor);
            }
        }
        let output: Vec<u8> = output_uv
            .into_iter()
            .flat_map(|v| v.to_array().into_iter().flat_map(f32::to_le_bytes))
            .collect();
        let accessor = append_accessor(
            &mut json,
            &mut bytes,
            buffer,
            json!({"componentType":5126,"count":xref.len(),"type":"VEC2"}),
            &output,
            None,
        );
        json["meshes"][m]["primitives"][p]["attributes"]["TEXCOORD_0"] = json!(accessor);
        let output: Vec<u8> = output_indices
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect();
        let accessor = append_accessor(
            &mut json,
            &mut bytes,
            buffer,
            json!({"componentType":5125,"count":output_indices.len(),"type":"SCALAR"}),
            &output,
            None,
        );
        json["meshes"][m]["primitives"][p]["indices"] = json!(accessor);
    }
    for material in json["materials"].as_array_mut().into_iter().flatten() {
        clear_transforms(material);
    }
    json["buffers"]
        .as_array_mut()
        .unwrap()
        .push(json!({"byteLength": bytes.len()}));
    let root = serde_json::from_value(json.clone()).map_err(|e| format!("repacked glTF: {e}"))?;
    let doc = gltf::Document::from_json(root).map_err(|e| format!("repacked glTF: {e}"))?;
    src.json = json;
    src.doc = doc;
    src.buffers.push(gltf::buffer::Data(bytes));
    for (i, bytes) in encoded {
        src.images[i] = bytes;
    }
    src.warnings.push(format!(
        "conform: UV charts repacked to {}x{} with four-texel islands and two-texel gutters",
        packed.size, packed.size
    ));
    Ok(())
}

fn copy_accessor(
    src: &GltfSource,
    id: usize,
    xref: &[u32],
    json: &mut serde_json::Value,
    bytes: &mut Vec<u8>,
) -> usize {
    let accessor = src.doc.accessors().nth(id).unwrap();
    let size = accessor.size();
    let stride = size.next_multiple_of(4);
    let mut dense = vec![0; accessor.count() * stride];
    if let Some(view) = accessor.view() {
        let data = &src.buffers[view.buffer().index()].0;
        for (i, element) in dense.chunks_exact_mut(stride).enumerate() {
            let at = view.offset() + accessor.offset() + i * view.stride().unwrap_or(size);
            element[..size].copy_from_slice(&data[at..at + size]);
        }
    }
    if let Some(sparse) = accessor.sparse() {
        let indices = sparse.indices();
        let iv = indices.view();
        let ib = &src.buffers[iv.buffer().index()].0;
        let values = sparse.values();
        let vv = values.view();
        let vb = &src.buffers[vv.buffer().index()].0;
        let width = indices.index_type().size();
        for i in 0..sparse.count() {
            let at = iv.offset() + indices.offset() + i * width;
            let mut index = [0u8; 4];
            index[..width].copy_from_slice(&ib[at..at + width]);
            let index = u32::from_le_bytes(index) as usize;
            let at = vv.offset() + values.offset() + i * size;
            dense[index * stride..index * stride + size].copy_from_slice(&vb[at..at + size]);
        }
    }
    let data: Vec<u8> = xref
        .iter()
        .flat_map(|&i| {
            dense[i as usize * stride..(i as usize + 1) * stride]
                .iter()
                .copied()
        })
        .collect();
    let mut description = src.json["accessors"][id].clone();
    description["count"] = json!(xref.len());
    append_accessor(
        json,
        bytes,
        src.buffers.len(),
        description,
        &data,
        Some(stride),
    )
}

fn append_accessor(
    json: &mut serde_json::Value,
    bytes: &mut Vec<u8>,
    buffer: usize,
    mut description: serde_json::Value,
    data: &[u8],
    stride: Option<usize>,
) -> usize {
    while !bytes.len().is_multiple_of(4) {
        bytes.push(0);
    }
    let view = json["bufferViews"].as_array().unwrap().len();
    let mut value = json!({"buffer":buffer,"byteOffset":bytes.len(),"byteLength":data.len()});
    if let Some(stride) = stride {
        value["byteStride"] = json!(stride);
    }
    json["bufferViews"].as_array_mut().unwrap().push(value);
    bytes.extend(data);
    let fields = description.as_object_mut().unwrap();
    fields.remove("sparse");
    fields.remove("byteOffset");
    fields.insert("bufferView".into(), json!(view));
    let accessor = json["accessors"].as_array().unwrap().len();
    json["accessors"].as_array_mut().unwrap().push(description);
    accessor
}

fn clear_transforms(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(map) => {
            if let Some(serde_json::Value::Object(ext)) = map.get_mut("extensions") {
                ext.remove("KHR_texture_transform");
            }
            for value in map.values_mut() {
                clear_transforms(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                clear_transforms(value);
            }
        }
        _ => {}
    }
}
