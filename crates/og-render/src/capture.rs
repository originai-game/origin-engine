//! Frame capture on both hosts: the offscreen target copied into a mapped
//! buffer. One async path: natively the call polls the device until the copy
//! is done; in the browser the mapping completes from the event loop (WebGPU's
//! `mapAsync`) or on the next submit or [`Renderer::poll`] (WebGL2), and the
//! future resolves then. Nothing reads the canvas, so the page needs no
//! `preserveDrawingBuffer`.

use crate::{RenderError, Renderer, RgbaImage};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

#[derive(Default)]
struct MapState {
    result: Option<Result<(), wgpu::BufferAsyncError>>,
    waker: Option<Waker>,
}

/// Resolves when `map_async`'s callback has run.
struct Mapped(Arc<Mutex<MapState>>);

impl Future for Mapped {
    type Output = Result<(), wgpu::BufferAsyncError>;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let mut state = self.0.lock().unwrap_or_else(|e| e.into_inner());
        match state.result.take() {
            Some(result) => Poll::Ready(result),
            None => {
                state.waker = Some(cx.waker().clone());
                Poll::Pending
            }
        }
    }
}

impl Renderer {
    /// Processes finished GPU work without blocking: completes pending
    /// captures on WebGL2 when no frame is being submitted.
    pub fn poll(&self) {
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    /// The last frame drawn to [`crate::RenderTarget::Offscreen`] as RGBA8,
    /// rows top to bottom (BGRA outputs are swizzled). Works on every host;
    /// natively it returns as soon as the GPU has finished the copy.
    pub async fn capture(&self) -> Result<RgbaImage, RenderError> {
        let (width, height) = (self.width, self.height);
        let swizzle = match self.output_format {
            wgpu::TextureFormat::Rgba8Unorm | wgpu::TextureFormat::Rgba8UnormSrgb => false,
            wgpu::TextureFormat::Bgra8Unorm | wgpu::TextureFormat::Bgra8UnormSrgb => true,
            other => return Err(RenderError::Unsupported(format!("capture of {other:?}"))),
        };
        let row = width * 4;
        let padded_row =
            row.div_ceil(wgpu::COPY_BYTES_PER_ROW_ALIGNMENT) * wgpu::COPY_BYTES_PER_ROW_ALIGNMENT;
        let buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("capture"),
            size: (padded_row * height) as u64,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("capture"),
            });
        encoder.copy_texture_to_buffer(
            wgpu::TexelCopyTextureInfo {
                texture: &self.targets.offscreen,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            wgpu::TexelCopyBufferInfo {
                buffer: &buffer,
                layout: wgpu::TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(padded_row),
                    rows_per_image: Some(height),
                },
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );
        self.queue.submit(Some(encoder.finish()));
        let state = Arc::new(Mutex::new(MapState::default()));
        let callback_state = state.clone();
        buffer
            .slice(..)
            .map_async(wgpu::MapMode::Read, move |result| {
                let mut s = callback_state.lock().unwrap_or_else(|e| e.into_inner());
                s.result = Some(result);
                if let Some(waker) = s.waker.take() {
                    waker.wake();
                }
            });
        #[cfg(not(target_arch = "wasm32"))]
        self.device
            .poll(wgpu::PollType::wait_indefinitely())
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        #[cfg(target_arch = "wasm32")]
        self.poll();
        Mapped(state)
            .await
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        let mut pixels = Vec::with_capacity((row * height) as usize);
        {
            let data = buffer
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
            for chunk in data.chunks_exact(padded_row as usize) {
                pixels.extend_from_slice(&chunk[..row as usize]);
            }
        }
        buffer.unmap();
        if swizzle {
            for p in pixels.chunks_exact_mut(4) {
                p.swap(0, 2);
            }
        }
        Ok(RgbaImage {
            width,
            height,
            pixels,
        })
    }
}
