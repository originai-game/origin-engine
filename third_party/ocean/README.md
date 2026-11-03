# ocean

The FFT sea of og-render (`crates/og-render/src/ocean_fft/`,
`crates/og-render/shaders/ocean_fft*.wgsl`) ports code from two MIT-licensed
projects. Their licences are kept here.

- **web-ocean** by Maksymilian Dendura, https://github.com/Noniv/web-ocean at
  commit `a07c7fd522ef0a897601c114f927a49d93d95520`, `LICENSE-web-ocean`.
  From `js/wgsl/sim.js`: the workgroup-memory radix-2 inverse FFT (bit-reversed
  loads, butterflies), the packing of eight real fields into four complex
  signals, and the fields' layout and mips.
- **blue-water-sea-and-sky** by Bill Yang,
  https://github.com/goldflag/blue-water-sea-and-sky at commit
  `16c048adb8c817c786642c051425ea382f27990d`, `LICENSE-blue-water`. From
  `ocean/src/spectrum.rs`: the seeded CPU spectrum (JONSWAP held to the
  saturation range, Hasselmann's spreading, the counter-based PCG draws, the
  disjoint cascade bands, frequencies folded to 4096 s, the rescale to the
  significant height). From `ocean/shaders/fft.wgsl`: the spectrum's texel
  layout and the float32 phase split.

Each ported file names its source and licence in its header.
