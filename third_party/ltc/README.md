# ltc

The GGX tables of Eric Heitz, Jonathan Dupuy, Stephen Hill and David
Neubelt's "Real-Time Polygonal-Light Shading with Linearly Transformed
Cosines" (ACM Transactions on Graphics, SIGGRAPH 2016), from
https://github.com/selfshadow/ltc_code at commit
`31e5e96b54f98f33098f8503003119ba2231a1c6` (`fit/results/ltc.js`, the
64 × 64 `g_ltc_1` and `g_ltc_2` tables three.js's RectAreaLight uses), under
the BSD-style licence in `LICENSE`.

`ltc32.bin` is those tables resampled bilinearly to 32 × 32 (texel (x, y) at
roughness x / 31 and sqrt(1 - cos θ) y / 31), each texel eight little-endian
IEEE halves: `g_ltc_1`'s four values, `g_ltc_2`'s first two, two zeros. 16 KiB:
one uniform buffer, so og-render's rect lights take no texture slot
(`crates/og-render/src/area.rs`).
