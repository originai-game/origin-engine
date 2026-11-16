# shellcheck shell=bash
# L4 pins shared by landing shells and the explicitly started milestone.
# Unlike the retired /home/user/gpu/env.sh, this sets no gate overrides.
export PATH="$HOME/.cargo/bin:$HOME/.local/bin:$PATH"
export RUSTC_WRAPPER='' OG_GFX_BACKEND=vulkan
export VK_ICD_FILENAMES=/usr/share/vulkan/icd.d/nvidia_icd.json
export XDG_RUNTIME_DIR=/tmp
