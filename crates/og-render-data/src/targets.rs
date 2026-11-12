/// How a render target's colour is stored.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum TargetFormat {
    /// 8-bit sRGB: the view's finished frame (exposed, post-processed and
    /// tone mapped), as a loaded colour texture.
    #[default]
    Rgba8,
    /// 16-bit float, linear: the view's HDR radiance before exposure, post
    /// and tone mapping.
    Rgba16Float,
}

impl TargetFormat {
    /// Bytes per texel.
    pub fn bytes(self) -> u64 {
        match self {
            TargetFormat::Rgba8 => 4,
            TargetFormat::Rgba16Float => 8,
        }
    }
}

/// What a texture of the scene's table is when it is a render target's.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct TargetTexture {
    pub width: u32,
    pub height: u32,
    pub kind: TargetKind,
}

/// A target texture's role.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TargetKind {
    Color(TargetFormat),
    /// The view's normalised linear depth (Rgba16Float).
    Depth,
    /// A streamed video's frame (8-bit sRGB, [`RenderScene::add_stream_texture`]):
    /// written by the host as frames are decoded, never drawn into by a view.
    Stream,
}

impl TargetTexture {
    /// Bytes allocated on the GPU.
    pub fn bytes(self) -> u64 {
        let texel = match self.kind {
            TargetKind::Color(f) => f.bytes(),
            TargetKind::Depth => 8,
            TargetKind::Stream => 4,
        };
        self.width as u64 * self.height as u64 * texel
    }
}
