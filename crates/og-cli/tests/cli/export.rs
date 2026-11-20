//! `og export gltf`: a cooked model leaves as a glTF binary that another
//! loader reads back as the engine draws it.
//!
//! A plant (two primitives: bark, and leaves in vertex colours) and a
//! textured prop drawn at two placements are exported, reloaded with the
//! gltf crate and held against the cooked models.
//!
//! Ways this could fail: a level other than the one asked for is written
//! (its triangles differ from the cooked level's); the instances are lost
//! or placed at the node's own transform (the bounds shrink or move); the
//! leaves' vertex colours are dropped (they draw the default grey); a
//! texture is not written, is attached to the wrong slot, or comes back
//! black or with its channels swapped.

use super::*;
use glam::{Mat4, Vec3};

const PLANT: &str = r#"type model
version 1
---
plant "oxeye" {
  kind daisy
  seed 7
  trunk {
    length 0.34
  }
  flowers {
    petals 16
    size 0.05
    color 1 1 0.96
    centerColor 0.95 0.7 0.08
  }
}
"#;

const POST: &str = r#"type model
version 1
---
material "painted" {
  color 1 1 1
  roughness 0.7
  map "red.png"
}
material "iron" {
  color 0.05 0.05 0.055
  metalness 1
  roughness 0.5
}
part "post" {
  material "painted"
  boxGeometry {
    width 0.2
    height 1
    depth 0.2
  }
  at {
    position -2 0.5 0
  }
  at {
    position 3 0.5 1
  }
}
part "cap" {
  material "iron"
  position 0 1.2 0
  sphereGeometry {
    radius 0.3
  }
}
"#;

/// The exported file at `out`, its buffers and images decoded.
fn reload(
    out: &Path,
) -> (
    gltf::Document,
    Vec<gltf::buffer::Data>,
    Vec<gltf::image::Data>,
) {
    gltf::import_slice(std::fs::read(out).unwrap()).unwrap()
}

/// The world-space bounds of every drawn vertex of the default scene, and
/// its triangles.
fn drawn(doc: &gltf::Document, buffers: &[gltf::buffer::Data]) -> (Vec3, Vec3, usize) {
    let (mut lo, mut hi, mut tris) = (Vec3::MAX, Vec3::MIN, 0);
    let mut todo: Vec<(gltf::Node, Mat4)> = doc
        .default_scene()
        .unwrap()
        .nodes()
        .map(|n| (n, Mat4::IDENTITY))
        .collect();
    while let Some((n, parent)) = todo.pop() {
        let world = parent * Mat4::from_cols_array_2d(&n.transform().matrix());
        if let Some(m) = n.mesh() {
            for p in m.primitives() {
                let r = p.reader(|b| Some(&buffers[b.index()]));
                for v in r.read_positions().unwrap() {
                    let w = world.transform_point3(Vec3::from(v));
                    lo = lo.min(w);
                    hi = hi.max(w);
                }
                tris += r.read_indices().unwrap().into_u32().count() / 3;
            }
        }
        todo.extend(n.children().map(|c| (c, world)));
    }
    (lo, hi, tris)
}

#[test]
fn an_exported_model_reloads_as_the_cooked_one() {
    let dir = TempDir::new("og-export-gltf");
    std::fs::write(
        dir.join("project.ogt"),
        "type project\nversion 1\n---\nname \"Export\"\nentry \"main.ts\"\n",
    )
    .unwrap();
    std::fs::write(dir.join("main.ts"), "export default {};").unwrap();
    let models = dir.join("assets/models");
    std::fs::create_dir_all(&models).unwrap();
    std::fs::write(models.join("oxeye.ogt"), PLANT).unwrap();
    std::fs::write(models.join("post.ogt"), POST).unwrap();
    image::RgbaImage::from_pixel(64, 64, image::Rgba([200, 40, 40, 255]))
        .save(models.join("red.png"))
        .unwrap();
    let export = |asset: &str, out: &Path, lod: Option<&str>| {
        let mut c = og();
        c.args(["export", "gltf", asset, dir.to_str().unwrap(), "-o"])
            .arg(out);
        if let Some(l) = lod {
            c.args(["--lod", l]);
        }
        let o = c.output().unwrap();
        assert!(o.status.success(), "{}", text(&o));
    };

    // The plant, at its full mesh and at its next level.
    let (full, coarse) = (dir.join("out/oxeye.glb"), dir.join("out/oxeye-1.glb"));
    export("models/oxeye.ogt", &full, None);
    let bundle = og_cook::runtime::Bundle::open(&dir.join("cooked")).unwrap();
    let plant = bundle.load_model("models/oxeye.ogt").unwrap();
    assert!(
        plant.primitives.iter().any(|p| p.lods.len() > 1),
        "the plant has no LOD chain to choose from"
    );
    let (doc, buffers, _) = reload(&full);
    assert_eq!(doc.meshes().count(), plant.meshes.len());
    assert_eq!(doc.materials().count(), plant.materials.len());
    assert_eq!(doc.textures().count(), plant.textures.len());
    let (lo, hi, tris) = drawn(&doc, &buffers);
    let level = |l: usize| -> usize { plant.primitives.iter().map(|p| p.triangle_count(l)).sum() };
    assert_eq!(tris, level(0));
    let tol = (plant.max - plant.min).max_element() * 1e-3;
    assert!(
        lo.abs_diff_eq(plant.min, tol) && hi.abs_diff_eq(plant.max, tol),
        "exported bounds {lo}..{hi}, cooked {}..{}",
        plant.min,
        plant.max
    );
    // The petals and the eye are vertex colours: white and yellow.
    let colors: Vec<[f32; 4]> = doc
        .meshes()
        .flat_map(|m| m.primitives())
        .filter_map(|p| {
            p.reader(|b| Some(&buffers[b.index()]))
                .read_colors(0)
                .map(|c| c.into_rgba_f32().collect::<Vec<_>>())
        })
        .flatten()
        .collect();
    assert!(
        colors.iter().any(|c| c[0] > 0.9 && c[2] < 0.3),
        "no yellow eye"
    );
    assert!(
        colors.iter().any(|c| c[0] > 0.9 && c[2] > 0.9),
        "no white petals"
    );
    export("models/oxeye.ogt", &coarse, Some("1"));
    let (doc, buffers, _) = reload(&coarse);
    assert_eq!(drawn(&doc, &buffers).2, level(1));
    assert!(level(1) < level(0));

    // The post: its two placements, and its colour map.
    let out = dir.join("out/post.glb");
    export("models/post.ogt", &out, None);
    let post = bundle.load_model("models/post.ogt").unwrap();
    let (doc, buffers, images) = reload(&out);
    assert_eq!(doc.materials().count(), post.materials.len());
    assert_eq!(doc.textures().count(), 1);
    let (lo, hi, _) = drawn(&doc, &buffers);
    let tol = (post.max - post.min).max_element() * 1e-3;
    assert!(
        lo.abs_diff_eq(post.min, tol) && hi.abs_diff_eq(post.max, tol),
        "exported bounds {lo}..{hi}, cooked {}..{}",
        post.min,
        post.max
    );
    assert!(
        hi.x > 3.0 && lo.x < -2.0,
        "the placements are lost: {lo}..{hi}"
    );
    let painted = doc
        .materials()
        .find(|m| m.name() == Some("painted"))
        .unwrap();
    let base = painted
        .pbr_metallic_roughness()
        .base_color_texture()
        .expect("the painted material has no colour map");
    let img = &images[base.texture().source().index()];
    let cooked = bundle.load_texture(&post.textures[0].id).unwrap();
    assert_eq!(
        (img.width, img.height),
        (cooked.meta.width, cooked.meta.height)
    );
    let px = img.pixels.len() / 4;
    let mean: Vec<f64> = (0..3)
        .map(|c| {
            img.pixels
                .iter()
                .skip(c)
                .step_by(4)
                .map(|&v| v as f64)
                .sum::<f64>()
                / px as f64
        })
        .collect();
    assert!(
        (mean[0] - 200.0).abs() < 12.0
            && (mean[1] - 40.0).abs() < 12.0
            && (mean[2] - 40.0).abs() < 12.0,
        "the colour map came back {mean:?}"
    );
    let iron = doc.materials().find(|m| m.name() == Some("iron")).unwrap();
    assert_eq!(iron.pbr_metallic_roughness().metallic_factor(), 1.0);
}
