//! JSON choices are pack-owned data; colours use the style's hex notation.
use super::{Scheme, Srgba};
use serde::{Deserialize, Deserializer, Serialize, Serializer, de::Error};

pub mod color {
    use super::*;
    pub fn serialize<S: Serializer>(value: &Srgba, s: S) -> Result<S::Ok, S::Error> {
        // Keep alpha as a float: converting it to a hex byte loses authored precision.
        (
            crate::tokens::hex(Srgba {
                alpha: 1.0,
                ..*value
            }),
            value.alpha,
        )
            .serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Srgba, D::Error> {
        let (hex, alpha) = <(String, f32)>::deserialize(d)?;
        let mut color = crate::style_file::parse_color(&hex)
            .ok_or_else(|| D::Error::custom("invalid colour"))?;
        if !(0.0..=1.0).contains(&alpha) {
            return Err(D::Error::custom("alpha must be 0..1"));
        }
        color.alpha = alpha;
        Ok(color)
    }
}

pub mod optional_color {
    use super::*;
    #[derive(Serialize, Deserialize)]
    struct Color(#[serde(with = "color")] Srgba);
    pub fn serialize<S: Serializer>(value: &Option<Srgba>, s: S) -> Result<S::Ok, S::Error> {
        value.map(Color).serialize(s)
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Option<Srgba>, D::Error> {
        Ok(Option::<Color>::deserialize(d)?.map(|c| c.0))
    }
}

pub mod scheme {
    use super::*;
    pub fn serialize<S: Serializer>(value: &Scheme, s: S) -> Result<S::Ok, S::Error> {
        s.serialize_str(if *value == Scheme::Dark {
            "dark"
        } else {
            "light"
        })
    }
    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Scheme, D::Error> {
        match String::deserialize(d)?.as_str() {
            "dark" => Ok(Scheme::Dark),
            "light" => Ok(Scheme::Light),
            _ => Err(D::Error::custom("expected dark or light")),
        }
    }
}
