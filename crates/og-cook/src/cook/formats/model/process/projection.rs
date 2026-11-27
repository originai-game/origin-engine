//! Registered views into prepared material UVs, using the existing CPU projection kernel.
//! Cameras and alpha masks are explicit inputs; cook never invents or fits concept art.
use super::{Scene, input};
use glam::{Vec2, Vec3};
use og_bake::projection::{
    GBuffer, Level, Paint, Texels, ViewKind, ViewSpec, fill_unpainted, padded, project, silhouette,
};
use og_shape::tools::{Image, Texture};

pub(super) struct Projection {
    size: u32,
    padding: u32,
    views: Vec<(ViewSpec, Image)>,
}

pub(super) fn read(
    b: &input::Project,
    read: &dyn Fn(&str) -> Result<Vec<u8>, String>,
) -> Result<Projection, String> {
    let mut views = Vec::new();
    for view in &b.views {
        let position = Vec3::from_array(view.position);
        let target = Vec3::from_array(view.target);
        if position.distance_squared(target) <= 1e-12 {
            return Err("projection camera position and target must differ".into());
        }
        let (fov, ortho) = match view.projection {
            input::Projection::Perspective { fov } => (fov, 0.0),
            input::Projection::Orthographic { ortho } => (0.0, ortho),
        };
        let mut spec = ViewSpec::looking(position, target, fov, ViewKind::Ground);
        spec.ortho = ortho;
        let path = &view.image;
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
    let size = b.size.unwrap_or(1024);
    if size == 0 || views.is_empty() {
        return Err("project needs positive size and at least one registered view".into());
    }
    Ok(Projection {
        size,
        padding: b.padding.unwrap_or(4),
        views,
    })
}

pub(super) fn surface(scene: &Scene, material: Option<usize>) -> Result<Level, String> {
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
                uvs.push(if material.is_none_or(|m| m == i) {
                    Vec2::new(uv.x as f32, 1.0 - uv.y as f32)
                } else {
                    Vec2::ZERO
                });
            }
            labels.push(i as u16);
        }
    }
    let indices = (0..positions.len() as u32).collect();
    Ok(Level::new(
        positions,
        Vec::new(),
        uvs,
        indices,
        labels,
        scene.parts.iter().map(|p| p.name.clone()).collect(),
    ))
}

pub(super) fn apply(scene: &mut Scene, d: &Projection) -> Result<(), String> {
    // One BVH for occlusion across every primitive; each UV raster targets only
    // its own material map, even when multiple primitives overlap in UV space.
    let whole = surface(scene, None)?;
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
        let own = surface(scene, Some(i))?;
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
