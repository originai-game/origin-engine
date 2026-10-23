//! The depth wave's render targets (docs/depth.md §8): the reference
//! frames of a security monitor fed by a second camera (`views-monitor`),
//! a mirror (`views-mirror`) and split-screen (`views-split`), and from
//! frames: an object on a layer only the monitor's camera sees shows on
//! the monitor and nowhere else.

use og_tests::gfx;
use std::path::PathBuf;

fn out_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_TARGET_TMPDIR")).join("og-targets")
}

fn check(scene: &str) {
    match gfx::check(scene, &out_dir()) {
        Ok(r) => println!("{r}"),
        Err(e) => panic!("{e}"),
    }
}

#[test]
fn views_monitor_matches_its_reference() {
    check("views-monitor");
}

#[test]
fn views_mirror_matches_its_reference() {
    check("views-mirror");
}

#[test]
fn views_split_matches_its_reference() {
    check("views-split");
}

/// The magenta cube is on layer 1 alone: the main camera (layer 0) leaves
/// it out, the monitor's camera (layers 0 and 1) draws it on the screen.
#[test]
fn a_layer_the_main_camera_lacks_shows_only_on_the_monitor() {
    let frame = gfx::render("views-monitor").expect("render").image;
    let magenta = |x: u32, y: u32| {
        let p = &frame.pixels[((y * frame.width + x) * 4) as usize..][..3];
        p[0] > 150 && p[2] > 130 && p[1] < 90
    };
    // The monitor on the back wall spans about x 545..735, y 183..290.
    let (mut on_screen, mut elsewhere) = (0, 0);
    for y in 0..frame.height {
        for x in 0..frame.width {
            if magenta(x, y) {
                if (545..735).contains(&x) && (183..290).contains(&y) {
                    on_screen += 1;
                } else {
                    elsewhere += 1;
                }
            }
        }
    }
    assert!(on_screen > 20, "{on_screen} magenta pixels on the monitor");
    assert_eq!(elsewhere, 0, "the main camera drew the layer-1 cube");
}
