pub const PREFILTER_MIPS: u32 = 5;

/// The sheen lobe (Estevez and Kulla's Charlie), shared with the physical
/// lit shader: the table's blue channel is its directional albedo.
pub const SHEEN_WGSL: &str = wgsl!("sheen.wgsl");
