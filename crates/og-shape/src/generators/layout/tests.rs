use glam::DVec2;

use super::geom::{Poly, area2, edge};
use super::{GridDesc, Layout, LayoutDesc, LotsDesc, PropDesc, Region, StreetDesc, layout};

fn v(x: f64, y: f64) -> DVec2 {
    DVec2::new(x, y)
}

fn town(seed: u64) -> LayoutDesc {
    LayoutDesc {
        seed,
        region: Region::Area(vec![
            v(0.0, 0.0),
            v(240.0, 0.0),
            v(240.0, 160.0),
            v(0.0, 160.0),
        ]),
        grid: Some(GridDesc {
            size: v(80.0, 60.0),
            turn: 12.0,
            jitter: 0.15,
            width: 9.0,
            sidewalk: 2.5,
        }),
        streets: vec![StreetDesc {
            name: "main".into(),
            points: vec![v(-10.0, 83.0), v(250.0, 77.0)],
            width: 14.0,
            sidewalk: 4.0,
        }],
        lots: LotsDesc {
            width: [6.0, 14.0],
            depth: [12.0, 30.0],
        },
        props: vec![
            PropDesc {
                name: "lamp".into(),
                every: 18.0,
                jitter: 2.0,
                kerb: 0.5,
                corners: true,
                street: None,
            },
            PropDesc {
                name: "stop".into(),
                every: 60.0,
                jitter: 0.0,
                kerb: 1.0,
                corners: false,
                street: Some("main".into()),
            },
        ],
    }
}

/// The area two convex polygons share.
fn shared(a: &[DVec2], b: &[DVec2]) -> f64 {
    let mut p = Poly::new(a.to_vec(), None);
    for i in 0..b.len() {
        let (_, n) = edge(b, i);
        p = p.clip(-n, -n.dot(b[i]), None);
        if p.len() < 3 {
            return 0.0;
        }
    }
    area2(&p.points) / 2.0
}

/// The distance from `p` to a street's centre line.
fn to_line(p: DVec2, pts: &[DVec2]) -> f64 {
    pts.windows(2)
        .map(|w| {
            let (a, b) = (w[0], w[1]);
            let t = ((p - a).dot(b - a) / (b - a).length_squared()).clamp(0.0, 1.0);
            p.distance(a + (b - a) * t)
        })
        .fold(f64::INFINITY, f64::min)
}

fn inside(p: DVec2, poly: &[DVec2]) -> bool {
    (0..poly.len()).all(|i| {
        let (_, n) = edge(poly, i);
        (p - poly[i]).dot(n) >= -1e-6
    })
}

#[test]
fn lots_lie_in_their_block_apart_and_each_fronts_its_street() {
    let l = layout(&town(7)).unwrap();
    assert!(l.blocks.len() >= 6, "{} blocks", l.blocks.len());
    assert!(l.lots.len() > 40, "{} lots", l.lots.len());
    for (i, lot) in l.lots.iter().enumerate() {
        assert!(area2(&lot.footprint) > 0.0, "lot {i} is counter-clockwise");
        let block = &l.blocks[lot.block].poly.points;
        let a = area2(&lot.footprint) / 2.0;
        assert!(
            (shared(&lot.footprint, block) - a).abs() < 1e-6 * a.max(1.0),
            "lot {i} leaves its block"
        );
        // Its front lies on its street's corridor edge: half the roadway
        // and the pavement from the centre line.
        let s = &l.streets[lot.street];
        for p in &lot.footprint[0..2] {
            let d = to_line(*p, &s.points);
            assert!(
                (d - (s.width / 2.0 + s.sidewalk)).abs() < 1e-6,
                "lot {i}'s front is {d} m from its street"
            );
        }
        // No deeper than its block's ring.
        let (_, n) = edge(&lot.footprint, 0);
        for p in &lot.footprint {
            let d = (*p - lot.footprint[0]).dot(n);
            assert!(
                d <= lot.depth + 1e-6,
                "lot {i} reaches {d} m back, its ring {}",
                lot.depth
            );
        }
        for (j, other) in l.lots.iter().enumerate().skip(i + 1) {
            assert!(
                shared(&lot.footprint, &other.footprint) < 1e-6,
                "lots {i} and {j} overlap"
            );
        }
    }
    // The lots tile each block's ring: inside each block, the lots and
    // the yard behind them make up the block.
    let lots: f64 = l.lots.iter().map(|x| area2(&x.footprint) / 2.0).sum();
    let yards: f64 = l.yards.iter().map(|x| area2(&x.footprint) / 2.0).sum();
    let blocks: f64 = l.blocks.iter().map(|b| b.poly.area()).sum();
    assert!(
        lots > 0.4 * blocks && lots <= blocks + 1e-6,
        "{lots} of {blocks}"
    );
    assert!((lots + yards - blocks).abs() < 1e-6 * blocks);
    assert!(!l.yards.is_empty());
    for yard in &l.yards {
        assert!(
            yard.footprint
                .iter()
                .all(|p| inside(*p, &l.blocks[yard.block].poly.points))
        );
        for lot in l.lots.iter().filter(|l| l.block == yard.block) {
            assert!(shared(&yard.footprint, &lot.footprint) < 1e-6);
        }
    }
}

#[test]
fn no_lot_or_block_crosses_a_street() {
    let l = layout(&town(3)).unwrap();
    // A block's every point stands at least its corridor from each street.
    for (b, block) in l.blocks.iter().enumerate() {
        for s in &l.streets {
            for p in &block.poly.points {
                let d = to_line(*p, &s.points);
                assert!(
                    d >= s.width / 2.0 + s.sidewalk - 1e-6,
                    "block {b} reaches {d} m from a street"
                );
            }
        }
    }
    // The main street cuts the grid's middle blocks: blocks lie on both
    // sides of it.
    let main = &l.streets[0];
    let side = |p: DVec2| (main.points[1] - main.points[0]).perp_dot(p - main.points[0]) > 0.0;
    let ups = l.blocks.iter().filter(|b| side(b.poly.points[0])).count();
    assert!(ups > 0 && ups < l.blocks.len());
}

#[test]
fn lot_widths_and_depths_keep_within_their_range() {
    let d = town(11);
    let l = layout(&d).unwrap();
    let wide = l
        .lots
        .iter()
        .filter(|x| x.frontage > d.lots.width[1] + 1e-9)
        .count();
    let narrow = l
        .lots
        .iter()
        .filter(|x| x.frontage < d.lots.width[0] - 1e-9)
        .count();
    // Only a frontage shorter than one lot, or between the greatest and
    // two of the least, cannot be cut within the range.
    assert!(
        wide + narrow <= l.lots.len() / 10,
        "{wide} wide and {narrow} narrow of {}",
        l.lots.len()
    );
    assert!(
        l.lots
            .iter()
            .all(|x| (d.lots.depth[0]..=d.lots.depth[1]).contains(&x.depth))
    );
}

#[test]
fn a_seed_is_a_plan_and_each_rule_moves_only_what_it_decides() {
    let a = layout(&town(5)).unwrap();
    assert_eq!(a, layout(&town(5)).unwrap());
    let other = layout(&town(6)).unwrap();
    assert_ne!(a.lots, other.lots);
    // Other props: the same lots, to the bit.
    let mut d = town(5);
    d.props[0].every = 25.0;
    d.props[0].jitter = 5.0;
    let moved = layout(&d).unwrap();
    assert_eq!(a.lots, moved.lots);
    assert_ne!(a.sites, moved.sites);
    // Another lot width range leaves the blocks where they were.
    let mut d = town(5);
    d.lots.width = [8.0, 10.0];
    assert_eq!(a.blocks, layout(&d).unwrap().blocks);
}

fn lots_of(l: &Layout) -> usize {
    l.lots.len()
}

#[test]
fn props_stand_on_the_pavement_at_their_spacing() {
    let d = town(9);
    let l = layout(&d).unwrap();
    let lamps: Vec<_> = l.sites.iter().filter(|s| s.prop == 0).collect();
    assert!(lamps.iter().any(|s| s.corner) && lamps.iter().any(|s| !s.corner));
    for s in &lamps {
        let st = &l.streets[s.street];
        // On the pavement, `kerb` in from the roadway's edge.
        let d = to_line(s.position, &st.points);
        if !s.corner {
            assert!(
                (d - (st.width / 2.0 + 0.5)).abs() < 1e-6,
                "a lamp {d} m from its street"
            );
        }
        // Never on a lot or a block.
        assert!(
            l.blocks.iter().all(|b| !inside(s.position, &b.poly.points)),
            "a lamp on a block"
        );
        // Facing the roadway.
        let near = s.position + s.facing * 0.1;
        assert!(to_line(near, &st.points) < to_line(s.position, &st.points));
    }
    // Neighbours along one block's kerb stand `every` apart, give or take
    // twice the jitter.
    let mut by_edge: std::collections::BTreeMap<(usize, usize, i64, i64), Vec<DVec2>> =
        Default::default();
    for s in lamps.iter().filter(|s| !s.corner) {
        let k = (
            (s.facing.x * 1e6).round() as i64,
            (s.facing.y * 1e6).round() as i64,
        );
        by_edge
            .entry((s.block, s.street, k.0, k.1))
            .or_default()
            .push(s.position);
    }
    let mut checked = 0;
    for pts in by_edge.values_mut() {
        let dir = pts.len() > 1;
        if !dir {
            continue;
        }
        let e = (pts[pts.len() - 1] - pts[0]).normalize();
        pts.sort_by(|a, b| a.dot(e).total_cmp(&b.dot(e)));
        for w in pts.windows(2) {
            let gap = w[0].distance(w[1]);
            assert!((gap - 18.0).abs() <= 4.0 + 1e-6, "lamps {gap} m apart");
            checked += 1;
        }
    }
    assert!(checked > 10);
    // A prop limited to one street stands only along it.
    let stops: Vec<_> = l.sites.iter().filter(|s| s.prop == 1).collect();
    assert!(!stops.is_empty() && stops.iter().all(|s| l.streets[s.street].name == "main"));
    let mut d = town(9);
    d.props[1].street = Some("mian".into());
    let e = layout(&d).unwrap_err();
    assert!(e.contains("mian") && e.contains("main"), "{e}");
}

#[test]
fn a_path_is_lined_on_both_sides() {
    let street = StreetDesc {
        name: "high".into(),
        points: vec![v(0.0, 0.0), v(100.0, 0.0), v(160.0, 50.0)],
        width: 10.0,
        sidewalk: 3.0,
    };
    let d = LayoutDesc {
        seed: 1,
        region: Region::Along {
            street: street.clone(),
            depth: 25.0,
        },
        grid: None,
        streets: Vec::new(),
        lots: LotsDesc {
            width: [6.0, 12.0],
            depth: [10.0, 20.0],
        },
        props: Vec::new(),
    };
    let l = layout(&d).unwrap();
    assert_eq!(l.blocks.len(), 4);
    let side = |p: DVec2| {
        let s = &street.points;
        let i = usize::from(p.x > 100.0);
        (s[i + 1] - s[i]).perp_dot(p - s[i]) > 0.0
    };
    let left = l.lots.iter().filter(|x| side(x.footprint[0])).count();
    assert!(
        left > 5 && lots_of(&l) - left > 5,
        "{left} of {}",
        lots_of(&l)
    );
    for x in &l.lots {
        // Its front's middle is on the corridor's edge (its corners, at
        // the outside of the bend, stand off the corner itself).
        let mid = (x.footprint[0] + x.footprint[1]) / 2.0;
        assert!((to_line(mid, &street.points) - 8.0).abs() < 1e-6);
        for (j, y) in l.lots.iter().enumerate() {
            if !std::ptr::eq(x, y) {
                assert!(
                    shared(&x.footprint, &y.footprint) < 1e-6,
                    "lot overlaps lot {j}"
                );
            }
        }
    }
}

#[test]
fn a_street_added_at_one_end_leaves_the_other_end_as_it_was() {
    let cross = |name: &str, x: f64| StreetDesc {
        name: name.into(),
        points: vec![v(x, -10.0), v(x, 170.0)],
        width: 10.0,
        sidewalk: 3.0,
    };
    let mut d = LayoutDesc {
        seed: 2,
        region: Region::Area(vec![
            v(0.0, 0.0),
            v(260.0, 0.0),
            v(260.0, 160.0),
            v(0.0, 160.0),
        ]),
        grid: None,
        streets: vec![cross("a", 70.0)],
        lots: LotsDesc {
            width: [6.0, 14.0],
            depth: [12.0, 30.0],
        },
        props: Vec::new(),
    };
    // Lots made before the far side's (the west) and after it (the east)
    // both stay: no draw depends on how many lots came before.
    let side = |l: &Layout, east: bool| -> Vec<Vec<DVec2>> {
        let mut v: Vec<_> = l
            .lots
            .iter()
            .filter(|x| {
                x.footprint
                    .iter()
                    .all(|p| if east { p.x > 130.0 } else { p.x < 130.0 })
            })
            .map(|x| x.footprint.clone())
            .collect();
        v.sort_by(|a, b| a[0].x.total_cmp(&b[0].x).then(a[0].y.total_cmp(&b[0].y)));
        v
    };
    d.streets[0] = cross("a", 130.0);
    let before = layout(&d).unwrap();
    for x in [40.0, 220.0] {
        let mut more = d.clone();
        more.streets.push(cross("b", x));
        let after = layout(&more).unwrap();
        let far = x < 130.0;
        assert!(!side(&before, far).is_empty());
        assert_eq!(
            side(&before, far),
            side(&after, far),
            "a street at {x} moved the lots across the town"
        );
        assert_ne!(before.lots.len(), after.lots.len());
    }
}
