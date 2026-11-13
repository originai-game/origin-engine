# Colour vision simulation and correction

The matrices in `crates/og-render/src/post/color_vision.rs`, applied by
`crates/og-render/shaders/color_filter.wgsl` (docs/ui.md §3.2).

- Simulation: libDaltonLens, https://github.com/DaltonLens/libDaltonLens,
  revision `1a01fc1bf8d8dd419af8343b80b05e98ba50a75d`, public domain
  (Unlicense, `LICENSE-libDaltonLens.txt`). Viénot, Brettel and Mollon 1999
  for protanopia and deuteranopia (`dl_vienot_*_rgbCvd_from_rgb`), Brettel,
  Viénot and Mollon 1997 for tritanopia (`brettel_tritan_params`), linear
  sRGB, Smith & Pokorny LMS. Severity blends towards the input as
  `dl_simulate_cvd` does. The unit test compares against outputs of
  libDaltonLens.c itself.
- Correction: Fidaner, Lin and Ozguven, "Analysis of Color Blindness"
  (Stanford Psych 221, 2005), daltonisation `rgb + E·(rgb − simulate(rgb))`
  with the per-deficiency error matrices of andrewwillmott/colour-blind-luts,
  https://github.com/andrewwillmott/colour-blind-luts, revision
  `12d87832134e3ab08757676fcde04bdefdc04461` (`CBLuts.cpp`), public domain
  (Unlicense, `LICENSE-colour-blind-luts.txt`).
