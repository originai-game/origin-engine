//! Eigendecomposition for symmetric 3x3 matrices.
//!
//! Origin Engine's change to glamx 0.3.1 (the root Cargo.toml's `[patch.crates-io]`,
//! `third_party/glamx/Cargo.toml`): the decomposition is a cyclic Jacobi iteration in
//! `f64`, built from `+ - * /`, comparisons and `sqrt` only, so its result is the same
//! bits on every host. glamx's own solver (Eberly's "A Robust Eigensolver for 3 x 3
//! Symmetric Matrices", the analytic eigenvalues through `acos` and `cos`) called the
//! host's libm, and parry's `MassProperties` (`with_inertia_matrix`, which every sum of
//! two mass properties ends in) gave a compound body a principal inertia frame that
//! differed by an ulp between native and wasm, so the two hosts' worlds drifted apart.
//! The eigenvector helpers below are the upstream ones, kept for the public API.

#![allow(clippy::manual_swap)] // Needed for spirv compatibility.

use glam::Vec3Swizzles;

#[cfg(not(feature = "std"))]
use simba::scalar::ComplexField;

/// Diagonalises the symmetric matrix `a` (`a[row][col]`, only the upper triangle read):
/// its eigenvalues in ascending order and, column for column (`v[row][col]`), their unit
/// eigenvectors as a right-handed orthonormal frame. Cyclic Jacobi rotations in `f64`;
/// a matrix with an infinite or NaN entry gives NaNs.
fn jacobi3(mut a: [[f64; 3]; 3]) -> ([f64; 3], [[f64; 3]; 3]) {
    for i in 0..3 {
        for j in i..3 {
            a[j][i] = a[i][j];
        }
    }
    let mut v = [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]];
    for _ in 0..32 {
        let off = a[0][1] * a[0][1] + a[0][2] * a[0][2] + a[1][2] * a[1][2];
        let diag = a[0][0] * a[0][0] + a[1][1] * a[1][1] + a[2][2] * a[2][2];
        if off <= f64::EPSILON * f64::EPSILON * diag || !off.is_finite() {
            break;
        }
        for (p, q) in [(0, 1), (0, 2), (1, 2)] {
            let apq = a[p][q];
            if apq == 0.0 {
                continue;
            }
            // tan of the rotation that zeroes a[p][q]: the smaller root.
            let theta = (a[q][q] - a[p][p]) / (2.0 * apq);
            let t = theta.signum() / (theta.abs() + (theta * theta + 1.0).sqrt());
            let c = 1.0 / (t * t + 1.0).sqrt();
            let s = t * c;
            a[p][p] -= t * apq;
            a[q][q] += t * apq;
            a[p][q] = 0.0;
            a[q][p] = 0.0;
            let r = 3 - p - q;
            let (arp, arq) = (a[r][p], a[r][q]);
            a[r][p] = c * arp - s * arq;
            a[p][r] = a[r][p];
            a[r][q] = s * arp + c * arq;
            a[q][r] = a[r][q];
            for row in &mut v {
                let (vp, vq) = (row[p], row[q]);
                row[p] = c * vp - s * vq;
                row[q] = s * vp + c * vq;
            }
        }
    }
    // Ascending eigenvalues, their columns following.
    let mut order = [0usize, 1, 2];
    for i in 0..2 {
        for j in i + 1..3 {
            if a[order[j]][order[j]] < a[order[i]][order[i]] {
                order.swap(i, j);
            }
        }
    }
    let values = [
        a[order[0]][order[0]],
        a[order[1]][order[1]],
        a[order[2]][order[2]],
    ];
    let mut vectors = [[0.0; 3]; 3];
    for (k, &col) in order.iter().enumerate() {
        for row in 0..3 {
            vectors[row][k] = v[row][col];
        }
    }
    // A permutation of the columns may have turned the frame left-handed.
    let x = [vectors[0][0], vectors[1][0], vectors[2][0]];
    let y = [vectors[0][1], vectors[1][1], vectors[2][1]];
    let z = [vectors[0][2], vectors[1][2], vectors[2][2]];
    let cross = [
        x[1] * y[2] - x[2] * y[1],
        x[2] * y[0] - x[0] * y[2],
        x[0] * y[1] - x[1] * y[0],
    ];
    if cross[0] * z[0] + cross[1] * z[1] + cross[2] * z[2] < 0.0 {
        for row in &mut vectors {
            row[2] = -row[2];
        }
    }
    (values, vectors)
}

/// Macro to generate SymmetricEigen3 for a specific scalar type.
macro_rules! impl_symmetric_eigen3 {
    ($SymmetricEigen3:ident, $Mat3:ty, $Vec3:ty, $Real:ty $(, #[$attr:meta])*) => {
        #[doc = concat!("The eigen decomposition of a symmetric 3x3 matrix (", stringify!($Real), " precision).")]
        #[derive(Clone, Copy, Debug, PartialEq)]
        #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
        $(#[$attr])*
        pub struct $SymmetricEigen3 {
            /// The eigenvalues of the symmetric 3x3 matrix.
            pub eigenvalues: $Vec3,
            /// The three eigenvectors of the symmetric 3x3 matrix (as columns).
            pub eigenvectors: $Mat3,
        }

        impl $SymmetricEigen3 {
            /// Computes the eigen decomposition of the given symmetric 3x3 matrix.
            ///
            /// The eigenvalues are returned in ascending order `eigen1 < eigen2 < eigen3`.
            /// This can be reversed with the [`reverse`](Self::reverse) method.
            /// The eigenvectors are unit length and form a right-handed frame.
            ///
            /// Only the upper triangle of `mat` is read. The result is the same bits on
            /// every host (see the module documentation).
            pub fn new(mat: $Mat3) -> Self {
                let c = mat.to_cols_array();
                let at = |row: usize, col: usize| c[col * 3 + row] as f64;
                let (values, vectors) = jacobi3([
                    [at(0, 0), at(0, 1), at(0, 2)],
                    [0.0, at(1, 1), at(1, 2)],
                    [0.0, 0.0, at(2, 2)],
                ]);
                let column = |k: usize| {
                    <$Vec3>::new(
                        vectors[0][k] as $Real,
                        vectors[1][k] as $Real,
                        vectors[2][k] as $Real,
                    )
                };
                Self {
                    eigenvalues: <$Vec3>::new(
                        values[0] as $Real,
                        values[1] as $Real,
                        values[2] as $Real,
                    ),
                    eigenvectors: <$Mat3>::from_cols(column(0), column(1), column(2)),
                }
            }

            /// Reverses the order of the eigenvalues and their corresponding eigenvectors.
            pub fn reverse(&self) -> Self {
                Self {
                    eigenvalues: self.eigenvalues.zyx(),
                    eigenvectors: <$Mat3>::from_cols(
                        self.eigenvectors.z_axis,
                        self.eigenvectors.y_axis,
                        self.eigenvectors.x_axis,
                    ),
                }
            }

            /// Computes the eigenvalues of a symmetric 3x3 matrix, in ascending order.
            pub fn eigenvalues(mat: $Mat3) -> $Vec3 {
                Self::new(mat).eigenvalues
            }

            /// Computes the unit-length eigenvector corresponding to the `eigenvalue1` of `mat` that was
            /// computed from the root of a cubic polynomial with a multiplicity of 1.
            ///
            /// If the other two eigenvalues are well separated, this method can be used for computing
            /// all three eigenvectors. However, to avoid numerical issues when eigenvalues are close to
            /// each other, it's recommended to use the `eigenvector2` method for the second eigenvector.
            ///
            /// The third eigenvector can be computed as the cross product of the first two.
            pub fn eigenvector1(mat: $Mat3, eigenvalue1: $Real) -> $Vec3 {
                let cols = mat - <$Mat3>::from_diagonal(<$Vec3>::splat(eigenvalue1).into());
                let c0xc1 = cols.x_axis.cross(cols.y_axis);
                let c0xc2 = cols.x_axis.cross(cols.z_axis);
                let c1xc2 = cols.y_axis.cross(cols.z_axis);
                let d0 = c0xc1.length_squared();
                let d1 = c0xc2.length_squared();
                let d2 = c1xc2.length_squared();

                let mut d_max = d0;
                let mut i_max = 0;

                if d1 > d_max {
                    d_max = d1;
                    i_max = 1;
                }
                if d2 > d_max {
                    d_max = d2;
                    i_max = 2;
                }

                // When all eigenvalues are equal (multiplicity 3), mat - eigenvalue * I is the
                // zero matrix, so all cross products are zero. Any unit vector is a valid
                // eigenvector in this case.
                if d_max < 1e-20 {
                    return <$Vec3>::X;
                }

                if i_max == 0 {
                    c0xc1 / d0.sqrt()
                } else if i_max == 1 {
                    c0xc2 / d1.sqrt()
                } else {
                    c1xc2 / d2.sqrt()
                }
            }

            /// Computes the unit-length eigenvector corresponding to the `eigenvalue2` of `mat` that was
            /// computed from the root of a cubic polynomial with a potential multiplicity of 2.
            ///
            /// The third eigenvector can be computed as the cross product of the first two.
            pub fn eigenvector2(mat: $Mat3, eigenvector1: $Vec3, eigenvalue2: $Real) -> $Vec3 {
                // Compute right-handed orthonormal set { U, V, W }, where W is eigenvector1.
                let (u, v) = Self::any_orthonormal_pair(eigenvector1);

                // The unit-length eigenvector is E = x0 * U + x1 * V. We need to compute x0 and x1.
                //
                // Define the symmetric 2x2 matrix M = J^T * (mat - eigenvalue2 * I), where J = [U V]
                // and I is a 3x3 identity matrix. This means that E = J * X, where X is a column vector
                // with rows x0 and x1. The 3x3 linear system (mat - eigenvalue2 * I) * E = 0 reduces to
                // the 2x2 linear system M * X = 0.
                //
                // When eigenvalue2 != eigenvalue3, M has rank 1 and is not the zero matrix.
                // Otherwise, it has rank 0, and it is the zero matrix.

                let au = mat * u;
                let av = mat * v;

                let mut m00 = u.dot(au) - eigenvalue2;
                let mut m01 = u.dot(av);
                let mut m11 = v.dot(av) - eigenvalue2;
                let (abs_m00, abs_m01, abs_m11) = (m00.abs(), m01.abs(), m11.abs());

                if abs_m00 >= abs_m11 {
                    let max_abs_component = abs_m00.max(abs_m01);
                    if max_abs_component > 0.0 {
                        if abs_m00 >= abs_m01 {
                            // m00 is the largest component of the row.
                            // Factor it out for normalization and discard to avoid underflow or overflow.
                            m01 /= m00;
                            m00 = 1.0 / (1.0 + m01 * m01).sqrt();
                            m01 *= m00;
                        } else {
                            // m01 is the largest component of the row.
                            // Factor it out for normalization and discard to avoid underflow or overflow.
                            m00 /= m01;
                            m01 = 1.0 / (1.0 + m00 * m00).sqrt();
                            m00 *= m01;
                        }
                        return m01 * u - m00 * v;
                    }
                } else {
                    let max_abs_component = abs_m11.max(abs_m01);
                    if max_abs_component > 0.0 {
                        if abs_m11 >= abs_m01 {
                            // m11 is the largest component of the row.
                            // Factor it out for normalization and discard to avoid underflow or overflow.
                            m01 /= m11;
                            m11 = 1.0 / (1.0 + m01 * m01).sqrt();
                            m01 *= m11;
                        } else {
                            // m01 is the largest component of the row.
                            // Factor it out for normalization and discard to avoid underflow or overflow.
                            m11 /= m01;
                            m01 = 1.0 / (1.0 + m11 * m11).sqrt();
                            m11 *= m01;
                        }
                        return m11 * u - m01 * v;
                    }
                }

                // M is the zero matrix, any unit-length solution suffices.
                u
            }

            /// Computes the third eigenvector as the cross product of the first two.
            /// If the given eigenvectors are valid, the returned vector should be unit length.
            pub fn eigenvector3(eigenvector1: $Vec3, eigenvector2: $Vec3) -> $Vec3 {
                eigenvector1.cross(eigenvector2)
            }

            /// Compute any orthonormal pair of vectors perpendicular to `v`.
            fn any_orthonormal_pair(v: $Vec3) -> ($Vec3, $Vec3) {
                let sign = if v.z >= 0.0 { 1.0 } else { -1.0 };
                let a = -1.0 / (sign + v.z);
                let b = v.x * v.y * a;
                let u = <$Vec3>::new(1.0 + sign * v.x * v.x * a, sign * b, -sign * v.x);
                let w = <$Vec3>::new(b, sign + v.y * v.y * a, -v.y);
                (u, w)
            }
        }
    };
}

impl_symmetric_eigen3!(SymmetricEigen3, glam::Mat3, glam::Vec3, f32);
impl_symmetric_eigen3!(SymmetricEigen3A, glam::Mat3A, glam::Vec3A, f32);
#[cfg(feature = "f64")]
impl_symmetric_eigen3!(DSymmetricEigen3, glam::DMat3, glam::DVec3, f64);

// f32 <-> f64 conversions
#[cfg(feature = "f64")]
impl From<SymmetricEigen3> for DSymmetricEigen3 {
    #[inline]
    fn from(e: SymmetricEigen3) -> Self {
        Self {
            eigenvalues: e.eigenvalues.as_dvec3(),
            eigenvectors: e.eigenvectors.as_dmat3(),
        }
    }
}

#[cfg(feature = "f64")]
impl From<DSymmetricEigen3> for SymmetricEigen3 {
    #[inline]
    fn from(e: DSymmetricEigen3) -> Self {
        Self {
            eigenvalues: e.eigenvalues.as_vec3(),
            eigenvectors: e.eigenvectors.as_mat3(),
        }
    }
}

#[cfg(feature = "f64")]
impl From<SymmetricEigen3A> for DSymmetricEigen3 {
    #[inline]
    fn from(e: SymmetricEigen3A) -> Self {
        Self {
            eigenvalues: e.eigenvalues.as_dvec3(),
            eigenvectors: e.eigenvectors.as_dmat3(),
        }
    }
}

#[cfg(feature = "f64")]
impl From<DSymmetricEigen3> for SymmetricEigen3A {
    #[inline]
    fn from(e: DSymmetricEigen3) -> Self {
        Self {
            eigenvalues: e.eigenvalues.as_vec3a(),
            eigenvectors: e.eigenvectors.as_mat3().into(),
        }
    }
}

// SymmetricEigen3 <-> SymmetricEigen3A conversions
impl From<SymmetricEigen3> for SymmetricEigen3A {
    #[inline]
    fn from(e: SymmetricEigen3) -> Self {
        Self {
            eigenvalues: e.eigenvalues.into(),
            eigenvectors: glam::Mat3A::from(e.eigenvectors),
        }
    }
}

impl From<SymmetricEigen3A> for SymmetricEigen3 {
    #[inline]
    fn from(e: SymmetricEigen3A) -> Self {
        Self {
            eigenvalues: e.eigenvalues.into(),
            eigenvectors: glam::Mat3::from(e.eigenvectors),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn eigen_3x3() {
        let mat =
            glam::Mat3::from_cols_array_2d(&[[2.0, 7.0, 8.0], [7.0, 6.0, 3.0], [8.0, 3.0, 0.0]]);
        let eigen = SymmetricEigen3::new(mat);

        assert_relative_eq!(
            eigen.eigenvalues,
            glam::Vec3::new(-7.6051016780, 0.5774961930, 15.0276054850),
            epsilon = 1.0e-6
        );
        // Each column is the eigenvector up to its sign.
        let want = [
            glam::Vec3::new(1.0754474819, -0.3328260589, -1.0).normalize(),
            glam::Vec3::new(-0.5420669703, 1.2530131898, -1.0).normalize(),
            glam::Vec3::new(1.3587443369, 1.3858835966, 1.0).normalize(),
        ];
        for (k, w) in want.iter().enumerate() {
            let got = eigen.eigenvectors.col(k);
            assert_relative_eq!(got.dot(*w).abs(), 1.0, epsilon = 1.0e-6);
        }
        assert!(eigen.eigenvectors.determinant() > 0.0);
    }

    #[test]
    fn eigen_3x3_diagonal() {
        let mat =
            glam::Mat3::from_cols_array_2d(&[[2.0, 0.0, 0.0], [0.0, 5.0, 0.0], [0.0, 0.0, 3.0]]);
        let eigen = SymmetricEigen3::new(mat);

        assert_eq!(eigen.eigenvalues, glam::Vec3::new(2.0, 3.0, 5.0));
        assert_eq!(
            glam::Mat3::from_cols(
                eigen.eigenvectors.x_axis.normalize().abs(),
                eigen.eigenvectors.y_axis.normalize().abs(),
                eigen.eigenvectors.z_axis.normalize().abs()
            ),
            glam::Mat3::from_cols_array_2d(&[[1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]])
        );
        assert!(eigen.eigenvectors.determinant() > 0.0);
    }

    #[cfg(feature = "f64")]
    #[test]
    fn eigen_3x3_f64() {
        let mat =
            glam::DMat3::from_cols_array_2d(&[[2.0, 7.0, 8.0], [7.0, 6.0, 3.0], [8.0, 3.0, 0.0]]);
        let eigen = DSymmetricEigen3::new(mat);

        assert_relative_eq!(
            eigen.eigenvalues,
            glam::DVec3::new(-7.6051016780, 0.5774961930, 15.0276054850),
            epsilon = 1.0e-8
        );
    }

    #[test]
    fn eigen_3x3_identity() {
        // Identity matrix has all eigenvalues equal to 1 (multiplicity 3)
        // Any orthonormal basis is a valid set of eigenvectors
        let mat = glam::Mat3::IDENTITY;
        let eigen = SymmetricEigen3::new(mat);

        assert_eq!(eigen.eigenvalues, glam::Vec3::ONE);

        // Verify eigenvectors are orthonormal (any orthonormal basis is valid)
        assert_relative_eq!(
            eigen.eigenvectors * eigen.eigenvectors.transpose(),
            glam::Mat3::IDENTITY,
            epsilon = 1e-6
        );
    }

    #[test]
    fn eigen_3x3_uniform_scaling() {
        // Uniform scaling matrix: 5*I has all eigenvalues equal to 5
        let mat = glam::Mat3::from_diagonal(glam::Vec3::splat(5.0));
        let eigen = SymmetricEigen3::new(mat);

        assert_eq!(eigen.eigenvalues, glam::Vec3::splat(5.0));

        // Verify eigenvectors are orthonormal
        assert_relative_eq!(
            eigen.eigenvectors * eigen.eigenvectors.transpose(),
            glam::Mat3::IDENTITY,
            epsilon = 1e-6
        );
    }

    /// A rotated box's inertia (values 5, 10, 14) comes back diagonalised, and the
    /// bits of one decomposition are pinned: the contract that every host gives the same.
    #[test]
    fn eigen_3x3_holds_its_bits() {
        let r = glam::Mat3::from_quat(glam::Quat::from_xyzw(0.1, 0.7, -0.3, 0.6).normalize());
        let mat = r * glam::Mat3::from_diagonal(glam::Vec3::new(14.0, 5.0, 10.0)) * r.transpose();
        let eigen = SymmetricEigen3::new(mat);
        assert_relative_eq!(eigen.eigenvalues, glam::Vec3::new(5.0, 10.0, 14.0), epsilon = 1e-4);
        let back = eigen.eigenvectors
            * glam::Mat3::from_diagonal(eigen.eigenvalues)
            * eigen.eigenvectors.transpose();
        assert_relative_eq!(back, mat, epsilon = 1e-4);
        assert!(eigen.eigenvectors.determinant() > 0.0);

        let mat = glam::Mat3::from_cols(
            glam::Vec3::new(2.0, -1.0, 0.5),
            glam::Vec3::new(-1.0, 3.0, 0.25),
            glam::Vec3::new(0.5, 0.25, 1.0),
        );
        let eigen = SymmetricEigen3::new(mat);
        let got: std::vec::Vec<u32> = eigen
            .eigenvalues
            .to_array()
            .iter()
            .chain(&eigen.eigenvectors.to_cols_array())
            .map(|x| x.to_bits())
            .collect();
        assert_eq!(got, PINNED, "{:?} {:?}", eigen.eigenvalues, eigen.eigenvectors);
    }

    /// The bits of the decomposition above, recorded once on arm64 macOS.
    const PINNED: [u32; 12] = [
        0x3f1a1144, 0x3fe3ba56, 0x40679e84, // 0.60182595, 1.779124, 3.61905
        0xbf00501d, 0xbe967327, 0x3f505b9d, // the first column
        0x3f2f20a1, 0x3ee202bc, 0x3f14a5d6, // the second
        0xbf07a792, 0x3f590a9f, 0xbca5c349, // the third
    ];
}
