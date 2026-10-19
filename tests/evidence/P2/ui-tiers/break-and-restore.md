# og-ui tiers round: break and restore

Each new reference tier was broken once by replacing a committed reference
with a wrong frame, seen to fail, and restored by copying the backup back
(SHA-1 checked equal before and after; no Git command touched the files).

| Tier and harness | Broken | Result while broken | Restored |
|---|---|---|---|
| `vulkan-cpu`, `cargo test -p og-ui --test gfx` (the zigbuild Linux binary in the og-lavapipe image) | `references/vulkan-cpu/sample-screen.png` replaced by the same frame mirrored left to right | **FAILED** (exit 101): `sample_screen_matches_its_reference_frame` panicked, "differs from …/vulkan-cpu/sample-screen.png: RMSE 0.16969 > 0.01; 26.193% outlier pixels > 0.5%" | `31f89a75…` again; gfx 5 of 5 and kit_gfx 1 of 1 pass, sample-screen and widget-set at RMSE 0 against `vulkan-cpu`, no fallback to `metal` |
| `webgl2`, `bun tests/web/reference-ui.ts --webgl --only locales` | `references/webgl2/kit-locale-mainMenu-en.png` replaced by `kit-locale-mainMenu-ar.png` | **FAILED** (exit 1): "FAIL kit-locale-mainMenu-en (webgl2, tick 60): 0.997 % outliers (≤ 0.5 %), RMSE 0.04836 (≤ 0.01) against crates/og-ui/references/webgl2/kit-locale-mainMenu-en.png"; 0 of 1 playtests matched | `999d2ee7…` again; the rerun passes, 15 of 15 locale screens at RMSE 0 |
| `webgpu`, `bun tests/web/reference-ui.ts --only skin-neon` (a skin sheet, cell by cell) | `references/webgpu/kit-skin-neon.png` replaced by `kit-skin-paper.png` | **FAILED** (exit 1): all 20 cells fail, e.g. "cell 0 of kit-skin-neon.png: 42.008 % outliers, RMSE 0.44169" | `8748ef5d…` again; passes in the web-reference gate step |
| `webgpu`, `bun tests/web/reference-gfx.ts --scene og-ui/glass` | `references/webgpu/glass.png` replaced by the same frame mirrored | **FAILED** (exit 1): "FAIL og-ui/glass (webgpu): 70.844 % outliers (≤ 0.5 %), RMSE 0.84087 (≤ 0.01) against crates/og-ui/references/webgpu/glass.png; metal: RMSE 0" | `130901b6…` again; passes in the web-reference gate step |

The `vulkan-cpu` kit frames are judged by the same `tests/frames.ts` and
`tests/skins.ts` as `metal`; their run on lavapipe without
`OG_UI_CAPTURE` compares against `references/vulkan-cpu/` and never prints
the "comparing with metal" note (NOTES.md).
