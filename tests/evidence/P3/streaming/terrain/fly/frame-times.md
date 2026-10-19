Frame times flying the terrains (bun bench/terrain-fly.ts, 1280x720, metal; engine.frame plus the GPU wait, after 30 warm-up frames).

| Terrain | Mode | Budget | Frames | p50 ms | p95 ms | p99 ms | max ms | mean ms | CPU p50 / p99 / max ms | Streamer worst ms (with warm-up) | > 16.7 ms | > 33.3 ms |
|---|---|---|---|---|---|---|---|---|---|---|---|---|
| 2k | stream | 16 MiB | 1200 | 1.346 | 2 | 2.192 | 3.112 | 1.432 | 0.229 / 0.419 / 0.491 | 0.062 | 0 | 0 |
| 2k | resident | 16 MiB | 1200 | 1.326 | 1.902 | 2.119 | 2.533 | 1.397 | 0.22 / 0.402 / 0.511 | 15.649 | 0 | 0 |
| 2k | stream | 4 MiB | 1200 | 1.334 | 1.956 | 2.19 | 2.763 | 1.408 | 0.211 / 0.423 / 0.543 | 0.038 | 0 | 0 |
| 8k | stream | 16 MiB | 2400 | 1.409 | 2.003 | 2.207 | 2.552 | 1.487 | 0.265 / 0.588 / 0.998 | 0.117 | 0 | 0 |
| 8k | resident | 16 MiB | 2400 | 1.415 | 2.19 | 2.629 | 8.182 | 1.538 | 0.263 / 1.22 / 6.422 | 23.611 | 0 | 0 |
| 8k | stream | 4 MiB | 2400 | 1.372 | 2.023 | 2.282 | 4.982 | 1.456 | 0.224 / 0.499 / 1.362 | 0.052 | 0 | 0 |

| Terrain | Mode | Budget | Tiles resident (max) | Waiting (max) | Samples waiting | Cached tiles (max) | CPU cache (max) | Tiles decoded | Process RSS |
|---|---|---|---|---|---|---|---|---|---|
| 2k | stream | 16 MiB | 85 | 0 | 0 of 41 | 85 | 12.7 MiB | 85 | 507 MiB |
| 2k | resident | 16 MiB | 85 | 0 | 0 of 41 | 85 | 12.7 MiB | 85 | 510 MiB |
| 2k | stream | 4 MiB | 26 | 3 | 2 of 41 | 59 | 8.8 MiB | 59 | 475 MiB |
| 8k | stream | 16 MiB | 106 | 21 | 5 of 81 | 228 | 34.1 MiB | 473 | 741 MiB |
| 8k | resident | 16 MiB | 106 | 0 | 0 of 81 | 228 | 34.1 MiB | 473 | 738 MiB |
| 8k | stream | 4 MiB | 26 | 5 | 1 of 81 | 68 | 10.2 MiB | 202 | 682 MiB |
