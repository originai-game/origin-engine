//! Native FX payloads. Handles in these fields index a typed, blob-local
//! resource table; they are never scene handles. Binding replaces every one
//! with the scene's handle. The enclosing blob versions this layout.
use super::{EmitterLinks, ParticleMesh};
use crate::effect::EffectLinks;
use bincode::Options;
use og_render_data as r;
use serde::{Deserialize, Serialize, de::DeserializeOwned};

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Resource {
    Texture(String),
    Array(String),
    Model(String),
    Builtin(String),
    Emitter(String),
    Effect(String),
    Shader(String),
}

impl Resource {
    /// Built-in geometry has no content dependency.
    pub fn asset(&self) -> Option<&str> {
        match self {
            Self::Builtin(_) => None,
            Self::Texture(id)
            | Self::Array(id)
            | Self::Model(id)
            | Self::Emitter(id)
            | Self::Effect(id)
            | Self::Shader(id) => Some(id),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Linked<T> {
    pub data: T,
    pub resources: Vec<Resource>,
}

pub type Emitter = Linked<r::ParticleEmitter>;
pub type Effect = Linked<r::EffectMaterial>;

pub(crate) fn encode<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .serialize(value)
        .map_err(|e| format!("native FX: {e}"))
}

pub(crate) fn decode<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    bincode::DefaultOptions::new()
        .with_fixint_encoding()
        .with_limit(bytes.len() as u64)
        .reject_trailing_bytes()
        .deserialize(bytes)
        .map_err(|e| format!("native FX: {e}"))
}

impl<T> Linked<T> {
    fn resource(&self, index: u32) -> Result<&Resource, String> {
        self.resources
            .get(index as usize)
            .ok_or_else(|| format!("FX resource {index} is out of range"))
    }

    fn texture(
        &self,
        h: Option<r::TextureHandle>,
        links: &mut dyn FnMut(&str) -> Result<Option<r::TextureHandle>, String>,
    ) -> Result<Option<r::TextureHandle>, String> {
        h.map(|h| match self.resource(h.0)? {
            Resource::Texture(id) => links(id),
            _ => Err("FX texture resource has the wrong kind".into()),
        })
        .transpose()
        .map(Option::flatten)
    }

    fn mesh(
        &self,
        h: r::MeshHandle,
        links: &mut EmitterLinks<'_>,
    ) -> Result<r::MeshHandle, String> {
        let mesh = match self.resource(h.0)? {
            Resource::Model(id) => ParticleMesh::Asset(id),
            Resource::Builtin(name) if super::PARTICLE_MESHES.contains(&name.as_str()) => {
                ParticleMesh::Builtin(name)
            }
            _ => return Err("FX mesh resource has the wrong kind".into()),
        };
        (links.mesh)(mesh).map(|h| h.unwrap_or(r::MeshHandle::from_bits(0)))
    }
}

impl Emitter {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        encode(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let value: Self = decode(bytes)?;
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), String> {
        self.bind(&mut EmitterLinks {
            texture: &mut |_| Ok(Some(r::TextureHandle(0))),
            array: &mut |_| Ok(Some(r::TextureArrayHandle(0))),
            mesh: &mut |_| Ok(Some(r::MeshHandle(0))),
            emitter: &mut |_| Ok(Some(r::EmitterHandle(0))),
            effect: &mut |_| Ok(Some(r::EffectMaterialHandle(0))),
        })
        .map(|_| ())
    }

    /// Compile native fields with resource-table indices instead of scene
    /// handles. Both the author cook and external importers use this boundary.
    pub fn capture(
        build: impl FnOnce(&mut EmitterLinks<'_>) -> Result<r::ParticleEmitter, String>,
    ) -> Result<Self, String> {
        let resources = std::cell::RefCell::new(Vec::new());
        let insert = |resource| {
            let mut rows = resources.borrow_mut();
            let index = rows.len() as u32;
            rows.push(resource);
            index
        };
        let data = build(&mut EmitterLinks {
            texture: &mut |id| Ok(Some(r::TextureHandle(insert(Resource::Texture(id.into()))))),
            array: &mut |id| {
                Ok(Some(r::TextureArrayHandle(insert(Resource::Array(
                    id.into(),
                )))))
            },
            mesh: &mut |mesh| {
                Ok(Some(r::MeshHandle(insert(match mesh {
                    ParticleMesh::Builtin(name) => Resource::Builtin(name.into()),
                    ParticleMesh::Asset(id) => Resource::Model(id.into()),
                }))))
            },
            emitter: &mut |id| Ok(Some(r::EmitterHandle(insert(Resource::Emitter(id.into()))))),
            effect: &mut |id| {
                Ok(Some(r::EffectMaterialHandle(insert(Resource::Effect(
                    id.into(),
                )))))
            },
        })?;
        data.validate()?;
        Ok(Self {
            data,
            resources: resources.into_inner(),
        })
    }

    pub fn bind(&self, links: &mut EmitterLinks<'_>) -> Result<r::ParticleEmitter, String> {
        self.data.validate()?;
        let mut em = self.data.clone();
        em.texture = self.texture(em.texture, links.texture)?;
        em.array = em
            .array
            .map(|h| match self.resource(h.0)? {
                Resource::Array(id) => (links.array)(id),
                _ => Err("FX array resource has the wrong kind".into()),
            })
            .transpose()?
            .flatten();
        if let r::EmitterShape::Mesh { mesh } = &mut em.shape {
            *mesh = self.mesh(*mesh, links)?;
        }
        if let r::ParticleRender::Mesh { mesh } = &mut em.render {
            *mesh = self.mesh(*mesh, links)?;
        }
        em.effect = em
            .effect
            .map(|h| match self.resource(h.0)? {
                Resource::Effect(id) => (links.effect)(id),
                _ => Err("FX effect resource has the wrong kind".into()),
            })
            .transpose()?
            .flatten();
        let mut subs = Vec::new();
        for s in &em.sub_emitters {
            let Resource::Emitter(id) = self.resource(s.emitter.0)? else {
                return Err("FX emitter resource has the wrong kind".into());
            };
            if let Some(emitter) = (links.emitter)(id)? {
                subs.push(r::SubEmitter { emitter, ..*s });
            }
        }
        em.sub_emitters = subs;
        em.validate()?;
        Ok(em)
    }
}

impl Effect {
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        self.validate()?;
        encode(self)
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let value: Self = decode(bytes)?;
        value.validate()?;
        Ok(value)
    }

    fn validate(&self) -> Result<(), String> {
        self.bind(&mut EffectLinks {
            texture: &mut |_| Ok(Some(r::TextureHandle(0))),
            shader: &mut |_| Ok(Some((r::ShaderHandle(0), Vec::new()))),
        })
        .map(|_| ())
    }

    /// Shader metadata fixes parameter order and defaults during the cook;
    /// the resulting slots are native fields, not names reparsed at runtime.
    pub fn capture(
        build: impl FnOnce(&mut EffectLinks<'_>) -> Result<r::EffectMaterial, String>,
        shader: &dyn Fn(&str) -> Result<Vec<r::ShaderParam>, String>,
    ) -> Result<Self, String> {
        let resources = std::cell::RefCell::new(Vec::new());
        let insert = |resource| {
            let mut rows = resources.borrow_mut();
            let index = rows.len() as u32;
            rows.push(resource);
            index
        };
        let data = build(&mut EffectLinks {
            texture: &mut |id| Ok(Some(r::TextureHandle(insert(Resource::Texture(id.into()))))),
            shader: &mut |id| {
                Ok(Some((
                    r::ShaderHandle(insert(Resource::Shader(id.into()))),
                    shader(id)?,
                )))
            },
        })?;
        data.validate()?;
        Ok(Self {
            data,
            resources: resources.into_inner(),
        })
    }

    pub fn bind(&self, links: &mut EffectLinks<'_>) -> Result<r::EffectMaterial, String> {
        self.data.validate()?;
        let mut m = self.data.clone();
        m.main.texture = self.texture(m.main.texture, links.texture)?;
        if let Some(slot) = &mut m.mask {
            slot.texture = self.texture(slot.texture, links.texture)?;
        }
        if let Some(erosion) = &mut m.erosion {
            erosion.noise.texture = self.texture(erosion.noise.texture, links.texture)?;
        }
        if let Some(flow) = &mut m.flow {
            flow.map.texture = self.texture(flow.map.texture, links.texture)?;
        }
        m.shader = m
            .shader
            .map(|h| match self.resource(h.0)? {
                Resource::Shader(id) => (links.shader)(id).map(|v| v.map(|v| v.0)),
                _ => Err("FX shader resource has the wrong kind".into()),
            })
            .transpose()?
            .flatten();
        m.validate()?;
        Ok(m)
    }
}
