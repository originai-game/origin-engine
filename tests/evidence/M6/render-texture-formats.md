# Texture format selection by tier (og-render, M6)

`compressed_formats_are_chosen_per_tier_and_draw_like_rgba8` in `crates/og-tests/tests/render.rs` on Apple M4 Pro, Metal (full tier). For each `RenderDesc::texture_compression` preference: the format the renderer chose for colour and for normal (two-channel) textures, the formats the device supports, and the RMSE / max channel difference of a block-encoded texture with pre-encoded mips drawn against the same texture uploaded as RGBA8. A format outside the device's set is refused with `RenderError::Unsupported` (asserted by the same test).

| preference | color | normal | supported | RMSE vs RGBA8 | max diff |
|---|---|---|---|---|---|
| Auto | Bc7 | Bc5 | [Rgba8, Bc7, Bc5, Astc4x4, Etc2Rgba8, EacRg11] | 0.00015 | 1 |
| PreferAstc | Astc4x4 | Astc4x4 | [Rgba8, Bc7, Bc5, Astc4x4, Etc2Rgba8, EacRg11] | 0.00012 | 1 |
| PreferEtc2 | Etc2Rgba8 | EacRg11 | [Rgba8, Bc7, Bc5, Astc4x4, Etc2Rgba8, EacRg11] | 0.00029 | 1 |
| Off | Rgba8 | Rgba8 | [Rgba8] | 0.00000 | 0 |

Metal on Apple silicon supports every format, so the table shows the preference order only; the WebGL2/mobile rows (ETC2/ASTC as the only compressed sets) have not been run on such a device.
