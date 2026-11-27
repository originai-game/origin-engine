//! Native fields, using the same bounded binary codec as skeletal assets.
use super::*;
use bincode::Options;

impl Behaviour {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(self)
            .map_err(|e| format!("behaviour: {e}"))
    }
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let behaviour: Self = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(bytes.len() as u64)
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(|e| format!("behaviour: {e}"))?;
        behaviour.validate()?;
        Ok(behaviour)
    }
}

pub(super) mod vector {
    use super::*;
    pub fn serialize<S: serde::Serializer>(v: &Vec3, s: S) -> Result<S::Ok, S::Error> {
        v.to_array().serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(d: D) -> Result<Vec3, D::Error> {
        <[f32; 3]>::deserialize(d).map(Vec3::from_array)
    }
}
pub(super) mod routes {
    use super::*;
    pub fn serialize<S: serde::Serializer>(
        routes: &[(String, Vec<Vec3>)],
        s: S,
    ) -> Result<S::Ok, S::Error> {
        routes
            .iter()
            .map(|(n, p)| (n, p.iter().map(|v| v.to_array()).collect::<Vec<_>>()))
            .collect::<Vec<_>>()
            .serialize(s)
    }
    pub fn deserialize<'de, D: serde::Deserializer<'de>>(
        d: D,
    ) -> Result<Vec<(String, Vec<Vec3>)>, D::Error> {
        Vec::<(String, Vec<[f32; 3]>)>::deserialize(d).map(|routes| {
            routes
                .into_iter()
                .map(|(n, p)| (n, p.into_iter().map(Vec3::from_array).collect()))
                .collect()
        })
    }
}
