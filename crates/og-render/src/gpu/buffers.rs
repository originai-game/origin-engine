//! Buffers that grow to what is written to them.

/// Replaces `buffer` with a larger one of the same usage when `len` bytes
/// do not fit; true when it did.
pub(crate) fn grow(
    device: &wgpu::Device,
    buffer: &mut wgpu::Buffer,
    label: &str,
    len: u64,
) -> bool {
    if len <= buffer.size() {
        return false;
    }
    *buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some(label),
        size: len.next_power_of_two(),
        usage: buffer.usage(),
        mapped_at_creation: false,
    });
    true
}

/// Writes `bytes` at the start of `buffer`, replacing it with a larger one first if needed.
pub(crate) fn grow_and_write(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    buffer: &mut wgpu::Buffer,
    label: &str,
    bytes: &[u8],
) {
    grow(device, buffer, label, bytes.len() as u64);
    if !bytes.is_empty() {
        queue.write_buffer(buffer, 0, bytes);
    }
}
