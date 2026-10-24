# Reload and orb memory, 2026-10-24

The runner reload rejection was an engine asset-registration ordering bug.
The reproduced CPU-renderer OOM was in the old Mesa environment. These
required separate fixes. No resolution, reference image, or budget changed.

## Tilemap restore and native ownership

The tilemap regression failed before the fix with `map 4294967297 is not
loaded`. Retained assets now register before the candidate restores the
world. `cargo test -p og-runtime --test tilemap_reload` checks two reloads,
the whole world hash, edited and empty cells, collisions, and the new system.

The native host also retained a device handle after engine destruction.
The backend now removes that handle when dropped. The native render tests
check replacement, destruction, and failed attachment. This defect was not
the cause of the single-instance orb memory growth below.

A live runner TS title edit displayed `RELOAD VERIFIED`, with the city,
rooftop, courier, and HUD intact. The scripted frame used:

```sh
og run <runner-project> --headless --commands \
  '0.3:screenshot:runner.png;0.4:dump_world:world.json;0.5:quit' --out <evidence>
```

## Mesa controlled comparison

The orb had 16 Xeon vCPUs, 31 GiB RAM, no swap, and a 30 GiB workload
limit. Both driver arms used the published 0.4.0 3D template and binaries,
LLVM 15.0.6, and 1280×720 rendering. The new driver was first isolated with
its matching libdrm, then installed by the proposed `.agents/setup`.

| Check | Mesa 22.3.6 | Mesa 25.0.7, libdrm 2.4.123 |
|---|---|---|
| 150-frame RSS | about 582 → 1485 MiB | stable, about 398–496 MiB |
| Engine live heap | about 19.4 MB | stable |
| Render median, two runs | 138–145 ms | 71–74 ms |
| `og test . --filter intro`, 4 GiB, no swap | kernel OOM kill at about 106 s | 17 checks pass, 32.90 s |
| Capped intro maximum RSS, `/usr/bin/time -v` | process killed | 439,036 KiB |
| Capped intro cgroup peak | scope ended before final sample | 369,246,208 bytes, no OOM |

The old-driver kernel receipt names `CONSTRAINT_MEMCG`, the rendering Bun
process, and `anon-rss:4151200kB`. Systemd records `oom-kill`; the wrapper
returns 143. With the unrestricted limit, the entire old-driver suite
passes in 6:23.53, with maximum RSS 5,798,904 KiB.

Forced Bun GC reduced JS heap but did not change the RSS slope. Waiting
for GPU completion every frame did not eliminate growth either. No-render
did not grow. The driver comparison preserves the world hash and captured
pixels (AE 0, RMSE 0). It isolates the failure to the driver stack, not a
specific upstream Mesa commit or the original unlogged user environment.

Setup ran twice successfully. The second run took 0.65 s and ran no apt
step. A clean login shell, without `VK_ICD_FILENAMES` or `LD_LIBRARY_PATH`,
selected the upgraded system driver. Its capped intro passed in 34.63 s,
with maximum RSS 446,940 KiB and cgroup peak 370,065,408 bytes, no OOM.
The final system-driver screenshot was inspected and matched the isolated
driver's pixels.

## Reload keeps the GPU completion measurement

The candidate waits for the first rendered frame instead of capturing an
unused second frame, reading it back, and encoding a PNG. Alternating
baseline and candidate runs used three edits in each of three runs per arm.
These are `og dev totalMs`, including rebuild, not just snapshot restore.

| Driver | Published median (range), ms | Candidate median (range), ms |
|---|---|---|
| Mesa 22.3.6 | 783.0 (583.1–1057.7) | 533.4 (378.9–724.4) |
| Mesa 25.0.7 | 235.0 (217.0–268.1) | 149.4 (106.5–179.2) |

All reloads succeeded. All nine candidate samples on the new driver met
300 ms. The exact candidate `run.ts` SHA256 is
`452d98f56258b98b953689db1a419271123bc2b199b8f42fd822947f53620dbc`.
The published CLI did not forward the new per-stage fields. The probe's
reload call itself stayed near 3 ms.

## Texture reuse and validation scope

The shared texture cache reuses validated cooked bytes across fresh
projects. Five alternating fresh-project runs on the Mac dev binary had
median 3017.8 ms without it and 840.4 ms with a primed cache. Every run's
21 blobs had identical SHA256 values. This does not accelerate the first
encoding of unseen content and is not an orb measurement.

The combined local functional gate passed all five steps in 201 s, with
the shared user cache disabled. Targeted reload tests also passed after
the final telemetry rename. The orb tested the exact TypeScript candidate
and setup script, not the three new Rust changes. Those Rust changes were
validated locally; no rebuilt Linux package or new npm release was tested.

Raw evidence is retained in the integrating thread's
`.amp/in/artifacts/orb-evidence.tar.gz`, `orb-validation/oom-confirmation.txt`,
and `texture-cache-benchmark.jsonl`. The orb reproduction thread is
[the Linux validation thread](https://ampcode.com/threads/T-109a58c5-ea30-4526-97a6-330c73f77839).
