//! Registered views into prepared material UVs, using the existing CPU projection kernel.
//! Cameras and alpha masks are explicit inputs; cook never invents or fits concept art.
use super::{Block, Line, Scene, check, number, whole};
use glam::{Vec2, Vec3};
use og_bake::projection::{
    GBuffer, Level, Paint, Texels, ViewKind, ViewSpec, fill_unpainted, padded, project, silhouette,
};
use og_shape::tools::{Image, Texture};

pub(crate) const LINES: &[Line] = &[
    Line::new(
        "size",
        "<1..8192>",
        "1024",
        "Output map side in texels per primitive.",
        "",
    ),
    Line::new(
        "padding",
        "<0..8192>",
        "4",
        "Grow UV chart gutters after filling unseen texels.",
        "",
    ),
    Line::new(
        "view",
        "\"<image>\" { ... }",
        "required",
        "Repeat square RGBA images registered to explicit cameras. Paths are model-relative; alpha zero leaves a pixel unpainted. No implicit silhouette fitting.",
        "",
    ),
];
pub(crate) const VIEW: &[Line] = &[
    Line::new(
        "position",
        "<x> <y> <z>",
        "required",
        "Camera position in model-local metres, Y up.",
        "",
    ),
    Line::new(
        "target",
        "<x> <y> <z>",
        "required",
        "Camera target, distinct from position. Vertical views use the kernel's fixed X right axis.",
        "",
    ),
    Line::new(
        "fov",
        "<degrees, 0..179>",
        "0",
        "Positive perspective vertical field of view; exclusive with ortho.",
        "",
    ),
    Line::new(
        "ortho",
        "<metres > 0>",
        "0",
        "Orthographic half-height and half-width; exclusive with positive fov. One projection size is required.",
        "",
    ),
];

pub(super) struct Projection {
    size: u32,
    padding: u32,
    views: Vec<(ViewSpec, Image)>,
}

pub(super) fn read(
    b: &Block,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Projection, String> {
    let vector = |b: &Block, key: &str| -> Result<Vec3, String> {
        let e = b
            .get(key)
            .ok_or_else(|| format!("projection view needs {key}"))?;
        if e.values.len() != 3 || e.block.is_some() {
            return Err(format!("{key} takes x y z"));
        }
        let v = Vec3::new(
            e.f32(0).map_err(|e| e.to_string())?,
            e.f32(1).map_err(|e| e.to_string())?,
            e.f32(2).map_err(|e| e.to_string())?,
        );
        if !v.is_finite() {
            return Err(format!("{key} must be finite"));
        }
        Ok(v)
    };
    let mut views = Vec::new();
    for e in b.get_all("view") {
        if e.values.len() != 1 {
            return Err("view takes an image path and camera block".into());
        }
        let camera = e.block.as_ref().ok_or("view needs a camera block")?;
        check(camera, VIEW)?;
        let position = vector(camera, "position")?;
        let target = vector(camera, "target")?;
        if position.distance_squared(target) <= 1e-12 {
            return Err("projection camera position and target must differ".into());
        }
        let fov = number(camera, "fov", 0.0, 179.0)? as f32;
        let ortho = number(camera, "ortho", 0.0, 1e9)? as f32;
        if (fov > 0.0) == (ortho > 0.0) {
            return Err("projection camera needs either positive fov or positive ortho".into());
        }
        let mut spec = ViewSpec::looking(position, target, fov, ViewKind::Ground);
        spec.ortho = ortho;
        let path = e.string(0).map_err(|e| e.to_string())?;
        let decoded = crate::cook::decode_image(&read(path)?)?;
        let image = Image {
            width: decoded.width,
            height: decoded.height,
            rgba: decoded.rgba,
        };
        if image.width != image.height || image.width == 0 || image.width > 8192 {
            return Err(format!(
                "projection view {path}: expected a square image, side 1..8192"
            ));
        }
        views.push((spec, image));
    }
    let size = whole(b, "size", 1024, 8192)?;
    if size == 0 || views.is_empty() {
        return Err("project needs positive size and at least one registered view".into());
    }
    Ok(Projection {
        size,
        padding: whole(b, "padding", 4, 8192)?,
        views,
    })
}

pub(super) fn apply(scene: &mut Scene, d: &Projection) -> Result<(), String> {
    // One BVH for occlusion across every primitive; each UV raster targets only
    // its own material map, even when multiple primitives overlap in UV space.
    if scene.parts.len() > u16::MAX as usize {
        return Err("projection has too many primitives".into());
    }
    let mut positions = Vec::new();
    let mut uvs = Vec::new();
    let mut labels = Vec::new();
    for (i, part) in scene.parts.iter().enumerate() {
        let uv = part
            .solid
            .uvs
            .as_ref()
            .ok_or_else(|| format!("projection part {:?} needs prepared UVs", part.name))?;
        for (triangle, uv) in part.solid.triangles.iter().zip(uv) {
            for (&index, uv) in triangle.iter().zip(uv) {
                let p = part.solid.positions[index as usize];
                positions.push(Vec3::new(p.x as f32, p.z as f32, -p.y as f32) * 0.001);
                uvs.push(Vec2::new(uv.x as f32, 1.0 - uv.y as f32));
            }
            labels.push(i as u16);
        }
    }
    let level = |uvs: Vec<Vec2>| {
        Level::new(
            positions.clone(),
            Vec::new(),
            uvs,
            (0..positions.len() as u32).collect(),
            labels.clone(),
            scene.parts.iter().map(|p| p.name.clone()).collect(),
        )
    };
    let whole = level(uvs.clone());
    let views: Vec<_> = d
        .views
        .iter()
        .map(|(spec, image)| {
            let g = GBuffer::cast(&whole, *spec, image.width, 0);
            let edge = silhouette(&g);
            (g, edge, image.rgba.as_chunks::<4>().0)
        })
        .collect();
    let mut maps = Vec::new();
    for (i, part) in scene.parts.iter().enumerate() {
        let own = level(
            uvs.iter()
                .enumerate()
                .map(|(j, uv)| {
                    if labels[j / 3] as usize == i {
                        *uv
                    } else {
                        Vec2::ZERO
                    }
                })
                .collect(),
        );
        let texels = Texels::rasterise(&own, d.size, 0);
        let mut paint = Paint::new(&texels);
        for (g, edge, image) in &views {
            project(&texels, &mut paint, g, edge, image, 1.0, 0);
        }
        if paint.coverage() == 0.0 {
            maps.push(None);
            continue;
        }
        fill_unpainted(&texels, &mut paint, 0);
        let rgb = padded(&texels, &paint, d.padding, [0, 0, 0]);
        let image = Image {
            width: d.size,
            height: d.size,
            rgba: rgb
                .into_iter()
                .flat_map(|p| [p[0], p[1], p[2], 255])
                .collect(),
        };
        maps.push(Some(Texture::from_bytes(
            &format!("{}.project.png", part.name),
            (og_shape_formats::texture::CODEC.png)(&image),
        )));
    }
    if maps.iter().all(Option::is_none) {
        return Err("no registered projection view painted a texel".into());
    }
    for (part, map) in scene.parts.iter_mut().zip(maps) {
        if let Some(map) = map {
            part.material.map = Some(map);
            part.material.color = Some([1.0; 3]);
        }
    }
    Ok(())
}
