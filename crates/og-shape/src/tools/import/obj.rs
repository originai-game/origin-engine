//! Import of Wavefront OBJ with its `.mtl` into parts.

use super::*;

/// A material as an MTL file gives it: the statements read, its maps by
/// keyword (lower case), and the keywords not read.
#[derive(Clone, Debug, Default)]
pub(crate) struct Mtl {
    pub(crate) kd: Option<[f64; 3]>,
    pub(crate) ke: Option<[f64; 3]>,
    pub(crate) d: Option<f64>,
    pub(crate) pr: Option<f64>,
    pub(crate) pm: Option<f64>,
    pub(crate) maps: BTreeMap<String, String>,
    pub(crate) unread: Vec<String>,
}

/// Reads Wavefront OBJ text. Each `o` starts a part (or each `g`, in a
/// file with no `o`); `usemtl` starts a new surface. Faces of more than
/// three corners are triangulated; negative (relative) indices are read;
/// `vt` becomes [`Solid::uvs`]; `v x y z r g b` becomes
/// [`Solid::colors`] (sRGB in the file, as three.js reads it, made
/// linear; a file whose colours exceed 1 is read as 0..=255), white on a
/// vertex without one. `mtllib` files are read through `resolve` for their
/// texture maps' sizes, and a part's first material becomes its
/// [`ImportedPart::material`]: `Kd` (sRGB), `d` or `Tr`, `Ke`, `Pr`, `Pm`
/// (the PBR extension), `map_Kd`, `map_Ke` and `norm`. Other statements
/// (specular `Ks`, `Ns`, a `bump` height map, …) are listed in
/// [`Imported::skipped`], as are points and lines.
/// Every part's transform is the identity.
pub fn import_obj(bytes: &[u8], d: &Import, resolve: Resolve) -> Result<Imported, ImportError> {
    let text = String::from_utf8_lossy(bytes);
    let unit = if d.unit_mm > 0.0 { d.unit_mm } else { 1000.0 };
    let axes = d.axes()?;
    let has_o = text.lines().any(|l| l.trim_start().starts_with("o "));
    let mut out = Imported::default();
    let mut positions: Vec<DVec3> = Vec::new();
    let mut texcoords: Vec<DVec2> = Vec::new();
    let mut used = BTreeMap::new();
    let mut image_names: BTreeMap<String, ()> = BTreeMap::new();
    // Each material as the MTL gives it, and the bytes of every image read.
    let mut mtl: BTreeMap<String, Mtl> = BTreeMap::new();
    let mut image_bytes: BTreeMap<String, Vec<u8>> = BTreeMap::new();
    // Per OBJ vertex, its colour as the file gives it.
    let mut vertex_colors: Vec<Option<[f64; 3]>> = Vec::new();
    // The part being read: name, global → local vertex map, solid, uvs.
    struct Building {
        name: String,
        local: BTreeMap<u32, u32>,
        solid: Solid,
        uvs: Vec<[DVec2; 3]>,
        all_uv: bool,
        any_uv: bool,
        surface: u32,
        material: String,
        materials: BTreeMap<String, u32>,
        /// Per local vertex, its OBJ vertex.
        global: Vec<usize>,
    }
    let start = |name: &str, used: &mut BTreeMap<String, ()>| Building {
        name: unique(name, used),
        local: BTreeMap::new(),
        solid: Solid::default(),
        uvs: Vec::new(),
        all_uv: true,
        any_uv: false,
        surface: 0,
        material: String::new(),
        materials: BTreeMap::new(),
        global: Vec::new(),
    };
    let finish = |b: Building, out: &mut Imported, globals: &mut Vec<Vec<usize>>| {
        if !b.solid.triangles.is_empty() {
            if !b.all_uv && b.any_uv {
                out.skipped
                    .push(format!("{}: vt on some faces only, so no UVs", b.name));
            }
            globals.push(b.global);
            out.parts.push(ImportedPart {
                clips: Vec::new(),
                name: b.name,
                solid: Solid {
                    uvs: if b.all_uv { Some(b.uvs) } else { None },
                    ..b.solid
                },
                transform: Affine::IDENTITY,
                material: Material {
                    name: b.material,
                    ..Default::default()
                },
                defaults: Vec::new(),
            });
        }
    };
    let mut globals: Vec<Vec<usize>> = Vec::new();
    let mut cur: Option<Building> = None;
    let (mut points, mut lines) = (0usize, 0usize);
    let mut joined = String::new();
    for (ln, raw) in text.lines().enumerate() {
        // Backslash continues a line.
        if let Some(head) = raw.strip_suffix('\\') {
            joined.push_str(head);
            joined.push(' ');
            continue;
        }
        joined.push_str(raw);
        let line = std::mem::take(&mut joined);
        let line = line.split('#').next().unwrap_or("");
        let mut tok = line.split_whitespace();
        let Some(key) = tok.next() else { continue };
        let bad = |what: &str| fmt_err(format!("OBJ line {}: {what}", ln + 1));
        let num = |s: Option<&str>| -> Result<f64, ImportError> {
            s.and_then(|s| s.parse::<f64>().ok())
                .ok_or_else(|| bad("a number is missing or unreadable"))
        };
        match key {
            "v" => {
                let p = DVec3::new(num(tok.next())?, num(tok.next())?, num(tok.next())?);
                let p = axes.apply(p) * unit;
                if !p.is_finite() {
                    return Err(ImportError::NotFinite("OBJ vertex".into()));
                }
                positions.push(p);
                // `v x y z r g b`: a vertex colour.
                vertex_colors.push(if tok.clone().nth(2).is_some() {
                    Some([num(tok.next())?, num(tok.next())?, num(tok.next())?])
                } else {
                    None
                });
            }
            "vt" => {
                let u = num(tok.next())?;
                let v = tok.next().map_or(Ok(0.0), |s| num(Some(s)))?;
                texcoords.push(DVec2::new(u, v));
            }
            "o" | "g" if (key == "o") == has_o => {
                let name = tok.collect::<Vec<_>>().join(" ");
                if let Some(b) = cur.take() {
                    finish(b, &mut out, &mut globals);
                }
                cur = Some(start(&name, &mut used));
            }
            "usemtl" => {
                let b = cur.get_or_insert_with(|| start("object", &mut used));
                let name = tok.collect::<Vec<_>>().join(" ");
                if b.material.is_empty() {
                    b.material = name.clone();
                }
                let next = b.materials.len() as u32;
                b.surface = *b.materials.entry(name).or_insert(next);
            }
            "mtllib" => {
                let file = tok.collect::<Vec<_>>().join(" ");
                match fetch(&file, resolve) {
                    Ok(bytes) => {
                        let mut current = String::new();
                        for l in String::from_utf8_lossy(&bytes).lines() {
                            let l = l.split('#').next().unwrap_or("").trim();
                            let Some(key) = l.split_whitespace().next() else {
                                continue;
                            };
                            let lower = key.to_ascii_lowercase();
                            let rest: Vec<&str> = l.split_whitespace().skip(1).collect();
                            if lower == "newmtl" {
                                current = rest.join(" ");
                                mtl.entry(current.clone()).or_default();
                                continue;
                            }
                            let m = mtl.entry(current.clone()).or_default();
                            let nums: Vec<f64> =
                                rest.iter().filter_map(|x| x.parse().ok()).collect();
                            let rgb = || (nums.len() >= 3).then(|| [nums[0], nums[1], nums[2]]);
                            let one = || nums.first().copied();
                            let is_map = lower.starts_with("map_")
                                || ["bump", "disp", "norm", "decal", "refl"]
                                    .contains(&lower.as_str());
                            let image =
                                is_map.then(|| rest.last().map(|x| x.to_string())).flatten();
                            let read = match lower.as_str() {
                                "kd" => rgb().map(|c| m.kd = Some(c)).is_some(),
                                "ke" => rgb().map(|c| m.ke = Some(c)).is_some(),
                                "d" => one().map(|x| m.d = Some(x)).is_some(),
                                "tr" => one().map(|x| m.d = Some(1.0 - x)).is_some(),
                                "pr" => one().map(|x| m.pr = Some(x)).is_some(),
                                "pm" => one().map(|x| m.pm = Some(x)).is_some(),
                                "map_kd" | "map_ke" | "norm" => {
                                    if let Some(img) = &image {
                                        m.maps.entry(lower.clone()).or_insert_with(|| img.clone());
                                    }
                                    image.is_some()
                                }
                                _ => false,
                            };
                            if !read && !m.unread.contains(&key.to_string()) {
                                m.unread.push(key.to_string());
                            }
                            if let Some(img) = image {
                                if image_names.insert(img.clone(), ()).is_some() {
                                    continue;
                                }
                                out.images.push(match fetch(&img, resolve) {
                                    Ok(b) => {
                                        let info = ImageInfo::from_bytes(&img, &b);
                                        image_bytes.insert(img.clone(), b);
                                        info
                                    }
                                    Err(e) => ImageInfo::missing(&img, &e.to_string()),
                                });
                            }
                        }
                    }
                    Err(e) => out.skipped.push(format!("material library {file:?}: {e}")),
                }
            }
            "p" => points += 1,
            "l" => lines += 1,
            "f" => {
                let b = cur.get_or_insert_with(|| start("object", &mut used));
                let mut corners: Vec<u32> = Vec::new();
                let mut corner_uv: Vec<Option<DVec2>> = Vec::new();
                for t in tok {
                    let mut f = t.split('/');
                    let resolve_index = |s: Option<&str>, len: usize| -> Option<usize> {
                        let v: i64 = s.filter(|s| !s.is_empty())?.parse().ok()?;
                        let i = if v < 0 { len as i64 + v } else { v - 1 };
                        (0..len as i64).contains(&i).then_some(i as usize)
                    };
                    let vi = resolve_index(f.next(), positions.len())
                        .ok_or_else(|| bad("a face names a vertex that does not exist"))?;
                    let ti = f.next();
                    let uv = match ti.filter(|s| !s.is_empty()) {
                        Some(_) => Some(
                            texcoords[resolve_index(ti, texcoords.len())
                                .ok_or_else(|| bad("a face names a vt that does not exist"))?],
                        ),
                        None => None,
                    };
                    let next = b.solid.positions.len() as u32;
                    let local = *b.local.entry(vi as u32).or_insert(next);
                    if local == next {
                        b.solid.push_vertex(positions[vi]);
                        b.global.push(vi);
                    }
                    corners.push(local);
                    corner_uv.push(uv);
                }
                if corners.len() < 3 {
                    return Err(bad("a face with fewer than three corners"));
                }
                if corners.len() > 3 {
                    out.polygons_triangulated += 1;
                }
                let tris = triangulate(&b.solid.positions, &corners);
                let name = b.name.clone();
                for t in tris {
                    b.solid.push_triangle(t, b.surface, &[&name]);
                    let uv = t.map(|v| {
                        let k = corners.iter().position(|&c| c == v).unwrap_or(0);
                        corner_uv[k]
                    });
                    if uv.iter().all(Option::is_some) {
                        b.any_uv = true;
                        b.uvs.push(uv.map(|u| u.unwrap_or(DVec2::ZERO)));
                    } else {
                        b.all_uv = false;
                        b.uvs.push([DVec2::ZERO; 3]);
                    }
                }
            }
            _ => {}
        }
    }
    if let Some(b) = cur.take() {
        finish(b, &mut out, &mut globals);
    }
    // Vertex colours: sRGB in the file, linear here; 0..=255 if any
    // exceeds 1.
    let colored = vertex_colors.iter().flatten().count();
    let scale = if vertex_colors.iter().flatten().flatten().any(|&c| c > 1.0) {
        255.0
    } else {
        1.0
    };
    let lin = |c: [f64; 3]| {
        let l = c.map(|x| crate::tools::export::linear(x / scale));
        crate::mesh::stored([l[0], l[1], l[2], 1.0])
    };
    for (p, global) in out.parts.iter_mut().zip(&globals) {
        if colored == 0 || !global.iter().any(|&g| vertex_colors[g].is_some()) {
            continue;
        }
        let per_vertex: Vec<[f32; 4]> = global
            .iter()
            .map(|&g| vertex_colors[g].map_or(crate::mesh::WHITE, lin))
            .collect();
        let colors = p
            .solid
            .triangles
            .iter()
            .map(|t| t.map(|v| per_vertex[v as usize]))
            .collect();
        p.solid.colors = Some(colors);
    }
    if colored > 0 && colored < vertex_colors.len() {
        out.skipped.push(format!(
            "{} of {} vertices have no colour and are white",
            vertex_colors.len() - colored,
            vertex_colors.len()
        ));
    }
    for p in &mut out.parts {
        let Some(m) = mtl.get(&p.material.name) else {
            continue;
        };
        let tex = |key: &str| {
            let img = m.maps.get(key)?;
            Some(Texture::from_bytes(img, image_bytes.get(img)?.clone()))
        };
        p.material = Material {
            name: p.material.name.clone(),
            color: m.kd,
            metalness: m.pm.unwrap_or(0.0),
            roughness: m.pr,
            emissive: m.ke.unwrap_or([0.0; 3]),
            opacity: m.d,
            map: tex("map_kd"),
            emissive_map: tex("map_ke"),
            normal_map: tex("norm"),
            ..Default::default()
        };
        for (key, img) in &m.maps {
            if !image_bytes.contains_key(img) {
                out.skipped.push(format!(
                    "{}: its {key} image {img:?} could not be read",
                    p.name
                ));
            }
        }
        if !m.unread.is_empty() {
            out.skipped.push(format!(
                "{}: its material's {} are not read",
                p.name,
                m.unread.join(", ")
            ));
        }
    }
    if points > 0 {
        out.skipped.push(format!("{points} point elements"));
    }
    if lines > 0 {
        out.skipped.push(format!("{lines} line elements"));
    }
    if out.parts.is_empty() {
        return Err(fmt_err("the file has no faces"));
    }
    Ok(out)
}
