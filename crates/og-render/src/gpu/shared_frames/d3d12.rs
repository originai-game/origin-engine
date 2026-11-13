//! Windows: the slots as D3D12 committed resources on a shared heap
//! (`D3D12_HEAP_FLAG_SHARED`), exported as NT handles with
//! `ID3D12Device::CreateSharedHandle`. The engine duplicates each handle
//! into the process at the other end of the socket (found with
//! `SIO_AF_UNIX_GETPEERPID`) and sends the duplicates' values in the
//! `Hello`, so the host owns them as it owns fds after `SCM_RIGHTS`.
//!
//! The resources allow simultaneous access: another device (the window's
//! D3D11) may read one while this one's queue is idle, without a state
//! transition back from render target, and the driver keeps it
//! uncompressed. The wire is a Windows AF_UNIX socket.

use super::{MAX_SLOTS, SLOT_FORMAT, slot_desc, write_frame};
use serde::{Deserialize, Serialize};
use std::os::windows::io::{AsRawHandle, AsRawSocket, FromRawHandle, OwnedHandle};
use uds_windows::UnixStream;
use wgpu::hal::api::Dx12;
use windows::Win32::Foundation::{
    DUPLICATE_SAME_ACCESS, DuplicateHandle, GENERIC_ALL, HANDLE, LUID,
};
use windows::Win32::Graphics::Direct3D12::{
    D3D12_HEAP_FLAG_SHARED, D3D12_HEAP_PROPERTIES, D3D12_HEAP_TYPE_DEFAULT, D3D12_RESOURCE_DESC,
    D3D12_RESOURCE_DIMENSION_TEXTURE2D, D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET,
    D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS, D3D12_RESOURCE_STATE_COMMON,
    D3D12_TEXTURE_LAYOUT_UNKNOWN, ID3D12Resource,
};
use windows::Win32::Graphics::Dxgi::Common::{DXGI_FORMAT_R8G8B8A8_TYPELESS, DXGI_SAMPLE_DESC};
use windows::Win32::Graphics::Dxgi::{CreateDXGIFactory1, IDXGIAdapter1, IDXGIFactory4};
use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcess, PROCESS_DUP_HANDLE};

/// What carries a slot's memory between the processes.
pub type SharedHandle = OwnedHandle;

/// Engine → host, once a generation.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Hello {
    pub width: u32,
    pub height: u32,
    pub slots: u32,
    /// One shared NT handle per slot, as a value valid in the host's
    /// process ([`send_hello`] duplicates them there).
    pub handles: Vec<u64>,
    /// The LUID of the adapter the resources live on: a handle opens only
    /// on a device of the same adapter.
    pub adapter_luid: u64,
    pub adapter: String,
}

fn luid_u64(luid: LUID) -> u64 {
    (luid.HighPart as u32 as u64) << 32 | luid.LowPart as u64
}

fn adapter_luid(device: &wgpu::Device) -> Result<u64, String> {
    let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("not a D3D12 device")?;
    Ok(luid_u64(unsafe { hal.raw_device().GetAdapterLuid() }))
}

fn adapter_name(luid: u64) -> Option<String> {
    let luid = LUID {
        LowPart: luid as u32,
        HighPart: (luid >> 32) as i32,
    };
    let factory: IDXGIFactory4 = unsafe { CreateDXGIFactory1() }.ok()?;
    let adapter: IDXGIAdapter1 = unsafe { factory.EnumAdapterByLuid(luid) }.ok()?;
    let desc = unsafe { adapter.GetDesc1() }.ok()?;
    Some(
        String::from_utf16_lossy(&desc.Description)
            .trim_end_matches('\0')
            .to_string(),
    )
}

fn hal_texture(resource: ID3D12Resource, width: u32, height: u32) -> wgpu::hal::dx12::Texture {
    unsafe {
        wgpu::hal::dx12::Device::texture_from_raw(
            resource,
            SLOT_FORMAT,
            wgpu::TextureDimension::D2,
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            1,
            1,
        )
    }
}

/// Engine: `slots` shareable textures on `device`, their NT handles and
/// the generation's `Hello`.
pub fn export_slots(
    device: &wgpu::Device,
    width: u32,
    height: u32,
    slots: u32,
) -> Result<(Hello, Vec<wgpu::Texture>, Vec<OwnedHandle>), String> {
    if !(1..=MAX_SLOTS).contains(&slots) {
        return Err(format!("1 to {MAX_SLOTS} slots, not {slots}"));
    }
    let luid = adapter_luid(device)?;
    let mut textures = Vec::new();
    let mut handles = Vec::new();
    for _ in 0..slots {
        let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("not a D3D12 device")?;
        let raw = hal.raw_device();
        // Typeless, so the engine's sRGB view and the host's UNORM
        // sampling view both fit it.
        let desc = D3D12_RESOURCE_DESC {
            Dimension: D3D12_RESOURCE_DIMENSION_TEXTURE2D,
            Alignment: 0,
            Width: width as u64,
            Height: height,
            DepthOrArraySize: 1,
            MipLevels: 1,
            Format: DXGI_FORMAT_R8G8B8A8_TYPELESS,
            SampleDesc: DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            Layout: D3D12_TEXTURE_LAYOUT_UNKNOWN,
            Flags: D3D12_RESOURCE_FLAG_ALLOW_RENDER_TARGET
                | D3D12_RESOURCE_FLAG_ALLOW_SIMULTANEOUS_ACCESS,
        };
        let heap = D3D12_HEAP_PROPERTIES {
            Type: D3D12_HEAP_TYPE_DEFAULT,
            ..Default::default()
        };
        let mut resource: Option<ID3D12Resource> = None;
        unsafe {
            raw.CreateCommittedResource(
                &heap,
                D3D12_HEAP_FLAG_SHARED,
                &desc,
                D3D12_RESOURCE_STATE_COMMON,
                None,
                &mut resource,
            )
        }
        .map_err(|e| format!("CreateCommittedResource: {e}"))?;
        let resource = resource.ok_or("CreateCommittedResource returned nothing")?;
        let handle = unsafe { raw.CreateSharedHandle(&resource, None, GENERIC_ALL.0, None) }
            .map_err(|e| format!("CreateSharedHandle: {e}"))?;
        handles.push(unsafe { OwnedHandle::from_raw_handle(handle.0) });
        drop(hal);
        textures.push(unsafe {
            device.create_texture_from_hal::<Dx12>(
                hal_texture(resource, width, height),
                &slot_desc(width, height),
                wgpu::TextureUses::UNINITIALIZED,
            )
        });
    }
    Ok((
        Hello {
            width,
            height,
            slots,
            handles: Vec::new(),
            adapter_luid: luid,
            adapter: adapter_name(luid).unwrap_or_default(),
        },
        textures,
        handles,
    ))
}

/// Host: the resource behind `handle` as a sampled texture on `device` (a
/// D3D12 one on the adapter the hello names).
pub fn import_slot(
    device: &wgpu::Device,
    hello: &Hello,
    handle: &OwnedHandle,
) -> Result<wgpu::Texture, String> {
    if adapter_luid(device)? != hello.adapter_luid {
        return Err("the frame's memory comes from another adapter".into());
    }
    let (width, height) = (hello.width, hello.height);
    let hal = unsafe { device.as_hal::<Dx12>() }.ok_or("not a D3D12 device")?;
    let mut resource: Option<ID3D12Resource> = None;
    unsafe {
        hal.raw_device()
            .OpenSharedHandle(HANDLE(handle.as_raw_handle()), &mut resource)
    }
    .map_err(|e| format!("OpenSharedHandle: {e}"))?;
    let resource = resource.ok_or("OpenSharedHandle returned nothing")?;
    let desc = unsafe { resource.GetDesc() };
    if desc.Width != width as u64 || desc.Height != height {
        return Err(format!(
            "shared resource is {}x{}, expected {width}x{height}",
            desc.Width, desc.Height
        ));
    }
    drop(hal);
    Ok(unsafe {
        device.create_texture_from_hal::<Dx12>(
            hal_texture(resource, width, height),
            &slot_desc(width, height),
            wgpu::TextureUses::RESOURCE,
        )
    })
}

// ── the Hello: length + JSON naming handles already duplicated ───────────

/// The id of the process at the other end of `stream`.
fn peer_pid(stream: &UnixStream) -> Result<u32, String> {
    use windows::Win32::Networking::WinSock::{SOCKET, WSAGetLastError, WSAIoctl};
    // _WSAIOR(IOC_VENDOR, 256), afunix.h
    const SIO_AF_UNIX_GETPEERPID: u32 = 0x5800_0100;
    let mut pid = 0u32;
    let mut returned = 0u32;
    let status = unsafe {
        WSAIoctl(
            SOCKET(stream.as_raw_socket() as usize),
            SIO_AF_UNIX_GETPEERPID,
            None,
            0,
            Some(&mut pid as *mut u32 as *mut _),
            size_of::<u32>() as u32,
            &mut returned,
            None,
            None,
        )
    };
    if status != 0 {
        return Err(format!("SIO_AF_UNIX_GETPEERPID: {:?}", unsafe {
            WSAGetLastError()
        }));
    }
    Ok(pid)
}

/// Engine: duplicates `handles` into the peer's process and sends the
/// `Hello` naming the duplicates.
pub fn send_hello(
    stream: &UnixStream,
    hello: &Hello,
    handles: &[OwnedHandle],
) -> Result<(), String> {
    let pid = peer_pid(stream)?;
    let host = unsafe { OpenProcess(PROCESS_DUP_HANDLE, false, pid) }
        .map_err(|e| format!("OpenProcess({pid}): {e}"))?;
    let host = unsafe { OwnedHandle::from_raw_handle(host.0) };
    let mut hello = hello.clone();
    hello.handles = handles
        .iter()
        .map(|handle| {
            let mut duplicate = HANDLE::default();
            unsafe {
                DuplicateHandle(
                    GetCurrentProcess(),
                    HANDLE(handle.as_raw_handle()),
                    HANDLE(host.as_raw_handle()),
                    &mut duplicate,
                    0,
                    false,
                    DUPLICATE_SAME_ACCESS,
                )
            }
            .map_err(|e| format!("DuplicateHandle: {e}"))?;
            Ok(duplicate.0 as u64)
        })
        .collect::<Result<_, String>>()?;
    let body = serde_json::to_vec(&hello).map_err(|e| e.to_string())?;
    write_frame(&mut &*stream, &body).map_err(|e| e.to_string())
}

/// Host: the `Hello` and the handles the engine duplicated into this
/// process, now owned here.
pub fn recv_hello(stream: &UnixStream) -> Result<(Hello, Vec<OwnedHandle>), String> {
    let body = super::read_frame(&mut &*stream).map_err(|e| e.to_string())?;
    let hello: Hello = serde_json::from_slice(&body).map_err(|e| e.to_string())?;
    if hello.handles.len() != hello.slots as usize {
        return Err(format!(
            "{} handles for {} slots",
            hello.handles.len(),
            hello.slots
        ));
    }
    let handles = hello
        .handles
        .iter()
        .map(|&h| unsafe { OwnedHandle::from_raw_handle(h as usize as _) })
        .collect();
    Ok((hello, handles))
}
