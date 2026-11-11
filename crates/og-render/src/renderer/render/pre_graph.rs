//! The passes the frame encodes before its render graph's: the compute
//! steps the graph's passes read (the crowds' cull, the wind, the
//! simulated meshes, the terrain's corners, the grass, the scatter). They
//! are not graph nodes. [`PRE_GRAPH`] lists them in the frame encoder's
//! order with what each needs; [`pre_graph`] encodes one where the device
//! has its needs, checked against them as it encodes.

use super::*;

/// What a pass before the graph needs of the device.
#[derive(Clone, Copy, Debug)]
pub(crate) struct PreGraphDesc {
    /// Its name, as [`Renderer::absent_passes`] reports it.
    pub name: &'static str,
    pub needs: Caps,
}

/// The wind field's and the trample map's steps (wind_field.rs,
/// trample.rs), before every pass that reads them.
pub(crate) const WIND: PreGraphDesc = PreGraphDesc {
    name: "wind",
    needs: Caps::COMPUTE,
};

/// The passes before the graph, in the frame encoder's order.
pub(crate) const PRE_GRAPH: &[PreGraphDesc] = &[WIND];

/// Encodes the pass `desc` with `encode` where `caps` has its needs, the
/// encoder checking what it opens against them (`Encoder::expect`), and
/// against the device alone again after it.
pub(super) fn pre_graph<R>(
    encoder: &mut Encoder,
    caps: Caps,
    desc: PreGraphDesc,
    encode: impl FnOnce(&mut Encoder) -> R,
) -> Option<R> {
    if !caps.contains(desc.needs) {
        return None;
    }
    encoder.expect(caps, desc.needs);
    let r = encode(encoder);
    encoder.expect(caps, caps);
    Some(r)
}
