//! Full-tile solids merge into few rectangles; partial shapes stay per
//! tile (full-width neighbours on a row merge); the order is deterministic.

use crate::support;
use glam::Vec2;
use og_tilemap::{Rect, collision_rects, collision_rects_all};
use support::*;

const FLOOR: usize = 0;
const SOLID: i32 = 60;

fn r(x0: f32, y0: f32, x1: f32, y1: f32) -> Rect {
    Rect {
        min: Vec2::new(x0, y0),
        max: Vec2::new(x1, y1),
    }
}

#[test]
fn a_room_outline_is_four_rectangles() {
    let mut s = state(&map(8, 6), "filled");
    for x in 0..8 {
        s.set(FLOOR, x, 0, SOLID).unwrap();
        s.set(FLOOR, x, 5, SOLID).unwrap();
    }
    for y in 1..5 {
        s.set(FLOOR, 0, y, SOLID).unwrap();
        s.set(FLOOR, 7, y, SOLID).unwrap();
    }
    assert_eq!(
        collision_rects(&s, FLOOR),
        vec![
            r(0.0, -1.0, 8.0, 0.0),
            r(0.0, -5.0, 1.0, -1.0),
            r(7.0, -5.0, 8.0, -1.0),
            r(0.0, -6.0, 8.0, -5.0),
        ]
    );
    // A full block becomes one rectangle.
    for y in 1..5 {
        for x in 1..7 {
            s.set(FLOOR, x, y, SOLID).unwrap();
        }
    }
    assert_eq!(collision_rects(&s, FLOOR), vec![r(0.0, -6.0, 8.0, 0.0)]);
}

#[test]
fn only_layers_with_collision_and_their_offsets_count() {
    let mut a = map(3, 3);
    a.layers[0].cells[0] = 61; // tile 60 + 1: solid
    a.layers[0].offset = Vec2::new(10.0, 0.0);
    let s = state(&a, "filled");
    assert!(collision_rects_all(&s).is_empty(), "no layer has collision");
    a.layers[0].collision = true;
    let s = state(&a, "filled");
    assert_eq!(collision_rects_all(&s), vec![r(10.0, -1.0, 11.0, 0.0)]);
}
