# The visibility buffer's resolve on PowerVR

Chrome 156 on the Pixel 10 (PowerVR D-Series DXT-48-1536, Dawn on Vulkan)
refuses the lit resolve pipeline of the visibility buffer
(`meshlet.wgsl` `fs_meshlet_resolve`, with `ANALYTIC` true) with
`CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN`. The module is valid
WGSL (Tint and naga accept it, and every other GPU makes the pipeline).

- `meshlet-resolve.wgsl`: the module, captured from the engine Worker on the
  phone at 012a073a3. `meshlet-resolve.pipeline.json` is its descriptor.
- `variants.wgsl`, `record-variants.wgsl`: entry points that each change one
  thing (record-variants: copies of `meshlet_record` with one part removed).
- `repro.ts`: makes each named entry's pipeline on a bare device in the
  phone's Chrome and prints the answer:

```
bun tests/evidence/compat/android/powervr-resolve/repro.ts --android
adapter                          img-tec d-series
fs_meshlet_resolve               refused (internal): CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN
fs_meshlet_resolve@ANALYTIC=0    ok
v_constlit                       ok
v_synth                          ok
v_record_unused                  refused (internal): CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN
v_rec_noanalytic                 ok
v_rec_onlydx                     ok
v_rec_nouv                       refused (internal): CreateGraphicsPipelines failed with VK_ERROR_UNKNOWN
```

Reading: `lit_fragment` with `ANALYTIC` compiles while its record's
gradients fold to constants (`v_synth`, `v_constlit`) and while at most
one of the record's gradient stores (`analytic_uv`, `analytic_dx`,
`analytic_dy`) stays (`v_rec_noanalytic`, `v_rec_onlydx`); with the record
read but its vertex unused (`v_record_unused`) or `dx` and `dy` both stored
(`v_rec_nouv`) it is refused. A minimal shader with the same dependent loads
and cross product compiles. No single construct is at fault: the driver's
compiler does not fit the whole shader.

The engine's answer (og-render `meshlet_gpu/vis.rs`): the resolve's
pipelines are made inside error scopes, and a refusal turns the visibility
buffer off for the device (`RenderCapabilities.visibilityBuffer` false,
`visibilityBufferRefused` the driver's message, one warning in the log); its
clusters draw on the hardware meshlet path. Splitting the resolve so that
PowerVR keeps the visibility buffer is later work.
