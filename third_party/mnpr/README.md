# mnpr

Santiago E. Montesdeoca's MNPR (Maya Non-Photorealistic Renderer),
https://github.com/semontesdeoca/MNPR, the watercolour stylisation of
"Edge- and substrate-based effects for watercolor stylization"
(Montesdeoca, Seah, Rall and Benvenuti, Expressive 2017) and "MNPR: a
framework for real-time expressive non-photorealistic rendering of 3D
computer graphics" (Montesdeoca et al., Expressive 2018):
`shaders/quadPigmentApplication.ogsfx` (the dry brush, pigment skipping
the paper where the paper's height rises past the wash's thickness),
`quadEdgeManipulation.ogsfx` (edge darkening as a pigment density, the
colour raised to one plus the edge's strength) and
`quadSeparable.ogsfx`'s substrate distortion (the frame displaced along
the paper's gradient). Ported, with Bousseau, Kaplan, Thollot and
Sillion's pigment density model ("Interactive watercolor rendering with
temporal coherence and abstraction", NPAR 2006: C(1 - (1 - C)(d - 1)) with
the density d about 1 from the paper's granulation and a turbulence), to
WGSL in `crates/og-render/shaders/stylise.wgsl` (`watercolor`) and to
Rust in `crates/og-cook/src/cook/conform/texture2d/paint.rs`
(`Watercolor`). Not ported: MNPR's gaps and overlaps, which need a
region's own displacement, and its hand-painted control maps, which the
engine takes from the painted colour itself.
MIT, Copyright (c) 2018 Nanyang Technological University, the notice in
`LICENSE`.
