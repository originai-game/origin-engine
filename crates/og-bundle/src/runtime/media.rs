//! Cooked textures, sounds and fonts.

use super::*;

/// A cooked texture: its metadata and KTX2 file.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTexture {
    pub id: String,
    pub meta: TextureMeta,
    pub ktx2: Vec<u8>,
    /// A streamed texture's whole chain, when this blob holds only its mip
    /// tail (`texture { stream true }`, G3-3); [`mips_id`] holds every level.
    pub mips: Option<MipChain>,
    /// Its source's EXIF orientation (1 to 8, 1 upright): the texels are
    /// as stored, and a UI image that honours it turns them upright at
    /// draw time (og-render's `set_texture_orientation`, CSS
    /// `image-orientation`).
    pub orientation: u32,
}

/// The whole mip chain of a texture whose blob holds its tail: level 0's
/// size and the levels (the tail is the chain from level `levels - tail
/// levels` down, byte for byte).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MipChain {
    pub width: u32,
    pub height: u32,
    pub levels: u32,
}

/// The longest side a streamed texture's mip tail keeps (its first level
/// at or under it): what draws until the whole chain is wanted and read.
pub const MIP_TAIL: u32 = 256;

/// The output holding every level of streamed texture `id`.
pub fn mips_id(id: &str) -> String {
    format!("{id}#mips")
}

/// The id of a glTF texture cooked once per image and use, whichever models
/// name it (outside streaming groups and art contracts): its image's hash,
/// its kind, whether it streams its levels (a model's `stream_textures`)
/// and, for a cut-out (the base colour of an alpha-tested material, whose
/// mips keep its coverage), the least alpha byte its material keeps. Models
/// loaded at run time share it ([`SharedParts`](super::SharedParts)).
pub fn gltf_texture_id(image_hash: u64, kind: u32, streamed: bool, cutout: Option<u8>) -> String {
    let stream = if streamed { "-stream" } else { "" };
    let cut = cutout.map(|c| format!("-cutout{c}")).unwrap_or_default();
    format!("{SHARED_TEXTURES}{image_hash:016x}-{kind}{stream}{cut}#tex")
}

/// Whether `id` names a texture [`gltf_texture_id`] made: the same bytes
/// for every model naming it.
pub fn is_shared_texture(id: &str) -> bool {
    id.starts_with(SHARED_TEXTURES)
}

const SHARED_TEXTURES: &str = "_gltf/";

/// The output holding the parts og.Wind moves of 2D texture `id`
/// (`texture { sway ... }` in its import settings): a texture of the parts
/// packed side by side, with [`SwayParts`] in its `sway` section.
pub fn sway_id(id: &str) -> String {
    format!("{id}#sway")
}

/// A 2D texture cut into the parts the wind moves (og-cook's sway2d.rs):
/// drawn in order at rest they are the texture, texel for texel.
#[derive(Clone, Debug, PartialEq)]
pub struct SwayParts {
    /// The texture's size (texels): the canvas the parts lie on.
    pub size: [u32; 2],
    /// The parts, in the order they draw: what stands still first.
    pub parts: Vec<SwayPart>,
}

/// One part of [`SwayParts`].
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SwayPart {
    /// Where its top-left texel lies in the `#sway` texture, and its
    /// width and height (texels).
    pub atlas: [u32; 2],
    pub size: [u32; 2],
    /// Where its top-left texel lies on the canvas (from the top left).
    pub at: [u32; 2],
    /// Its centre on the canvas (texels, from the top left): where it
    /// reads the wind.
    pub centre: [f32; 2],
    /// How much of the plant's lean it takes (0: it stands still): a
    /// clump of leaves more the farther it is from the crown's centre, the
    /// crown's dark copy a little more than its loosest clump, a row of a
    /// bending trunk and what rides it by its height squared.
    pub lean: f32,
    /// How much of the gusts' loop it walks (0: none; the wood and the
    /// dark copy).
    pub flutter: f32,
}

impl SwayParts {
    /// Bytes of one part in the blob section.
    const PART: usize = 40;

    /// The blob section `sway`: the canvas's size and the count, then each
    /// part's six u32s and four f32s, little-endian.
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut b = Vec::with_capacity(12 + self.parts.len() * Self::PART);
        for v in [self.size[0], self.size[1], self.parts.len() as u32] {
            b.extend(v.to_le_bytes());
        }
        for p in &self.parts {
            for v in [p.atlas, p.size, p.at].as_flattened() {
                b.extend(v.to_le_bytes());
            }
            for v in [p.centre[0], p.centre[1], p.lean, p.flutter] {
                b.extend(v.to_le_bytes());
            }
        }
        b
    }

    pub fn from_bytes(b: &[u8]) -> Option<SwayParts> {
        let word = |i: usize| -> Option<[u8; 4]> { b.get(i..i + 4)?.try_into().ok() };
        let u = |i: usize| word(i).map(u32::from_le_bytes);
        let f = |i: usize| word(i).map(f32::from_le_bytes);
        let count = u(8)? as usize;
        (b.len() == 12 + count.checked_mul(Self::PART)?).then_some(())?;
        let parts = (0..count)
            .map(|k| {
                let at = 12 + k * Self::PART;
                Some(SwayPart {
                    atlas: [u(at)?, u(at + 4)?],
                    size: [u(at + 8)?, u(at + 12)?],
                    at: [u(at + 16)?, u(at + 20)?],
                    centre: [f(at + 24)?, f(at + 28)?],
                    lean: f(at + 32)?,
                    flutter: f(at + 36)?,
                })
            })
            .collect::<Option<Vec<_>>>()?;
        Some(SwayParts {
            size: [u(0)?, u(4)?],
            parts,
        })
    }
}

impl MipChain {
    /// The blob section `mips` (three little-endian u32s), as the cook
    /// writes it.
    pub fn to_bytes(self) -> Vec<u8> {
        [self.width, self.height, self.levels]
            .iter()
            .flat_map(|v| v.to_le_bytes())
            .collect()
    }

    pub fn from_bytes(b: &[u8]) -> Option<MipChain> {
        let v = |i: usize| {
            Some(u32::from_le_bytes(
                b.get(i * 4..i * 4 + 4)?.try_into().ok()?,
            ))
        };
        (b.len() == 12).then_some(())?;
        Some(MipChain {
            width: v(0)?,
            height: v(1)?,
            levels: v(2)?,
        })
    }
}

impl CookedTexture {
    /// ETC1S normal-mode texels store X in RGB and Y in alpha.
    pub fn normal_xyra(&self) -> bool {
        self.meta.kind == crate::format::kind::NORMAL
            && self.meta.codec == codec::ETC1S
            && self.meta.has_alpha != 0
    }

    /// A `kind pixel` texture: drawn without mips and sampled nearest.
    pub fn is_pixel_art(&self) -> bool {
        self.meta.kind == crate::format::kind::PIXEL
    }

    /// Level 0 as RGBA8 for og-render (which uploads RGBA8 and builds its own
    /// mips today). RGBA8 textures decode everywhere; Basis Universal ones
    /// need the `transcode` feature (native) and are refused without it.
    pub fn to_texture_data(&self) -> Result<TextureData, LoadError> {
        let bad = |reason: String| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        };
        let color_space = if self.meta.color_space == 0 {
            ColorSpace::Srgb
        } else {
            ColorSpace::Linear
        };
        let (width, height, mut rgba) = match self.meta.codec {
            codec::RGBA8 => ktx2::decode_rgba8(&self.ktx2, 0).map_err(bad)?,
            _ => self.transcode_basis().map_err(bad)?,
        };
        if self.normal_xyra() {
            decode_normal_xyra(&mut rgba);
        }
        if self.is_pixel_art() {
            return Ok(TextureData {
                width,
                height,
                rgba,
                color_space,
                mips: Mips::None,
            });
        }
        Ok(TextureData {
            width,
            height,
            rgba,
            color_space,
            mips: Mips::Generate,
        })
    }

    /// The encoding this texture should reach a device that takes `formats`
    /// in: its kind's (normal maps `normal`, or `color` for a map cooked
    /// without y in alpha, linear data `data`, colour `color`), or RGBA8 for pixel art, distance fields, lossless cooks, a
    /// size not in whole 4x4 blocks, or no renderer yet (`None`).
    /// The key its prepared copy is kept under, on every host (og-runtime's
    /// native cache, og-host-web's): a hash of its bytes, its metadata and
    /// the encoding it is prepared to ([`Self::target_format`]), beside its
    /// id. Two bundles' textures of one id never share an entry, and a
    /// device that takes other encodings asks for its own.
    pub fn prepared_key(&self, formats: Option<&TextureFormats>) -> u64 {
        use std::hash::{Hash, Hasher};
        let mut hash = std::collections::hash_map::DefaultHasher::new();
        self.ktx2.hash(&mut hash);
        self.target_format(formats).hash(&mut hash);
        let m = &self.meta;
        [
            m.width,
            m.height,
            m.levels,
            m.kind,
            m.codec,
            m.color_space,
            m.has_alpha,
            m.supercompression,
        ]
        .hash(&mut hash);
        hash.finish()
    }

    pub fn target_format(&self, formats: Option<&TextureFormats>) -> TextureFormat {
        let Some(f) = formats else {
            return TextureFormat::Rgba8;
        };
        let lossless = self.meta.codec == codec::RGBA8;
        let blocks = self.meta.width.is_multiple_of(4) && self.meta.height.is_multiple_of(4);
        if lossless || !blocks {
            return TextureFormat::Rgba8;
        }
        if self.normal_xyra() {
            // BC5/EAC move alpha to green; their sampler rebuilds Z. A
            // device without either needs decoded XYZ, not xxx/y in ASTC.
            return [f.normal, TextureFormat::EacRg11, TextureFormat::Bc5]
                .into_iter()
                .find(|t| t.two_channel() && f.supported.contains(t))
                .unwrap_or(TextureFormat::Rgba8);
        }
        match self.meta.kind {
            // A two-channel format takes y from alpha: cooks before phase 2
            // left alpha opaque there, so their maps go as colour.
            crate::format::kind::NORMAL if f.normal.two_channel() && self.meta.has_alpha == 0 => {
                f.color
            }
            crate::format::kind::NORMAL => f.normal,
            crate::format::kind::SINGLE => f.single,
            crate::format::kind::LINEAR => f.data,
            crate::format::kind::COLOR
                if self.meta.has_alpha == 0 && self.meta.codec == codec::ETC1S =>
            {
                f.color_opaque
            }
            crate::format::kind::COLOR => f.color,
            _ => TextureFormat::Rgba8,
        }
    }

    /// The texture for og-render on a device that takes `formats`
    /// (`RenderInfo::texture_formats`): every cooked mip level transcoded
    /// to [`CookedTexture::target_format`] (BC7/BC1/BC4/BC5 on desktop GPUs,
    /// ASTC 4x4 on Apple and most Android, ETC2/EAC elsewhere), or
    /// RGBA8 level 0 as [`CookedTexture::to_texture_data`] gives it when the
    /// target is RGBA8. Native transcoding needs the `transcode` feature.
    pub fn to_texture_source(
        &self,
        formats: Option<&TextureFormats>,
    ) -> Result<TextureSource, LoadError> {
        let target = self.target_format(formats);
        if target == TextureFormat::Rgba8 {
            return self.to_texture_data().map(TextureSource::Rgba);
        }
        let bad = |reason: String| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        };
        let levels = self.transcode_levels(target).map_err(bad)?;
        let color_space =
            if self.meta.color_space == 0 && !target.two_channel() && !target.single_channel() {
                ColorSpace::Srgb
            } else {
                ColorSpace::Linear
            };
        let upload = TextureUpload {
            desc: TextureDesc {
                width: self.meta.width,
                height: self.meta.height,
                format: target,
                mip_levels: levels.len() as u32,
                color_space,
            },
            levels,
        };
        upload.desc.validate(&upload.levels).map_err(bad)?;
        Ok(TextureSource::Encoded(upload))
    }

    /// Every mip level transcoded to `format` (a block format).
    #[cfg(feature = "transcode")]
    fn transcode_levels(&self, format: TextureFormat) -> Result<Vec<Vec<u8>>, String> {
        use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat as T};
        let to = match format {
            TextureFormat::Bc7 => T::BC7_RGBA,
            TextureFormat::Bc1 => T::BC1_RGB,
            TextureFormat::Bc4 => T::BC4_R,
            TextureFormat::Bc5 => T::BC5_RG,
            TextureFormat::Astc4x4 => T::ASTC_4x4_RGBA,
            TextureFormat::Etc2Rgba8 => T::ETC2_RGBA,
            TextureFormat::Etc2Rgb8 => T::ETC1_RGB,
            TextureFormat::EacR11 => T::ETC2_EAC_R11,
            TextureFormat::EacRg11 => T::ETC2_EAC_RG11,
            TextureFormat::Rgba8 => T::RGBA32,
        };
        let basis = ktx2::to_basis(&self.ktx2)?;
        let mut t = Transcoder::new();
        t.prepare_transcoding(&basis)
            .map_err(|_| "basisu refused the texture".to_string())?;
        let count = t.image_level_count(&basis, 0);
        let mut levels = Vec::with_capacity(count as usize);
        for level_index in 0..count {
            let data = t
                .transcode_image_level(
                    &basis,
                    to,
                    TranscodeParameters {
                        image_index: 0,
                        level_index,
                        ..Default::default()
                    },
                )
                .map_err(|e| format!("basisu transcode of level {level_index} failed: {e:?}"))?;
            levels.push(data);
        }
        t.end_transcoding();
        Ok(levels)
    }

    #[cfg(not(feature = "transcode"))]
    fn transcode_levels(&self, format: TextureFormat) -> Result<Vec<Vec<u8>>, String> {
        Err(format!(
            "transcoding to {format:?} needs og-cook's `transcode` feature (native) or the web host's transcoder"
        ))
    }

    #[cfg(feature = "transcode")]
    fn transcode_basis(&self) -> Result<(u32, u32, Vec<u8>), String> {
        use basis_universal::{TranscodeParameters, Transcoder, TranscoderTextureFormat};
        let basis = ktx2::to_basis(&self.ktx2)?;
        let mut t = Transcoder::new();
        t.prepare_transcoding(&basis)
            .map_err(|_| "basisu refused the texture".to_string())?;
        let rgba = t
            .transcode_image_level(
                &basis,
                TranscoderTextureFormat::RGBA32,
                TranscodeParameters {
                    image_index: 0,
                    level_index: 0,
                    ..Default::default()
                },
            )
            .map_err(|e| format!("basisu transcode failed: {e:?}"))?;
        t.end_transcoding();
        Ok((self.meta.width, self.meta.height, rgba))
    }

    #[cfg(not(feature = "transcode"))]
    fn transcode_basis(&self) -> Result<(u32, u32, Vec<u8>), String> {
        Err("a Basis Universal texture needs a transcoder: the host's (web) or og-cook's `transcode` feature (native)".into())
    }
}

/// Restore the XYZ channels of an ETC1S normal-mode RGBA decode. Native
/// fallback uploads and the web host use the same reconstruction.
pub fn decode_normal_xyra(rgba: &mut [u8]) {
    for p in rgba.as_chunks_mut::<4>().0 {
        let x = f32::from(p[0]) / 127.5 - 1.0;
        let y = f32::from(p[3]) / 127.5 - 1.0;
        p[1] = p[3];
        p[2] = ((1.0 - x * x - y * y).max(0.0).sqrt() * 127.5 + 127.5).round() as u8;
    }
}

/// A cooked sound.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedSound {
    pub id: String,
    pub meta: SoundMeta,
    /// The QOA file (`meta.codec` QOA; empty for Opus); of a sound cooked
    /// in chunks (`meta.chunk_count` > 0), its header and first chunk only.
    pub qoa: Vec<u8>,
    /// The Opus sound (`meta.codec` Opus, `crate::opus`; empty for QOA).
    pub opus: Vec<u8>,
}

/// A sound chunk blob's index and QOA frames, read from its file's bytes
/// without a bundle (a mixer thread reading a chunk the moment a voice
/// needs it), its type and cook `stamp` checked.
pub fn sound_chunk_of(bytes: &[u8], path: &str, stamp: u64) -> Result<(u32, Vec<u8>), LoadError> {
    let aligned = og_blob::AlignedBytes::from_slice(bytes);
    let expect = Expect {
        type_hash: format::SoundChunk::type_hash(),
        schema_version: format::SoundChunk::schema_version(),
        stamp: Some(stamp),
    };
    let blob = Blob::open(aligned.as_slice(), path, &expect)?;
    let meta = one::<format::SoundChunkMeta>(&blob)?;
    let data = blob.raw("opus").or_else(|_| blob.raw("qoa"))?;
    Ok((meta.chunk, data.to_vec()))
}

/// A video stream's index (`<id>#stream`) read from its file's bytes
/// without a bundle (they land after a player first wants a frame, fetched
/// on the web), its type and cook `stamp` checked: its shape, its codec's
/// configuration and each frame's place in its chunk. Every chunk starts on
/// a key frame.
pub fn video_stream_of(
    bytes: &[u8],
    path: &str,
    stamp: u64,
    id: &str,
) -> Result<CookedVideoStream, LoadError> {
    let aligned = og_blob::AlignedBytes::from_slice(bytes);
    let expect = Expect {
        type_hash: format::VideoStream::type_hash(),
        schema_version: format::VideoStream::schema_version(),
        stamp: Some(stamp),
    };
    let blob = Blob::open(aligned.as_slice(), path, &expect)?;
    let meta = one::<format::VideoStreamMeta>(&blob)?;
    let units = table::<format::VideoUnitRow>(&blob, "units")?.to_vec();
    let chunked = meta.chunk_frames > 0
        && meta.chunks == meta.frames.div_ceil(meta.chunk_frames)
        && (0..meta.chunks).all(|n| {
            units
                .get((n * meta.chunk_frames) as usize)
                .is_some_and(|u| u.key != 0)
        });
    if units.len() != meta.frames as usize || units.is_empty() || !chunked {
        return Err(invalid(
            &blob,
            "a stream's units disagree with its meta row, or a chunk starts on no key frame",
        ));
    }
    Ok(CookedVideoStream {
        id: asset_id(id).to_string(),
        meta,
        config: blob.raw("config")?.to_vec(),
        units,
    })
}

/// A cooked video stream's index ([`Bundle::video_stream`]): its shape,
/// its codec's configuration and each frame's unit (its place in its
/// chunk, whether it is a key frame).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedVideoStream {
    pub id: String,
    pub meta: crate::format::VideoStreamMeta,
    pub config: Vec<u8>,
    pub units: Vec<crate::format::VideoUnitRow>,
}

impl CookedVideoStream {
    /// The units of frames `first .. first + frames` of chunk `data`, each
    /// with whether it is a key frame; refused past the chunk's bytes.
    pub fn split<'a>(&self, chunk: &'a CookedVideoChunk) -> Result<Vec<(bool, &'a [u8])>, String> {
        let rows = self
            .units
            .get(chunk.first as usize..(chunk.first + chunk.frames) as usize)
            .ok_or_else(|| format!("{}: chunk {} is past its frames", self.id, chunk.chunk))?;
        rows.iter()
            .map(|r| {
                chunk
                    .data
                    .get(r.offset as usize..r.offset as usize + r.size as usize)
                    .map(|d| (r.key != 0, d))
                    .ok_or_else(|| format!("{}: a unit past chunk {}'s data", self.id, chunk.chunk))
            })
            .collect()
    }
}

/// A video chunk blob (`<stream id>.<n>`) read from its file's bytes
/// without a bundle, its type and cook `stamp` checked.
pub fn video_chunk_of(bytes: &[u8], path: &str, stamp: u64) -> Result<CookedVideoChunk, LoadError> {
    let aligned = og_blob::AlignedBytes::from_slice(bytes);
    let expect = Expect {
        type_hash: format::VideoChunk::type_hash(),
        schema_version: format::VideoChunk::schema_version(),
        stamp: Some(stamp),
    };
    let blob = Blob::open(aligned.as_slice(), path, &expect)?;
    let meta = one::<format::VideoChunkMeta>(&blob)?;
    Ok(CookedVideoChunk {
        chunk: meta.chunk,
        first: meta.first,
        frames: meta.frames,
        data: blob.raw("data")?.to_vec(),
    })
}

/// A cooked video chunk: frames `first .. first + frames` of its stream.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedVideoChunk {
    pub chunk: u32,
    pub first: u32,
    pub frames: u32,
    pub data: Vec<u8>,
}

/// The id of chunk `n` of a video stream (`base/clips/intro#stream.3`, the
/// file `base/clips/intro.stream.3.ogb`).
pub fn video_chunk_id(stream: &str, n: u32) -> String {
    format!("{stream}.{n}")
}

/// A cooked texture array: its shape and its layers' texture ids, in
/// layer order ([`Bundle::texture_array`]).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTextureArray {
    pub id: String,
    pub meta: format::TextureArrayMeta,
    pub layers: Vec<String>,
}

impl CookedTextureArray {
    /// The array og-render uploads from its layers' textures, each in the
    /// encoding `formats` takes (RGBA8 without a renderer; native only:
    /// the web host hands the layers over transcoded).
    #[cfg(feature = "transcode")]
    pub fn to_texture_array(
        &self,
        bundle: &Bundle,
        formats: Option<&og_render_data::TextureFormats>,
    ) -> Result<og_render_data::TextureArrayData, LoadError> {
        let mut layers = Vec::with_capacity(self.layers.len());
        for l in &self.layers {
            let t = bundle.load_texture(l)?;
            layers.push(t.to_texture_source(formats)?);
        }
        Ok(og_render_data::TextureArrayData { layers })
    }
}

/// The id of chunk `n` of a sound cooked in chunks (`audio/theme.mp3#chunk3`).
pub fn chunk_id(id: &str, n: u32) -> String {
    format!("{}#chunk{n}", asset_id(id))
}

impl CookedSound {
    /// Whether it was cooked in chunks (the others are blobs of their own).
    pub fn chunked(&self) -> bool {
        self.meta.chunk_count > 0
    }

    /// Whether it is an Opus sound (else QOA).
    pub fn is_opus(&self) -> bool {
        self.meta.codec == crate::format::sound_codec::OPUS
    }

    /// Interleaved 16-bit PCM, whichever its codec (of a sound cooked in
    /// chunks: refused, its samples are not all here).
    pub fn decode(&self) -> Result<qoa::Decoded, LoadError> {
        let bad = |reason: String| LoadError::Invalid {
            path: self.id.clone(),
            reason,
        };
        if self.chunked() {
            return Err(bad("cooked in chunks: decode it chunk by chunk".into()));
        }
        if self.is_opus() {
            let d = crate::opus::decode(&self.opus).map_err(bad)?;
            return Ok(qoa::Decoded {
                channels: d.channels,
                sample_rate: d.sample_rate,
                samples: d.samples,
            });
        }
        qoa::decode(&self.qoa).map_err(bad)
    }
}

/// A cooked font: its metrics and file; an icon set's glyphs, sorted by
/// code point, and distance-field pages (RGBA8, linear).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedFont {
    pub id: String,
    pub meta: FontMeta,
    pub glyphs: Vec<GlyphRow>,
    /// Kerning pairs between cooked characters, sorted by (left, right).
    pub kerning: Vec<KernRow>,
    pub pages: Vec<TextureData>,
    /// The font file, whole: what text is laid out and rastered from
    /// (`None` for an icon set, which draws from `pages`).
    pub file: Option<Vec<u8>>,
    /// An icon set's names (`file.icons`, cook::icons): icon
    /// `i` is the glyph at U+E000 + i. Empty for a text font.
    pub names: Vec<String>,
    /// An icon set's toned icons (its `tones` section, cook::icons): the
    /// sticker line, then each toned icon's glyphs and colour roles.
    /// Empty when no icon has tones.
    pub tones: String,
    /// A pixel font's grid, its pixels to the em (`font { pixel <grid> }`,
    /// the blob's `pixel` section); 0 for an outline font.
    pub pixel: u32,
}

impl CookedFont {
    pub fn glyph(&self, c: char) -> Option<&GlyphRow> {
        self.glyphs
            .binary_search_by_key(&(c as u32), |g| g.codepoint)
            .ok()
            .map(|i| &self.glyphs[i])
    }
}
