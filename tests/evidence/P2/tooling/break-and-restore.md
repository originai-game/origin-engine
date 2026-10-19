# Break and restore, phase 2 tooling round

| Test | The break | How it failed |
|---|---|---|
| packages/engine `reload-asset.test.ts` | the runtime before 3d7ad18 (no model refresh after a reload) | 0 red pixels after the edit (more than 2,000 wanted) |
| `tests/web/reference-gfx.ts --webgl` | `og_gfx_capture` awaiting `Renderer::capture()` without polling the device | the first capture never returned on WebGL2 |
| `tests/web/reference-gfx.ts --webgl` | no WebGL2 references (every scene against metal) | showcase RMSE 0.0107, overlay 0.0103, water-3d 0.0258 over 0.01; msaa had no reference |
