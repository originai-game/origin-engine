//! A painted sphere and a material using every field, shared by the
//! kernel's golden cases (`golden/look.rs`) and `og model`'s.

use og_shape::generators::solids::{Generator, SphereDesc, textured};
use og_shape::math::{DVec3, HashError};
use og_shape::mesh::Solid;
use og_shape::tools::{ColorSpace, Material, Texture};

fn fail(e: impl std::fmt::Display) -> HashError {
    eprintln!("look: {e}");
    HashError::NaN
}

pub fn png(seed: u8, w: u32, h: u32) -> Vec<u8> {
    og_shape_formats::png::write::png(&og_shape::tools::Image {
        width: w,
        height: h,
        rgba: (0..w * h * 4)
            .map(|i| ((i * 37 + u32::from(seed) * 11) % 256) as u8)
            .collect(),
    })
}

pub fn affine(p: DVec3) -> [f64; 4] {
    [
        0.5 + p.x / 200.0,
        0.5 + p.y / 300.0,
        0.5 - p.z / 250.0,
        0.75,
    ]
}

pub fn painted_sphere() -> Result<Solid, HashError> {
    let mut s = textured(&Generator::Sphere(SphereDesc {
        radius: 20.0,
        ..Default::default()
    }))
    .map_err(fail)?;
    s.colors = Some(
        s.triangles
            .iter()
            .map(|t| t.map(|v| og_shape::mesh::stored(affine(s.positions[v as usize]))))
            .collect(),
    );
    Ok(s)
}

pub fn material() -> Material {
    let tex = |seed: u8| {
        Some(Texture::from_bytes(
            &format!("t{seed}.png"),
            png(seed, 4, 2),
        ))
    };
    Material {
        name: "painted".into(),
        color: Some([0.8, 0.5, 0.25]),
        metalness: 0.25,
        roughness: Some(0.375),
        emissive: [0.25, 0.0, 0.5],
        opacity: Some(0.5),
        map: tex(0),
        color_space: ColorSpace::Linear,
        emissive_intensity: Some(4.0),
        alpha_test: Some(0.25),
        double_sided: true,
        metalness_map: tex(1),
        normal_map: tex(2),
        normal_scale: Some(0.5),
        emissive_map: tex(3),
        ao_map: tex(4),
        ao_map_intensity: Some(0.75),
        ..Default::default()
    }
}
