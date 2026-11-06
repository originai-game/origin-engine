use super::*;

/// A GIF of a 4x4 block on white drifting 2 texels right, then 1 down,
/// then back where it started: `og sprite inspect` keys the white border
/// and reports each frame's drift from the first, the last frame as a
/// duplicate of the first and a loop seam of 0. Ways this could fail: the
/// background is measured as the figure (bounds the whole canvas), drift
/// is taken from the previous frame, the duplicate goes unreported.
#[test]
fn sprite_inspect_measures_drift_duplicates_and_the_seam() {
    let at = |x: u32, y: u32| {
        image::RgbaImage::from_fn(16, 12, move |px, py| {
            if (x..x + 4).contains(&px) && (y..y + 4).contains(&py) {
                image::Rgba([30, 60, 200, 255])
            } else {
                image::Rgba([255, 255, 255, 255])
            }
        })
    };
    let mut gif = Vec::new();
    {
        let mut enc = image::codecs::gif::GifEncoder::new(&mut gif);
        enc.encode_frames(
            [
                (at(4, 4), 100),
                (at(6, 4), 50),
                (at(6, 5), 100),
                (at(4, 4), 100),
            ]
            .into_iter()
            .map(|(img, ms)| {
                image::Frame::from_parts(img, 0, 0, image::Delay::from_numer_denom_ms(ms, 1))
            }),
        )
        .unwrap();
    }
    let dir = scratch_root();
    let file = dir.join("drift.gif");
    std::fs::write(&file, gif).unwrap();
    let out = og()
        .args(["sprite", "inspect", file.to_str().unwrap(), "--json"])
        .output()
        .unwrap();
    assert!(out.status.success(), "{}", text(&out));
    let r: serde_json::Value = serde_json::from_slice(&out.stdout).unwrap();
    assert_eq!(r["format"], "GIF");
    assert_eq!(r["keyed"]["colour"], "#ffffff");
    let frames = r["frames"].as_array().unwrap();
    assert_eq!(frames.len(), 4);
    assert_eq!(frames[1]["delay"], 0.05);
    assert_eq!(frames[0]["bounds"], serde_json::json!([4, 4, 8, 8]));
    assert_eq!(frames[2]["centroidDrift"], serde_json::json!([2.0, 1.0]));
    assert_eq!(frames[2]["boundsDrift"], serde_json::json!([2, 1, 2, 1]));
    assert_eq!(frames[3]["duplicateOf"], 0);
    assert_eq!(r["maxCentroidDrift"], serde_json::json!([2.0, 1.0]));
    assert_eq!(r["seam"], 0.0);
    assert!(r["stepMean"].as_f64().unwrap() > 0.0);
}
