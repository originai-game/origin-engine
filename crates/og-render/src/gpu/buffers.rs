//! Buffers that grow to what is written to them.

/// Replaces `buffer` with a larger one of the same usage when `len` bytes
/// do not fit; true when it did.
#[track_caller]
pub(crate) fn grow(
    device: &wgpu::Device,
    buffer: &mut wgpu::Buffer,
    label: &str,
    len: u64,
) -> bool {
    if len <= buffer.size() {
        return false;
    }
    *buffer = crate::traffic::create_buffer(
        device,
        &wgpu::BufferDescriptor {
            label: Some(label),
            size: len.next_power_of_two(),
            usage: buffer.usage(),
            mapped_at_creation: false,
        },
    );
    true
}

/// Writes `bytes` at the start of `buffer`, replacing it with a larger one first if needed.
#[track_caller]
pub(crate) fn grow_and_write(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    bytes: &[u8],
) {
    grow(device, buffer, label, bytes.len() as u64);
    if !bytes.is_empty() {
        crate::traffic::write_buffer(queue, buffer, 0, bytes);
    }
}

/// CPU bytes last uploaded to one retained stream. The buffer handle ties
/// the snapshot to its allocation, so even a same-sized replacement gets
/// fully initialized. Only this writer may change that buffer's contents.
#[derive(Default)]
pub(crate) struct RetainedUpload {
    buffer: Option<wgpu::Buffer>,
    /// Or the data texture it writes ([`RetainedUpload::write_data`]).
    texture: Option<wgpu::Texture>,
    bytes: Vec<u8>,
}

impl RetainedUpload {
    /// [`RetainedUpload::write`] into a data texture (data_texture.rs)
    /// of whole-texel elements, grown (twice the texels) when they do not
    /// fit; returns whether it was made again.
    #[track_caller]
    pub(crate) fn write_data<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        texture: &mut crate::gpu::data_texture::DataTexture,
        label: &str,
        values: &[T],
    ) -> bool {
        let bytes: &[u8] = bytemuck::cast_slice(values);
        let texels = (bytes.len() / 16) as u32;
        let grew = texels > texture.texels;
        if grew {
            *texture = crate::gpu::data_texture::DataTexture::new(
                device,
                label,
                texture.texture.format(),
                texels.next_power_of_two(),
                texture.texture.usage(),
            );
        }
        let previous =
            (self.texture.as_ref() == Some(&texture.texture)).then_some(self.bytes.as_slice());
        changed_ranges(previous, values, |range| {
            texture.write(queue, (range.start / 16) as u32, &bytes[range]);
        });
        if self.texture.as_ref() != Some(&texture.texture) {
            self.texture = Some(texture.texture.clone());
        }
        self.bytes.resize(bytes.len(), 0);
        self.bytes.copy_from_slice(bytes);
        grew
    }

    #[track_caller]
    pub(crate) fn write<T: bytemuck::Pod>(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        buffer: &mut wgpu::Buffer,
        label: &str,
        values: &[T],
    ) {
        let bytes = bytemuck::cast_slice(values);
        grow(device, buffer, label, bytes.len() as u64);
        let previous = (self.buffer.as_ref() == Some(buffer)).then_some(self.bytes.as_slice());
        changed_ranges(previous, values, |range| {
            crate::traffic::write_buffer(queue, buffer, range.start as u64, &bytes[range]);
        });
        if self.buffer.as_ref() != Some(buffer) {
            self.buffer = Some(buffer.clone());
        }
        self.bytes.resize(bytes.len(), 0);
        self.bytes.copy_from_slice(bytes);
    }
}

/// Visits byte ranges of maximal runs of changed whole stream elements.
/// Element sizes must be nonzero multiples of COPY_BUFFER_ALIGNMENT. A missing
/// snapshot means a new allocation, not an empty previous live prefix.
/// Truncated tails need no clearing: draws use live counts; regrown tails
/// are new bytes and must be uploaded even if old allocation bytes match.
pub(crate) fn changed_ranges<T: bytemuck::Pod>(
    previous: Option<&[u8]>,
    values: &[T],
    mut write: impl FnMut(std::ops::Range<usize>),
) {
    let alignment = wgpu::COPY_BUFFER_ALIGNMENT as usize;
    let stride = std::mem::size_of::<T>();
    assert!(stride > 0);
    assert_eq!(stride % alignment, 0);
    let bytes = bytemuck::cast_slice(values);
    let Some(previous) = previous else {
        if !bytes.is_empty() {
            write(0..bytes.len());
        }
        return;
    };
    let mut start = None;
    for offset in (0..bytes.len()).step_by(stride) {
        let end = offset + stride;
        if previous.get(offset..end) != Some(&bytes[offset..end]) {
            start.get_or_insert(offset);
        } else if let Some(begin) = start.take() {
            write(begin..offset);
        }
    }
    if let Some(begin) = start {
        write(begin..bytes.len());
    }
}
