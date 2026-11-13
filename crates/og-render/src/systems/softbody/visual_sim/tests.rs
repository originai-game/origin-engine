use super::*;

#[test]
fn a_frame_steps_the_ticks_since_the_last_and_starts_again_on_a_jump_or_going_back() {
    assert_eq!(
        plan(None, 100),
        Plan {
            reset: true,
            steps: PREROLL,
            first: 40
        }
    );
    assert_eq!(
        plan(Some(100), 100),
        Plan {
            reset: false,
            steps: 0,
            first: 101
        }
    );
    assert_eq!(
        plan(Some(100), 103),
        Plan {
            reset: false,
            steps: 3,
            first: 101
        }
    );
    assert_eq!(plan(Some(100), 108).steps, 8);
    assert!(plan(Some(100), 109).reset, "a jump past the catch-up");
    assert!(plan(Some(100), 99).reset, "a world that went back");
}

fn grid(columns: u32, rows: u32, spacing: f32) -> Vec<Vec3> {
    (0..rows)
        .flat_map(|y| (0..columns).map(move |x| Vec3::new(x as f32, -(y as f32), 0.0) * spacing))
        .collect()
}

#[test]
fn colours_never_share_a_point_and_cover_every_edge() {
    let rest = grid(7, 5, 0.2);
    let cloth = Some(SimulationSurface::Cloth {
        columns: 7,
        rows: 5,
    });
    let body = VisualBody::default();
    let e = edges(cloth, &body, &rest, &[]);
    // Structural 6*5 + 7*4, shear 2*6*4, bending 5*5 + 7*3.
    assert_eq!(e.len(), 30 + 28 + 48 + 25 + 21);
    let (coloured, table) = colour(&e, rest.len());
    assert_eq!(coloured.len(), e.len());
    for &(first, count) in &table {
        let mut seen = std::collections::BTreeSet::new();
        for e in &coloured[first as usize..(first + count) as usize] {
            assert!(
                seen.insert(e.a) && seen.insert(e.b),
                "colour shares a point"
            );
        }
    }
    assert!(table.len() <= 16, "{} colours", table.len());
    let diag = e.iter().find(|e| e.a == 0 && e.b == 8).unwrap();
    assert!((diag.rest - 0.2 * std::f32::consts::SQRT_2).abs() < 1e-6);
}

#[test]
fn pinned_points_weigh_nothing_and_the_rest_are_tethered_to_the_nearest_pin() {
    let rest = grid(4, 3, 0.5);
    let cloth = Some(SimulationSurface::Cloth {
        columns: 4,
        rows: 3,
    });
    let body = VisualBody {
        pinned: Arc::from([0u32, 3]),
        ..VisualBody::default()
    };
    let t = Topology::build(cloth, &body, &rest, &[]);
    let w = &t.words;
    let inv = w[h::INV_MASS] as usize;
    let m: Vec<f32> = (0..12).map(|i| f32::from_bits(w[inv + i])).collect();
    assert_eq!(m[0], 0.0);
    assert_eq!(m[3], 0.0);
    assert!(m[1..3].iter().chain(&m[4..]).all(|&x| x == 1.0));
    let tether = w[h::TETHER] as usize;
    // Point 9 (x 1, y 2) is nearest pin 0: sqrt(0.5² + 1²).
    assert_eq!(w[tether + 9 * 2], 0);
    assert!((f32::from_bits(w[tether + 9 * 2 + 1]) - 1.25f32.sqrt()).abs() < 1e-6);
    // Point 10 (x 2) is nearest pin 3.
    assert_eq!(w[tether + 10 * 2], 3);
    assert_eq!(w[tether], u32::MAX, "a pin has no tether");
    assert!(t.matches(cloth, &body, &rest, &[]));
    let moved: Vec<Vec3> = rest
        .iter()
        .map(|p| *p + Vec3::new(3.0, 1.0, -2.0))
        .collect();
    assert!(
        t.matches(cloth, &body, &moved, &[]),
        "a moved rest shape keeps it"
    );
    let wider = grid(4, 3, 0.6);
    assert!(
        !t.matches(cloth, &body, &wider, &[]),
        "new rest lengths rebuild it"
    );
}
