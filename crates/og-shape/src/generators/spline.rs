//! Natural cubic splines through values at increasing parameters
//! (sand3d's `_through`, `solids.py`): global and C², the curve a CAD loft
//! runs its points on. `+ - * /` only.

/// The second derivatives of the natural cubic spline through `values` at
/// the strictly increasing `params` (zero at both ends). Fewer than three
/// points give zeros: the spline is straight.
pub fn second_derivatives(params: &[f64], values: &[f64]) -> Vec<f64> {
    let m = values.len();
    let mut z = vec![0.0; m];
    if m < 3 {
        return z;
    }
    let h: Vec<f64> = params.windows(2).map(|w| w[1] - w[0]).collect();
    let (mut a, mut b, mut c, mut r) = (vec![0.0; m], vec![1.0; m], vec![0.0; m], vec![0.0; m]);
    for i in 1..m - 1 {
        a[i] = h[i - 1];
        b[i] = 2.0 * (h[i - 1] + h[i]);
        c[i] = h[i];
        r[i] = 6.0 * ((values[i + 1] - values[i]) / h[i] - (values[i] - values[i - 1]) / h[i - 1]);
    }
    for i in 1..m {
        let w = a[i] / b[i - 1];
        b[i] -= w * c[i - 1];
        r[i] -= w * r[i - 1];
    }
    z[m - 1] = r[m - 1] / b[m - 1];
    for i in (0..m - 1).rev() {
        z[i] = (r[i] - c[i] * z[i + 1]) / b[i];
    }
    z
}

/// The spline at fraction `f` of span `k` (from `params[k]` to
/// `params[k + 1]`), with `second` from [`second_derivatives`].
pub fn eval(params: &[f64], values: &[f64], second: &[f64], k: usize, f: f64) -> f64 {
    let h = params[k + 1] - params[k];
    let (u, w) = (1.0 - f, f);
    values[k] * u
        + values[k + 1] * w
        + ((u * u * u - u) * second[k] + (w * w * w - w) * second[k + 1]) * (h * h / 6.0)
}
