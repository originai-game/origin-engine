//! og.destruction.material's presets.

/// og.destruction.material: (toughness, strength, friction, tension,
/// density, impactThreshold, impactRadius) per MATERIAL_*, the values the
/// demolition-yard props were tuned to.
pub const MATERIALS: [[f32; 7]; 5] = [
    // Brick: tough bricks, weak mortar held by friction where it is
    // pressed and next to nothing pulled apart; heavy blocks land at
    // hundreds of N·s, so a lower threshold lets falling blocks knock
    // the piers apart.
    [800.0, 3000.0, 0.7, 0.1, 1900.0, 400.0, 0.4],
    // Concrete: a column's top, dropping onto its stump when its middle is
    // shot out, lands at over 1000 N·s; below 1500 that crushes the stump.
    [2000.0, 20000.0, 0.7, 1.0, 2400.0, 1500.0, 0.4],
    // Wood: weak across the grain, a hit spread over most of a crate.
    [120.0, 30000.0, 0.7, 1.0, 600.0, 40.0, 0.9],
    // Glass: shatters at a touch, no stress solver.
    [25.0, 0.0, 0.7, 1.0, 2500.0, 2.0, 0.8],
    // Dry stone: no mortar, no tension.
    [1500.0, 4000.0, 0.8, 0.0, 2600.0, 500.0, 0.4],
];
