//! Which way a mesh is up and faces: the 24 axis turns and the measures that choose one.

use super::*;

/// One of the six axis directions.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Axis {
    /// +X.
    PosX,
    /// −X.
    NegX,
    /// +Y.
    PosY,
    /// −Y.
    NegY,
    /// +Z.
    PosZ,
    /// −Z.
    NegZ,
}

impl Axis {
    /// All six, in the order `+x -x +y -y +z -z`.
    pub const ALL: [Axis; 6] = [
        Axis::PosX,
        Axis::NegX,
        Axis::PosY,
        Axis::NegY,
        Axis::PosZ,
        Axis::NegZ,
    ];

    /// Reads `+x`, `-x`, `x` (= `+x`), … in either case.
    pub fn parse(s: &str) -> Option<Axis> {
        let s = s.trim().to_ascii_lowercase();
        let (neg, name) = match s.as_bytes() {
            [b'+', n] => (false, *n),
            [b'-', n] => (true, *n),
            [n] => (false, *n),
            _ => return None,
        };
        let i = match name {
            b'x' => 0,
            b'y' => 1,
            b'z' => 2,
            _ => return None,
        };
        Some(Axis::of(i, neg))
    }

    /// `+x`, `-x`, … as [`Axis::parse`] reads it.
    pub fn name(self) -> &'static str {
        ["+x", "-x", "+y", "-y", "+z", "-z"][self as usize]
    }

    /// The coordinate it runs along: 0, 1 or 2.
    pub fn index(self) -> usize {
        self as usize / 2
    }

    /// Whether it points down its coordinate.
    pub fn negative(self) -> bool {
        self as usize % 2 == 1
    }

    /// The opposite direction.
    pub fn opposite(self) -> Axis {
        Axis::of(self.index(), !self.negative())
    }

    #[doc(hidden)]
    pub fn of(index: usize, negative: bool) -> Axis {
        Axis::ALL[index * 2 + usize::from(negative)]
    }

    /// The unit vector.
    pub fn vector(self) -> DVec3 {
        let mut v = [0.0; 3];
        v[self.index()] = if self.negative() { -1.0 } else { 1.0 };
        DVec3::from_array(v)
    }

    /// `self × other` when both are axes at right angles.
    pub fn cross(self, other: Axis) -> Option<Axis> {
        let (a, b) = (self.index(), other.index());
        if a == b {
            return None;
        }
        let c = 3 - a - b;
        // e_a × e_b = +e_c when (a, b, c) is a cyclic order.
        let cyclic = (a + 1) % 3 == b;
        Some(Axis::of(c, cyclic == (self.negative() != other.negative())))
    }
}

impl fmt::Display for Axis {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.name())
    }
}

/// An axis-aligned turn: output coordinate `k` is input coordinate
/// `from[k]`, negated where `neg[k]`. Exact: it moves and negates
/// numbers, and rounds none.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct AxisTurn {
    pub(crate) from: [usize; 3],
    pub(crate) neg: [bool; 3],
}

impl AxisTurn {
    #[doc(hidden)]
    pub const IDENTITY: AxisTurn = AxisTurn {
        from: [0, 1, 2],
        neg: [false; 3],
    };

    /// The turn that takes input `up` to +Z and input `front` to −Y.
    /// Either may be left out: with only `up`, the smallest turn that
    /// stands `up` up; with only `front`, the smallest that brings it to
    /// −Y; with neither, no turn.
    pub fn new(up: Option<Axis>, front: Option<Axis>) -> Result<AxisTurn, String> {
        use Axis::*;
        let (up, front) = match (up, front) {
            (None, None) => return Ok(AxisTurn::IDENTITY),
            (Some(u), Some(f)) => (u, f),
            (Some(u), None) => (
                u,
                match u {
                    PosX | NegX | PosZ => NegY,
                    NegZ => PosY,
                    PosY => PosZ,
                    NegY => NegZ,
                },
            ),
            (None, Some(f)) => (
                match f {
                    PosX | NegX | PosY | NegY => PosZ,
                    PosZ => PosY,
                    NegZ => NegY,
                },
                f,
            ),
        };
        let Some(side) = up.cross(front) else {
            return Err(format!(
                "up {up} and front {front} lie along one axis; give two axes at right angles"
            ));
        };
        // The images: up → +Z, front → −Y, up × front → +X (Z × −Y = X),
        // so the turn keeps handedness.
        let mut t = AxisTurn::IDENTITY;
        for (input, output) in [(side, PosX), (front, NegY), (up, PosZ)] {
            t.from[output.index()] = input.index();
            t.neg[output.index()] = input.negative() != output.negative();
        }
        Ok(t)
    }

    /// The turn applied to a point or a direction.
    pub fn apply(&self, v: DVec3) -> DVec3 {
        let a = v.to_array();
        let c = |k: usize| {
            if self.neg[k] {
                -a[self.from[k]]
            } else {
                a[self.from[k]]
            }
        };
        DVec3::new(c(0), c(1), c(2))
    }

    /// Where input axis `a` goes.
    pub fn image(&self, a: Axis) -> Axis {
        let k = (0..3).find(|&k| self.from[k] == a.index()).unwrap_or(0);
        Axis::of(k, a.negative() != self.neg[k])
    }

    /// `self ∘ other`: `other` first.
    pub fn then(&self, other: &AxisTurn) -> AxisTurn {
        let mut t = AxisTurn::IDENTITY;
        for k in 0..3 {
            t.from[k] = other.from[self.from[k]];
            t.neg[k] = self.neg[k] != other.neg[self.from[k]];
        }
        t
    }

    /// The turn applied after an affine map (to its columns and its
    /// translation): `self ∘ a`, exact.
    pub fn after(&self, a: &Affine) -> Affine {
        Affine {
            x: self.apply(a.x),
            y: self.apply(a.y),
            z: self.apply(a.z),
            t: self.apply(a.t),
        }
    }

    /// The turn back.
    pub fn inverse(&self) -> AxisTurn {
        let mut t = AxisTurn::IDENTITY;
        for k in 0..3 {
            t.from[self.from[k]] = k;
            t.neg[self.from[k]] = self.neg[k];
        }
        t
    }

    /// The quarter turns about Z, counter-clockwise: 0 to 3.
    pub fn quarter(n: u32) -> AxisTurn {
        // (x, y) ↦ (−y, x) once.
        let once = AxisTurn {
            from: [1, 0, 2],
            neg: [true, false, false],
        };
        (0..n % 4).fold(AxisTurn::IDENTITY, |t, _| once.then(&t))
    }
}

/// The input axis an axis turn sends to `output`.
pub fn axes_input(t: &AxisTurn, output: Axis) -> Axis {
    Axis::ALL
        .into_iter()
        .find(|&a| t.image(a) == output)
        .unwrap_or(output)
}

/// The numbers that tell which way a mesh stands, with no verdict
/// (`guides/meshtools.md`): its box, where its origin sits, which way
/// its surface faces, and where its mass sits in the box.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct AxisMeasures {
    /// The box's extents along X, Y and Z, mm.
    pub size: DVec3,
    /// The origin less the centre of the box's bottom face (lowest Z), mm
    /// (as `MeshMeasures::pivot_from_base`).
    pub pivot_from_base: DVec3,
    /// The axis the box is longest along: 0, 1 or 2 (the first of equals).
    pub longest: usize,
    /// The axis it is shortest along.
    pub shortest: usize,
    /// The fraction of the surface's area facing each way, in the order
    /// of [`Axis::ALL`] (`+x -x +y -y +z -z`): each triangle counts for
    /// the axis its normal leans along most (the first of equals). They
    /// sum to 1 (0 for a mesh with no area).
    pub facing: [f64; 6],
    /// Where the mass sits in the box, per axis: 0 at the low side, 1 at
    /// the high side, 0.5 in the middle.
    pub mass: DVec3,
    /// Whether [`AxisMeasures::mass`] is the centre of volume (every part
    /// closed, with a volume above 0) or of area.
    pub mass_of_volume: bool,
}

/// [`AxisMeasures`] of `parts` taken together.
pub fn axis_measures(parts: &[Solid]) -> AxisMeasures {
    let mut m = AxisMeasures::default();
    let Some((lo, hi)) = bounds(parts, |p| p) else {
        return m;
    };
    m.size = hi - lo;
    let mid = (lo + hi) / 2.0;
    m.pivot_from_base = DVec3::ZERO - DVec3::new(mid.x, mid.y, lo.z);
    let s = m.size.to_array();
    for k in 1..3 {
        if s[k] > s[m.longest] {
            m.longest = k;
        }
        if s[k] < s[m.shortest] {
            m.shortest = k;
        }
    }
    let (mut area, mut area_moment) = (0.0, DVec3::ZERO);
    let (mut volume, mut volume_moment) = (0.0, DVec3::ZERO);
    let mut closed = true;
    for part in parts {
        let mut part_volume = 0.0;
        let mut part_moment = DVec3::ZERO;
        for t in &part.triangles {
            // About the box's centre, for fewer bits lost.
            let [a, b, c] = t.map(|v| part.positions[v as usize] - mid);
            let n = (b - a).cross(c - a);
            let twice = n.length();
            area += twice / 2.0;
            area_moment += (a + b + c) * (twice / 6.0);
            let w = n.abs().to_array();
            let k = if w[0] >= w[1] && w[0] >= w[2] {
                0
            } else if w[1] >= w[2] {
                1
            } else {
                2
            };
            let neg = n.to_array()[k] < 0.0;
            m.facing[2 * k + usize::from(neg)] += twice / 2.0;
            let v = a.dot(b.cross(c)) / 6.0;
            part_volume += v;
            part_moment += (a + b + c) * (v / 4.0);
        }
        closed &= part_volume > 0.0 && is_closed(part);
        volume += part_volume;
        volume_moment += part_moment;
    }
    if area > 0.0 {
        for f in &mut m.facing {
            *f /= area;
        }
    }
    let centre = if closed && volume > 0.0 {
        m.mass_of_volume = true;
        volume_moment / volume
    } else if area > 0.0 {
        area_moment / area
    } else {
        DVec3::ZERO
    } + mid;
    let rel = |k: usize| {
        let (l, h, c) = (lo.to_array()[k], hi.to_array()[k], centre.to_array()[k]);
        if h > l { (c - l) / (h - l) } else { 0.5 }
    };
    m.mass = DVec3::new(rel(0), rel(1), rel(2));
    m
}

/// Every edge (positions welded exactly) has two triangles running it
/// opposite ways.
pub fn is_closed(s: &Solid) -> bool {
    let (_, tris) = weld_exact(s);
    if tris.is_empty() {
        return false;
    }
    let mut edges: BTreeMap<(u32, u32), (u32, u32)> = BTreeMap::new();
    for t in &tris {
        for k in 0..3 {
            let (a, b) = (t[k], t[(k + 1) % 3]);
            if a == b {
                continue;
            }
            let e = edges.entry(edge_key(a, b)).or_default();
            if a < b {
                e.0 += 1;
            } else {
                e.1 += 1;
            }
        }
    }
    edges.values().all(|&n| n == (1, 1))
}

#[doc(hidden)]
pub fn vec3(v: DVec3) -> Json {
    Json::Arr(v.to_array().map(num).to_vec())
}

#[doc(hidden)]
pub fn num(v: f64) -> Json {
    if v.is_finite() {
        Json::Num(v)
    } else {
        Json::Null
    }
}

#[doc(hidden)]
pub fn axis_name(k: usize) -> Json {
    Json::Str(["x", "y", "z"][k].into())
}

impl AxisMeasures {
    #[doc(hidden)]
    pub fn json(&self) -> Json {
        Json::Obj(vec![
            ("size", vec3(self.size)),
            ("pivot_from_base", vec3(self.pivot_from_base)),
            ("longest", axis_name(self.longest)),
            ("shortest", axis_name(self.shortest)),
            (
                "facing",
                Json::Obj(
                    Axis::ALL
                        .iter()
                        .zip(self.facing)
                        .map(|(a, f)| (a.name(), num(f)))
                        .collect(),
                ),
            ),
            (
                "mass",
                Json::Obj(vec![
                    (
                        "of",
                        Json::Str(
                            if self.mass_of_volume {
                                "volume"
                            } else {
                                "area"
                            }
                            .into(),
                        ),
                    ),
                    ("in_box", vec3(self.mass)),
                ]),
            ),
        ])
    }

    /// The measures as one line of JSON, lengths in mm: `size`,
    /// `pivot_from_base`, `longest`, `shortest`, `facing` (`+x` … `-z`)
    /// and `mass` (`of`: `volume` or `area`; `in_box`: 0 to 1 per axis).
    pub fn to_json(&self) -> String {
        let mut out = String::new();
        self.json().write(&mut out);
        out
    }
}
