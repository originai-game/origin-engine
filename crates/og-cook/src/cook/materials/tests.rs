use super::*;

const LIBRARY: &str = "type materials
version 1
---
material \"brick\" {
  label wall
  tile 2 1
  brief \"London stock brick, soot-darkened, lime mortar\"
  relief 0.02
  map \"textures/brick.png\"
  normalMap \"textures/brick-n.png\"
}
material \"shopfront\" {
  label shopfront
  tile 4 4
  facade true
  brief \"a Victorian shopfront bay\"
  map \"textures/shop.png\"
}
material \"glass\" {
  label glass
  color 0.02 0.03 0.04
}
";

/// A 64 x 32 image that is not periodic: a ramp across, another up, and
/// a fine checker on top, as a photo's lighting gradient over its detail.
fn ramp(w: u32, h: u32) -> Vec<u8> {
    let mut img = image::RgbImage::new(w, h);
    for (x, y, p) in img.enumerate_pixels_mut() {
        let detail = if (x / 2 + y / 2) % 2 == 0 { 20.0 } else { 0.0 };
        let r = 40.0 + 160.0 * x as f64 / (w - 1) as f64 + detail;
        let g = 60.0 + 120.0 * y as f64 / (h - 1) as f64 + detail;
        *p = image::Rgb([r as u8, g as u8, (90.0 + detail) as u8]);
    }
    let mut out = Vec::new();
    img.write_to(&mut std::io::Cursor::new(&mut out), image::ImageFormat::Png)
        .unwrap();
    out
}

fn rgb(png: &[u8]) -> image::RgbImage {
    image::load_from_memory(png).unwrap().to_rgb8()
}

#[test]
fn a_library_asks_for_its_briefed_maps_only() {
    let r = requests(LIBRARY).unwrap();
    assert_eq!(r.len(), 2);
    assert_eq!(
        (r[0].kind, r[0].source.as_str(), r[0].tile),
        ("material", "textures/brick.source.png", [2.0, 1.0])
    );
    assert_eq!(r[0].normal_map.as_deref(), Some("textures/brick-n.png"));
    assert_eq!((r[1].kind, r[1].normal_map.as_ref()), ("facade", None));
    // A relief with no normal map to write is refused, naming the material.
    let e = requests(&LIBRARY.replace("  normalMap \"textures/brick-n.png\"\n", "")).unwrap_err();
    assert!(e.contains("`brick`") && e.contains("normalMap"), "{e}");
}

#[test]
fn make_closes_the_seam_and_keeps_the_detail() {
    let r = &requests(LIBRARY).unwrap()[0];
    let m = make(r, &ramp(64, 32)).unwrap();
    // From a step far over the detail's to one no larger than it.
    assert!(m.seam_before > 5.0 * m.interior, "{m:?}");
    assert!(m.seam_after < 1.2 * m.interior, "{m:?}");
    assert_eq!(m.density, [32.0, 32.0]);
    let map = rgb(&m.files[0].1);
    // The checker survives: neighbours inside still differ by its step.
    let step = map.get_pixel(20, 10)[2] as i32 - map.get_pixel(22, 10)[2] as i32;
    assert!(step.abs() >= 15, "{step}");
    // Made twice, the same bytes.
    assert_eq!(make(r, &ramp(64, 32)).unwrap().files, m.files);
}

#[test]
fn a_facade_wraps_along_the_street_not_up_it() {
    let r = &requests(LIBRARY).unwrap()[1];
    let m = make(r, &ramp(64, 32)).unwrap();
    let map = rgb(&m.files[0].1);
    let (w, h) = map.dimensions();
    let across: i32 = (0..h)
        .map(|y| (map.get_pixel(0, y)[0] as i32 - map.get_pixel(w - 1, y)[0] as i32).abs())
        .sum::<i32>()
        / h as i32;
    let up: i32 = (0..w)
        .map(|x| (map.get_pixel(x, 0)[1] as i32 - map.get_pixel(x, h - 1)[1] as i32).abs())
        .sum::<i32>()
        / w as i32;
    // No larger than the checker step (20).
    assert!(across <= 20, "{across}");
    // Its ground stays its ground: the ramp up the image is kept.
    assert!(up > 60, "{up}");
}

#[test]
fn the_normal_of_a_slope_leans_away_from_its_rise() {
    // Brighter to the right and towards the top of the image: the surface
    // rises right and up, so its normal leans left (red under 128) and
    // down (green under 128, glTF's green up the image).
    let (w, h) = (16u32, 16u32);
    let mut img = image::RgbImage::new(w, h);
    for (x, y, p) in img.enumerate_pixels_mut() {
        // Matching edge heights need no periodic correction. At (6,10)
        // this tent rises twice as steeply right as up.
        let v = (60 + 8 * x.min(w - 1 - x) + 4 * y.min(h - 1 - y)) as u8;
        *p = image::Rgb([v, v, v]);
    }
    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();
    let mut r = requests(LIBRARY).unwrap().remove(0);
    r.relief = 1.0;
    r.tile = [1.0, 1.0];
    let m = make(&r, &bytes).unwrap();
    let n = rgb(&m.files[1].1);
    let c = n.get_pixel(6, 10).0;
    assert!(c[0] < 120 && c[1] < 125 && c[2] > 128, "{c:?}");
    assert!(c[0] < c[1], "the steeper rise is across: {c:?}");
    // A negative relief sinks the lighter: the same slope, leaning the
    // other way.
    r.relief = -1.0;
    let sunk = make(&r, &bytes).unwrap();
    let s = rgb(&sunk.files[1].1).get_pixel(6, 10).0;
    assert!(s[0] > 136 && s[1] > 131, "{s:?}");
}

#[test]
fn an_image_off_a_power_of_two_is_resampled_down_to_one() {
    let r = &requests(LIBRARY).unwrap()[0];
    let m = make(r, &ramp(100, 40)).unwrap();
    assert_eq!((m.width, m.height), (64, 32));
    assert_eq!(rgb(&m.files[0].1).dimensions(), (64, 32));
    assert_eq!(m.density, [32.0, 32.0]);
    let e = make(r, &ramp(100, 6)).unwrap_err();
    assert!(
        e.contains("brick.source.png") && e.contains("100 x 6"),
        "{e}"
    );
}

#[test]
fn packed_roughness_and_cavity_maps_follow_the_authored_numbers() {
    let library = LIBRARY.replace("  relief 0.02", "  relief 0.02\n  roughnessRange 0.2 0.8\n  metalnessMap \"textures/brick-mr.png\"\n  occlusion 1 1\n  aoMap \"textures/brick-ao.png\"");
    let mut r = requests(&library).unwrap().remove(0);
    let mut img = image::RgbImage::from_pixel(16, 16, image::Rgb([255; 3]));
    img.put_pixel(8, 8, image::Rgb([0; 3]));
    let mut bytes = Vec::new();
    img.write_to(
        &mut std::io::Cursor::new(&mut bytes),
        image::ImageFormat::Png,
    )
    .unwrap();
    let m = make(&r, &bytes).unwrap();
    let mr = rgb(&m.files[2].1);
    assert_eq!(mr.get_pixel(8, 8).0, [255, 51, 255]);
    assert_eq!(mr.get_pixel(4, 4).0, [255, 204, 255]);
    let ao = rgb(&m.files[3].1);
    assert_eq!(ao.get_pixel(8, 8).0, [0; 3], "the cavity is dark");
    assert_eq!(
        ao.get_pixel(4, 4).0,
        [255; 3],
        "a flat surface is not darkened"
    );
    r.relief = -0.02;
    let inverted = make(&r, &bytes).unwrap();
    assert_eq!(
        rgb(&inverted.files[3].1).get_pixel(8, 8).0,
        [255; 3],
        "negative relief makes the dark texel a peak"
    );
}

#[test]
fn material_destinations_cannot_escape_or_overwrite_inputs() {
    for path in [
        "../brick.png",
        "/brick.png",
        "textures/../brick.png",
        "textures/./brick.png",
        "textures//brick.png",
        "C:\\brick.png",
    ] {
        let text = LIBRARY.replace("textures/brick.png", path);
        assert!(requests(&text).is_err(), "{path}");
    }
    let text = LIBRARY.replace("textures/brick-n.png", "textures/brick.source.png");
    assert!(requests(&text).unwrap_err().contains("used twice"));
}
