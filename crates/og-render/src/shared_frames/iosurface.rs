//! macOS: the slots are BGRA8 IOSurfaces, wrapped as a Metal texture on
//! each side's device (`newTextureWithDescriptor:iosurface:plane:`).
//!
//! The surfaces travel as Mach send rights (`IOSurfaceCreateMachPort`) by a
//! one-shot bootstrap rendezvous: the engine checks in a fresh service
//! name, names it in the `Hello` on the socket, and answers the first
//! request whose sender (by the message's audit trailer) is the socket's
//! peer process with one port descriptor per slot. The host turns each
//! right back into the surface with `IOSurfaceLookupFromMachPort`. No global
//! IOSurface ID is used, so no other process can look the frames up.

use super::{MAX_SLOTS, SLOT_FORMAT, slot_desc, write_frame};
use mach2::bootstrap::{bootstrap_check_in, bootstrap_look_up, bootstrap_port};
use mach2::kern_return::KERN_SUCCESS;
use mach2::mach_port::{mach_port_allocate, mach_port_deallocate, mach_port_mod_refs};
use mach2::message::*;
use mach2::port::{MACH_PORT_NULL, MACH_PORT_RIGHT_RECEIVE, mach_port_t};
use mach2::traps::mach_task_self;
use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_core_foundation::{CFDictionary, CFNumber, CFRetained, CFString};
use objc2_io_surface::{
    IOSurfaceRef, kIOSurfaceBytesPerElement, kIOSurfaceHeight, kIOSurfacePixelFormat,
    kIOSurfaceWidth,
};
use objc2_metal::{
    MTLDevice, MTLPixelFormat, MTLStorageMode, MTLTexture, MTLTextureDescriptor, MTLTextureType,
    MTLTextureUsage,
};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::os::fd::AsRawFd;
use std::os::unix::net::UnixStream;
use wgpu::hal::api::Metal;

/// `kCVPixelFormatType_32BGRA`.
const PIXEL_FORMAT_BGRA: i32 = i32::from_be_bytes(*b"BGRA");

/// What carries a slot's memory between the processes: the surface.
pub type SharedHandle = CFRetained<IOSurfaceRef>;

/// Engine → host, once a generation; the surfaces follow by Mach rendezvous.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hello {
    pub width: u32,
    pub height: u32,
    pub slots: u32,
    pub adapter: String,
    /// The bootstrap name the engine's rendezvous is checked in under, for
    /// this one host.
    pub mach_service: String,
    /// `MTLDevice.registryID` of the engine's GPU.
    pub registry_id: u64,
}

const REQUEST_ID: i32 = 0x4f47_0001;
const REPLY_ID: i32 = 0x4f47_0002;
const TIMEOUT_MS: u32 = 10_000;

type MetalDevice = ProtocolObject<dyn MTLDevice>;

fn metal_device<R>(device: &wgpu::Device, f: impl FnOnce(&MetalDevice) -> R) -> Result<R, String> {
    let hal = unsafe { device.as_hal::<Metal>() }.ok_or("not a Metal device")?;
    Ok(f(hal.raw_device()))
}

/// A BGRA surface of `width` × `height`.
fn new_surface(width: u32, height: u32) -> Result<SharedHandle, String> {
    let keys: [&CFString; 4] = unsafe {
        [
            kIOSurfaceWidth,
            kIOSurfaceHeight,
            kIOSurfaceBytesPerElement,
            kIOSurfacePixelFormat,
        ]
    };
    let values = [
        CFNumber::new_i32(width as i32),
        CFNumber::new_i32(height as i32),
        CFNumber::new_i32(4),
        CFNumber::new_i32(PIXEL_FORMAT_BGRA),
    ];
    let values: Vec<&CFNumber> = values.iter().map(|v| &**v).collect();
    let properties = CFDictionary::from_slices(&keys, &values);
    unsafe { IOSurfaceRef::new(properties.as_opaque()) }.ok_or_else(|| "IOSurfaceCreate".into())
}

/// `surface` as a Metal texture on `device`, readable and renderable.
fn surface_texture(
    device: &MetalDevice,
    surface: &IOSurfaceRef,
) -> Result<Retained<ProtocolObject<dyn MTLTexture>>, String> {
    let desc = unsafe {
        MTLTextureDescriptor::texture2DDescriptorWithPixelFormat_width_height_mipmapped(
            MTLPixelFormat::BGRA8Unorm,
            surface.width(),
            surface.height(),
            false,
        )
    };
    desc.setUsage(
        MTLTextureUsage::RenderTarget
            | MTLTextureUsage::ShaderRead
            | MTLTextureUsage::PixelFormatView,
    );
    // Apple GPUs share the surface's memory; a discrete GPU keeps a managed copy.
    desc.setStorageMode(if device.hasUnifiedMemory() {
        MTLStorageMode::Shared
    } else {
        MTLStorageMode::Managed
    });
    device
        .newTextureWithDescriptor_iosurface_plane(&desc, surface, 0)
        .ok_or_else(|| "newTextureWithDescriptor:iosurface:plane:".into())
}

fn wgpu_texture(
    device: &wgpu::Device,
    raw: Retained<ProtocolObject<dyn MTLTexture>>,
    initial: wgpu::TextureUses,
) -> wgpu::Texture {
    let (width, height) = (raw.width() as u32, raw.height() as u32);
    let hal = unsafe {
        wgpu::hal::metal::Device::texture_from_raw(
            raw,
            SLOT_FORMAT,
            MTLTextureType::Type2D,
            1,
            1,
            wgpu::hal::CopyExtent {
                width,
                height,
                depth: 1,
            },
            None,
        )
    };
    unsafe { device.create_texture_from_hal::<Metal>(hal, &slot_desc(width, height), initial) }
}

/// Engine: `slots` surfaces wrapped as textures on `device`, and the
/// generation's `Hello`.
pub fn export_slots(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    slots: u32,
) -> Result<(Hello, Vec<wgpu::Texture>, Vec<SharedHandle>), String> {
    if !(1..=MAX_SLOTS).contains(&slots) {
        return Err(format!("1 to {MAX_SLOTS} slots, not {slots}"));
    }
    let (adapter, registry_id) = metal_device(device, |d| (d.name().to_string(), d.registryID()))?;
    let mut textures = Vec::new();
    let mut surfaces = Vec::new();
    for _ in 0..slots {
        let surface = new_surface(width, height)?;
        let raw = metal_device(device, |d| surface_texture(d, &surface))??;
        textures.push(wgpu_texture(device, raw, wgpu::TextureUses::UNINITIALIZED));
        surfaces.push(surface);
    }
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.subsec_nanos());
    Ok((
        Hello {
            width,
            height,
            slots,
            adapter,
            mach_service: format!("ai.origingame.og-frames.{}.{nanos:08x}", std::process::id()),
            registry_id,
        },
        textures,
        surfaces,
    ))
}

/// Host: `surface` as a sampled texture on `device` (wgpu).
pub fn import_slot(
    device: &wgpu::Device,
    hello: &Hello,
    surface: &SharedHandle,
) -> Result<wgpu::Texture, String> {
    if surface.width() != hello.width as usize || surface.height() != hello.height as usize {
        return Err(format!(
            "surface is {}x{}, the hello says {}x{}",
            surface.width(),
            surface.height(),
            hello.width,
            hello.height
        ));
    }
    let raw = metal_device(device, |d| surface_texture(d, surface))??;
    Ok(wgpu_texture(device, raw, wgpu::TextureUses::RESOURCE))
}

// ── the Hello: JSON on the socket, the surfaces by Mach rendezvous ───────

/// A receive right this process owns; dropped, the port dies.
struct ReceiveRight(mach_port_t);

impl Drop for ReceiveRight {
    fn drop(&mut self) {
        unsafe { mach_port_mod_refs(mach_task_self(), self.0, MACH_PORT_RIGHT_RECEIVE, -1) };
    }
}

/// A send right this process holds a reference to.
struct SendRight(mach_port_t);

impl Drop for SendRight {
    fn drop(&mut self) {
        unsafe { mach_port_deallocate(mach_task_self(), self.0) };
    }
}

#[repr(C)]
struct RequestBuffer {
    header: mach_msg_header_t,
    trailer: mach_msg_audit_trailer_t,
}

#[repr(C)]
struct Reply {
    header: mach_msg_header_t,
    body: mach_msg_body_t,
    ports: [mach_msg_port_descriptor_t; MAX_SLOTS as usize],
}

#[repr(C)]
struct ReplyBuffer {
    reply: Reply,
    trailer: mach_msg_trailer_t,
}

fn check(what: &str, kr: i32) -> Result<(), String> {
    if kr != KERN_SUCCESS {
        return Err(format!("{what}: {kr:#x}"));
    }
    Ok(())
}

fn peer_pid(stream: &UnixStream) -> Result<i32, String> {
    let mut pid: libc::pid_t = 0;
    let mut len = size_of::<libc::pid_t>() as libc::socklen_t;
    let r = unsafe {
        libc::getsockopt(
            stream.as_raw_fd(),
            libc::SOL_LOCAL,
            libc::LOCAL_PEERPID,
            (&raw mut pid).cast(),
            &mut len,
        )
    };
    if r != 0 {
        return Err(format!(
            "LOCAL_PEERPID: {}",
            std::io::Error::last_os_error()
        ));
    }
    Ok(pid)
}

/// Engine: sends `hello` on `stream`, then hands `surfaces` to the
/// socket's peer when it asks the service the hello names.
pub fn send_hello(
    stream: &UnixStream,
    hello: &Hello,
    surfaces: &[SharedHandle],
) -> Result<(), String> {
    if surfaces.len() != hello.slots as usize {
        return Err("one surface per slot".into());
    }
    let peer = peer_pid(stream)?;
    let name = CString::new(hello.mach_service.as_str()).map_err(|e| e.to_string())?;
    let mut port = MACH_PORT_NULL;
    check("bootstrap_check_in", unsafe {
        bootstrap_check_in(bootstrap_port, name.as_ptr(), &mut port)
    })?;
    let service = ReceiveRight(port);
    let body = serde_json::to_vec(hello).map_err(|e| e.to_string())?;
    write_frame(&mut &*stream, &body).map_err(|e| e.to_string())?;

    // The request: an empty message with a send-once reply right.
    let mut request: RequestBuffer = unsafe { std::mem::zeroed() };
    check("mach_msg (receive request)", unsafe {
        mach_msg(
            &mut request.header,
            MACH_RCV_MSG
                | MACH_RCV_TIMEOUT
                | ((MACH_MSG_TRAILER_FORMAT_0 as i32) << 28)
                | ((MACH_RCV_TRAILER_AUDIT as i32) << 24),
            0,
            size_of::<RequestBuffer>() as u32,
            service.0,
            TIMEOUT_MS,
            MACH_PORT_NULL,
        )
    })?;
    let reply_port = SendRight(request.header.msgh_remote_port);
    if request.header.msgh_id != REQUEST_ID {
        return Err(format!("unexpected message {:#x}", request.header.msgh_id));
    }
    // audit_token_t: val[5] is the sender's pid.
    let sender = request.trailer.msgh_audit.val[5] as i32;
    if sender != peer {
        return Err(format!(
            "rendezvous asked by pid {sender}, the socket's peer is {peer}"
        ));
    }

    let mut reply: Reply = unsafe { std::mem::zeroed() };
    reply.header.msgh_bits =
        MACH_MSGH_BITS(MACH_MSG_TYPE_MOVE_SEND_ONCE, 0) | MACH_MSGH_BITS_COMPLEX;
    reply.header.msgh_remote_port = reply_port.0;
    reply.header.msgh_id = REPLY_ID;
    reply.body.msgh_descriptor_count = surfaces.len() as u32;
    for (descriptor, surface) in reply.ports.iter_mut().zip(surfaces) {
        let right = surface.create_mach_port();
        if right == MACH_PORT_NULL {
            return Err("IOSurfaceCreateMachPort".into());
        }
        *descriptor = mach_msg_port_descriptor_t::new(right, MACH_MSG_TYPE_MOVE_SEND);
    }
    let size = size_of::<mach_msg_header_t>()
        + size_of::<mach_msg_body_t>()
        + surfaces.len() * size_of::<mach_msg_port_descriptor_t>();
    reply.header.msgh_size = size as u32;
    // The send-once right is consumed by the send.
    std::mem::forget(reply_port);
    check("mach_msg (send surfaces)", unsafe {
        mach_msg(
            &mut reply.header,
            MACH_SEND_MSG | MACH_SEND_TIMEOUT,
            size as u32,
            0,
            MACH_PORT_NULL,
            TIMEOUT_MS,
            MACH_PORT_NULL,
        )
    })?;
    drop(service);
    Ok(())
}

/// Host: the hello on `stream` and its surfaces, one per slot.
pub fn recv_hello(stream: &UnixStream) -> Result<(Hello, Vec<SharedHandle>), String> {
    let body = super::read_frame(&mut &*stream).map_err(|e| e.to_string())?;
    let hello: Hello = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    if hello.slots > MAX_SLOTS {
        return Err(format!("{} slots", hello.slots));
    }
    let name = CString::new(hello.mach_service.as_str()).map_err(|e| e.to_string())?;
    let mut port = MACH_PORT_NULL;
    check("bootstrap_look_up", unsafe {
        bootstrap_look_up(bootstrap_port, name.as_ptr(), &mut port)
    })?;
    let service = SendRight(port);
    let mut port = MACH_PORT_NULL;
    check("mach_port_allocate", unsafe {
        mach_port_allocate(mach_task_self(), MACH_PORT_RIGHT_RECEIVE, &mut port)
    })?;
    let reply_port = ReceiveRight(port);

    let mut request: mach_msg_header_t = unsafe { std::mem::zeroed() };
    request.msgh_bits = MACH_MSGH_BITS(MACH_MSG_TYPE_COPY_SEND, MACH_MSG_TYPE_MAKE_SEND_ONCE);
    request.msgh_size = size_of::<mach_msg_header_t>() as u32;
    request.msgh_remote_port = service.0;
    request.msgh_local_port = reply_port.0;
    request.msgh_id = REQUEST_ID;
    check("mach_msg (request surfaces)", unsafe {
        mach_msg(
            &mut request,
            MACH_SEND_MSG | MACH_SEND_TIMEOUT,
            request.msgh_size,
            0,
            MACH_PORT_NULL,
            TIMEOUT_MS,
            MACH_PORT_NULL,
        )
    })?;

    let mut buffer: ReplyBuffer = unsafe { std::mem::zeroed() };
    check("mach_msg (receive surfaces)", unsafe {
        mach_msg(
            &mut buffer.reply.header,
            MACH_RCV_MSG | MACH_RCV_TIMEOUT,
            0,
            size_of::<ReplyBuffer>() as u32,
            reply_port.0,
            TIMEOUT_MS,
            MACH_PORT_NULL,
        )
    })?;
    let reply = &buffer.reply;
    if reply.header.msgh_id != REPLY_ID || reply.header.msgh_bits & MACH_MSGH_BITS_COMPLEX == 0 {
        return Err(format!("unexpected reply {:#x}", reply.header.msgh_id));
    }
    let count = (reply.body.msgh_descriptor_count as usize).min(MAX_SLOTS as usize);
    // Every right received is released, whatever the lookups give.
    let rights: Vec<SendRight> = reply.ports[..count]
        .iter()
        .filter(|d| d.type_ == MACH_MSG_PORT_DESCRIPTOR as u8)
        .map(|d| SendRight(d.name))
        .collect();
    if rights.len() != hello.slots as usize {
        return Err(format!(
            "{} surfaces for {} slots",
            rights.len(),
            hello.slots
        ));
    }
    let surfaces = rights
        .iter()
        .map(|right| {
            IOSurfaceRef::lookup_from_mach_port(right.0)
                .ok_or_else(|| "IOSurfaceLookupFromMachPort".to_string())
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok((hello, surfaces))
}
