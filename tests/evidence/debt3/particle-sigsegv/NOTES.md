# debt3/particle-sigsegv: the particle test's SIGSEGV is the Vulkan loader's

Base `30376326` (origin/main). Branch `debt/particle-sigsegv`. Closes row
81 of `docs/phase-1.md`: og-tests' `a_600_tick_gpu_particle_run_is_deterministic`
crashed with SIGSEGV on lavapipe in CI, once on ci-green's gate job and
again in run 40224280502 (ci/known-frames, 2026-10-25, after 3.121 s,
right after the CPU variant had passed beside it).

## The answer

Reproduced and diagnosed. The crash is in the **Khronos Vulkan loader**
(`libvulkan.so.1`), not in lavapipe/llvmpipe, wgpu or og-render's particle
path. It is a use-after-free between two Vulkan instances on two threads
of one process, and the test made two renderers (two `wgpu::Instance`s,
two devices) at once on two threads:

- thread B, `Renderer::new` → `request_adapter` → `vkEnumeratePhysicalDevices`
  → `unload_drivers_without_physical_devices` (loader.c:6674): the first
  enumeration of an instance frees and `dlclose`s that instance's
  `loader_icd_term`s for the drivers with no device (intel, intel_hasvk,
  radeon, nouveau, virtio, gfxstream: the ICD files mesa-vulkan-drivers
  installs beside lavapipe's). It holds `loader_lock`;
- thread A, `Renderer::new` → `create_sampler` (og-render lib.rs:2562, the
  first labelled object after the device) → wgpu-hal names it →
  `vkSetDebugUtilsObjectNameEXT` → `terminator_SetDebugUtilsObjectNameEXT`
  → `loader_get_icd_and_device` (loader.c:1331), which walks **every**
  instance's `icd_terms` list under a different mutex,
  `loader_global_instance_list_lock`, and dereferences the freed entry
  (`obj=0x2`; another crash read `obj=0x6962617061635f65`, the ASCII
  "e_capabi" of whatever reused the block).

Nothing orders the two locks, so the walk races the free. Upstream fixed it
in KhronosGroup/Vulkan-Loader 5ee27b30 (2026-02-18, "Use recursive mutexes
to fix deadlocks in loader": `loader_global_instance_list_lock` removed, the
walk takes `loader_lock`, with a threading test of the debug-utils naming
functions), first released in v1.4.345. `unload_drivers_without_physical_devices`
arrived in v1.3.290, so every loader from 1.3.290 to 1.4.344 has the race.
Debian trixie (CI's `debian:trixie` container) ships 1.4.309; the orb's
bookworm ships 1.3.239, which has no unload and never crashed. Metal has
no Vulkan loader: the Mac lane is not exposed.

Other loader entry points walk the same list unlocked in 1.4.309
(`vkGetDeviceProcAddr`, `terminator_CreateSwapchainKHR`, the debug marker
and tag terminators, `loader_layer_destroy_device`), and one crash had the
other thread in `vkCreateDevice` instead of the unload. So no lock inside
og-render could make two renderers on two threads safe on these loaders:
wgpu-hal names every labelled object on its own. The only safe shape is
one renderer's setup and teardown never overlapping another's work in the
same process.

## The change

- `crates/og-tests/tests/effects.rs`, `particles_are_deterministic`: the
  two 600-tick runs one after the other on the test's thread (each
  renderer dropped before the next is made), not the second on a spawned
  thread. Same assertions; the frames are the same by construction (the
  test asserts the two runs byte-equal, as before).
- `.config/nextest.toml`: the two particle determinism tests leave the
  `threads-required = 2` override (they use one thread now); the pile and
  the walk keep it.
- No engine change: no production host makes two renderers at once in a
  process (og-host-node makes one per call on the JS thread, `og render`'s
  jobs are processes, the web host has no loader).
  `gpu_and_cpu_paths_simulate_the_same_particles` already makes its two
  renderers one after the other in a loop: not exposed, unchanged.
- No new test. The behaviour is the system loader's, intermittent, and only
  on an affected loader; the evidence that the old shape fails and the new
  one does not is the stress below, not a committed test.

Left exposed: `cargo test` (the gate's fallback when nextest is missing,
`scripts/check.sh` run_tests) runs a binary's tests on threads of one
process, so any two GPU tests there can meet the same race on an affected
loader. nextest (CI and the gate) runs one process per test. Not changed
here.

## How it was reproduced

Toolchain: the orb (Debian bookworm, 16 vCPU, gVisor), the dev profile
with `CARGO_PROFILE_DEV_DEBUG=line-tables-only` (CI's setting):

    CARGO_PROFILE_DEV_DEBUG=line-tables-only cargo test -p og-tests --test effects --no-run

CI's stack in a chroot (the binary is the orb's; glibc 2.41 runs it):

    sudo debootstrap --variant=minbase --include=libvulkan1,mesa-vulkan-drivers,libgcc-s1,gdb trixie /opt/trixie
    # bind /proc /dev /sys /tmp and the checkout; then, for symbols:
    echo 'deb http://deb.debian.org/debian-debug trixie-debug main' > /etc/apt/sources.list.d/debug.list
    apt-get install libvulkan1-dbgsym

giving libvulkan1 1.4.309.0-1, mesa-vulkan-drivers 25.0.7-2+deb13u1 and
libllvm19 1:19.1.7-3+b1 (CI's: workflow comment "its Lavapipe is the
og-lavapipe image's (LLVM 19.1.7)"). The orb's own: libvulkan1
1.3.239.0-1, mesa-vulkan-drivers 25.0.7-2~bpo12+1, libllvm15.

Each run is one process under gdb, the backtrace kept on a signal:

    gdb -q -batch -ex "set print thread-events off" -ex run \
      -ex "info threads" -ex "thread apply all bt 40" -ex "info registers rip" -ex 'x/6i $pc' \
      --args target/debug/deps/effects-<hash> --exact a_600_tick_gpu_particle_run_is_deterministic --test-threads=1

The full test takes 30 s alone here and the crash comes in its first
seconds (CI: 3.121 s), so a scratch stress test (not committed) repeated
only that part: per iteration, the test's two renderers (`RenderDesc` of
`small()` with the path, `fx::particles_scene()`), made and stepped 10
frames on two threads at once as the old test did (or one after the
other with `STRESS_SERIAL=1`), read back and compared, about 3 s each.

The load: 4 to 8 such processes at once on 16 vCPU (load average 30 to
56), the shape of the gate's `gpu` group of 4 on CI's 4 vCPU runner;
then the quiet runs, one process alone.

## Counts

A run is one process; a stress iteration is one pair of renderers. A
crashed stress process counts the iterations it finished plus the one it
crashed in.

| Loader (Mesa, LLVM) | Load | Shape | What ran | Runs | SIGSEGV |
|---|---|---|---|---|---|
| 1.3.239 (orb: 25.0.7 bpo, LLVM 15) | 4 to 8 at once | two threads (old) | full GPU test | 56 | 0 |
| 1.3.239 | 4 to 8 at once | two threads (old) | stress, GPU path | 400 | 0 |
| 1.4.309 (CI: 25.0.7 deb13u1, LLVM 19.1.7) | 4 to 8 at once | two threads (old) | full GPU test | 76 | **2** (runs 7 and 23 of two loops) |
| 1.4.309 | 4 to 8 at once | two threads (old) | stress, GPU path | 228 | **10** |
| 1.4.309 | 4 to 8 at once | two threads (old) | stress, CPU path | 98 | **5** |
| 1.4.309 | alone | two threads (old) | stress, GPU path | 120 | **2** (after 37 and 28) |
| 1.4.309 | 4 to 8 at once | one thread (new) | stress, GPU path | 500 | 0 |
| 1.4.309 | 4 to 8 at once | one thread (new) | stress, CPU path | 200 | 0 |
| 1.4.309 | 4 to 8 at once | one thread (new) | full GPU test | 28 | 0 |
| 1.4.309 | 4 to 8 at once | one thread (new) | full CPU test | 28 | 0 |

The old shape's stress crashed 17 times in 446 iterations on CI's loader
(about 1 in 26, with or without load); the new shape ran 700 without one
(by chance alone, with that rate, about e^-27). The full test meets the
window once a run, at its start, so it crashes less often than the stress
(2 in 76 here), and in CI in its first seconds (3.121 s). The orb's own
loader never crashed: it has no driver unload.

Every passing run of the new shape asserts what the old one did (the two
600-tick frames byte-equal, the jump to tick 600 equal to stepping, more
than 200 bright pixels, at least 6 systems).
`gpu_and_cpu_paths_simulate_the_same_particles` passed on CI's loader
(`alive: gpu 90 cpu 90`).

Wall time alone on CI's loader (16 vCPU, no gdb, `finished in`):

| Test | Old (two threads) | New (one thread) |
|---|---|---|
| `a_600_tick_gpu_particle_run_is_deterministic` | 20.40 s, 20.63 s | 34.91 s, 34.30 s |
| `a_600_tick_cpu_particle_run_is_deterministic` | 17.80 s, 18.94 s | 33.41 s, 33.13 s |

Each takes one nextest slot instead of two now, so the gate's work is the
same and the lavapipe threads have the cores the second slot held; the
tests themselves take about 14 s more here (on CI's 4 vCPU the CPU
variant took 58 s in run 40224280502). Neither becomes the run's long
pole (CI: `weather_presets_match_their_references` 124 s, og-cook's
`every_thread_count_cooks_the_same_bytes` 102 s).

## The backtrace

`backtrace-gpu-test.txt`: the real test (old shape, unchanged binary) on
CI's loader, both threads that touch the loader, symbolized with
libvulkan1-dbgsym. Thread 3 is the spawned second run (effects.rs:145),
thread 2 the test's own (effects.rs:146). Every crash, classified by its
faulting frame and the other thread's:

| Crashes | Faulting thread | The other renderer's thread |
|---|---|---|
| 17 | `loader_get_icd_and_device` ← `terminator_SetDebugUtilsObjectNameEXT` ← wgpu-hal `create_sampler` (og-render lib.rs:2562) | `dlclose` ← `unload_drivers_without_physical_devices` ← `vkEnumeratePhysicalDevices` ← wgpu `request_adapter` (og-render lib.rs:2241) |
| 1 | the same | the same, read from addresses before the debug symbols were installed (loader+0x2d01f = `loader_get_icd_and_device`+111, loader+0x39bd7 = `unload_drivers_without_physical_devices`+679) |
| 1 | the same | `terminator_CreateDevice` ← `vkCreateDevice` (a second writer of the lists the walk reads) |

None faulted in lavapipe, llvmpipe, wgpu-core or og-render: the
lavapipe and wgpu threads were idle in `pthread_cond_wait` in every one.
og-render's particle path has no `unsafe` (the crate's only one is the
pipeline cache, which og-tests never opens: no `set_pipeline_cache_dir`).

## What the Mac lane owes

Nothing. Metal has no Vulkan loader, no frame moves (the test compares its
runs with each other and saves its frame to the target directory, not to
a reference), and no reference is re-recorded.
