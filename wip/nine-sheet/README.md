# WIP repro: pack textures draw nothing on the web (not for landing)

From the paint thread T-e2c409a7, on main 8aa14ec9b.

`project/` draws every built-in pack's nine-slice parts (Button.plate, Chip.frame, Plate.frame, Panel.frame) plus a plain
`<Image>` of `@styles/scifi/art/chip.png` at (800, 200). Natively (`og run`, lavapipe and Colab L4) all of it draws.
On the dev web build in headless Chrome 157 (SwiftShader) none of the art draws, the plain Image included; the backdrop does.

    OG_WASM_VARIANTS=dev scripts/check.sh --only wasm-bindgen && OG_GATE=1 sh packages/engine/scripts/build.sh
    cd packages/engine && PACKS=scifi DPRS=1 bun ../../wip/nine-sheet/web.ts $PWD /tmp/nweb

`web.ts` cooks the project, builds the web export, opens it with `page.ts` (tests/web/reference.page.ts with a `dpr`
param), runs `pack:<name>` and `sheet`, and saves `probe("capture")` as `web-<pack>-<dpr>.png`.
`python3 gen.py <engine checkout>` regenerates `project/src/main.tsx` (without the Image) and Chrome border-image pages in ref/.
