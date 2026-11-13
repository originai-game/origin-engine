//! The aurora's deposition table: how much of each line a precipitating electron excites at each height.

use super::*;

/// The U.S. Standard Atmosphere 1976's mass density (km, kg/m³).
const US76: [(f64, f64); 23] = [
    (80.0, 1.846e-5),
    (85.0, 6.958e-6),
    (90.0, 3.416e-6),
    (95.0, 1.393e-6),
    (100.0, 5.604e-7),
    (105.0, 2.325e-7),
    (110.0, 9.708e-8),
    (115.0, 4.289e-8),
    (120.0, 2.222e-8),
    (130.0, 8.152e-9),
    (140.0, 3.831e-9),
    (150.0, 2.076e-9),
    (160.0, 1.233e-9),
    (170.0, 7.815e-10),
    (180.0, 5.194e-10),
    (190.0, 3.581e-10),
    (200.0, 2.541e-10),
    (220.0, 1.367e-10),
    (250.0, 6.073e-11),
    (300.0, 1.916e-11),
    (350.0, 7.014e-12),
    (400.0, 2.803e-12),
    (500.0, 5.215e-13),
];

/// The density at `z` km (kg/m³), log-linear between the table's heights.
fn density(z: f64) -> f64 {
    let k = US76
        .windows(2)
        .position(|w| z < w[1].0)
        .unwrap_or(US76.len() - 2);
    let (a, b) = (US76[k], US76[k + 1]);
    let scale = (b.0 - a.0) / (a.1 / b.1).ln();
    a.1 * (-(z - a.0) / scale).exp()
}

/// The mass of air above `z` km (kg/m²), each interval integrated as the
/// exponential it is, the last one's scale height above the table.
fn shielding(z: f64) -> f64 {
    let mut m = 0.0;
    for w in US76.windows(2) {
        let (a, b) = (w[0], w[1]);
        if z >= b.0 {
            continue;
        }
        let scale = (b.0 - a.0) / (a.1 / b.1).ln() * 1000.0;
        let from = z.max(a.0);
        m += scale * (density(from) - b.1);
    }
    let (a, b) = (US76[US76.len() - 2], US76[US76.len() - 1]);
    let top = (b.0 - a.0) / (a.1 / b.1).ln() * 1000.0;
    m + top * density(z.max(b.0))
}

/// Lazarev's energy deposition (keV per km per electron) of electrons of
/// `e` keV where the air's density is `d` g/cm³ under `m` g/cm² of air
/// (Lawlor & Genetti 2011, section 2.1).
fn lazarev(d: f64, m: f64, e: f64) -> f64 {
    let me = 4.6e-6 * e.powf(1.65);
    let r = m / me;
    let l = 4.2 * r * (-r * r - r).exp() + 0.48 * (-17.4 * r.powf(1.37)).exp();
    l * e * d / me * 1e5
}

/// The fraction of the air that is atomic oxygen at `z` km (MSIS-like:
/// a few percent at 100 km, most of it above 200 km).
fn oxygen(z: f64) -> f64 {
    0.9 / (1.0 + (-(z - 160.0) / 30.0).exp())
}

/// The spectrum's energies (keV, 0.1 to 40, log-spaced) and each one's
/// width.
fn energies() -> impl Iterator<Item = (f64, f64)> {
    (0..48).map(|k| {
        let e = 0.1 * 400f64.powf(k as f64 / 47.0);
        (e, e * 400f64.ln() / 47.0)
    })
}

/// The volume emission rates (per km, per unit energy flux into the
/// column) at `n` heights for characteristic energy `e0`, of a Maxwellian
/// spectrum's deposition: the prompt nitrogen bands (427.8 nm and the
/// first positive band), OI 557.7 nm and OI 630.0 nm, before their
/// efficiencies. `deposits[z][k]`: Lazarev's deposition at height z of
/// the k-th energy of [`energies`].
pub(super) fn emission_rows_of(e0: f64, heights: &[f64], deposits: &[Vec<f64>]) -> Vec<[f64; 3]> {
    let dz = (TABLE_TOP - TABLE_BOTTOM) / heights.len() as f64;
    // The spectrum: the primaries and, with a tenth of
    // the energy flux, the soft electrons that come down with every
    // discrete arc (secondaries and degraded primaries returning from the
    // ionosphere below, trapped and reflected above the accelerating
    // region: Evans, J. Geophys. Res. 79, 1974), a Maxwellian of 0.25 keV.
    // They deposit high, where the red line glows: an arc's top fades from
    // green into a faint red even when its primaries are hard.
    //
    // A discrete arc's primaries have fallen through a field-aligned
    // potential (the inverted V: Evans 1974): an accelerated Maxwellian,
    // nothing below the potential V and a source temperature kT above it
    // (Strickland, Meier, Hecht & Christensen, J. Geophys. Res. 94, 1989).
    // At keV energies V = 1.6 e0 and kT = 0.2 e0, whose mean energy is a
    // Maxwellian of e0's (2 e0), so `energy` keeps its meaning; lacking
    // the Maxwellian's high tail, the primaries stop within a few
    // kilometres of one depth, the sharp, bright lower edge of an arc, the
    // light fading upward. Soft precipitation (diffuse aurora, the cusp)
    // is not accelerated: below 1.5 keV the potential fades out, the mean
    // energy kept, to a plain Maxwellian at 0.5 keV.
    let maxwellian = |e: f64, k: f64| e * (-e / k).exp();
    let a = smooth(0.5, 1.5, e0);
    let (v, kt) = (ACCELERATION * a * e0, (1.0 - ACCELERATION * a / 2.0) * e0);
    let accelerated = |e: f64| {
        if e >= v {
            e * (-(e - v) / kt).exp()
        } else {
            0.0
        }
    };
    let flux = |f: &dyn Fn(f64) -> f64| -> f64 { energies().map(|(e, de)| f(e) * e * de).sum() };
    let (primary, soft) = (flux(&accelerated), flux(&|e| maxwellian(e, SOFT_E0)));
    let deposit: Vec<f64> = deposits
        .iter()
        .map(|row| {
            energies()
                .zip(row)
                .map(|((e, de), a)| {
                    let n = (1.0 - SOFT_SHARE) * accelerated(e) / primary
                        + SOFT_SHARE * maxwellian(e, SOFT_E0) / soft;
                    n * a * de
                })
                .sum()
        })
        .collect();
    let total: f64 = deposit.iter().sum::<f64>() * dz;
    let n2_ref = 1.0 - oxygen(100.0);
    heights
        .iter()
        .zip(&deposit)
        .map(|(&z, &a)| {
            let a = a / total;
            let rho = density(z);
            // Quenching by collisions: the green line's 0.7 s state below
            // about 97 km, the red line's 110 s state below about 210 km.
            let green = a / (1.0 + rho / 1.2e-6);
            let red = a * oxygen(z) / (1.0 + rho / 1.5e-10);
            let n2 = a * (1.0 - oxygen(z)) / n2_ref;
            [n2, green, red]
        })
        .collect()
}

/// The table's heights (km, texel centres) and Lazarev's deposition at
/// each for each energy.
fn deposits(n: usize) -> (Vec<f64>, Vec<Vec<f64>>) {
    let dz = (TABLE_TOP - TABLE_BOTTOM) / n as f64;
    let heights: Vec<f64> = (0..n)
        .map(|x| TABLE_BOTTOM + (x as f64 + 0.5) * dz)
        .collect();
    let deposits = heights
        .iter()
        .map(|&z| {
            let d = density(z) * 1e-3; // g/cm³
            let m = shielding(z) * 0.1; // g/cm²
            energies().map(|(e, _)| lazarev(d, m, e)).collect()
        })
        .collect();
    (heights, deposits)
}

#[cfg(test)]
pub(super) fn emission_rows(e0: f64, n: usize) -> Vec<[f64; 3]> {
    let (heights, deposits) = deposits(n);
    emission_rows_of(e0, &heights, &deposits)
}

/// The table: `TABLE_ROWS` rows of `n` texels (rgba: prompt N₂, 557.7,
/// 630.0, 0), each row's characteristic energy `TABLE_E0` times
/// 2^(row × `TABLE_OCTAVES`).
pub(crate) fn table(n: usize) -> Vec<[f32; 4]> {
    let (heights, deposits) = deposits(n);
    (0..TABLE_ROWS)
        .flat_map(|row| {
            let e0 = TABLE_E0 * 2f64.powf(f64::from(row) * TABLE_OCTAVES);
            emission_rows_of(e0, &heights, &deposits)
                .into_iter()
                .map(|[a, b, c]| [a as f32, b as f32, c as f32, 0.0])
        })
        .collect()
}

/// The lines' colours in linear sRGB per photon (radiant power falls as
/// 1/λ), weighted by their efficiencies: the prompt nitrogen bands, the
/// green line, the red line.
pub(super) fn line_colors() -> [Vec3; 3] {
    // CIE 1931 2° colour matching values (x̄, ȳ, z̄) at the line, linearly
    // interpolated from the 5 nm table.
    let line = |xyz: [f32; 3], nm: f32| {
        let [x, y, z] = xyz;
        let rgb = Vec3::new(
            3.2406 * x - 1.5372 * y - 0.4986 * z,
            -0.9689 * x + 1.8758 * y + 0.0415 * z,
            0.0557 * x - 0.2040 * y + 1.0570 * z,
        );
        // Into the gamut by adding white (desaturating, keeping the hue).
        let lift = (-rgb.min_element()).max(0.0);
        (rgb + Vec3::splat(lift)) * (557.7 / nm)
    };
    let blue = line([0.2535, 0.0097, 1.233], 427.8);
    let first_positive = line([0.1649, 0.0610, 0.0], 660.0);
    let green = line([0.5566, 0.9973, 0.0047], 557.7);
    let red = line([0.6424, 0.2650, 0.0], 630.0);
    [
        blue * EFFICIENCY_4278 + first_positive * EFFICIENCY_1PG,
        green,
        red * EFFICIENCY_6300,
    ]
}
