//! The codec physics state is saved and triangle meshes are baked with.

pub fn codec() -> impl bincode::Options {
    use bincode::Options;
    bincode::DefaultOptions::new().reject_trailing_bytes()
}

/// `value`'s bytes appended to `out` through [`codec`]. Everything a save
/// writes goes through this one encoder, so each type's serialisation is
/// compiled once: bincode's `serialize` and `serialized_size` would each
/// compile the whole rapier world's again (a size pass, a writer of
/// another type), tens of kilobytes of wasm (`the former test sample`).
pub fn encode<T: serde::Serialize + ?Sized>(out: &mut Vec<u8>, value: &T) -> bincode::Result<()> {
    use bincode::Options;
    codec().serialize_into(out, value)
}

/// A value [`encode`] wrote, alone in `bytes`.
#[cfg(feature = "physics3d")]
pub fn decode<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> bincode::Result<T> {
    use bincode::Options;
    codec().deserialize(bytes)
}
