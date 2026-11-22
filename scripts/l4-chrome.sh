#!/bin/bash
# Chrome for the L4's web-compat runs (scripts/final-l4-pass.sh web-compat), passed to the web
# harnesses as --chrome. It drops the GPU switches the harness's launch adds and puts in the L4
# set (the wgpu fork thread's): ANGLE on OG_L4_ANGLE (default vulkan) with WebGPU on Vulkan and no
# Vulkan surface, and with OG_L4_COMPAT=1 the opengles adapter, whose WebGPU is compatibility mode.
#
#   OG_L4_CHROME_BIN   the Chrome for Testing binary (required)
#   OG_L4_CHROME_LOG   where Chrome's stderr is appended (default /tmp/chrome-<pid>.log); the
#                      engine's "[og] WebGPU error:" lines land there
[ -n "${OG_L4_CHROME_BIN:-}" ] || { echo "l4-chrome.sh: set OG_L4_CHROME_BIN to Chrome" >&2; exit 2; }
args=()
for a in "$@"; do
  case "$a" in --use-vulkan=*|--use-angle=*|--enable-features=*|--use-webgpu-adapter=*) ;; *) args+=("$a") ;; esac
done
extra=(--use-angle="${OG_L4_ANGLE:-vulkan}" --enable-features=WebGPU,Vulkan --disable-vulkan-surface --enable-logging=stderr --no-sandbox)
[ "${OG_L4_COMPAT:-}" = 1 ] && extra+=(--use-webgpu-adapter=opengles)
# The NVIDIA ICD on the L4; a CPU host (the dry run) keeps its own.
[ -f /usr/share/vulkan/icd.d/nvidia_icd.json ] && export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json
exec "$OG_L4_CHROME_BIN" "${extra[@]}" "${args[@]}" 2>>"${OG_L4_CHROME_LOG:-/tmp/chrome-$$.log}"
