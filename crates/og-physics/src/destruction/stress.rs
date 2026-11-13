//! The stress solver of NVIDIA Blast, ported to Rust: for a support graph
//! of nodes (chunks, a static node of mass 0 for the world) joined by
//! bonds, the bond forces that keep every node from moving relative to its
//! neighbours under the accelerations it is given (gravity), found by a
//! conjugate-gradient solve of the normal equations, and from them each
//! bond's compression, tension and shear.
//!
//! Ported from `blast/source/shared/stress_solver/` (`stress.cpp`,
//! `bond.h`, `coupling.h`, `math/cgnr.h`) and the bond stress of
//! `blast/source/sdk/extensions/stress/NvBlastExtStressSolver.cpp`
//! (`calcSolverBondStresses`, `mapStressToRange`) of NVIDIA-Omniverse/PhysX,
//! Copyright (c) 2008-2025 NVIDIA Corporation, BSD-3-Clause
//! (`third_party/blast/LICENSE.md`, `third_party/blast/README.md`).
//!
//! Differences from the original: scalar f32 only (Blast's SISD path; its
//! AVX path computes the same), the mass scale is the arithmetic rather
//! than the geometric mean of the masses (a preconditioning factor; the
//! geometric mean needs `ln` and `exp`, which are not the same bits on
//! every host, while `+ - * /` and `sqrt` are), and inertia is a scalar
//! per node (Blast's `SolverNodeS`). Nothing else changes the arithmetic.

use glam::Vec3;
use serde::{Deserialize, Serialize};

/// Blast's `AngLin6`: an angular and a linear part.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct AngLin6 {
    pub ang: Vec3,
    pub lin: Vec3,
}

impl AngLin6 {
    fn dot(self, o: AngLin6) -> f32 {
        self.ang.dot(o.ang) + self.lin.dot(o.lin)
    }
    fn madd(self, s: f32, o: AngLin6) -> AngLin6 {
        AngLin6 {
            ang: self.ang + o.ang * s,
            lin: self.lin + o.lin * s,
        }
    }
}

/// A node of the support graph: Blast's `SolverNodeS`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SolverNode {
    /// Its centre of mass.
    pub com: Vec3,
    /// 0: static (the world, or an anchored node).
    pub mass: f32,
    /// A scalar moment of inertia (0: static).
    pub inertia: f32,
}

/// A bond between two nodes: Blast's `SolverBond`.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct SolverBond {
    pub nodes: [u32; 2],
    pub centroid: Vec3,
}

#[derive(Clone, Copy, Debug, Default)]
struct Coupling {
    offset0: Vec3,
    node0: u32,
    offset1: Vec3,
    node1: u32,
}

#[derive(Clone, Copy, Debug, Default)]
struct InertiaS {
    i: f32,
    m: f32,
}

/// Blast's `StressProcessor`, prepared once for a graph.
#[derive(Clone, Debug, Default)]
pub struct StressProcessor {
    recip_sqrt_i: Vec<InertiaS>,
    couplings: Vec<Coupling>,
    length_scale: f32,
    mass_scale: f32,
    nodes: usize,
}

impl StressProcessor {
    /// `StressProcessor::prepare` with `centerBonds` and `equalizeMasses`
    /// off.
    pub fn prepare(nodes: &[SolverNode], bonds: &[SolverBond]) -> StressProcessor {
        let mut couplings = Vec::with_capacity(bonds.len());
        let mut offsets_to_scale = 0u32;
        let mut length_scale = 0.0f32;
        for bond in bonds {
            let (b0, b1) = (bond.nodes[0] as usize, bond.nodes[1] as usize);
            let (n0, n1) = (nodes[b0], nodes[b1]);
            let offset0 = if n0.mass > 0.0 {
                bond.centroid - n0.com
            } else {
                n1.com - bond.centroid
            };
            let offset1 = if n1.mass > 0.0 {
                bond.centroid - n1.com
            } else {
                n0.com - bond.centroid
            };
            if n0.mass > 0.0 {
                offsets_to_scale += 1;
                length_scale += offset0.dot(offset0).sqrt();
            }
            if n1.mass > 0.0 {
                offsets_to_scale += 1;
                length_scale += offset1.dot(offset1).sqrt();
            }
            couplings.push(Coupling {
                offset0,
                node0: bond.nodes[0],
                offset1,
                node1: bond.nodes[1],
            });
        }
        let length_scale = if offsets_to_scale > 0 && length_scale > 0.0 {
            length_scale / offsets_to_scale as f32
        } else {
            1.0
        };
        let recip = 1.0 / length_scale;
        for c in &mut couplings {
            c.offset0 *= recip;
            c.offset1 *= recip;
        }
        let (mut sum, mut count) = (0.0f32, 0u32);
        for n in nodes {
            if n.mass > 0.0 {
                sum += n.mass;
                count += 1;
            }
        }
        let mass_scale = if count > 0 { sum / count as f32 } else { 1.0 };
        let inertia_scale = mass_scale * length_scale * length_scale;
        let recip_sqrt_i = nodes
            .iter()
            .map(|n| {
                let i = if n.inertia > 0.0 {
                    inertia_scale / n.inertia
                } else {
                    0.0
                };
                let m = if n.mass > 0.0 {
                    mass_scale / n.mass
                } else {
                    0.0
                };
                InertiaS {
                    i: i.sqrt(),
                    m: m.sqrt(),
                }
            })
            .collect();
        StressProcessor {
            recip_sqrt_i,
            couplings,
            length_scale,
            mass_scale,
            nodes: nodes.len(),
        }
    }

    pub fn bond_count(&self) -> usize {
        self.couplings.len()
    }

    /// y = C x (`CouplingMatrixOps::rmul`), then (I^-1/2) y.
    fn rmul(&self, y: &mut [AngLin6], x: &[AngLin6]) {
        y.fill(AngLin6::default());
        for (c, xj) in self.couplings.iter().zip(x) {
            let (i0, i1) = (c.node0 as usize, c.node1 as usize);
            y[i0].ang += xj.ang - c.offset0.cross(xj.lin);
            y[i0].lin += xj.lin;
            y[i1].ang -= xj.ang - c.offset1.cross(xj.lin);
            y[i1].lin -= xj.lin;
        }
        for (yi, s) in y.iter_mut().zip(&self.recip_sqrt_i) {
            yi.ang *= s.i;
            yi.lin *= s.m;
        }
    }

    /// y = (C^T) (I^-1/2) x (`BondMatrixOps::lmul`).
    fn lmul(&self, y: &mut [AngLin6], x: &[AngLin6], scratch: &mut [AngLin6]) {
        for ((s, xi), r) in scratch.iter_mut().zip(x).zip(&self.recip_sqrt_i) {
            *s = AngLin6 {
                ang: xi.ang * r.i,
                lin: xi.lin * r.m,
            };
        }
        for (yj, c) in y.iter_mut().zip(&self.couplings) {
            let (x0, x1) = (scratch[c.node0 as usize], scratch[c.node1 as usize]);
            *yj = AngLin6 {
                ang: x0.ang - x1.ang,
                lin: x0.lin - x1.lin + c.offset0.cross(x0.ang) - c.offset1.cross(x1.ang),
            };
        }
    }

    /// `StressProcessor::solve`: the bond impulses (forces, when the
    /// velocities are accelerations) that cancel the nodes' `velocities`,
    /// warm-started from `impulses` when `warm`. Answers the iterations
    /// taken, negative when it did not converge within `max_iter` (0: six
    /// times the larger of the node and bond counts).
    pub fn solve(
        &self,
        impulses: &mut [AngLin6],
        velocities: &[AngLin6],
        max_iter: u32,
        tolerance: f32,
        warm: bool,
    ) -> i32 {
        let n_bonds = self.couplings.len();
        let m = self.nodes;
        let recip_length_scale = 1.0 / self.length_scale;
        if warm {
            let recip_lin = recip_length_scale / self.mass_scale;
            let recip_ang = recip_length_scale * recip_lin;
            for j in impulses.iter_mut() {
                j.ang *= recip_ang;
                j.lin *= recip_lin;
            }
        }
        // b = -(I^1/2) v
        let b: Vec<AngLin6> = velocities
            .iter()
            .zip(&self.recip_sqrt_i)
            .map(|(v, s)| AngLin6 {
                ang: v.ang / -(if s.i > 0.0 { s.i } else { 1.0 }),
                lin: v.lin * (-recip_length_scale / if s.m > 0.0 { s.m } else { 1.0 }),
            })
            .collect();
        let max_iter = if max_iter > 0 {
            max_iter
        } else {
            6 * (m.max(n_bonds) as u32)
        };
        let it = self.cgnr(impulses, &b, tolerance, max_iter, warm);
        let lin_scale = self.length_scale * self.mass_scale;
        let ang_scale = self.length_scale * lin_scale;
        for j in impulses.iter_mut() {
            j.ang *= ang_scale;
            j.lin *= lin_scale;
        }
        it
    }

    /// Golub and Van Loan's CGNR (§11.3.9), as `math/cgnr.h`, cold or warm.
    fn cgnr(&self, x: &mut [AngLin6], b: &[AngLin6], tol: f32, max_it: u32, warm: bool) -> i32 {
        let (m, n) = (self.nodes, self.couplings.len());
        let mut z = vec![AngLin6::default(); n];
        let mut p = vec![AngLin6::default(); n];
        let mut r = b.to_vec();
        let mut s = vec![AngLin6::default(); m];
        let mut scratch = vec![AngLin6::default(); m];
        let b_sq: f32 = b.iter().map(|v| v.dot(*v)).sum();
        let delta_sq = tol * tol * b_sq;
        if warm {
            self.rmul(&mut s, x);
            for (ri, si) in r.iter_mut().zip(&s) {
                ri.ang -= si.ang;
                ri.lin -= si.lin;
            }
        } else {
            x.fill(AngLin6::default());
        }
        let mut z_last_sq = 0.0f32;
        let mut started = false;
        let mut it = 0u32;
        loop {
            self.lmul(&mut z, &r, &mut scratch);
            let z_sq: f32 = z.iter().map(|v| v.dot(*v)).sum();
            if z_sq <= delta_sq {
                break;
            }
            if started {
                let beta = z_sq / z_last_sq;
                for (pi, zi) in p.iter_mut().zip(&z) {
                    *pi = zi.madd(beta, *pi);
                }
            } else {
                p.copy_from_slice(&z);
                started = true;
            }
            z_last_sq = z_sq;
            self.rmul(&mut s, &p);
            let s_sq: f32 = s.iter().map(|v| v.dot(*v)).sum();
            if s_sq <= 0.0 {
                break;
            }
            let mu = z_sq / s_sq;
            for (xi, pi) in x.iter_mut().zip(&p) {
                *xi = xi.madd(mu, *pi);
            }
            for (ri, si) in r.iter_mut().zip(&s) {
                *ri = ri.madd(-mu, *si);
            }
            it += 1;
            if it >= max_it {
                break;
            }
        }
        if it < max_it { it as i32 } else { -(it as i32) }
    }
}

/// A bond's stress, in pascals: `normal` along the bond normal (negative
/// is compression), `bend` the bending moment's stress at the joint's
/// edges (on one edge it adds to `normal`, on the other it takes from it)
/// and `shear` across it.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct BondStress {
    pub normal: f32,
    pub bend: f32,
    pub shear: f32,
}

/// A bond's stress from its force (`calcSolverBondStresses`): `normal`
/// along the bond normal, aligned with the displacement from node 0 to
/// node 1, and `shear`, over `area`, with the angular part read as linear
/// stress across `node_dist` (twist as shear, bending as `bend`).
pub fn bond_stress(force: AngLin6, normal: Vec3, area: f32, node_dist: f32) -> BondStress {
    if area <= 0.0 {
        return BondStress::default();
    }
    let n_lin = force.lin.dot(normal);
    let mut shear = (force.lin.length_squared() - n_lin * n_lin).max(0.0).sqrt() / area;
    let n_ang = force.ang.dot(normal).abs();
    let twist = n_ang / area;
    let mut bend = (force.ang.length_squared() - n_ang * n_ang).max(0.0).sqrt() / area;
    if node_dist > 0.0 {
        shear += twist * 2.0 / node_dist;
        bend *= 2.0 / node_dist;
    } else {
        bend = 0.0;
    }
    BondStress {
        normal: n_lin / area,
        bend,
        shear,
    }
}

/// The limits of a material, in pascals (`ExtStressSolverSettings`): below
/// the elastic limit a bond holds; between it and the fatal limit it takes
/// damage in proportion; past the fatal limit it breaks at once.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
pub struct StressLimits {
    pub compression_elastic: f32,
    pub compression_fatal: f32,
    pub tension_elastic: f32,
    pub tension_fatal: f32,
    pub shear_elastic: f32,
    pub shear_fatal: f32,
    /// Mohr–Coulomb friction: the shear a bond holds grows by this much
    /// per pascal of compression across it (tan of the friction angle), so
    /// a joint pressed by what stands on it holds where an unloaded one
    /// slips (masonry's bed joints against its head joints). 0 is none,
    /// Blast's limits as they are.
    pub friction: f32,
}

impl StressLimits {
    /// og.Destructible.strength's limits: `strength` in shear and that times
    /// `tension` (og.Destructible.tension, a fraction) in tension,
    /// thirty times it in compression (masonry's and concrete's ratio at a
    /// game's low strengths: a pier that stands on its weight is not
    /// crushed by it), each fatal at twice its elastic
    /// limit, and `friction` (Mohr–Coulomb); strength 0 none (the solver
    /// breaks nothing).
    pub fn from_strength(strength: f32, friction: f32, tension: f32) -> StressLimits {
        if strength <= 0.0 {
            let inf = f32::INFINITY;
            return StressLimits {
                compression_elastic: inf,
                compression_fatal: inf,
                tension_elastic: inf,
                tension_fatal: inf,
                shear_elastic: inf,
                shear_fatal: inf,
                friction: 0.0,
            };
        }
        StressLimits {
            compression_elastic: strength * 30.0,
            compression_fatal: strength * 60.0,
            tension_elastic: strength * tension,
            tension_fatal: strength * tension * 2.0,
            shear_elastic: strength,
            shear_fatal: strength * 2.0,
            friction: friction.max(0.0),
        }
    }

    /// The share of a bond's health a stress takes in one evaluation
    /// (`ExtStressSolverImpl::generateFractureCommand`'s multiplier).
    ///
    /// Blast adds the bending to the normal stress in its own sign, so a
    /// joint in compression only ever sees more compression; here each edge
    /// is judged, `normal` plus and minus `bend`: a compressed joint whose
    /// load lies far enough off its middle (a block resting on the corner of
    /// the one below, its weight past the edge) pulls apart at its far edge,
    /// the eccentricity check of masonry limit analysis, and a joint that
    /// holds no tension lets the block tip off. Friction holds by the
    /// compression of the whole joint.
    pub fn damage(&self, s: BondStress) -> f32 {
        let excess = |s: f32, e: f32, f: f32| {
            if s > e {
                let d = f - e;
                (s - e) / if d > 0.0 { d } else { 1.0 }
            } else {
                0.0
            }
        };
        let compression = (s.bend - s.normal).max(0.0);
        let tension = (s.normal + s.bend).max(0.0);
        let mut m = 0.0;
        if compression > self.compression_elastic {
            m += excess(
                compression,
                self.compression_elastic,
                self.compression_fatal,
            );
        }
        if tension > self.tension_elastic {
            m += excess(tension, self.tension_elastic, self.tension_fatal);
        }
        // Compression across the bond holds it against shear (Coulomb).
        let held = self.friction * (-s.normal).max(0.0);
        m + excess(s.shear, self.shear_elastic + held, self.shear_fatal + held)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_block_on_the_ground_is_held_in_compression_by_its_weight() {
        // Node 0 the world, node 1 a 10 kg block above it.
        let nodes = [
            SolverNode::default(),
            SolverNode {
                com: Vec3::new(0.0, 0.5, 0.0),
                mass: 10.0,
                inertia: 10.0 / 6.0,
            },
        ];
        let bonds = [SolverBond {
            nodes: [0, 1],
            centroid: Vec3::ZERO,
        }];
        let sp = StressProcessor::prepare(&nodes, &bonds);
        let g = Vec3::new(0.0, -9.81, 0.0);
        let v = [
            AngLin6::default(),
            AngLin6 {
                ang: Vec3::ZERO,
                lin: g,
            },
        ];
        let mut j = [AngLin6::default()];
        assert!(sp.solve(&mut j, &v, 0, 1e-4, false) >= 0);
        // The bond pushes node 1 up with its weight: J applied to node 1
        // with the minus sign of rmul, so J.lin is -m g's opposite.
        let force = j[0];
        let BondStress { normal, shear, .. } = bond_stress(force, Vec3::Y, 1.0, 0.5);
        assert!((normal.abs() - 98.1).abs() < 0.1, "{force:?} {normal}");
        assert!(shear.abs() < 1e-2);
        assert!(normal < 0.0, "compression: {normal}");
    }

    #[test]
    fn a_cantilever_bond_carries_its_weight_in_shear_and_bending() {
        // A 10 kg arm whose centre is 1 m beside the bond to the world.
        let nodes = [
            SolverNode::default(),
            SolverNode {
                com: Vec3::new(1.0, 0.0, 0.0),
                mass: 10.0,
                inertia: 10.0 / 6.0,
            },
        ];
        let bonds = [SolverBond {
            nodes: [0, 1],
            centroid: Vec3::ZERO,
        }];
        let sp = StressProcessor::prepare(&nodes, &bonds);
        let v = [
            AngLin6::default(),
            AngLin6 {
                ang: Vec3::ZERO,
                lin: Vec3::new(0.0, -9.81, 0.0),
            },
        ];
        let mut j = [AngLin6::default()];
        sp.solve(&mut j, &v, 0, 1e-5, false);
        // Shear 98.1 N across the x-normal bond, torque 98.1 N·m about z.
        assert!((j[0].lin.y.abs() - 98.1).abs() < 0.1, "{:?}", j[0]);
        assert!((j[0].ang.z.abs() - 98.1).abs() < 0.5, "{:?}", j[0]);
        let BondStress { bend, shear, .. } = bond_stress(j[0], Vec3::X, 0.1, 1.0);
        assert!(shear > 900.0 && bend > 1900.0, "{bend} {shear}");
    }

    #[test]
    fn limits_map_stress_to_damage() {
        let l = StressLimits {
            compression_elastic: 100.0,
            compression_fatal: 200.0,
            tension_elastic: 10.0,
            tension_fatal: 20.0,
            shear_elastic: 10.0,
            shear_fatal: 30.0,
            friction: 0.0,
        };
        assert_eq!(l.damage(st(-50.0, 0.0, 0.0)), 0.0);
        assert_eq!(l.damage(st(-150.0, 0.0, 0.0)), 0.5);
        assert_eq!(l.damage(st(15.0, 0.0, 20.0)), 0.5 + 0.5);
        // Friction: the same shear slips an unloaded joint and is held by
        // one pressed at 40 Pa (0.5 × 40 = 20 more it holds).
        let f = StressLimits { friction: 0.5, ..l };
        assert_eq!(f.damage(st(0.0, 0.0, 20.0)), 0.5);
        assert_eq!(f.damage(st(-40.0, 0.0, 20.0)), 0.0);
        assert_eq!(f.damage(st(-40.0, 0.0, 40.0)), 0.5);
        // Tension takes nothing from it.
        assert_eq!(f.damage(st(5.0, 0.0, 20.0)), 0.5);
        // A pressed joint bent past its compression pulls apart at one
        // edge (50 − 40 = 10 Pa, at the elastic limit: no damage; 55 − 40
        // = 15, half way to fatal), and crushes at the other only past the
        // compression limit (40 + 55 = 95 < 100).
        assert_eq!(l.damage(st(-40.0, 50.0, 0.0)), 0.0);
        assert_eq!(l.damage(st(-40.0, 55.0, 0.0)), 0.5);
        assert_eq!(l.damage(st(-60.0, 55.0, 0.0)), 0.15);
    }

    fn st(normal: f32, bend: f32, shear: f32) -> BondStress {
        BondStress {
            normal,
            bend,
            shear,
        }
    }
}
