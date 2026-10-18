//! Draw chunks: quads at their cells with flipped regions, animated tiles
//! with every frame's region, and culling to a rectangle.

mod support;
use glam::Vec2;
use og_tilemap::{animated_frame, chunk_quads, chunks_in, Tileset, CHUNK, FLIP_X, FLIP_Y};
use support::*;

#[test]
fn quads_carry_their_cell_flips_and_animations() {
    let mut a = map(20, 20);
    a.layers[0].cells[0] = 3; // tile 2
    a.layers[0].cells[1] = 3 | FLIP_X;
    a.layers[0].cells[20] = 3 | FLIP_Y;
    a.layers[0].cells[21] = 51; // tile 50, animated
    a.layers[0].cells[17] = 1; // tile 0, in chunk (1, 0)
    let s = state(&a, "filled");
    let ts = tileset("filled");
    let c = chunk_quads(&s, 0, 0, 0, 128, 128);
    assert_eq!(c.quads.len(), 4);
    let plain = ts.uv(2, 128, 128);
    assert_eq!(c.quads[0].uv, plain);
    assert_eq!(c.quads[0].min, Vec2::new(0.0, -1.0));
    assert_eq!(c.quads[0].size, Vec2::ONE);
    assert_eq!(
        c.quads[1].uv,
        [plain[2], plain[1], plain[0], plain[3]],
        "flipped in x"
    );
    assert_eq!(
        c.quads[2].uv,
        [plain[0], plain[3], plain[2], plain[1]],
        "flipped in y"
    );
    assert_eq!(c.quads[3].min, Vec2::new(1.0, -2.0));
    assert_eq!(c.animated.len(), 1);
    let an = &c.animated[0];
    assert_eq!(an.quad, 3);
    assert_eq!(
        an.frames.to_vec(),
        vec![
            ts.uv(50, 128, 128),
            ts.uv(51, 128, 128),
            ts.uv(52, 128, 128)
        ]
    );
    assert_eq!(an.durations.to_vec(), vec![0.25; 3]);
    assert_eq!(chunk_quads(&s, 0, 1, 0, 128, 128).quads.len(), 1);
}

#[test]
fn regions_are_inset_and_animations_loop() {
    let ts = Tileset::parse(
        "type tileset\nversion 1\n---\ntexture \"t.png\"\ngrid 16 16 4 2\nmargin 1\nspacing 2\n",
    )
    .unwrap();
    let uv = ts.uv(5, 73, 37); // column 1, row 1
    let e = 1.0 / 512.0;
    assert_eq!(
        uv,
        [
            (19.0 + e) / 73.0,
            (19.0 + e) / 37.0,
            (35.0 - e) / 73.0,
            (35.0 - e) / 37.0
        ]
    );
    let d = [0.1, 0.2, 0.3];
    assert_eq!(animated_frame(&d, 0.0), 0);
    assert_eq!(animated_frame(&d, 0.15), 1);
    assert_eq!(animated_frame(&d, 0.35), 2);
    assert_eq!(animated_frame(&d, 0.75), 1, "loops after 0.6 s");
}

#[test]
fn culling_lists_the_chunks_a_rectangle_touches() {
    let s = state(&map(64, 40), "filled");
    let c: Vec<_> = chunks_in(&s, 0, Vec2::new(10.0, -20.0), Vec2::new(20.0, -1.0)).collect();
    assert_eq!(c, vec![(0, 0), (1, 0), (0, 1), (1, 1)]);
    let all: Vec<_> =
        chunks_in(&s, 0, Vec2::new(-100.0, -100.0), Vec2::new(100.0, 100.0)).collect();
    assert_eq!(all.len(), (64 / CHUNK * 40u32.div_ceil(CHUNK)) as usize);
    assert_eq!(
        chunks_in(&s, 0, Vec2::new(70.0, -5.0), Vec2::new(80.0, -1.0)).count(),
        0
    );
}
