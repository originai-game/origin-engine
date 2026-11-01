//! Animation blobs, skin streams and morph targets.

use super::*;

/// The animation blob id of a model id (`models/x.glb` → `models/x.glb#anim`).
pub fn anim_id(id: &str) -> String {
    let id = asset_id(id);
    if id.ends_with("#anim") {
        id.to_string()
    } else {
        format!("{id}#anim")
    }
}

/// The joints and weights of one skinned model primitive, per cooked vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct SkinStream {
    /// The primitive's index in the model blob ([`CookedModel::primitives`]).
    pub primitive: usize,
    pub mesh: usize,
    /// Indices into the skin's joints.
    pub joints: Vec<[u16; 4]>,
    /// Summing to 1 (cooked as unorm16 summing to 65535).
    pub weights: Vec<[f32; 4]>,
}

/// The morph targets of one model primitive, per cooked vertex.
#[derive(Clone, Debug, PartialEq)]
pub struct MorphTargets {
    pub primitive: usize,
    pub mesh: usize,
    /// Per target: its name (empty when unnamed) and default weight.
    pub names: Vec<String>,
    pub default_weights: Vec<f32>,
    /// Per target, per vertex: the position delta.
    pub positions: Vec<Vec<Vec3>>,
    /// Per target, per vertex: the normal delta, when every target has one.
    pub normals: Option<Vec<Vec<Vec3>>>,
}

/// A decoded animation blob.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedAnimation {
    pub id: String,
    pub skeleton: og_anim::SkeletonAsset,
    pub clips: Vec<og_anim::ClipAsset>,
    pub skin_streams: Vec<SkinStream>,
    pub morphs: Vec<MorphTargets>,
}

impl CookedAnimation {
    /// The clip called `name`.
    pub fn clip(&self, name: &str) -> Option<&og_anim::ClipAsset> {
        self.clips.iter().find(|c| c.name == name)
    }

    /// Decodes an animation blob from its bytes alone, without a manifest
    /// (the cook reads back the blobs it retargets from and onto).
    pub fn from_bytes(id: &str, bytes: &[u8]) -> Result<CookedAnimation, LoadError> {
        let aligned = og_blob::AlignedBytes::from_slice(bytes);
        let expect = Expect {
            type_hash: format::Animation::type_hash(),
            schema_version: format::Animation::schema_version(),
            stamp: None,
        };
        let blob = Blob::open(aligned.as_slice(), id, &expect)?;
        CookedAnimation::decode(asset_id(id), &blob)
    }

    pub(super) fn decode(id: &str, blob: &Blob) -> Result<CookedAnimation, LoadError> {
        let bad = |r: String| invalid(blob, r);
        let meta = one::<AnimMeta>(blob)?;
        let nodes = table::<NodeRow>(blob, "nodes")?;
        let hidden = blob.table::<u32>("hidden", 0)?.into_owned();
        let skins = table::<SkinRow>(blob, "skins")?;
        let joints = blob.table::<u32>("joints", 0)?.into_owned();
        let ibms = blob.table::<f32>("inverse_binds", 0)?.into_owned();
        let clips = table::<ClipRow>(blob, "clips")?;
        let channels = table::<ChannelRow>(blob, "channels")?;
        let samples = blob.table::<f32>("samples", 0)?.into_owned();
        let packed = blob.table::<u16>("packed", 0)?.into_owned();
        let events = table::<EventRow>(blob, "events")?;
        let streams = table::<SkinStreamRow>(blob, "skin_streams")?;
        let skin_data = blob.raw("skin_data")?;
        let morphs = table::<MorphRow>(blob, "morphs")?;
        let targets = table::<TargetRow>(blob, "targets")?;
        let morph_data = blob.raw("morph_data")?;
        let strings = blob.raw("strings")?;
        let s = |o: u32, l: u32| {
            pool_str(strings, o, l)
                .map(str::to_string)
                .ok_or_else(|| bad("string out of range".into()))
        };
        if nodes.len() != meta.node_count as usize
            || hidden.len() != meta.hidden_count as usize
            || clips.len() != meta.clip_count as usize
            || ibms.len() != joints.len() * 16
        {
            return Err(bad("counts disagree with the meta row".into()));
        }
        let mut skeleton = og_anim::SkeletonAsset {
            hidden,
            ..Default::default()
        };
        for n in &nodes {
            skeleton.nodes.push(og_anim::SkeletonNode {
                name: s(n.name_offset, n.name_len)?,
                parent: n.parent,
                rest: og_anim::Trs::new(
                    Vec3::new(n.tx, n.ty, n.tz),
                    Quat::from_xyzw(n.rx, n.ry, n.rz, n.rw),
                    Vec3::new(n.sx, n.sy, n.sz),
                ),
                mesh: n.mesh,
            });
        }
        for k in &skins {
            let r = k.joint_first as usize..(k.joint_first + k.joint_count) as usize;
            let js = joints
                .get(r.clone())
                .ok_or_else(|| bad("skin joints out of range".into()))?
                .to_vec();
            let inverse_bind = r
                .map(|j| Mat4::from_cols_slice(&ibms[j * 16..j * 16 + 16]))
                .collect();
            skeleton.skins.push(og_anim::Skin {
                name: s(k.name_offset, k.name_len)?,
                joints: js,
                inverse_bind,
            });
        }
        skeleton
            .validate()
            .map_err(|e| bad(e.reason().to_string()))?;
        let mut out_clips = Vec::with_capacity(clips.len());
        for c in &clips {
            let rows = channels
                .get(c.channel_first as usize..(c.channel_first + c.channel_count) as usize)
                .ok_or_else(|| bad("clip channels out of range".into()))?;
            let mut chs = Vec::with_capacity(rows.len());
            for ch in rows {
                let path = og_anim::ChannelPath::from_code(ch.path)
                    .ok_or_else(|| bad(format!("channel path {}", ch.path)))?;
                let data = crate::anim_codec::samples(ch, c.frames, &samples, &packed)
                    .map_err(|e| bad(format!("clip channel: {e}")))?;
                let node_name = skeleton
                    .nodes
                    .get(ch.node as usize)
                    .map(|n| n.name.clone())
                    .ok_or_else(|| bad("channel node out of range".into()))?;
                chs.push(og_anim::Channel {
                    node: ch.node,
                    node_name,
                    path,
                    step: ch.interpolation == format::interpolation::STEP,
                    lanes: ch.lanes,
                    samples: data,
                });
            }
            let evs = events
                .get(c.event_first as usize..(c.event_first + c.event_count) as usize)
                .ok_or_else(|| bad("clip events out of range".into()))?
                .iter()
                .map(|e| {
                    Ok(og_anim::ClipEvent {
                        name: s(e.name_offset, e.name_len)?,
                        time: e.time,
                    })
                })
                .collect::<Result<Vec<_>, LoadError>>()?;
            let clip = og_anim::ClipAsset {
                name: s(c.name_offset, c.name_len)?,
                duration: c.duration,
                rate: c.rate,
                source_rate: c.source_rate,
                frames: c.frames,
                channels: chs,
                events: evs,
            };
            clip.validate().map_err(|e| bad(e.reason().to_string()))?;
            out_clips.push(clip);
        }
        let slice = |data: &'_ [u8], o: u32, l: u32| -> Result<std::ops::Range<usize>, LoadError> {
            let (o, l) = (o as usize, l as usize);
            (o + l <= data.len())
                .then_some(o..o + l)
                .ok_or_else(|| bad("stream out of range".into()))
        };
        let mut skin_streams = Vec::with_capacity(streams.len());
        for st in &streams {
            let n = st.vertex_count as usize;
            let raw = meshopt_codec::decode_vertex_buffer(
                &skin_data[slice(skin_data, st.data_offset, st.data_size)?],
                n,
                16,
            )
            .map_err(|e| bad(format!("skin stream: {e}")))?;
            let u16s: Vec<u16> = raw
                .as_chunks::<2>()
                .0
                .iter()
                .map(|b| u16::from_le_bytes([b[0], b[1]]))
                .collect();
            let mut joints_out = Vec::with_capacity(n);
            let mut weights_out = Vec::with_capacity(n);
            for v in u16s.as_chunks::<8>().0 {
                joints_out.push([v[0], v[1], v[2], v[3]]);
                weights_out.push([
                    v[4] as f32 / 65535.0,
                    v[5] as f32 / 65535.0,
                    v[6] as f32 / 65535.0,
                    v[7] as f32 / 65535.0,
                ]);
            }
            skin_streams.push(SkinStream {
                primitive: st.primitive as usize,
                mesh: st.mesh as usize,
                joints: joints_out,
                weights: weights_out,
            });
        }
        let mut out_morphs = Vec::with_capacity(morphs.len());
        for m in &morphs {
            let (n, k, moved) = (
                m.vertex_count as usize,
                m.target_count as usize,
                m.moved as usize,
            );
            let stride = if m.normals != 0 { 24 } else { 12 };
            let decode = |o: u32, l: u32, stride: usize| -> Result<Vec<u8>, LoadError> {
                if moved == 0 {
                    return Ok(Vec::new());
                }
                meshopt_codec::decode_vertex_buffer(
                    &morph_data[slice(morph_data, o, l)?],
                    moved,
                    stride,
                )
                .map_err(|e| bad(format!("morph stream: {e}")))
            };
            let raw = decode(m.data_offset, m.data_size, stride)?;
            let index = decode(m.index_offset, m.index_size, 4)?;
            let f: Vec<f32> = raw
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| f32::from_le_bytes([b[0], b[1], b[2], b[3]]))
                .collect();
            let index: Vec<usize> = index
                .as_chunks::<4>()
                .0
                .iter()
                .map(|b| u32::from_le_bytes(*b) as usize)
                .collect();
            let lanes = stride / 4;
            let rows = targets
                .get(m.target_first as usize..m.target_first as usize + k)
                .ok_or_else(|| bad("morph targets out of range".into()))?;
            let mut positions = vec![vec![Vec3::ZERO; n]; k];
            let mut normals = (m.normals != 0).then(|| vec![vec![Vec3::ZERO; n]; k]);
            for (t, row) in rows.iter().enumerate() {
                let (first, count) = (row.moved_first as usize, row.moved_count as usize);
                if first + count > moved {
                    return Err(bad("morph target's moved vertices out of range".into()));
                }
                for (e, &v) in index.iter().enumerate().skip(first).take(count) {
                    if v >= n {
                        return Err(bad(format!("morph vertex {v} of {n}")));
                    }
                    let at = e * lanes;
                    positions[t][v] = Vec3::new(f[at], f[at + 1], f[at + 2]);
                    if let Some(normals) = normals.as_mut() {
                        normals[t][v] = Vec3::new(f[at + 3], f[at + 4], f[at + 5]);
                    }
                }
            }
            out_morphs.push(MorphTargets {
                primitive: m.primitive as usize,
                mesh: m.mesh as usize,
                names: rows
                    .iter()
                    .map(|t| s(t.name_offset, t.name_len))
                    .collect::<Result<_, _>>()?,
                default_weights: rows.iter().map(|t| t.default_weight).collect(),
                positions,
                normals,
            });
        }
        Ok(CookedAnimation {
            id: id.to_string(),
            skeleton,
            clips: out_clips,
            skin_streams,
            morphs: out_morphs,
        })
    }
}
