//! Frame capture on both hosts: the offscreen target copied into a mapped
//! buffer. Nothing reads the canvas, so the page needs no
//! `preserveDrawingBuffer`.
//!
//! A capture is a request: [`Renderer::begin_capture`] records the copy and
//! starts the mapping; the [`CaptureRequest`] then completes without
//! blocking ([`CaptureRequest::try_finish`], for a host's request/complete
//! loop), natively with a bound ([`CaptureRequest::wait`]) or as a future
//! ([`CaptureRequest::finish`]). Dropping the request (or the
//! [`Renderer::capture`] future) cancels it and frees the buffer. In the
//! browser the mapping completes from the event loop (WebGPU's `mapAsync`)
//! or on the next submit or [`Renderer::poll`] (WebGL2).
//!
//! A renderer that presents to a surface (the web host's canvas) draws its
//! frames there, not offscreen: [`Renderer::set_keep_frame`] has each
//! presented frame copied into the offscreen target as well, so a capture
//! started after it reads the frame the loop drew (`og render`'s
//! every-frame capture on the web, through the probe's `Capture::frame`).
//!
//! [`png`] encodes a frame for the probe's `capture` and `screenshot`.

use crate::{Encoder, RenderError, Renderer, RgbaImage};
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex};
use std::task::{Context, Poll, Waker};

/// How long [`Renderer::capture`] waits natively before it gives up.
pub const CAPTURE_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(10);

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

/// A capture in flight: the copy of the last offscreen frame into a
/// buffer being mapped. Dropping it cancels the capture.
pub struct CaptureRequest {
    buffer: wgpu::Buffer,
    state: Arc<Mutex<MapState>>,
    #[cfg_attr(target_arch = "wasm32", allow(dead_code))]
    submission: wgpu::SubmissionIndex,
    width: u32,
    height: u32,
    padded_row: u32,
    swizzle: bool,
}

impl CaptureRequest {
    /// Whether the mapping has completed (successfully or not), without
    /// polling the device.
    pub fn is_ready(&self) -> bool {
        self.state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .result
            .is_some()
    }

    /// Polls the device once without blocking: `Ok(Some(frame))` once the
    /// copy has landed, `Ok(None)` while it is pending. After `Some` (or an
    /// error) the request is spent.
    pub fn try_finish(&mut self, renderer: &Renderer) -> Result<Option<RgbaImage>, RenderError> {
        renderer.poll();
        let result = self
            .state
            .lock()
            .unwrap_or_else(|e| e.into_inner())
            .result
            .take();
        match result {
            None => Ok(None),
            Some(r) => {
                r.map_err(|e| RenderError::Readback(e.to_string()))?;
                self.read().map(Some)
            }
        }
    }

    /// Blocks until the copy has landed or `timeout` passed (native only).
    #[cfg(not(target_arch = "wasm32"))]
    pub fn wait(
        mut self,
        renderer: &Renderer,
        timeout: std::time::Duration,
    ) -> Result<RgbaImage, RenderError> {
        renderer
            .device
            .poll(wgpu::PollType::Wait {
                submission_index: Some(self.submission.clone()),
                timeout: Some(timeout),
            })
            .map_err(|e| match e {
                wgpu::PollError::Timeout => {
                    RenderError::Readback(format!("capture timed out after {timeout:?}"))
                }
                other => RenderError::Readback(other.to_string()),
            })?;
        match self.try_finish(renderer)? {
            Some(image) => Ok(image),
            None => Err(RenderError::Readback(format!(
                "capture not mapped after {timeout:?}"
            ))),
        }
    }

    /// Resolves when the mapping completes (the host bounds the wait: drop
    /// the future to cancel). On WebGL2 something must poll the device:
    /// the next frame's submit or [`Renderer::poll`].
    pub async fn finish(self) -> Result<RgbaImage, RenderError> {
        Mapped(self.state.clone())
            .await
            .map_err(|e| RenderError::Readback(e.to_string()))?;
        self.read()
    }

    /// Cancels the capture (the same as dropping it).
    pub fn cancel(self) {}

    fn read(&self) -> Result<RgbaImage, RenderError> {
        let row = self.width * 4;
        let mut pixels = Vec::with_capacity((row * self.height) as usize);
        {
            let data = self
                .buffer
                .slice(..)
                .get_mapped_range()
                .map_err(|e| RenderError::Readback(format!("{e:?}")))?;
            for chunk in data.chunks_exact(self.padded_row as usize) {
                pixels.extend_from_slice(&chunk[..row as usize]);
            }
        }
        self.buffer.unmap();
        if self.swizzle {
            for p in pixels.as_chunks_mut::<4>().0 {
                p.swap(0, 2);
            }
        }
        Ok(RgbaImage {
            width: self.width,
            height: self.height,
            pixels,
        })
    }
}

impl Drop for CaptureRequest {
    fn drop(&mut self) {
        self.buffer.destroy();
    }
}

impl Renderer {
    /// Processes finished GPU work without blocking: completes pending
    /// captures on WebGL2 when no frame is being submitted.
    pub fn poll(&self) {
        let _ = self.device.poll(wgpu::PollType::Poll);
    }

    /// Keep each frame presented to the surface in the offscreen target too
    /// (one copy per frame), so [`Renderer::begin_capture`] reads the frame
    /// the loop drew. Refused when the surface cannot be copied from (its
    /// device lacks `COPY_SRC` on surfaces) or there is no surface (a
    /// headless renderer draws offscreen already).
    pub fn set_keep_frame(&mut self, on: bool) -> Result<(), RenderError> {
        let Some((_, config)) = &self.surface else {
            return Err(RenderError::Unsupported(
                "keep_frame on a renderer without a surface (it draws offscreen)".into(),
            ));
        };
        if on && !config.usage.contains(wgpu::TextureUsages::COPY_SRC) {
            return Err(RenderError::Unsupported(
                "this device's surface frames cannot be copied (no COPY_SRC)".into(),
            ));
        }
        self.keep_frame = on;
        Ok(())
    }

    /// Whether presented frames are kept offscreen ([`Renderer::set_keep_frame`]).
    pub fn keeps_frame(&self) -> bool {
        self.keep_frame
    }

    /// Copies `frame` (a texture of the output's size and format, with
    /// `COPY_SRC`) into the offscreen target now: what each presented frame
    /// goes through with [`Renderer::set_keep_frame`], for a host that
    /// draws its frames elsewhere.
    pub fn keep_texture(&self, frame: &wgpu::Texture) {
        let mut encoder = Encoder::untimed(self.device.create_command_encoder(
            &wgpu::CommandEncoderDescriptor {
                label: Some("keep frame"),
            },
        ));
        self.keep_presented(&mut encoder, frame);
        self.queue.submit(Some(encoder.finish()));
    }

    /// Encodes the copy of a presented surface frame into the offscreen
    /// target (the same size and format: both follow the output).
    pub(crate) fn keep_presented(&self, encoder: &mut crate::Encoder, frame: &wgpu::Texture) {
        let size = wgpu::Extent3d {
            width: self.out_width.min(frame.width()),
            height: self.out_height.min(frame.height()),
            depth_or_array_layers: 1,
        };
        encoder.copy_texture_to_texture(
            frame.as_image_copy(),
            self.persistent.offscreen.as_image_copy(),
            size,
        );
    }

    /// Starts a capture of the last frame drawn to
    /// [`crate::RenderTarget::Offscreen`] (see the module docs).
    pub fn begin_capture(&self) -> Result<CaptureRequest, RenderError> {
        let (width, height) = (self.out_width, self.out_height);
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
                texture: &self.persistent.offscreen,
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
        let submission = self.queue.submit(Some(encoder.finish()));
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
        Ok(CaptureRequest {
            buffer,
            state,
            submission,
            width,
            height,
            padded_row,
            swizzle,
        })
    }

    /// The last frame drawn to [`crate::RenderTarget::Offscreen`] as RGBA8,
    /// rows top to bottom (BGRA outputs are swizzled). Works on every host;
    /// natively it waits at most [`CAPTURE_TIMEOUT`]; in the browser it
    /// resolves when the mapping does (drop the future to cancel).
    pub async fn capture(&self) -> Result<RgbaImage, RenderError> {
        let request = self.begin_capture()?;
        #[cfg(not(target_arch = "wasm32"))]
        {
            request.wait(self, CAPTURE_TIMEOUT)
        }
        #[cfg(target_arch = "wasm32")]
        {
            self.poll();
            request.finish().await
        }
    }
}

/// A PNG of an RGBA8 image (the probe's `capture` and `screenshot`): every
/// row but the first filtered with `Up` and deflated as it is filtered by
/// fdeflate, whose one fixed code for small residuals and runs of zeros is
/// made for filtered image rows. A 1280×720 frame encodes two to four times
/// faster than at zlib's fastest level and 10 to 35 times faster than at
/// level 6, in a file about 1.5 times level 6's on a detailed frame and up
/// to 8 times on a flat one (`tests/evidence/r4/perf-capture/NOTES.md`);
/// the pixels decode exactly.
pub fn png(image: &RgbaImage) -> Vec<u8> {
    let stride = image.width as usize * 4;
    let mut out = Vec::with_capacity(image.pixels.len() / 3 + 64);
    out.extend_from_slice(b"\x89PNG\r\n\x1a\n");
    let mut ihdr = [0u8; 13];
    ihdr[..4].copy_from_slice(&image.width.to_be_bytes());
    ihdr[4..8].copy_from_slice(&image.height.to_be_bytes());
    // 8-bit RGBA, deflate, adaptive filtering, not interlaced.
    ihdr[8..].copy_from_slice(&[8, 6, 0, 0, 0]);
    png_chunk(&mut out, b"IHDR", &ihdr);
    // IDAT is written in place: its length is known once the stream ends.
    let idat = out.len();
    out.extend_from_slice(&[0, 0, 0, 0]);
    out.extend_from_slice(b"IDAT");
    let mut z = fdeflate::Compressor::new(out).expect("a Vec takes every write");
    let mut row = vec![0u8; stride + 1];
    let mut prev: &[u8] = &[];
    for (y, cur) in image.pixels.chunks_exact(stride.max(1)).enumerate() {
        if y == 0 {
            row[0] = 0;
            row[1..].copy_from_slice(cur);
        } else {
            row[0] = 2;
            for ((f, c), p) in row[1..].iter_mut().zip(cur).zip(prev) {
                *f = c.wrapping_sub(*p);
            }
        }
        z.write_data(&row).expect("a Vec takes every write");
        prev = cur;
    }
    let mut out = z.finish().expect("a Vec takes every write");
    let len = (out.len() - idat - 8) as u32;
    out[idat..idat + 4].copy_from_slice(&len.to_be_bytes());
    let crc = crc32(&out[idat + 4..]);
    out.extend_from_slice(&crc.to_be_bytes());
    png_chunk(&mut out, b"IEND", &[]);
    out
}

fn png_chunk(out: &mut Vec<u8>, kind: &[u8; 4], data: &[u8]) {
    out.extend_from_slice(&(data.len() as u32).to_be_bytes());
    let at = out.len();
    out.extend_from_slice(kind);
    out.extend_from_slice(data);
    let crc = crc32(&out[at..]);
    out.extend_from_slice(&crc.to_be_bytes());
}

/// CRC-32 (ISO-HDLC, PNG's) sixteen bytes at a time: a byte-at-a-time
/// table was a third of the encode once deflate was fast. The tables are
/// made on first use, not stored in the module.
fn crc32(data: &[u8]) -> u32 {
    static TABLES: std::sync::OnceLock<Box<[[u32; 256]; 16]>> = std::sync::OnceLock::new();
    let t = TABLES.get_or_init(|| {
        let mut t = Box::new([[0u32; 256]; 16]);
        for n in 0..256 {
            let mut c = n as u32;
            for _ in 0..8 {
                c = if c & 1 != 0 {
                    0xedb8_8320 ^ (c >> 1)
                } else {
                    c >> 1
                };
            }
            t[0][n] = c;
        }
        for k in 1..16 {
            for n in 0..256 {
                let p = t[k - 1][n];
                t[k][n] = (p >> 8) ^ t[0][(p & 0xff) as usize];
            }
        }
        t
    });
    let mut c = !0u32;
    let (blocks, rest) = data.as_chunks::<16>();
    for b in blocks {
        let x = c ^ u32::from_le_bytes([b[0], b[1], b[2], b[3]]);
        c = t[15][(x & 0xff) as usize]
            ^ t[14][((x >> 8) & 0xff) as usize]
            ^ t[13][((x >> 16) & 0xff) as usize]
            ^ t[12][(x >> 24) as usize];
        for (i, &byte) in b.iter().enumerate().skip(4) {
            c ^= t[15 - i][byte as usize];
        }
    }
    for &b in rest {
        c = t[0][((c ^ u32::from(b)) & 0xff) as usize] ^ (c >> 8);
    }
    !c
}
