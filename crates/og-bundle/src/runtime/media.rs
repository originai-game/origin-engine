//! Cooked textures, sounds and fonts.

use super::*;

/// A cooked texture: its metadata and KTX2 file.
#[derive(Clone, Debug, PartialEq)]
pub struct CookedTexture {
    pub id: String,
    pub meta: TextureMeta,
    pub ktx2: Vec<u8>,
}

impl CookedTexture {
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
        let (width, height, rgba) = match self.meta.codec {
            codec::RGBA8 => ktx2::decode_rgba8(&self.ktx2, 0).map_err(bad)?,
            _ => self.transcode_basis().map_err(bad)?,
        };
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
    pub fn target_format(&self, formats: Option<&TextureFormats>) -> TextureFormat {
        let Some(f) = formats else {
            return TextureFormat::Rgba8;
        };
        let lossless = self.meta.codec == codec::RGBA8;
        let blocks = self.meta.width.is_multiple_of(4) && self.meta.height.is_multiple_of(4);
        if lossless || !blocks {
            return TextureFormat::Rgba8;
        }
        match self.meta.kind {
            // A two-channel format takes y from alpha: cooks before phase 2
            // left alpha opaque there, so their maps go as colour.
            crate::format::kind::NORMAL if f.normal.two_channel() && self.meta.has_alpha == 0 => {
                f.color
            }
            crate::format::kind::NORMAL => f.normal,
            crate::format::kind::LINEAR => f.data,
            crate::format::kind::COLOR => f.color,
            _ => TextureFormat::Rgba8,
        }
    }

    /// The texture for og-render on a device that takes `formats`
    /// (`RenderInfo::texture_formats`): every cooked mip level transcoded
    /// to [`CookedTexture::target_format`] (BC7 and BC5 on desktop GPUs,
    /// ASTC 4x4 on Apple and most Android, ETC2 and EAC elsewhere), or
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
        let color_space = if self.meta.color_space == 0 && !target.two_channel() {
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
            TextureFormat::Bc5 => T::BC5_RG,
            TextureFormat::Astc4x4 => T::ASTC_4x4_RGBA,
            TextureFormat::Etc2Rgba8 => T::ETC2_RGBA,
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
    Ok((meta.chunk, blob.raw("qoa")?.to_vec()))
}

/// A video stream blob (`<id>#stream`) read from its file's bytes without a
/// bundle (they land after `og.load.video`, fetched on the web), its type
/// and cook `stamp` checked.
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
    let rows = table::<format::VideoUnitRow>(&blob, "units")?;
    let data = blob.raw("data")?;
    if rows.len() != meta.frames as usize || rows.is_empty() || rows[0].key == 0 {
        return Err(invalid(
            &blob,
            "a stream's units disagree with its meta row, or it starts on no key frame",
        ));
    }
    let units = rows
        .iter()
        .map(|r| {
            let end = r.offset as usize + r.size as usize;
            data.get(r.offset as usize..end)
                .map(|d| (r.key != 0, d.to_vec()))
                .ok_or_else(|| invalid(&blob, "a unit past the stream's data"))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(CookedVideoStream {
        id: asset_id(id).to_string(),
        meta,
        config: blob.raw("config")?.to_vec(),
        units,
    })
}

/// A cooked video stream ([`Bundle::video_stream`]): its shape, its codec's
/// configuration and each frame's unit (whether it is a key frame, its bytes).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedVideoStream {
    pub id: String,
    pub meta: crate::format::VideoStreamMeta,
    pub config: Vec<u8>,
    pub units: Vec<(bool, Vec<u8>)>,
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
        formats: Option<&og_render::TextureFormats>,
    ) -> Result<og_render::TextureArrayData, LoadError> {
        let mut layers = Vec::with_capacity(self.layers.len());
        for l in &self.layers {
            let t = bundle.load_texture(l)?;
            layers.push(t.to_texture_source(formats)?);
        }
        Ok(og_render::TextureArrayData { layers })
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

/// A cooked font: metrics, glyphs sorted by code point, atlas pages (RGBA8,
/// linear; RGB is the MSDF, A the true SDF).
#[derive(Clone, Debug, PartialEq)]
pub struct CookedFont {
    pub id: String,
    pub meta: FontMeta,
    pub glyphs: Vec<GlyphRow>,
    /// Kerning pairs between cooked characters, sorted by (left, right).
    pub kerning: Vec<KernRow>,
    pub pages: Vec<TextureData>,
    /// The font file, for a font cooked with a shaped script (og-ui shapes
    /// its words) or with `runtime true` (characters outside the charset
    /// are rendered from it when a text needs them).
    pub ttf: Option<Vec<u8>>,
    /// An icon set's names (an `.ogt` of `type icons`, cook::icons): icon
    /// `i` is the glyph at U+E000 + i. Empty for a text font.
    pub names: Vec<String>,
}

impl CookedFont {
    pub fn glyph(&self, c: char) -> Option<&GlyphRow> {
        self.glyphs
            .binary_search_by_key(&(c as u32), |g| g.codepoint)
            .ok()
            .map(|i| &self.glyphs[i])
    }
}
