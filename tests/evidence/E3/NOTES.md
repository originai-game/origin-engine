# E3 evidence

One line per file: what it is, how it was produced, at which revision.

- `skin-pixel-sheet.png` — crates/og-ui/references/metal/kit-skin-pixel.png, recorded by OG_UI_CAPTURE=1 og test templates/ui-kit (skin-pixel: every screen but the gallery at 1280×720, box-filtered to 640×360, 7 × 3 cells) (rev 7db0908..24d3663, 2026-10-18, sha256 0ff18dfdbda1337a)
- `skin-cartoon-sheet.png` — the same for cartoon (kit-skin-cartoon.png) (rev 7db0908..24d3663, 2026-10-18, sha256 3c79df9bc1485ef4)
- `skin-fantasy-sheet.png` — the same for fantasy (kit-skin-fantasy.png) (rev 7db0908..24d3663, 2026-10-18, sha256 debe8c042110c751)
- `skin-scifi-sheet.png` — the same for scifi (kit-skin-scifi.png) (rev 7db0908..24d3663, 2026-10-18, sha256 485997096ff18b4d)
- `skin-paper-sheet.png` — the same for paper (kit-skin-paper.png) (rev 7db0908..24d3663, 2026-10-18, sha256 bb7d536b36d932f1)
- `skin-neon-sheet.png` — the same for neon (kit-skin-neon.png) (rev 7db0908..24d3663, 2026-10-18, sha256 6bd14947fe53d605)
- `mainMenu-before-after.png` — top left: the frames-desktop mainMenu capture at b94d12d (the platform tokens); then the skin-<pack> captures of mainMenu (pixel, cartoon, fantasy; scifi, paper, neon), 640×360 each, PIL (rev 7db0908..24d3663, 2026-10-18, sha256 364a1419c65b6caa)
- `results-before-after.png` — the same for results (rev 7db0908..24d3663, 2026-10-18, sha256 ad94f414df15394c)
- `transition-iris-countup.png` — og run templates/ui-kit --headless --size 1280x720 with the script in transition-transcript.md: frames at 0.4–0.8 s (top) and 0.9, 1.0, 1.2, 1.5, 2.4 s (bottom), 320×180 each (rev 7db0908..24d3663, 2026-10-18, sha256 20b28fa7fb903846)
- `transition-transcript.md` — the same run's dump_ui readings: the screen built and the Score text every 0.1 s (rev 7db0908..24d3663, 2026-10-18, sha256 c95b304871c7750e)
- `break-and-restore.md` — written during the round: each break, the test run, the result; and the pixel-identity check of the default skin (rev 7db0908..24d3663, 2026-10-18, sha256 dc348615486831e0)
- `fx-showcase-badge-diff.png` — the fx-showcase badge after 00653bf: per station the old metal reference, the new frame and the changed-pixel mask over x 1100–1271, y 640–699 at 2×, PIL (rev c1290aa, 2026-10-18, sha256 e7f97e6b98bc722e)
- `fx-showcase-badge.md` — the same change measured on metal, webgpu and webgl2: RMSE 0 outside the badge on every station; which references were re-recorded and the owed vulkan-cpu tier (rev c1290aa, 2026-10-18, sha256 f51ec2563876c4ab)
