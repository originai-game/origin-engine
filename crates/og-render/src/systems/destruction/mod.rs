//! Destruction: the debris layer.

pub(crate) mod debris;

/// The system's modules for [`crate::shader_sources`]: a label and its WGSL.
pub(crate) fn shader_sources(add: &mut dyn FnMut(&str, String)) {
    add("debris", crate::debris::source());
}
