//! Native rig fields, shared by cook and load. The enclosing blob versions
//! this layout; there is no author parser or old-layout fallback here.
use super::*;
use bincode::Options;

impl Rig2dAsset {
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        self.validate()?;
        bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .serialize(self)
            .map_err(|e| Error::new(ErrorKind::Parse, format!("2D rig: {e}")))
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self> {
        let rig: Self = bincode::DefaultOptions::new()
            .with_fixint_encoding()
            .with_limit(bytes.len() as u64)
            .reject_trailing_bytes()
            .deserialize(bytes)
            .map_err(|e| Error::new(ErrorKind::Parse, format!("2D rig: {e}")))?;
        rig.validate()?;
        Ok(rig)
    }
}
