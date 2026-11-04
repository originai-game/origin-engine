# FidelityFX LPM HDR display mapper

Source: AMD GPUOpen, https://github.com/GPUOpen-Effects/FidelityFX-LPM,
revision `ed6ecd5b8963d2ec24603809b90ecfa00a1c3614`.
`ffx_lpm.h` is retained verbatim, with its MIT copyright/permission notice.
The port is `crates/og-render/shaders/hdr_lpm.wgsl`.

We specialize the full-precision Rec.709/scRGB path: working/output primaries
are Rec.709, no soft gamut conversion or secondary matrix, contrast,
shoulder and saturation exponents one, crosstalk (1, 0.5, 1/32).
The luma/ratio reconstruction and two crosstalk compensation steps are
LpmMap's. CPU calibration solves LpmSetup's rational curve for scene white
1 at paper white and scene maximum 256 at peak, rather than its fixed
18%-of-peak mid-grey. Equal paper white and peak uses a clipped linear
SDR range, avoiding the degenerate rational curve. Black returns black.

Output remains linear sRGB in paper-white units through post and UI.
The final pass scales to DXGI/Vulkan scRGB's 80-nit units or relative EDR white,
then encodes extended sRGB on WebGPU. No PQ or wide-gamut primaries are
claimed. Surface-path support and requested/resolved mode are distinct;
neither measures the display's actual luminance. HDR PNG captures are
explicitly clipped SDR previews, never HDR acceptance evidence.
